//! A candidate that repeats the base compares unchanged, even when the values a run mints are
//! fresh random tokens, and a masked field still keeps its format.
//!
//! The runs are the login-rate-limit probe of the corpus, admitted through `admit::observed_run`
//! so the tests judge what the gate judges. Random tokens come from a fixed seed, so every
//! build draws the same ones.

use std::collections::BTreeMap;

use acan_intent::compare::{self, Contrast};
use acan_intent::{
    CaptureOutcome, Channels, Comparison, Completeness, ContentHash, CoreDerived, CorpusProbe,
    Fixture, HttpMethod, Observation, ObservationHash, ObservedRun, Outcome, ProbeCorpus,
    ProbeHash, RequestRecord, Run, RunOutcome, RunRole, UnknownReason, admit,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

const PROBE_CORPUS: &str = include_str!("../fixtures/probe-corpus.json");
const RUN: &str = include_str!("../fixtures/run.json");
const OBSERVATION: &str = include_str!("../fixtures/observation.json");
const COMMIT: &str = "a27a5f30da19543ee34362516e29bf7561214156";
const SEED: u64 = 0xA11C_E5EE_D001_0001;
const ITERATIONS: usize = 1000;
/// The length of a 256-bit token in base64url.
const TOKEN_LENGTH: usize = 43;

fn parse<T: DeserializeOwned>(text: &str) -> T {
    serde_json::from_str(text).expect("a fixture decodes")
}

/// SplitMix64, a small generator, so the tests need no random-number dependency.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A random token of `length` characters from the base64url alphabet.
    fn token(&mut self, length: usize) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        (0..length)
            .map(|_| char::from(ALPHABET[(self.next_u64() % 64) as usize]))
            .collect()
    }
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

/// The body of one response: JSON, or text the runner keeps opaque.
enum Body {
    Json(Value),
    Text {
        content_type: &'static str,
        text: &'static str,
    },
}

