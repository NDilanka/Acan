//! Runs the origin's derived jobs and records each result as a `SystemCmd`.
//!
//! The loop starts each derived job once per key. A job fails when its work errors (a model
//! call, a reply that cannot become a command, or a git step) or when the origin refuses its
//! result. A failed job is retried with backoff up to [`MAX_JOB_ATTEMPTS`]. When the last
//! attempt fails on git, a land or a witness run reports that failure as its own result, so
//! the queue or the gate moves on. Other failed jobs wait until their key leaves the derived
//! set and returns, so a broken model can neither spin the loop nor burn provider credits
//! without user activity. A refused result is never retried: the origin decided it against the
//! world the job read, and a changed world derives a new key. Accounting-only batches never wake
//! the loop; the writer guarantees that.
//!
//! The loop also keeps each task's worktree in step with its claim. A claimed task gets a
//! worktree as soon as the loop sees the claim. A task that leaves its claim loses its worktree
//! and branch, so a reclaim starts from the current protected head. That step is not a job,
//! because the core derives no key for it.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use tokio::time::Instant;

use acan_core::{
    Check, ClashId, Command, CommitId, GateRunId, Job, OrderId, OrderRev, ReadbackId, Rejection,
    Step, SystemCmd, TaskId, TaskState, WitnessId, WitnessKind, WitnessOutcome, World,
};
use acan_llm::{CallRecord, LlmError, ModelClient, Role};
use tokio::task::{self, JoinHandle, JoinSet};

use crate::git::LandFailure;
use crate::host::{Host, SubmitError};
use crate::model::{self, ReplyError};

/// Total attempts one job key gets before the loop stops retrying it. Chosen so a broken model
/// shows up in seconds during a demo while a live provider is called at most this many times
/// for the same stuck job.
const MAX_JOB_ATTEMPTS: u32 = 5;

/// The delay before the first retry of a failed job. Each later retry waits twice as long.
const RETRY_BACKOFF: Duration = Duration::from_millis(20);

/// Starts the job loop for `host`.
pub fn spawn<C: ModelClient + 'static>(host: Host<C>) -> JoinHandle<()> {
    tokio::spawn(run(host))
}

