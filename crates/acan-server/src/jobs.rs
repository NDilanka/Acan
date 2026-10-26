//! Runs the origin's derived model jobs and records each result as a `SystemCmd`.
//!
//! The loop starts each derived job once. A job leaves the in-flight set when its outcome is
//! recorded or refused. A failed job runs again only after the next accepted change, so a broken
//! model cannot spin the loop.

use std::collections::{BTreeSet, HashMap};

use acan_core::{ClashId, Command, Job, OrderId, OrderRev, ReadbackId, Step, SystemCmd, World};
use acan_llm::{CallRecord, LlmError, ModelClient, Role};
use tokio::task::{self, JoinHandle, JoinSet};

use crate::host::{Host, SubmitError};
use crate::model::{self, ReplyError};

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
}

async fn run<C: ModelClient + 'static>(host: Host<C>) {
    // The set is owned by this loop, so aborting the loop aborts every job it started. A job
    // must never write to the log after its server has stopped.
    let mut tasks: JoinSet<Job> = JoinSet::new();
    let mut in_flight: HashMap<task::Id, Job> = HashMap::new();
    let mut unhandled: BTreeSet<&'static str> = BTreeSet::new();
    loop {
        tokio::select! {
            () = host.jobs_woken() => {
                start_derived(&host, &mut tasks, &mut in_flight, &mut unhandled).await;
            }
            Some(finished) = tasks.join_next() => match finished {
                Ok(job) => in_flight.retain(|_, running| *running != job),
                Err(err) => {
                    tracing::error!(error = %err, "a model job task failed");
                    in_flight.remove(&err.id());
                }
            },
        }
    }
}

async fn start_derived<C: ModelClient + 'static>(
    host: &Host<C>,
    tasks: &mut JoinSet<Job>,
    in_flight: &mut HashMap<task::Id, Job>,
    unhandled: &mut BTreeSet<&'static str>,
) {
    for job in host.read(World::jobs).await {
        match job {
            Job::Split { .. } | Job::Review { .. } | Job::Explain { .. } => {
                if in_flight.values().any(|running| *running == job) {
                    continue;
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

async fn run_one<C: ModelClient>(host: Host<C>, job: Job) -> Job {
    match execute(&host, &job).await {
        Ok(Some(command)) => apply(&host, &job, command).await,
        Ok(None) => {}
        Err(err) => {
            tracing::warn!(?job, error = %err, "model job failed; it runs again after the next change");
        }
    }
    job
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
    let Some(request) = host.read(|world| model::split_request(world, order)).await else {
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
    let prepared = host
        .read(|world| {
            let request = model::review_request(world, order, readbacks)?;
            let machine = world.order(order)?.machine.clone();
            Some((request, machine))
        })
        .await;
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
    let awaiting = host.read(|world| awaiting_review(world, order, rev)).await;
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
    let Some(request) = host
        .read(|world| model::explain_request(world, clash))
        .await
    else {
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
    apply(host, job, command).await;
}

async fn apply<C: ModelClient>(host: &Host<C>, job: &Job, command: SystemCmd) {
    if let Err(err) = host.submit(Command::System(command)).await {
        match err {
            SubmitError::Rejected(rejection) => {
                tracing::info!(?job, %rejection, "the origin refused a job result");
            }
            SubmitError::Internal(err) => {
                tracing::error!(?job, error = ?err, "recording a job result failed");
            }
        }
    }
}