/// One HTTP step as the runner records it.
struct Hit {
    step: &'static str,
    status: u16,
    sent_at_ms: u64,
    body: Body,
    cookies: Vec<(&'static str, String)>,
}

impl Hit {
    fn observation(&self, run_id: &str, probe: ProbeHash) -> Observation {
        let mut observation: Observation = parse(OBSERVATION);
        observation.run_id = run_id.parse().expect("a run id");
        observation.commit = COMMIT.parse().expect("a commit");
        observation.probe = probe;
        observation.step_id = self.step.parse().expect("a step id");
        observation.raw_ref = None;
        observation.request = Some(RequestRecord {
            method: HttpMethod::Post,
            route_template: "/api/login".parse().expect("a route template"),
            actor: None,
            client: "client-a".parse().expect("a client"),
            sent_at_ms: self.sent_at_ms,
        });
        observation.completeness = Completeness::Complete {};
        observation.channels = Channels {
            status: Some(self.status),
            headers: BTreeMap::new(),
            cookies: self
                .cookies
                .iter()
                .map(|(name, value)| ((*name).to_owned(), value.clone()))
                .collect(),
            body: None,
            side_effects: BTreeMap::new(),
        };
        match &self.body {
            Body::Json(value) => observation.channels.body = Some(value.clone()),
            Body::Text { content_type, text } => observation
                .channels
                .set_body_text(text, Some(content_type))
                .expect("the body is kept"),
        }
        observation
    }
}

/// The login sequence: six failed logins at one-second steps, then a successful login whose
/// body lists two codes and whose cookie is the session `token`.
fn sequence(token: &str, codes: &[String; 2]) -> Vec<Hit> {
    sequence_with(
        token,
        Body::Json(json!({ "links": [{ "code": codes[0] }, { "code": codes[1] }] })),
    )
}

/// The login sequence with `body` as the successful login's response.
fn sequence_with(token: &str, body: Body) -> Vec<Hit> {
    let mut hits: Vec<Hit> = ["s1", "s2", "s3", "s4", "s5", "s6"]
        .into_iter()
        .zip([1000, 2000, 3000, 4000, 5000, 6000])
        .map(|(step, sent_at_ms)| Hit {
            step,
            status: 401,
            sent_at_ms,
            body: Body::Json(json!({ "error": "invalid credentials" })),
            cookies: Vec::new(),
        })
        .collect();
    hits.push(Hit {
        step: "s8",
        status: 200,
        sent_at_ms: 61_000,
        body,
        cookies: vec![("session", token.to_owned())],
    });
    hits
}

/// Admits one run of the login probe. The recipe's signup and login come first, and the login
/// sets the session cookie `token`, which the capture `alice_session` binds.
fn admit_run(login: &Login, id: &str, role: RunRole, token: &str, hits: Vec<Hit>) -> ObservedRun {
    let probe = login.probe.spec.probe_hash().expect("the probe hashes");
    let signup = Hit {
        step: "f1",
        status: 201,
        sent_at_ms: 0,
        body: Body::Json(json!({ "ok": true })),
        cookies: Vec::new(),
    };
    let login_hit = Hit {
        step: "f2",
        status: 200,
        sent_at_ms: 0,
        body: Body::Json(json!({ "ok": true })),
        cookies: vec![("session", token.to_owned())],
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
    let stored: Vec<(ObservationHash, Observation)> = all
        .iter()
        .map(|hit| {
            let observation = hit.observation(id, probe);
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

/// Compares a candidate with two bases under the login probe, the way the gate does.
fn compare_runs(
    login: &Login,
    base_a: &ObservedRun,
    base_b: &ObservedRun,
    candidate: &ObservedRun,
) -> CoreDerived<Comparison> {
    let contrast = Contrast {
        probe: &login.probe,
        fixture: &login.fixture,
        base_a,
        base_b,
        candidate,
    };
    let masks = compare::derive_masks(&contrast).expect("the masks derive");
    compare::compare(&contrast, &masks).expect("the comparison runs")
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

/// Three runs whose session tokens and link codes are all fresh random tokens. The bases and
/// the candidate draw separately, so their values differ in every masked field.
fn random_trio(rng: &mut SplitMix64) -> [(String, [String; 2]); 3] {
    std::array::from_fn(|_| {
        let token = rng.token(TOKEN_LENGTH);
        let codes = [rng.token(TOKEN_LENGTH), rng.token(TOKEN_LENGTH)];
        (token, codes)
    })
}

#[test]
fn a_no_op_candidate_with_fresh_random_tokens_compares_unchanged() {
    let login = Login::load();
    let mut rng = SplitMix64(SEED);
    let mut not_unchanged = Vec::new();
    for iteration in 0..ITERATIONS {
        let [(token_a, codes_a), (token_b, codes_b), (token_c, codes_c)] = random_trio(&mut rng);
        let base_a = admit_run(
            &login,
            "run-base-a",
            RunRole::BaseA,
            &token_a,
            sequence(&token_a, &codes_a),
        );
        let base_b = admit_run(
            &login,
            "run-base-b",
            RunRole::BaseB,
            &token_b,
            sequence(&token_b, &codes_b),
        );
        let candidate = admit_run(
            &login,
            "run-candidate",
            RunRole::Candidate,
            &token_c,
            sequence(&token_c, &codes_c),
        );
        let comparison = compare_runs(&login, &base_a, &base_b, &candidate);
        if !matches!(comparison.value().outcome, Outcome::Unchanged {}) {
            not_unchanged.push(format!(
                "iteration {iteration}: {:?}",
                comparison.value().outcome
            ));
        }
    }
    let unstable = not_unchanged
        .iter()
        .filter(|line| line.contains("UnstableBase"))
        .count();
    assert!(
        not_unchanged.is_empty(),
        "{} of {ITERATIONS} no-op candidates did not compare unchanged ({unstable} unstable base, {} other); first: {:?}",
        not_unchanged.len(),
        not_unchanged.len() - unstable,
        not_unchanged.first()
    );
}

#[test]
fn a_one_char_or_wrong_alphabet_candidate_token_is_still_a_difference() {
    let login = Login::load();
    let mut rng = SplitMix64(SEED ^ 1);
    let [(token_a, codes_a), (token_b, codes_b), (token_c, _)] = random_trio(&mut rng);
    let base_a = admit_run(
        &login,
        "run-base-a",
        RunRole::BaseA,
        &token_a,
        sequence(&token_a, &codes_a),
    );
    let base_b = admit_run(
        &login,
        "run-base-b",
        RunRole::BaseB,
        &token_b,
        sequence(&token_b, &codes_b),
    );
    let mut wrong_alphabet = rng.token(TOKEN_LENGTH);
    wrong_alphabet.replace_range(0..1, "+");
    for (name, code) in [
        ("one character", "x".to_owned()),
        ("wrong alphabet", wrong_alphabet),
    ] {
        let codes_c = [code, rng.token(TOKEN_LENGTH)];
        let candidate = admit_run(
            &login,
            "run-candidate",
            RunRole::Candidate,
            &token_c,
            sequence(&token_c, &codes_c),
        );
        let comparison = compare_runs(&login, &base_a, &base_b, &candidate);
        assert!(
            differences(&comparison).contains(&("s8".to_owned(), "body.links[0].code".to_owned())),
            "a {name} candidate token was not a difference: {:?}",
            comparison.value().outcome
        );
    }
}

const HTML_TYPE: &str = "text/html; charset=utf-8";
const PAGE: &str = "<!doctype html><title>Links for alice</title>";
const PAGE_ONE_BYTE_OFF: &str = "<!doctype html><title>Links for alicf</title>";

/// A login run whose successful response is the HTML `page`, not JSON.
fn page_run(
    login: &Login,
    id: &str,
    role: RunRole,
    token: &str,
    page: &'static str,
) -> ObservedRun {
    admit_run(
        login,
        id,
        role,
        token,
        sequence_with(
            token,
            Body::Text {
                content_type: HTML_TYPE,
                text: page,
            },
        ),
    )
}

#[test]
fn an_identical_html_body_on_every_run_compares_unchanged() {
    let login = Login::load();
    let a = page_run(&login, "run-base-a", RunRole::BaseA, "tok-base-a", PAGE);
    let b = page_run(&login, "run-base-b", RunRole::BaseB, "tok-base-b", PAGE);
    let c = page_run(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand",
        PAGE,
    );
    let comparison = compare_runs(&login, &a, &b, &c);
    assert!(
        matches!(comparison.value().outcome, Outcome::Unchanged {}),
        "{:?}",
        comparison.value().outcome
    );
}

#[test]
fn an_html_body_that_differs_by_one_byte_is_a_difference_at_its_hash() {
    let login = Login::load();
    let a = page_run(&login, "run-base-a", RunRole::BaseA, "tok-base-a", PAGE);
    let b = page_run(&login, "run-base-b", RunRole::BaseB, "tok-base-b", PAGE);
    let c = page_run(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand",
        PAGE_ONE_BYTE_OFF,
    );
    let comparison = compare_runs(&login, &a, &b, &c);
    assert!(
        differences(&comparison)
            .contains(&("s8".to_owned(), "body.acan_opaque_body.sha256".to_owned())),
        "{:?}",
        comparison.value().outcome
    );
}

#[test]
fn html_bodies_that_differ_between_the_bases_are_an_unstable_base() {
    let login = Login::load();
    let a = page_run(&login, "run-base-a", RunRole::BaseA, "tok-base-a", PAGE);
    let b = page_run(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b",
        PAGE_ONE_BYTE_OFF,
    );
    let c = page_run(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand",
        PAGE,
    );
    assert!(matches!(
        compare_runs(&login, &a, &b, &c).value().outcome,
        Outcome::Unknown {
            reason: UnknownReason::UnstableBase {}
        }
    ));
}

#[test]
fn an_opaque_body_is_never_masked_even_when_the_plan_lists_it() {
    let mut login = Login::load();
    login.probe.normalization.maskable_paths.push(
        "body.acan_opaque_body.sha256"
            .parse()
            .expect("a maskable path"),
    );
    let a = page_run(&login, "run-base-a", RunRole::BaseA, "tok-base-a", PAGE);
    let b = page_run(
        &login,
        "run-base-b",
        RunRole::BaseB,
        "tok-base-b",
        PAGE_ONE_BYTE_OFF,
    );
    let c = page_run(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand",
        PAGE,
    );
    assert!(matches!(
        compare_runs(&login, &a, &b, &c).value().outcome,
        Outcome::Unknown {
            reason: UnknownReason::UnstableBase {}
        }
    ));
}

#[test]
fn a_json_body_where_the_base_had_an_html_body_is_a_difference() {
    let login = Login::load();
    let a = page_run(&login, "run-base-a", RunRole::BaseA, "tok-base-a", PAGE);
    let b = page_run(&login, "run-base-b", RunRole::BaseB, "tok-base-b", PAGE);
    let c = admit_run(
        &login,
        "run-candidate",
        RunRole::Candidate,
        "tok-cand",
        sequence(
            "tok-cand",
            &["code-one-0001".to_owned(), "code-two-0002".to_owned()],
        ),
    );
    assert!(matches!(
        compare_runs(&login, &a, &b, &c).value().outcome,
        Outcome::Changed { .. }
    ));
}
