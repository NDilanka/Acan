//! Shared helpers: real git repositories in temporary directories.
// Each test binary compiles this module and uses a subset of it.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use acan_core::{Generation, TaskId};
use acan_git::{Author, Repo, TaskWorktree};

/// A temporary repository opened as a [`Repo`].
pub struct TestRepo {
    pub root: PathBuf,
    pub repo: Repo,
    _dir: tempfile::TempDir,
}

/// Run git synchronously, failing the test on a non-zero exit.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

/// A repository on branch `main` with one commit, opened as a [`Repo`].
pub async fn init() -> TestRepo {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_path_buf();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.name", "Tester"]);
    git(&root, &["config", "user.email", "tester@example.com"]);
    std::fs::write(root.join("shared.txt"), "base\n").expect("write base");
    git(&root, &["add", "."]);
    git(&root, &["commit", "-q", "-m", "base"]);
    let repo = Repo::open(&root).await.expect("open repo");
    TestRepo {
        root,
        repo,
        _dir: dir,
    }
}

pub fn task_id(raw: &str) -> TaskId {
    raw.parse().expect("valid task id")
}

/// Write `files` into `root`, commit on the current branch, return the head.
pub fn commit_files(root: &Path, files: &[(&str, &str)], message: &str) -> String {
    for (path, content) in files {
        let full = root.join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&full, content).expect("write file");
    }
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", message]);
    git(root, &["rev-parse", "HEAD"])
}

/// Create the task worktree and commit `files` in it.
pub async fn task_with_files(repo: &Repo, task: &str, files: &[(&str, &str)]) -> TaskWorktree {
    let worktree = repo
        .worktrees()
        .create(task_id(task), Generation::new(1))
        .await
        .expect("create worktree");
    for (path, content) in files {
        let full = worktree.path().join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&full, content).expect("write file");
    }
    worktree
        .commit_all("task work", &Author::new("Agent", "agent@example.com"))
        .await
        .expect("commit task work");
    worktree
}
