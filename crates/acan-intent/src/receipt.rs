//! Classification and receipt counts: the receipt as a pure function of its inputs.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use crate::compare::Contrast;
use crate::episode::{Contest, DiscrepancyEpisode};
use crate::error::Unsupported;
use crate::gate::PreparedCommit;
use crate::ledger::FoldedLedger;
use crate::observation::Mask;
use crate::probe::ProbeCorpus;
use crate::report::{GateState, Receipt};
use crate::scope::{Scope, ScopeConfirmation};
use crate::verdict::Comparison;

/// Why a receipt could not be built. Never a green receipt.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ReceiptError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// One probe's contrast, masks and comparison.
#[derive(Debug, Clone, Copy)]
pub struct ProbeResult<'a> {
    /// The runs.
    pub contrast: Contrast<'a>,
    /// The masks derived from its base runs.
    pub masks: &'a [Mask],
    /// The comparison.
    pub comparison: &'a Comparison,
}

/// Everything a receipt is a function of.
#[derive(Debug, Clone, Copy)]
pub struct ReceiptInputs<'a> {
    /// The confirmed scope.
    pub scope: &'a Scope,
    /// Its confirmation.
    pub confirmation: &'a ScopeConfirmation,
    /// The prepared candidate.
    pub prepared: &'a PreparedCommit,
    /// The folded ruling log.
    pub ledger: &'a FoldedLedger,
    /// The protected probe corpus.
    pub corpus: &'a ProbeCorpus,
    /// One result per probe run.
    pub results: &'a [ProbeResult<'a>],
    /// The order's episodes.
    pub episodes: &'a [DiscrepancyEpisode],
    /// The order's contests.
    pub contests: &'a [Contest],
    /// Where the gate is.
    pub gate_state: GateState,
}

/// Classifies every item and counts the classes.
///
/// # Errors
///
/// Returns [`ReceiptError`] when the inputs are inconsistent or classification fails.
pub fn build_receipt(_inputs: &ReceiptInputs<'_>) -> Result<Receipt, ReceiptError> {
    Err(Unsupported("P01: build_receipt").into())
}
