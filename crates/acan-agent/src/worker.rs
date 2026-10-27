//! The worker's lifecycle: list, read back, claim, declare, wait for Building, edit, commit, and
//! report. The origin decides every rule. This module sequences the calls, renews the lease, and
//! keeps the worker's own writes behind the origin's build clearance and its fence.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use acan_core::{ClashState, Locus, Settlement};
use acan_llm::{ChatRequest, Message, ModelClient, Role, ToolCall, ToolSpec};
use anyhow::{Context, bail};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::grounding::RepoFiles;
use crate::mcp::{McpClient, Reply};
use crate::tools::Worktree;

const PLAN_PROMPT: &str = "You plan one task for an AI coding worker in a shared repository. Reply with JSON: summary, one or two sentences on the change you will make; files, the existing repository files you will edit, each copied exactly from the repository file list; and new_files, the files you will create, only when they do not exist yet and their directory is already in the repository. Do not invent paths. List every file you will write and no other.";

const EDIT_PROMPT: &str = "You are an AI coding worker. Your task is cleared to build, and you may change only the files you declared. Read a file before you write it. Run the tests when you can. When the change is complete, call commit once with a short message, then stop.";

/// How a worker is run. The token is never printed, so `Debug` leaves it out.
#[derive(Clone)]
pub struct WorkerConfig {
    /// The origin's base URL, such as `http://127.0.0.1:7311`.
    pub server: String,
    /// The agent's bearer token.
    pub token: String,
    /// The repository the origin serves. Its tracked files ground each plan, so a plan can only
    /// declare paths the repository holds, or new files in its existing directories.
    pub repo: PathBuf,
    /// The command `run_tests` runs in the worktree. `None` means no tests are configured.
    pub test_command: Option<String>,
    /// The most model calls the edit loop may make before the worker gives up.
    pub max_steps: usize,
    /// How long one wait may last: for work to appear, a verdict, or build clearance.
    pub wait: Duration,
    /// How often the worker renews its lease. Keep it well under the origin's 60 second lease.
    pub heartbeat: Duration,
    /// How often a wait polls the origin.
    pub poll: Duration,
}

impl fmt::Debug for WorkerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerConfig")
            .field("server", &self.server)
            .field("token", &"<redacted>")
            .field("repo", &self.repo)
            .field("test_command", &self.test_command)
            .field("max_steps", &self.max_steps)
            .field("wait", &self.wait)
            .field("heartbeat", &self.heartbeat)
            .field("poll", &self.poll)
            .finish()
    }
}

/// Where a worker run ended.
///
/// Every exit except `Built` and `Lost` releases the claim first. If that release cannot reach
/// the origin, the lease ends on its own once renewals stop, after the origin's 60 second lease.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Exit {
    /// The build was reported. The origin's gate runs next.
    Built {
        /// The task.
        task: String,
        /// The generation the build was reported under.
        generation: u64,
        /// The commit that was reported.
        head: String,
    },
    /// No open task could be claimed before the wait ended.
    NoTask,
    /// The task did not clear to build before the wait ended. The worker released it.
    NotCleared {
        /// The task.
        task: String,
        /// The stage the task was last in.
        stage: String,
    },
    /// The origin stopped renewing the claim, so it is no longer this worker's. The worker
    /// stopped its work at once and did not release a claim the origin had already ended.
    Lost {
        /// The task.
        task: String,
        /// Why the origin refused the renewal, or why the claim is no longer held.
        reason: String,
    },
    /// The edit loop ended without a commit. The worker released the task.
    NoCommit {
        /// The task.
        task: String,
        /// Why the loop ended.
        reason: String,
    },
    /// A Narrow settlement took files the task needs, and the task cannot carry on without
    /// them. The worker released the task.
    Narrowed {
        /// The task.
        task: String,
        /// Why the work cannot go on without the files it gave up.
        reason: String,
    },
    /// A call was refused or a dependency failed.
    Failed {
        /// What failed.
        reason: String,
    },
}

impl fmt::Display for Exit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Built {
                task,
                generation,
                head,
            } => write!(
                f,
                "built {task} at generation {generation}; reported {head}"
            ),
            Self::NoTask => f.write_str("no task could be claimed"),
            Self::NotCleared { task, stage } => {
                write!(f, "{task} did not clear to build; it was {stage}")
            }
            Self::Lost { task, reason } => write!(f, "lost {task}: {reason}"),
            Self::NoCommit { task, reason } => write!(f, "{task} was not committed: {reason}"),
            Self::Narrowed { task, reason } => {
                write!(
                    f,
                    "{task} could not carry on after a narrow settlement: {reason}"
                )
            }
            Self::Failed { reason } => write!(f, "failed: {reason}"),
        }
    }
}

/// One worker run: where it ended, and the step log it wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// Where the run ended.
    pub exit: Exit,
    /// Every step, in order, as the log printed it.
    pub steps: Vec<String>,
}

/// Runs one worker to an exit. It never panics on a refused call or a failed model step; each of
/// those ends the run with an [`Exit`] that says what happened.
pub async fn run<C: ModelClient>(config: &WorkerConfig, model: &C) -> Outcome {
    let mut steps = Vec::new();
    let exit = match drive(config, model, &mut steps).await {
        Ok(exit) => exit,
        Err(err) => Exit::Failed {
            reason: format!("{err:#}"),
        },
    };
    tracing::info!(exit = %exit, "the worker finished");
    Outcome { exit, steps }
}

/// What the model plans for a task. Its files are the footprint the worker declares.
#[derive(Debug, Deserialize, JsonSchema)]
struct Plan {
    /// One or two sentences on the change.
    summary: String,
    /// The existing files the change edits, each copied exactly from the repository file list.
    files: Vec<String>,
    /// The files the change creates. List a file here only when it does not exist yet.
    #[serde(default)]
    new_files: Vec<String>,
}

/// A plan the repository accepted: the summary, the footprint, and the fixes the check made.
#[derive(Debug)]
struct Planned {
    summary: String,
    footprint: Vec<String>,
    fixes: Vec<String>,
}

