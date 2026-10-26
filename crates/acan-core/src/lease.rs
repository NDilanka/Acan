//! Claim leases and fencing.
//!
//! ```text
//! Open --Claim: registered agent, readback matched at current rev, no open question-->
//!     Claimed, generation = last + 1                         [LeaseGranted]
//! Claimed --Renew(fence): generation current, now < expires_at--> Claimed [LeaseRenewed]
//! Claimed --Release(fence)--> Open                           [LeaseEnded Released]
//! Claimed --now >= expires_at (Tick or any command)--> Open  [LeaseEnded Expired]
//! Claimed --AnswerQuestion on the order--> Open              [LeaseEnded Superseded]
//! Claimed{Gating} --gate passed--> Queued                    [LeaseEnded Delivered]
//! Claimed --CancelOrder--> Dropped                           [LeaseEnded Cancelled]
//! Any fenced write whose generation is not current: Rejection::StaleGeneration, no event.
//! ```
//!
//! A claim by the live holder returns no events. A claim against an expired lease emits
//! `LeaseEnded{Expired}` then `LeaseGranted` in one batch, so expiry never waits for a tick.

use serde::{Deserialize, Serialize};

use crate::event::{Event, Rejection};
use crate::ids::{AgentId, Generation, ReadbackId, TaskId, Timestamp};
use crate::world::{Ctx, World};

/// How long a lease lives without a renewal.
///
/// Workers renew well inside this window; a dead worker frees its task this long after its
/// last renewal.
pub const LEASE_TTL_MS: u64 = 60_000;

/// A live lease on a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lease {
    /// The agent holding the task.
    pub holder: AgentId,
    /// The fencing token for this grant.
    pub generation: Generation,
    /// When the lease ends unless renewed.
    pub expires_at: Timestamp,
}

/// A task's claim: a lease and the matched readback it was granted on. Neither half is optional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    /// The lease.
    pub lease: Lease,
    /// The readback whose verdict matched the order's revision at grant time.
    pub readback: ReadbackId,
}

/// What an agent sends with every write after a claim. The origin rejects any mismatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fence {
    /// The claimed task.
    pub task: TaskId,
    /// The generation the agent was granted.
    pub generation: Generation,
}

/// Why a lease ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseEnd {
    /// The holder released it.
    Released,
    /// It was not renewed in time.
    Expired,
    /// A question was answered and the witness set changed under it.
    Superseded,
    /// Its gate passed; the task moved to the land queue.
    Delivered,
    /// The order was cancelled.
    Cancelled,
}

impl Claim {
    /// The fence an agent must present for this claim.
    #[must_use]
    pub fn fence(&self, task: TaskId) -> Fence {
        Fence {
            task,
            generation: self.lease.generation,
        }
    }
}

pub(crate) fn decide_claim(
    world: &World,
    ctx: &Ctx<'_>,
    agent: &AgentId,
    task: TaskId,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}

pub(crate) fn decide_renew(
    world: &World,
    ctx: &Ctx<'_>,
    agent: &AgentId,
    fence: Fence,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}

pub(crate) fn decide_release(
    world: &World,
    ctx: &Ctx<'_>,
    agent: &AgentId,
    fence: Fence,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}

/// Checks `fence` against the task's live lease at `ctx.now`. Every fenced command calls this first.
pub(crate) fn check_fence(
    world: &World,
    ctx: &Ctx<'_>,
    agent: &AgentId,
    fence: Fence,
) -> Result<(), Rejection> {
    todo!()
}

/// `LeaseEnded{Expired}` for every lease past its deadline at `ctx.now`. Runs before every command.
pub(crate) fn expire_due(world: &World, ctx: &Ctx<'_>) -> Vec<Event> {
    todo!()
}
