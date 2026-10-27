//! Landing task work onto the protected branch.

mod common;

use std::path::Path;

use acan_core::{CommitId, FilePath};
use acan_git::{Author, LandOutcome};

use common::{commit_files, git, init, task_id, task_with_files};

fn head(raw: &str) -> CommitId {
    raw.parse().expect("commit id")
}

fn canonical(path: &Path) -> std::path::PathBuf {
    path.canonicalize().expect("path exists")
}

/// The file's text with LF endings. A checkout with `core.autocrlf` writes CRLF.
fn read_text(path: &Path) -> String {
    std::fs::read_to_string(path)
        .expect("file exists")
        .replace("\r\n", "\n")
}

#[tokio::test]
async fn land_fast_forwards_onto_the_gated_commit() {
    let t = init().await;
    let expected = t.repo.protected_head().await.expect("protected head");
    let worktree = task_with_files(&t.repo, "t-1", &[("feat.txt", "feature\n")]).await;
    let gated = worktree.head().await.expect("gated head");

    let outcome = t
        .repo
        .land(task_id("t-1"), &gated, &expected)
        .await
        .expect("land");
    let LandOutcome::Landed { commit } = outcome else {
        panic!("expected Landed, got {outcome:?}");
    };
    assert_eq!(commit, gated, "a fast-forward lands the gated commit");
    assert_eq!(
        t.repo.protected_head().await.expect("protected head"),
        gated
    );
    // Main is checked out here and clean, so its files follow the land.
    assert_eq!(read_text(&t.root.join("feat.txt")), "feature\n");
    assert_eq!(git(&t.root, &["status", "--porcelain"]), "");

    // Landing again with the protected head as expected finds the task contained.
    let again = t
        .repo
        .land(task_id("t-1"), &gated, &gated)
        .await
        .expect("re-land");
    assert_eq!(
        again,
        LandOutcome::Landed { commit },
        "landing again reports the same commit"
    );
}

#[tokio::test]
async fn land_merges_only_the_gated_commit() {
    let t = init().await;
    let expected = t.repo.protected_head().await.expect("protected head");
    let worktree = task_with_files(&t.repo, "t-1", &[("feat.txt", "gated\n")]).await;
    let gated = worktree.head().await.expect("gated head");
    // The agent commits again after the gate tested `gated`.
    std::fs::write(worktree.path().join("late.txt"), "untested\n").expect("write late file");
    let late = worktree
        .commit_all("later work", &Author::new("Agent", "agent@example.com"))
        .await
        .expect("commit later work");

    let outcome = t
        .repo
        .land(task_id("t-1"), &gated, &expected)
        .await
        .expect("land");
    assert_eq!(
        outcome,
        LandOutcome::Landed {
            commit: gated.clone()
        }
    );
    assert_eq!(
        t.repo.protected_head().await.expect("protected head"),
        gated
    );
    assert!(
        !t.root.join("late.txt").exists(),
        "work after the gate never reaches the protected branch"
    );
    assert_eq!(
        git(&t.root, &["rev-parse", "acan/t-1"]),
        late.to_string(),
        "the task branch keeps the later commit"
    );
}

#[tokio::test]
async fn land_merges_non_overlapping_work() {
    let t = init().await;
    let worktree = task_with_files(&t.repo, "t-1", &[("task.txt", "from task\n")]).await;
    let gated = worktree.head().await.expect("task head");
    // The protected branch moves after the task started.
    let main_head = commit_files(&t.root, &[("main.txt", "from main\n")], "main work");

    let outcome = t
        .repo
        .land(task_id("t-1"), &gated, &head(&main_head))
        .await
        .expect("land");
    let LandOutcome::Landed { commit } = outcome else {
        panic!("expected Landed, got {outcome:?}");
    };
    assert_eq!(
        t.repo.protected_head().await.expect("protected head"),
        commit
    );

    let parents = git(
        &t.root,
        &["rev-list", "--parents", "-n", "1", commit.as_str()],
    );
    let want = format!("{commit} {main_head} {gated}");
    assert_eq!(parents, want, "a merge commit with both parents");

    assert_eq!(git(&t.root, &["show", "main:task.txt"]), "from task");
    assert_eq!(git(&t.root, &["show", "main:main.txt"]), "from main");
    // The clean main checkout receives both sides of the merge.
    assert_eq!(read_text(&t.root.join("task.txt")), "from task\n");
    assert_eq!(git(&t.root, &["status", "--porcelain"]), "");
}

