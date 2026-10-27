//! `acan-worker`: one harness worker. It takes a task from an acan origin over MCP and builds it
//! in its worktree. The step log goes to stderr; the final exit goes to stdout.

use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use acan_agent::{Exit, WorkerConfig, run};
use acan_llm::{ApiKeys, LlmConfig, OpenAiCompatible};
use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};

/// One worker run against one origin.
// No `Debug`: the struct holds the agent token, and a derived `Debug` would print it.
#[derive(Parser)]
#[command(name = "acan-worker", version, about = "Runs one acan worker over MCP")]
struct Cli {
    /// The origin's base URL, such as `http://127.0.0.1:7311`.
    #[arg(long)]
    server: String,
    /// The agent's bearer token.
    #[arg(long)]
    token: String,
    /// The role to run as. The worker is the only role.
    #[arg(long, value_enum, default_value_t = Role::Worker)]
    role: Role,
    /// The model routing file.
    #[arg(long, default_value = "crates/acan-llm/llm.toml")]
    llm: PathBuf,
    /// A file of provider keys. A missing file is fine.
    #[arg(long, default_value = ".env")]
    env: PathBuf,
    /// The command `run_tests` runs in the worktree. Without it, `run_tests` reports none is set.
    #[arg(long)]
    test_command: Option<String>,
    /// Seconds one wait may take: for work to appear, a verdict, or build clearance.
    #[arg(long, default_value_t = 600)]
    wait_secs: u64,
    /// Seconds between lease renewals. Keep it well under the origin's 60 second lease.
    #[arg(long, default_value_t = 20)]
    heartbeat_secs: u64,
    /// The most model calls the edit loop may make.
    #[arg(long, default_value_t = 30)]
    max_steps: usize,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Role {
    Worker,
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();
    match run_cli().await {
        Ok(exit) => {
            println!("{exit}");
            if matches!(exit, Exit::Built { .. }) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(err) => {
            eprintln!("acan-worker: {err:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run_cli() -> Result<Exit> {
    let cli = Cli::parse();
    match cli.role {
        Role::Worker => {}
    }
    load_env(&cli.env)?;
    let config =
        LlmConfig::load(&cli.llm).with_context(|| format!("loading {}", cli.llm.display()))?;
    let keys = ApiKeys::from_env(&config);
    let model = OpenAiCompatible::new(config, keys).context("building the model client")?;
    let worker = WorkerConfig {
        server: cli.server,
        token: cli.token,
        test_command: cli.test_command,
        max_steps: cli.max_steps,
        wait: Duration::from_secs(cli.wait_secs),
        heartbeat: Duration::from_secs(cli.heartbeat_secs),
        poll: Duration::from_millis(500),
    };
    Ok(run(&worker, &model).await.exit)
}

/// Loads the provider keys from `path`. A missing file is normal; a file that does not parse is
/// an error, so keys are not silently missing.
///
/// The parser's error quotes the line it failed on, and that line can hold a key, so the error
/// names the file and the failure without the parser's text.
fn load_env(path: &Path) -> Result<()> {
    match dotenvy::from_path(path) {
        Ok(()) => Ok(()),
        Err(err) if err.not_found() => Ok(()),
        Err(dotenvy::Error::LineParse(..)) => {
            bail!(
                "the provider key file {} does not parse; check its lines",
                path.display()
            )
        }
        Err(dotenvy::Error::Io(err)) => {
            bail!(
                "the provider key file {} could not be read: {}",
                path.display(),
                err.kind()
            )
        }
        Err(_) => bail!("the provider key file {} could not be read", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::load_env;

    #[test]
    fn a_key_file_that_does_not_parse_is_reported_without_its_text() {
        let dir = tempfile::tempdir().expect("temporary directory");
        let path = dir.path().join(".env");
        // A line that starts with a quote is not a key, so the parser fails on the line itself,
        // which is where a key would sit.
        fs::write(&path, "\"sk-live-secret-value\" oops\n").expect("the key file");
        let err = load_env(&path).expect_err("a file that does not parse is an error");
        let text = format!("{err:#}");
        assert!(
            !text.contains("sk-live-secret-value"),
            "the message carries the key: {text}"
        );
        assert!(text.contains(".env"), "the message names the file: {text}");
    }

    #[test]
    fn a_missing_key_file_is_normal() {
        let dir = tempfile::tempdir().expect("temporary directory");
        load_env(&dir.path().join("absent.env")).expect("a missing file loads nothing");
    }
}
