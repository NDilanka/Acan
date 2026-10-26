//! Worktree lifecycle: create, commit, list, remove.

mod common;

use acan_core::FilePath;
use acan_git::Author;

use common::{git, init, task_id, task_with_files};

#[tokio::test]
async fn create_twice_returns_the_same_worktree() {
    let t = init().await;
    let task = task_id("t-1");

    let first = t.repo.worktrees().create(task).await.expect("first create");
    assert_eq!(first.branch(), "acan/t-1");
    assert!(first.path().ends_with(".acan/worktrees/t-1"));
    let head = first.head().await.expect("head");
    assert_eq!(head, t.repo.protected_head().await.expect("protected head"));

    let second = t
        .repo
        .worktrees()
        .create(task)
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
        .create(task_id("t-1"))
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
        .create(task_id("t-1"))
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
        .create(task_id("t-2"))
        .await
        .expect("create t-2");
    t.repo
        .worktrees()
        .create(task_id("t-1"))
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
        .create(task_id("t-1"))
        .await
        .expect("recreate");
    assert_eq!(
        worktree.head().await.expect("head"),
        t.repo.protected_head().await.expect("protected head"),
        "a fresh worktree starts at the protected head"
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
