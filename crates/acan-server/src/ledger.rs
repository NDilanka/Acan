//! The ruling log, `.acan/rulings.log`: one canonical JSON line per human decision, append
//! only, hash chained.
//!
//! The log is the one authority for rulings. Nothing derived from it is stored: [`load`]
//! rebuilds expectations, notes and retired probes in memory through
//! [`acan_intent::fold::fold`] on every call, with no model call. A truncated tail, a broken
//! chain, a non-canonical line or a reused sequence number is an error; it never yields a
//! shorter log, a default expectation or a green receipt.
//!
//! Each line is the RFC 8785 canonical JSON of its event followed by `\n`, so an event's chain
//! hash is the SHA-256 of its line. [`append`] never edits the log in place. Holding an
//! exclusive lock on the `.lock` sibling, it writes the whole new log to the `.tmp` sibling,
//! flushes it to disk and renames it over the log. A crash at any point leaves the old log or
//! the new one, never a mix, so readers take no lock.
//!
//! An absent log is the empty log. A log deleted or cut back by whole lines therefore reads as
//! an earlier revision; only a head recorded elsewhere, such as a receipt's
//! [`acan_intent::GateContext`], can expose that.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use acan_intent::fold::{self, FoldError};
use acan_intent::{
    CanonicalError, ContentHash, CoreDerived, FoldedLedger, LedgerEvent, LedgerHash, LedgerHead,
    canonical_json,
};

/// The log's file name inside the state directory.
pub(crate) const RULINGS_LOG: &str = "rulings.log";

const EMPTY: LedgerHead = LedgerHead {
    event_seq: 0,
    hash: None,
};

