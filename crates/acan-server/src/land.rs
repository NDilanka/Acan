//! The final gate and installation of the tested merged commit.
//!
//! Only the prepared merged commit a receipt was built for may land, and only while every field
//! of its gate context still matches. A moved protected head, a newer ruling log or any other
//! changed input forces a new preparation and a new run. An error never enqueues an install.
//!
//! Every function here returns [`Unsupported`] until its owner implements it.

use acan_intent::{CommitSha, GateContext, Receipt, Unsupported};

/// Why a candidate may not land.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum LandError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
}

/// Proof that the final gate passed for one merged commit on one protected head. Only
/// [`final_gate`] makes one, so [`install`] cannot be reached without it.
#[derive(Debug)]
pub(crate) struct Clearance {
    merged_commit: CommitSha,
    protected_head: CommitSha,
}

/// Checks that `receipt` passes and was built for exactly `current`.
///
/// # Errors
///
/// Returns [`LandError`] when the receipt does not pass or any context field differs.
pub(crate) fn final_gate(
    _receipt: &Receipt,
    _current: &GateContext,
) -> Result<Clearance, LandError> {
    Err(Unsupported("I01: final_gate").into())
}

/// Fast-forwards the protected branch from the cleared head to the cleared merged commit.
///
/// # Errors
///
/// Returns [`LandError`] when the protected head has moved or the update fails.
pub(crate) async fn install(
    _repo: &acan_git::Repo,
    _clearance: Clearance,
) -> Result<CommitSha, LandError> {
    Err(Unsupported("I01: install").into())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    fn assert_unsupported<T: std::fmt::Debug>(result: Result<T, LandError>, operation: &str) {
        match result {
            Err(LandError::Unsupported(Unsupported(name))) => assert_eq!(name, operation),
            other => panic!("{operation} must fail closed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn the_final_gate_and_install_fail_closed() {
        let receipt: Receipt =
            serde_json::from_str(include_str!("../../acan-intent/fixtures/receipt.json"))
                .expect("the receipt fixture decodes");
        let context = &receipt.prepared.context;
        assert_unsupported(final_gate(&receipt, context), "I01: final_gate");

        let dir = tempfile::tempdir().expect("temporary repository");
        let status = Command::new("git")
            .arg("-C")
            .arg(dir.path())
            .args(["init", "-q", "-b", "main"])
            .status()
            .expect("git runs");
        assert!(status.success(), "git init succeeds");
        let repo = acan_git::Repo::open(dir.path())
            .await
            .expect("the repository opens");
        let clearance = Clearance {
            merged_commit: context.merged_commit.clone(),
            protected_head: context.protected_head.clone(),
        };
        assert_unsupported(install(&repo, clearance).await, "I01: install");
    }
}
