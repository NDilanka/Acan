//! The loop end to end on a real git repository. An order posted over HTTP is split, read back,
//! reviewed, claimed, built in its task worktree, gated, and landed. The offline models stand in
//! for the foreman. A scripted agent drives the worker side.
//!
//! The agent sends its commands through the writer, because agents cannot send commands over
//! HTTP yet. Everything else goes through the public surface: member commands over HTTP, the
//! agent's worktree route over HTTP, and the stored log.

use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command as Process;
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Duration;

use acan_core::{
    AgentCmd, AgentId, Check, Command, CommitId, DeclarationInput, Event, Fence, FilePath,
    GateVerdict, HumanCmd, Locus, MachineKey, Origin, Stage, Stamped, TaskId, TaskState,
    WitnessOutcome, WitnessSpec, World,
};
use tokio::time::{sleep, timeout};

use crate::offline::Offline;
use crate::{DEFAULT_WITNESS_CONCURRENCY, Running, ServerConfig, start};

const OWNER_TOKEN: &str = "acan_owner_e2e";
const AGENT_TOKEN: &str = "acan_agent_e2e";
const AGENT: &str = "agent-1";
const MACHINE: &str = "model";
const LANDED: &str = "landed.txt";
const AGENT_CONTENT: &str = "written by the agent\n";
/// Passes when the built commit carries the file, so it checks the commit rather than the disk.
const WITNESS_COMMAND: &str = "git cat-file -e HEAD:landed.txt";
/// The longest one step of a chain may take. A step runs git processes and, for a witness, one
/// pwsh start. Measured on this Windows checkout: 1.5 to 3 seconds idle, 6.3 seconds with two
/// flood tests running, and one unexplained slow run took 11.3 seconds. Sixty seconds leaves
/// about five times the worst of those, so a real stall still fails within a minute.
const WAIT: Duration = Duration::from_secs(60);
const POLL: Duration = Duration::from_millis(20);

#[tokio::test]
async fn an_order_is_gated_in_its_worktree_and_lands_on_the_protected_branch() {
    let repo = user_repo();
    let root = repo.path();
    let running = start_plant(root).await;
    let status_before = git(root, &["status", "--porcelain"]);
    let task = TaskId::from_str("t-0").expect("valid task id");

    post_order(&running).await;
    read_back(&running, task).await;

    let before_claim = worktree_route(&running, AGENT_TOKEN, task).await;
    assert_eq!(
        before_claim.status(),
        reqwest::StatusCode::FORBIDDEN,
        "an agent without the lease is not told where the worktree is"
    );
    let fence = claim(&running, task).await;
    let worktree = worktree_dir(root, task);
    wait_for_dir(&worktree).await;

    let response = worktree_route(&running, AGENT_TOKEN, task).await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = response.json().await.expect("the route answers JSON");
    let told = PathBuf::from(body["path"].as_str().expect("the route names the path"));
    assert_eq!(
        canonical(&told),
        canonical(&worktree),
        "the lease holder is told the worktree the loop created"
    );
    assert_eq!(body["branch"], "acan/t-0");
    let owner_view = worktree_route(&running, OWNER_TOKEN, task).await;
    assert_eq!(
        owner_view.status(),
        reqwest::StatusCode::FORBIDDEN,
        "a member token does not read a worktree"
    );

    let built = build(&running, root, task, fence, LANDED, AGENT_CONTENT).await;
    settle(&running, "the task to land", |_, world| {
        in_state(world, task, |state| {
            matches!(state, TaskState::Landed { .. })
        })
        .then_some(())
    })
    .await;

    let (log, _) = snapshot(&running).await;
    assert!(
        log.iter().any(|stamped| matches!(
            &stamped.event,
            Event::WitnessReported {
                outcome: WitnessOutcome::Pass,
                ..
            }
        )),
        "the witness passed on the built commit"
    );
    assert_eq!(
        git(root, &["show", "main:landed.txt"]),
        AGENT_CONTENT,
        "the protected branch carries the agent's file"
    );
    assert_eq!(
        git(root, &["rev-parse", "main"]).trim(),
        built.to_string(),
        "the task branch fast-forwarded the protected branch"
    );
    assert_eq!(
        git(root, &["status", "--porcelain"]),
        status_before,
        "the user's checkout status is unchanged by the land"
    );
    assert!(
        !root.join(LANDED).exists(),
        "the land never writes the user's working tree"
    );
    running.stop().await.expect("the server stops");
}