/// Why the ruling log could not be read or written. Every variant refuses: none of them stands
/// for an empty or shorter log.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum LedgerError {
    /// The log, its lock or its temporary file could not be read or written.
    #[error("cannot read or write {}", path.display())]
    Io {
        /// The file.
        path: PathBuf,
        /// The cause.
        #[source]
        source: io::Error,
    },
    /// The last line has no newline: a write was cut off or the file was truncated.
    #[error("line {line} of the ruling log is cut off before its newline")]
    TruncatedTail {
        /// The cut line, counting from 1.
        line: u64,
    },
    /// A line is not a ledger event.
    #[error("line {line} of the ruling log does not decode")]
    Malformed {
        /// The line, counting from 1.
        line: u64,
        /// The decode error.
        #[source]
        source: serde_json::Error,
    },
    /// A line decodes but is not its event's canonical JSON, so it was not written here.
    #[error("line {line} of the ruling log is not the canonical form of its event")]
    NotCanonical {
        /// The line, counting from 1.
        line: u64,
    },
    /// A line's sequence number is not its position: an event is missing or repeated.
    #[error("line {line} of the ruling log holds event_seq {found}")]
    OutOfSequence {
        /// The line, counting from 1, which is the sequence number it must hold.
        line: u64,
        /// The sequence number it holds.
        found: u64,
    },
    /// An event's `prev_hash` is not the chain hash of the event before it.
    #[error("event {event_seq} does not name the chain hash of the event before it")]
    BrokenChain {
        /// The event.
        event_seq: u64,
    },
    /// The log moved past the head the caller read. Nothing was written.
    #[error("the ruling log is at {actual:?}, not the expected {expected:?}")]
    StaleHead {
        /// The head the caller expected.
        expected: LedgerHead,
        /// The head the log has.
        actual: LedgerHead,
    },
    /// The event's sequence number or `prev_hash` does not follow the expected head. Nothing
    /// was written.
    #[error("event {event_seq} does not chain onto {expected:?}")]
    DoesNotChain {
        /// The head it was appended after.
        expected: LedgerHead,
        /// The event.
        event_seq: u64,
    },
    /// The event's canonical JSON does not decode back to the same event, so the log could not
    /// hold it faithfully. Nothing was written.
    #[error("event {event_seq} does not decode back to itself")]
    DoesNotRoundTrip {
        /// The event.
        event_seq: u64,
    },
    /// An event has no canonical JSON form.
    #[error(transparent)]
    Canonical(#[from] CanonicalError),
    /// The verified log does not fold.
    #[error("the ruling log does not fold")]
    Fold(#[from] FoldError),
    /// The fold reports a different head than the verified chain.
    #[error("the fold reports head {folded:?}, the chain {log:?}")]
    HeadMismatch {
        /// The head of the verified chain.
        log: LedgerHead,
        /// The head the fold reports.
        folded: LedgerHead,
    },
}

/// Reads and verifies every event in `log`, in order. An absent log has no events.
///
/// # Errors
///
/// Returns [`LedgerError`] when the log cannot be read, a line is cut off, does not decode or
/// is not canonical, a sequence number is out of place, or the hash chain is broken.
pub(crate) fn read(log: &Path) -> Result<Vec<LedgerEvent>, LedgerError> {
    Ok(Verified::read(log)?.events)
}

/// The log's current revision. An absent log is revision zero.
///
/// # Errors
///
/// Returns [`LedgerError`] when the log cannot be read or verified, as for [`read`].
pub(crate) fn head(log: &Path) -> Result<LedgerHead, LedgerError> {
    Ok(Verified::read(log)?.head)
}

/// Appends `event` after `expected` and returns the new head.
///
/// Re-appending the event that is already the head writes nothing and returns the current
/// head, so retrying an append whose outcome is unknown, after a crash or an error, cannot land
/// it twice. Any other event must name `expected`, which must be the log's current head, and
/// follow it by sequence number and `prev_hash`. Concurrent appends are serialized by the lock
/// file, so of two writers that read the same head only one lands.
///
/// # Errors
///
/// Returns [`LedgerError`] when the existing log does not verify, the head moved past
/// `expected`, the event does not chain onto `expected` or does not round-trip, or the write
/// fails. The log is unchanged unless the error came after the rename, which a retry of the
/// same event then reports as success.
pub(crate) fn append(
    log: &Path,
    expected: &LedgerHead,
    event: &LedgerEvent,
) -> Result<LedgerHead, LedgerError> {
    let _lock = lock(log)?;
    let current = Verified::read(log)?;
    if current.events.last() == Some(event) {
        return Ok(current.head);
    }
    if *expected != current.head {
        return Err(LedgerError::StaleHead {
            expected: *expected,
            actual: current.head,
        });
    }
    let event_seq = event.event_seq();
    if Some(event_seq) != current.head.event_seq.checked_add(1)
        || event.prev_hash() != current.head.hash
    {
        return Err(LedgerError::DoesNotChain {
            expected: *expected,
            event_seq,
        });
    }
    let line = canonical_json(event)?;
    if !serde_json::from_slice::<LedgerEvent>(&line).is_ok_and(|decoded| decoded == *event) {
        return Err(LedgerError::DoesNotRoundTrip { event_seq });
    }
    let mut bytes = current.bytes;
    bytes.extend_from_slice(&line);
    bytes.push(b'\n');
    replace_file(log, &temp_path(log), &bytes).map_err(|source| io_error(log, source))?;
    Ok(LedgerHead {
        event_seq,
        hash: Some(chain_hash(&line)),
    })
}

/// Reads and verifies `log`, then folds it into expectations, notes and retired probes.
///
/// # Errors
///
/// Returns [`LedgerError`] when the log cannot be read or verified, as for [`read`], when the
/// fold refuses it, or when the fold's head disagrees with the verified chain.
pub(crate) fn load(log: &Path) -> Result<CoreDerived<FoldedLedger>, LedgerError> {
    let Verified { events, head, .. } = Verified::read(log)?;
    let folded = fold::fold(&events)?;
    if folded.value().head != head {
        return Err(LedgerError::HeadMismatch {
            log: head,
            folded: folded.value().head,
        });
    }
    Ok(folded)
}

/// The temporary file [`append`] writes before renaming it over `log`. A crash can leave it
/// behind; it is never read and the next append replaces it.
pub(crate) fn temp_path(log: &Path) -> PathBuf {
    sibling(log, ".tmp")
}

/// Replaces `target` with `bytes` so that a crash leaves the old file or the new one: writes
/// `temp`, flushes it to disk, renames it over `target`, and on unix flushes the directory.
/// Removes `temp` if writing or renaming fails.
///
/// # Errors
///
/// Returns the I/O error of the failing step.
pub(crate) fn replace_file(target: &Path, temp: &Path, bytes: &[u8]) -> io::Result<()> {
    let written = write_synced(temp, bytes).and_then(|()| fs::rename(temp, target));
    if written.is_err() {
        let _ = fs::remove_file(temp);
    }
    written?;
    // std cannot open a directory for flushing on Windows; NTFS journals the rename itself.
    #[cfg(unix)]
    sync_parent(target)?;
    Ok(())
}

/// A log that read back whole and verified: its bytes, its events and its head.
struct Verified {
    bytes: Vec<u8>,
    events: Vec<LedgerEvent>,
    head: LedgerHead,
}

impl Verified {
    fn read(log: &Path) -> Result<Self, LedgerError> {
        let bytes = match fs::read(log) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(source) => return Err(io_error(log, source)),
        };
        let mut events = Vec::new();
        let mut head = EMPTY;
        for (line_no, chunk) in (1u64..).zip(bytes.split_inclusive(|&byte| byte == b'\n')) {
            let Some(line) = chunk.strip_suffix(b"\n") else {
                return Err(LedgerError::TruncatedTail { line: line_no });
            };
            events.push(verify_line(line_no, line, &head)?);
            head = LedgerHead {
                event_seq: line_no,
                hash: Some(chain_hash(line)),
            };
        }
        Ok(Self {
            bytes,
            events,
            head,
        })
    }
}

