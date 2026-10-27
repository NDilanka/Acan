//! The worker's local tools on a real git worktree. A write is allowed only to a declared file,
//! every path stays inside the worktree, and a commit refuses a change outside the declaration.

use std::fs;
use std::path::Path;
use std::process::Command as Process;

use acan_agent::Worktree;

const DECLARED: &str = "model/store.ts";

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temporary repository");
    let root = dir.path();
    git(root, &["init", "-q", "-b", "main"]);
    git(root, &["config", "user.name", "Test User"]);
    git(root, &["config", "user.email", "test@example.test"]);
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

/// A link inside the worktree that points outside it must not carry reads or writes out.
/// Symlinks need a privilege on Windows, so this check runs on unix only.
#[cfg(unix)]
#[tokio::test]
async fn a_symlink_that_leaves_the_worktree_is_refused() {
    let dir = repo();
    let outside = tempfile::tempdir().expect("a directory outside the worktree");
    let secret = outside.path().join("secret.txt");
    fs::write(&secret, "outside\n").expect("the outside file");
    let link = "model/link.ts";
    std::os::unix::fs::symlink(&secret, dir.path().join(link)).expect("the link");
    let tools = Worktree::new(dir.path().to_path_buf(), &[link.to_owned()], None);

    assert!(
        tools.read_file(link).await.is_err(),
        "reading through the link is refused"
    );
    assert!(
        tools.write_file(link, "changed\n").is_err(),
        "writing through the link is refused"
    );
    assert_eq!(
        fs::read_to_string(&secret).expect("readable"),
        "outside\n",
        "the outside file is untouched"
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