#[tokio::test]
async fn a_failing_witness_sends_the_task_back_to_building_and_lands_nothing() {
    let repo = user_repo();
    let root = repo.path();
    let running = start_plant(root).await;
    let task = TaskId::from_str("t-0").expect("valid task id");
    post_order(&running).await;
    read_back(&running, task).await;
    let fence = claim(&running, task).await;
    wait_for_dir(&worktree_dir(root, task)).await;
    let protected_before = git(root, &["rev-parse", "main"]);

    build(
        &running,
        root,
        task,
        fence,
        "other.txt",
        "not the file the witness checks\n",
    )
    .await;
    let failing = settle(&running, "the gate to close on a failure", |log, _| {
        log.iter().find_map(|stamped| match &stamped.event {
            Event::GateClosed {
                verdict: GateVerdict::Failed { failing },
                ..
            } => Some(failing.clone()),
            _ => None,
        })
    })
    .await;
    assert_eq!(failing.len(), 1, "the one witness is named as failing");

    let (log, origin) = snapshot(&running).await;
    assert!(
        log.iter().any(|stamped| matches!(
            &stamped.event,
            Event::WitnessReported {
                outcome: WitnessOutcome::Fail { .. },
                ..
            }
        )),
        "the check ran and failed; it did not error"
    );
    assert!(
        in_state(origin.world(), task, |state| matches!(
            state,
            TaskState::Claimed {
                stage: Stage::Declared { .. } | Stage::Building { .. },
                ..
            }
        )),
        "a failed gate returns the task to declared or building"
    );
    assert!(
        !log.iter().any(|stamped| matches!(
            &stamped.event,
            Event::LandFailed { .. } | Event::Landed { .. }
        )),
        "nothing reached the land queue"
    );
    assert_eq!(
        git(root, &["rev-parse", "main"]),
        protected_before,
        "the protected branch did not move"
    );
    running.stop().await.expect("the server stops");
}

#[tokio::test]
async fn a_conflicting_land_reports_the_files_and_reopens_the_task() {
    let repo = user_repo();
    let root = repo.path();
    let running = start_plant(root).await;
    let task = TaskId::from_str("t-0").expect("valid task id");
    post_order(&running).await;
    read_back(&running, task).await;
    let fence = claim(&running, task).await;
    wait_for_dir(&worktree_dir(root, task)).await;

    // The user lands a different version of the same file while the agent works.
    let user_commit = commit_on_protected(root, LANDED, "written by the user\n");
    build(&running, root, task, fence, LANDED, AGENT_CONTENT).await;

    let reason = settle(&running, "the land to fail", |log, _| {
        log.iter().find_map(|stamped| match &stamped.event {
            Event::LandFailed {
                task: failed,
                reason,
            } if *failed == task => Some(reason.clone()),
            _ => None,
        })
    })
    .await;
    assert_eq!(
        reason, "merge conflict in landed.txt",
        "the conflict names the file"
    );
    settle(&running, "the task to reopen", |_, world| {
        in_state(world, task, |state| matches!(state, TaskState::Open)).then_some(())
    })
    .await;
    assert_eq!(
        git(root, &["rev-parse", "main"]).trim(),
        user_commit.to_string(),
        "a refused land leaves the protected branch at the user's commit"
    );
    running.stop().await.expect("the server stops");
}

