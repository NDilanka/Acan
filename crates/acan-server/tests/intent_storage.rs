//! Observation storage on a real filesystem: what a restart, a damaged file and a crash mid-write
//! leave behind. The rulings have no file of their own: the core ledger holds them, and its
//! persistence is covered by the gate tests.

#[path = "../src/observations.rs"]
mod observations;

use std::fs;
use std::path::{Path, PathBuf};

use acan_intent::{Observation, canonical_json};
use observations::ObservationError;

fn observation() -> Observation {
    serde_json::from_str(include_str!("../../acan-intent/fixtures/observation.json"))
        .expect("the observation fixture decodes")
}

fn incomplete_observation() -> Observation {
    serde_json::from_str(include_str!(
        "../../acan-intent/fixtures/observation-incomplete.json"
    ))
    .expect("the incomplete observation fixture decodes")
}

fn stored_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .expect("the store lists")
        .map(|entry| entry.expect("a store entry").path())
        .collect();
    files.sort();
    files
}

#[test]
fn an_observation_reads_back_by_its_hash_and_is_stored_once() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let store = dir.path().join("observations");
    let observation = observation();

    let hash = observations::put(&store, &observation).expect("the observation stores");
    assert_eq!(hash, observation.observation_hash().expect("hashing"));
    assert_eq!(
        observations::get(&store, &hash).expect("the observation reads back"),
        observation
    );
    assert_eq!(
        observations::put(&store, &observation).expect("storing it again succeeds"),
        hash
    );
    let files = stored_files(&store);
    assert_eq!(files.len(), 1, "one file and no temporary left: {files:?}");
    assert_eq!(
        fs::read(&files[0]).expect("the stored file reads"),
        canonical_json(&observation).expect("canonical JSON"),
        "the file holds the canonical JSON the hash covers"
    );

    let other = incomplete_observation();
    let other_hash = observations::put(&store, &other).expect("a second observation stores");
    assert_ne!(other_hash, hash);
    assert_eq!(
        observations::get(&store, &other_hash).expect("it reads back"),
        other
    );
    assert_eq!(
        observations::get(&store, &hash).expect("the first still reads"),
        observation
    );
}

#[test]
fn a_missing_observation_is_an_error_not_a_default() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let store = dir.path().join("observations");
    let observation = observation();
    let hash = observation.observation_hash().expect("hashing");

    assert!(
        matches!(
            observations::get(&store, &hash),
            Err(ObservationError::Missing { hash: missing }) if missing == hash
        ),
        "a store that was never written has nothing"
    );

    observations::put(&store, &observation).expect("the observation stores");
    for file in stored_files(&store) {
        fs::remove_file(file).expect("losing the stored file");
    }
    assert!(
        matches!(
            observations::get(&store, &hash),
            Err(ObservationError::Missing { hash: missing }) if missing == hash
        ),
        "a lost file is missing, not empty"
    );
}

#[test]
fn a_crash_before_rename_leaves_no_observation() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let store = dir.path().join("observations");
    fs::create_dir_all(&store).expect("the store exists");
    let observation = observation();
    let hash = observation.observation_hash().expect("hashing");
    let staged = canonical_json(&observation).expect("canonical JSON");
    fs::write(store.join("crashed.json.1-0.tmp"), &staged)
        .expect("leaving a temporary file behind");

    assert!(
        matches!(
            observations::get(&store, &hash),
            Err(ObservationError::Missing { hash: missing }) if missing == hash
        ),
        "a temporary file that was never renamed is not the observation"
    );

    observations::put(&store, &observation).expect("the observation stores");
    assert_eq!(
        observations::get(&store, &hash).expect("the stored observation reads back"),
        observation
    );
}

#[test]
fn an_altered_observation_is_refused_until_stored_again() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let store = dir.path().join("observations");
    let observation = observation();
    let hash = observations::put(&store, &observation).expect("the observation stores");
    let [file] = stored_files(&store)
        .try_into()
        .expect("exactly one stored file");
    let original = fs::read(&file).expect("the stored file reads");
    let mut spaced = original.clone();
    spaced.push(b' ');

    for (case, bytes) in [
        ("truncated", original[..original.len() / 2].to_vec()),
        ("emptied", Vec::new()),
        (
            "replaced by another observation",
            canonical_json(&incomplete_observation()).expect("canonical JSON"),
        ),
        ("re-encoded with a trailing space", spaced),
    ] {
        fs::write(&file, bytes).expect("altering the stored file");
        assert!(
            matches!(
                observations::get(&store, &hash),
                Err(ObservationError::Altered { hash: altered }) if altered == hash
            ),
            "{case}: get refuses"
        );
    }

    observations::put(&store, &observation).expect("storing it again repairs the file");
    assert_eq!(
        observations::get(&store, &hash).expect("the repaired file reads"),
        observation
    );
}
