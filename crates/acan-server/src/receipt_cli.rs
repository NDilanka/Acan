//! Shadow receipts: `acan receipt --repo <clone> --base <ref> --head <ref>`.
//!
//! Runs the contrast between two refs of any clone that has a protected app contract and
//! returns the receipt, with no orders and no Acan workers. The receipt is the same pure
//! function of observations, masks, scope and rulings that gates an order.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use std::path::PathBuf;

use acan_intent::{Receipt, Unsupported};

/// Why no shadow receipt could be made.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum ShadowError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// The arguments of `acan receipt`.
#[derive(Debug, Clone, clap::Args)]
pub(crate) struct ShadowArgs {
    /// The clone to run. Its `.acan/app.json` is the protected app contract.
    #[arg(long)]
    pub(crate) repo: PathBuf,
    /// The base ref.
    #[arg(long)]
    pub(crate) base: String,
    /// The head ref, observed against the base.
    #[arg(long)]
    pub(crate) head: String,
}

/// Runs the contrast between `args.base` and `args.head` and builds the receipt.
///
/// # Errors
///
/// Returns [`ShadowError`] when either ref cannot be resolved, the app contract is missing, or
/// the contrast or receipt cannot be made.
pub(crate) async fn run(_args: &ShadowArgs) -> Result<Receipt, ShadowError> {
    Err(Unsupported("DF1: receipt").into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_shadow_receipt_fails_closed() {
        let dir = tempfile::tempdir().expect("temporary clone");
        let args = ShadowArgs {
            repo: dir.path().to_path_buf(),
            base: "main".to_owned(),
            head: "feature".to_owned(),
        };
        match run(&args).await {
            Err(ShadowError::Unsupported(Unsupported(name))) => assert_eq!(name, "DF1: receipt"),
            other => panic!("the shadow receipt must fail closed, got {other:?}"),
        }
    }
}
