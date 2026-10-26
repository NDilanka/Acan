//! The repository handle: worktree management and landing.

use std::path::{Path, PathBuf};

use acan_core::{CommitId, FilePath, TaskId};

use crate::git;
use crate::{Error, TaskWorktree};

/// A git repository Acan works in: task worktrees plus the protected branch.
///
/// All git access shells out to the `git` CLI, so worktrees and merges behave
/// exactly like the user's own git. Methods are async and each git call has a
/// timeout.
#[derive(Debug)]
pub struct Repo {
    root: PathBuf,
    protected: String,
    author_name: String,
    author_email: String,
}

impl Repo {
    /// Open the repository containing `path`, protecting the checked-out branch.
    ///
    /// # Errors
    /// Fails when `path` is not in a git repository, or `HEAD` is detached.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let root = repo_root(path.as_ref()).await?;
        // `symbolic-ref` names the branch even before its first commit; it
        // fails only when HEAD is detached.
        let args = ["symbolic-ref", "--short", "-q", "HEAD"];
        let out = git::run(&root, &args, &[]).await?;
        if !out.status.success() {
            return Err(Error::DetachedHead(root));
        }
        Self::finish_open(root, out.stdout.trim().to_owned()).await
    }

    /// Open the repository containing `path`, protecting `protected` instead of
    /// the checked-out branch.
    ///
    /// # Errors
    /// Fails when `path` is not in a git repository.
    pub async fn open_with_protected(
        path: impl AsRef<Path>,
        protected: impl Into<String>,
    ) -> Result<Self, Error> {
        let root = repo_root(path.as_ref()).await?;
        Self::finish_open(root, protected.into()).await
    }

    async fn finish_open(root: PathBuf, protected: String) -> Result<Self, Error> {
        let author_name = config(&root, "user.name")
            .await?
            .unwrap_or_else(|| "Acan".to_owned());
        let author_email = config(&root, "user.email")
            .await?
            .unwrap_or_else(|| "acan@localhost".to_owned());
        Ok(Self {
            root,
            protected,
            author_name,
            author_email,
        })
    }

    /// The repository's top-level directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The branch finished work lands on.
    #[must_use]
    pub fn protected_branch(&self) -> &str {
        &self.protected
    }

    /// The current head of the protected branch.
    ///
    /// # Errors
    /// Fails if the protected branch cannot be read.
    pub async fn protected_head(&self) -> Result<CommitId, Error> {
        let reference = format!("refs/heads/{}", self.protected);
        let out = git::ok(&self.root, &["rev-parse", &reference], &[]).await?;
        Ok(out.trim().parse()?)
    }

    /// The task worktrees of this repository.
    #[must_use]
    pub fn worktrees(&self) -> Worktrees<'_> {
        Worktrees { repo: self }
    }

    /// Merge the task branch into the protected branch, in queue order.
    ///
    /// `expected_head` is the protected head the caller decided the land at.
    /// The merge is computed with `git merge-tree` and committed without a
    /// checkout, then installed with `git update-ref` guarded by the old value,
    /// so a concurrent move of the protected branch is detected and reported as
    /// [`LandOutcome::HeadMoved`], never overwritten. The user's working tree
    /// is never touched; a checkout of the protected branch elsewhere keeps
    /// its files, and its status reflects the new head until the user updates
    /// it there. Fast-forwards when the task branch contains the protected
    /// head.
    ///
    /// Idempotent: when the task branch is already contained in the protected
    /// branch, the commit that landed it is reported again without merging
    /// twice.
    ///
    /// # Errors
    /// Fails when the task has no branch, or git itself fails. Merge conflicts
    /// and stale heads are outcomes, not errors.
    pub async fn land(&self, task: TaskId, expected_head: &CommitId) -> Result<LandOutcome, Error> {
        let branch_ref = format!("refs/heads/acan/{task}");
        let task_head = self.branch_head(&branch_ref).await?;
        let current = self.protected_head().await?;

        // A retried land after a crash finds the task branch already merged.
        // Report the commit that landed it instead of merging again.
        if self.is_ancestor(&task_head, &current).await? {
            let commit = self
                .landed_commit(expected_head, &task_head, &current)
                .await?;
            return Ok(LandOutcome::Landed { commit });
        }

        if current != *expected_head {
            return Ok(LandOutcome::HeadMoved { current });
        }

        let protected_ref = format!("refs/heads/{}", self.protected);

        if self.is_ancestor(&current, &task_head).await? {
            return self
                .move_protected(&protected_ref, &task_head, expected_head)
                .await;
        }

        let out = git::run(
            &self.root,
            &[
                "merge-tree",
                "--write-tree",
                "--name-only",
                "-z",
                current.as_str(),
                task_head.as_str(),
            ],
            &[],
        )
        .await?;
        let mut tokens = out.stdout.split('\0');
        let tree = tokens
            .next()
            .filter(|token| !token.is_empty())
            .ok_or_else(|| Error::BadOutput {
                args: "merge-tree --write-tree".to_owned(),
                detail: "missing tree id".to_owned(),
            })?;
        if !out.status.success() {
            // After the tree id, `-z --name-only` prints one conflicting file
            // per NUL-terminated token up to the empty token that ends the
            // section.
            let files = tokens
                .take_while(|token| !token.is_empty())
                .map(|token| Ok(token.parse::<FilePath>()?))
                .collect::<Result<Vec<FilePath>, Error>>()?;
            return Ok(LandOutcome::Conflict { files });
        }

        let message = format!("acan: land task {task}");
        let envs = self.author_env();
        let args = [
            "-c",
            "commit.gpgsign=false",
            "commit-tree",
            tree,
            "-p",
            current.as_str(),
            "-p",
            task_head.as_str(),
            "-m",
            &message,
        ];
        let merged: CommitId = git::ok(&self.root, &args, &envs).await?.trim().parse()?;
        self.move_protected(&protected_ref, &merged, expected_head)
            .await
    }

    /// Point the protected branch at `commit`, but only if it still sits at
    /// `expected_head`; a concurrent move is reported, never clobbered.
    async fn move_protected(
        &self,
        protected_ref: &str,
        commit: &CommitId,
        expected_head: &CommitId,
    ) -> Result<LandOutcome, Error> {
        let args = [
            "update-ref",
            protected_ref,
            commit.as_str(),
            expected_head.as_str(),
        ];
        let out = git::run(&self.root, &args, &[]).await?;
        if out.status.success() {
            return Ok(LandOutcome::Landed {
                commit: commit.clone(),
            });
        }
        let current = self.protected_head().await?;
        if current != *expected_head {
            return Ok(LandOutcome::HeadMoved { current });
        }
        // The ref did not move, yet the update failed: a real git error.
        Err(git::ensure_ok(out, &args).expect_err("status was not success"))
    }

    /// The commit a finished land recorded, for the retry path.
    async fn landed_commit(
        &self,
        expected_head: &CommitId,
        task_head: &CommitId,
        current: &CommitId,
    ) -> Result<CommitId, Error> {
        // A fast-forward land made the protected head the task head.
        if self.is_ancestor(expected_head, task_head).await? {
            return Ok(task_head.clone());
        }
        // A merge land committed one child of the expected head, and later
        // lands build on it, so it is the first first-parent commit after it.
        let range = format!("{expected_head}..{current}");
        let out = git::ok(
            &self.root,
            &["rev-list", "--first-parent", "--reverse", &range],
            &[],
        )
        .await?;
        let commit = out
            .lines()
            .next()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .ok_or_else(|| Error::BadOutput {
                args: format!("rev-list {range}"),
                detail: "no commit landed after the expected head".to_owned(),
            })?;
        Ok(commit.parse()?)
    }

    async fn is_ancestor(&self, older: &CommitId, newer: &CommitId) -> Result<bool, Error> {
        let args = [
            "merge-base",
            "--is-ancestor",
            older.as_str(),
            newer.as_str(),
        ];
        let out = git::run(&self.root, &args, &[]).await?;
        match out.status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => git::ensure_ok(out, &args).map(|_| false),
        }
    }

    async fn branch_head(&self, branch_ref: &str) -> Result<CommitId, Error> {
        let commitish = format!("{branch_ref}^{{commit}}");
        let args = ["rev-parse", "--verify", "--quiet", &commitish];
        let out = git::run(&self.root, &args, &[]).await?;
        if !out.status.success() {
            let task = branch_ref
                .strip_prefix("refs/heads/acan/")
                .unwrap_or(branch_ref);
            return Err(Error::UnknownTask(task.to_owned()));
        }
        Ok(out.stdout.trim().parse()?)
    }

    fn author_env(&self) -> [(String, String); 4] {
        [
            ("GIT_AUTHOR_NAME".to_owned(), self.author_name.clone()),
            ("GIT_AUTHOR_EMAIL".to_owned(), self.author_email.clone()),
            ("GIT_COMMITTER_NAME".to_owned(), self.author_name.clone()),
            ("GIT_COMMITTER_EMAIL".to_owned(), self.author_email.clone()),
        ]
    }
}

