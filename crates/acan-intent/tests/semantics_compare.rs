//! The comparison and predicate contracts the review named, end to end over admitted runs:
//! which three runs a comparison binds, how a capture that did not bind reads, what identity
//! placeholders can and cannot spell, what a declared relation means on the base runs, and when
//! the rate window has nothing to judge.
//!
//! Every run is admitted through `admit::observed_run`, so the tests judge what the gate judges.
//! The runs are the login-rate-limit probe of the corpus, as in `semantics.rs`.

use std::collections::BTreeMap;

use acan_intent::compare::{self, CompareError, Contrast};
use acan_intent::predicate::{self, PredicateError};
use acan_intent::{
    CaptureOutcome, Channels, CheckResult, Comparison, Completeness, ContentHash, CoreDerived,
    CorpusProbe, Fixture, HttpMethod, MissingCapture, Observation, ObservationHash, ObservedRun,
    Outcome, Predicate, ProbeCorpus, RequestRecord, Run, RunOutcome, RunRole, RunnerHash, Scope,
    UnknownReason, admit,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

const PROBE_CORPUS: &str = include_str!("../fixtures/probe-corpus.json");
const RUN: &str = include_str!("../fixtures/run.json");
const OBSERVATION: &str = include_str!("../fixtures/observation.json");
const SCOPE: &str = include_str!("../fixtures/scope.json");
const COMMIT: &str = "a27a5f30da19543ee34362516e29bf7561214156";
const SIX_FAILURES: [u16; 6] = [401; 6];
const FIVE_THEN_LIMITED: [u16; 6] = [401, 401, 401, 401, 401, 429];

fn parse<T: DeserializeOwned>(text: &str) -> T {
    serde_json::from_str(text).expect("a fixture decodes")
}

/// The login probe and its fixture, as the corpus holds them.
struct Login {
    probe: CorpusProbe,
    fixture: Fixture,
}

impl Login {
    fn load() -> Self {
        let corpus: ProbeCorpus = parse(PROBE_CORPUS);
        let probe = corpus
            .probes
            .iter()
            .find(|probe| probe.label == "login-rate-limit")
            .expect("the corpus has the login probe")
            .clone();
        let fixture = corpus
            .fixtures
            .iter()
            .find(|fixture| fixture.id == probe.spec.fixture)
            .expect("the probe's fixture is in the corpus")
            .clone();
        Self { probe, fixture }
    }
}

/// One HTTP step as the runner records it.
#[derive(Clone)]
struct Hit {
    step: &'static str,
    route: &'static str,
    client: &'static str,
    sent_at_ms: u64,
    status: u16,
    body: Value,
    cookies: Vec<(&'static str, String)>,
}

impl Hit {
    /// A login from client-a to the login route, answered `status`.
    fn at(step: &'static str, status: u16, sent_at_ms: u64) -> Self {
        Self {
            step,
            route: "/api/login",
            client: "client-a",
            sent_at_ms,
            status,
            body: body_for(status),
            cookies: Vec::new(),
        }
    }

    fn observation(&self, run_id: &str, probe: acan_intent::ProbeHash) -> Observation {
        let mut observation: Observation = parse(OBSERVATION);
        observation.run_id = run_id.parse().expect("a run id");
        observation.commit = COMMIT.parse().expect("a commit");
        observation.probe = probe;
        observation.step_id = self.step.parse().expect("a step id");
        observation.raw_ref = None;
        observation.request = Some(RequestRecord {
            method: HttpMethod::Post,
            route_template: self.route.parse().expect("a route template"),
            actor: None,
            client: self.client.parse().expect("a client"),
            sent_at_ms: self.sent_at_ms,
        });
        observation.completeness = Completeness::Complete {};
        observation.channels = Channels {
            status: Some(self.status),
            headers: if self.status == 429 {
                BTreeMap::from([("retry-after".to_owned(), "60".to_owned())])
            } else {
                BTreeMap::new()
            },
            cookies: self
                .cookies
                .iter()
                .map(|(name, value)| ((*name).to_owned(), value.clone()))
                .collect(),
            body: Some(self.body.clone()),
            side_effects: BTreeMap::new(),
        };
        observation
    }
}

fn body_for(status: u16) -> Value {
    match status {
        401 => json!({ "error": "invalid credentials" }),
        429 => json!({ "status": "limited", "error": "too many attempts" }),
        _ => json!({ "ok": true }),
    }
}

/// Six logins at one-second steps, then the correct login at 61 seconds.
fn sequence(statuses: [u16; 6]) -> Vec<Hit> {
    let steps = ["s1", "s2", "s3", "s4", "s5", "s6"];
    let times = [1000, 2000, 3000, 4000, 5000, 6000];
    let mut hits: Vec<Hit> = steps
        .iter()
        .zip(statuses)
        .zip(times)
        .map(|((step, status), at)| Hit::at(step, status, at))
        .collect();
    hits.push(Hit::at("s8", 200, 61_000));
    hits
}

fn hit_mut<'h>(hits: &'h mut [Hit], step: &str) -> &'h mut Hit {
    hits.iter_mut()
        .find(|hit| hit.step == step)
        .expect("the step is in the sequence")
}