/// A claimed task and the generation the origin granted. `files` is the footprint as declared,
/// already in the origin's normalized form.
#[derive(Debug, Clone)]
struct Claim {
    task: String,
    generation: u64,
    files: Vec<String>,
}

/// What one attempt at a task produced.
enum Attempt {
    Claimed(Claim),
    Skipped(String),
}

/// A place a Narrow settlement took from the claim, and why. The place is the clash's locus.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Given {
    /// The place the claim gave up.
    at: Locus,
    /// The clash, the task that keeps the place, and the settlement, for the re-plan and the log.
    why: String,
}

/// Where the origin stands on clearing a claim to build.
#[derive(Debug, PartialEq, Eq)]
enum Clearance {
    /// The origin has cleared the claim to build, and this worker still holds that generation.
    Cleared,
    /// The claim is held but not building yet. The text is the stage it is in.
    Waiting(String),
    /// The claim is no longer this worker's at this generation. The text says why.
    Lost(String),
    /// A Narrow settlement sent the claim back to declaring and took these places. The worker
    /// must declare the files it keeps.
    Narrowed(Vec<Given>),
}

/// The end of the edit loop.
enum Edit {
    Committed(String),
    Stopped(String),
    NotCleared(String),
    Lost(String),
    Narrowed(String),
}

/// What one tool call produced.
enum ToolStep {
    Answer(String),
    Committed(String),
    NotCleared(String),
    Lost(String),
    Narrowed(Vec<Given>),
}

/// How many calls one tool got in an edit, and how many of them were answered with a failure.
#[derive(Debug, Default)]
struct Count {
    calls: usize,
    failed: usize,
}

/// The tool calls one edit made, by name.
#[derive(Debug, Default)]
struct Tally(BTreeMap<String, Count>);

impl Tally {
    fn record(&mut self, name: &str, failed: bool) {
        let count = self.0.entry(name.to_owned()).or_default();
        count.calls += 1;
        if failed {
            count.failed += 1;
        }
    }

