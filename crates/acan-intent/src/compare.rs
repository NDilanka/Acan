//! Run-wide normalization, base-only masks, and base-versus-candidate comparison.
//!
//! Every input is admitted ([`ObservedRun`]) and every output is [`CoreDerived`]; a decoded
//! observation, mask or comparison never reaches these functions.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use crate::error::Unsupported;
use crate::observation::{CanonicalRun, Mask, ObservedRun};
use crate::probe::{CorpusProbe, Fixture};
use crate::provenance::CoreDerived;
use crate::verdict::Comparison;

/// Why a comparison could not be made. Never a pass.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CompareError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// One run with everything run-wide normalization reads: the probe and its plan, the fixture
/// whose recipe ran first, and the admitted run with every observation of its sequence and its
/// capture fingerprints.
#[derive(Debug, Clone, Copy)]
pub struct RunContext<'a> {
    /// The probe, with its normalization plan.
    pub probe: &'a CorpusProbe,
    /// The fixture whose recipe steps precede the probe's.
    pub fixture: &'a Fixture,
    /// The admitted run.
    pub run: &'a ObservedRun,
}

/// The three runs of one probe: base A, base B and the candidate.
#[derive(Debug, Clone, Copy)]
pub struct Contrast<'a> {
    /// The probe, with its normalization plan.
    pub probe: &'a CorpusProbe,
    /// The fixture all three runs started from.
    pub fixture: &'a Fixture,
    /// The first base run.
    pub base_a: &'a ObservedRun,
    /// The second base run.
    pub base_b: &'a ObservedRun,
    /// The candidate run.
    pub candidate: &'a ObservedRun,
}

/// Normalizes a whole run under its probe's plan.
///
/// Reads the sequence in order, recipe steps first. Every capture binding resolves from the
/// observation of its capture step's `from` step through its [`crate::CaptureSource`], and the
/// resolved value must hash to the run's [`crate::CaptureOutcome::Captured`] fingerprint, so a
/// later observation canonicalizes by what earlier steps bound in this run. Projections keep
/// format, ordering and cardinality; step-qualified relations are checked across steps. Pure:
/// no hidden state and no blob reads.
///
/// # Errors
///
/// Returns [`CompareError`] when the run cannot be normalized, such as a capture whose value
/// does not match its fingerprint.
pub fn canonicalize(_context: &RunContext<'_>) -> Result<CoreDerived<CanonicalRun>, CompareError> {
    Err(Unsupported("P01: canonicalize").into())
}

/// Derives masks from base A and base B only. The candidate run is never read.
///
/// # Errors
///
/// Returns [`CompareError`] when the base runs cannot be compared.
pub fn derive_masks(_contrast: &Contrast<'_>) -> Result<Vec<CoreDerived<Mask>>, CompareError> {
    Err(Unsupported("P01: derive_masks").into())
}

/// Compares the candidate against base under `masks`.
///
/// # Errors
///
/// Returns [`CompareError`] when the runs cannot be compared.
pub fn compare(
    _contrast: &Contrast<'_>,
    _masks: &[CoreDerived<Mask>],
) -> Result<CoreDerived<Comparison>, CompareError> {
    Err(Unsupported("P01: compare").into())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::hash::ObservationHash;
    use crate::observation::Observation;
    use crate::probe::{ProbeCorpus, Run};

    pub(crate) fn corpus() -> ProbeCorpus {
        serde_json::from_str(include_str!("../fixtures/probe-corpus.json")).expect("corpus")
    }

    pub(crate) fn admitted_run() -> ObservedRun {
        let run: Run = serde_json::from_str(include_str!("../fixtures/run.json")).expect("run");
        let observation: Observation =
            serde_json::from_str(include_str!("../fixtures/observation.json")).expect("obs");
        let hash: ObservationHash = observation.observation_hash().expect("hash");
        ObservedRun::admitted(run, vec![(hash, observation)])
    }

    #[test]
    fn every_comparison_stub_fails_closed() {
        let corpus = corpus();
        let run = admitted_run();
        let context = RunContext {
            probe: &corpus.probes[0],
            fixture: &corpus.fixtures[0],
            run: &run,
        };
        let contrast = Contrast {
            probe: &corpus.probes[0],
            fixture: &corpus.fixtures[0],
            base_a: &run,
            base_b: &run,
            candidate: &run,
        };
        let failed = |op| Err(CompareError::Unsupported(Unsupported(op)));
        assert_eq!(
            canonicalize(&context).map(drop),
            failed("P01: canonicalize")
        );
        assert_eq!(
            derive_masks(&contrast).map(drop),
            failed("P01: derive_masks")
        );
        assert_eq!(compare(&contrast, &[]).map(drop), failed("P01: compare"));
    }
}
