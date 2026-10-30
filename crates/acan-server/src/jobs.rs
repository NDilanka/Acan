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
//! The loop also keeps each task's worktree in step with its live generation. A claimed task's
//! generation is its lease generation; a queued task keeps the generation of the gate run that
//! passed. The worktree stays while either holds, and a different generation resets it before
//! creating it again, so a reclaim starts at the current protected head. The worktree goes only
//! when the task has no live generation. That step is not a job, because the core derives no key
//! for it. After a land, and before its result is recorded, every claimed task that is not
//! building yet is fast-forwarded onto the landed commit, so a serialized waiter builds on what
//! its predecessor landed.
//!
//! A `Barrier` job sleeps until its order's declare barrier is due and then sends a `Tick`,
//! the same command the lease expiry runs on.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::time::Instant;

use acan_core::{
    Check, ClashId, Command, CommitId, GateRunId, Generation, Job, OrderId, OrderRev, Rejection,
    Stage, Step, SystemCmd, TaskId, TaskState, Timestamp, WitnessId, WitnessKind, WitnessOutcome,
    World,
};
use acan_llm::{CallRecord, LlmError, ModelClient, Role};
use tokio::task::{self, JoinHandle, JoinSet};

use crate::git::{LandFailure, Workspace};
use crate::host::{self, Host, SubmitError};
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
    /// The worktree now recorded at this generation.
    Create(Generation),
    /// The worktree of this generation was removed.
    Remove(Generation),
    /// The create found the task's live generation had moved and changed nothing.
    Stale,
}

/// The worktrees of the tasks, as the loop last left them, and the steps still running.
#[derive(Default)]
struct WorktreeSync {
    /// The generation recorded for each task whose worktree exists on disk.
    present: HashMap<TaskId, Generation>,
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
            Ok(present) => {
                for (task, generation) in present {
                    // A worktree with no recorded generation predates this and cannot be matched
                    // to a live claim. Recording zero makes the first reconcile remove it; a live
                    // claim's generation is never zero, so the removal cannot match it.
                    worktrees
                        .present
                        .insert(task, generation.unwrap_or(Generation::new(0)));
                }
            }
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
                        Change::Create(generation) => {
                            worktrees.present.insert(task, generation);
                        }
                        Change::Remove(generation) => {
                            // A newer generation may have been recorded while this removal ran.
                            if worktrees.present.get(&task) == Some(&generation) {
                                worktrees.present.remove(&task);
                            }
                        }
                        // Nothing was created; the reconcile below retries at the generation
                        // the task actually holds now.
                        Change::Stale => {}
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

/// Brings each task's worktree in line with its live generation: created while the task is
/// claimed or queued, reset and created again when the generation changed, removed once the task
/// has none. A task with a step in flight waits for that step, which reconciles again when it
/// finishes.
fn reconcile<C: ModelClient + 'static>(host: &Host<C>, worktrees: &mut WorktreeSync) {
    let Some(workspace) = host.workspace() else {
        return;
    };
    let live = host.read(live_generations);
    let create: Vec<(TaskId, Generation)> = live
        .iter()
        .filter(|(task, generation)| worktrees.present.get(task) != Some(generation))
        .map(|(task, generation)| (*task, *generation))
        .collect();
    let remove: Vec<(TaskId, Generation)> = worktrees
        .present
        .iter()
        .filter(|(task, _)| !live.contains_key(task))
        .map(|(task, generation)| (*task, *generation))
        .collect();
    for (task, generation) in create {
        if is_running(worktrees, task) {
            continue;
        }
        let workspace = Arc::clone(&workspace);
        let host = host.clone();
        let id = worktrees
            .ops
            .spawn(async move {
                // The world may have moved between the check above and this step. Under the
                // task lock the generation is still what reconcile captured, so a witness of
                // the old generation cannot be running in the worktree this step replaces,
                // and a newer claim's create has already won the lock.
                let _task = workspace.lock_task(task).await;
                let still_live =
                    host.read(|world| live_generations(world).get(&task) == Some(&generation));
                if !still_live {
                    return (task, Change::Stale, Ok(()));
                }
                let result = workspace.worktree(task, generation).await.map(drop);
                (task, Change::Create(generation), result)
            })
            .id();
        worktrees.running.insert(id, task);
    }
    for (task, generation) in remove {
        if is_running(worktrees, task) {
            continue;
        }
        let workspace = Arc::clone(&workspace);
        let id = worktrees
            .ops
            .spawn(async move {
                let result = workspace.reset(task, generation).await;
                (task, Change::Remove(generation), result)
            })
            .id();
        worktrees.running.insert(id, task);
    }
}

