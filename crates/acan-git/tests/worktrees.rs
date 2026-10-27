//! Worktree lifecycle: create, commit, list, remove.

mod common;

use acan_core::{FilePath, Generation};
use acan_git::Author;

use common::{commit_files, git, init, task_id, task_with_files};

#[tokio::test]
async fn create_twice_returns_the_same_worktree() {
    let t = init().await;
    let task = task_id("t-1");

    let first = t
        .repo
        .worktrees()
        .create(task, Generation::new(1))
        .await
        .expect("first create");
    assert_eq!(first.branch(), "acan/t-1");
    assert!(first.path().ends_with(".acan/worktrees/t-1"));
    let head = first.head().await.expect("head");
    assert_eq!(head, t.repo.protected_head().await.expect("protected head"));

    let second = t
        .repo
        .worktrees()
        .create(task, Generation::new(1))
        .await
        .expect("second create");
    assert_eq!(first, second);

    let listed = git(&t.root, &["worktree", "list"]);
    assert_eq!(
        listed.lines().count(),
        2,
        "main plus the task worktree: {listed}"
    );
}

#[tokio::test]
async fn the_worktree_directory_stays_out_of_git_status() {
    let t = init().await;
    t.repo
        .worktrees()
        .create(task_id("t-1"), Generation::new(1))
        .await
        .expect("create");
    let status = git(&t.root, &["status", "--porcelain"]);
    assert_eq!(status, "", "worktree creation must not dirty the repo");
}

#[tokio::test]
async fn commit_all_and_changed_files() {
    let t = init().await;
    let base = t.repo.protected_head().await.expect("base");
    let worktree = t
        .repo
        .worktrees()
        .create(task_id("t-1"), Generation::new(1))
        .await
        .expect("create");

    std::fs::create_dir_all(worktree.path().join("src")).expect("mkdir");
    std::fs::write(worktree.path().join("src/new.txt"), "task work\n").expect("write");
    let head = worktree
        .commit_all("add new.txt", &Author::new("Agent", "agent@example.com"))
        .await
        .expect("commit");
    assert_ne!(head, base);

    let changed = worktree.changed_files(&base).await.expect("changed files");
    let want: Vec<FilePath> = vec!["src/new.txt".parse().expect("path")];
    assert_eq!(changed, want);

    let again = worktree
        .commit_all("nothing to do", &Author::new("Agent", "agent@example.com"))
        .await
        .expect("empty commit is a no-op");
    assert_eq!(again, head, "no changes must not stack an empty commit");
}

#[tokio::test]
async fn changed_files_are_repo_relative_with_forward_slashes() {
    let t = init().await;
    let base = t.repo.protected_head().await.expect("base");
    let worktree = task_with_files(&t.repo, "t-1", &[("a/b/c.txt", "deep\n")]).await;
    let changed = worktree.changed_files(&base).await.expect("changed files");
    assert_eq!(
        changed,
        vec!["a/b/c.txt".parse::<FilePath>().expect("path")]
    );
}

#[tokio::test]
async fn list_returns_every_live_task_worktree_sorted() {
    let t = init().await;
    t.repo
        .worktrees()
        .create(task_id("t-2"), Generation::new(1))
        .await
        .expect("create t-2");
    t.repo
        .worktrees()
        .create(task_id("t-1"), Generation::new(1))
        .await
        .expect("create t-1");

    let listed = t.repo.worktrees().list().await.expect("list");
    let tasks: Vec<String> = listed.iter().map(|w| w.task().to_string()).collect();
    assert_eq!(tasks, vec!["t-1".to_owned(), "t-2".to_owned()]);
    assert!(listed.iter().all(|w| w.branch().starts_with("acan/")));
}

#[tokio::test]
async fn remove_cleans_the_worktree_and_branch() {
    let t = init().await;
    let worktree = task_with_files(&t.repo, "t-1", &[("junk.txt", "x\n")]).await;
    let path = worktree.path().to_path_buf();

    t.repo
        .worktrees()
        .remove(task_id("t-1"))
        .await
        .expect("remove");
    assert!(!path.exists(), "worktree directory removed");
    let branches = git(&t.root, &["branch", "--list", "acan/t-1"]);
    assert_eq!(branches, "", "task branch removed");

    t.repo
        .worktrees()
        .remove(task_id("t-1"))
        .await
        .expect("removing twice is a no-op");
    let listed = t.repo.worktrees().list().await.expect("list");
    assert!(listed.is_empty());
}

#[tokio::test]
async fn a_removed_task_can_be_created_again() {
    let t = init().await;
    task_with_files(&t.repo, "t-1", &[("junk.txt", "x\n")]).await;
    t.repo
        .worktrees()
        .remove(task_id("t-1"))
        .await
        .expect("remove");

    let worktree = t
        .repo
        .worktrees()
        .create(task_id("t-1"), Generation::new(1))
        .await
        .expect("recreate");
    assert_eq!(
        worktree.head().await.expect("head"),
        t.repo.protected_head().await.expect("protected head"),
        "a fresh worktree starts at the protected head"
    );
}