fn with_cookie(mut hits: Vec<Hit>, step: &str, value: &str) -> Vec<Hit> {
    hit_mut(&mut hits, step)
        .cookies
        .push(("session", value.to_owned()));
    hits
}

fn with_codes(codes: [&str; 2]) -> Vec<Hit> {
    let mut hits = sequence(SIX_FAILURES);
    hit_mut(&mut hits, "s8").body = json!({
        "links": [{ "code": codes[0] }, { "code": codes[1] }]
    });
    hits
}

/// Admits one run of the login probe. The recipe's signup and login come first, and the login
/// sets the session cookie `token`, which the capture `alice_session` binds. `edit` changes the
/// run record before admission; every observation takes the record's commit.
fn admit_with(
    login: &Login,
    id: &str,
    role: RunRole,
    token: &str,
    hits: Vec<Hit>,
    edit: impl FnOnce(&mut Run),
) -> ObservedRun {
    let probe = login.probe.spec.probe_hash().expect("the probe hashes");
    let signup = Hit {
        route: "/api/signup",
        ..Hit::at("f1", 201, 0)
    };
    let login_hit = Hit {
        cookies: vec![("session", token.to_owned())],
        ..Hit::at("f2", 200, 0)
    };
    let mut all = vec![signup, login_hit];
    all.extend(hits);
    let mut record: Run = parse(RUN);
    record.id = id.parse().expect("a run id");
    record.probe = probe;
    record.commit = COMMIT.parse().expect("a commit");
    record.role = role;
    record.env.fixture_hash = login.fixture.fixture_hash().expect("the fixture hashes");
    record.outcome = RunOutcome::Completed {};
    record.captures = vec![CaptureOutcome::Captured {
        binding: "alice_session".parse().expect("a binding"),
        step_id: "f3".parse().expect("a step id"),
        value_hash: ContentHash::of(&Value::String(token.to_owned()))
            .expect("the value hashes")
            .into(),
    }];
    edit(&mut record);
    let stored: Vec<(ObservationHash, Observation)> = all
        .iter()
        .map(|hit| {
            let mut observation = hit.observation(id, probe);
            observation.commit = record.commit.clone();
            (
                observation
                    .observation_hash()
                    .expect("the observation hashes"),
                observation,
            )
        })
        .collect();
    admit::observed_run(&login.probe, &login.fixture, record, stored).expect("the run is admitted")
}

/// The capture record of a run whose capture `alice_session` did not bind for `reason`.
fn lost_capture(reason: MissingCapture) -> impl FnOnce(&mut Run) {
    move |run| {
        run.captures = vec![CaptureOutcome::Missing {
            binding: "alice_session".parse().expect("a binding"),
            step_id: "f3".parse().expect("a step id"),
            reason,
        }];
    }
}

