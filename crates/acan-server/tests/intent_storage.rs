//! The ruling log and observation storage on a real filesystem: what a restart, a damaged
//! file, a retried or racing append and a crash mid-write leave behind.

#[path = "../src/ledger.rs"]
mod ledger;
#[path = "../src/observations.rs"]
mod observations;

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use acan_intent::{ContentHash, LedgerEvent, LedgerHead, Observation, canonical_json, fold};
use ledger::{LedgerError, RULINGS_LOG};
use observations::ObservationError;
use serde_json::{Value, json};
use tempfile::TempDir;

const EMPTY: LedgerHead = LedgerHead {
    event_seq: 0,
    hash: None,
};

/// Events 1 to 5 of the contract's fixture ledger, one real hash chain.
fn chain() -> Vec<LedgerEvent> {
    [
        include_str!("../../acan-intent/fixtures/ledger-decision.json"),
        include_str!("../../acan-intent/fixtures/ledger-note.json"),
        include_str!("../../acan-intent/fixtures/ledger-decision-dashboard.json"),
        include_str!("../../acan-intent/fixtures/ledger-decision-supersede.json"),
        include_str!("../../acan-intent/fixtures/ledger-probe-retired.json"),
    ]
    .into_iter()
    .map(|text| serde_json::from_str(text).expect("a ledger fixture decodes"))
    .collect()
}

fn edited(event: &LedgerEvent, field: &str, value: Value) -> LedgerEvent {
    let mut raw = serde_json::to_value(event).expect("an event encodes");
    raw[field] = value;
    serde_json::from_value(raw).expect("the edited event decodes")
}

fn head_of(event: &LedgerEvent) -> LedgerHead {
    LedgerHead {
        event_seq: event.event_seq(),
        hash: Some(event.chain_hash().expect("an event hashes")),
    }
}

fn line(event: &LedgerEvent) -> Vec<u8> {
    let mut line = canonical_json(event).expect("an event has canonical JSON");
    line.push(b'\n');
    line
}

fn lines(events: &[LedgerEvent]) -> Vec<u8> {
    events.iter().flat_map(line).collect()
}

fn log_with(events: &[LedgerEvent]) -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("temporary directory");
    let log = dir.path().join(RULINGS_LOG);
    let mut head = EMPTY;
    for event in events {
        head = ledger::append(&log, &head, event).expect("the fixture chain appends");
    }
    (dir, log)
}

fn read(log: &Path) -> Vec<LedgerEvent> {
    ledger::read(log).expect("the log verifies")
}

fn truncated_at_line_2<T>(result: &Result<T, LedgerError>) -> bool {
    matches!(result, Err(LedgerError::TruncatedTail { line: 2 }))
}

#[test]
fn the_fixture_chain_appends_and_reads_back_unchanged() {
    let events = chain();
    let dir = tempfile::tempdir().expect("temporary directory");
    let log = dir.path().join(RULINGS_LOG);
    assert_eq!(ledger::head(&log).expect("an absent log reads"), EMPTY);
    assert!(read(&log).is_empty(), "an absent log has no events");

    let mut head = EMPTY;
    for event in &events {
        let next = ledger::append(&log, &head, event).expect("the next event appends");
        assert_eq!(next, head_of(event), "append returns the event's own head");
        head = next;
    }

    assert_eq!(read(&log), events);
    assert_eq!(ledger::head(&log).expect("the log verifies"), head);
    assert_eq!(
        fs::read(&log).expect("the log reads"),
        lines(&events),
        "one canonical line per event, nothing else"
    );
    assert!(
        !ledger::temp_path(&log).exists(),
        "a finished append leaves no temporary file"
    );
}

#[test]
fn a_restart_rebuilds_the_same_derived_state() {
    let events = chain();
    let (_dir, log) = log_with(&events);
    let restarted = tempfile::tempdir().expect("temporary directory");
    let copy = restarted.path().join(RULINGS_LOG);
    fs::copy(&log, &copy).expect("copying only the log");

    assert_eq!(read(&copy), events, "the log alone restores every event");
    assert_eq!(
        ledger::head(&copy).expect("the copy verifies"),
        head_of(&events[4])
    );
    match (ledger::load(&copy), fold::fold(&events)) {
        (Ok(loaded), Ok(folded)) => {
            assert_eq!(
                loaded, folded,
                "load is exactly the fold of the logged events"
            );
        }
        (Err(LedgerError::Fold(loaded)), Err(folded)) => {
            assert_eq!(loaded, folded, "a fold refusal surfaces as a refusal");
        }
        (loaded, folded) => {
            panic!("load must be exactly the fold of the log, got {loaded:?} and {folded:?}")
        }
    }
}

