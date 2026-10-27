//! The git side of the server: task worktrees and landing on the protected branch.
//!
//! Worktree creation and landing both move refs and add worktrees in the user's repository, so
//! they take one lock and never overlap. A witness run takes the lock only to ensure its worktree
//! exists, then runs without it.

use acan_core::{CommitId, TaskId};
use acan_git::{LandOutcome, Repo, TaskWorktree};
use tokio::sync::Mutex;

/// The repository one server works in.
pub(crate) struct Workspace {
    repo: Repo,
    writes: Mutex<()>,
}

impl Workspace {
    pub(crate) fn new(repo: Repo) -> Self {
        Self {
            repo,
            writes: Mutex::new(()),
        }
    }

    /// The task's worktree, created at the protected head when it does not exist yet.
    ///
    /// # Errors
    ///
    /// Returns the git error when the worktree cannot be created or reattached.
    pub(crate) async fn worktree(&self, task: TaskId) -> Result<TaskWorktree, acan_git::Error> {
        let _writes = self.writes.lock().await;
        self.repo.worktrees().create(task).await
    }

    /// Lands `task` onto the protected branch, given the commit its gate tested.
    ///
    /// The land refuses when the task branch is no longer at `head`, so it never publishes work
    /// the gate did not test. A protected branch that moved is refreshed and the land retried
    /// once. A conflict is a failed land, not an error, and names the conflicting files.
    ///
    /// # Errors
    ///
    /// Returns the git error when git itself fails. A repeated land finds the task already merged
    /// and reports the same commit, so the caller may retry it.
    pub(crate) async fn land(
        &self,
        task: TaskId,
        head: &CommitId,
    ) -> Result<Result<CommitId, String>, acan_git::Error> {
        let _writes = self.writes.lock().await;
        let worktree = self.repo.worktrees().create(task).await?;
        let built = worktree.head().await?;
        if built != *head {
            return Ok(Err(format!(
                "the task branch is at {built}, not the commit that passed its gate ({head})"
            )));
        }
        let expected = self.repo.protected_head().await?;
        let outcome = match self.repo.land(task, &expected).await? {
            LandOutcome::HeadMoved { current } => self.repo.land(task, &current).await?,
            settled => settled,
        };
        let landed: Result<CommitId, String> = match outcome {
            LandOutcome::HeadMoved { .. } => {
                Err("the protected branch moved again while landing".to_owned())
            }
            settled => settled.into(),
        };
        Ok(landed)
    }
}
