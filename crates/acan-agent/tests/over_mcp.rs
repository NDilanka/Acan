//! The harness over MCP against a real origin. The origin, its writer, its git worktrees, and its
//! gate are real. The foreman and the workers are scripted, so no test needs the network.
//!
//! Each worker's first edit waits on a gate the test opens after it has seen the clash. A real
//! model call takes longer than a second declaration, so the gate stands in for that delay. The
//! worker's own write still waits for build clearance, which the origin withholds while the
//! clash is open.

use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command as Process;
use std::str::FromStr;
use std::time::Duration;

use acan_agent::{Exit, McpClient, Outcome, Reply, WorkerConfig, run};
use acan_core::{
    AgentId, Check, ClashId, Event, Generation, HumanCmd, MachineKey, Origin, Rejection,
    Settlement, Stage, Stamped, TaskId, TaskState, WitnessSpec, World,
};
use acan_llm::{
    CallRecord, ChatRequest, ChatResponse, FunctionCall, LlmError, Message, ModelClient,
    Reply as StubReply, Role, Stub, ToolCall, ToolKind, Usage, UsageLedger,
};
use acan_server::{DEFAULT_WITNESS_CONCURRENCY, Running, ServerConfig, start};
use serde_json::{Value, json};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};

const OWNER_TOKEN: &str = "acan_owner_over_mcp";
const ONE_TOKEN: &str = "acan_agent_one_over_mcp";
const TWO_TOKEN: &str = "acan_agent_two_over_mcp";
const ONE: &str = "agent-1";
const TWO: &str = "agent-2";
const MACHINE: &str = "model";
const FILE: &str = "model/store.ts";
const ORIGINAL: &str = "export function load() { return 1; }\n";
const CHANGED: &str = "export function load() { return 2; }\n";
/// Passes when the built commit carries the file, so the gate checks the commit, not the disk.
const WITNESS: &str = "git cat-file -e HEAD:model/store.ts";
/// The longest one step may take. A worker's write waits on clearance, and clearance waits on the
/// settlement, so this bounds every wait in the scenario.
const WAIT: Duration = Duration::from_secs(20);
const POLL: Duration = Duration::from_millis(20);
/// How long the test lets the workers run after the gate opens, to show nothing is cleared.
const HOLD: Duration = Duration::from_millis(300);

/// The foreman answers by the shape of its request: a review carries readbacks, an explanation
/// carries a clash, and a split carries a machine. It splits the order into two tasks on one
/// machine, matches every readback, and explains a clash without a suggestion.
struct Foreman;

impl ModelClient for Foreman {
    async fn chat(&self, role: Role, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        let context: Value = request
            .last_user_text()
            .and_then(|text| serde_json::from_str(text).ok())
            .ok_or(LlmError::NoScriptedReply { role })?;
        let reply = if context.get("readbacks").is_some() {
            review(&context)
        } else if context.get("clash").is_some() {
            explain()
        } else if context.get("machine").is_some() {
            split()
        } else {
            return Err(LlmError::NoScriptedReply { role });
        };
        Ok(answer(role, reply.to_string()))
    }
}

fn answer(role: Role, content: String) -> ChatResponse {
    ChatResponse {
        content: Some(content),
        tool_calls: Vec::new(),
        finish_reason: Some("stop".to_owned()),
        call: CallRecord {
            role,
            provider: "scripted".to_owned(),
            model: "foreman".to_owned(),
            usage: Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
            },
            latency: Duration::ZERO,
        },
    }
}

fn split() -> Value {
    let task = |title: &str| {
        json!({
            "title": title,
            "brief": format!("{title} in {FILE}"),
            "scope": [MACHINE],
        })
    };
    json!({ "tasks": [task("Change the loader"), task("Change the loader again")] })
}

fn review(context: &Value) -> Value {
    let witnesses: Vec<Value> = context["witnesses"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|witness| witness["id"].as_str())
        .map(|id| json!({ "witness": id, "judgement": { "judgement": "match" } }))
        .collect();
    let readbacks: Vec<Value> = context["readbacks"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|readback| readback["readback"].as_str())
        .map(|id| json!({ "readback": id, "witnesses": witnesses }))
        .collect();
    json!({ "readbacks": readbacks, "question": null })
}

fn explain() -> Value {
    json!({
        "consequence": "Both tasks change the same file.",
        "breaks": null,
        "suggested": null,
    })
}