fn is_running(worktrees: &WorktreeSync, task: TaskId) -> bool {
    worktrees.running.values().any(|running| *running == task)
}

/// The generation each task's worktree must sit at. A claimed task uses its lease generation. A
/// queued task keeps the generation of the gate run that passed, so its branch survives until it
/// lands. Every other state has none, and its worktree is removed.
fn live_generations(world: &World) -> HashMap<TaskId, Generation> {
    world
        .orders()
        .flat_map(move |order| world.tasks_of(order.id))
        .filter_map(|task| {
            let generation = match &task.state {
                TaskState::Claimed { claim, .. } => Some(claim.lease.generation),
                TaskState::Queued { run, .. } => world.gate(*run).map(|gate| gate.generation),
                TaskState::Open | TaskState::Landed { .. } | TaskState::Dropped => None,
            };
            generation.map(|generation| (task.id, generation))
        })
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
        // Review is retired. The server sends no review to any model and writes no verdict, so a
        // derived review job does nothing.
        Job::Review { .. } => Ok(None),
        Job::Explain { clash } => explain(host, job, *clash).await,
        Job::Barrier { deadline, .. } => {
            sleep_until(*deadline).await;
            Ok(Some(SystemCmd::Tick))
        }
        Job::RunWitness { run, witness } => run_witness(host, *run, *witness).await,
        Job::Land { task, head } => land(host, *task, head).await,
    }
}

/// Sleeps until the origin's clock reaches `deadline`. The writer stamps commands with the same
/// clock, so the `Tick` sent afterwards finds the barrier due.
async fn sleep_until(deadline: Timestamp) {
    loop {
        let left = deadline.as_millis().saturating_sub(host::now().as_millis());
        if left == 0 {
            return;
        }
        tokio::time::sleep(Duration::from_millis(left)).await;
    }
}

async fn split<C: ModelClient>(
    host: &Host<C>,
    job: &Job,
    order: OrderId,
    rev: OrderRev,
) -> Result<Option<SystemCmd>, JobError> {
    let machines = model::plant_machines(&host.scan().index);
    let Some(request) = host.read(|world| model::split_request(world, order, &machines)) else {
        return Ok(None);
    };
    let reply = host
        .client()
        .chat_json::<model::SplitReply>(Role::Foreman, request)
        .await?;
    record_usage(host, job, &reply.call, Step::Split).await;
    Ok(Some(model::split_result(
        order,
        rev,
        &machines,
        reply.value,
    )?))
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
    let Some(task) = host.read(|world| world.gate(run).map(|gate| gate.task)) else {
        return Ok(None);
    };
    let Some(outcome) = check_at(host, task, run, witness).await? else {
        return Ok(None);
    };
    Ok(Some(SystemCmd::WitnessResult {
        run,
        witness,
        outcome,
    }))
}

