//! The portable receipt: plain JSON and deterministic pull request Markdown.
//!
//! Rendering is a pure function of the [`Receipt`]. The same receipt always renders to the
//! same bytes. A renderer never recomputes a class or a count; it prints the receipt's own.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use acan_intent::{Receipt, Unsupported};

/// Why a receipt could not be rendered.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum RenderError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// The receipt as plain JSON.
///
/// # Errors
///
/// Returns [`RenderError`] when the receipt cannot be serialized.
pub(crate) fn render_json(_receipt: &Receipt) -> Result<String, RenderError> {
    Err(Unsupported("R01: render_json").into())
}

/// The receipt as Markdown for a pull request: base and merged commit, probe and route
/// denominators, masks, errors, stale evidence, notes that are not enforced, bounce attempts
/// and uncovered routes.
///
/// # Errors
///
/// Returns [`RenderError`] when the receipt cannot be rendered.
pub(crate) fn render_markdown(_receipt: &Receipt) -> Result<String, RenderError> {
    Err(Unsupported("R01: render_markdown").into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_unsupported(result: Result<String, RenderError>, operation: &str) {
        match result {
            Err(RenderError::Unsupported(Unsupported(name))) => assert_eq!(name, operation),
            other => panic!("{operation} must fail closed, got {other:?}"),
        }
    }

    #[test]
    fn every_renderer_fails_closed() {
        let receipt: Receipt =
            serde_json::from_str(include_str!("../../acan-intent/fixtures/receipt.json"))
                .expect("the receipt fixture decodes");
        assert_unsupported(render_json(&receipt), "R01: render_json");
        assert_unsupported(render_markdown(&receipt), "R01: render_markdown");
    }
}