/// A worker model. Plans answer at once. An edit waits on the gate first, so the test decides
/// when the worker may try to write.
struct Worker {
    script: Stub,
    gate: watch::Receiver<bool>,
}

impl Worker {
    fn new(gate: watch::Receiver<bool>) -> Self {
        Self {
            script: worker_script(),
            gate,
        }
    }
}

impl ModelClient for Worker {
    async fn chat(&self, role: Role, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        if !request.tools.is_empty() {
            let mut gate = self.gate.clone();
            // The sender lives until the test ends, so a closed channel only means the test is over.
            let _ = gate.wait_for(|open| *open).await;
        }
        self.script.chat(role, request).await
    }
}

/// The worker's script: plan the declared file, write it, commit it, then stop.
fn worker_script() -> Stub {
    Stub::default()
        .when(
            Role::Worker,
            |request| request.tools.is_empty(),
            StubReply::json(&json!({ "summary": "change the loader", "files": [FILE] })),
        )
        .when(
            Role::Worker,
            |request| tool_replies(request) == 0,
            StubReply {
                tool_calls: vec![tool_call(
                    "write_file",
                    json!({ "path": FILE, "content": CHANGED }),
                )],
                ..StubReply::default()
            },
        )
        .when(
            Role::Worker,
            |request| tool_replies(request) == 1,
            StubReply {
                tool_calls: vec![tool_call(
                    "commit",
                    json!({ "message": "change the loader" }),
                )],
                ..StubReply::default()
            },
        )
        .when(
            Role::Worker,
            |_| true,
            StubReply::text("the change is committed"),
        )
}

fn tool_replies(request: &ChatRequest) -> usize {
    request
        .messages
        .iter()
        .filter(|message| matches!(message, Message::Tool { .. }))
        .count()
}

fn tool_call(name: &str, arguments: Value) -> ToolCall {
    ToolCall {
        id: format!("call-{name}"),
        kind: ToolKind::Function,
        function: FunctionCall {
            name: name.to_owned(),
            arguments: arguments.to_string(),
        },
    }
}

fn worker_config(base: &str, token: &str) -> WorkerConfig {
    WorkerConfig {
        server: base.to_owned(),
        token: token.to_owned(),
        test_command: None,
        max_steps: 5,
        wait: WAIT,
        heartbeat: Duration::from_secs(1),
        poll: POLL,
    }
}

fn spawn_worker(
    base: &str,
    token: &'static str,
    gate: watch::Receiver<bool>,
) -> JoinHandle<Outcome> {
    let config = worker_config(base, token);
    tokio::spawn(async move {
        let model = Worker::new(gate);
        run(&config, &model).await
    })
}

/// A repository with `main` as the protected branch and the user's checkout on `user-work`.
fn user_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temporary repository");
    let root = dir.path();
    git(root, &["init", "-q", "-b", "main"]);
    git(root, &["config", "user.name", "Test User"]);
    git(root, &["config", "user.email", "test@example.test"]);
    // Windows checkouts convert to CRLF by default, and the file is compared byte for byte.
    git(root, &["config", "core.autocrlf", "false"]);
    fs::create_dir_all(root.join(MACHINE)).expect("machine directory");
    fs::write(root.join(FILE), ORIGINAL).expect("fixture");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "initial"]);
    git(root, &["switch", "-q", "-c", "user-work"]);
    dir
}

/// Runs git in `dir` and returns its stdout. Panics with git's stderr when the command fails.
/// A lock held by the server's own git is contention, so it is retried.
fn git(dir: &Path, args: &[&str]) -> String {
    for attempt in 1..=5 {
        let output = Process::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("git runs");
        if output.status.success() {
            return String::from_utf8(output.stdout).expect("git prints UTF-8");
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("index.lock") || attempt == 5 {
            panic!("git {args:?} failed: {stderr}");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    unreachable!("the loop returns or panics")
}

fn worktree_dir(root: &Path, task: TaskId) -> PathBuf {
    root.join(".acan").join("worktrees").join(task.to_string())
}

/// Writes the token file before the first start, so the test holds an owner and two agents.
fn write_tokens(root: &Path) {
    let state = root.join(".acan");
    fs::create_dir_all(&state).expect("state directory");
    fs::write(
        state.join("tokens.json"),
        format!(
            r#"[
              {{"token": "{OWNER_TOKEN}", "principal": {{"kind": "member", "id": "owner"}}}},
              {{"token": "{ONE_TOKEN}", "principal": {{"kind": "agent", "id": "{ONE}"}}}},
              {{"token": "{TWO_TOKEN}", "principal": {{"kind": "agent", "id": "{TWO}"}}}}
            ]"#
        ),
    )
    .expect("tokens file");
}

async fn start_origin(root: &Path) -> Running {
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
    start(config, Foreman, UsageLedger::default())
        .await
        .expect("the origin starts on the test repository")
}

fn base_url(running: &Running) -> String {
    format!("http://{}", running.local_addr)
}

fn client() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new)
}