fn contrast<'a>(
    login: &'a Login,
    base_a: &'a ObservedRun,
    base_b: &'a ObservedRun,
    candidate: &'a ObservedRun,
) -> Contrast<'a> {
    Contrast {
        probe: &login.probe,
        fixture: &login.fixture,
        base_a,
        base_b,
        candidate,
    }
}

fn comparison_of(contrast: &Contrast<'_>) -> CoreDerived<Comparison> {
    let masks = compare::derive_masks(contrast).expect("the masks derive");
    compare::compare(contrast, &masks).expect("the comparison runs")
}

fn verdict_of(predicate: &Predicate, contrast: &Contrast<'_>) -> CheckResult {
    let comparison = comparison_of(contrast);
    predicate::evaluate(predicate, contrast, &comparison, &[]).expect("the predicate evaluates")
}

/// The differences of a changed comparison, as `(step, path)` pairs. Empty otherwise.
fn differences(comparison: &CoreDerived<Comparison>) -> Vec<(String, String)> {
    match &comparison.value().outcome {
        Outcome::Changed { differences } => differences
            .iter()
            .map(|difference| {
                (
                    difference.step_id.as_str().to_owned(),
                    difference.path.as_str().to_owned(),
                )
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// The scope's rate window: the first acceptance of its first `may` item.
fn rate_window() -> Predicate {
    let scope: Scope = parse(SCOPE);
    scope.body.may[0]
        .acceptance
        .iter()
        .next()
        .expect("the acceptance exists")
        .predicate
        .clone()
}

/// The rate window with its JSON edited by `edit`.
fn rate_window_with(edit: impl FnOnce(&mut Value)) -> Predicate {
    let mut value = serde_json::to_value(rate_window()).expect("the predicate serializes");
    edit(&mut value);
    serde_json::from_value(value).expect("the edited window decodes")
}

fn policy(value: &mut Value) -> &mut Value {
    &mut value["policy"]
}

/// Whether the session capture of `run` is present, as a predicate.
fn session_is(check: &str, run: &str) -> Predicate {
    serde_json::from_value(json!({
        "predicate": check,
        "at": { "ref": "capture", "run": run, "binding": "alice_session" },
    }))
    .expect("the predicate decodes")
}

// Which three runs a comparison binds.

#[test]
fn a_faithful_trio_with_distinct_sessions_is_unchanged_and_passes() {
    let login = Login::load();
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        "tok-base-a-0001",
        sequence(FIVE_THEN_LIMITED),
        |_| {},
    );
    let b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        sequence(FIVE_THEN_LIMITED),
        |_| {},
    );
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand-0003",
        sequence(FIVE_THEN_LIMITED),
        |_| {},
    );
    let contrast = contrast(&login, &a, &b, &c);
    let comparison = comparison_of(&contrast);
    assert!(matches!(comparison.value().outcome, Outcome::Unchanged {}));
    assert!(compare::verify_comparison(&contrast, &comparison).is_ok());
    assert_eq!(verdict_of(&rate_window(), &contrast), CheckResult::Pass {});
}

#[test]
fn a_comparison_of_other_runs_is_refused() {
    let login = Login::load();
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        "tok-base-a-0001",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand-0003",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let other = admit_with(
        &login,
        "run-other",
        RunRole::Candidate,
        "tok-cand-0004",
        sequence(FIVE_THEN_LIMITED),
        |_| {},
    );
    let made = comparison_of(&contrast(&login, &a, &b, &c));
    let crossed_candidate = contrast(&login, &a, &b, &other);
    assert!(matches!(
        compare::verify_comparison(&crossed_candidate, &made),
        Err(CompareError::ForeignComparison { .. })
    ));
    assert!(matches!(
        predicate::evaluate(&rate_window(), &crossed_candidate, &made, &[]),
        Err(PredicateError::Compare(
            CompareError::ForeignComparison { .. }
        ))
    ));
    let other_base_a = admit_with(
        &login,
        "run-other-base-a",
        RunRole::BaseA,
        "tok-base-a-0004",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let crossed_base = contrast(&login, &other_base_a, &b, &c);
    assert!(matches!(
        predicate::evaluate(&rate_window(), &crossed_base, &made, &[]),
        Err(PredicateError::Compare(
            CompareError::ForeignComparison { .. }
        ))
    ));
}

#[test]
fn bases_in_swapped_roles_are_refused_before_any_comparison() {
    let login = Login::load();
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        "tok-base-a-0001",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand-0003",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let swapped = Contrast {
        base_a: &b,
        base_b: &a,
        ..contrast(&login, &a, &b, &c)
    };
    assert!(matches!(
        compare::compare(&swapped, &[]),
        Err(CompareError::WrongRole { .. })
    ));
    assert!(matches!(
        compare::derive_masks(&swapped),
        Err(CompareError::WrongRole { .. })
    ));
}