#[tokio::test]
async fn a_land_onto_a_protected_branch_that_moved_merges_only_the_gated_commit() {
    let repo = user_repo();
    let root = repo.path();
    let running = start_plant(root).await;
    let task = TaskId::from_str("t-0").expect("valid task id");
    post_order(&running).await;
    read_back(&running, task).await;
    let fence = claim(&running, task).await;
    wait_for_dir(&worktree_dir(root, task)).await;

    // The user lands a different file while the agent works, so the land is a real merge.
    let user_commit = commit_on_protected(root, "user.txt", "written by the user\n");
    let built = build(&running, root, task, fence, LANDED, AGENT_CONTENT).await;
    settle(&running, "the task to land", |_, world| {
        in_state(world, task, |state| {
            matches!(state, TaskState::Landed { .. })
        })
        .then_some(())
    })
    .await;

    assert_eq!(
        git(root, &["rev-parse", "main^1"]).trim(),
        user_commit.to_string(),
        "the merge's first parent is the protected head it merged onto"
    );
    assert_eq!(
        git(root, &["rev-parse", "main^2"]).trim(),
        built.to_string(),
        "the merge's second parent is the gated commit"
    );
    assert_eq!(git(root, &["show", "main:landed.txt"]), AGENT_CONTENT);
    assert_eq!(
        git(root, &["show", "main:user.txt"]),
        "written by the user\n",
        "the user's file survives the land"
    );
    running.stop().await.expect("the server stops");
}

#[tokio::test]
async fn a_reclaim_after_a_refused_land_starts_from_the_protected_head_the_user_moved_to() {
    let repo = user_repo();
    let root = repo.path();
    let running = start_plant(root).await;
    let task = TaskId::from_str("t-0").expect("valid task id");
    post_order(&running).await;
    read_back(&running, task).await;
    let fence = claim(&running, task).await;
    wait_for_dir(&worktree_dir(root, task)).await;

    let user_commit = commit_on_protected(root, LANDED, "written by the user\n");
    build(&running, root, task, fence, LANDED, AGENT_CONTENT).await;
    settle(&running, "the land to fail", |log, _| {
        log.iter().find_map(|stamped| match &stamped.event {
            Event::LandFailed {
                task: failed,
                reason,
            } if *failed == task => Some(reason.clone()),
            _ => None,
        })
    })
    .await;
    settle(&running, "the task to reopen", |_, world| {
        in_state(world, task, |state| matches!(state, TaskState::Open)).then_some(())
    })
    .await;
    // The directory goes first and the branch after it, so both are awaited.
    settle(
        &running,
        "the reopened worktree and branch to be removed",
        |_, _| {
            (!worktree_dir(root, task).exists() && !branch_exists(root, "acan/t-0")).then_some(())
        },
    )
    .await;

    let fence = claim(&running, task).await;
    wait_for_dir(&worktree_dir(root, task)).await;
    assert_eq!(
        git(&worktree_dir(root, task), &["rev-parse", "HEAD"]).trim(),
        user_commit.to_string(),
        "the reclaimed worktree starts at the protected head the user moved to"
    );
    let built = build(&running, root, task, fence, LANDED, AGENT_CONTENT).await;
    settle(&running, "the reclaimed task to land", |_, world| {
        in_state(world, task, |state| {
            matches!(state, TaskState::Landed { .. })
        })
        .then_some(())
    })
    .await;
    assert_eq!(
        git(root, &["rev-parse", "main"]).trim(),
        built.to_string(),
        "the reclaimed task fast-forwards the protected branch"
    );
    assert_eq!(git(root, &["show", "main:landed.txt"]), AGENT_CONTENT);
    running.stop().await.expect("the server stops");
}

/// A repository with `main` as the protected branch and the user's checkout on `user-work`. The
/// server must never touch that checkout.
fn user_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temporary repository");
    let root = dir.path();
    git(root, &["init", "-q", "-b", "main"]);
    git(root, &["config", "user.name", "Test User"]);
    git(root, &["config", "user.email", "test@example.test"]);
    fs::create_dir_all(root.join(MACHINE)).expect("machine directory");
    fs::write(
        root.join(MACHINE).join("store.ts"),
        "export function load() { return 1; }\n",
    )
    .expect("fixture");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "initial"]);
    git(root, &["switch", "-q", "-c", "user-work"]);
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

