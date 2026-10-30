//! The final gate and installation of the tested merged commit.
//!
//! The gate passes only when the receipt passes on its own terms and was built for exactly the
//! context the gate computes now. Installation moves the protected branch to the prepared merged
//! commit by compare-and-swap, so a protected branch that moved since the run is refused, never
//! installed over. Every error leaves the protected branch where it was.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use acan_git::{InstallOutcome, Preparation, Repo};
use acan_intent::{
    CommitSha, CoreDerived, EpisodeId, FoldedLedger, GateContext, GateState, ItemId, ItemState,
    LedgerEvent, LedgerHead, Receipt, RulingStatus,
};

use crate::ledger::{self, LedgerError};

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

/// Why the ruling log could not be opened or appended.
#[derive(Debug, thiserror::Error)]
pub(crate) enum RulingError {
    /// The state directory could not be created, so nothing can be appended to it.
    #[error("cannot create the state directory {}: {source}", path.display())]
    StateDir {
        /// The directory.
        path: PathBuf,
        /// The cause.
        #[source]
        source: io::Error,
    },
    /// The log does not verify, fold, or accept the answer.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
}

/// The ruling log in a state directory. Every open verifies and folds the log from disk, so a
/// restart rebuilds the ruling state from the file, and nothing held in memory survives.
#[derive(Debug)]
pub(crate) struct RulingLog {
    log: PathBuf,
}

impl RulingLog {
    /// Opens the ruling log in `state_dir`. The directory is created first, so the first append
    /// always has a place to write. An absent log is empty.
    ///
    /// # Errors
    ///
    /// Returns [`RulingError::StateDir`] when the directory cannot be created, and
    /// [`RulingError::Ledger`] when the log does not verify or fold.
    pub(crate) fn open(state_dir: &Path) -> Result<Self, RulingError> {
        fs::create_dir_all(state_dir).map_err(|source| RulingError::StateDir {
            path: state_dir.to_path_buf(),
            source,
        })?;
        let log = state_dir.join(ledger::RULINGS_LOG);
        ledger::load(&log)?;
        Ok(Self { log })
    }

    /// The folded ruling state, read from disk.
    ///
    /// # Errors
    ///
    /// Returns [`RulingError::Ledger`] when the log does not verify or fold.
    pub(crate) fn folded(&self) -> Result<CoreDerived<FoldedLedger>, RulingError> {
        Ok(ledger::load(&self.log)?)
    }

    /// The episodes with an answer in the log. A decision or a note answers the episode it names.
    ///
    /// # Errors
    ///
    /// Returns [`RulingError::Ledger`] when the log does not verify.
    pub(crate) fn answered(&self) -> Result<BTreeSet<EpisodeId>, RulingError> {
        Ok(ledger::read(&self.log)?
            .iter()
            .filter_map(answered_episode)
            .collect())
    }

    /// Appends `event` after the log's current head and returns the new head.
    ///
    /// # Errors
    ///
    /// Returns [`RulingError::Ledger`] when the log does not verify or the append is refused.
    pub(crate) fn answer(&self, event: &LedgerEvent) -> Result<LedgerHead, RulingError> {
        let head = ledger::head(&self.log)?;
        Ok(ledger::append(&self.log, &head, event)?)
    }
}

/// The episode an event answers, when it answers one.
fn answered_episode(event: &LedgerEvent) -> Option<EpisodeId> {
    match event {
        LedgerEvent::DecisionConfirmed(decision) => decision.episode_id,
        LedgerEvent::NoteRecorded(note) => note.episode_id,
        _ => None,
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
