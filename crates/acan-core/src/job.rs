//! Work the host must do outside the core, derived from state.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ids::{ClashId, CommitId, GateRunId, OrderId, OrderRev, ReadbackId, TaskId, WitnessId};

/// A job the host runs and reports back as a `SystemCmd`.
///
/// Jobs are derived by [`crate::World::jobs`], never queued, so a crash restarts them. The
/// host keeps an in-flight set keyed by the job value and skips jobs already running. A
/// result for a job that is no longer derived is rejected as stale.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "job", rename_all = "snake_case")]
pub enum Job {
    /// Ask the foreman to split an order. Reported as `SplitReady`.
    Split {
        /// The order.
        order: OrderId,
        /// The revision to split at.
        rev: OrderRev,
    },
    /// Ask the foreman to review readbacks. Reported as `ReadbacksReviewed`.
    ///
    /// The readback set is part of the key, so a new readback derives a new job.
    Review {
        /// The order.
        order: OrderId,
        /// The revision to judge against.
        rev: OrderRev,
        /// The readbacks needing a verdict at `rev`.
        readbacks: BTreeSet<ReadbackId>,
    },
    /// Ask the foreman to explain a clash. Reported as `ClashExplained`.
    Explain {
        /// The clash.
        clash: ClashId,
    },
    /// Run one witness of a gate run. Reported as `WitnessResult`.
    RunWitness {
        /// The run.
        run: GateRunId,
        /// The witness.
        witness: WitnessId,
    },
    /// Merge the front of the land queue. Reported as `LandResult`.
    Land {
        /// The task.
        task: TaskId,
        /// The commit that passed its gate.
        head: CommitId,
    },
}
