//! The final gate and installation of the tested merged commit.
//!
//! The gate passes only when the receipt passes on its own terms and was built for exactly the
//! context the gate computes now. Installation moves the protected branch to the prepared merged
//! commit by compare-and-swap, so a protected branch that moved since the run is refused, never
//! installed over. Every error leaves the protected branch where it was.

use std::collections::BTreeSet;

use acan_core::{GateRunId, TaskId, TaskState, World};
use acan_git::{InstallOutcome, Preparation, Repo};
use acan_intent::{
    CommitSha, EpisodeId, GateContext, GateState, ItemId, ItemState, LedgerEvent, Receipt,
    RulingStatus, Scope, ScopeConfirmation,
};

/// One reason a receipt does not pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Blocker {
    /// The item that does not pass, when one does.
    pub(crate) item: Option<ItemId>,
    /// What does not pass.
    pub(crate) reason: String,
}

/// Why a candidate may not land.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum LandError {
    /// The receipt does not pass. Each blocker names what failed.
    #[error("the receipt blocks the candidate: {} reason(s)", .0.len())]
    Blocked(Vec<Blocker>),
    /// The receipt was built for another context than the gate computes now.
    #[error("the receipt was built for another context than the gate sees now")]
    ContextChanged,
    /// The protected branch did not move to the prepared commit.
    #[error("the protected branch did not move: {0:?}")]
    NotInstalled(InstallOutcome),
    /// Git failed.
    #[error(transparent)]
    Git(#[from] acan_git::Error),
}

/// Proof that the final gate passed for one merged commit on one protected head. Only
/// [`final_gate`] makes one, so [`install`] cannot run without it.
#[derive(Debug)]
pub(crate) struct Clearance {
    preparation: Preparation,
    tested: GateContext,
}

/// Every reason `receipt` does not pass. An empty list is a pass: at least one item was checked,
/// every item is preserved or has its obligations fulfilled, every obligation was checked and
/// held, no scope item went unreached, and no required ruling failed.
pub(crate) fn blockers(receipt: &Receipt) -> Vec<Blocker> {
    let mut found = Vec::new();
    if matches!(
        receipt.gate_state,
        GateState::Blocked | GateState::Stopped | GateState::Landed
    ) {
        found.push(Blocker {
            item: None,
            reason: format!("the gate is {:?}", receipt.gate_state),
        });
    }
    if receipt.items.is_empty() {
        found.push(Blocker {
            item: None,
            reason: "no item was checked".to_owned(),
        });
    }
    for item in &receipt.items {
        if !item.class.counts_toward_pass() {
            found.push(Blocker {
                item: Some(item.item_id.clone()),
                reason: format!("classified {:?}", item.class),
            });
        }
        if item.state == ItemState::Residue {
            found.push(Blocker {
                item: Some(item.item_id.clone()),
                reason: "waiting for a human ruling".to_owned(),
            });
        }
    }
    for missing in &receipt.not_reached {
        found.push(Blocker {
            item: Some(missing.item_id.clone()),
            reason: "no probe reached it".to_owned(),
        });
    }
    let behavior = receipt.behavior;
    if behavior.obligations_total == 0
        || behavior.failed != 0
        || behavior.unknown != 0
        || behavior.passed != behavior.obligations_total
    {
        found.push(Blocker {
            item: None,
            reason: format!(
                "{} of {} obligations passed, {} failed, {} unknown",
                behavior.passed, behavior.obligations_total, behavior.failed, behavior.unknown
            ),
        });
    }
    if receipt.check_counts().is_err() {
        found.push(Blocker {
            item: None,
            reason: "the class counts do not match the items".to_owned(),
        });
    }
    for check in &receipt.rulings {
        if !matches!(
            check.status,
            RulingStatus::Held {} | RulingStatus::NotEnforced { .. }
        ) {
            found.push(Blocker {
                item: None,
                reason: format!("ruling {:?} is {:?}", check.ruling_id, check.status),
            });
        }
    }
    found
}

/// Clears the candidate when `receipt` passes and was built for exactly `current`, the context
/// the gate computes now for `preparation`.
///
/// The context is checked first. A receipt built for another context, such as a ledger head an
/// answer has since moved, is refused as changed and its blockers are never read.
///
/// # Errors
///
/// Returns [`LandError::ContextChanged`] when its context differs from `current`, or `current`
/// does not name `preparation`, and [`LandError::Blocked`] when the receipt does not pass.
pub(crate) fn final_gate(
    receipt: &Receipt,
    current: &GateContext,
    preparation: &Preparation,
) -> Result<Clearance, LandError> {
    if receipt.prepared.context != *current || preparation.bind(current.clone()) != *current {
        return Err(LandError::ContextChanged);
    }
    let blockers = blockers(receipt);
    if !blockers.is_empty() {
        return Err(LandError::Blocked(blockers));
    }
    Ok(Clearance {
        preparation: preparation.clone(),
        tested: current.clone(),
    })
}

