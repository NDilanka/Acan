//! The model jobs against an in-memory world. Prompts carry what the model judges with, and a
//! failed live call answers with the offline reply and is metered as offline.

mod support;

use std::collections::{BTreeMap, BTreeSet};

use acan_core::{
    Command, Judgement, OrderId, OrderRev, ReadbackId, ReadbackVerdict, SystemCmd, TaskId,
    TaskSpec, WitnessId,
};
use acan_llm::{ApiKey, ApiKeys, ChatRequest, LlmConfig, ModelClient, Role};
use acan_server::Models;
use acan_server::model::{self, ReplyError, SplitReply, SplitTask};
use serde_json::{Value, json};
use support::Demo;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const ORDER_SENTENCE: &str = "make the greeting configurable from the settings file";

fn user_json(request: &ChatRequest) -> Value {
    let text = request
        .last_user_text()
        .expect("every model request carries its context as the user message");
    serde_json::from_str(text).expect("the context is JSON")
}

/// Splits `order` into two fixed tasks, the way a live split would.
fn split_in_two(demo: &mut Demo, order: OrderId) -> [TaskId; 2] {
    let rev = order_rev(demo, order);
    let spec = |title: &str| TaskSpec {
        title: title.to_owned(),
        brief: format!("{title}, then run the greet tests"),
        scope: vec![support::machine(support::UI)],
    };
    demo.apply(Command::System(SystemCmd::SplitReady {
        order,
        rev,
        tasks: vec![spec("Read the greeting"), spec("Add the setting")],
    }));
    match demo.tasks_of(order).as_slice() {
        [first, second] => [*first, *second],
        other => panic!("two tasks were expected, got {other:?}"),
    }
}

fn order_rev(demo: &Demo, order: OrderId) -> OrderRev {
    demo.world().order(order).expect("the order exists").rev
}

/// Stands in for the foreman's review: every readback matches every witness at the current
/// revision.
fn approve(demo: &mut Demo, order: OrderId, readbacks: &BTreeSet<ReadbackId>) {
    let (rev, witnesses) = {
        let found = demo.world().order(order).expect("the order exists");
        (found.rev, found.witnesses.clone())
    };
    let verdicts = readbacks
        .iter()
        .map(|readback| {
            let per_witness: BTreeMap<WitnessId, Judgement> = witnesses
                .iter()
                .map(|witness| (*witness, Judgement::Match))
                .collect();
            (*readback, ReadbackVerdict { rev, per_witness })
        })
        .collect();
    demo.apply(Command::System(SystemCmd::ReadbacksReviewed {
        order,
        rev,
        verdicts,
        question: None,
    }));
}

#[test]
fn the_split_prompt_lists_the_plant_machines_and_the_order() {
    let mut demo = Demo::open();
    let order = demo.post_order(ORDER_SENTENCE);
    let machines = model::plant_machines(demo.index());

    let request = model::split_request(demo.world(), order, &machines).expect("the order exists");
    let context = user_json(&request);

    assert_eq!(
        context["machines"],
        json!(["api", "ui"]),
        "the model may name only machines on the plant"
    );
    assert_eq!(context["machine"], "ui", "the order's own machine");
    assert_eq!(context["order"], ORDER_SENTENCE);
    assert_eq!(
        context["witnesses"][0]["reason"], "the greeting keeps its default",
        "the witnesses the split must keep"
    );
}

#[test]
fn a_split_that_names_a_machine_off_the_plant_is_refused() {
    let mut demo = Demo::open();
    let order = demo.post_order(ORDER_SENTENCE);
    let machines = model::plant_machines(demo.index());
    let rev = order_rev(&demo, order);
    let reply = SplitReply {
        tasks: vec![SplitTask {
            title: "Add a database column".to_owned(),
            brief: "Add the column and its migration".to_owned(),
            scope: vec!["db".to_owned()],
        }],
    };

    let err =
        model::split_result(order, rev, &machines, reply).expect_err("db is not on the plant");

    assert!(
        matches!(&err, ReplyError::UnknownMachine(name) if name == "db"),
        "got {err:?}"
    );
}