/// Whether `branch` exists in the repository that `dir` belongs to.
fn branch_exists(dir: &Path, branch: &str) -> bool {
    Process::new("git")
        .arg("-C")
        .arg(dir)
        .args(["show-ref", "--verify", "--quiet"])
        .arg(format!("refs/heads/{branch}"))
        .status()
        .expect("git runs")
        .success()
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().expect("the path exists")
}

fn worktree_dir(root: &Path, task: TaskId) -> PathBuf {
    root.join(".acan").join("worktrees").join(task.to_string())
}

/// Writes the token file before the first start, so the test holds an owner and an agent token.
fn write_tokens(root: &Path) {
    let state = root.join(".acan");
    fs::create_dir_all(&state).expect("state directory");
    fs::write(
        state.join("tokens.json"),
        format!(
            r#"[
              {{"token": "{OWNER_TOKEN}", "principal": {{"kind": "member", "id": "owner"}}}},
              {{"token": "{AGENT_TOKEN}", "principal": {{"kind": "agent", "id": "{AGENT}"}}}}
            ]"#
        ),
    )
    .expect("tokens file");
}

async fn start_plant(root: &Path) -> Running {
    write_tokens(root);
    let config = ServerConfig {
        repo: root.to_path_buf(),
        addr: SocketAddr::from(([127, 0, 0, 1], 0)),
        team: false,
        web: None,
        protected_branch: Some("main".to_owned()),
        witness_http_base_url: None,
        witness_concurrency: DEFAULT_WITNESS_CONCURRENCY,
    };
    let offline = Offline::default();
    let ledger = offline.ledger().clone();
    start(config, offline, ledger)
        .await
        .expect("the server starts on the test repository")
}

fn url(running: &Running, path: &str) -> String {
    format!("http://{}{path}", running.local_addr)
}

/// One client for every request, so each poll reuses its connection instead of building a client.
fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new)
}

fn agent() -> AgentId {
    AgentId::from_str(AGENT).expect("valid agent id")
}

fn machine() -> MachineKey {
    MachineKey::from_str(MACHINE).expect("valid machine key")
}

async fn post_order(running: &Running) {
    let response = client()
        .post(url(running, "/api/commands"))
        .bearer_auth(OWNER_TOKEN)
        .json(&HumanCmd::PostOrder {
            machine: machine(),
            sentence: "land the agent's file".to_owned(),
            witnesses: vec![WitnessSpec {
                check: Check::Test {
                    command: WITNESS_COMMAND.to_owned(),
                },
                reason: "the built commit carries the file".to_owned(),
            }],
        })
        .send()
        .await
        .expect("the order request is sent");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "the order is posted"
    );
}

/// Waits for the offline split, registers the agent, and reads the task back. Returns once the
/// review has matched the readback, so the task can be claimed.
async fn read_back(running: &Running, task: TaskId) {
    settle(running, "the offline split", |_, world| {
        world.task(task).map(drop)
    })
    .await;
    let response = client()
        .post(url(running, "/api/commands"))
        .bearer_auth(OWNER_TOKEN)
        .json(&HumanCmd::RegisterAgent { agent: agent() })
        .send()
        .await
        .expect("the registration is sent");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "the agent is registered"
    );
    agent_command(
        running,
        AgentCmd::PostReadback {
            task,
            summary: "write the file".to_owned(),
            touches: vec![Locus::Machine { machine: machine() }],
        },
    )
    .await;
    settle(running, "the offline review", |_, world| {
        world
            .readbacks_of(task)
            .any(|readback| readback.verdict.is_some())
            .then_some(())
    })
    .await;
}

/// Claims the task and returns the fence the agent must present. The lease grant creates the
/// worktree on the job loop; the caller waits for it.
async fn claim(running: &Running, task: TaskId) -> Fence {
    agent_command(running, AgentCmd::Claim { task }).await;
    settle(running, "the claim", |_, world| {
        match &world.task(task)?.state {
            TaskState::Claimed { claim, .. } => Some(claim.fence(task)),
            _ => None,
        }
    })
    .await
}