#[test]
fn runs_that_differ_in_commit_runtime_or_runner_are_incompatible() {
    let login = Login::load();
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        "tok-base-a-0001",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand-0003",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let other_commit = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        sequence(SIX_FAILURES),
        |run| {
            run.commit = "0123456789abcdef0123456789abcdef01234567"
                .parse()
                .expect("a commit");
        },
    );
    let other_runtime = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        sequence(SIX_FAILURES),
        |run| run.env.runtime = "bun-9.9.9".parse().expect("a runtime id"),
    );
    let other_runner = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        sequence(SIX_FAILURES),
        |run| {
            run.runner = RunnerHash::from(
                ContentHash::of(&json!("another-runner")).expect("the value hashes"),
            );
        },
    );
    let plain_b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let candidate_runtime = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand-0003",
        sequence(SIX_FAILURES),
        |run| run.env.runtime = "bun-9.9.9".parse().expect("a runtime id"),
    );
    for (name, b, c) in [
        ("commit", &other_commit, &c),
        ("runtime", &other_runtime, &c),
        ("runner", &other_runner, &c),
        ("candidate runtime", &plain_b, &candidate_runtime),
    ] {
        let contrast = contrast(&login, &a, b, c);
        assert!(
            matches!(
                compare::compare(&contrast, &[]),
                Err(CompareError::Incompatible { .. })
            ),
            "a changed {name} was not refused as incompatible"
        );
        assert!(
            matches!(
                compare::derive_masks(&contrast),
                Err(CompareError::Incompatible { .. })
            ),
            "a changed {name} derived masks"
        );
    }
}

#[test]
fn a_timed_out_candidate_is_unknown_for_the_window() {
    let login = Login::load();
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        "tok-base-a-0001",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let mut hits = sequence(FIVE_THEN_LIMITED);
    hits.retain(|hit| hit.step != "s8");
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand-0003",
        hits,
        |run| {
            run.outcome = RunOutcome::TimedOut {
                step_id: "s6".parse().expect("a step id"),
            };
        },
    );
    assert_eq!(
        verdict_of(&rate_window(), &contrast(&login, &a, &b, &c)),
        CheckResult::Unknown {
            reason: UnknownReason::RunFailed {}
        }
    );
}

// Captures that did not bind.

#[test]
fn a_capture_that_did_not_bind_is_unknown_and_never_absent() {
    let login = Login::load();
    let absent = session_is("absent", "candidate");
    let present = session_is("present", "candidate");
    for reason in [
        MissingCapture::Absent,
        MissingCapture::Malformed,
        MissingCapture::StepFailed,
    ] {
        let a = admit_with(
            &login,
            "run-base-a",
            RunRole::BaseA,
            "tok-base-a-0001",
            sequence(SIX_FAILURES),
            |_| {},
        );
        let b = admit_with(
            &login,
            "run-base-b",
            RunRole::BaseB,
            "tok-base-b-0002",
            sequence(SIX_FAILURES),
            |_| {},
        );
        let c = admit_with(
            &login,
            "run-candidate",
            RunRole::Candidate,
            "tok-cand-0003",
            sequence(SIX_FAILURES),
            lost_capture(reason),
        );
        let contrast = contrast(&login, &a, &b, &c);
        for predicate in [&absent, &present] {
            assert_eq!(
                verdict_of(predicate, &contrast),
                CheckResult::Unknown {
                    reason: UnknownReason::Incomplete {}
                },
                "a capture that did not bind ({reason:?}) read as a value"
            );
        }
    }
}