#[tokio::test]
async fn land_reports_conflicting_files_and_touches_nothing() {
    let t = init().await;
    let worktree = task_with_files(&t.repo, "t-1", &[("shared.txt", "task version\n")]).await;
    let gated = worktree.head().await.expect("task head");
    let main_head = commit_files(&t.root, &[("shared.txt", "main version\n")], "main work");
    let before_status = git(&t.root, &["status", "--porcelain"]);
    let before_branch = git(&t.root, &["rev-parse", "acan/t-1"]);

    let outcome = t
        .repo
        .land(task_id("t-1"), &gated, &head(&main_head))
        .await
        .expect("land");
    let LandOutcome::Conflict { files } = outcome else {
        panic!("expected Conflict, got {outcome:?}");
    };
    assert_eq!(files, vec!["shared.txt".parse::<FilePath>().expect("path")]);

    assert_eq!(
        t.repo.protected_head().await.expect("protected head"),
        head(&main_head),
        "the protected branch did not move"
    );
    assert_eq!(
        git(&t.root, &["status", "--porcelain"]),
        before_status,
        "the user's working tree is untouched"
    );
    assert_eq!(
        git(&t.root, &["rev-parse", "acan/t-1"]),
        before_branch,
        "the task branch did not move"
    );
}

#[tokio::test]
async fn land_reports_a_moved_protected_head() {
    let t = init().await;
    let stale = t.repo.protected_head().await.expect("protected head");
    let worktree = task_with_files(&t.repo, "t-1", &[("task.txt", "from task\n")]).await;
    let gated = worktree.head().await.expect("task head");
    let moved = head(&commit_files(
        &t.root,
        &[("main.txt", "from main\n")],
        "main work",
    ));

    let outcome = t
        .repo
        .land(task_id("t-1"), &gated, &stale)
        .await
        .expect("land");
    assert_eq!(
        outcome,
        LandOutcome::HeadMoved {
            current: moved.clone()
        }
    );
    assert_eq!(
        t.repo.protected_head().await.expect("protected head"),
        moved,
        "the protected branch kept the concurrent move"
    );
}

#[tokio::test]
async fn landing_a_merge_twice_reports_the_same_merge_commit() {
    let t = init().await;
    let worktree = task_with_files(&t.repo, "t-1", &[("task.txt", "from task\n")]).await;
    let gated = worktree.head().await.expect("task head");
    let main_head = head(&commit_files(
        &t.root,
        &[("main.txt", "from main\n")],
        "main work",
    ));

    let first = t
        .repo
        .land(task_id("t-1"), &gated, &main_head)
        .await
        .expect("land");
    let LandOutcome::Landed { commit } = first else {
        panic!("expected Landed, got {first:?}");
    };

    // A retry that still names the head it decided at finds the task contained.
    let second = t
        .repo
        .land(task_id("t-1"), &gated, &main_head)
        .await
        .expect("re-land");
    assert_eq!(
        second,
        LandOutcome::Landed {
            commit: commit.clone()
        },
        "a retried land does not merge again"
    );
    // A retry after the protected head moved on reports the same merge.
    let third = t
        .repo
        .land(task_id("t-1"), &gated, &commit)
        .await
        .expect("re-land at the new head");
    assert_eq!(third, LandOutcome::Landed { commit }, "still the merge");
    let count = git(&t.root, &["rev-list", "--count", "main"]);
    assert_eq!(count, "4", "base, main work, task work, one merge");
}

#[tokio::test]
async fn land_of_a_commit_missing_from_the_repository_is_an_error() {
    let t = init().await;
    let expected = t.repo.protected_head().await.expect("protected head");
    let missing = head("0123456789abcdef0123456789abcdef01234567");
    let err = t
        .repo
        .land(task_id("t-9"), &missing, &expected)
        .await
        .expect_err("no such commit");
    assert!(matches!(err, acan_git::Error::Git { .. }), "{err:?}");
}

