//! Gate runs: a task's gated witnesses against one commit, and the land queue.
//!
//! ```text
//! Building --ReportBuilt(fence, head)--> Gating          [BuildReported]
//! Gating --WitnessResult x n--> verdict Running           [WitnessReported]
//!        verdict Passed --> Queued                        [GateClosed, LeaseEnded Delivered]
//!        verdict Failed --> Building                      [GateClosed]
//! Queued --LandResult Ok--> Landed                        [Landed]
//! Queued --LandResult Err--> Open                         [LandFailed]
//! ```

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::event::{Event, Rejection};
use crate::ids::{AgentId, CommitId, GateRunId, Generation, OrderRev, TaskId, WitnessId};
use crate::lease::Fence;
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
        todo!()
    }
}

/// `ReportBuilt`: from `Building` only. Freezes the order's gated witnesses into a new run.
pub(crate) fn decide_report_built(
    world: &World,
    ctx: &Ctx<'_>,
    agent: &AgentId,
    fence: Fence,
    head: CommitId,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}

/// `WitnessResult`: rejected unless `run` is the task's current gating run and `witness` is
/// frozen in it. Closes the run when the verdict leaves `Running`.
pub(crate) fn decide_witness_result(
    world: &World,
    run: GateRunId,
    witness: WitnessId,
    outcome: WitnessOutcome,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}

/// `LandResult`: rejected unless the task is `Queued` at `head` and at the front of the land queue.
pub(crate) fn decide_land_result(
    world: &World,
    task: TaskId,
    head: CommitId,
    result: Result<CommitId, String>,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}
