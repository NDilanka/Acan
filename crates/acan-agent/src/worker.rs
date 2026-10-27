//! The worker's lifecycle: list, read back, claim, declare, wait for Building, edit, commit, and
//! report. The origin decides every rule. This module sequences the calls, renews the lease, and
//! keeps the worker's own writes behind the origin's build clearance and its fence.

use std::collections::{BTreeSet, HashSet};
use std::fmt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use acan_llm::{ChatRequest, Message, ModelClient, Role, ToolCall, ToolSpec};
use anyhow::{Context, bail};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::mcp::{McpClient, Reply};
use crate::tools::{Worktree, normalize_path};

const PLAN_PROMPT: &str = "You plan one task for an AI coding worker in a shared repository. Reply with JSON: summary, one or two sentences on the change you will make, and files, the repository-relative paths you will write. List every file you will write and no other.";

const EDIT_PROMPT: &str = "You are an AI coding worker. Your task is cleared to build, and you may change only the files you declared. Read a file before you write it. Run the tests when you can. When the change is complete, call commit once with a short message, then stop.";

/// How a worker is run. The token is never printed, so `Debug` leaves it out.
#[derive(Clone)]
pub struct WorkerConfig {
    /// The origin's base URL, such as `http://127.0.0.1:7311`.
    pub server: String,
    /// The agent's bearer token.
    pub token: String,
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
    /// The repository-relative paths the change writes.
    files: Vec<String>,
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

/// Where the origin stands on clearing a claim to build.
#[derive(Debug, PartialEq, Eq)]
enum Clearance {
    /// The origin has cleared the claim to build, and this worker still holds that generation.
    Cleared,
    /// The claim is held but not building yet. The text is the stage it is in.
    Waiting(String),
    /// The claim is no longer this worker's at this generation. The text says why.
    Lost(String),
}

/// The end of the edit loop.
enum Edit {
    Committed(String),
    Stopped(String),
    NotCleared(String),
    Lost(String),
}

/// What one tool call produced.
enum ToolStep {
    Answer(String),
    Committed(String),
    NotCleared(String),
    Lost(String),
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
        result = work(config, model, &mcp, &claim, steps) => result,
        refused = &mut renewals => {
            let reason = refused.context("the heartbeat task stopped unexpectedly")?;
            return Ok(Exit::Lost { task: claim.task.clone(), reason });
        }
    };
    renewals.abort();
    match finished {
        Ok(exit @ (Exit::Built { .. } | Exit::Lost { .. })) => Ok(exit),
        Ok(exit) => {
            release(&mcp, &claim, steps).await;
            Ok(exit)
        }
        Err(err) => {
            release(&mcp, &claim, steps).await;
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
    let planned = plan_work(model, row).await?;
    let files = match footprint(&planned.files) {
        Ok(files) => files,
        Err(reason) => {
            return Ok(Attempt::Skipped(format!("{task}: plan refused: {reason}")));
        }
    };
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
                files,
            }))
        }
    }
}

