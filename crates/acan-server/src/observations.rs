//! Raw observation storage, addressed by content hash.
//!
//! Each observation is one file, `<64 hex digits>.json`, holding its canonical JSON, whose
//! SHA-256 is its [`ObservationHash`]. [`get`] checks the hash before decoding and again after,
//! so a missing, truncated or edited file is an error, never an empty or default observation,
//! and a lost file cannot turn into a pass. A file appears under its final name only by
//! rename, so a crash mid-write leaves the observation missing, not half there.

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use acan_intent::{CanonicalError, ContentHash, Observation, ObservationHash, canonical_json};

use crate::ledger::replace_file;

/// Distinguishes the temporary files of concurrent puts within one process.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Why an observation could not be stored or read. Every variant refuses: none of them stands
/// for an empty or default observation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum ObservationError {
    /// Nothing is stored under the hash.
    #[error("observation {hash} is not stored")]
    Missing {
        /// The hash asked for.
        hash: ObservationHash,
    },
    /// The stored file's content does not have the hash it is stored under.
    #[error("stored observation {hash} does not have that hash")]
    Altered {
        /// The hash it is stored under.
        hash: ObservationHash,
    },
    /// The stored file has the right hash but is not an observation this build decodes.
    #[error("stored observation {hash} does not decode")]
    Malformed {
        /// The hash it is stored under.
        hash: ObservationHash,
        /// The decode error.
        #[source]
        source: serde_json::Error,
    },
    /// The observation's canonical JSON does not decode back to the same observation, so the
    /// store could not return it faithfully. Nothing was written.
    #[error("observation {hash} does not decode back to itself")]
    DoesNotRoundTrip {
        /// Its hash.
        hash: ObservationHash,
    },
    /// The store directory or a file in it could not be read or written.
    #[error("cannot read or write {}", path.display())]
    Io {
        /// The file or directory.
        path: PathBuf,
        /// The cause.
        #[source]
        source: io::Error,
    },
    /// The observation has no canonical JSON form.
    #[error(transparent)]
    Canonical(#[from] CanonicalError),
}

/// Stores `observation` under `dir`, creating `dir` if needed, and returns its hash. Storing
/// the same observation twice is not an error and writes nothing new. A stored file whose
/// content was altered is replaced with the observation's true content.
///
/// # Errors
///
/// Returns [`ObservationError`] when the observation cannot be hashed, does not round-trip, or
/// cannot be written.
pub(crate) fn put(
    dir: &Path,
    observation: &Observation,
) -> Result<ObservationHash, ObservationError> {
    let hash = observation.observation_hash()?;
    let bytes = canonical_json(observation)?;
    if !serde_json::from_slice::<Observation>(&bytes).is_ok_and(|decoded| decoded == *observation) {
        return Err(ObservationError::DoesNotRoundTrip { hash });
    }
    let name = file_name(hash);
    let path = dir.join(&name);
    match fs::read(&path) {
        Ok(stored) if stored == bytes => return Ok(hash),
        Ok(_) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(source) => return Err(io_error(&path, source)),
    }
    fs::create_dir_all(dir).map_err(|source| io_error(dir, source))?;
    let temp = dir.join(format!(
        "{name}.{}-{}.tmp",
        process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    replace_file(&path, &temp, &bytes).map_err(|source| io_error(&path, source))?;
    Ok(hash)
}

/// Reads the observation stored under `hash`, verifying that its content still has that hash.
///
/// # Errors
///
/// Returns [`ObservationError`] when it is missing, unreadable, altered or undecodable.
pub(crate) fn get(dir: &Path, hash: &ObservationHash) -> Result<Observation, ObservationError> {
    let hash = *hash;
    let path = dir.join(file_name(hash));
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Err(ObservationError::Missing { hash });
        }
        Err(source) => return Err(io_error(&path, source)),
    };
    if ContentHash::of_bytes(&bytes) != hash.content() {
        return Err(ObservationError::Altered { hash });
    }
    let observation: Observation = serde_json::from_slice(&bytes)
        .map_err(|source| ObservationError::Malformed { hash, source })?;
    if observation.observation_hash()? != hash {
        return Err(ObservationError::Altered { hash });
    }
    Ok(observation)
}

/// `<64 hex digits>.json`. The `sha256:` prefix is left out because `:` is not allowed in
/// Windows file names.
fn file_name(hash: ObservationHash) -> String {
    let mut name =
        hash.content()
            .as_bytes()
            .iter()
            .fold(String::with_capacity(69), |mut name, byte| {
                let _ = write!(name, "{byte:02x}");
                name
            });
    name.push_str(".json");
    name
}

fn io_error(path: &Path, source: io::Error) -> ObservationError {
    ObservationError::Io {
        path: path.to_owned(),
        source,
    }
}