#[test]
fn a_missing_capture_on_a_base_run_is_unknown_too() {
    let login = Login::load();
    let absent = session_is("absent", "candidate");
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        "tok-base-a-0001",
        sequence(SIX_FAILURES),
        lost_capture(MissingCapture::Absent),
    );
    let b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand-0003",
        sequence(SIX_FAILURES),
        |_| {},
    );
    assert_eq!(
        verdict_of(&absent, &contrast(&login, &a, &b, &c)),
        CheckResult::Unknown {
            reason: UnknownReason::Incomplete {}
        }
    );
}

// Identity placeholders keep type and format, and cannot be spelled by application text.

#[test]
fn a_long_hex_identity_and_a_short_text_are_different_values() {
    let login = Login::load();
    let hex_a = "3f2b8c1e5d4a4b6c8d9e0f1a2b3c4d5e";
    let hex_b = "0a1b2c3d4e5f60718293a4b5c6d7e8f9";
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        hex_a,
        with_cookie(sequence(SIX_FAILURES), "s8", hex_a),
        |_| {},
    );
    let b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        hex_b,
        with_cookie(sequence(SIX_FAILURES), "s8", hex_b),
        |_| {},
    );
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "x",
        with_cookie(sequence(SIX_FAILURES), "s8", "x"),
        |_| {},
    );
    let comparison = comparison_of(&contrast(&login, &a, &b, &c));
    assert!(
        matches!(comparison.value().outcome, Outcome::Changed { .. }),
        "a hex identity and a text identity compared equal"
    );
    assert!(differences(&comparison).contains(&("s8".to_owned(), "cookie.session".to_owned())));
}

#[test]
fn identities_of_one_format_are_equal_whatever_their_length() {
    let login = Login::load();
    let hex_a = "3f2b8c1e5d4a4b6c8d9e0f1a2b3c4d5e";
    let hex_b = "0a1b2c3d4e5f60718293a4b5c6d7e8f9";
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        hex_a,
        with_cookie(sequence(SIX_FAILURES), "s8", hex_a),
        |_| {},
    );
    let b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        hex_b,
        with_cookie(sequence(SIX_FAILURES), "s8", hex_b),
        |_| {},
    );
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "abcd1234",
        with_cookie(sequence(SIX_FAILURES), "s8", "abcd1234"),
        |_| {},
    );
    assert!(matches!(
        comparison_of(&contrast(&login, &a, &b, &c)).value().outcome,
        Outcome::Unchanged {}
    ));
}

#[test]
fn literal_text_that_spells_a_placeholder_is_a_difference() {
    let login = Login::load();
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        "tok-base-a-0001",
        with_cookie(sequence(SIX_FAILURES), "s8", "tok-base-a-0001"),
        |_| {},
    );
    let b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        with_cookie(sequence(SIX_FAILURES), "s8", "tok-base-b-0002"),
        |_| {},
    );
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand-0003",
        with_cookie(sequence(SIX_FAILURES), "s8", "<capture:alice_session>"),
        |_| {},
    );
    let contrast = contrast(&login, &a, &b, &c);
    let comparison = comparison_of(&contrast);
    assert!(differences(&comparison).contains(&("s8".to_owned(), "cookie.session".to_owned())));
    let canonical = compare::canonicalize(&compare::RunContext {
        probe: contrast.probe,
        fixture: contrast.fixture,
        run: contrast.candidate,
    })
    .expect("the candidate canonicalizes");
    let shown = canonical
        .value()
        .observations
        .iter()
        .find(|observation| observation.step_id.as_str() == "s8")
        .and_then(|observation| observation.channels.cookies.get("session"))
        .cloned();
    assert_eq!(shown.as_deref(), Some("\\<capture:alice_session>"));
}