    /// The counts in name order, such as `read_file 9, run_tests 4 (4 failed)`.
    fn summary(&self) -> String {
        if self.0.is_empty() {
            return "no tool calls".to_owned();
        }
        self.0
            .iter()
            .map(|(name, count)| {
                if count.failed == 0 {
                    format!("{name} {}", count.calls)
                } else {
                    format!("{name} {} ({} failed)", count.calls, count.failed)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

async fn drive<C: ModelClient>(
    config: &WorkerConfig,
    model: &C,
    steps: &mut Vec<String>,
) -> anyhow::Result<Exit> {
    let mcp = McpClient::new(&config.server, config.token.clone());
    mcp.connect().await?;
    log(steps, format!("connected to {} over MCP", config.server));
    let Some(claim) = claim_next(config, model, &mcp, steps).await? else {
        return Ok(Exit::NoTask);
    };
    log(
        steps,
        format!("claimed {} at generation {}", claim.task, claim.generation),
    );
    // The lease only survives while renewals arrive, so the heartbeat runs beside the work. When
    // the origin refuses a renewal, the claim is no longer ours and the work stops at once.
    let mut renewals = tokio::spawn(heartbeat(mcp.clone(), claim.clone(), config.heartbeat));
    let finished = tokio::select! {
        result = work(config, model, &mcp, claim.clone(), steps) => result,
        refused = &mut renewals => {
            let reason = refused.context("the heartbeat task stopped unexpectedly")?;
            return Ok(Exit::Lost { task: claim.task.clone(), reason });
        }
    };
    renewals.abort();
    match finished {
        Ok(exit @ (Exit::Built { .. } | Exit::Lost { .. })) => Ok(exit),
        Ok(exit) => {
            release(&mcp, &claim, &exit.to_string(), steps).await;
            Ok(exit)
        }
        Err(err) => {
            release(&mcp, &claim, &format!("{err:#}"), steps).await;
            Err(err)
        }
    }
}

/// Renews the lease until the origin refuses a renewal, and returns the origin's words for the
/// refusal. A renewal that does not reach the origin is tried again on the next beat.
async fn heartbeat(mcp: McpClient, claim: Claim, every: Duration) -> String {
    loop {
        tokio::time::sleep(every).await;
        let renewal = mcp
            .call(
                "heartbeat_task",
                json!({ "task": claim.task, "generation": claim.generation }),
            )
            .await;
        match renewal {
            Ok(Reply::Done(_)) => {}
            Ok(Reply::Refused(reason)) => {
                tracing::warn!(task = %claim.task, %reason, "the origin refused a heartbeat");
                return reason;
            }
            Err(err) => {
                tracing::warn!(task = %claim.task, error = ?err, "a heartbeat did not reach the origin");
            }
        }
    }
}

/// Tries open tasks in the origin's order until one is claimed or the wait ends. A task that is
/// refused or not matched is not tried again in this run.
async fn claim_next<C: ModelClient>(
    config: &WorkerConfig,
    model: &C,
    mcp: &McpClient,
    steps: &mut Vec<String>,
) -> anyhow::Result<Option<Claim>> {
    let deadline = Instant::now() + config.wait;
    let mut skipped = HashSet::new();
    loop {
        for row in open_tasks(mcp).await? {
            let task = string_field(&row, "task");
            if skipped.contains(&task) {
                continue;
            }
            match try_claim(config, model, mcp, &row, steps).await? {
                Attempt::Claimed(claim) => return Ok(Some(claim)),
                Attempt::Skipped(reason) => {
                    log(steps, format!("skipped {reason}"));
                    skipped.insert(task);
                }
            }
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        tokio::time::sleep(config.poll).await;
    }
}

async fn try_claim<C: ModelClient>(
    config: &WorkerConfig,
    model: &C,
    mcp: &McpClient,
    row: &Value,
    steps: &mut Vec<String>,
) -> anyhow::Result<Attempt> {
    // Every attempt reads back a fresh plan, so the files the claim declares are the files the
    // matched readback describes. A readback left from an earlier run is not reused.
    let task = string_field(row, "task");
    let repo = RepoFiles::load(&config.repo).await?;
    let planned = match plan_work(model, &repo, row, &[], steps).await {
        Ok(planned) => planned,
        Err(reasons) => {
            return Ok(Attempt::Skipped(format!(
                "{task}: plan refused after {PLAN_ATTEMPTS} tries: {reasons}"
            )));
        }
    };
    for fix in &planned.fixes {
        log(steps, format!("plan fix on {task}: {fix}"));
    }
    let posted = mcp
        .call(
            "post_readback",
            json!({
                "task": task,
                "summary": planned.summary,
                "touches": touches(row),
            }),
        )
        .await?;
    if let Reply::Refused(reason) = posted {
        return Ok(Attempt::Skipped(format!(
            "{task}: readback refused: {reason}"
        )));
    }
    log(steps, format!("read back {task}: {}", planned.summary));
    let status = wait_for_verdict(config, mcp, &task).await?;
    if status != "match" {
        return Ok(Attempt::Skipped(format!("{task}: readback is {status}")));
    }
    match mcp.call("claim_task", json!({ "task": task })).await? {
        Reply::Refused(reason) => Ok(Attempt::Skipped(format!("{task}: claim refused: {reason}"))),
        Reply::Done(value) => {
            let generation = value["generation"]
                .as_u64()
                .context("claim_task named no generation")?;
            Ok(Attempt::Claimed(Claim {
                task,
                generation,
                files: planned.footprint,
            }))
        }
    }
}

/// Polls until the readback has a verdict. Returns `pending` when the wait ends first.
async fn wait_for_verdict(
    config: &WorkerConfig,
    mcp: &McpClient,
    task: &str,
) -> anyhow::Result<String> {
    let deadline = Instant::now() + config.wait;
    loop {
        let status = readback_status(&get_task(mcp, task).await?);
        if status != "pending" {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            return Ok(status);
        }
        tokio::time::sleep(config.poll).await;
    }
}

/// Declares, waits for build clearance, and runs the edit loop. Ends with a commit reported, or
/// with an exit the caller settles: a stop that releases the claim, or a loss the origin already
/// made.
async fn work<C: ModelClient>(
    config: &WorkerConfig,
    model: &C,
    mcp: &McpClient,
    mut claim: Claim,
    steps: &mut Vec<String>,
) -> anyhow::Result<Exit> {
    let declared = mcp
        .call(
            "declare_changes",
            json!({
                "task": claim.task,
                "generation": claim.generation,
                "files": claim.files,
            }),
        )
        .await?;
    let view = match declared {
        Reply::Done(view) => view,
        Reply::Refused(reason) => {
            return Ok(Exit::Failed {
                reason: format!("declare_changes refused on {}: {reason}", claim.task),
            });
        }
    };
    let clashes = view["clashes_found"].as_array().map_or(0, Vec::len);
    log(
        steps,
        format!(
            "declared {} file(s) on {}; {clashes} new clash(es)",
            claim.files.len(),
            claim.task
        ),
    );
    loop {
        match wait_for_build(config, mcp, &claim, steps).await? {
            Clearance::Cleared => break,
            Clearance::Waiting(stage) => {
                return Ok(Exit::NotCleared {
                    task: claim.task.clone(),
                    stage,
                });
            }
            Clearance::Lost(reason) => {
                return Ok(Exit::Lost {
                    task: claim.task.clone(),
                    reason,
                });
            }
            Clearance::Narrowed(given) => {
                if let Err(reason) =
                    redeclare(config, model, mcp, &mut claim, &given, steps).await?
                {
                    return Ok(Exit::Narrowed {
                        task: claim.task.clone(),
                        reason,
                    });
                }
            }
        }
    }
    let path = match mcp
        .call("get_worktree", json!({ "task": claim.task }))
        .await?
    {
        Reply::Done(view) => string_field(&view, "path"),
        Reply::Refused(reason) => {
            return Ok(Exit::Failed {
                reason: format!("get_worktree refused on {}: {reason}", claim.task),
            });
        }
    };
    log(steps, format!("worktree for {}: {path}", claim.task));
    let mut tools = Worktree::new(
        PathBuf::from(path),
        &claim.files,
        config.test_command.clone(),
    );
    let brief = format!(
        "Task {}. Declared files: {}.",
        claim.task,
        claim.files.join(", ")
    );
    match edit(config, model, mcp, &mut claim, &mut tools, brief, steps).await? {
        Edit::Committed(head) => report(mcp, &claim, head, steps).await,
        Edit::Stopped(reason) => Ok(Exit::NoCommit {
            task: claim.task.clone(),
            reason,
        }),
        Edit::NotCleared(stage) => Ok(Exit::NotCleared {
            task: claim.task.clone(),
            stage,
        }),
        Edit::Lost(reason) => Ok(Exit::Lost {
            task: claim.task.clone(),
            reason,
        }),
        Edit::Narrowed(reason) => Ok(Exit::Narrowed {
            task: claim.task.clone(),
            reason,
        }),
    }
}

/// Reads the task's own view and says whether this worker may write. The origin's `you_hold` and
/// `generation` are the fence: a write is allowed only while this worker holds the generation it
/// was granted, and the task is building.
fn clearance(detail: &Value, claim: &Claim) -> anyhow::Result<Clearance> {
    let state = string_field(detail, "state");
    let generation = detail["generation"].as_u64();
    let holds = detail["you_hold"].as_bool() == Some(true) && generation == Some(claim.generation);
    if state != "claimed" || !holds {
        let now = match generation {
            Some(generation) => generation.to_string(),
            None => "none".to_owned(),
        };
        return Ok(Clearance::Lost(format!(
            "{} is {state} at generation {now}; this worker holds generation {}",
            claim.task, claim.generation
        )));
    }
    let stage = string_field(detail, "stage");
    if stage == "building" {
        return Ok(Clearance::Cleared);
    }
    // A Narrow settlement sends the claim back to declaring. Its clash names the places it gave up.
    if stage == "declaring" {
        let given = narrowed(detail, claim)?;
        if !given.is_empty() {
            return Ok(Clearance::Narrowed(given));
        }
    }
    Ok(Clearance::Waiting(stage))
}

/// The places a Narrow settlement took from `claim`: every settled clash that names the claim's
/// task as the one that narrows, at a place the claim still declares. A clash at a place that is
/// not a file is kept, so the re-plan can name it as the reason the work cannot go on.
///
/// # Errors
///
/// Returns the reason when a clash's state or place does not read as the origin's types.
fn narrowed(detail: &Value, claim: &Claim) -> anyhow::Result<Vec<Given>> {
    let mut given = Vec::new();
    for clash in detail["clashes"].as_array().into_iter().flatten() {
        let id = clash["clash"].as_str().unwrap_or_default();
        let state: ClashState = serde_json::from_value(clash["state"].clone())
            .with_context(|| format!("clash {id} has a state the worker does not read"))?;
        let ClashState::Settled {
            choice: Settlement::Narrow { task },
            ..
        } = state
        else {
            continue;
        };
        if task.to_string() != claim.task {
            continue;
        }
        let at: Locus = serde_json::from_value(clash["at"].clone())
            .with_context(|| format!("clash {id} has a place the worker does not read"))?;
        if file_of(&at).is_some_and(|file| !claim.files.contains(&file)) {
            continue;
        }
        let other = if clash["a"].as_str() == Some(claim.task.as_str()) {
            clash["b"].as_str()
        } else {
            clash["a"].as_str()
        };
        let why = format!(
            "clash {id} with {} was settled as narrow, so {} gives up its place",
            other.unwrap_or("another task"),
            claim.task
        );
        given.push(Given { at, why });
    }
    Ok(given)
}

/// The file a place lies in, for a file or a symbol. A machine or a belt lies in no file.
fn file_of(at: &Locus) -> Option<String> {
    match at {
        Locus::File { file } | Locus::Symbol { file, .. } => Some(file.to_string()),
        Locus::Machine { .. } | Locus::Belt { .. } => None,
    }
}

/// The distinct files the narrowing took. A place that is not a file cannot be re-planned around,
/// so the error names the clash that took it.
fn given_files(given: &[Given]) -> Result<Vec<String>, String> {
    let mut files: Vec<String> = Vec::new();
    for taken in given {
        let file = file_of(&taken.at).ok_or_else(|| {
            format!(
                "{}: the place is not a file, so the work cannot be re-planned around it",
                taken.why
            )
        })?;
        if !files.contains(&file) {
            files.push(file);
        }
    }
    Ok(files)
}

/// Polls until the origin clears the claim to build, the claim is lost, or the wait ends.
async fn wait_for_build(
    config: &WorkerConfig,
    mcp: &McpClient,
    claim: &Claim,
    steps: &mut Vec<String>,
) -> anyhow::Result<Clearance> {
    let deadline = Instant::now() + config.wait;
    let mut last = String::new();
    loop {
        let detail = get_task(mcp, &claim.task).await?;
        let stage = match clearance(&detail, claim)? {
            Clearance::Waiting(stage) => stage,
            decided => return Ok(decided),
        };
        if stage != last {
            log(
                steps,
                format!(
                    "{} is {stage}; waiting for the origin to clear it to build",
                    claim.task
                ),
            );
            last = stage.clone();
        }
        if Instant::now() >= deadline {
            return Ok(Clearance::Waiting(stage));
        }
        tokio::time::sleep(config.poll).await;
    }
}

/// The tool-calling loop. The model edits through the local tools. A write or a commit waits
/// for build clearance first, so no change lands before the origin allows it.
async fn edit<C: ModelClient>(
    config: &WorkerConfig,
    model: &C,
    mcp: &McpClient,
    claim: &mut Claim,
    tools: &mut Worktree,
    brief: String,
    steps: &mut Vec<String>,
) -> anyhow::Result<Edit> {
    let mut messages = vec![Message::system(EDIT_PROMPT), Message::user(brief)];
    let mut specs = Worktree::specs();
    specs.push(get_task_spec());
    let mut tally = Tally::default();
    for _ in 0..config.max_steps {
        let request = ChatRequest {
            messages: messages.clone(),
            tools: specs.clone(),
            ..ChatRequest::default()
        };
        let response = model
            .chat(Role::Worker, request)
            .await
            .context("the edit step failed")?;
        messages.push(Message::Assistant {
            content: response.content.clone(),
            tool_calls: response.tool_calls.clone(),
        });
        if response.tool_calls.is_empty() {
            let reason = response
                .content
                .unwrap_or_else(|| "the model stopped without a commit".to_owned());
            return Ok(Edit::Stopped(reason));
        }
        for call in response.tool_calls {
            let name = call.function.name.clone();
            match run_tool(config, mcp, claim, tools, &call, steps).await? {
                ToolStep::Answer(text) => {
                    // A failing test run is an answer too, so it counts as a failed call.
                    let failed = text.starts_with("error:") || text.starts_with("fail:");
                    tally.record(&name, failed);
                    messages.push(Message::Tool {
                        tool_call_id: call.id,
                        content: text,
                    });
                }
                ToolStep::Committed(head) => return Ok(Edit::Committed(head)),
                ToolStep::NotCleared(stage) => return Ok(Edit::NotCleared(stage)),
                ToolStep::Lost(reason) => return Ok(Edit::Lost(reason)),
                ToolStep::Narrowed(given) => {
                    // The write or commit that met the narrowing did not run. The task keeps
                    // going on the files it still declares, so the model is told what changed.
                    tally.record(&name, true);
                    match narrow_in_edit(config, model, mcp, claim, tools, &given, steps).await? {
                        Ok(text) => messages.push(Message::Tool {
                            tool_call_id: call.id,
                            content: text,
                        }),
                        Err(reason) => return Ok(Edit::Narrowed(reason)),
                    }
                }
            }
        }
    }
    Ok(Edit::Stopped(format!(
        "the step limit ran out before a commit after {} model calls: {}",
        config.max_steps,
        tally.summary()
    )))
}

/// Gives up the files a narrowing took while the task was building or waiting to build. Restores
/// them, re-plans and declares the files that remain, and returns the answer for the call that met
/// the narrowing. The error is the reason the work cannot go on.
async fn narrow_in_edit<C: ModelClient>(
    config: &WorkerConfig,
    model: &C,
    mcp: &McpClient,
    claim: &mut Claim,
    tools: &mut Worktree,
    given: &[Given],
    steps: &mut Vec<String>,
) -> anyhow::Result<Result<String, String>> {
    let lost = match given_files(given) {
        Ok(lost) => lost,
        Err(reason) => return Ok(Err(reason)),
    };
    if let Err(reason) = tools.restore(&lost).await {
        return Ok(Err(format!(
            "could not restore {} after the narrowing: {reason}",
            lost.join(", ")
        )));
    }
    if let Err(reason) = redeclare(config, model, mcp, claim, given, steps).await? {
        return Ok(Err(reason));
    }
    tools.declare(&claim.files);
    Ok(Ok(format!(
        "The declaration changed. {} is no longer yours and was restored, so your writes to it are refused now. You may write only: {}. The write or commit that met the change did not run; continue with the files you may write.",
        lost.join(", "),
        claim.files.join(", ")
    )))
}

/// Re-plans a claim that a Narrow settlement cut, and declares the files it keeps. The re-plan may
/// not name a file the narrowing took, and it is told which files it lost and why. The error is
/// the reason the work cannot go on: no plan keeps to the files that remain, the plan keeps no file,
/// or the origin refuses the declaration. Every such reason releases the claim.
async fn redeclare<C: ModelClient>(
    config: &WorkerConfig,
    model: &C,
    mcp: &McpClient,
    claim: &mut Claim,
    given: &[Given],
    steps: &mut Vec<String>,
) -> anyhow::Result<Result<(), String>> {
    let lost = match given_files(given) {
        Ok(lost) => lost,
        Err(reason) => return Ok(Err(reason)),
    };
    let row = get_task(mcp, &claim.task).await?;
    let repo = RepoFiles::load(&config.repo).await?;
    let planned = match plan_work(model, &repo, &row, given, steps).await {
        Ok(planned) => planned,
        Err(reasons) => {
            return Ok(Err(format!(
                "no plan without {} was accepted: {reasons}",
                lost.join(", ")
            )));
        }
    };
    for fix in &planned.fixes {
        log(steps, format!("plan fix on {}: {fix}", claim.task));
    }
    if planned.footprint.is_empty() {
        return Ok(Err(format!(
            "the work cannot be done without {}: {}",
            lost.join(", "),
            planned.summary
        )));
    }
    let declared = mcp
        .call(
            "declare_changes",
            json!({
                "task": claim.task,
                "generation": claim.generation,
                "files": planned.footprint,
            }),
        )
        .await?;
    let view = match declared {
        Reply::Done(view) => view,
        Reply::Refused(reason) => {
            return Ok(Err(format!(
                "declare_changes refused on {}: {reason}",
                claim.task
            )));
        }
    };
    let clashes = view["clashes_found"].as_array().map_or(0, Vec::len);
    log(
        steps,
        format!(
            "narrowed {}: gave up {}; declared {} file(s) without them; {clashes} new clash(es)",
            claim.task,
            lost.join(", "),
            planned.footprint.len()
        ),
    );
    claim.files = planned.footprint;
    Ok(Ok(()))
}

/// Waits for build clearance before a write or a commit. The error carries the step that ends
/// the edit: the write may not run.
async fn cleared(
    config: &WorkerConfig,
    mcp: &McpClient,
    claim: &Claim,
    steps: &mut Vec<String>,
) -> anyhow::Result<Result<(), ToolStep>> {
    Ok(match wait_for_build(config, mcp, claim, steps).await? {
        Clearance::Cleared => Ok(()),
        Clearance::Waiting(stage) => Err(ToolStep::NotCleared(stage)),
        Clearance::Lost(reason) => Err(ToolStep::Lost(reason)),
        Clearance::Narrowed(given) => Err(ToolStep::Narrowed(given)),
    })
}

async fn run_tool(
    config: &WorkerConfig,
    mcp: &McpClient,
    claim: &Claim,
    tools: &Worktree,
    call: &ToolCall,
    steps: &mut Vec<String>,
) -> anyhow::Result<ToolStep> {
    let args: Value = match serde_json::from_str(&call.function.arguments) {
        Ok(args) => args,
        Err(err) => {
            return Ok(ToolStep::Answer(format!(
                "error: the arguments are not JSON: {err}"
            )));
        }
    };
    let answer = match call.function.name.as_str() {
        "read_file" => tools.read_file(&string_arg(&args, "path")).await,
        "list_files" => tools.list_files(&string_arg(&args, "path")),
        "run_tests" => tools.run_tests().await,
        "write_file" => {
            if let Err(step) = cleared(config, mcp, claim, steps).await? {
                return Ok(step);
            }
            let path = string_arg(&args, "path");
            let written = tools.write_file(&path, &string_arg(&args, "content"));
            match &written {
                Ok(_) => log(steps, format!("wrote {path} on {}", claim.task)),
                Err(reason) => log(
                    steps,
                    format!("write to {path} refused on {}: {reason}", claim.task),
                ),
            }
            written
        }
        "commit" => {
            if let Err(step) = cleared(config, mcp, claim, steps).await? {
                return Ok(step);
            }
            match tools.commit(&string_arg(&args, "message")).await {
                Ok(head) => {
                    log(steps, format!("committed {head} on {}", claim.task));
                    return Ok(ToolStep::Committed(head));
                }
                Err(reason) => {
                    log(steps, format!("commit refused on {}: {reason}", claim.task));
                    Err(reason)
                }
            }
        }
        "get_task" => get_task(mcp, &claim.task)
            .await
            .map(|detail| detail.to_string())
            .map_err(|err| format!("{err:#}")),
        other => Err(format!("there is no tool named {other}")),
    };
    Ok(ToolStep::Answer(match answer {
        Ok(text) => text,
        Err(reason) => format!("error: {reason}"),
    }))
}

async fn report(
    mcp: &McpClient,
    claim: &Claim,
    head: String,
    steps: &mut Vec<String>,
) -> anyhow::Result<Exit> {
    let reported = mcp
        .call(
            "report_built",
            json!({
                "task": claim.task,
                "generation": claim.generation,
                "head": head,
            }),
        )
        .await?;
    match reported {
        Reply::Done(_) => {
            log(steps, format!("reported {head} on {}", claim.task));
            Ok(Exit::Built {
                task: claim.task.clone(),
                generation: claim.generation,
                head,
            })
        }
        Reply::Refused(reason) => Ok(Exit::Failed {
            reason: format!("report_built refused on {}: {reason}", claim.task),
        }),
    }
}

/// Gives the claim back and logs why. The reason is the exit the run ended with, or the error it
/// failed on, so every release in the log says what caused it.
async fn release(mcp: &McpClient, claim: &Claim, why: &str, steps: &mut Vec<String>) {
    let released = mcp
        .call(
            "release_task",
            json!({ "task": claim.task, "generation": claim.generation }),
        )
        .await;
    match released {
        Ok(Reply::Done(_)) => log(steps, format!("released {}: {why}", claim.task)),
        Ok(Reply::Refused(reason)) => {
            log(
                steps,
                format!("release of {} refused: {reason}", claim.task),
            );
        }
        Err(err) => log(
            steps,
            format!(
                "release of {} did not reach the origin: {err:#}",
                claim.task
            ),
        ),
    }
}

/// How many planning calls a task gets. A live model can miss the requested schema, or name a path
/// the repository does not hold. Each miss is sent back with its reasons, and after the last miss
/// the task is skipped, so a plan that cannot be grounded never reaches the origin.
const PLAN_ATTEMPTS: usize = 3;

/// Plans one task and checks the plan against the repository. A refused plan is sent back with
/// the reasons and the file list. The error is the reasons of the last refusal.
///
/// `given` holds the places a Narrow settlement took from the task. A plan may not name them. A
/// narrowed task whose plan keeps no file at all gets that plan back with its summary, so the
/// caller can release the claim with the reason.
async fn plan_work<C: ModelClient>(
    model: &C,
    repo: &RepoFiles,
    row: &Value,
    given: &[Given],
    steps: &mut Vec<String>,
) -> Result<Planned, String> {
    let task = string_field(row, "task");
    let brief = planning_brief(repo, row, given).await;
    let lost: Vec<String> = given
        .iter()
        .filter_map(|taken| file_of(&taken.at))
        .collect();
    let mut refusal: Vec<String> = Vec::new();
    for attempt in 1..=PLAN_ATTEMPTS {
        let mut messages = vec![Message::system(PLAN_PROMPT), Message::user(brief.clone())];
        if !refusal.is_empty() {
            messages.push(Message::user(format!(
                "Your last plan was refused:\n{}\nReply with a corrected plan. Copy each existing file's path exactly from the repository file list, and list a new file under new_files only when its directory is in that list.",
                refusal.join("\n")
            )));
        }
        let request = ChatRequest {
            messages,
            ..ChatRequest::default()
        };
        refusal = match model.chat_json::<Plan>(Role::Worker, request).await {
            Ok(structured) => {
                let plan = structured.value;
                if !given.is_empty() && plan.files.is_empty() && plan.new_files.is_empty() {
                    return Ok(Planned {
                        summary: plan.summary,
                        footprint: Vec::new(),
                        fixes: Vec::new(),
                    });
                }
                match repo.check(&plan.files, &plan.new_files) {
                    Ok(checked) => {
                        let taken: Vec<String> = checked
                            .footprint
                            .iter()
                            .filter(|file| lost.contains(file))
                            .cloned()
                            .collect();
                        if taken.is_empty() {
                            return Ok(Planned {
                                summary: plan.summary,
                                footprint: checked.footprint,
                                fixes: checked.fixes,
                            });
                        }
                        taken
                            .iter()
                            .map(|file| {
                                format!(
                                    "{file} was given up under a narrow settlement, so it may not be declared again"
                                )
                            })
                            .collect()
                    }
                    Err(reasons) => reasons,
                }
            }
            Err(err) => vec![format!(
                "the planning call failed, or its reply missed the plan schema: {err}"
            )],
        };
        log(
            steps,
            format!(
                "plan refused on {task} (attempt {attempt} of {PLAN_ATTEMPTS}): {}",
                refusal.join("; ")
            ),
        );
    }
    Err(refusal.join("; "))
}

/// The planning message for one task: its title, brief and scope, the repository's tracked files,
/// and the full text of the files its scope machines hold. A narrowed task also gets the places it
/// gave up and why.
async fn planning_brief(repo: &RepoFiles, row: &Value, given: &[Given]) -> String {
    let scope: Vec<String> = row["scope"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|machine| machine.as_str().map(str::to_owned))
        .collect();
    let brief = format!(
        "Task: {}\nBrief: {}\nScope machines: {}\n\nRepository files (tracked; use these paths exactly):\n{}\n\nFull text of the files in the scope machines:\n{}",
        string_field(row, "title"),
        string_field(row, "brief"),
        scope.join(", "),
        repo.listing(),
        repo.scope_text(&scope).await,
    );
    if given.is_empty() {
        return brief;
    }
    let gave_up: Vec<String> = given
        .iter()
        .map(|taken| {
            let place = file_of(&taken.at).unwrap_or_else(|| format!("{:?}", taken.at));
            format!("- {place}: {}", taken.why)
        })
        .collect();
    format!(
        "{brief}\n\nYour declaration was narrowed, so you gave up these places, and you may not declare them again:\n{}\nPlan the change without them. List only the files the change still needs. If the change cannot be done without them, reply with an empty files list and say why in summary.",
        gave_up.join("\n")
    )
}

/// The machines a task is scoped to, as the loci a readback touches.
fn touches(row: &Value) -> Value {
    let loci: Vec<Value> = row["scope"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|machine| json!({ "at": "machine", "machine": machine }))
        .collect();
    Value::Array(loci)
}

async fn open_tasks(mcp: &McpClient) -> anyhow::Result<Vec<Value>> {
    match mcp.call("list_tasks", json!({})).await? {
        Reply::Done(Value::Array(rows)) => Ok(rows
            .into_iter()
            .filter(|row| row["state"] == "open")
            .collect()),
        Reply::Done(other) => bail!("list_tasks answered {other}"),
        Reply::Refused(reason) => bail!("list_tasks was refused: {reason}"),
    }
}

async fn get_task(mcp: &McpClient, task: &str) -> anyhow::Result<Value> {
    match mcp.call("get_task", json!({ "task": task })).await? {
        Reply::Done(detail) => Ok(detail),
        Reply::Refused(reason) => bail!("get_task on {task} was refused: {reason}"),
    }
}

/// The status of this agent's latest readback, or `none` when it has not read back.
fn readback_status(detail: &Value) -> String {
    detail["your_readback"]["status"]
        .as_str()
        .unwrap_or("none")
        .to_owned()
}

fn get_task_spec() -> ToolSpec {
    ToolSpec {
        name: "get_task".to_owned(),
        description: "Read the current state of your task, including its stage and clashes."
            .to_owned(),
        parameters: json!({ "type": "object", "properties": {} }),
    }
}

fn string_field(value: &Value, key: &str) -> String {
    value[key].as_str().unwrap_or_default().to_owned()
}

fn string_arg(args: &Value, key: &str) -> String {
    args[key].as_str().unwrap_or_default().to_owned()
}

fn log(steps: &mut Vec<String>, text: String) {
    tracing::info!("{text}");
    steps.push(text);
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use acan_llm::{Reply, Stub};

    use super::*;

    fn repo() -> RepoFiles {
        RepoFiles::from_paths("unused", vec!["src/a.ts".to_owned()])
    }

    /// The text of every message in a request, so a test can look for a reason in it.
    fn text_of(request: &ChatRequest) -> String {
        serde_json::to_string(&request.messages).unwrap_or_default()
    }

    #[tokio::test]
    async fn a_planning_reply_that_misses_its_schema_is_asked_again() {
        let calls = AtomicUsize::new(0);
        let model = Stub::default()
            .when(
                Role::Worker,
                move |_| calls.fetch_add(1, Ordering::SeqCst) == 0,
                Reply::text("not a plan"),
            )
            .when(
                Role::Worker,
                |_| true,
                Reply::text(r#"{"summary":"write the file","files":["src/a.ts"]}"#),
            );
        let row = json!({ "title": "t", "brief": "b", "scope": ["src"], "task": "t-1" });
        let planned = plan_work(&model, &repo(), &row, &[], &mut Vec::new())
            .await
            .expect("the second reply is a plan");
        assert_eq!(planned.footprint, vec!["src/a.ts".to_owned()]);
    }

    #[tokio::test]
    async fn a_refused_plan_is_sent_back_with_its_reasons_and_the_file_list() {
        // The first reply names a path the repository does not hold. The second call must carry
        // that refusal and the file list, and the corrected reply then passes.
        let model = Stub::default()
            .when(
                Role::Worker,
                |request| !text_of(request).contains("was refused"),
                Reply::text(r#"{"summary":"s","files":["middleware/a.ts"]}"#),
            )
            .when(
                Role::Worker,
                |request| {
                    let text = text_of(request);
                    text.contains("is not a tracked file") && text.contains("- src/a.ts")
                },
                Reply::text(r#"{"summary":"s","files":["src/a.ts"]}"#),
            );
        let row = json!({ "title": "t", "brief": "b", "scope": ["src"], "task": "t-1" });
        let mut steps = Vec::new();
        let planned = plan_work(&model, &repo(), &row, &[], &mut steps)
            .await
            .expect("the corrected plan passes");
        assert_eq!(planned.footprint, vec!["src/a.ts".to_owned()]);
        assert!(
            steps
                .iter()
                .any(|step| step.starts_with("plan refused on t-1 (attempt 1 of 3)")),
            "{steps:?}"
        );
    }

    #[tokio::test]
    async fn a_plan_that_never_passes_is_refused_after_its_tries_with_the_reason() {
        let model = Stub::default().when(
            Role::Worker,
            |_| true,
            Reply::text(r#"{"summary":"s","files":["nope.ts"]}"#),
        );
        let row = json!({ "title": "t", "brief": "b", "scope": ["src"], "task": "t-1" });
        let mut steps = Vec::new();
        let reasons = plan_work(&model, &repo(), &row, &[], &mut steps)
            .await
            .expect_err("no attempt names a tracked file");
        assert!(
            reasons.contains("`nope.ts` is not a tracked file"),
            "{reasons}"
        );
        let refusals = steps
            .iter()
            .filter(|step| step.starts_with("plan refused"))
            .count();
        assert_eq!(refusals, PLAN_ATTEMPTS);
    }

    fn claim() -> Claim {
        Claim {
            task: "t-1".to_owned(),
            generation: 3,
            files: vec!["model/store.ts".to_owned()],
        }
    }

    fn view(state: &str, stage: Value, holder: bool, generation: Value) -> Value {
        json!({ "state": state, "stage": stage, "you_hold": holder, "generation": generation })
    }

    fn decide(detail: &Value, claim: &Claim) -> Clearance {
        clearance(detail, claim).expect("the view reads")
    }

    /// A clash row as `get_task` reports it. The state is the origin's `ClashState` in JSON.
    fn clash_row(id: &str, a: &str, b: &str, at: Value, state: Value) -> Value {
        json!({ "clash": id, "a": a, "b": b, "at": at, "state": state })
    }

    /// A clash that a settlement narrowed onto task `task`, by its number.
    fn narrowed_onto(task: u64) -> Value {
        json!({
            "state": "settled",
            "choice": { "choice": "narrow", "task": task },
            "by": "owner",
        })
    }

    fn two_file_repo() -> RepoFiles {
        RepoFiles::from_paths("unused", vec!["src/a.ts".to_owned(), "src/b.ts".to_owned()])
    }

    /// The place `t-1` gave up in a narrowing: `src/a.ts`.
    fn gave_up_a() -> Vec<Given> {
        vec![Given {
            at: Locus::File {
                file: "src/a.ts".parse().expect("a file path"),
            },
            why: "clash c-1 with t-2 was settled as narrow, so t-1 gives up its place".to_owned(),
        }]
    }

    #[test]
    fn only_the_holder_at_its_generation_may_write_and_only_once_building() {
        let claim = claim();
        assert_eq!(
            decide(&view("claimed", json!("building"), true, json!(3)), &claim),
            Clearance::Cleared
        );
        assert_eq!(
            decide(&view("claimed", json!("declared"), true, json!(3)), &claim),
            Clearance::Waiting("declared".to_owned())
        );
        // A replacement holds a newer generation.
        assert!(matches!(
            decide(&view("claimed", json!("building"), false, json!(4)), &claim),
            Clearance::Lost(_)
        ));
        // The same generation with another holder is still not this worker's.
        assert!(matches!(
            decide(&view("claimed", json!("building"), false, json!(3)), &claim),
            Clearance::Lost(_)
        ));
        // A released task is open again, with no holder.
        assert!(matches!(
            decide(&view("open", Value::Null, false, Value::Null), &claim),
            Clearance::Lost(_)
        ));
    }

    #[test]
    fn a_narrowing_of_this_claim_names_the_place_it_gave_up() {
        let claim = claim();
        let detail = json!({
            "state": "claimed",
            "stage": "declaring",
            "you_hold": true,
            "generation": 3,
            "clashes": [
                clash_row("c-1", "t-1", "t-2", json!({ "at": "file", "file": "model/store.ts" }), narrowed_onto(1)),
                // Narrowed onto the other task, so this claim keeps its place.
                clash_row("c-2", "t-1", "t-2", json!({ "at": "file", "file": "model/store.ts" }), narrowed_onto(2)),
                // A place the claim does not declare is not one it gave up.
                clash_row("c-3", "t-1", "t-2", json!({ "at": "file", "file": "model/other.ts" }), narrowed_onto(1)),
                // An open clash narrows nothing yet.
                clash_row("c-4", "t-1", "t-2", json!({ "at": "file", "file": "model/store.ts" }), json!({ "state": "open" })),
            ],
        });
        let Clearance::Narrowed(given) = clearance(&detail, &claim).expect("the view reads") else {
            panic!("a narrowing of this claim is reported");
        };
        assert_eq!(given.len(), 1, "{given:?}");
        assert_eq!(
            given[0].at,
            Locus::File {
                file: "model/store.ts".parse().expect("a file path"),
            }
        );
        assert!(given[0].why.contains("c-1"), "{}", given[0].why);
    }

    #[test]
    fn a_place_that_is_not_a_file_cannot_be_planned_around() {
        let machine = Given {
            at: Locus::Machine {
                machine: "model".parse().expect("a machine key"),
            },
            why: "clash c-1 with t-2 was settled as narrow, so t-1 gives up its place".to_owned(),
        };
        let reason = given_files(&[machine]).expect_err("a machine is not a file");
        assert!(reason.contains("not a file"), "{reason}");
    }

    #[test]
    fn the_tally_counts_each_tool_by_name_and_its_failures() {
        let mut tally = Tally::default();
        assert_eq!(tally.summary(), "no tool calls");
        tally.record("run_tests", true);
        tally.record("read_file", false);
        tally.record("run_tests", false);
        assert_eq!(tally.summary(), "read_file 1, run_tests 2 (1 failed)");
    }

    #[tokio::test]
    async fn a_replan_that_names_a_given_up_file_is_sent_back_without_it() {
        // The first reply names the file the narrowing took. The second call carries that refusal,
        // and its reply declares the other file.
        let model = Stub::default()
            .when(
                Role::Worker,
                |request| !text_of(request).contains("was given up under a narrow settlement"),
                Reply::text(r#"{"summary":"s","files":["src/a.ts"]}"#),
            )
            .when(
                Role::Worker,
                |_| true,
                Reply::text(r#"{"summary":"s","files":["src/b.ts"]}"#),
            );
        let row = json!({ "title": "t", "brief": "b", "scope": ["src"], "task": "t-1" });
        let mut steps = Vec::new();
        let planned = plan_work(&model, &two_file_repo(), &row, &gave_up_a(), &mut steps)
            .await
            .expect("the second plan passes");
        assert_eq!(planned.footprint, vec!["src/b.ts".to_owned()]);
        assert!(
            steps
                .iter()
                .any(|step| step.starts_with("plan refused on t-1 (attempt 1 of 3)")),
            "{steps:?}"
        );
    }

    #[tokio::test]
    async fn a_replan_that_keeps_no_file_returns_its_summary_for_the_release() {
        let model = Stub::default().when(
            Role::Worker,
            |_| true,
            Reply::text(r#"{"summary":"the loader needs src/a.ts","files":[]}"#),
        );
        let row = json!({ "title": "t", "brief": "b", "scope": ["src"], "task": "t-1" });
        let planned = plan_work(
            &model,
            &two_file_repo(),
            &row,
            &gave_up_a(),
            &mut Vec::new(),
        )
        .await
        .expect("an empty re-plan is returned, not refused");
        assert!(planned.footprint.is_empty(), "{:?}", planned.footprint);
        assert_eq!(planned.summary, "the loader needs src/a.ts");
    }
}