#[test]
fn a_truncated_tail_is_refused_and_never_extended() {
    let events = chain();
    let (_dir, log) = log_with(&events[..2]);
    let whole = fs::read(&log).expect("the log reads");
    let last = line(&events[1]).len();

    for cut in [1, 2, last / 2, last - 1] {
        let damaged = &whole[..whole.len() - cut];
        fs::write(&log, damaged).expect("damaging the log");

        assert!(
            truncated_at_line_2(&ledger::read(&log)),
            "cut {cut}: read refuses"
        );
        assert!(
            truncated_at_line_2(&ledger::head(&log)),
            "cut {cut}: head refuses"
        );
        assert!(
            truncated_at_line_2(&ledger::load(&log)),
            "cut {cut}: load refuses before folding"
        );
        assert!(
            truncated_at_line_2(&ledger::append(&log, &head_of(&events[1]), &events[2])),
            "cut {cut}: no event lands after a damaged tail"
        );
        assert!(
            truncated_at_line_2(&ledger::append(&log, &head_of(&events[0]), &events[1])),
            "cut {cut}: nor does a retry of the cut event"
        );
        assert_eq!(
            fs::read(&log).expect("the log reads"),
            damaged,
            "cut {cut}: a refused append leaves the damage for a human"
        );
    }
}

#[test]
fn a_duplicate_append_writes_nothing() {
    let events = chain();
    let (_dir, log) = log_with(&events[..1]);
    let once = fs::read(&log).expect("the log reads");

    let retried = ledger::append(&log, &EMPTY, &events[0])
        .expect("a retry of the append that wrote the head is not an error");
    assert_eq!(retried, head_of(&events[0]));
    assert!(
        matches!(
            ledger::append(&log, &head_of(&events[0]), &events[0]),
            Err(LedgerError::Duplicate { event_seq: 1 })
        ),
        "re-appending the head after reading it back is refused"
    );
    assert_eq!(
        fs::read(&log).expect("the log reads"),
        once,
        "neither call writes anything"
    );

    ledger::append(&log, &head_of(&events[0]), &events[1]).expect("the next event appends");
    assert!(
        matches!(
            ledger::append(&log, &EMPTY, &events[0]),
            Err(LedgerError::StaleHead { .. })
        ),
        "an event that is no longer the head is not a duplicate"
    );
    let conflicting = edited(&events[1], "text", json!("a different note"));
    assert!(
        matches!(
            ledger::append(&log, &head_of(&events[0]), &conflicting),
            Err(LedgerError::StaleHead { .. })
        ),
        "a different event reusing a sequence number is refused"
    );
    assert_eq!(read(&log), events[..2]);
}

#[test]
fn a_stale_expected_head_is_refused() {
    let events = chain();
    let (_dir, log) = log_with(&events[..1]);
    let seen = head_of(&events[0]);
    ledger::append(&log, &seen, &events[1]).expect("the first writer lands");

    let rival = edited(&events[1], "text", json!("a rival note"));
    match ledger::append(&log, &seen, &rival) {
        Err(LedgerError::StaleHead { expected, actual }) => {
            assert_eq!(expected, seen);
            assert_eq!(actual, head_of(&events[1]));
        }
        other => panic!("the second writer must be refused, got {other:?}"),
    }
    assert_eq!(
        read(&log),
        events[..2],
        "only the first writer's event is logged"
    );
}

#[test]
fn racing_writers_land_exactly_one_event() {
    let events = chain();
    let (_dir, log) = log_with(&events[..1]);
    let seen = head_of(&events[0]);
    let rivals: Vec<LedgerEvent> = (0..8)
        .map(|n| edited(&events[1], "text", json!(format!("rival {n}"))))
        .collect();

    let (log_ref, seen_ref) = (&log, &seen);
    let results: Vec<Result<LedgerHead, LedgerError>> = thread::scope(|scope| {
        let writers: Vec<_> = rivals
            .iter()
            .map(|rival| scope.spawn(move || ledger::append(log_ref, seen_ref, rival)))
            .collect();
        writers
            .into_iter()
            .map(|writer| writer.join().expect("a writer thread finishes"))
            .collect()
    });

    let landed: Vec<usize> = results
        .iter()
        .enumerate()
        .filter_map(|(index, result)| result.is_ok().then_some(index))
        .collect();
    assert_eq!(landed.len(), 1, "exactly one writer lands: {results:?}");
    assert!(
        results
            .iter()
            .all(|result| result.is_ok() || matches!(result, Err(LedgerError::StaleHead { .. }))),
        "every other writer is told the head moved: {results:?}"
    );
    assert_eq!(read(&log), [events[0].clone(), rivals[landed[0]].clone()]);
}

