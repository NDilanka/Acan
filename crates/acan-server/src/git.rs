//! The git side of the server: task worktrees and landing on the protected branch.
//!
//! Creating, removing, and landing move refs and worktrees in the user's repository, so
//! they take one lock and never overlap. A witness run takes its task's lock instead, so
//! two runs of one task never share a worktree directory, and takes the shared lock only
//! to make sure that worktree exists.

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};

use acan_core::{CommitId, TaskId};
use acan_git::{LandOutcome, Repo, TaskWorktree};
use tokio::sync::{Mutex, OwnedMutexGuard};

/// The repository one server works in.
pub(crate) struct Workspace {
    repo: Repo,
    writes: Mutex<()>,
    /// One lock per task. The table only grows, one small entry per task that ever ran a
    /// witness, which a hackathon-sized repository never makes large.
    task_locks: StdMutex<HashMap<TaskId, Arc<Mutex<()>>>>,
}

/// Why a land did not land.
#[derive(Debug)]
pub(crate) enum LandFailure {
    /// The land was decided against, and nothing moved. The reason goes to the origin, which
    /// reopens the task.
    Refused(String),
    /// Git failed, so whether the land happened is unknown. The job retries it.
    Git(acan_git::Error),
}

impl Workspace {
    pub(crate) fn new(repo: Repo) -> Self {
        Self {
            repo,
            writes: Mutex::new(()),
            task_locks: StdMutex::new(HashMap::new()),
        }
    }

    /// Holds `task`'s lock until the guard drops.
    pub(crate) async fn lock_task(&self, task: TaskId) -> OwnedMutexGuard<()> {
        let cell = {
            let mut locks = self.task_locks.lock().expect("lock poisoned");
            Arc::clone(locks.entry(task).or_default())
        };
        cell.lock_owned().await
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

    /// The tasks that have a worktree on disk.
    ///
    /// # Errors
    ///
    /// Returns the git error when the worktree directory cannot be read.
    pub(crate) async fn worktree_tasks(&self) -> Result<Vec<TaskId>, acan_git::Error> {
        let worktrees = self.repo.worktrees().list().await?;
        Ok(worktrees.iter().map(|worktree| *worktree.task()).collect())
    }

    /// Removes `task`'s worktree and branch, so the next claim starts from the protected head.
    ///
    /// The task lock keeps the removal out of a witness run that shares the worktree. The loop
    /// creates a claimed task's worktree only after this step finishes, so a reclaim that
    /// arrives while this waits never reuses the old worktree.
    ///
    /// # Errors
    ///
    /// Returns the git error when the leftovers cannot be removed.
    pub(crate) async fn reset(&self, task: TaskId) -> Result<(), acan_git::Error> {
        let _task = self.lock_task(task).await;
        let _writes = self.writes.lock().await;
        self.repo.worktrees().remove(task).await
    }

    /// Lands the `gated` commit of `task` onto the protected branch.
    ///
    /// The commit that passed the gate is what lands, whatever the task branch holds by now.
    /// A protected branch that moved is refreshed and the land retried once. A conflict, a
    /// dirty checkout, or a second move is a refusal: nothing moved, and the reason goes to
    /// the origin.
    ///
    /// # Errors
    ///
    /// Returns [`LandFailure::Git`] when git itself fails.
    pub(crate) async fn land(
        &self,
        task: TaskId,
        gated: &CommitId,
    ) -> Result<CommitId, LandFailure> {
        let _writes = self.writes.lock().await;
        let mut expected = self.repo.protected_head().await.map_err(LandFailure::Git)?;
        for _ in 0..2 {
            let outcome = self
                .repo
                .land(task, gated, &expected)
                .await
                .map_err(LandFailure::Git)?;
            match outcome {
                LandOutcome::HeadMoved { current } => expected = current,
                settled => {
                    let landed: Result<CommitId, String> = settled.into();
                    return landed.map_err(LandFailure::Refused);
                }
            }
        }
        Err(LandFailure::Refused(
            "the protected branch moved again while landing".to_owned(),
        ))
    }
}