/// Runs the witness in the task's worktree, once that worktree sits at the gated commit.
///
/// The world is read again under the task lock. A gate run that is no longer the task's current
/// run, or a generation that no longer matches, returns no result and creates no worktree. A
/// worktree at another commit is an error outcome, which the gate counts as a failure. A git
/// failure is an error of the job and follows the retry path.
async fn check_at<C: ModelClient>(
    host: &Host<C>,
    task: TaskId,
    run: GateRunId,
    witness: WitnessId,
) -> Result<Option<WitnessOutcome>, JobError> {
    let Some(workspace) = host.workspace() else {
        return Ok(Some(WitnessOutcome::Error {
            detail: "this server has no git repository to run witnesses in".to_owned(),
        }));
    };
    // Two runs of one task would share its worktree directory, so they take turns. The task lock
    // comes before the global slot, so a run waiting for its task never holds a slot.
    let _task = workspace.lock_task(task).await;
    let current = host.read(|world| current_witness(world, task, run, witness));
    let Some((generation, head, check)) = current else {
        return Ok(None);
    };
    let _slot = host
        .witness_slots()
        .acquire_owned()
        .await
        .expect("the witness semaphore is never closed");
    let worktree = workspace.worktree(task, generation).await?;
    let built = worktree.head().await?;
    if built != head {
        return Ok(Some(WitnessOutcome::Error {
            detail: format!("the task branch is at {built}, not the gated commit {head}"),
        }));
    }
    Ok(Some(
        acan_witness::run(&check, worktree.path(), &host.witness_limits()).await,
    ))
}

/// The generation, commit and check of `run`, when it is still `task`'s current gate run and
/// `witness` is still frozen in it. A run that closed, or a generation that moved on, is `None`.
fn current_witness(
    world: &World,
    task: TaskId,
    run: GateRunId,
    witness: WitnessId,
) -> Option<(Generation, CommitId, Check)> {
    let gate = world.gate(run)?;
    let current = world.task(task).is_some_and(|found| {
        matches!(
            &found.state,
            TaskState::Claimed {
                stage: Stage::Gating { run: live, .. },
                ..
            } if *live == run
        )
    });
    let generation_holds = world.task(task).is_some_and(|found| {
        matches!(
            &found.state,
            TaskState::Claimed { claim, .. } if claim.lease.generation == gate.generation
        )
    });
    if !current || !generation_holds {
        return None;
    }
    let WitnessKind::Check { check } = &world.witness(witness)?.kind else {
        return None;
    };
    Some((gate.generation, gate.head.clone(), check.clone()))
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
            Ok(commit) => {
                catch_up(host, &workspace, &commit).await;
                Ok(commit)
            }
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

/// Moves the worktree of every task not yet cleared to write onto the commit that just landed.
///
/// This runs before the land result is recorded. A `Serialize` or `SplitAtContract` waiter is
/// cleared to build only once that result marks its predecessor landed, so its agent starts on
/// the landed code and its own land does not conflict with it. A worker writes nothing before
/// its task is building, and a worktree with work of its own is left where it is. A failure is
/// logged and does not undo the land: the waiter then builds on its old base, and a conflicting
/// land is refused and reopens it.
async fn catch_up<C: ModelClient>(host: &Host<C>, workspace: &Workspace, landed: &CommitId) {
    for (task, generation) in host.read(not_yet_building) {
        // Under the task lock the generation cannot move, so the worktree of a newer claim is
        // never reset back to this one.
        let _task = workspace.lock_task(task).await;
        let still = host.read(|world| not_yet_building(world).get(&task) == Some(&generation));
        if !still {
            continue;
        }
        match workspace.fast_forward(task, generation, landed).await {
            Ok(true) => tracing::debug!(%task, %landed, "moved a waiting worktree onto the land"),
            Ok(false) => tracing::info!(
                %task,
                %landed,
                "a waiting worktree has work of its own; it stays on its base"
            ),
            Err(err) => tracing::warn!(
                %task,
                error = %err,
                "moving a waiting worktree onto the land failed; it stays on its base"
            ),
        }
    }
}

/// The lease generation of each claimed task that is not building yet.
fn not_yet_building(world: &World) -> HashMap<TaskId, Generation> {
    world
        .orders()
        .flat_map(move |order| world.tasks_of(order.id))
        .filter_map(|task| match &task.state {
            TaskState::Claimed {
                claim,
                stage: Stage::Declaring | Stage::Declared { .. },
            } => Some((task.id, claim.lease.generation)),
            _ => None,
        })
        .collect()
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
