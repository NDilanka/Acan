//! The tour director: an ordered walk over one immutable receipt snapshot.
//!
//! The director model proposes a tour. Trusted code admits it only when every step names a mark
//! that exists in the view, every mandatory item is reachable and the tour is bound to the
//! view's receipt hash. A refused or missing tour falls back to the deterministic order, and
//! the plain card works with no tour at all. A tour never changes a class, a count or a state.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use acan_intent::provenance::ModelProposal;
use acan_intent::{PlantView, Tour, TourView, Unsupported};
use acan_llm::ModelClient;

/// Why no tour could be made or admitted.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum TourError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// Asks the director model for a tour of `view`.
///
/// # Errors
///
/// Returns [`TourError`] when the model call fails or its reply is not a tour.
pub(crate) async fn direct<C: ModelClient>(
    _client: &C,
    _view: &PlantView,
) -> Result<ModelProposal<Tour>, TourError> {
    Err(Unsupported("T01: direct").into())
}

/// Validates a proposed tour against `view` and admits it as a directed tour.
///
/// # Errors
///
/// Returns [`TourError`] when any step breaks a tour rule.
pub(crate) fn admit(
    _view: &PlantView,
    _proposal: ModelProposal<Tour>,
) -> Result<TourView, TourError> {
    Err(Unsupported("T01: admit").into())
}

/// The deterministic tour of `view`: residue, then violations, then bounced, then fulfilled.
///
/// # Errors
///
/// Returns [`TourError`] when the view has nothing to visit.
pub(crate) fn deterministic(_view: &PlantView) -> Result<TourView, TourError> {
    Err(Unsupported("T01: deterministic").into())
}

#[cfg(test)]
mod tests {
    use acan_llm::Stub;

    use super::*;

    fn assert_unsupported<T: std::fmt::Debug>(result: Result<T, TourError>, operation: &str) {
        match result {
            Err(TourError::Unsupported(Unsupported(name))) => assert_eq!(name, operation),
            other => panic!("{operation} must fail closed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn every_tour_operation_fails_closed() {
        let directed: PlantView = serde_json::from_str(include_str!(
            "../../acan-intent/fixtures/plant-view-v1.json"
        ))
        .expect("the directed view decodes");
        let Some(TourView::Directed { model, tour }) = directed.tour else {
            panic!("the directed fixture carries a directed tour");
        };
        let plain: PlantView = serde_json::from_str(include_str!(
            "../../acan-intent/fixtures/plant-view-v1-plain.json"
        ))
        .expect("the plain view decodes");

        assert_unsupported(direct(&Stub::default(), &plain).await, "T01: direct");
        assert_unsupported(admit(&plain, ModelProposal::new(tour, model)), "T01: admit");
        assert_unsupported(deterministic(&plain), "T01: deterministic");
    }
}
