//! Runs the origin's derived model jobs and records each result as a `SystemCmd`.
//!
//! The loop starts each derived job once per key. A job fails when the model call fails, the
//! reply cannot become a command, or the origin refuses the result. A failed job is retried
//! with backoff up to [`MAX_JOB_ATTEMPTS`], then left alone until its key leaves the derived
//! set and returns, so a broken model can neither spin the loop nor burn provider credits
//! without user activity. A refused result whose job is no longer derived is simply dropped,
//! because its key is gone. Accounting-only batches never wake the loop; the writer guarantees
//! that.

use std::collections::{BTreeSet, HashMap};
use std::time::Duration;

use tokio::time::Instant;

use acan_core::{
    ClashId, Command, Job, OrderId, OrderRev, ReadbackId, Rejection, Step, SystemCmd, World,
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
    // The set is owned by this loop, so aborting the loop aborts every job it started. A job
    // must never write to the log after its server has stopped.
    let mut tasks: JoinSet<(Job, Result<(), JobError>)> = JoinSet::new();
    let mut in_flight: HashMap<task::Id, Job> = HashMap::new();
    let mut retries: HashMap<Job, Retry> = HashMap::new();
    let mut unhandled: BTreeSet<&'static str> = BTreeSet::new();
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
                start_derived(&host, &mut tasks, &mut in_flight, &mut retries, &mut unhandled);
            }
            Some(finished) = tasks.join_next() => match finished {
                Ok((job, Ok(()))) => {
                    in_flight.retain(|_, running| *running != job);
                    retries.remove(&job);
                }
                Ok((job, Err(err))) => {
                    in_flight.retain(|_, running| *running != job);
                    record_failure(&mut retries, job, err);
                }
                Err(err) => {
                    tracing::error!(error = %err, "a model job task failed");
                    in_flight.remove(&err.id());
                }
            },
            () = wait_until(next_retry) => {
                start_derived(&host, &mut tasks, &mut in_flight, &mut retries, &mut unhandled);
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

fn record_failure(retries: &mut HashMap<Job, Retry>, job: Job, err: JobError) {
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
    unhandled: &mut BTreeSet<&'static str>,
) {
    let jobs = host.read(World::jobs);
    // A key that left the derived set and came back is a fresh job and earns fresh attempts.
    retries.retain(|job, _| jobs.contains(job));
    let now = Instant::now();
    for job in jobs {
        match job {
            Job::Split { .. } | Job::Review { .. } | Job::Explain { .. } => {
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
            Job::RunWitness { .. } => log_unhandled(unhandled, "run_witness"),
            Job::Land { .. } => log_unhandled(unhandled, "land"),
        }
    }
}

fn log_unhandled(seen: &mut BTreeSet<&'static str>, kind: &'static str) {
    if seen.insert(kind) {
        tracing::warn!(
            job = kind,
            "this server build does not run this job kind yet"
        );
    }
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
        tracing::debug!(?job, error = %err, "model job attempt failed");
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
        Job::RunWitness { .. } | Job::Land { .. } => {
            unreachable!("the job loop starts only model jobs")
        }
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
