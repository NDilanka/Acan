//! Predicate evaluation and the save check for proposed rulings.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use crate::compare::Contrast;
use crate::error::Unsupported;
use crate::expr::Predicate;
use crate::verdict::{CheckResult, Comparison};

/// Why a predicate could not be evaluated. Never a pass.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PredicateError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// Evaluates `predicate` over one contrast and its comparison.
///
/// # Errors
///
/// Returns [`PredicateError`] when evaluation fails.
pub fn evaluate(
    _predicate: &Predicate,
    _contrast: &Contrast<'_>,
    _comparison: &Comparison,
) -> Result<CheckResult, PredicateError> {
    Err(Unsupported("P01: evaluate").into())
}

/// Succeeds only if `predicate` passes on every accepted contrast and fails on every rejected
/// one. A proposed ruling is saved only after this succeeds.
///
/// # Errors
///
/// Returns [`PredicateError`] when the predicate misjudges an example or cannot be evaluated.
pub fn check_examples(
    _predicate: &Predicate,
    _accepts: &[(Contrast<'_>, &Comparison)],
    _rejects: &[(Contrast<'_>, &Comparison)],
) -> Result<(), PredicateError> {
    Err(Unsupported("P01: check_examples").into())
}
