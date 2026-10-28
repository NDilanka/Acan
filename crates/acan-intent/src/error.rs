//! The error every unimplemented contract operation returns.

/// An operation whose owning unit has not landed yet. Names the unit and the operation, such as
/// `P01: compare`. Callers treat it like any other failure: it is never a pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("unsupported: {0}")]
pub struct Unsupported(pub &'static str);