/// Moves the protected branch to the cleared merged commit, if it still sits at the head the
/// commit was prepared on.
///
/// # Errors
///
/// Returns [`LandError::NotInstalled`] with git's outcome when the branch did not move, and
/// [`LandError::Git`] when git fails.
pub(crate) async fn install(repo: &Repo, clearance: Clearance) -> Result<CommitSha, LandError> {
    match repo
        .install(&clearance.preparation, &clearance.tested)
        .await?
    {
        InstallOutcome::Installed { commit } => Ok(commit),
        outcome => Err(LandError::NotInstalled(outcome)),
    }
}

/// The receipt of a scope-first task waiting in the land queue, with the scope and confirmation
/// it was judged against. A delivery is read from the world, so it can be stale by the time it
/// is used. [`Delivery::stands`] checks it again.
#[derive(Debug, Clone)]
pub(crate) struct Delivery {
    /// The queued task.
    pub(crate) task: TaskId,
    /// The gate run that passed.
    pub(crate) run: GateRunId,
    /// The receipt the run passed with.
    pub(crate) receipt: Receipt,
    /// The scope the receipt was judged against.
    pub(crate) scope: Scope,
    /// The confirmation of that scope.
    pub(crate) confirmation: ScopeConfirmation,
}

impl Delivery {
    /// The delivery of a queued scope-first task. Each missing piece is named, because a refusal
    /// is the land result the origin records.
    ///
    /// # Errors
    ///
    /// Returns the refusal reason when the task is missing, not queued, has no scope, its run has
    /// no receipt, or its scope was revised since the candidate passed and was not confirmed.
    pub(crate) fn of(world: &World, task: TaskId) -> Result<Self, String> {
        let found = world
            .task(task)
            .ok_or_else(|| format!("task {task} is not in the origin"))?;
        let TaskState::Queued { run, .. } = &found.state else {
            return Err(format!("task {task} is not queued"));
        };
        let lane = world
            .lane(found.order)
            .ok_or_else(|| format!("order {} has no scope", found.order))?;
        let receipt = lane
            .runs
            .get(run)
            .and_then(|record| record.receipt.as_ref())
            .ok_or_else(|| format!("run {run} has no receipt"))?
            .receipt
            .clone();
        let confirmation = lane.confirmation.clone().ok_or_else(|| {
            "the scope was revised after the candidate passed, so it must be confirmed again"
                .to_owned()
        })?;
        Ok(Self {
            task,
            run: *run,
            receipt,
            scope: lane.scope.clone(),
            confirmation,
        })
    }

    /// Whether this delivery still stands in `world`: the task is still queued on the same run,
    /// and the scope and its confirmation are the ones the run was judged against. A revision, a
    /// cancel, or a newer run each refuse it.
    ///
    /// # Errors
    ///
    /// Returns the refusal reason when the delivery no longer stands.
    pub(crate) fn stands(&self, world: &World) -> Result<(), String> {
        let now = Self::of(world, self.task)?;
        if now.run == self.run && now.scope == self.scope && now.confirmation == self.confirmation {
            Ok(())
        } else {
            Err("the queued delivery changed after the candidate passed".to_owned())
        }
    }
}

/// The episodes with an answer in `events`, the core ledger in log order. A decision or a note
/// answers the episode it names. The core ledger is the one authority for rulings, so answers are
/// read from it and never from a file.
pub(crate) fn answered(events: &[LedgerEvent]) -> BTreeSet<EpisodeId> {
    events.iter().filter_map(answered_episode).collect()
}

/// The episode an event answers, when it answers one. Every variant is named, so a new kind of
/// event forces a decision here instead of silently leaving its episode pending.
fn answered_episode(event: &LedgerEvent) -> Option<EpisodeId> {
    match event {
        LedgerEvent::DecisionConfirmed(decision) => decision.episode_id,
        LedgerEvent::NoteRecorded(note) => note.episode_id,
        LedgerEvent::ProbeRetired(_) => None,
    }
}

/// The items of `receipt` that wait for a human ruling and whose episode has no answer in
/// `answered`. These are the pending rulings: a receipt item in residue, with no decision or note
/// in the folded log for its episode.
///
/// The gate refuses every residue item, answered or not (see [`blockers`]). An answer moves the
/// ledger head, so a receipt built before it fails the context check in [`final_gate`]. Only a
/// new run on the answered head can clear it.
pub(crate) fn pending(receipt: &Receipt, answered: &BTreeSet<EpisodeId>) -> Vec<ItemId> {
    receipt
        .items
        .iter()
        .filter(|item| {
            item.state == ItemState::Residue
                && item
                    .episode_id
                    .as_ref()
                    .is_none_or(|episode| !answered.contains(episode))
        })
        .map(|item| item.item_id.clone())
        .collect()
}
