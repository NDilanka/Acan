//! The worker's local tools on a real git worktree. A write is allowed only to a declared file,
//! every path stays inside the worktree, and a commit refuses a change outside the declaration.

use std::fs;
use std::path::Path;
use std::process::Command as Process;

use acan_agent::{Worktree, normalize_path};

const DECLARED: &str = "model/store.ts";

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temporary repository");
    let root = dir.path();
    git(root, &["init", "-q", "-b", "main"]);
    git(root, &["config", "user.name", "Test User"]);
    git(root, &["config", "user.email", "test@example.test"]);
    // Windows checkouts convert to CRLF by default, and the files are compared byte for byte.
    git(root, &["config", "core.autocrlf", "false"]);
    fs::create_dir_all(root.join("model")).expect("machine directory");
    fs::write(root.join(DECLARED), "original\n").expect("fixture");
    fs::write(root.join("model/other.ts"), "other\n").expect("fixture");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "initial"]);
    dir
}

/// Runs git in `dir` and returns its stdout. Panics with git's stderr when the command fails.
fn git(dir: &Path, args: &[&str]) -> String {
    let output = Process::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("git prints UTF-8")
}

fn worktree(root: &Path) -> Worktree {
    Worktree::new(root.to_path_buf(), &[DECLARED.to_owned()], None)
}

#[test]
fn a_write_is_allowed_only_to_a_declared_file() {
    let dir = repo();
    let tools = worktree(dir.path());
    let refused = tools
        .write_file("model/other.ts", "changed\n")
        .expect_err("an undeclared file is refused");
    assert!(
        refused.contains("not declared"),
        "the refusal names the rule: {refused}"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("model/other.ts")).expect("readable"),
        "other\n",
        "the refused write left the file alone"
    );
    tools
        .write_file(DECLARED, "changed\n")
        .expect("the declared file is written");
    assert_eq!(
        fs::read_to_string(dir.path().join(DECLARED)).expect("readable"),
        "changed\n"
    );
}

#[tokio::test]
async fn paths_inside_the_worktree_are_read_listed_and_written() {
    let dir = repo();
    let tools = Worktree::new(
        dir.path().to_path_buf(),
        &[DECLARED.to_owned(), "model/new/fresh.ts".to_owned()],
        None,
    );
    assert_eq!(
        tools
            .read_file(DECLARED)
            .await
            .expect("an inside file reads"),
        "original\n"
    );
    let listing = tools
        .list_files("model")
        .expect("an inside directory lists");
    assert!(listing.contains("store.ts"), "{listing}");
    tools
        .write_file("model/new/fresh.ts", "fresh\n")
        .expect("a declared file in a new directory is written");
    assert_eq!(
        fs::read_to_string(dir.path().join("model/new/fresh.ts")).expect("readable"),
        "fresh\n"
    );
}

#[tokio::test]
async fn paths_outside_the_worktree_are_refused() {
    let dir = repo();
    let tools = worktree(dir.path());
    for path in [
        "../escape.txt",
        "/etc/passwd",
        "model/../../escape",
        ".git/config",
    ] {
        assert!(
            tools.read_file(path).await.is_err(),
            "reading {path:?} is refused"
        );
        assert!(
            tools.write_file(path, "x").is_err(),
            "writing {path:?} is refused"
        );
    }
    assert!(
        tools.list_files("..").is_err(),
        "listing outside the worktree is refused"
    );
}

/// Creates a link at `at` to `target`. Windows needs the directory form for a directory, and a
/// Windows account without developer mode or elevation cannot create links at all.
fn link(target: &Path, at: &Path, is_dir: bool) {
    #[cfg(unix)]
    {
        let _ = is_dir;
        std::os::unix::fs::symlink(target, at).expect("the link is created");
    }
    #[cfg(windows)]
    {
        let made = if is_dir {
            std::os::windows::fs::symlink_dir(target, at)
        } else {
            std::os::windows::fs::symlink_file(target, at)
        };
        made.expect("the link is created (Windows needs developer mode or elevation for links)");
    }
}

/// A link inside the worktree that points outside it must not carry reads or writes out.
#[tokio::test]
async fn a_symlink_that_leaves_the_worktree_is_refused() {
    let dir = repo();
    let outside = tempfile::tempdir().expect("a directory outside the worktree");
    let secret = outside.path().join("secret.txt");
    fs::write(&secret, "outside\n").expect("the outside file");
    let link_path = "model/link.ts";
    link(&secret, &dir.path().join(link_path), false);
    let tools = Worktree::new(dir.path().to_path_buf(), &[link_path.to_owned()], None);

    assert!(
        tools.read_file(link_path).await.is_err(),
        "reading through the link is refused"
    );
    assert!(
        tools.write_file(link_path, "changed\n").is_err(),
        "writing through the link is refused"
    );
    assert_eq!(
        fs::read_to_string(&secret).expect("readable"),
        "outside\n",
        "the outside file is untouched"
    );
}

/// A declared path that is a link to an undeclared file in the same worktree must not carry the
/// write to that file, and neither may a declared path under a linked directory.
#[test]
fn a_write_never_goes_through_a_link_even_inside_the_worktree() {
    let dir = repo();
    let root = dir.path();
    link(
        &root.join("model/other.ts"),
        &root.join("model/linked.ts"),
        false,
    );
    link(&root.join("model"), &root.join("alias"), true);
    let tools = Worktree::new(
        root.to_path_buf(),
        &["model/linked.ts".to_owned(), "alias/other.ts".to_owned()],
        None,
    );

    assert!(
        tools.write_file("model/linked.ts", "changed\n").is_err(),
        "a declared link is not written through"
    );
    assert!(
        tools.write_file("alias/other.ts", "changed\n").is_err(),
        "a declared path under a linked directory is not written through"
    );
    assert_eq!(
        fs::read_to_string(root.join("model/other.ts")).expect("readable"),
        "other\n",
        "the undeclared file is untouched"
    );
}

