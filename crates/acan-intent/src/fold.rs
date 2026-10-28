//! Replays the ruling log into derived state, with no model call.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use crate::error::Unsupported;
use crate::ledger::{FoldedLedger, LedgerEvent};
use crate::provenance::CoreDerived;

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
pub fn fold(_events: &[LedgerEvent]) -> Result<CoreDerived<FoldedLedger>, FoldError> {
    Err(Unsupported("P01: fold").into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_fails_closed() {
        let events: Vec<LedgerEvent> = [
            include_str!("../fixtures/ledger-decision.json"),
            include_str!("../fixtures/ledger-note.json"),
            include_str!("../fixtures/ledger-decision-dashboard.json"),
            include_str!("../fixtures/ledger-decision-supersede.json"),
            include_str!("../fixtures/ledger-probe-retired.json"),
        ]
        .into_iter()
        .map(|text| serde_json::from_str(text).expect("a ledger fixture"))
        .collect();
        assert_eq!(
            fold(&events),
            Err(FoldError::Unsupported(Unsupported("P01: fold")))
        );
    }
}
