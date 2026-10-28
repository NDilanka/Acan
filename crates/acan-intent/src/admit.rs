//! Admission: the only way stored records become evidence the gate may weigh.
//!
//! Storage (S01) reads observations by hash and decodes them as plain records. Before any of
//! them reaches normalization, masking, comparison, a predicate or a receipt, it passes through
//! a function here, which checks it against the stored hashes and the run record and returns an
//! admitted type with private fields. Nothing else constructs one.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use crate::compare::Contrast;
use crate::error::Unsupported;
use crate::hash::ObservationHash;
use crate::ledger::ContrastExample;
use crate::observation::{Observation, ObservedRun};
use crate::predicate::RetainedExample;
use crate::probe::{CorpusProbe, Fixture, Run};
use crate::provenance::CoreDerived;
use crate::verdict::Comparison;

/// Why stored records were not admitted. Never evidence, never a pass.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum AdmitError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// Admits one run and its stored observations as evidence.
///
/// Checks, all of which must hold: every `stored` hash equals its observation's
/// [`Observation::observation_hash`]; every observation names `run`'s id, commit and probe;
/// `run.probe` is `probe`'s [`crate::ProbeSpec::probe_hash`] and `run.env.fixture_hash` is
/// `fixture`'s [`Fixture::fixture_hash`]; the observations follow the sequence (the fixture
/// recipe's steps, then the probe's) in order, each step at most once; and every
/// [`crate::CaptureOutcome::Captured`] names a capture step of that sequence. A step with no
/// observation is allowed only after the run's outcome says where it stopped.
///
/// # Errors
///
/// Returns [`AdmitError`] when any check fails.
pub fn observed_run(
    _probe: &CorpusProbe,
    _fixture: &Fixture,
    _run: Run,
    _stored: Vec<(ObservationHash, Observation)>,
) -> Result<ObservedRun, AdmitError> {
    Err(Unsupported("P01: admit::observed_run").into())
}

/// Admits a retained ruling example: the contrast rebuilt from its runs, checked against the
/// manifest it was retained under.
///
/// Checks that `contrast`'s probe and fixture are the manifest's, that each admitted run's
/// record and observation hashes are the manifest's for its role, and that `comparison` is the
/// comparison of exactly that contrast. The example's identity is
/// [`ContrastExample::contrast_example_hash`].
///
/// # Errors
///
/// Returns [`AdmitError`] when the contrast does not match the manifest.
pub fn retained_example<'a>(
    _manifest: &ContrastExample,
    _contrast: Contrast<'a>,
    _comparison: &'a CoreDerived<Comparison>,
) -> Result<RetainedExample<'a>, AdmitError> {
    Err(Unsupported("P01: admit::retained_example").into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::ProbeCorpus;

    #[test]
    fn admission_fails_closed() {
        let corpus: ProbeCorpus =
            serde_json::from_str(include_str!("../fixtures/probe-corpus.json")).expect("corpus");
        let run: Run = serde_json::from_str(include_str!("../fixtures/run.json")).expect("run");
        let observation: Observation =
            serde_json::from_str(include_str!("../fixtures/observation.json")).expect("obs");
        let hash = observation.observation_hash().expect("hash");
        assert_eq!(
            observed_run(
                &corpus.probes[0],
                &corpus.fixtures[0],
                run,
                vec![(hash, observation)]
            ),
            Err(AdmitError::Unsupported(Unsupported(
                "P01: admit::observed_run"
            )))
        );
    }
}