/// Decodes line `line_no` and checks that it is canonical, holds sequence number `line_no`
/// and chains onto `prev`.
fn verify_line(line_no: u64, line: &[u8], prev: &LedgerHead) -> Result<LedgerEvent, LedgerError> {
    let event: LedgerEvent =
        serde_json::from_slice(line).map_err(|source| LedgerError::Malformed {
            line: line_no,
            source,
        })?;
    if canonical_json(&event)? != line {
        return Err(LedgerError::NotCanonical { line: line_no });
    }
    if event.event_seq() != line_no {
        return Err(LedgerError::OutOfSequence {
            line: line_no,
            found: event.event_seq(),
        });
    }
    if event.prev_hash() != prev.hash {
        return Err(LedgerError::BrokenChain { event_seq: line_no });
    }
    Ok(event)
}

/// The chain hash of an event whose canonical JSON is `line`, equal to
/// [`LedgerEvent::chain_hash`].
fn chain_hash(line: &[u8]) -> LedgerHash {
    LedgerHash::from(ContentHash::of_bytes(line))
}

/// Takes the exclusive append lock. The OS releases it when the returned file closes, even if
/// the process dies, so a crash never leaves the log locked.
fn lock(log: &Path) -> Result<File, LedgerError> {
    let path = sibling(log, ".lock");
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .map_err(|source| io_error(&path, source))?;
    file.lock().map_err(|source| io_error(&path, source))?;
    Ok(file)
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

fn io_error(path: &Path, source: io::Error) -> LedgerError {
    LedgerError::Io {
        path: path.to_owned(),
        source,
    }
}

fn write_synced(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(unix)]
fn sync_parent(path: &Path) -> io::Result<()> {
    let dir = match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir,
        _ => Path::new("."),
    };
    File::open(dir)?.sync_all()
}