/// Sends one member command over HTTP as the owner.
async fn owner_command(running: &Running, cmd: &HumanCmd) -> reqwest::Response {
    client()
        .post(format!("{}/api/commands", base_url(running)))
        .bearer_auth(OWNER_TOKEN)
        .json(cmd)
        .send()
        .await
        .expect("the command is sent")
}

async fn post_order(running: &Running) {
    let response = owner_command(
        running,
        &HumanCmd::PostOrder {
            machine: MachineKey::from_str(MACHINE).expect("valid machine key"),
            sentence: "change the loader".to_owned(),
            witnesses: vec![WitnessSpec {
                check: Check::Test {
                    command: WITNESS.to_owned(),
                },
                reason: "the built commit carries the change".to_owned(),
            }],
        },
    )
    .await;
    assert!(
        response.status().is_success(),
        "the order is posted: {}",
        response.status()
    );
}

async fn register(running: &Running, agent: &str) {
    let response = owner_command(
        running,
        &HumanCmd::RegisterAgent {
            agent: AgentId::from_str(agent).expect("valid agent id"),
        },
    )
    .await;
    assert!(
        response.status().is_success(),
        "{agent} is registered: {}",
        response.status()
    );
}

/// The stored log, replayed to a world. Read through the owner's events route.
async fn events(running: &Running) -> Vec<Stamped> {
    client()
        .get(format!("{}/api/events", base_url(running)))
        .bearer_auth(OWNER_TOKEN)
        .send()
        .await
        .expect("the events request is sent")
        .error_for_status()
        .expect("the owner may read the log")
        .json()
        .await
        .expect("the log is JSON")
}