/// What [`Repo::land`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
#[non_exhaustive]
pub enum LandOutcome {
    /// The task landed; the protected branch now points at this commit.
    Landed {
        /// The new protected head: the task head on a fast-forward, else the
        /// merge commit.
        commit: CommitId,
    },
    /// The merge conflicts. The protected branch did not move.
    Conflict {
        /// The conflicting repo-relative paths.
        files: Vec<FilePath>,
    },
    /// The protected branch moved since `expected_head` was read. Nothing was
    /// merged; retry against `current`.
    HeadMoved {
        /// The protected head now.
        current: CommitId,
    },
}

impl From<LandOutcome> for Result<CommitId, String> {
    /// The shape [`acan_core::SystemCmd::LandResult`] carries.
    fn from(outcome: LandOutcome) -> Self {
        match outcome {
            LandOutcome::Landed { commit } => Ok(commit),
            LandOutcome::Conflict { files } => {
                let names: Vec<String> = files.iter().map(ToString::to_string).collect();
                Err(format!("merge conflict in {}", names.join(", ")))
            }
            LandOutcome::HeadMoved { current } => {
                Err(format!("protected branch moved to {current}"))
            }
        }
    }
}

/// The task worktrees of a [`Repo`], borrowed from it.
#[derive(Debug)]
pub struct Worktrees<'r> {
    repo: &'r Repo,
}

