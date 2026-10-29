//! The error type for all git I/O.

use std::path::PathBuf;

/// Every failure this crate can report.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The path is not inside a git repository.
    #[error("not a git repository: {0}")]
    NotARepo(PathBuf),
    /// `HEAD` is detached, so the current branch cannot name the protected branch.
    #[error("repository at {0} has a detached HEAD; open with an explicit protected branch")]
    DetachedHead(PathBuf),
    /// The worktree slot is occupied by a directory that is not on the task branch.
    #[error("worktree path exists on a different branch: {0}")]
    SlotTaken(PathBuf),
    /// A path this crate must hand to git is not valid UTF-8.
    #[error("path is not valid UTF-8: {0}")]
    NonUtf8Path(PathBuf),
    /// Git exited non-zero where no exit code carries meaning.
    #[error("`git {args}` failed (exit {code:?}): {stderr}")]
    Git {
        /// The arguments after `git -C <dir>`.
        args: String,
        /// The exit code, when the process exited normally.
        code: Option<i32>,
        /// Trimmed stderr.
        stderr: String,
    },
    /// Git did not finish within the timeout; the process was killed.
    #[error("`git {0}` timed out")]
    Timeout(String),
    /// Git could not be spawned at all.
    #[error("could not start `git {args}`: {source}")]
    Spawn {
        /// The arguments after `git -C <dir>`.
        args: String,
        /// The spawn failure.
        #[source]
        source: std::io::Error,
    },
    /// Git succeeded but its output did not match the shape this crate parses.
    #[error("unexpected output from `git {args}`: {detail}")]
    BadOutput {
        /// The arguments after `git -C <dir>`.
        args: String,
        /// What was wrong with the output.
        detail: String,
    },
    /// Filesystem I/O outside git itself (the exclude file, worktree leftovers).
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Git printed an id that `acan-core` rejects.
    #[error(transparent)]
    ParseId(#[from] acan_core::ParseIdError),
    /// Git printed a commit or path the `acan-intent` contract rejects, such as a path no
    /// portable checkout can hold.
    #[error(transparent)]
    Contract(#[from] acan_intent::InvalidValue),
    /// A manifest has no canonical JSON form.
    #[error(transparent)]
    Canonical(#[from] acan_intent::CanonicalError),
    /// A protected path in the protected head is a symlink or a submodule, which cannot be
    /// restored as a read-only file.
    #[error("protected path {path} in the protected head has mode {mode}, not a regular file")]
    ProtectedNotAFile {
        /// The path in the protected head.
        path: String,
        /// Its git mode.
        mode: String,
    },
    /// The prepared tree holds a symlink or a submodule, which a materialized run cannot
    /// carry byte for byte.
    #[error("{path} has mode {mode}; only regular files can be materialized")]
    UnsupportedEntry {
        /// The path in the prepared tree.
        path: String,
        /// Its git mode.
        mode: String,
    },
    /// Two paths of the prepared tree differ only in case, so a case-insensitive
    /// filesystem would hold one file where the tree has two.
    #[error("{0} collides with another path of the prepared tree when case is ignored")]
    CaseCollision(String),
    /// The directory to materialize into exists and is not empty.
    #[error("materialize target is not empty: {0}")]
    TargetNotEmpty(PathBuf),
    /// After restoring, the protected files on disk differ from the protected head's.
    #[error("protected inputs in {0} do not match the protected head after restoring")]
    OverlayMismatch(PathBuf),
}