#[tokio::test]
async fn a_new_generation_resets_the_worktree_to_the_protected_head() {
    let t = init().await;
    let task = task_id("t-1");
    let old = task_with_files(&t.repo, "t-1", &[("old.txt", "old claim\n")]).await;
    let old_head = old.head().await.expect("old head");
    let protected = t.repo.protected_head().await.expect("protected head");
    assert_ne!(old_head, protected, "the old claim has its own commit");

    let fresh = t
        .repo
        .worktrees()
        .create(task, Generation::new(2))
        .await
        .expect("create at the new generation");
    assert_eq!(
        fresh.head().await.expect("fresh head"),
        protected,
        "a new generation starts at the protected head"
    );
    assert!(
        !fresh.path().join("old.txt").exists(),
        "the old claim's file is gone"
    );
    let history = git(fresh.path(), &["rev-list", "--count", "HEAD"]);
    assert_eq!(history, "1", "none of the old claim's commits remain");
    assert_eq!(
        t.repo.worktrees().generation(task).await.expect("recorded"),
        Some(Generation::new(2))
    );
}

#[tokio::test]
async fn a_stale_reset_does_not_delete_a_newer_generation() {
    let t = init().await;
    let task = task_id("t-1");
    task_with_files(&t.repo, "t-1", &[("old.txt", "old claim\n")]).await;
    let fresh = t
        .repo
        .worktrees()
        .create(task, Generation::new(2))
        .await
        .expect("newer generation");
    let head = fresh.head().await.expect("newer head");

    t.repo
        .worktrees()
        .reset(task, Generation::new(1))
        .await
        .expect("stale reset is a no-op");
    assert!(fresh.path().is_dir(), "the newer worktree stays");
    assert_eq!(
        git(&t.root, &["rev-parse", "acan/t-1"]),
        head.to_string(),
        "the newer branch stays"
    );

    t.repo
        .worktrees()
        .reset(task, Generation::new(2))
        .await
        .expect("the matching reset removes it");
    assert!(!fresh.path().exists());
    assert_eq!(git(&t.root, &["branch", "--list", "acan/t-1"]), "");
}

#[tokio::test]
async fn a_recorded_generation_survives_reopening_the_repository() {
    let t = init().await;
    let task = task_id("t-1");
    t.repo
        .worktrees()
        .create(task, Generation::new(4))
        .await
        .expect("create");
    let reopened = acan_git::Repo::open(&t.root).await.expect("reopen");
    assert_eq!(
        reopened
            .worktrees()
            .generation(task)
            .await
            .expect("recorded"),
        Some(Generation::new(4)),
        "the generation is stored on the branch, not in memory"
    );
}

#[tokio::test]
async fn open_rejects_a_non_repository() {
    let dir = tempfile::tempdir().expect("tempdir");
    let err = acan_git::Repo::open(dir.path())
        .await
        .expect_err("not a repo");
    assert!(matches!(err, acan_git::Error::NotARepo(_)), "{err:?}");
}

#[tokio::test]
async fn fast_forward_moves_a_worktree_with_no_work_of_its_own() {
    let t = init().await;
    let worktree = t
        .repo
        .worktrees()
        .create(task_id("t-1"), Generation::new(1))
        .await
        .expect("create");
    let landed = commit_files(&t.root, &[("shared.txt", "landed\n")], "land");
    let to = landed.parse().expect("commit id");

    assert!(worktree.fast_forward(&to).await.expect("fast-forward"));
    assert_eq!(worktree.head().await.expect("head"), to);
    assert_eq!(
        std::fs::read_to_string(worktree.path().join("shared.txt"))
            .expect("read")
            .replace('\r', ""),
        "landed\n",
        "the working tree carries the landed file"
    );
    assert!(
        worktree.fast_forward(&to).await.expect("again"),
        "a worktree already at the commit stays there"
    );
}

#[tokio::test]
async fn fast_forward_leaves_a_worktree_with_its_own_work() {
    let t = init().await;
    let committed = task_with_files(&t.repo, "t-1", &[("task.txt", "task\n")]).await;
    let dirty = t
        .repo
        .worktrees()
        .create(task_id("t-2"), Generation::new(1))
        .await
        .expect("create");
    std::fs::write(dirty.path().join("shared.txt"), "unsaved\n").expect("write");
    let landed = commit_files(&t.root, &[("shared.txt", "landed\n")], "land");
    let to = landed.parse().expect("commit id");

    let committed_head = committed.head().await.expect("head");
    assert!(!committed.fast_forward(&to).await.expect("checked"));
    assert_eq!(
        committed.head().await.expect("head"),
        committed_head,
        "a worktree with its own commit is not moved"
    );
    let dirty_head = dirty.head().await.expect("head");
    assert!(!dirty.fast_forward(&to).await.expect("checked"));
    assert_eq!(dirty.head().await.expect("head"), dirty_head);
    assert_eq!(
        std::fs::read_to_string(dirty.path().join("shared.txt")).expect("read"),
        "unsaved\n",
        "an uncommitted change is kept"
    );
}
