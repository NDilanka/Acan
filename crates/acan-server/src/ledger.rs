//! The ruling log, `.acan/rulings.log`: one line per human decision, append only, hash chained.
//!
//! The log is the one authority for rulings. Expectations and every database index are derived
//! from it and rebuild from it with no model call. A truncated tail, a broken chain or a
//! duplicate append is an error; it never yields a shorter log, a default expectation or a
//! green receipt.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use std::path::Path;

use acan_intent::{FoldedLedger, LedgerEvent, LedgerHead, Unsupported};

/// The log's file name inside the state directory.
pub(crate) const RULINGS_LOG: &str = "rulings.log";

/// Why the ruling log could not be read or written.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum LedgerError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// Reads and verifies every event in `log`, in order.
///
/// # Errors
///
/// Returns [`LedgerError`] when the log cannot be read, a line does not decode, or the hash
/// chain is broken.
pub(crate) fn read(_log: &Path) -> Result<Vec<LedgerEvent>, LedgerError> {
    Err(Unsupported("S01: ledger::read").into())
}

/// The log's current revision.
///
/// # Errors
///
/// Returns [`LedgerError`] when the log cannot be read or verified.
pub(crate) fn head(_log: &Path) -> Result<LedgerHead, LedgerError> {
    Err(Unsupported("S01: ledger::head").into())
}

/// Appends `event` after `expected` and returns the new head. Fails when the log has moved
/// past `expected`, so a retried append cannot land twice.
///
/// # Errors
///
/// Returns [`LedgerError`] when the head moved, the event does not chain onto `expected`, or
/// the write fails.
pub(crate) fn append(
    _log: &Path,
    _expected: &LedgerHead,
    _event: &LedgerEvent,
) -> Result<LedgerHead, LedgerError> {
    Err(Unsupported("S01: ledger::append").into())
}

/// Reads `log` and folds it into expectations and notes.
///
/// # Errors
///
/// Returns [`LedgerError`] when the log cannot be read, verified or folded.
pub(crate) fn load(_log: &Path) -> Result<FoldedLedger, LedgerError> {
    Err(Unsupported("S01: ledger::load").into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decision() -> LedgerEvent {
        serde_json::from_str(include_str!(
            "../../acan-intent/fixtures/ledger-decision.json"
        ))
        .expect("the decision fixture decodes")
    }

    fn assert_unsupported<T: std::fmt::Debug>(result: Result<T, LedgerError>, operation: &str) {
        match result {
            Err(LedgerError::Unsupported(Unsupported(name))) => assert_eq!(name, operation),
            other => panic!("{operation} must fail closed, got {other:?}"),
        }
    }

    #[test]
    fn every_ledger_operation_fails_closed() {
        let dir = tempfile::tempdir().expect("temporary directory");
        let log = dir.path().join(RULINGS_LOG);
        assert_unsupported(read(&log), "S01: ledger::read");
        assert_unsupported(head(&log), "S01: ledger::head");
        let empty = LedgerHead {
            event_seq: 0,
            hash: None,
        };
        assert_unsupported(append(&log, &empty, &decision()), "S01: ledger::append");
        assert_unsupported(load(&log), "S01: ledger::load");
        assert!(!log.exists(), "a stub must not create the log");
    }
}