// Declared relations are constraints on every run.

#[test]
fn a_relation_broken_on_the_base_runs_is_unknown_not_unchanged() {
    let login = Login::load();
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        "tok-base-a-0001",
        with_codes(["aaaa1111", "aaaa1111"]),
        |_| {},
    );
    let b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        with_codes(["bbbb2222", "bbbb2222"]),
        |_| {},
    );
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand-0003",
        with_codes(["cccc3333", "cccc3333"]),
        |_| {},
    );
    let comparison = comparison_of(&contrast(&login, &a, &b, &c));
    assert!(
        matches!(
            comparison.value().outcome,
            Outcome::Unknown {
                reason: UnknownReason::Incomparable { .. }
            }
        ),
        "every run breaking the unique relation was not unknown: {:?}",
        comparison.value().outcome
    );
}

#[test]
fn a_relation_that_holds_on_one_base_only_is_unstable() {
    let login = Login::load();
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        "tok-base-a-0001",
        with_codes(["aaaa1111", "bbbb2222"]),
        |_| {},
    );
    let b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        with_codes(["cccc3333", "cccc3333"]),
        |_| {},
    );
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand-0003",
        with_codes(["dddd4444", "eeee5555"]),
        |_| {},
    );
    assert!(matches!(
        comparison_of(&contrast(&login, &a, &b, &c)).value().outcome,
        Outcome::Unknown {
            reason: UnknownReason::UnstableBase {}
        }
    ));
}

// The rate window has nothing to judge when no attempt is allowed or none follows its anchor.

#[test]
fn a_window_that_rejects_every_attempt_is_unknown_not_a_pass() {
    let login = Login::load();
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        "tok-base-a-0001",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand-0003",
        sequence([429; 6]),
        |_| {},
    );
    assert!(matches!(
        verdict_of(&rate_window(), &contrast(&login, &a, &b, &c)),
        CheckResult::Unknown {
            reason: UnknownReason::Incomparable { .. }
        }
    ));
}

#[test]
fn a_window_with_no_request_after_its_anchor_is_unknown() {
    let login = Login::load();
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        "tok-base-a-0001",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand-0003",
        sequence(FIVE_THEN_LIMITED),
        |_| {},
    );
    // The anchor is the correct login at 61 s; every matching attempt was sent before it.
    let anchored = rate_window_with(|value| {
        policy(value)["anchor"] = json!({ "kind": "step", "step_id": "s8" });
    });
    assert!(matches!(
        verdict_of(&anchored, &contrast(&login, &a, &b, &c)),
        CheckResult::Unknown {
            reason: UnknownReason::Incomparable { .. }
        }
    ));
}

#[test]
fn a_request_sent_exactly_at_the_anchor_is_inside_the_window() {
    let login = Login::load();
    let a = admit_with(
        &login,
        "run-base-a",
        RunRole::BaseA,
        "tok-base-a-0001",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let b = admit_with(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b-0002",
        sequence(SIX_FAILURES),
        |_| {},
    );
    let c = admit_with(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand-0003",
        sequence(SIX_FAILURES),
        |_| {},
    );
    // s6 is sent at exactly 6000 ms, the anchor. Its attempt is the only one at or after the
    // anchor, and it is an allowed attempt, so the window has something to judge and passes.
    let anchored = rate_window_with(|value| {
        policy(value)["anchor"] = json!({ "kind": "step", "step_id": "s6" });
    });
    assert_eq!(
        verdict_of(&anchored, &contrast(&login, &a, &b, &c)),
        CheckResult::Pass {}
    );
}
