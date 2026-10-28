//! Predicate evaluation and the save check for proposed rulings.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use crate::compare::Contrast;
use crate::error::Unsupported;
use crate::expr::Predicate;
use crate::hash::ContrastExampleHash;
use crate::provenance::CoreDerived;
use crate::verdict::{CheckResult, Comparison};

/// Why a predicate could not be evaluated. Never a pass.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PredicateError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// A retained ruling example, rebuilt from storage and checked against its manifest by
/// [`crate::admit::retained_example`]. Nothing else constructs one.
#[derive(Debug, Clone, Copy)]
pub struct RetainedExample<'a> {
    hash: ContrastExampleHash,
    contrast: Contrast<'a>,
    comparison: &'a CoreDerived<Comparison>,
}

impl<'a> RetainedExample<'a> {
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "P01 admits examples through admit.rs")
    )]
    pub(crate) fn admitted(
        hash: ContrastExampleHash,
        contrast: Contrast<'a>,
        comparison: &'a CoreDerived<Comparison>,
    ) -> Self {
        Self {
            hash,
            contrast,
            comparison,
        }
    }

    /// The manifest's hash, which a decision lists.
    #[must_use]
    pub fn hash(&self) -> ContrastExampleHash {
        self.hash
    }

    /// The rebuilt contrast.
    #[must_use]
    pub fn contrast(&self) -> &Contrast<'a> {
        &self.contrast
    }

    /// Its comparison.
    #[must_use]
    pub fn comparison(&self) -> &'a CoreDerived<Comparison> {
        self.comparison
    }
}

/// Evaluates `predicate` over one contrast and its comparison.
///
/// An `equals_base` with a [`crate::BaseAnchor::Fixed`] anchor compares the candidate with the
/// base runs of the example in `anchors` whose hash it names; a [`crate::BaseAnchor::Fresh`]
/// anchor uses `contrast`'s own base runs. A fixed anchor missing from `anchors` is
/// [`CheckResult::Unknown`], never a pass and never a silent switch to the fresh base.
///
/// # Errors
///
/// Returns [`PredicateError`] when evaluation fails.
pub fn evaluate(
    _predicate: &Predicate,
    _contrast: &Contrast<'_>,
    _comparison: &CoreDerived<Comparison>,
    _anchors: &[RetainedExample<'_>],
) -> Result<CheckResult, PredicateError> {
    Err(Unsupported("P01: evaluate").into())
}

/// Succeeds only if `predicate` passes on every accepted example and fails on every rejected
/// one, and `ruled_on` is among them and judged the same way. A proposed ruling is saved only
/// after this succeeds; fixed anchors resolve among the accepted and rejected examples.
///
/// # Errors
///
/// Returns [`PredicateError`] when the predicate misjudges an example, `ruled_on` is not listed,
/// or an example cannot be evaluated.
pub fn check_examples(
    _predicate: &Predicate,
    _ruled_on: &RetainedExample<'_>,
    _accepts: &[RetainedExample<'_>],
    _rejects: &[RetainedExample<'_>],
) -> Result<(), PredicateError> {
    Err(Unsupported("P01: check_examples").into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::tests::{admitted_run, corpus};
    use crate::hash::ContentHash;
    use crate::scope::Scope;

    #[test]
    fn every_predicate_stub_fails_closed() {
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
        let predicate = &scope.body.may[0].acceptance.first().predicate;
        let example = RetainedExample::admitted(
            ContentHash::of_bytes(b"example").into(),
            contrast,
            &comparison,
        );
        assert_eq!(
            evaluate(predicate, &contrast, &comparison, &[]),
            Err(PredicateError::Unsupported(Unsupported("P01: evaluate")))
        );
        assert_eq!(
            check_examples(predicate, &example, &[example], &[]),
            Err(PredicateError::Unsupported(Unsupported(
                "P01: check_examples"
            )))
        );
    }
}
