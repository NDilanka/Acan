//! Runs Acan's three model jobs once each against the configured provider, in an in-memory
//! world, and prints what each one returned.
//!
//! Run with `cargo run -p acan-server --example live_jobs`. The routing file defaults to
//! `crates/acan-llm/llm.toml`; `--config <path>` picks another. Keys come from the environment
//! or `crates/acan-llm/.env`, and are never printed.
//!
//! Each job prints the provider and model that answered, the latency, the tokens, and the parsed
//! reply. Every reply is then applied to the origin, so a reply the origin would refuse stops the
//! run with the origin's reason. A job a failed live call answered offline shows `provider
//! offline`, with a warning on stderr.

#[path = "../tests/support/mod.rs"]
mod support;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use acan_core::{ClashId, Command, OrderId, ReadbackId, SystemCmd, TaskId};
use acan_llm::{ApiKeys, CallRecord, LlmConfig, ModelClient, Role};
use acan_server::Models;
use acan_server::model::{self, ExplainReply, ReviewReply, SplitReply};
use support::Demo;

const ORDER: &str = "make the greeting configurable from the settings file";

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::WARN)
        .init();
    dotenvy::from_path(Path::new(env!("CARGO_MANIFEST_DIR")).join("../acan-llm/.env")).ok();
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn config_path() -> Result<PathBuf, String> {
    let mut args = std::env::args().skip(1);
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../acan-llm/llm.toml");
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--config" => {
                path = PathBuf::from(args.next().ok_or("--config needs a path")?);
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(path)
}

/// Adds the source chain, so a DNS or TLS cause shows up next to the variant name.
fn describe(err: &dyn std::error::Error) -> String {
    let mut text = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        text = format!("{text}, caused by {cause}");
        source = cause.source();
    }
    text
}

fn show(job: &str, call: &CallRecord) {
    println!("{job}: provider {} model {}", call.provider, call.model);
    println!(
        "  latency {} ms, prompt tokens {}, completion tokens {}",
        call.latency.as_millis(),
        call.usage.prompt_tokens,
        call.usage.completion_tokens
    );
    if call.provider == "offline" {
        println!("  the live call failed, so this is the offline reply");
    }
}

fn apply(demo: &mut Demo, cmd: SystemCmd) -> Result<(), String> {
    demo.try_apply(Command::System(cmd))
        .map(|_| ())
        .map_err(|rejection| format!("the origin refused the reply: {rejection}"))
}

async fn run() -> Result<(), String> {
    let path = config_path()?;
    let config = LlmConfig::load(&path)
        .map_err(|err| format!("{err}, copy llm.example.toml to llm.toml or pass --config"))?;
    let keys = ApiKeys::from_env(&config);
    let models = Models::live(config, keys).map_err(|err| err.to_string())?;

    let mut demo = Demo::open();
    let order = demo.post_order(ORDER);
    split(&models, &mut demo, order).await?;
    let [first, second] = two_tasks(&demo, order)?;
    let readbacks = support::read_back(&mut demo, [first, second]);
    review(&models, &mut demo, order, &readbacks).await?;
    if !cleared(&demo, order, &readbacks) {
        return Err(
            "the live review did not clear both readbacks, so the tasks cannot be claimed and \
             no clash forms; the explain job did not run"
                .to_owned(),
        );
    }
    let clash = support::claim_and_declare(&mut demo, [first, second]);
    explain(&models, &mut demo, clash).await?;
    println!("all three jobs parsed, and the origin accepted each reply");
    Ok(())
}

async fn split(models: &Models, demo: &mut Demo, order: OrderId) -> Result<(), String> {
    let machines = model::plant_machines(demo.index());
    let request = model::split_request(demo.world(), order, &machines)
        .ok_or("the order is missing from the world")?;
    let reply = models
        .chat_json::<SplitReply>(Role::Foreman, request)
        .await
        .map_err(|err| describe(&err))?;
    show("split", &reply.call);
    println!("  parsed: {:#?}", reply.value);
    let rev = demo
        .world()
        .order(order)
        .ok_or("the order is missing from the world")?
        .rev;
    let cmd =
        model::split_result(order, rev, &machines, reply.value).map_err(|err| err.to_string())?;
    apply(demo, cmd)?;
    println!("  the origin accepted the split");
    Ok(())
}

/// The first two tasks of the split. The demo reads back and claims two tasks, since a clash
/// needs two declarations.
fn two_tasks(demo: &Demo, order: OrderId) -> Result<[TaskId; 2], String> {
    match demo.tasks_of(order).as_slice() {
        [first, second, ..] => Ok([*first, *second]),
        other => Err(format!(
            "the split made {} task(s), and the demo needs two, so the review and explain jobs \
             did not run",
            other.len()
        )),
    }
}

async fn review(
    models: &Models,
    demo: &mut Demo,
    order: OrderId,
    readbacks: &BTreeSet<ReadbackId>,
) -> Result<(), String> {
    let request = model::review_request(demo.world(), order, readbacks)
        .ok_or("the order is missing from the world")?;
    let reply = models
        .chat_json::<ReviewReply>(Role::Foreman, request)
        .await
        .map_err(|err| describe(&err))?;
    show("review", &reply.call);
    println!("  parsed: {:#?}", reply.value);
    let (machine, rev) = {
        let found = demo
            .world()
            .order(order)
            .ok_or("the order is missing from the world")?;
        (found.machine.clone(), found.rev)
    };
    let cmd = model::review_result(order, &machine, rev, readbacks, reply.value)
        .map_err(|err| err.to_string())?;
    apply(demo, cmd)?;
    println!("  the origin accepted the review");
    Ok(())
}

/// Whether every readback of `readbacks` now carries a verdict that lets its task claim.
fn cleared(demo: &Demo, order: OrderId, readbacks: &BTreeSet<ReadbackId>) -> bool {
    let Some(found) = demo.world().order(order) else {
        return false;
    };
    readbacks.iter().all(|id| {
        demo.world()
            .readback(*id)
            .and_then(|readback| readback.verdict.as_ref())
            .is_some_and(|verdict| verdict.matches(found))
    })
}

async fn explain(models: &Models, demo: &mut Demo, clash: ClashId) -> Result<(), String> {
    let request =
        model::explain_request(demo.world(), clash).ok_or("the clash is missing from the world")?;
    let reply = models
        .chat_json::<ExplainReply>(Role::Fast, request)
        .await
        .map_err(|err| describe(&err))?;
    show("explain", &reply.call);
    println!("  parsed: {:#?}", reply.value);
    let explanation = model::explanation(reply.value).map_err(|err| err.to_string())?;
    apply(demo, SystemCmd::ClashExplained { clash, explanation })?;
    println!("  the origin accepted the explanation");
    Ok(())
}
