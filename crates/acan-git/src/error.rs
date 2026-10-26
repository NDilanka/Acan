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
    /// The task has no worktree branch to land.
    #[error("task has no worktree branch: {0}")]
    UnknownTask(String),
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
}
