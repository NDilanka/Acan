//! The final gate and installation of the tested merged commit.
//!
//! The gate passes only when the receipt passes on its own terms and was built for exactly the
//! context the gate computes now. Installation moves the protected branch to the prepared merged
//! commit by compare-and-swap, so a protected branch that moved since the run is refused, never
//! installed over. Every error leaves the protected branch where it was.

use acan_git::{InstallOutcome, Preparation, Repo};
use acan_intent::{CommitSha, GateContext, GateState, ItemId, Receipt, RulingStatus};

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
/// # Errors
///
/// Returns [`LandError::Blocked`] when the receipt does not pass, and
/// [`LandError::ContextChanged`] when its context differs from `current`, or `current` does not
/// name `preparation`.
pub(crate) fn final_gate(
    receipt: &Receipt,
    current: &GateContext,
    preparation: &Preparation,
) -> Result<Clearance, LandError> {
    let blockers = blockers(receipt);
    if !blockers.is_empty() {
        return Err(LandError::Blocked(blockers));
    }
    if receipt.prepared.context != *current || preparation.bind(current.clone()) != *current {
        return Err(LandError::ContextChanged);
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