/// The planned footprint in the origin's form. Aliases of one path merge, so the origin sees each
/// file once. A plan with no file, or with a path outside the worktree, is refused before it is
/// read back.
fn footprint(files: &[String]) -> Result<Vec<String>, String> {
    let mut normalized = BTreeSet::new();
    for file in files {
        normalized.insert(normalize_path(file)?);
    }
    if normalized.is_empty() {
        return Err("the plan names no file".to_owned());
    }
    Ok(normalized.into_iter().collect())
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
    claim: &Claim,
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
    match wait_for_build(config, mcp, claim, steps).await? {
        Clearance::Cleared => {}
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
    match edit(config, model, mcp, claim, &mut tools, brief, steps).await? {
        Edit::Committed(head) => report(mcp, claim, head, steps).await,
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
    }
}

/// Reads the task's own view and says whether this worker may write. The origin's `you_hold` and
/// `generation` are the fence: a write is allowed only while this worker holds the generation it
/// was granted, and the task is building.
fn clearance(detail: &Value, claim: &Claim) -> Clearance {
    let state = string_field(detail, "state");
    let generation = detail["generation"].as_u64();
    let holds = detail["you_hold"].as_bool() == Some(true) && generation == Some(claim.generation);
    if state != "claimed" || !holds {
        let now = match generation {
            Some(generation) => generation.to_string(),
            None => "none".to_owned(),
        };
        return Clearance::Lost(format!(
            "{} is {state} at generation {now}; this worker holds generation {}",
            claim.task, claim.generation
        ));
    }
    let stage = string_field(detail, "stage");
    if stage == "building" {
        Clearance::Cleared
    } else {
        Clearance::Waiting(stage)
    }
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
        let stage = match clearance(&detail, claim) {
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
    claim: &Claim,
    tools: &mut Worktree,
    brief: String,
    steps: &mut Vec<String>,
) -> anyhow::Result<Edit> {
    let mut messages = vec![Message::system(EDIT_PROMPT), Message::user(brief)];
    let mut specs = Worktree::specs();
    specs.push(get_task_spec());
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
            match run_tool(config, mcp, claim, tools, &call, steps).await? {
                ToolStep::Answer(text) => messages.push(Message::Tool {
                    tool_call_id: call.id,
                    content: text,
                }),
                ToolStep::Committed(head) => return Ok(Edit::Committed(head)),
                ToolStep::NotCleared(stage) => return Ok(Edit::NotCleared(stage)),
                ToolStep::Lost(reason) => return Ok(Edit::Lost(reason)),
            }
        }
    }
    Ok(Edit::Stopped(
        "the step limit ran out before a commit".to_owned(),
    ))
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
            if written.is_ok() {
                log(steps, format!("wrote {path} on {}", claim.task));
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
                Err(reason) => Err(reason),
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

async fn release(mcp: &McpClient, claim: &Claim, steps: &mut Vec<String>) {
    let released = mcp
        .call(
            "release_task",
            json!({ "task": claim.task, "generation": claim.generation }),
        )
        .await;
    match released {
        Ok(Reply::Done(_)) => log(steps, format!("released {}", claim.task)),
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

/// How many times a planning call is tried. A live model can return a reply that misses the
/// requested schema, and one miss should not end the worker's run before it declares anything.
const PLAN_ATTEMPTS: usize = 3;

async fn plan_work<C: ModelClient>(model: &C, row: &Value) -> anyhow::Result<Plan> {
    let request = ChatRequest {
        messages: vec![
            Message::system(PLAN_PROMPT),
            Message::user(
                json!({
                    "title": row["title"],
                    "brief": row["brief"],
                    "scope": row["scope"],
                })
                .to_string(),
            ),
        ],
        ..ChatRequest::default()
    };
    let mut last_error = None;
    for attempt in 1..=PLAN_ATTEMPTS {
        match model.chat_json::<Plan>(Role::Worker, request.clone()).await {
            Ok(structured) => return Ok(structured.value),
            Err(err) => {
                tracing::warn!(attempt, error = %err, "a planning call failed; trying again");
                last_error = Some(err);
            }
        }
    }
    let err = last_error.expect("PLAN_ATTEMPTS is at least one");
    Err(err).context("planning the readback failed")
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
        let row = json!({ "title": "t", "brief": "b", "scope": ["src"] });
        let plan = plan_work(&model, &row)
            .await
            .expect("the second reply is a plan");
        assert_eq!(plan.files, vec!["src/a.ts".to_owned()]);
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

    #[test]
    fn only_the_holder_at_its_generation_may_write_and_only_once_building() {
        let claim = claim();
        assert_eq!(
            clearance(&view("claimed", json!("building"), true, json!(3)), &claim),
            Clearance::Cleared
        );
        assert_eq!(
            clearance(&view("claimed", json!("declared"), true, json!(3)), &claim),
            Clearance::Waiting("declared".to_owned())
        );
        // A replacement holds a newer generation.
        assert!(matches!(
            clearance(&view("claimed", json!("building"), false, json!(4)), &claim),
            Clearance::Lost(_)
        ));
        // The same generation with another holder is still not this worker's.
        assert!(matches!(
            clearance(&view("claimed", json!("building"), false, json!(3)), &claim),
            Clearance::Lost(_)
        ));
        // A released task is open again, with no holder.
        assert!(matches!(
            clearance(&view("open", Value::Null, false, Value::Null), &claim),
            Clearance::Lost(_)
        ));
    }

    #[test]
    fn a_plan_merges_aliases_and_refuses_paths_outside_the_worktree() {
        assert_eq!(
            footprint(&[
                "./model/store.ts".to_owned(),
                "model/store.ts".to_owned(),
                "model//util.ts".to_owned(),
            ]),
            Ok(vec![
                "model/store.ts".to_owned(),
                "model/util.ts".to_owned()
            ])
        );
        assert!(footprint(&["../escape.ts".to_owned()]).is_err());
        assert!(footprint(&["/etc/passwd".to_owned()]).is_err());
        assert!(footprint(&[]).is_err());
    }
}