#[test]
fn an_event_that_does_not_chain_is_refused() {
    let events = chain();
    let (_dir, log) = log_with(&events[..1]);
    let head = head_of(&events[0]);
    let elsewhere = json!(ContentHash::of_bytes(b"elsewhere").to_string());

    let cases = [
        ("a skipped sequence number", events[2].clone()),
        (
            "a wrong previous hash",
            edited(&events[1], "prev_hash", elsewhere),
        ),
        (
            "no previous hash",
            edited(&events[1], "prev_hash", Value::Null),
        ),
    ];
    for (case, event) in cases {
        assert!(
            matches!(
                ledger::append(&log, &head, &event),
                Err(LedgerError::DoesNotChain { .. })
            ),
            "{case} is refused"
        );
    }
    assert_eq!(read(&log), events[..1], "nothing was written");
}

#[test]
fn a_crash_between_write_and_rename_leaves_the_old_log() {
    let events = chain();
    let (_dir, log) = log_with(&events[..2]);
    let staged = lines(&events[..3]);
    let temp = ledger::temp_path(&log);

    for (case, left_behind) in [
        ("written but not renamed", &staged[..]),
        ("cut off mid-write", &staged[..staged.len() / 2]),
    ] {
        fs::write(&temp, left_behind).expect("leaving a temporary file behind");
        assert_eq!(read(&log), events[..2], "{case}: the old log stands");
        assert_eq!(
            ledger::head(&log).expect("the log verifies"),
            head_of(&events[1]),
            "{case}: the head did not move"
        );
    }

    let retried =
        ledger::append(&log, &head_of(&events[1]), &events[2]).expect("the retried append lands");
    assert_eq!(retried, head_of(&events[2]));
    assert_eq!(read(&log), events[..3], "the retried event is logged once");
    assert!(!temp.exists(), "the rename consumed the temporary file");
}

/// A case name, the log's bytes, and the refusal read must give.
type DamagedLog = (&'static str, Vec<u8>, fn(&LedgerError) -> bool);

#[test]
fn every_damaged_line_is_refused() {
    let events = chain();
    let canonical = lines(&events[..3]);
    let mut non_canonical = serde_json::to_vec(&events[0]).expect("an event encodes");
    non_canonical.push(b'\n');
    non_canonical.extend(lines(&events[1..3]));
    let mut trailing_space = canonical_json(&events[0]).expect("canonical JSON");
    trailing_space.extend_from_slice(b" \n");
    trailing_space.extend(lines(&events[1..3]));
    let crlf: Vec<u8> = canonical
        .split_inclusive(|&byte| byte == b'\n')
        .flat_map(|chunk| [&chunk[..chunk.len() - 1], &b"\r\n"[..]].concat())
        .collect();
    let edited_first = [
        line(&edited(&events[0], "label", json!("an edited label"))),
        lines(&events[1..3]),
    ]
    .concat();
    let gap = [line(&events[0]), line(&events[2])].concat();
    let blank = [line(&events[0]), b"\n".to_vec(), lines(&events[1..3])].concat();
    let claimed_predecessor = [
        line(&edited(
            &events[0],
            "prev_hash",
            json!(ContentHash::of_bytes(b"before the log").to_string()),
        )),
        lines(&events[1..3]),
    ]
    .concat();

    let cases: [DamagedLog; 7] = [
        ("keys out of canonical order", non_canonical, |err| {
            matches!(err, LedgerError::NotCanonical { line: 1 })
        }),
        ("a trailing space", trailing_space, |err| {
            matches!(err, LedgerError::NotCanonical { line: 1 })
        }),
        ("CRLF line endings", crlf, |err| {
            matches!(err, LedgerError::NotCanonical { line: 1 })
        }),
        ("an edited earlier event", edited_first, |err| {
            matches!(err, LedgerError::BrokenChain { event_seq: 2 })
        }),
        ("a missing middle event", gap, |err| {
            matches!(err, LedgerError::OutOfSequence { line: 2, found: 3 })
        }),
        ("a blank line", blank, |err| {
            matches!(err, LedgerError::Malformed { line: 2, .. })
        }),
        ("event 1 naming a predecessor", claimed_predecessor, |err| {
            matches!(err, LedgerError::BrokenChain { event_seq: 1 })
        }),
    ];

    let (_dir, log) = log_with(&[]);
    for (case, bytes, expected) in cases {
        fs::write(&log, &bytes).expect("writing a damaged log");
        match ledger::read(&log) {
            Err(err) if expected(&err) => {}
            other => panic!("{case}: read must refuse, got {other:?}"),
        }
        assert!(ledger::load(&log).is_err(), "{case}: load refuses");
        assert!(
            ledger::append(&log, &head_of(&events[2]), &events[3]).is_err(),
            "{case}: nothing lands after the damage"
        );
        assert_eq!(
            fs::read(&log).expect("the log reads"),
            bytes,
            "{case}: untouched"
        );
    }
}

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
