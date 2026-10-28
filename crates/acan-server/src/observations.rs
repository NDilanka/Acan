//! Raw observation storage, addressed by content hash.
//!
//! An observation is stored once under its [`ObservationHash`] and read back only by that hash.
//! A missing or altered observation is an error, never an empty or default observation, so a
//! lost file cannot turn into a pass.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use std::path::Path;

use acan_intent::{Observation, ObservationHash, Unsupported};

/// Why an observation could not be stored or read.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum ObservationError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// Stores `observation` under `dir` and returns its hash. Storing the same observation twice
/// is not an error and stores nothing new.
///
/// # Errors
///
/// Returns [`ObservationError`] when the observation cannot be hashed or written.
pub(crate) fn put(
    _dir: &Path,
    _observation: &Observation,
) -> Result<ObservationHash, ObservationError> {
    Err(Unsupported("S01: observations::put").into())
}

/// Reads the observation stored under `hash`, verifying that its content still has that hash.
///
/// # Errors
///
/// Returns [`ObservationError`] when it is missing, unreadable or altered.
pub(crate) fn get(_dir: &Path, _hash: &ObservationHash) -> Result<Observation, ObservationError> {
    Err(Unsupported("S01: observations::get").into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_unsupported<T: std::fmt::Debug>(
        result: Result<T, ObservationError>,
        operation: &str,
    ) {
        match result {
            Err(ObservationError::Unsupported(Unsupported(name))) => assert_eq!(name, operation),
            other => panic!("{operation} must fail closed, got {other:?}"),
        }
    }

    #[test]
    fn every_observation_operation_fails_closed() {
        let dir = tempfile::tempdir().expect("temporary directory");
        let observation: Observation =
            serde_json::from_str(include_str!("../../acan-intent/fixtures/observation.json"))
                .expect("the observation fixture decodes");
        let hash = observation
            .observation_hash()
            .expect("the observation hashes");
        assert_unsupported(put(dir.path(), &observation), "S01: observations::put");
        assert_unsupported(get(dir.path(), &hash), "S01: observations::get");
    }
}
