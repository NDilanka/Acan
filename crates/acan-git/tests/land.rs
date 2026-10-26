//! Landing task branches onto the protected branch.

mod common;

use acan_core::{CommitId, FilePath};
use acan_git::LandOutcome;

use common::{commit_files, git, init, task_id, task_with_files};

#[tokio::test]
async fn land_fast_forwards_when_the_task_contains_the_protected_head() {
    let t = init().await;
    let expected = t.repo.protected_head().await.expect("protected head");
    let worktree = task_with_files(&t.repo, "t-1", &[("feat.txt", "feature\n")]).await;
    let task_head = worktree.head().await.expect("task head");

    let outcome = t.repo.land(task_id("t-1"), &expected).await.expect("land");
    let LandOutcome::Landed { commit } = outcome else {
        panic!("expected Landed, got {outcome:?}");
    };
    assert_eq!(commit, task_head, "a fast-forward lands the task head");
    assert_eq!(
        t.repo.protected_head().await.expect("protected head"),
        task_head
    );
    // The ref moved without a checkout: the user's files stay as they were.
    assert!(
        !t.root.join("feat.txt").exists(),
        "the user's working tree is untouched"
    );

    let again = t
        .repo
        .land(task_id("t-1"), &expected)
        .await
        .expect("re-land");
    assert_eq!(
        again,
        LandOutcome::Landed { commit },
        "landing twice reports the same commit"
    );
}

#[tokio::test]
async fn land_merges_non_overlapping_work() {
    let t = init().await;
    let worktree = task_with_files(&t.repo, "t-1", &[("task.txt", "from task\n")]).await;
    let task_head = worktree.head().await.expect("task head");
    // The protected branch moves after the task started.
    let main_head = commit_files(&t.root, &[("main.txt", "from main\n")], "main work");

    let outcome = t
        .repo
        .land(task_id("t-1"), &main_head.parse().expect("head"))
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
    let want = format!("{commit} {main_head} {task_head}");
    assert_eq!(parents, want, "a merge commit with both parents");

    assert_eq!(git(&t.root, &["show", "main:task.txt"]), "from task");
    assert_eq!(git(&t.root, &["show", "main:main.txt"]), "from main");
    // The ref moved without a checkout: the user's files stay as they were.
    assert!(
        !t.root.join("task.txt").exists(),
        "the user's working tree is untouched"
    );
}

#[tokio::test]
async fn land_reports_conflicting_files_and_touches_nothing() {
    let t = init().await;
    task_with_files(&t.repo, "t-1", &[("shared.txt", "task version\n")]).await;
    let main_head = commit_files(&t.root, &[("shared.txt", "main version\n")], "main work");
    let before_status = git(&t.root, &["status", "--porcelain"]);
    let before_branch = git(&t.root, &["rev-parse", "acan/t-1"]);

    let outcome = t
        .repo
        .land(task_id("t-1"), &main_head.parse().expect("head"))
        .await
        .expect("land");
    let LandOutcome::Conflict { files } = outcome else {
        panic!("expected Conflict, got {outcome:?}");
    };
    assert_eq!(files, vec!["shared.txt".parse::<FilePath>().expect("path")]);

    assert_eq!(
        t.repo.protected_head().await.expect("protected head"),
        main_head.parse::<CommitId>().expect("head"),
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
    task_with_files(&t.repo, "t-1", &[("task.txt", "from task\n")]).await;
    let moved: CommitId = commit_files(&t.root, &[("main.txt", "from main\n")], "main work")
        .parse()
        .expect("head");

    let outcome = t.repo.land(task_id("t-1"), &stale).await.expect("land");
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
    task_with_files(&t.repo, "t-1", &[("task.txt", "from task\n")]).await;
    let main_head = commit_files(&t.root, &[("main.txt", "from main\n")], "main work");
    let expected: CommitId = main_head.parse().expect("head");

    let first = t.repo.land(task_id("t-1"), &expected).await.expect("land");
    let LandOutcome::Landed { commit } = first else {
        panic!("expected Landed, got {first:?}");
    };

    let second = t
        .repo
        .land(task_id("t-1"), &expected)
        .await
        .expect("re-land");
    assert_eq!(
        second,
        LandOutcome::Landed { commit },
        "a retried land does not merge again"
    );
    let count = git(&t.root, &["rev-list", "--count", "main"]);
    assert_eq!(count, "4", "base, main work, task work, one merge");
}

#[tokio::test]
async fn land_without_a_task_branch_is_an_error() {
    let t = init().await;
    let head = t.repo.protected_head().await.expect("protected head");
    let err = t
        .repo
        .land(task_id("t-9"), &head)
        .await
        .expect_err("no branch");
    assert!(matches!(err, acan_git::Error::UnknownTask(_)), "{err:?}");
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
