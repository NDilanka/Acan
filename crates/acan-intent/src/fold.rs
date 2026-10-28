//! Replays the ruling log into derived state, with no model call.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use crate::error::Unsupported;
use crate::ledger::{FoldedLedger, LedgerEvent};

/// Why the log could not be folded. Replay stops; nothing is guessed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum FoldError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// Folds every event of the log, in order.
///
/// # Errors
///
/// Returns [`FoldError`] on a gap, a broken chain, a conflicting supersession, or any event
/// the rules in [`crate::ledger`] refuse.
pub fn fold(_events: &[LedgerEvent]) -> Result<FoldedLedger, FoldError> {
    Err(Unsupported("P01: fold").into())
}