/// Polls the stored log until `done` accepts it. A timeout names the wait that stalled.
async fn settle<T>(
    running: &Running,
    what: &str,
    done: impl Fn(&[Stamped], &World) -> Option<T>,
) -> T {
    timeout(WAIT, async {
        loop {
            let log = events(running).await;
            let origin = Origin::replay(log.iter().cloned()).expect("the stored log replays");
            if let Some(found) = done(&log, origin.world()) {
                return found;
            }
            sleep(POLL).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for {what}"))
}

fn tasks_of_world(world: &World) -> Vec<TaskId> {
    world
        .orders()
        .flat_map(|order| world.tasks_of(order.id))
        .map(|task| task.id)
        .collect()
}

/// The task `agent` holds a claim on, if any.
fn held_by(world: &World, agent: &AgentId) -> Option<TaskId> {
    world
        .orders()
        .flat_map(|order| world.tasks_of(order.id))
        .find_map(|task| match &task.state {
            TaskState::Claimed { claim, .. } if &claim.lease.holder == agent => Some(task.id),
            _ => None,
        })
}

fn is_stage(world: &World, task: TaskId, check: fn(&Stage) -> bool) -> bool {
    matches!(
        world.task(task).map(|found| &found.state),
        Some(TaskState::Claimed { stage, .. }) if check(stage)
    )
}

fn is_declared(stage: &Stage) -> bool {
    matches!(stage, Stage::Declared { .. })
}

fn is_building(stage: &Stage) -> bool {
    matches!(stage, Stage::Building { .. })
}

/// The first clash in the log, with the key's two tasks in `a`, `b` order.
fn clash_parties(log: &[Stamped]) -> Option<(ClashId, TaskId, TaskId)> {
    log.iter().find_map(|stamped| match &stamped.event {
        Event::ClashFound { clash, key, .. } => Some((*clash, key.a(), key.b())),
        _ => None,
    })
}

/// Whether a task was cleared to build after the first clash in the log.
fn cleared_after_clash(log: &[Stamped]) -> bool {
    log.iter()
        .skip_while(|stamped| !matches!(stamped.event, Event::ClashFound { .. }))
        .any(|stamped| matches!(stamped.event, Event::BuildCleared { .. }))
}

/// Waits until the task's worktree is checked out. Git creates the directory and the `.git`
/// file before it writes the index, so a status read in that window lists every file as deleted.
async fn wait_for_checkout(path: &Path) {
    timeout(WAIT, async {
        loop {
            if path.join(".git").exists() && git_status(path) == Some(String::new()) {
                return;
            }
            sleep(POLL).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("the worktree {} was never checked out", path.display()));
}

/// The porcelain status of `dir`, or `None` while git cannot report it.
fn git_status(dir: &Path) -> Option<String> {
    let output = Process::new("git")
        .arg("-C")
        .arg(dir)
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_workers_on_one_file_clash_before_either_writes() {
    let repo = user_repo();
    let root = repo.path();
    let running = start_origin(root).await;
    let base = base_url(&running);
    post_order(&running).await;
    register(&running, ONE).await;
    register(&running, TWO).await;
    let tasks = settle(&running, "the split into two tasks", |_, world| {
        let tasks = tasks_of_world(world);
        (tasks.len() == 2).then_some(tasks)
    })
    .await;
    assert_eq!(tasks.len(), 2, "the order splits into two tasks");

    let (gate, open) = watch::channel(false);
    let one = spawn_worker(&base, ONE_TOKEN, open.clone());
    let two = spawn_worker(&base, TWO_TOKEN, open);

    let first = settle(&running, "agent-1 to claim a task", |_, world| {
        held_by(world, &AgentId::from_str(ONE).expect("valid agent id"))
    })
    .await;
    let second = settle(&running, "agent-2 to claim a task", |_, world| {
        held_by(world, &AgentId::from_str(TWO).expect("valid agent id"))
    })
    .await;
    assert_ne!(first, second, "each worker claims its own task");

    let (clash, key_a, _) = settle(&running, "both declarations to clash", |log, world| {
        let found = clash_parties(log)?;
        let both_declared = [first, second]
            .iter()
            .all(|task| is_stage(world, *task, is_declared));
        both_declared.then_some(found)
    })
    .await;
    // The menu offers the serialize in the clash key's order, so the key's `a` goes first.
    let (first_task, first_worker, second_task, second_worker) = if key_a == first {
        (first, one, second, two)
    } else {
        (second, two, first, one)
    };
    for task in [first, second] {
        let worktree = worktree_dir(root, task);
        wait_for_checkout(&worktree).await;
        assert_eq!(
            git(&worktree, &["status", "--porcelain"]),
            "",
            "{task} has no change while the clash is open"
        );
    }
    let log = events(&running).await;
    assert!(
        !cleared_after_clash(&log),
        "no task is cleared to build while the clash is open"
    );

    gate.send_replace(true);
    sleep(HOLD).await;
    let log = events(&running).await;
    assert!(
        !cleared_after_clash(&log),
        "opening the gate does not clear a task past the clash"
    );
    for task in [first, second] {
        let worktree = worktree_dir(root, task);
        assert_eq!(
            fs::read_to_string(worktree.join(FILE)).expect("the file is readable"),
            ORIGINAL,
            "{task} has not written {FILE}"
        );
    }

    let response = owner_command(
        &running,
        &HumanCmd::SettleClash {
            clash,
            choice: Settlement::Serialize {
                first: first_task,
                then: second_task,
            },
        },
    )
    .await;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    assert!(status.is_success(), "the clash settles: {status} {body}");

    settle(&running, "the first task to be cleared", |log, world| {
        let cleared = log.iter().any(
            |stamped| matches!(&stamped.event, Event::BuildCleared { task } if *task == first_task),
        );
        (cleared && is_stage(world, first_task, is_building)).then_some(())
    })
    .await;
    assert!(
        !is_stage(&running_world(&running).await, second_task, is_building),
        "the second task waits for the first to land"
    );

    let outcome = timeout(WAIT, first_worker)
        .await
        .expect("the first worker finishes in time")
        .expect("the first worker does not panic");
    assert!(
        matches!(&outcome.exit, Exit::Built { task, .. } if task == &first_task.to_string()),
        "the first worker builds its task: {}",
        outcome.exit
    );
    assert_eq!(
        fs::read_to_string(worktree_dir(root, first_task).join(FILE))
            .expect("the file is readable"),
        CHANGED,
        "the first worker wrote its change"
    );
    second_worker.abort();
    running.stop().await.expect("the origin stops");
}

async fn running_world(running: &Running) -> World {
    let log = events(running).await;
    Origin::replay(log)
        .expect("the stored log replays")
        .world()
        .clone()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mcp_refuses_a_wrong_or_member_token_as_the_http_routes_do() {
    let repo = user_repo();
    let running = start_origin(repo.path()).await;
    let base = base_url(&running);
    let initialize = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} });

    let wrong = mcp_post(&running, "acan_not_a_token", &initialize).await;
    assert_eq!(wrong.status(), reqwest::StatusCode::UNAUTHORIZED);
    let wrong_body: Value = wrong.json().await.expect("the refusal is JSON");
    let http = client()
        .get(format!("{base}/api/tasks/t-0/worktree"))
        .bearer_auth("acan_not_a_token")
        .send()
        .await
        .expect("the worktree request is sent");
    assert_eq!(http.status(), reqwest::StatusCode::UNAUTHORIZED);
    let http_body: Value = http.json().await.expect("the refusal is JSON");
    assert_eq!(
        wrong_body, http_body,
        "MCP refuses a bad token as HTTP does"
    );

    let member = mcp_post(&running, OWNER_TOKEN, &initialize).await;
    assert_eq!(
        member.status(),
        reqwest::StatusCode::FORBIDDEN,
        "a member token may not use the agent endpoint"
    );

    let refused = McpClient::new(&base, "acan_not_a_token".to_owned())
        .connect()
        .await
        .expect_err("the harness is refused with a bad token");
    assert!(
        refused.to_string().contains("refused the agent token"),
        "the harness names the refusal: {refused:#}"
    );
    running.stop().await.expect("the origin stops");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tools_list_offers_the_nine_tools() {
    let repo = user_repo();
    let running = start_origin(repo.path()).await;
    let response = mcp_post(
        &running,
        ONE_TOKEN,
        &json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
    )
    .await;
    assert!(response.status().is_success());
    let body: Value = response.json().await.expect("the listing is JSON");
    let mut names: Vec<&str> = body["result"]["tools"]
        .as_array()
        .expect("the listing has tools")
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "claim_task",
            "declare_changes",
            "get_task",
            "get_worktree",
            "heartbeat_task",
            "list_tasks",
            "post_readback",
            "release_task",
            "report_built",
        ]
    );
    running.stop().await.expect("the origin stops");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_stale_fence_is_refused_with_the_rejection_text() {
    let repo = user_repo();
    let root = repo.path();
    let running = start_origin(root).await;
    let base = base_url(&running);
    post_order(&running).await;
    register(&running, ONE).await;
    let task = settle(&running, "the split", |_, world| {
        tasks_of_world(world).first().copied()
    })
    .await;

    let mcp = McpClient::new(&base, ONE_TOKEN.to_owned());
    mcp.connect().await.expect("the harness connects");
    let posted = mcp
        .call(
            "post_readback",
            json!({
                "task": task.to_string(),
                "summary": "change the loader",
                "touches": [{ "at": "machine", "machine": MACHINE }],
            }),
        )
        .await
        .expect("the readback call completes");
    assert!(matches!(posted, Reply::Done(_)), "the readback is accepted");
    timeout(WAIT, async {
        loop {
            let detail = mcp
                .call("get_task", json!({ "task": task.to_string() }))
                .await
                .expect("get_task completes");
            if let Reply::Done(detail) = detail
                && detail["your_readback"]["status"] == "match"
            {
                return;
            }
            sleep(POLL).await;
        }
    })
    .await
    .expect("the foreman matches the readback");
    let claimed = mcp
        .call("claim_task", json!({ "task": task.to_string() }))
        .await
        .expect("the claim call completes");
    let Reply::Done(claimed) = claimed else {
        panic!("the claim is accepted");
    };
    let held = u32::try_from(
        claimed["generation"]
            .as_u64()
            .expect("the claim names a generation"),
    )
    .expect("the generation fits in u32");

    let stale = mcp
        .call(
            "declare_changes",
            json!({
                "task": task.to_string(),
                "generation": held + 1,
                "files": [FILE],
            }),
        )
        .await
        .expect("the declare call completes");
    let expected = Rejection::StaleGeneration {
        current: Some(Generation::new(held)),
    }
    .to_string();
    assert_eq!(
        stale,
        Reply::Refused(expected.clone()),
        "a stale fence is refused with the rejection text"
    );

    let raw = mcp_post(
        &running,
        ONE_TOKEN,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "heartbeat_task",
                "arguments": { "task": task.to_string(), "generation": held + 1 },
            },
        }),
    )
    .await;
    let body: Value = raw.json().await.expect("the answer is JSON");
    assert!(
        body.get("error").is_none(),
        "a rejection is not a transport error"
    );
    assert_eq!(body["result"]["isError"], json!(true));
    assert_eq!(body["result"]["content"][0]["text"], json!(expected));

    let renewed = mcp
        .call(
            "heartbeat_task",
            json!({ "task": task.to_string(), "generation": held }),
        )
        .await
        .expect("the renewal call completes");
    assert!(
        matches!(renewed, Reply::Done(_)),
        "the holder's current fence renews"
    );
    running.stop().await.expect("the origin stops");
}

/// A worker whose plan answers at once and whose edit step fails, as a model outage would.
struct FailingEdit {
    script: Stub,
}

impl ModelClient for FailingEdit {
    async fn chat(&self, role: Role, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        if request.tools.is_empty() {
            return self.script.chat(role, request).await;
        }
        Err(LlmError::NoScriptedReply { role })
    }
}

/// A worker that announces its first edit step, then waits on the gate as `Worker` does. The
/// announcement tells the test the worker is past clearance and the worktree, inside the edit.
struct AnnouncedEdit {
    script: Stub,
    gate: watch::Receiver<bool>,
    entered: watch::Sender<bool>,
}

impl ModelClient for AnnouncedEdit {
    async fn chat(&self, role: Role, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        if !request.tools.is_empty() {
            self.entered.send_replace(true);
            let mut gate = self.gate.clone();
            let _ = gate.wait_for(|open| *open).await;
        }
        self.script.chat(role, request).await
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_lost_claim_stops_the_work_and_nothing_is_written() {
    let repo = user_repo();
    let root = repo.path();
    let running = start_origin(root).await;
    let base = base_url(&running);
    post_order(&running).await;
    register(&running, ONE).await;
    settle(&running, "the split", |_, world| {
        (tasks_of_world(world).len() == 2).then_some(())
    })
    .await;

    // The gate stays shut, so the worker waits inside its first edit step, holding its claim.
    let (gate, open) = watch::channel(false);
    let (entered_tx, mut entered) = watch::channel(false);
    let config = worker_config(&base, ONE_TOKEN);
    let worker = tokio::spawn(async move {
        let model = AnnouncedEdit {
            script: worker_script(),
            gate: open,
            entered: entered_tx,
        };
        run(&config, &model).await
    });
    timeout(WAIT, entered.wait_for(|seen| *seen))
        .await
        .expect("the worker reaches its edit step")
        .expect("the worker is still running");
    let one = AgentId::from_str(ONE).expect("valid agent id");
    let task = settle(&running, "agent-1 to hold the task", |_, world| {
        held_by(world, &one)
    })
    .await;
    let worktree = worktree_dir(root, task);
    wait_for_checkout(&worktree).await;

    // The claim ends at the origin, as an expired lease would end it. The worker's next renewal
    // is refused, and that refusal is what stops its work.
    let holder = McpClient::new(&base, ONE_TOKEN.to_owned());
    holder.connect().await.expect("the holder connects");
    let Reply::Done(detail) = holder
        .call("get_task", json!({ "task": task.to_string() }))
        .await
        .expect("get_task completes")
    else {
        panic!("get_task is accepted for the holder");
    };
    let generation = detail["generation"]
        .as_u64()
        .expect("the claim names a generation");
    let released = holder
        .call(
            "release_task",
            json!({ "task": task.to_string(), "generation": generation }),
        )
        .await
        .expect("the release completes");
    assert!(
        matches!(released, Reply::Done(_)),
        "the claim is released: {released:?}"
    );

    let outcome = timeout(WAIT, worker)
        .await
        .expect("the worker stops once its lease is refused")
        .expect("the worker does not panic");
    assert!(
        matches!(&outcome.exit, Exit::Lost { task: lost, .. } if lost == &task.to_string()),
        "the worker reports the loss: {}",
        outcome.exit
    );

    // The gate opens only after the work has stopped, so nothing the model does can write now.
    // The origin may already have removed the released task's worktree, so a missing file is
    // fine; a file with the change in it is not.
    gate.send_replace(true);
    sleep(HOLD).await;
    if let Ok(text) = fs::read_to_string(worktree.join(FILE)) {
        assert_eq!(text, ORIGINAL, "the lost worker wrote nothing");
    }
    running.stop().await.expect("the origin stops");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_model_error_releases_the_claim() {
    let repo = user_repo();
    let running = start_origin(repo.path()).await;
    let base = base_url(&running);
    post_order(&running).await;
    register(&running, ONE).await;
    settle(&running, "the split", |_, world| {
        (tasks_of_world(world).len() == 2).then_some(())
    })
    .await;

    let config = worker_config(&base, ONE_TOKEN);
    let worker = tokio::spawn(async move {
        let model = FailingEdit {
            script: worker_script(),
        };
        run(&config, &model).await
    });
    let outcome = timeout(WAIT, worker)
        .await
        .expect("the worker finishes after the model fails")
        .expect("the worker does not panic");
    assert!(
        matches!(&outcome.exit, Exit::Failed { reason } if reason.contains("the edit step failed")),
        "the worker names the failed step: {}",
        outcome.exit
    );
    assert!(
        outcome
            .steps
            .iter()
            .any(|step| step.starts_with("released ")),
        "the claim was released: {:?}",
        outcome.steps
    );
    let world = running_world(&running).await;
    assert!(
        held_by(&world, &AgentId::from_str(ONE).expect("valid agent id")).is_none(),
        "the origin holds no claim for agent-1"
    );
    running.stop().await.expect("the origin stops");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_envelope_is_checked_before_any_method_runs() {
    let repo = user_repo();
    let running = start_origin(repo.path()).await;
    let bad_request = reqwest::StatusCode::BAD_REQUEST;

    let wrong_version = mcp_post(
        &running,
        ONE_TOKEN,
        &json!({ "jsonrpc": "1.0", "id": 7, "method": "ping" }),
    )
    .await;
    assert_eq!(wrong_version.status(), bad_request);
    let body: Value = wrong_version.json().await.expect("the refusal is JSON");
    assert_eq!(body["error"]["code"], json!(-32600));
    assert_eq!(body["id"], json!(7), "the refusal answers the request's id");

    let no_version = mcp_post(&running, ONE_TOKEN, &json!({ "id": 8, "method": "ping" })).await;
    assert_eq!(no_version.status(), bad_request);
    let body: Value = no_version.json().await.expect("the refusal is JSON");
    assert_eq!(body["error"]["code"], json!(-32600));

    let batch = mcp_post(
        &running,
        ONE_TOKEN,
        &json!([{ "jsonrpc": "2.0", "id": 9, "method": "ping" }]),
    )
    .await;
    assert_eq!(batch.status(), bad_request);
    let body: Value = batch.json().await.expect("the refusal is JSON");
    assert_eq!(body["error"]["code"], json!(-32600));
    assert_eq!(
        body["id"],
        Value::Null,
        "a message that is not an object has no id"
    );

    let object_id = mcp_post(
        &running,
        ONE_TOKEN,
        &json!({ "jsonrpc": "2.0", "id": { "x": 1 }, "method": "ping" }),
    )
    .await;
    assert_eq!(object_id.status(), bad_request);
    let body: Value = object_id.json().await.expect("the refusal is JSON");
    assert_eq!(body["error"]["code"], json!(-32600));
    assert_eq!(body["id"], Value::Null, "an object id is not a valid id");

    // A null id is still a request, so it gets an answer that carries the null id.
    let null_id = mcp_post(
        &running,
        ONE_TOKEN,
        &json!({ "jsonrpc": "2.0", "id": null, "method": "ping" }),
    )
    .await;
    assert_eq!(null_id.status(), reqwest::StatusCode::OK);
    let body: Value = null_id.json().await.expect("the answer is JSON");
    assert_eq!(
        body.get("id"),
        Some(&Value::Null),
        "the answer carries null"
    );
    assert_eq!(body["result"], json!({}));

    // A notification has no id and gets no reply body.
    let notice = mcp_post(
        &running,
        ONE_TOKEN,
        &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
    )
    .await;
    assert_eq!(notice.status(), reqwest::StatusCode::ACCEPTED);
    running.stop().await.expect("the origin stops");
}

async fn mcp_post(running: &Running, token: &str, body: &Value) -> reqwest::Response {
    client()
        .post(format!("{}/mcp", base_url(running)))
        .bearer_auth(token)
        .json(body)
        .send()
        .await
        .expect("the MCP request is sent")
}
