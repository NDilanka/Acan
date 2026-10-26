//! The host: one world's origin behind one lock, its stored log, its live broadcast, and the current scan.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use acan_core::{Command, Ctx, Origin, Rejection, Seq, Stamped, SystemCmd, Timestamp, World};
use acan_llm::{ModelClient, UsageLedger};
use anyhow::Context;
use tokio::sync::{Mutex, Notify, broadcast};

use crate::auth::Tokens;
use crate::index::Scan;
use crate::store::EventStore;

/// How many events a live subscriber may fall behind before the server disconnects it.
///
/// A subscriber that falls further behind must resume from its last seq. Dropping events
/// silently would leave its replica wrong.
pub const LIVE_BUFFER: usize = 256;

/// Why a command did not take effect.
#[derive(Debug, thiserror::Error)]
pub enum SubmitError {
    /// The origin refused the command. Nothing was stored.
    #[error(transparent)]
    Rejected(#[from] Rejection),
    /// Persisting the accepted events, or reading the log back, failed.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// Everything a [`Host`] owns at startup.
pub struct HostParts<C> {
    /// The origin, already opened or replayed from the stored log.
    pub origin: Origin,
    /// The stored log the origin writes to.
    pub store: EventStore,
    /// The bearer tokens.
    pub tokens: Tokens,
    /// The scan the origin reads for detection.
    pub scan: Scan,
    /// The usage ledger the model client records into.
    pub ledger: UsageLedger,
    /// The model client that runs derived jobs.
    pub client: C,
    /// The repository being served.
    pub repo: PathBuf,
}

/// One world's host. Cloning shares the same origin, log and broadcast.
pub struct Host<C> {
    inner: Arc<Inner<C>>,
}

struct Inner<C> {
    origin: Mutex<Origin>,
    store: EventStore,
    tokens: Tokens,
    scan: RwLock<Arc<Scan>>,
    events: broadcast::Sender<Stamped>,
    wake: Notify,
    ledger: UsageLedger,
    client: C,
    repo: PathBuf,
}

impl<C> Clone for Host<C> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<C: ModelClient> Host<C> {
    /// Builds a host from its parts. The caller decides when to run jobs and install scans.
    #[must_use]
    pub fn new(parts: HostParts<C>) -> Self {
        let (events, _) = broadcast::channel(LIVE_BUFFER);
        Self {
            inner: Arc::new(Inner {
                origin: Mutex::new(parts.origin),
                store: parts.store,
                tokens: parts.tokens,
                scan: RwLock::new(Arc::new(parts.scan)),
                events,
                wake: Notify::new(),
                ledger: parts.ledger,
                client: parts.client,
                repo: parts.repo,
            }),
        }
    }

    /// The bearer tokens.
    #[must_use]
    pub fn tokens(&self) -> &Tokens {
        &self.inner.tokens
    }

    /// The stored log.
    #[must_use]
    pub fn store(&self) -> &EventStore {
        &self.inner.store
    }

    /// Model usage recorded by the client.
    #[must_use]
    pub fn ledger(&self) -> &UsageLedger {
        &self.inner.ledger
    }

    /// The model client that runs derived jobs.
    #[must_use]
    pub fn client(&self) -> &C {
        &self.inner.client
    }

    /// The repository being served.
    #[must_use]
    pub fn repo(&self) -> &Path {
        &self.inner.repo
    }

    /// The current scan. Readers keep the `Arc` for as long as they need a consistent view.
    #[must_use]
    pub fn scan(&self) -> Arc<Scan> {
        Arc::clone(&self.inner.scan.read().expect("scan lock poisoned"))
    }

    /// A receiver for events accepted from now on.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<Stamped> {
        self.inner.events.subscribe()
    }

    /// Reads the origin's state under its lock. `read` must not wait on anything else.
    pub async fn read<R>(&self, read: impl FnOnce(&World) -> R) -> R {
        let origin = self.inner.origin.lock().await;
        read(origin.world())
    }

    /// Decides `command` against the current scan. On acceptance, persists the events, then
    /// broadcasts them and wakes the job loop. A refused command changes nothing.
    ///
    /// # Errors
    ///
    /// Returns [`SubmitError::Rejected`] when the origin refuses the command, and
    /// [`SubmitError::Internal`] when the events cannot be stored.
    pub async fn submit(&self, command: Command) -> Result<Vec<Stamped>, SubmitError> {
        let scan = self.scan();
        let ctx = Ctx {
            now: now(),
            index: &scan.index,
        };
        let mut origin = self.inner.origin.lock().await;
        let events = origin.execute(&ctx, command)?;
        if events.is_empty() {
            return Ok(events);
        }
        if let Err(err) = self.inner.store.append(&events).await {
            // The origin applied the batch in memory before the write failed. Rebuild it from
            // the stored log so memory and disk agree again.
            let log = self.inner.store.load_from(Seq::new(0)).await?;
            *origin = Origin::replay(log).context("rebuilding the origin after a failed write")?;
            return Err(SubmitError::Internal(err));
        }
        for event in &events {
            // A send fails only when nobody is subscribed, which is fine.
            let _ = self.inner.events.send(event.clone());
        }
        self.inner.wake.notify_one();
        Ok(events)
    }

    /// Scans the repository again and records the new code index.
    ///
    /// # Errors
    ///
    /// Returns an error when the scan fails, or when the `IndexUpdated` event cannot be stored.
    pub async fn rescan(&self) -> Result<Vec<Stamped>, SubmitError> {
        let repo = self.inner.repo.clone();
        let scan = tokio::task::spawn_blocking(move || crate::index::scan(&repo))
            .await
            .context("the scan task did not finish")??;
        *self.inner.scan.write().expect("scan lock poisoned") = Arc::new(scan);
        self.submit(Command::System(SystemCmd::IndexUpdated)).await
    }

    /// Asks the job loop to look for derived jobs again.
    pub fn wake_jobs(&self) {
        self.inner.wake.notify_one();
    }

    /// Resolves when the job loop should look for derived jobs.
    pub async fn jobs_woken(&self) {
        self.inner.wake.notified().await;
    }
}

/// The origin's clock: unix milliseconds at the moment of the call.
pub(crate) fn now() -> Timestamp {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before the unix epoch")
        .as_millis();
    Timestamp::from_millis(u64::try_from(millis).expect("clock is within u64 milliseconds"))
}