#[test]
fn the_review_prompt_carries_the_witnesses_and_each_readback() {
    let mut demo = Demo::open();
    let order = demo.post_order(ORDER_SENTENCE);
    let [first, second] = split_in_two(&mut demo, order);
    let readbacks = support::read_back(&mut demo, [first, second]);

    let request = model::review_request(demo.world(), order, &readbacks).expect("the order exists");
    let context = user_json(&request);

    assert_eq!(
        context["witnesses"][0]["reason"], "the greeting keeps its default",
        "each judgement is made against a witness"
    );
    let judged: BTreeSet<String> = context["readbacks"]
        .as_array()
        .expect("readbacks is a list")
        .iter()
        .map(|readback| {
            readback["readback"]
                .as_str()
                .expect("each readback has an id")
                .to_owned()
        })
        .collect();
    let expected: BTreeSet<String> = readbacks.iter().map(ToString::to_string).collect();
    assert_eq!(judged, expected, "every readback is in the prompt");
}

#[test]
fn the_explain_prompt_carries_both_declared_footprints_and_the_menu() {
    let mut demo = Demo::open();
    let order = demo.post_order(ORDER_SENTENCE);
    let tasks = split_in_two(&mut demo, order);
    let readbacks = support::read_back(&mut demo, tasks);
    approve(&mut demo, order, &readbacks);
    let clash = support::claim_and_declare(&mut demo, tasks);

    let request = model::explain_request(demo.world(), clash).expect("the clash exists");
    let context = user_json(&request);

    let declared = json!([{ "file": support::GREET_FILE, "name": support::GREET }]);
    let parties = context["tasks"].as_array().expect("two parties");
    assert_eq!(parties.len(), 2, "both tasks are in the prompt");
    for party in parties {
        assert_eq!(
            party["declared"]["symbols"], declared,
            "each party's declared symbols are in the prompt"
        );
    }
    assert!(
        context["menu"]
            .as_array()
            .is_some_and(|menu| !menu.is_empty()),
        "the menu of legal choices is in the prompt"
    );
    assert_eq!(
        context["witnesses"][0]["reason"], "the greeting keeps its default",
        "the witnesses a breaking choice would name are in the prompt"
    );
}

/// A loopback address with nothing listening, so a request to it is refused at once.
async fn unreachable_url() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a local port is free");
    let address = listener.local_addr().expect("the listener has an address");
    drop(listener);
    format!("http://{address}/v1")
}

/// Reads one HTTP request, headers and body, so the reply is never written into an unread body.
async fn drain_request(socket: &mut TcpStream) {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    loop {
        let read = socket
            .read(&mut chunk)
            .await
            .expect("the request can be read");
        if read == 0 {
            return;
        }
        buffer.extend_from_slice(&chunk[..read]);
        let Some(end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        let headers = String::from_utf8_lossy(&buffer[..end]).to_ascii_lowercase();
        let length = headers
            .lines()
            .find_map(|line| line.strip_prefix("content-length:"))
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        if buffer.len() >= end + 4 + length {
            return;
        }
    }
}

/// Serves every connection with one chat-completions reply whose text is `content`, and reports
/// 120 prompt and 30 completion tokens. Returns the base URL for the provider.
async fn serve_reply(content: &str) -> String {
    let reply = json!({
        "choices": [{ "message": { "role": "assistant", "content": content }, "finish_reason": "stop" }],
        "usage": { "prompt_tokens": 120, "completion_tokens": 30 },
    })
    .to_string();
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a local port is free");
    let address = listener.local_addr().expect("the listener has an address");
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let reply = reply.clone();
            tokio::spawn(async move {
                drain_request(&mut socket).await;
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{reply}",
                    reply.len()
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.shutdown().await;
            });
        }
    });
    format!("http://{address}/v1")
}

/// A routing file where every role uses one `baseten` provider at `base_url`.
fn config(base_url: &str) -> LlmConfig {
    LlmConfig::from_toml_str(&format!(
        r#"
[[providers]]
name = "baseten"
base_url = "{base_url}"
api_key_env = "BASETEN_API_KEY"
default_model = "meta-models/muse-spark-open-1026"
timeout_secs = 5

[routing.foreman]
provider = "baseten"

[routing.worker]
provider = "baseten"

[routing.fast]
provider = "baseten"
"#
    ))
    .expect("the test config is valid")
}

