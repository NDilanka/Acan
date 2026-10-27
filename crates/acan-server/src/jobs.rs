//! Runs the origin's derived jobs and records each result as a `SystemCmd`.
//!
//! The loop starts each derived job once per key. A job fails when its work errors (a model
//! call, a reply that cannot become a command, or a git step) or when the origin refuses its
//! result. A failed job is retried with backoff up to [`MAX_JOB_ATTEMPTS`], then left alone until
//! its key leaves the derived set and returns, so a broken model or repository can neither spin
//! the loop nor burn provider credits without user activity. A refused result is never retried:
//! the origin decided it against the world the job read, and a changed world derives a new key.
//! A refused result whose job is no longer derived is simply dropped, because its key is gone.
//! Accounting-only batches never wake the loop; the writer guarantees that.
//!
//! The loop also creates the worktree of each claimed task as soon as its lease is granted. That
//! step is not a job, because the core derives no key for it. A failed creation is retried on the
//! next change, and the agent route creates the worktree on demand as well.

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

async fn run<C: ModelClient + 'static>(host: Host<C>) {
    // The sets are owned by this loop, so aborting the loop aborts every task it started. A job
    // must never write to the log after its server has stopped.
    let mut tasks: JoinSet<(Job, Result<(), JobError>)> = JoinSet::new();
    let mut in_flight: HashMap<task::Id, Job> = HashMap::new();
    let mut retries: HashMap<Job, Retry> = HashMap::new();
    let mut worktrees: JoinSet<(TaskId, Result<(), acan_git::Error>)> = JoinSet::new();
    let mut ensured: HashSet<TaskId> = HashSet::new();
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
                ensure_worktrees(&host, &mut worktrees, &mut ensured);
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
                    tracing::error!(error = %err, "a model job task failed");
                    in_flight.remove(&err.id());
                }
            },
            Some(finished) = worktrees.join_next() => match finished {
                Ok((task, Ok(()))) => tracing::debug!(%task, "the task worktree is ready"),
                Ok((task, Err(err))) => {
                    ensured.remove(&task);
                    tracing::warn!(
                        %task,
                        error = %err,
                        "the task worktree failed; the next change retries it"
                    );
                }
                // The task id is lost with the panic, so the task stays marked as ensured. The
                // agent route still creates its worktree on demand.
                Err(err) => tracing::error!(error = %err, "a task worktree job failed"),
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
        tracing::error!(?job, attempts, error = %err, "model job kept failing; leaving it until it changes");
    } else {
        tracing::warn!(?job, attempt = attempts, error = %err, "model job failed; retrying with backoff");
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
        match retries.get(&job) {
            Some(retry) if retry.attempts >= MAX_JOB_ATTEMPTS => continue,
            Some(retry) if retry.not_before > now => continue,
            _ => {}
        }
        let id = tasks.spawn(run_one(host.clone(), job.clone())).id();
        in_flight.insert(id, job);
    }
}

/// Creates the worktree of every claimed task that has none yet. Each creation runs as its own
/// task, so a slow git call delays no job.
fn ensure_worktrees<C: ModelClient>(
    host: &Host<C>,
    worktrees: &mut JoinSet<(TaskId, Result<(), acan_git::Error>)>,
    ensured: &mut HashSet<TaskId>,
) {
    let Some(workspace) = host.workspace() else {
        return;
    };
    for task in host.read(claimed_tasks) {
        if !ensured.insert(task) {
            continue;
        }
        let workspace = Arc::clone(&workspace);
        worktrees.spawn(async move {
            let created = workspace.worktree(task).await.map(drop);
            (task, created)
        });
    }
}

fn claimed_tasks(world: &World) -> Vec<TaskId> {
    world
        .orders()
        .flat_map(move |order| world.tasks_of(order.id))
        .filter(|task| matches!(task.state, TaskState::Claimed { .. }))
        .map(|task| task.id)
        .collect()
}

async fn run_one<C: ModelClient>(host: Host<C>, job: Job) -> (Job, Result<(), JobError>) {
    let result = match execute(&host, &job).await {
        Ok(Some(command)) => host
            .submit(Command::System(command))
            .await
            .map(drop)
            .map_err(JobError::from),
        Ok(None) => Ok(()),
        Err(err) => Err(err),
    };
    if let Err(err) = &result {
        tracing::debug!(?job, error = %err, "job attempt failed");
    }
    (job, result)
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

/// Merges the front of the land queue into the protected branch. The core derives a `Land` only
/// for that front, so merges happen one at a time, in queue order.
async fn land<C: ModelClient>(
    host: &Host<C>,
    task: TaskId,
    head: &CommitId,
) -> Result<Option<SystemCmd>, JobError> {
    let result = match host.workspace() {
        Some(workspace) => workspace.land(task, head).await?,
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