#[tokio::test]
async fn land_into_a_dirty_protected_checkout_refuses_and_moves_nothing() {
    let t = init().await;
    let expected = t.repo.protected_head().await.expect("protected head");
    let worktree = task_with_files(&t.repo, "t-1", &[("feat.txt", "feature\n")]).await;
    let gated = worktree.head().await.expect("gated head");
    std::fs::write(t.root.join("shared.txt"), "local edit\n").expect("local edit");

    let outcome = t
        .repo
        .land(task_id("t-1"), &gated, &expected)
        .await
        .expect("land");
    let LandOutcome::CheckoutDirty { path } = outcome.clone() else {
        panic!("expected CheckoutDirty, got {outcome:?}");
    };
    assert_eq!(canonical(&path), canonical(&t.root), "names the checkout");
    assert_eq!(
        t.repo.protected_head().await.expect("protected head"),
        expected,
        "the protected branch did not move"
    );
    assert_eq!(
        std::fs::read_to_string(t.root.join("shared.txt")).expect("edit"),
        "local edit\n",
        "the user's edit is kept"
    );
    assert!(!t.root.join("feat.txt").exists());

    let reason: Result<CommitId, String> = outcome.into();
    let reason = reason.expect_err("a refused land carries a reason");
    assert_eq!(
        reason,
        format!(
            "{} has uncommitted changes; commit or stash to land",
            path.display()
        )
    );

    // Once the user discards the edit, the same land goes through.
    git(&t.root, &["checkout", "--", "shared.txt"]);
    let retried = t
        .repo
        .land(task_id("t-1"), &gated, &expected)
        .await
        .expect("land again");
    assert_eq!(retried, LandOutcome::Landed { commit: gated });
}

#[tokio::test]
async fn land_leaves_a_checkout_on_another_branch_alone() {
    let t = init().await;
    git(&t.root, &["switch", "-q", "-c", "user-work"]);
    let expected = t.repo.protected_head().await.expect("protected head");
    let worktree = task_with_files(&t.repo, "t-1", &[("feat.txt", "feature\n")]).await;
    let gated = worktree.head().await.expect("gated head");

    let outcome = t
        .repo
        .land(task_id("t-1"), &gated, &expected)
        .await
        .expect("land");
    assert_eq!(
        outcome,
        LandOutcome::Landed {
            commit: gated.clone()
        }
    );
    assert_eq!(
        t.repo.protected_head().await.expect("protected head"),
        gated
    );
    assert!(
        !t.root.join("feat.txt").exists(),
        "a checkout of another branch keeps its files"
    );
    assert_eq!(git(&t.root, &["status", "--porcelain"]), "");
}

#[tokio::test]
async fn land_updates_a_clean_linked_checkout_of_the_protected_branch() {
    let t = init().await;
    git(&t.root, &["switch", "-q", "-c", "user-work"]);
    let linked = tempfile::tempdir().expect("linked checkout directory");
    let linked_path = linked.path().join("main");
    let linked_arg = linked_path.to_str().expect("UTF-8 path");
    git(&t.root, &["worktree", "add", "-q", linked_arg, "main"]);
    let expected = t.repo.protected_head().await.expect("protected head");
    let worktree = task_with_files(&t.repo, "t-1", &[("feat.txt", "feature\n")]).await;
    let gated = worktree.head().await.expect("gated head");

    let outcome = t
        .repo
        .land(task_id("t-1"), &gated, &expected)
        .await
        .expect("land");
    assert_eq!(outcome, LandOutcome::Landed { commit: gated });
    assert_eq!(
        read_text(&linked_path.join("feat.txt")),
        "feature\n",
        "the linked checkout of main receives the landed files"
    );
    assert_eq!(git(&linked_path, &["status", "--porcelain"]), "");
    assert!(!t.root.join("feat.txt").exists());
}

#[tokio::test]
async fn land_outcome_converts_to_the_core_land_result() {
    let commit: CommitId = "aaa111".parse().expect("commit");
    let landed: Result<CommitId, String> = LandOutcome::Landed {
        commit: commit.clone(),
    }
    .into();
    assert_eq!(landed, Ok(commit));

    let conflict: Result<CommitId, String> = LandOutcome::Conflict {
        files: vec!["a.txt".parse().expect("path")],
    }
    .into();
    assert_eq!(conflict, Err("merge conflict in a.txt".to_owned()));

    let moved: Result<CommitId, String> = LandOutcome::HeadMoved {
        current: "bbb222".parse().expect("commit"),
    }
    .into();
    assert_eq!(moved, Err("protected branch moved to bbb222".to_owned()));
}