fn keys(with_key: bool) -> ApiKeys {
    let mut keys = ApiKeys::default();
    if with_key {
        keys.insert("baseten", ApiKey::new("test-key-not-real"));
    }
    keys
}

/// The split prompt for a fresh order, with the order and the machine list it was built from.
fn split_case() -> (Demo, OrderId, Vec<acan_core::MachineKey>, ChatRequest) {
    let mut demo = Demo::open();
    let order = demo.post_order(ORDER_SENTENCE);
    let machines = model::plant_machines(demo.index());
    let request = model::split_request(demo.world(), order, &machines).expect("the order exists");
    (demo, order, machines, request)
}

#[tokio::test]
async fn an_unreachable_provider_answers_with_the_offline_split() {
    let url = unreachable_url().await;
    let models = Models::live(config(&url), keys(true)).expect("the client builds");
    let (demo, order, machines, request) = split_case();

    let split = models
        .chat_json::<SplitReply>(Role::Foreman, request)
        .await
        .expect("the offline reply answers the call");
    let cmd = model::split_result(order, order_rev(&demo, order), &machines, split.value)
        .expect("the offline split names the order's machine");

    assert!(
        matches!(&cmd, SystemCmd::SplitReady { tasks, .. } if tasks.len() == 1),
        "got {cmd:?}"
    );
    let records = models.ledger().records();
    assert_eq!(records.len(), 1, "the failed live call is not metered");
    assert_eq!(
        records[0].provider, "offline",
        "the answer is marked offline"
    );
    assert_eq!(records[0].role, Role::Foreman);
}

#[tokio::test]
async fn a_missing_key_answers_with_the_offline_split() {
    let url = unreachable_url().await;
    let models = Models::live(config(&url), keys(false)).expect("the client builds");
    let (_demo, _order, _machines, request) = split_case();

    let split = models
        .chat_json::<SplitReply>(Role::Foreman, request)
        .await
        .expect("the offline reply answers the call");

    assert_eq!(split.value.tasks.len(), 1, "the offline split is one task");
    let records = models.ledger().records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].provider, "offline");
}

#[tokio::test]
async fn a_reply_that_breaks_the_schema_falls_back_and_both_calls_are_metered() {
    let url = serve_reply("this is prose, not the split the schema asks for").await;
    let models = Models::live(config(&url), keys(true)).expect("the client builds");
    let (_demo, _order, _machines, request) = split_case();

    let split = models
        .chat_json::<SplitReply>(Role::Foreman, request)
        .await
        .expect("the offline reply answers the call");

    assert_eq!(split.value.tasks.len(), 1, "the offline split answered");
    let records = models.ledger().records();
    assert_eq!(
        records.len(),
        2,
        "the live call and the offline call are both metered"
    );
    assert_eq!(records[0].provider, "baseten", "the live call was metered");
    assert_eq!(records[0].usage.prompt_tokens, 120);
    assert_eq!(records[0].usage.completion_tokens, 30);
    assert_eq!(
        records[1].provider, "offline",
        "the answer is marked offline"
    );
}

#[tokio::test]
async fn a_live_reply_that_matches_the_schema_is_used_as_live() {
    let content = json!({
        "tasks": [{ "title": "Read the greeting", "brief": "Read it from settings", "scope": ["ui"] }]
    })
    .to_string();
    let url = serve_reply(&content).await;
    let models = Models::live(config(&url), keys(true)).expect("the client builds");
    let (_demo, _order, _machines, request) = split_case();

    let split = models
        .chat_json::<SplitReply>(Role::Foreman, request)
        .await
        .expect("the live reply parses");

    assert_eq!(
        split.value.tasks[0].title, "Read the greeting",
        "the live reply is used"
    );
    let records = models.ledger().records();
    assert_eq!(records.len(), 1, "one live call, no fallback");
    assert_eq!(records[0].provider, "baseten");
    assert_eq!(records[0].model, "meta-models/muse-spark-open-1026");
}