#[test]
fn a_path_has_one_form_to_the_origin_and_the_worktree() {
    assert_eq!(
        normalize_path("./model/store.ts").expect("a plain path"),
        DECLARED
    );
    assert_eq!(
        normalize_path("model//store.ts").expect("a plain path"),
        DECLARED
    );
    for bad in [
        "model/store.ts.",
        "model/store.ts ",
        "model/a:b",
        "./",
        ".GIT/config",
    ] {
        assert!(normalize_path(bad).is_err(), "accepted {bad:?}");
    }
}

/// A backslash is a separator on Windows and a name character elsewhere, so the form it takes
/// depends on the platform the worktree lives on. The origin only ever sees the `/` form.
#[test]
fn a_backslash_path_is_refused_or_read_as_a_separator_by_platform() {
    let backslash = normalize_path("model\\store.ts");
    if cfg!(windows) {
        assert_eq!(backslash.expect("a separator on Windows"), DECLARED);
    } else {
        assert!(backslash.is_err(), "a name the origin does not keep");
    }
}

#[test]
fn a_declared_alias_names_the_declared_file_for_writes() {
    let dir = repo();
    let tools = Worktree::new(
        dir.path().to_path_buf(),
        &["./model/store.ts".to_owned()],
        None,
    );
    tools
        .write_file("model/store.ts", "changed\n")
        .expect("the plain form of a declared alias is written");
    assert_eq!(
        fs::read_to_string(dir.path().join(DECLARED)).expect("readable"),
        "changed\n"
    );
}

#[tokio::test]
async fn a_commit_refuses_a_change_outside_the_declared_files() {
    let dir = repo();
    let root = dir.path();
    let tools = worktree(root);
    tools
        .write_file(DECLARED, "changed\n")
        .expect("the declared file is written");
    fs::write(root.join("model/other.ts"), "sneaked\n").expect("an undeclared change");

    let refused = tools
        .commit("change the store")
        .await
        .expect_err("the commit is refused");
    assert!(
        refused.contains("changed outside the declared files")
            && refused.contains("model/other.ts"),
        "the refusal names the file: {refused}"
    );
    assert_eq!(
        git(root, &["diff", "--cached", "--name-only"]),
        "",
        "the refused change is not staged"
    );
    assert_eq!(
        git(root, &["rev-parse", "HEAD"]).trim(),
        git(root, &["rev-parse", "main"]).trim(),
        "the refused commit did not move HEAD"
    );
}

#[tokio::test]
async fn a_commit_of_only_declared_changes_moves_head() {
    let dir = repo();
    let root = dir.path();
    let tools = worktree(root);
    tools
        .write_file(DECLARED, "changed\n")
        .expect("the declared file is written");

    let head = tools
        .commit("change the store")
        .await
        .expect("the declared change commits");
    assert_eq!(
        git(root, &["rev-parse", "HEAD"]).trim(),
        head,
        "the returned commit is HEAD"
    );
    assert_eq!(
        git(root, &["show", "--name-only", "--format=", "HEAD"]).trim(),
        DECLARED,
        "the commit carries only the declared file"
    );
}

#[tokio::test]
async fn run_tests_needs_a_configured_command() {
    let dir = repo();
    let tools = worktree(dir.path());
    let refused = tools
        .run_tests()
        .await
        .expect_err("no test command is configured");
    assert!(refused.contains("no test command"), "{refused}");
}

#[tokio::test]
async fn run_tests_reports_pass_and_fail_as_the_gate_does() {
    let dir = repo();
    let passing = Worktree::new(dir.path().to_path_buf(), &[], Some("exit 0".to_owned()));
    assert_eq!(passing.run_tests().await.expect("the command runs"), "pass");

    let failing = Worktree::new(dir.path().to_path_buf(), &[], Some("exit 3".to_owned()));
    let text = failing
        .run_tests()
        .await
        .expect("a failing command is reported, not refused");
    assert!(text.starts_with("fail:") && text.contains('3'), "{text}");
}

#[tokio::test]
async fn restore_brings_back_a_tracked_file_and_removes_an_untracked_one() {
    let dir = repo();
    let root = dir.path();
    let new = "model/new.ts";
    let tools = Worktree::new(
        root.to_path_buf(),
        &[DECLARED.to_owned(), new.to_owned()],
        None,
    );
    tools
        .write_file(DECLARED, "changed\n")
        .expect("the tracked file is written");
    tools
        .write_file(new, "new\n")
        .expect("the new file is written");

    tools
        .restore(&[DECLARED.to_owned(), new.to_owned()])
        .await
        .expect("both files are restored");

    assert_eq!(
        fs::read_to_string(root.join(DECLARED)).expect("the file is readable"),
        "original\n",
        "the tracked file is back at its committed text"
    );
    assert!(!root.join(new).exists(), "the untracked file is removed");
}

#[test]
fn declare_replaces_the_files_a_write_may_touch() {
    let dir = repo();
    let mut tools = worktree(dir.path());
    tools.declare(&["model/other.ts".to_owned()]);
    tools
        .write_file(DECLARED, "changed\n")
        .expect_err("the old declaration no longer allows the write");
    tools
        .write_file("model/other.ts", "changed\n")
        .expect("the new declaration allows the write");
}
