//! Per-task git worktrees and queue-ordered landing onto the protected branch.
//!
//! Each agent task works in its own worktree under `.acan/worktrees/<task>` on
//! branch `acan/<task>`. When a task passes its gate, [`Repo::land`] merges the
//! gated commit into the protected branch and reports a plain [`LandOutcome`] the
//! server turns into `acan_core::SystemCmd::LandResult`.
//!
//! All git access shells out to the `git` CLI, so behavior matches the user's
//! own git, and every call has a timeout with stderr captured into [`Error`].

mod error;
mod git;
mod repo;
mod worktree;

pub use acan_core;

pub use error::Error;
pub use repo::{LandOutcome, Repo, Worktrees};
pub use worktree::{Author, TaskWorktree};