/// The scripted agent's build: declare the file, wait for the build to start, write and commit
/// the file in its worktree, and report the commit.
async fn build(
    running: &Running,
    root: &Path,
    task: TaskId,
    fence: Fence,
    file: &str,
    content: &str,
) -> CommitId {
    agent_command(
        running,
        AgentCmd::Declare {
            fence,
            footprint: DeclarationInput {
                files: vec![FilePath::from_str(file).expect("valid file path")],
                ..DeclarationInput::default()
            },
        },
    )
    .await;
    settle(running, "the build to start", |_, world| {
        in_state(world, task, |state| {
            matches!(
                state,
                TaskState::Claimed {
                    stage: Stage::Building { .. },
                    ..
                }
            )
        })
        .then_some(())
    })
    .await;
    let head = commit_file(&worktree_dir(root, task), file, content);
    agent_command(
        running,
        AgentCmd::ReportBuilt {
            fence,
            head: head.clone(),
        },
    )
    .await;
    head
}

async fn agent_command(running: &Running, cmd: AgentCmd) {
    running
        .writer
        .submit(Command::Agent {
            agent: agent(),
            cmd,
        })
        .await
        .expect("the origin accepts the agent's command");
}

/// Writes `file` in `dir`, commits it, and returns the new commit.
fn commit_file(dir: &Path, file: &str, content: &str) -> CommitId {
    fs::write(dir.join(file), content).expect("the file is written");
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", &format!("write {file}")]);
    git(dir, &["rev-parse", "HEAD"])
        .trim()
        .parse()
        .expect("git prints a commit id")
}

/// Commits `content` to `file` on the protected branch, as the user would. A scratch checkout
/// does the commit, so the user's own checkout stays put.
fn commit_on_protected(root: &Path, file: &str, content: &str) -> CommitId {
    let scratch = tempfile::tempdir().expect("scratch directory");
    let checkout = scratch.path().join("main");
    let checkout_arg = checkout.to_str().expect("UTF-8 path");
    git(root, &["worktree", "add", "-q", checkout_arg, "main"]);
    let commit = commit_file(&checkout, file, content);
    git(root, &["worktree", "remove", "--force", checkout_arg]);
    commit
}

async fn worktree_route(running: &Running, token: &str, task: TaskId) -> reqwest::Response {
    client()
        .get(url(running, &format!("/api/tasks/{task}/worktree")))
        .bearer_auth(token)
        .send()
        .await
        .expect("the worktree request is sent")
}

async fn wait_for_dir(path: &Path) {
    timeout(WAIT, async {
        while !path.is_dir() {
            sleep(POLL).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("the worktree {} was never created", path.display()));
}

fn in_state(world: &World, task: TaskId, state: impl Fn(&TaskState) -> bool) -> bool {
    world.task(task).is_some_and(|t| state(&t.state))
}

/// The stored log and the world it replays to, read through the owner's events route.
async fn snapshot(running: &Running) -> (Vec<Stamped>, Origin) {
    let log: Vec<Stamped> = client()
        .get(url(running, "/api/events"))
        .bearer_auth(OWNER_TOKEN)
        .send()
        .await
        .expect("the events request is sent")
        .error_for_status()
        .expect("the owner may read the log")
        .json()
        .await
        .expect("the log is JSON");
    let origin = Origin::replay(log.iter().cloned()).expect("the stored log replays");
    (log, origin)
}

/// Polls the stored log until `done` accepts it, and returns what it found. A timeout names the
/// newest events, so a stalled step shows where the chain stopped.
async fn settle<T>(
    running: &Running,
    what: &str,
    done: impl Fn(&[Stamped], &World) -> Option<T>,
) -> T {
    let result = timeout(WAIT, async {
        loop {
            let (log, origin) = snapshot(running).await;
            if let Some(found) = done(&log, origin.world()) {
                return found;
            }
            sleep(POLL).await;
        }
    })
    .await;
    let Ok(found) = result else {
        let (log, _) = snapshot(running).await;
        let newest: Vec<&Event> = log
            .iter()
            .rev()
            .take(4)
            .map(|stamped| &stamped.event)
            .collect();
        panic!("timed out after {WAIT:?} waiting for {what}; newest events first: {newest:?}");
    };
    found
}
