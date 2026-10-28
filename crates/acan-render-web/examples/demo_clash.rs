//! Writes the demo clash into a repository's world log, so the product screen has one to show.
//!
//! The server has no agent route on this branch, so the script runs while the server is stopped.
//! Start `acan serve` once to create `.acan/world.db`, stop it, run this example, and serve again.
//! Every event comes from the origin's own `execute`, so the log replays like any other.
//!
//! Run with `cargo run -p acan-render-web --example demo_clash -- <repo>`.

use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use acan_core::{MemberId, Origin, Seq, Timestamp};
use acan_render_web::scenario::{self, Target};
use acan_server::{EventStore, scan};
use anyhow::{Context, Result, bail};

/// The order the script posts. The product screen shows it on the order pin.
const SENTENCE: &str = "change the login session so both callers can read it";

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    let repo = repo_arg()?;
    let scanned = scan(&repo)?;
    let store = EventStore::open(&repo.join(".acan").join("world.db")).await?;
    let log = store.load_from(Seq::new(0)).await?;
    if log.is_empty() {
        bail!(
            "{} has no world yet; run acan serve once first",
            repo.display()
        );
    }
    let mut origin = Origin::replay(log)?;
    let founder = MemberId::from_str("owner")?;
    let target = Target::first_symbol(&scanned.index, SENTENCE)
        .context("the repository has no symbol to clash on")?;
    let now = Timestamp::from_millis(unix_millis()?);
    let events = scenario::clash_script(&mut origin, &scanned.index, &founder, &target, now)
        .map_err(|rejection| anyhow::anyhow!("the origin refused the script: {rejection:?}"))?;
    store.append(&events).await?;
    let clash = scenario::clash_id(&origin).context("the script left no clash")?;
    println!(
        "appended {} events: order on {}, clash {clash}",
        events.len(),
        target.machine
    );
    Ok(())
}

fn repo_arg() -> Result<PathBuf> {
    let mut args = std::env::args().skip(1);
    let repo = args.next().context("usage: demo_clash <repo>")?;
    if args.next().is_some() {
        bail!("usage: demo_clash <repo>");
    }
    Ok(PathBuf::from(repo))
}

fn unix_millis() -> Result<u64> {
    let elapsed = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(u64::try_from(elapsed.as_millis())?)
}
