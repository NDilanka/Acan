//! Calls the provider configured for one role and prints the reply and usage.
//!
//! Run with `cargo run -p acan-llm --example smoke -- --role foreman`.
//! Keys come from the environment or `crates/acan-llm/.env`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use acan_llm::{
    ApiKeys, ChatRequest, LlmConfig, Message, ModelClient, OpenAiCompatible, Role, UnknownRole,
};

const DEFAULT_PROMPT: &str = "Reply with the single word OK.";

struct Args {
    role: Role,
    config: PathBuf,
    prompt: String,
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut parsed = Args {
        role: Role::Foreman,
        config: Path::new(env!("CARGO_MANIFEST_DIR")).join("llm.toml"),
        prompt: DEFAULT_PROMPT.to_owned(),
    };
    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--role" => {
                parsed.role = value.parse().map_err(|err: UnknownRole| err.to_string())?;
            }
            "--config" => parsed.config = PathBuf::from(value),
            "--prompt" => parsed.prompt = value,
            other => return Err(format!("unknown flag {other}")),
        }
    }
    Ok(parsed)
}

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::from_path(Path::new(env!("CARGO_MANIFEST_DIR")).join(".env")).ok();
    dotenvy::dotenv().ok();
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
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

async fn run() -> Result<(), String> {
    let args = parse_args(std::env::args().skip(1))?;
    let config = LlmConfig::load(&args.config)
        .map_err(|err| format!("{err}, copy llm.example.toml to llm.toml or pass --config"))?;
    let keys = ApiKeys::from_env(&config);
    let client = OpenAiCompatible::new(config, keys).map_err(|err| err.to_string())?;

    let request = ChatRequest {
        messages: vec![Message::user(args.prompt)],
        ..Default::default()
    };
    let response = client
        .chat(args.role, request)
        .await
        .map_err(|err| describe(&err))?;

    let call = &response.call;
    println!(
        "role {} provider {} model {}",
        call.role, call.provider, call.model
    );
    println!(
        "latency {} ms, prompt tokens {}, completion tokens {}",
        call.latency.as_millis(),
        call.usage.prompt_tokens,
        call.usage.completion_tokens
    );
    println!("reply:");
    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
