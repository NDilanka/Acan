//! Observe jobs: each probe runs on the base twice and on the prepared merged commit once.
//!
//! Every run starts a fresh app from the exact commit under the protected app contract and
//! stores its observations by hash. A run that cannot start, times out or is cancelled is
//! recorded with that outcome; it is never dropped and never counted as a pass.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use std::path::Path;

use acan_intent::{
    AppContract, ObservationHash, PreparedCommit, ProbeCorpus, ProbeHash, Run, Unsupported,
};

/// Why a contrast could not be observed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum ObserveError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// One run and the hashes of the observations it stored, in step order.
#[derive(Debug, Clone)]
pub(crate) struct RecordedRun {
    /// The run record.
    pub(crate) run: Run,
    /// The stored observations.
    pub(crate) observations: Vec<ObservationHash>,
}

/// The three runs of one probe.
#[derive(Debug, Clone)]
pub(crate) struct ProbeRuns {
    /// The probe.
    pub(crate) probe: ProbeHash,
    /// The first base run.
    pub(crate) base_a: RecordedRun,
    /// The second base run.
    pub(crate) base_b: RecordedRun,
    /// The candidate run.
    pub(crate) candidate: RecordedRun,
}

/// Runs every probe in `corpus` against `prepared` and stores the observations under
/// `observations`.
///
/// # Errors
///
/// Returns [`ObserveError`] when the contrast cannot be set up at all. Individual run failures
/// are recorded in their [`Run`] instead.
pub(crate) async fn observe(
    _observations: &Path,
    _prepared: &PreparedCommit,
    _app: &AppContract,
    _corpus: &ProbeCorpus,
) -> Result<Vec<ProbeRuns>, ObserveError> {
    Err(Unsupported("I01: observe").into())
}

#[cfg(test)]
mod tests {
    use acan_intent::Receipt;

    use super::*;

    #[tokio::test]
    async fn observe_fails_closed() {
        let dir = tempfile::tempdir().expect("temporary directory");
        let receipt: Receipt =
            serde_json::from_str(include_str!("../../acan-intent/fixtures/receipt.json"))
                .expect("the receipt fixture decodes");
        let app: AppContract =
            serde_json::from_str(include_str!("../../acan-intent/fixtures/app-contract.json"))
                .expect("the app contract fixture decodes");
        let corpus: ProbeCorpus =
            serde_json::from_str(include_str!("../../acan-intent/fixtures/probe-corpus.json"))
                .expect("the probe corpus fixture decodes");
        match observe(dir.path(), &receipt.prepared, &app, &corpus).await {
            Err(ObserveError::Unsupported(Unsupported(name))) => assert_eq!(name, "I01: observe"),
            other => panic!("observe must fail closed, got {other:?}"),
        }
    }
}
