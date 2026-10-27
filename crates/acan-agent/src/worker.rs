//! The worker's lifecycle: list, read back, claim, declare, wait for Building, edit, commit, and
//! report. The origin decides every rule. This module sequences the calls, renews the lease, and
//! keeps the worker's own writes behind the origin's build clearance.

use std::collections::HashSet;
use std::fmt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use acan_llm::{ChatRequest, Message, ModelClient, Role, ToolCall, ToolSpec};
use anyhow::{Context, bail};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::mcp::{McpClient, Reply};
use crate::tools::Worktree;

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
/// A `Failed` run that could not release its claim, because the error came before the release
/// or the origin was unreachable, leaves the lease to the origin: the heartbeat stops, so the
/// lease ends after the origin's 60 second lease unless the task has already entered its gate.
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

/// A claimed task and the generation the origin granted.
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

/// The end of the edit loop.
enum Edit {
    Committed(String),
    Stopped(String),
    NotCleared(String),
}

/// What one tool call produced.
enum ToolStep {
    Answer(String),
    Committed(String),
    NotCleared(String),
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
    // The lease only survives while renewals arrive, so the heartbeat runs beside the work and
    // stops whichever way the work ends.
    let renewals = tokio::spawn(heartbeat(mcp.clone(), claim.clone(), config.heartbeat));
    let exit = work(config, model, &mcp, &claim, steps).await;
    renewals.abort();
    exit
}

/// Renews the lease until the origin refuses a renewal or the run aborts it.
async fn heartbeat(mcp: McpClient, claim: Claim, every: Duration) {
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
                return;
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
                files: planned.files,
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
/// with the lease released.
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
            release(mcp, claim, steps).await;
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
    if let Err(stage) = wait_for_build(config, mcp, claim, steps).await? {
        release(mcp, claim, steps).await;
        return Ok(Exit::NotCleared {
            task: claim.task.clone(),
            stage,
        });
    }
    let path = match mcp
        .call("get_worktree", json!({ "task": claim.task }))
        .await?
    {
        Reply::Done(view) => string_field(&view, "path"),
        Reply::Refused(reason) => {
            release(mcp, claim, steps).await;
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
        Edit::Stopped(reason) => {
            release(mcp, claim, steps).await;
            Ok(Exit::NoCommit {
                task: claim.task.clone(),
                reason,
            })
        }
        Edit::NotCleared(stage) => {
            release(mcp, claim, steps).await;
            Ok(Exit::NotCleared {
                task: claim.task.clone(),
                stage,
            })
        }
    }
}

/// Polls until the origin says the task is Building. The error names the stage it stopped in,
/// or the state if the lease has already ended.
async fn wait_for_build(
    config: &WorkerConfig,
    mcp: &McpClient,
    claim: &Claim,
    steps: &mut Vec<String>,
) -> anyhow::Result<Result<(), String>> {
    let deadline = Instant::now() + config.wait;
    let mut last = String::new();
    loop {
        let detail = get_task(mcp, &claim.task).await?;
        let state = string_field(&detail, "state");
        if state != "claimed" {
            return Ok(Err(state));
        }
        let stage = string_field(&detail, "stage");
        if stage == "building" {
            return Ok(Ok(()));
        }
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
            return Ok(Err(stage));
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
            }
        }
    }
    Ok(Edit::Stopped(
        "the step limit ran out before a commit".to_owned(),
    ))
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
            if let Err(stage) = wait_for_build(config, mcp, claim, steps).await? {
                return Ok(ToolStep::NotCleared(stage));
            }
            tools.write_file(&string_arg(&args, "path"), &string_arg(&args, "content"))
        }
        "commit" => {
            if let Err(stage) = wait_for_build(config, mcp, claim, steps).await? {
                return Ok(ToolStep::NotCleared(stage));
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
    let structured = model
        .chat_json::<Plan>(Role::Worker, request)
        .await
        .context("planning the readback failed")?;
    Ok(structured.value)
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
