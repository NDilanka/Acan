//! A task's worktree: one branch checked out under `.acan/worktrees/`.

use std::path::{Path, PathBuf};

use acan_core::{CommitId, FilePath, TaskId};

use crate::Error;
use crate::git;

/// Who authors a commit made through this crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Author {
    /// The display name.
    pub name: String,
    /// The email address.
    pub email: String,
}

impl Author {
    /// An author from a name and an email.
    #[must_use]
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
        }
    }
}

/// A checked-out worktree for one task, on branch `acan/<task>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskWorktree {
    path: PathBuf,
    branch: String,
    task: TaskId,
}

impl TaskWorktree {
    pub(crate) fn new(path: PathBuf, task: TaskId) -> Self {
        let branch = format!("acan/{task}");
        Self { path, branch, task }
    }

    /// The worktree's directory.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The task branch, `acan/<task>`.
    #[must_use]
    pub fn branch(&self) -> &str {
        &self.branch
    }

    /// The task this worktree belongs to.
    #[must_use]
    pub fn task(&self) -> &TaskId {
        &self.task
    }

    /// The worktree's `HEAD`.
    ///
    /// # Errors
    /// Fails if git cannot read the worktree's head.
    pub async fn head(&self) -> Result<CommitId, Error> {
        let out = git::ok(&self.path, &["rev-parse", "HEAD"], &[]).await?;
        Ok(out.trim().parse()?)
    }

    /// Stage everything and commit it as `author`, returning the new head.
    ///
    /// With no changes the worktree is left alone and the current head is
    /// returned, so a retry after a successful commit does not stack an empty
    /// commit.
    ///
    /// # Errors
    /// Fails if git cannot stage or commit.
    pub async fn commit_all(&self, message: &str, author: &Author) -> Result<CommitId, Error> {
        let status = git::ok(&self.path, &["status", "--porcelain"], &[]).await?;
        if !status.trim().is_empty() {
            git::ok(&self.path, &["add", "-A"], &[]).await?;
            let name = format!("user.name={}", author.name);
            let email = format!("user.email={}", author.email);
            // Signing is off: an unattended harness cannot answer a key prompt.
            let args = [
                "-c",
                name.as_str(),
                "-c",
                email.as_str(),
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-q",
                "-m",
                message,
            ];
            git::ok(&self.path, &args, &[]).await?;
        }
        self.head().await
    }

    /// Fast-forward this worktree to `to` when it has no work of its own: no
    /// uncommitted change, and a head that `to` already contains. Returns whether
    /// the worktree now sits at `to`. A worktree with work of its own is left
    /// alone, so nothing an agent wrote is moved or lost.
    ///
    /// # Errors
    /// Fails if git cannot read the worktree or move it.
    pub async fn fast_forward(&self, to: &CommitId) -> Result<bool, Error> {
        let status = git::ok(&self.path, &["status", "--porcelain"], &[]).await?;
        if !status.trim().is_empty() {
            return Ok(false);
        }
        let head = self.head().await?;
        if head == *to {
            return Ok(true);
        }
        let args = ["merge-base", "--is-ancestor", head.as_str(), to.as_str()];
        let out = git::run(&self.path, &args, &[]).await?;
        match out.status.code() {
            Some(0) => {}
            Some(1) => return Ok(false),
            _ => return git::ensure_ok(out, &args).map(|_| false),
        }
        git::ok(&self.path, &["merge", "--ff-only", "-q", to.as_str()], &[]).await?;
        Ok(true)
    }

    /// Files this worktree changed relative to `base`, as repo-relative
    /// forward-slash paths, sorted as git prints them.
    ///
    /// The diff runs from the merge base, so work landed on the protected
    /// branch since the task started does not appear.
    ///
    /// # Errors
    /// Fails if git cannot diff, or a printed path is not a valid [`FilePath`].
    pub async fn changed_files(&self, base: &CommitId) -> Result<Vec<FilePath>, Error> {
        let range = format!("{}...HEAD", base.as_str());
        let out = git::ok(&self.path, &["diff", "--name-only", "-z", &range], &[]).await?;
        out.split('\0')
            .filter(|token| !token.is_empty())
            .map(|token| Ok(token.parse::<FilePath>()?))
            .collect()
    }
}
