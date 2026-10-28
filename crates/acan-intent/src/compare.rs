//! Canonical observations, base-only masks, and base-versus-candidate comparison.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use crate::error::Unsupported;
use crate::observation::{CanonicalObservation, Mask, NormalizationPlan, Observation};
use crate::probe::{CorpusProbe, Run};
use crate::verdict::Comparison;

/// Why a comparison could not be made. Never a pass.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CompareError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// One run with the observations of its steps.
#[derive(Debug, Clone, Copy)]
pub struct RunEvidence<'a> {
    /// The run record.
    pub run: &'a Run,
    /// One observation per step that produced one, in step order.
    pub observations: &'a [Observation],
}

/// The three runs of one probe: base A, base B and the candidate.
#[derive(Debug, Clone, Copy)]
pub struct Contrast<'a> {
    /// The probe, with its normalization plan.
    pub probe: &'a CorpusProbe,
    /// The first base run.
    pub base_a: RunEvidence<'a>,
    /// The second base run.
    pub base_b: RunEvidence<'a>,
    /// The candidate run.
    pub candidate: RunEvidence<'a>,
}

/// Normalizes one observation under `plan`.
///
/// # Errors
///
/// Returns [`CompareError`] when the observation cannot be normalized.
pub fn canonicalize(
    _observation: &Observation,
    _plan: &NormalizationPlan,
) -> Result<CanonicalObservation, CompareError> {
    Err(Unsupported("P01: canonicalize").into())
}

/// Derives masks from base A and base B only. The candidate run is never read.
///
/// # Errors
///
/// Returns [`CompareError`] when the base runs cannot be compared.
pub fn derive_masks(_contrast: &Contrast<'_>) -> Result<Vec<Mask>, CompareError> {
    Err(Unsupported("P01: derive_masks").into())
}

/// Compares the candidate against base under `masks`.
///
/// # Errors
///
/// Returns [`CompareError`] when the runs cannot be compared.
pub fn compare(_contrast: &Contrast<'_>, _masks: &[Mask]) -> Result<Comparison, CompareError> {
    Err(Unsupported("P01: compare").into())
}
