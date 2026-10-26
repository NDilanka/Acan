//! The acan server: one world's origin, its event log, and the HTTP and WebSocket surface around it.
//!
//! [`start`] opens the repository's world in `.acan/world.db`, scans the code, and serves the API.
//! The origin lives behind one lock inside the host. Each accepted batch is persisted before it is
//! broadcast, and derived model jobs run in the background and report back as system commands.

mod auth;
mod host;
mod http;
mod index;
mod jobs;
mod model;
mod models;
mod store;
mod stream;

use std::fs;
use std::io;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use acan_core::{Command, MemberId, Origin, Seq, SystemCmd, WorldId, WorldRef};
use acan_llm::{ModelClient, UsageLedger};
use anyhow::{Context, bail};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub use auth::{Principal, Token, Tokens};
pub use host::{Host, HostParts, LIVE_BUFFER, SubmitError};
pub use index::{IndexError, Scan, code_index, scan};
pub use models::Models;
pub use store::EventStore;

const STATE_DIR: &str = ".acan";
const DB_FILE: &str = "world.db";
const OWNER: &str = "owner";

/// What to serve, and where.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// The repository whose world this server runs.
    pub repo: PathBuf,
    /// The address to listen on. Port 0 picks a free port.
    pub addr: SocketAddr,
    /// Allows a non-loopback address. Without it, binding elsewhere is refused.
    pub team: bool,
    /// A directory served for every path the API does not claim, such as the renderer page.
    pub web: Option<PathBuf>,
}

/// A server that is listening. Dropping it does not stop the server.
#[derive(Debug)]
pub struct Running {
    /// The address the server bound, with the real port.
    pub local_addr: SocketAddr,
    /// The owner token. Present only on the first start, when it was minted.
    pub owner_token: Option<Token>,
    server: JoinHandle<io::Result<()>>,
    jobs: JoinHandle<()>,
}

impl Running {
    /// Waits until the server stops.
    ///
    /// # Errors
    ///
    /// Returns an error when the listener fails or the server task panics.
    pub async fn wait(self) -> anyhow::Result<()> {
        self.server
            .await
            .context("the server task did not finish")??;
        Ok(())
    }

    /// Asks the listener and the job loop to stop. Both stop at their next await point, and
    /// the jobs they started stop with them.
    pub fn stop(&self) {
        self.server.abort();
        self.jobs.abort();
    }
}

/// Opens the repository's world, replays it, scans the code, and starts listening.
///
/// # Errors
///
/// Returns an error when the bind address is non-loopback without `team`, the state directory,
/// tokens or database cannot be opened, the stored log does not replay, the scan fails, or the
/// address cannot be bound.
pub async fn start<C: ModelClient + 'static>(
    config: ServerConfig,
    client: C,
    ledger: UsageLedger,
) -> anyhow::Result<Running> {
    if !config.addr.ip().is_loopback() && !config.team {
        bail!(
            "refusing to bind {} without --team, since that exposes this world to the network",
            config.addr.ip()
        );
    }

    let state_dir = config.repo.join(STATE_DIR);
    prepare_state_dir(&state_dir)?;
    let founder = MemberId::from_str(OWNER)?;
    let (tokens, owner_token) = Tokens::load_or_create(&state_dir, &founder)?;
    let store = EventStore::open(&state_dir.join(DB_FILE)).await?;

    let log = store.load_from(Seq::new(0)).await?;
    let origin = if log.is_empty() {
        let (origin, opened) = Origin::open(world_ref(&config.repo)?, founder, host::now());
        store.append(&[opened]).await?;
        origin
    } else {
        Origin::replay(log)?
    };

    let repo = config.repo.clone();
    let scan = tokio::task::spawn_blocking(move || scan(&repo))
        .await
        .context("the scan task did not finish")??;

    let host = Host::new(HostParts {
        origin,
        store,
        tokens,
        scan,
        ledger,
        client,
        repo: config.repo.clone(),
    });
    host.submit(Command::System(SystemCmd::IndexUpdated))
        .await?;

    // Bind before the job loop starts, so a failed bind leaves no writer running.
    let listener = TcpListener::bind(config.addr)
        .await
        .with_context(|| format!("binding {}", config.addr))?;
    let local_addr = listener.local_addr()?;
    let jobs = jobs::spawn(host.clone());
    host.wake_jobs();
    let app = http::router(host, config.web);
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    Ok(Running {
        local_addr,
        owner_token,
        server,
        jobs,
    })
}

/// Creates the state directory. Its own `.gitignore` keeps the tokens and database out of git.
fn prepare_state_dir(dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    let ignore = dir.join(".gitignore");
    if !ignore.exists() {
        fs::write(&ignore, "*\n").with_context(|| format!("writing {}", ignore.display()))?;
    }
    Ok(())
}

fn world_ref(repo: &Path) -> anyhow::Result<WorldRef> {
    let name = repo
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("world");
    let id = WorldId::from_str(name).or_else(|_| WorldId::from_str("world"))?;
    Ok(WorldRef {
        id,
        repo: repo.display().to_string(),
    })
}
