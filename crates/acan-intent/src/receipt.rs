//! Classification and receipt counts: the receipt as a pure function of its inputs.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use crate::compare::Contrast;
use crate::episode::{Contest, DiscrepancyEpisode};
use crate::error::Unsupported;
use crate::gate::PreparedCommit;
use crate::ledger::FoldedLedger;
use crate::observation::Mask;
use crate::predicate::RetainedExample;
use crate::probe::ProbeCorpus;
use crate::provenance::CoreDerived;
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
    pub masks: &'a [CoreDerived<Mask>],
    /// The comparison.
    pub comparison: &'a CoreDerived<Comparison>,
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
    pub ledger: &'a CoreDerived<FoldedLedger>,
    /// The protected probe corpus.
    pub corpus: &'a ProbeCorpus,
    /// One result per probe run.
    pub results: &'a [ProbeResult<'a>],
    /// The retained examples fixed `equals_base` anchors name.
    pub anchors: &'a [RetainedExample<'a>],
    /// The order's episodes.
    pub episodes: &'a [DiscrepancyEpisode],
    /// The order's contests.
    pub contests: &'a [Contest],
    /// Where the gate is.
    pub gate_state: GateState,
}

/// Classifies every item and counts the classes. The counts always equal the items
/// ([`Receipt::check_counts`]).
///
/// # Errors
///
/// Returns [`ReceiptError`] when the inputs are inconsistent or classification fails.
pub fn build_receipt(_inputs: &ReceiptInputs<'_>) -> Result<CoreDerived<Receipt>, ReceiptError> {
    Err(Unsupported("P01: build_receipt").into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::tests::{admitted_run, corpus};

    #[test]
    fn build_receipt_fails_closed() {
        let corpus = corpus();
        let run = admitted_run();
        let contrast = Contrast {
            probe: &corpus.probes[0],
            fixture: &corpus.fixtures[0],
            base_a: &run,
            base_b: &run,
            candidate: &run,
        };
        let comparison = CoreDerived::derived(crate::verdict::tests::unchanged(&corpus));
        let scope: Scope =
            serde_json::from_str(include_str!("../fixtures/scope.json")).expect("scope");
        let confirmation: ScopeConfirmation =
            serde_json::from_str(include_str!("../fixtures/scope-confirmation.json"))
                .expect("confirmation");
        let receipt: Receipt =
            serde_json::from_str(include_str!("../fixtures/receipt.json")).expect("receipt");
        let ledger = CoreDerived::derived(FoldedLedger::empty());
        let results = [ProbeResult {
            contrast,
            masks: &[],
            comparison: &comparison,
        }];
        let inputs = ReceiptInputs {
            scope: &scope,
            confirmation: &confirmation,
            prepared: &receipt.prepared,
            ledger: &ledger,
            corpus: &corpus,
            results: &results,
            anchors: &[],
            episodes: &[],
            contests: &[],
            gate_state: GateState::Running,
        };
        assert_eq!(
            build_receipt(&inputs),
            Err(ReceiptError::Unsupported(Unsupported("P01: build_receipt")))
        );
    }
}