#[derive(Debug, thiserror::Error)]
enum JobError {
    #[error(transparent)]
    Model(#[from] LlmError),
    #[error(transparent)]
    Reply(#[from] ReplyError),
    #[error(transparent)]
    Git(#[from] acan_git::Error),
    #[error("the origin refused the result: {0}")]
    Rejected(#[source] Rejection),
    #[error("recording the result failed: {0}")]
    Store(#[source] anyhow::Error),
}

impl From<SubmitError> for JobError {
    fn from(err: SubmitError) -> Self {
        match err {
            SubmitError::Rejected(rejection) => Self::Rejected(rejection),
            SubmitError::Internal(err) => Self::Store(err),
        }
    }
}

/// How often a job key failed, and when it may run again.
struct Retry {
    attempts: u32,
    not_before: Instant,
}

/// What a task worktree step does.
#[derive(Debug, Clone, Copy)]
enum Change {
    Create,
    Remove,
}

/// The worktrees of the tasks, as the loop last left them, and the steps still running.
#[derive(Default)]
struct WorktreeSync {
    /// Tasks whose worktree exists on disk.
    present: HashSet<TaskId>,
    /// Creations and removals in flight. A task waits for its own step before its next one.
    ops: JoinSet<(TaskId, Change, Result<(), acan_git::Error>)>,
    running: HashMap<task::Id, TaskId>,
}

async fn run<C: ModelClient + 'static>(host: Host<C>) {
    // The sets are owned by this loop, so aborting the loop aborts every task it started. A job
    // must never write to the log after its server has stopped.
    let mut tasks: JoinSet<(Job, Result<(), JobError>)> = JoinSet::new();
    let mut in_flight: HashMap<task::Id, Job> = HashMap::new();
    let mut retries: HashMap<Job, Retry> = HashMap::new();
    let mut worktrees = WorktreeSync::default();
    if let Some(workspace) = host.workspace() {
        // Worktrees left by an earlier run belong to tasks that may no longer hold a claim.
        match workspace.worktree_tasks().await {
            Ok(present) => worktrees.present.extend(present),
            Err(err) => tracing::warn!(
                error = %err,
                "listing the task worktrees failed; leftovers from an earlier run stay"
            ),
        }
    }
    loop {
        let next_retry = retries
            .iter()
            .filter(|(job, retry)| {
                retry.attempts < MAX_JOB_ATTEMPTS
                    && !in_flight.values().any(|running| running == *job)
            })
            .map(|(_, retry)| retry.not_before)
            .min();
        tokio::select! {
            () = host.jobs_woken() => {
                start_derived(&host, &mut tasks, &mut in_flight, &mut retries);
                reconcile(&host, &mut worktrees);
            }
            Some(finished) = tasks.join_next() => match finished {
                Ok((job, Ok(()))) => {
                    in_flight.retain(|_, running| *running != job);
                    retries.remove(&job);
                }
                Ok((job, Err(err))) => {
                    in_flight.retain(|_, running| *running != job);
                    record_failure(&mut retries, job, &err);
                }
                Err(err) => {
                    tracing::error!(error = %err, "a job task failed");
                    in_flight.remove(&err.id());
                }
            },
            Some(finished) = worktrees.ops.join_next() => match finished {
                Ok((task, change, Ok(()))) => {
                    worktrees.running.retain(|_, running| *running != task);
                    match change {
                        Change::Create => {
                            worktrees.present.insert(task);
                        }
                        Change::Remove => {
                            worktrees.present.remove(&task);
                        }
                    }
                    reconcile(&host, &mut worktrees);
                }
                Ok((task, change, Err(err))) => {
                    worktrees.running.retain(|_, running| *running != task);
                    tracing::warn!(
                        %task,
                        ?change,
                        error = %err,
                        "a task worktree step failed; the next change retries it"
                    );
                }
                // The task is lost with the panic, so its step is not recorded as done. The
                // next change finds the worktree state unchanged and tries again.
                Err(err) => {
                    worktrees.running.remove(&err.id());
                    tracing::error!(error = %err, "a task worktree job failed");
                }
            },
            () = wait_until(next_retry) => {
                start_derived(&host, &mut tasks, &mut in_flight, &mut retries);
            }
        }
    }
}

/// Waits for the next retry time, or forever when nothing is waiting to retry.
async fn wait_until(next_retry: Option<Instant>) {
    match next_retry {
        Some(at) => tokio::time::sleep_until(at).await,
        None => std::future::pending().await,
    }
}

fn record_failure(retries: &mut HashMap<Job, Retry>, job: Job, err: &JobError) {
    if matches!(err, JobError::Rejected(_)) {
        // Running the job again would repeat the same decision. A witness run is too costly to
        // repeat for nothing, so the key waits until the world moves it out of the derived set.
        tracing::warn!(?job, error = %err, "the origin refused a job result; not retrying it");
        retries.insert(
            job,
            Retry {
                attempts: MAX_JOB_ATTEMPTS,
                not_before: Instant::now(),
            },
        );
        return;
    }
    let attempts = retries.get(&job).map_or(0, |retry| retry.attempts) + 1;
    if attempts >= MAX_JOB_ATTEMPTS {
        tracing::error!(?job, attempts, error = %err, "job kept failing; leaving it until it changes");
    } else {
        tracing::warn!(?job, attempt = attempts, error = %err, "job failed; retrying with backoff");
    }
    retries.insert(
        job,
        Retry {
            attempts,
            not_before: Instant::now() + RETRY_BACKOFF * 2u32.pow(attempts.saturating_sub(1)),
        },
    );
}

fn start_derived<C: ModelClient + 'static>(
    host: &Host<C>,
    tasks: &mut JoinSet<(Job, Result<(), JobError>)>,
    in_flight: &mut HashMap<task::Id, Job>,
    retries: &mut HashMap<Job, Retry>,
) {
    let jobs = host.read(World::jobs);
    // A key that left the derived set and came back is a fresh job and earns fresh attempts.
    retries.retain(|job, _| jobs.contains(job));
    let now = Instant::now();
    for job in jobs {
        if in_flight.values().any(|running| *running == job) {
            continue;
        }
        let attempt = match retries.get(&job) {
            Some(retry) if retry.attempts >= MAX_JOB_ATTEMPTS => continue,
            Some(retry) if retry.not_before > now => continue,
            Some(retry) => retry.attempts + 1,
            None => 1,
        };
        let id = tasks
            .spawn(run_one(host.clone(), job.clone(), attempt))
            .id();
        in_flight.insert(id, job);
    }
}

/// Brings each task's worktree in line with its claim: created while the task is claimed,
/// removed once it is not. A task with a step in flight waits for that step, which reconciles
/// again when it finishes.
fn reconcile<C: ModelClient + 'static>(host: &Host<C>, worktrees: &mut WorktreeSync) {
    let Some(workspace) = host.workspace() else {
        return;
    };
    let claimed = host.read(claimed_tasks);
    let create: Vec<TaskId> = claimed.difference(&worktrees.present).copied().collect();
    let remove: Vec<TaskId> = worktrees.present.difference(&claimed).copied().collect();
    for task in create {
        if is_running(worktrees, task) {
            continue;
        }
        let workspace = Arc::clone(&workspace);
        let id = worktrees
            .ops
            .spawn(async move {
                let result = workspace.worktree(task).await.map(drop);
                (task, Change::Create, result)
            })
            .id();
        worktrees.running.insert(id, task);
    }
    for task in remove {
        if is_running(worktrees, task) {
            continue;
        }
        let workspace = Arc::clone(&workspace);
        let id = worktrees
            .ops
            .spawn(async move {
                let result = workspace.reset(task).await;
                (task, Change::Remove, result)
            })
            .id();
        worktrees.running.insert(id, task);
    }
}

fn is_running(worktrees: &WorktreeSync, task: TaskId) -> bool {
    worktrees.running.values().any(|running| *running == task)
}

fn claimed_tasks(world: &World) -> HashSet<TaskId> {
    world
        .orders()
        .flat_map(move |order| world.tasks_of(order.id))
        .filter(|task| matches!(task.state, TaskState::Claimed { .. }))
        .map(|task| task.id)
        .collect()
}

async fn run_one<C: ModelClient>(
    host: Host<C>,
    job: Job,
    attempt: u32,
) -> (Job, Result<(), JobError>) {
    let result = match execute(&host, &job).await {
        Ok(Some(command)) => host
            .submit(Command::System(command))
            .await
            .map(drop)
            .map_err(JobError::from),
        Ok(None) => Ok(()),
        Err(err) => {
            if attempt >= MAX_JOB_ATTEMPTS {
                report_exhausted(&host, &job, &err).await;
            }
            Err(err)
        }
    };
    if let Err(err) = &result {
        tracing::debug!(?job, attempt, error = %err, "job attempt failed");
    }
    (job, result)
}

/// Reports a git failure that used up every attempt as the job's own result. The origin then
/// moves the land queue or the gate on, instead of waiting for a change that may never come.
async fn report_exhausted<C: ModelClient>(host: &Host<C>, job: &Job, err: &JobError) {
    if !matches!(err, JobError::Git(_)) {
        return;
    }
    let detail = format!("git failed on every attempt: {err}");
    let command = match job {
        Job::Land { task, head } => SystemCmd::LandResult {
            task: *task,
            head: head.clone(),
            result: Err(detail),
        },
        Job::RunWitness { run, witness } => SystemCmd::WitnessResult {
            run: *run,
            witness: *witness,
            outcome: WitnessOutcome::Error { detail },
        },
        _ => return,
    };
    if let Err(submit_err) = host.submit(Command::System(command)).await {
        tracing::error!(
            ?job,
            error = %submit_err,
            "reporting the failed job to the origin failed"
        );
    }
}

async fn execute<C: ModelClient>(host: &Host<C>, job: &Job) -> Result<Option<SystemCmd>, JobError> {
    match job {
        Job::Split { order, rev } => split(host, job, *order, *rev).await,
        Job::Review {
            order,
            rev,
            readbacks,
        } => review(host, job, *order, *rev, readbacks).await,
        Job::Explain { clash } => explain(host, job, *clash).await,
        Job::RunWitness { run, witness } => run_witness(host, *run, *witness).await,
        Job::Land { task, head } => land(host, *task, head).await,
    }
}

async fn split<C: ModelClient>(
    host: &Host<C>,
    job: &Job,
    order: OrderId,
    rev: OrderRev,
) -> Result<Option<SystemCmd>, JobError> {
    let Some(request) = host.read(|world| model::split_request(world, order)) else {
        return Ok(None);
    };
    let reply = host
        .client()
        .chat_json::<model::SplitReply>(Role::Foreman, request)
        .await?;
    record_usage(host, job, &reply.call, Step::Split).await;
    Ok(Some(model::split_result(order, rev, reply.value)?))
}

async fn review<C: ModelClient>(
    host: &Host<C>,
    job: &Job,
    order: OrderId,
    rev: OrderRev,
    readbacks: &BTreeSet<ReadbackId>,
) -> Result<Option<SystemCmd>, JobError> {
    let prepared = host.read(|world| {
        let request = model::review_request(world, order, readbacks)?;
        let machine = world.order(order)?.machine.clone();
        Some((request, machine))
    });
    let Some((request, machine)) = prepared else {
        return Ok(None);
    };
    let reply = host
        .client()
        .chat_json::<model::ReviewReply>(Role::Foreman, request)
        .await?;
    record_usage(host, job, &reply.call, Step::Review).await;
    // Another review job may have judged some of these readbacks while the model ran. The origin
    // refuses a batch that names any readback no longer awaiting review, so drop those here.
    let awaiting = host.read(|world| awaiting_review(world, order, rev));
    let allowed: BTreeSet<ReadbackId> = readbacks.intersection(&awaiting).copied().collect();
    if allowed.is_empty() {
        return Ok(None);
    }
    Ok(Some(model::review_result(
        order,
        &machine,
        rev,
        &allowed,
        reply.value,
    )?))
}

/// The readbacks the current review job for `order` at `rev` still awaits, or none when that
/// job is no longer derived.
fn awaiting_review(world: &World, order: OrderId, rev: OrderRev) -> BTreeSet<ReadbackId> {
    world
        .jobs()
        .into_iter()
        .find_map(|job| match job {
            Job::Review {
                order: derived,
                rev: derived_rev,
                readbacks,
            } if derived == order && derived_rev == rev => Some(readbacks),
            _ => None,
        })
        .unwrap_or_default()
}

async fn explain<C: ModelClient>(
    host: &Host<C>,
    job: &Job,
    clash: ClashId,
) -> Result<Option<SystemCmd>, JobError> {
    let Some(request) = host.read(|world| model::explain_request(world, clash)) else {
        return Ok(None);
    };
    let reply = host
        .client()
        .chat_json::<model::ExplainReply>(Role::Fast, request)
        .await?;
    record_usage(host, job, &reply.call, Step::Explain).await;
    Ok(Some(SystemCmd::ClashExplained {
        clash,
        explanation: model::explanation(reply.value)?,
    }))
}

/// Runs one frozen witness against the commit its gate tests, and reports what it found.
async fn run_witness<C: ModelClient>(
    host: &Host<C>,
    run: GateRunId,
    witness: WitnessId,
) -> Result<Option<SystemCmd>, JobError> {
    let Some((task, head, check)) = host.read(|world| {
        let gate = world.gate(run)?;
        let WitnessKind::Check { check } = &world.witness(witness)?.kind else {
            return None;
        };
        Some((gate.task, gate.head.clone(), check.clone()))
    }) else {
        return Ok(None);
    };
    let outcome = check_at(host, task, &head, &check).await?;
    Ok(Some(SystemCmd::WitnessResult {
        run,
        witness,
        outcome,
    }))
}

/// Runs `check` in the task's worktree, once that worktree sits at the gated commit. A worktree
/// at another commit is an error outcome, which the gate counts as a failure. A git failure is an
/// error of the job and follows the retry path.
async fn check_at<C: ModelClient>(
    host: &Host<C>,
    task: TaskId,
    head: &CommitId,
    check: &Check,
) -> Result<WitnessOutcome, JobError> {
    let Some(workspace) = host.workspace() else {
        return Ok(WitnessOutcome::Error {
            detail: "this server has no git repository to run witnesses in".to_owned(),
        });
    };
    // Two runs of one task would share its worktree directory, so they take turns. The task lock
    // comes before the global slot, so a run waiting for its task never holds a slot.
    let _task = workspace.lock_task(task).await;
    let _slot = host
        .witness_slots()
        .acquire_owned()
        .await
        .expect("the witness semaphore is never closed");
    let worktree = workspace.worktree(task).await?;
    let built = worktree.head().await?;
    if built != *head {
        return Ok(WitnessOutcome::Error {
            detail: format!("the task branch is at {built}, not the gated commit {head}"),
        });
    }
    Ok(acan_witness::run(check, worktree.path(), &host.witness_limits()).await)
}

/// Merges the gated commit at the front of the land queue into the protected branch. The core
/// derives a `Land` only for that front, so merges happen one at a time, in queue order.
async fn land<C: ModelClient>(
    host: &Host<C>,
    task: TaskId,
    head: &CommitId,
) -> Result<Option<SystemCmd>, JobError> {
    let result = match host.workspace() {
        Some(workspace) => match workspace.land(task, head).await {
            Ok(commit) => Ok(commit),
            Err(LandFailure::Refused(reason)) => Err(reason),
            Err(LandFailure::Git(err)) => return Err(JobError::Git(err)),
        },
        None => Err("this server has no git repository to land in".to_owned()),
    };
    Ok(Some(SystemCmd::LandResult {
        task,
        head: head.clone(),
        result,
    }))
}

async fn record_usage<C: ModelClient>(host: &Host<C>, job: &Job, call: &CallRecord, step: Step) {
    let command = SystemCmd::ModelUsed {
        model: call.model.clone(),
        step,
        input_tokens: call.usage.prompt_tokens,
        output_tokens: call.usage.completion_tokens,
    };
    // Metering is best-effort: a lost usage event must not retry the model call it paid for.
    if let Err(err) = host.submit(Command::System(command)).await {
        tracing::warn!(?job, error = %err, "recording model usage failed");
    }
}
