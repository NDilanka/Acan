//! Gate runs: a task's gated witnesses against one commit, and the land queue.
//!
//! ```text
//! Building --ReportBuilt(fence, head)--> Gating          [BuildReported]
//! Gating --WitnessResult x n--> verdict Running           [WitnessReported]
//!        verdict Passed --> Queued                        [GateClosed, LeaseEnded Delivered]
//!        verdict Failed --> Declared                      [GateClosed, then BuildCleared if unblocked]
//! Gating --ClashFound on the task--> Declared             (run abandoned; late results rejected)
//! Queued --LandResult Ok--> Landed                        [Landed]
//! Queued --LandResult Err--> Open                         [LandFailed]
//! ```
//!
//! A failed gate also renews the lease from the close (`LeaseRenewed`), because the lease
//! did not run down while the gate was running.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::event::{Event, Rejection};
use crate::ids::{AgentId, CommitId, GateRunId, Generation, OrderRev, TaskId, WitnessId};
use crate::lease::{self, Fence, LEASE_TTL_MS, LeaseEnd};
use crate::order::{Stage, TaskState, Witness};
use crate::world::{Ctx, World};

/// One run of a task's gated witnesses against one commit. The witness set is frozen at start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateRun {
    /// The run id.
    pub id: GateRunId,
    /// The task under test.
    pub task: TaskId,
    /// The claim generation that reported the commit.
    pub generation: Generation,
    /// The order revision whose witnesses were frozen.
    pub rev: OrderRev,
    /// The commit under test.
    pub head: CommitId,
    /// The gated witnesses, frozen when the run started.
    pub witnesses: Vec<WitnessId>,
    /// Results so far.
    pub results: BTreeMap<WitnessId, WitnessOutcome>,
}

/// The result of one witness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum WitnessOutcome {
    /// The check passed.
    Pass,
    /// The check ran and failed.
    Fail {
        /// What failed.
        detail: String,
    },
    /// The check could not run. Never counts as a pass.
    Error {
        /// What went wrong.
        detail: String,
    },
}

/// A gate run's verdict, derived from its results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum GateVerdict {
    /// At least one frozen witness has no result yet.
    Running,
    /// Every frozen witness passed.
    Passed,
    /// Every frozen witness has a result and these did not pass.
    Failed {
        /// The witnesses that failed or errored.
        failing: Vec<WitnessId>,
    },
}

impl GateRun {
    /// Derives the verdict from the results. A run with no gated witnesses passes.
    #[must_use]
    pub fn verdict(&self) -> GateVerdict {
        if self.witnesses.iter().any(|w| !self.results.contains_key(w)) {
            return GateVerdict::Running;
        }
        let failing: Vec<WitnessId> = self
            .witnesses
            .iter()
            .copied()
            .filter(|w| !matches!(self.results.get(w), Some(WitnessOutcome::Pass)))
            .collect();
        if failing.is_empty() {
            GateVerdict::Passed
        } else {
            GateVerdict::Failed { failing }
        }
    }
}

/// `ReportBuilt`: from `Building` only. Freezes the order's gated witnesses into a new run.
/// A run with no gated witnesses closes as passed in the same batch.
pub(crate) fn decide_report_built(
    world: &World,
    ctx: &Ctx<'_>,
    agent: &AgentId,
    fence: Fence,
    head: CommitId,
) -> Result<Vec<Event>, Rejection> {
    let stage = lease::check_fence(world, ctx, agent, fence)?;
    if !matches!(stage, Stage::Building { .. }) {
        return Err(Rejection::wrong_state("task", stage.name()));
    }
    let task = world.task(fence.task).expect("a fenced task exists");
    let order = world
        .order(task.order)
        .ok_or_else(|| Rejection::unknown("order", task.order))?;
    let witnesses: Vec<WitnessId> = order
        .witnesses
        .iter()
        .copied()
        .filter(|&w| world.witness(w).is_some_and(Witness::gated))
        .collect();
    let run = world.next_gate_id()?;
    let empty = witnesses.is_empty();
    let mut events = vec![Event::BuildReported {
        task: fence.task,
        generation: fence.generation,
        head,
        run,
        rev: order.rev,
        witnesses,
    }];
    if empty {
        events.extend(close(ctx, fence, run, GateVerdict::Passed));
    }
    Ok(events)
}

/// `WitnessResult`: rejected unless `run` is the task's current gating run and `witness` is
/// frozen in it without a result. Closes the run when the verdict leaves `Running`.
pub(crate) fn decide_witness_result(
    world: &World,
    ctx: &Ctx<'_>,
    run: GateRunId,
    witness: WitnessId,
    outcome: WitnessOutcome,
) -> Result<Vec<Event>, Rejection> {
    let gate = world
        .gate(run)
        .ok_or_else(|| Rejection::unknown("gate run", run))?;
    let current = world.task(gate.task).is_some_and(|t| {
        matches!(&t.state, TaskState::Claimed { stage: Stage::Gating { run: r, .. }, .. } if *r == run)
    });
    if !current {
        return Err(Rejection::wrong_state("gate run", "closed"));
    }
    if !gate.witnesses.contains(&witness) {
        return Err(Rejection::invalid("the witness is not frozen in this run"));
    }
    if gate.results.contains_key(&witness) {
        return Err(Rejection::wrong_state("witness", "reported"));
    }
    let mut after = gate.clone();
    after.results.insert(witness, outcome.clone());
    let verdict = after.verdict();
    let mut events = vec![Event::WitnessReported {
        run,
        witness,
        outcome,
    }];
    if verdict != GateVerdict::Running {
        let fence = Fence {
            task: gate.task,
            generation: gate.generation,
        };
        events.extend(close(ctx, fence, run, verdict));
    }
    Ok(events)
}

/// Closes a run: a pass delivers the lease, a failure renews it from now.
fn close(ctx: &Ctx<'_>, fence: Fence, run: GateRunId, verdict: GateVerdict) -> [Event; 2] {
    let follow = if verdict == GateVerdict::Passed {
        Event::LeaseEnded {
            task: fence.task,
            generation: fence.generation,
            why: LeaseEnd::Delivered,
        }
    } else {
        Event::LeaseRenewed {
            task: fence.task,
            generation: fence.generation,
            expires_at: ctx.now.plus_millis(LEASE_TTL_MS),
        }
    };
    [Event::GateClosed { run, verdict }, follow]
}

/// `LandResult`: rejected unless the task is `Queued` at `head` and at the front of the land queue.
pub(crate) fn decide_land_result(
    world: &World,
    task: TaskId,
    head: &CommitId,
    result: Result<CommitId, String>,
) -> Result<Vec<Event>, Rejection> {
    let t = world
        .task(task)
        .ok_or_else(|| Rejection::unknown("task", task))?;
    let TaskState::Queued { head: queued, .. } = &t.state else {
        return Err(Rejection::wrong_state("task", t.state.name()));
    };
    if queued != head {
        return Err(Rejection::invalid(
            "the head is not the commit that passed the gate",
        ));
    }
    if world.land_queue().first() != Some(&task) {
        return Err(Rejection::wrong_state(
            "task",
            "behind the front of the land queue",
        ));
    }
    let event = match result {
        Ok(commit) => Event::Landed { task, commit },
        Err(reason) => Event::LandFailed { task, reason },
    };
    Ok(vec![event])
}