impl Worktrees<'_> {
    /// Create the worktree for `task` on branch `acan/<task>`, starting at the
    /// protected head.
    ///
    /// Idempotent: calling it again for a live worktree returns it unchanged.
    ///
    /// # Errors
    /// Fails when the slot holds a foreign directory, or git cannot create the
    /// worktree.
    pub async fn create(&self, task: TaskId) -> Result<TaskWorktree, Error> {
        let dir = self.dir_for(task);
        let branch = format!("acan/{task}");
        if dir.exists() {
            let current = git::ok(&dir, &["rev-parse", "--abbrev-ref", "HEAD"], &[]).await?;
            if current.trim() == branch {
                return Ok(TaskWorktree::new(dir, task));
            }
            return Err(Error::SlotTaken(dir));
        }
        self.ensure_excluded().await?;
        let dir_arg = path_str(&dir)?;
        if self.branch_exists(&branch).await? {
            // The branch survived an earlier removal; reattach it.
            git::ok(&self.repo.root, &["worktree", "add", dir_arg, &branch], &[]).await?;
        } else {
            let base = format!("refs/heads/{}", self.repo.protected);
            git::ok(
                &self.repo.root,
                &["worktree", "add", dir_arg, "-b", &branch, &base],
                &[],
            )
            .await?;
        }
        Ok(TaskWorktree::new(dir, task))
    }

    /// Remove the worktree and branch of `task`, including uncommitted work.
    ///
    /// Idempotent: removing a task with no worktree or branch is a no-op.
    ///
    /// # Errors
    /// Fails if git or the filesystem cannot remove the leftovers.
    pub async fn remove(&self, task: TaskId) -> Result<(), Error> {
        let dir = self.dir_for(task);
        let branch = format!("acan/{task}");
        if dir.exists() {
            let dir_arg = path_str(&dir)?;
            let out = git::run(
                &self.repo.root,
                &["worktree", "remove", "--force", dir_arg],
                &[],
            )
            .await?;
            if !out.status.success() {
                // Git lost track of it; drop the files directly.
                git::ok(&self.repo.root, &["worktree", "prune"], &[]).await?;
                if dir.exists() {
                    tokio::fs::remove_dir_all(&dir).await?;
                }
            }
        }
        if self.branch_exists(&branch).await? {
            git::ok(&self.repo.root, &["branch", "-D", &branch], &[]).await?;
        }
        Ok(())
    }

    /// Every live task worktree, sorted by task id.
    ///
    /// # Errors
    /// Fails if the worktree directory cannot be read.
    pub async fn list(&self) -> Result<Vec<TaskWorktree>, Error> {
        let base = self.repo.root.join(".acan").join("worktrees");
        let mut entries = match tokio::fs::read_dir(&base).await {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(Error::Io(e)),
        };
        let mut found = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            // A directory that is not a task id is not ours; leave it alone.
            if let Ok(task) = name.parse::<TaskId>() {
                found.push(TaskWorktree::new(entry.path(), task));
            }
        }
        found.sort_by(|a, b| a.task().cmp(b.task()));
        Ok(found)
    }

    fn dir_for(&self, task: TaskId) -> PathBuf {
        self.repo
            .root
            .join(".acan")
            .join("worktrees")
            .join(task.to_string())
    }

    async fn branch_exists(&self, branch: &str) -> Result<bool, Error> {
        let reference = format!("refs/heads/{branch}");
        let args = ["show-ref", "--verify", "--quiet", &reference];
        let out = git::run(&self.repo.root, &args, &[]).await?;
        match out.status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => git::ensure_ok(out, &args).map(|_| false),
        }
    }

    /// Worktrees live inside the repository, so keep them out of `git status`.
    async fn ensure_excluded(&self) -> Result<(), Error> {
        let out = git::ok(&self.repo.root, &["rev-parse", "--git-common-dir"], &[]).await?;
        let common = PathBuf::from(out.trim());
        let common = if common.is_absolute() {
            common
        } else {
            self.repo.root.join(common)
        };
        let exclude = common.join("info").join("exclude");
        let existing = tokio::fs::read_to_string(&exclude)
            .await
            .unwrap_or_default();
        if existing.lines().any(|line| line.trim() == "/.acan/") {
            return Ok(());
        }
        let mut content = existing;
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str("/.acan/\n");
        tokio::fs::write(&exclude, content).await?;
        Ok(())
    }
}

async fn repo_root(path: &Path) -> Result<PathBuf, Error> {
    let args = ["rev-parse", "--show-toplevel"];
    let out = git::run(path, &args, &[]).await.map_err(|e| match e {
        e @ (Error::Timeout(_) | Error::Spawn { .. }) => e,
        _ => Error::NotARepo(path.to_path_buf()),
    })?;
    if !out.status.success() {
        return Err(Error::NotARepo(path.to_path_buf()));
    }
    Ok(PathBuf::from(out.stdout.trim()))
}

/// A repository-local config value, `None` when unset.
async fn config(root: &Path, key: &str) -> Result<Option<String>, Error> {
    let out = git::run(root, &["config", key], &[]).await?;
    if !out.status.success() {
        return Ok(None);
    }
    let value = out.stdout.trim();
    Ok((!value.is_empty()).then(|| value.to_owned()))
}

fn path_str(path: &Path) -> Result<&str, Error> {
    path.to_str()
        .ok_or_else(|| Error::NonUtf8Path(path.to_path_buf()))
}

const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Repo>();
};
