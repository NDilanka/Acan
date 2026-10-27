//! The repository handle: worktree management and landing.

use std::path::{Path, PathBuf};

use acan_core::{CommitId, FilePath, Generation, TaskId};

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

    /// Merge `gated`, the commit a task's gate tested, into the protected branch.
    ///
    /// `gated` is merged as given, not the task branch's current head, so commits
    /// the agent makes after the gate never land untested. `expected_head` is the
    /// protected head the caller decided the land at. The merge is computed with
    /// `git merge-tree` and committed without a checkout, then installed with
    /// `git update-ref` guarded by the old value, so a concurrent move of the
    /// protected branch is detected and reported as [`LandOutcome::HeadMoved`],
    /// never overwritten.
    ///
    /// A checkout that has the protected branch checked out, the user's main one
    /// included, moves with the branch. A clean checkout gets the new files through
    /// its own `git merge --ff-only`. A checkout with uncommitted changes stops the
    /// land with [`LandOutcome::CheckoutDirty`]. An untracked or ignored file the land
    /// would overwrite stops it with [`LandOutcome::CheckoutBlocked`]. A checkout that
    /// switched branches between the decision and the merge is
    /// [`LandOutcome::CheckoutMoved`]. Nothing moves in any of those cases.
    ///
    /// Idempotent: when the protected branch already contains `gated`, the commit
    /// that landed it is reported again without merging twice.
    ///
    /// # Errors
    /// Fails when git itself fails, including when `gated` is not a commit of this
    /// repository. Conflicts, stale heads, a dirty or blocked checkout, and a
    /// checkout that switched branches are outcomes.
    pub async fn land(
        &self,
        task: TaskId,
        gated: &CommitId,
        expected_head: &CommitId,
    ) -> Result<LandOutcome, Error> {
        let current = self.protected_head().await?;

        // A retried land after a crash finds the task already merged. Report the
        // commit that landed it instead of merging again.
        if self.is_ancestor(gated, &current).await? {
            let commit = self.landing_commit(gated, &current).await?;
            return Ok(LandOutcome::Landed { commit });
        }
        if current != *expected_head {
            return Ok(LandOutcome::HeadMoved { current });
        }
        if self.is_ancestor(&current, gated).await? {
            return self.advance(gated, expected_head).await;
        }

        let out = git::run(
            &self.root,
            &[
                "merge-tree",
                "--write-tree",
                "--name-only",
                "-z",
                current.as_str(),
                gated.as_str(),
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
            gated.as_str(),
            "-m",
            &message,
        ];
        let merged: CommitId = git::ok(&self.root, &args, &envs).await?.trim().parse()?;
        self.advance(&merged, expected_head).await
    }

    /// Point the protected branch at `commit`, which descends from `expected_head`.
    async fn advance(
        &self,
        commit: &CommitId,
        expected_head: &CommitId,
    ) -> Result<LandOutcome, Error> {
        match self.checkout_of_protected().await? {
            Some(checkout) => {
                self.advance_checkout(&checkout, commit, expected_head)
                    .await
            }
            None => self.advance_ref(commit, expected_head).await,
        }
    }

    /// Move the protected ref only if it still sits at `expected_head`. Used when no
    /// checkout has the protected branch checked out.
    async fn advance_ref(
        &self,
        commit: &CommitId,
        expected_head: &CommitId,
    ) -> Result<LandOutcome, Error> {
        let protected_ref = format!("refs/heads/{}", self.protected);
        let args = [
            "update-ref",
            protected_ref.as_str(),
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

    /// Move the protected branch through the checkout that has it checked out.
    ///
    /// The checkout's own `merge --ff-only` moves the branch and its files in one
    /// step. The flags name the merge this crate means, so a configured `mergeOptions`
    /// cannot turn it into a squash that exits 0 with the branch unmoved, and hooks
    /// cannot run. A checkout that is no longer on the protected branch, or whose head
    /// moved, is reported before the merge. A refusal to overwrite local files is a
    /// land failure, not a git error.
    async fn advance_checkout(
        &self,
        checkout: &Path,
        commit: &CommitId,
        expected_head: &CommitId,
    ) -> Result<LandOutcome, Error> {
        let dirty = git::ok(
            checkout,
            &["status", "--porcelain", "--untracked-files=no"],
            &[],
        )
        .await?;
        if !dirty.trim().is_empty() {
            return Ok(LandOutcome::CheckoutDirty {
                path: checkout.to_path_buf(),
            });
        }
        // Re-checked immediately before the merge: the branch can switch, or its head
        // move, between the decision above and here.
        match self.checkout_branch(checkout).await? {
            Some(branch) if branch == self.protected => {}
            _ => {
                return Ok(LandOutcome::CheckoutMoved {
                    path: checkout.to_path_buf(),
                });
            }
        }
        let current = self.protected_head().await?;
        if current != *expected_head {
            return Ok(LandOutcome::HeadMoved { current });
        }
        let hooks = self.empty_hooks_dir().await?;
        let hooks_arg = format!("core.hooksPath={hooks}");
        let args = [
            "-c",
            hooks_arg.as_str(),
            "merge",
            "--ff-only",
            "--no-squash",
            "--no-autostash",
            "--no-overwrite-ignore",
            "--quiet",
            commit.as_str(),
        ];
        let out = git::run(checkout, &args, &[]).await?;
        if out.status.success() {
            // A configured merge option can make git exit 0 without moving the branch.
            // Only the postcondition says the land happened.
            if self.protected_head().await? == *commit {
                return Ok(LandOutcome::Landed {
                    commit: commit.clone(),
                });
            }
            return Err(Error::BadOutput {
                args: "merge --ff-only".to_owned(),
                detail: format!("merge exited 0 but the protected branch is not at {commit}"),
            });
        }
        if out.stderr.contains("would be overwritten by merge") {
            return Ok(LandOutcome::CheckoutBlocked {
                path: checkout.to_path_buf(),
            });
        }
        let current = self.protected_head().await?;
        if current != *expected_head {
            return Ok(LandOutcome::HeadMoved { current });
        }
        Err(git::ensure_ok(out, &args).expect_err("status was not success"))
    }

    /// The branch `checkout` has checked out, or `None` when its head is detached.
    async fn checkout_branch(&self, checkout: &Path) -> Result<Option<String>, Error> {
        let out = git::run(checkout, &["symbolic-ref", "--short", "-q", "HEAD"], &[]).await?;
        if !out.status.success() {
            return Ok(None);
        }
        Ok(Some(out.stdout.trim().to_owned()))
    }

    /// An empty hooks directory, so the automated merge runs no hooks.
    async fn empty_hooks_dir(&self) -> Result<String, Error> {
        let out = git::ok(&self.root, &["rev-parse", "--git-common-dir"], &[]).await?;
        let common = PathBuf::from(out.trim());
        let common = if common.is_absolute() {
            common
        } else {
            self.root.join(common)
        };
        let hooks = common.join("acan-hooks");
        tokio::fs::create_dir_all(&hooks).await?;
        path_str(&hooks).map(str::to_owned)
    }

    /// The checkout that has the protected branch checked out, the user's main
    /// checkout included.
    async fn checkout_of_protected(&self) -> Result<Option<PathBuf>, Error> {
        let out = git::ok(&self.root, &["worktree", "list", "--porcelain"], &[]).await?;
        let wanted = format!("branch refs/heads/{}", self.protected);
        let mut path: Option<PathBuf> = None;
        for line in out.lines() {
            if let Some(found) = line.strip_prefix("worktree ") {
                path = Some(PathBuf::from(found));
            } else if line == wanted {
                return Ok(path);
            }
        }
        Ok(None)
    }

    /// The commit that landed `gated` on the way to `head`: `gated` itself after a
    /// fast-forward, else the merge commit whose second parent it is. Falls back to
    /// `head` when neither sits on the first-parent history, as when the user merged
    /// the task by hand.
    async fn landing_commit(&self, gated: &CommitId, head: &CommitId) -> Result<CommitId, Error> {
        if gated == head {
            return Ok(head.clone());
        }
        // Only the commits after the gated one can be the merge that landed it.
        let range = format!("{gated}..{head}");
        let args = ["log", "--first-parent", "--format=%H %P", &range];
        let out = git::ok(&self.root, &args, &[]).await?;
        for line in out.lines() {
            let mut ids = line.split_whitespace();
            let Some(commit) = ids.next() else {
                continue;
            };
            // Parents after the first are the ones a landing merge adds.
            let landed =
                commit == gated.as_str() || ids.skip(1).any(|parent| parent == gated.as_str());
            if landed {
                return Ok(commit.parse()?);
            }
        }
        Ok(head.clone())
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
        /// The new protected head: the gated commit on a fast-forward, else the
        /// merge commit. On a retry, the commit that landed the task.
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
    /// The checkout of the protected branch has uncommitted changes to tracked
    /// files, so landing would overwrite them. Nothing moved; commit or stash there
    /// and land again.
    CheckoutDirty {
        /// The checkout that has the protected branch.
        path: PathBuf,
    },
    /// The checkout holds an untracked or ignored file the land would overwrite.
    /// Nothing moved, and the file is kept; move it and land again.
    CheckoutBlocked {
        /// The checkout that has the protected branch.
        path: PathBuf,
    },
    /// The checkout that had the protected branch switched away from it between the
    /// decision and the merge. Nothing moved.
    CheckoutMoved {
        /// The checkout that switched.
        path: PathBuf,
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
            LandOutcome::CheckoutDirty { path } => Err(format!(
                "{} has uncommitted changes; commit or stash to land",
                path.display()
            )),
            LandOutcome::CheckoutBlocked { path } => Err(format!(
                "{} has untracked files the land would overwrite; move them to land",
                path.display()
            )),
            LandOutcome::CheckoutMoved { path } => Err(format!(
                "{} left the protected branch; land again",
                path.display()
            )),
        }
    }
}

/// The task worktrees of a [`Repo`], borrowed from it.
#[derive(Debug)]
pub struct Worktrees<'r> {
    repo: &'r Repo,
}

impl Worktrees<'_> {
    /// Create the worktree for `task` at `generation`, on branch `acan/<task>`,
    /// starting at the protected head.
    ///
    /// A worktree already recorded at `generation` is returned unchanged. A worktree
    /// recorded at another generation is removed first, branch included, so a reclaim
    /// starts at the current protected head and keeps none of the earlier claim's
    /// commits. The generation is stored on the branch, so a restart can tell a live
    /// claim's worktree from a stale one.
    ///
    /// # Errors
    /// Fails when the slot holds a foreign directory, or git cannot create the
    /// worktree.
    pub async fn create(
        &self,
        task: TaskId,
        generation: Generation,
    ) -> Result<TaskWorktree, Error> {
        let dir = self.dir_for(task);
        let branch = format!("acan/{task}");
        if dir.exists() {
            let current = git::ok(&dir, &["rev-parse", "--abbrev-ref", "HEAD"], &[]).await?;
            if current.trim() != branch {
                return Err(Error::SlotTaken(dir));
            }
            if self.recorded_generation(&branch).await? == Some(generation) {
                return Ok(TaskWorktree::new(dir, task));
            }
            // The directory goes now so the new worktree can take its path. The branch stays
            // until the new one exists, so a failed create leaves the old claim recoverable.
            self.remove_worktree(task).await?;
        }
        let stale = self.branch_exists(&branch).await?
            && self.recorded_generation(&branch).await? != Some(generation);
        let stale_tip = if stale {
            self.branch_tip(&branch).await?
        } else {
            None
        };
        self.ensure_excluded().await?;
        let dir_arg = path_str(&dir)?;
        if let Some(tip) = &stale_tip {
            // The old branch goes only once the new worktree can take its name. A failure
            // here leaves the branch, so the old claim stays recoverable.
            self.delete_branch_at(&branch, tip).await?;
        }
        if self.branch_exists(&branch).await? {
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
        self.record_generation(&branch, generation).await?;
        Ok(TaskWorktree::new(dir, task))
    }

    /// Remove the worktree and branch of `task`, but only when the recorded generation
    /// is still `expected`.
    ///
    /// A newer generation recorded since the caller decided to remove is left in place,
    /// so a late removal never deletes a newer claim's worktree. A task with no recorded
    /// generation is removed, which clears leftovers from before generations were stored.
    ///
    /// # Errors
    /// Fails if git or the filesystem cannot remove the leftovers.
    pub async fn reset(&self, task: TaskId, expected: Generation) -> Result<(), Error> {
        let branch = format!("acan/{task}");
        if self
            .recorded_generation(&branch)
            .await?
            .is_some_and(|recorded| recorded != expected)
        {
            return Ok(());
        }
        // The generation can change between the check above and the removal, so the branch
        // delete itself is guarded by the tip read here. A newer claim moves that tip.
        let tip = self.branch_tip(&branch).await?;
        self.remove_worktree(task).await?;
        if let Some(tip) = tip {
            self.delete_branch_at(&branch, &tip).await?;
        }
        Ok(())
    }

    /// The tip of `branch`, when the branch exists.
    async fn branch_tip(&self, branch: &str) -> Result<Option<CommitId>, Error> {
        if !self.branch_exists(branch).await? {
            return Ok(None);
        }
        let reference = format!("refs/heads/{branch}");
        let out = git::ok(&self.repo.root, &["rev-parse", &reference], &[]).await?;
        Ok(Some(out.trim().parse()?))
    }

    /// Delete `branch` only if it still points at `tip`. A branch that moved is a newer claim's.
    ///
    /// A failed delete is ignored only when the observed state changed: the branch is gone, its
    /// tip moved, or its recorded generation changed. Any other failure surfaces, because the
    /// caller must not stamp a new generation over a branch it failed to delete.
    async fn delete_branch_at(&self, branch: &str, tip: &CommitId) -> Result<(), Error> {
        let reference = format!("refs/heads/{branch}");
        let observed = self.recorded_generation(branch).await?;
        let args = ["update-ref", "-d", reference.as_str(), tip.as_str()];
        let out = git::run(&self.repo.root, &args, &[]).await?;
        if out.status.success() {
            return Ok(());
        }
        // A failed delete of a branch that moved, or that a newer generation recreated at the
        // same commit, leaves that generation's branch in place.
        if !self.branch_exists(branch).await?
            || self.branch_tip(branch).await? != Some(tip.clone())
            || self.recorded_generation(branch).await? != observed
        {
            return Ok(());
        }
        git::ensure_ok(out, &args).map(drop)
    }

    /// The generation recorded on `task`'s branch, when the branch exists and records one.
    ///
    /// # Errors
    /// Fails if git cannot read the branch config.
    pub async fn generation(&self, task: TaskId) -> Result<Option<Generation>, Error> {
        let branch = format!("acan/{task}");
        self.recorded_generation(&branch).await
    }

    /// Remove the worktree and branch of `task`, including uncommitted work.
    ///
    /// Idempotent: removing a task with no worktree or branch is a no-op.
    ///
    /// # Errors
    /// Fails if git or the filesystem cannot remove the leftovers.
    pub async fn remove(&self, task: TaskId) -> Result<(), Error> {
        let branch = format!("acan/{task}");
        self.remove_worktree(task).await?;
        if self.branch_exists(&branch).await? {
            git::ok(&self.repo.root, &["branch", "-D", &branch], &[]).await?;
        }
        Ok(())
    }

    /// Remove the worktree directory of `task`, leaving its branch for the caller to delete.
    async fn remove_worktree(&self, task: TaskId) -> Result<(), Error> {
        let dir = self.dir_for(task);
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

    /// `branch.acan/<task>.acanGeneration`, set when the worktree is created.
    async fn recorded_generation(&self, branch: &str) -> Result<Option<Generation>, Error> {
        let key = format!("branch.{branch}.acanGeneration");
        let Some(raw) = config(&self.repo.root, &key).await? else {
            return Ok(None);
        };
        Ok(raw.parse::<u32>().ok().map(Generation::new))
    }

    async fn record_generation(&self, branch: &str, generation: Generation) -> Result<(), Error> {
        let key = format!("branch.{branch}.acanGeneration");
        let value = generation.get().to_string();
        git::ok(&self.repo.root, &["config", &key, &value], &[]).await?;
        Ok(())
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
