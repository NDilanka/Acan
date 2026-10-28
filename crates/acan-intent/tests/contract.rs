//! Contract tests: every fixture round-trips, unknown fields and versions are refused, the pass
//! rule holds over every class, and every stub fails closed.

use std::path::{Path, PathBuf};

use acan_intent::compare::{self, CompareError, Contrast, RunEvidence};
use acan_intent::fold::{self, FoldError};
use acan_intent::predicate::{self, PredicateError};
use acan_intent::receipt::{self, ProbeResult, ReceiptError, ReceiptInputs};
use acan_intent::{
    AppContract, Comparison, Contest, ContestRequest, ContestResponse, Coverage,
    DiscrepancyEpisode, EvidenceRow, FieldPattern, FoldedLedger, GateState, GetReceiptRequest,
    GetReceiptResponse, GetRulingsRequest, GetRulingsResponse, GetScopeRequest, GetScopeResponse,
    ItemClass, ItemState, LedgerEvent, MaskablePath, McpError, McpErrorCode, Observation,
    ObservationPath, PlantView, PostReadbackRequest, PostReadbackResponse, ProbeCorpus, Receipt,
    RouteSource, Run, Scope, ScopeConfirmation, ScopeDraft, ScopeItem, Selector, TourView,
    Unsupported, WindowMs,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn raw(name: &str) -> Value {
    let path = fixtures().join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("{name} is not JSON: {err}"))
}

fn load<T: DeserializeOwned>(name: &str) -> T {
    serde_json::from_value(raw(name)).unwrap_or_else(|err| panic!("decoding {name}: {err}"))
}

/// Decodes, encodes, decodes again, and checks both the values and the JSON are unchanged.
fn round_trip<T: DeserializeOwned + Serialize + PartialEq + std::fmt::Debug>(name: &str) {
    let original = raw(name);
    let first: T = serde_json::from_value(original.clone())
        .unwrap_or_else(|err| panic!("decoding {name}: {err}"));
    let encoded = serde_json::to_value(&first).expect("encoding");
    if let Some(pointer) = first_difference(&encoded, &original, String::new()) {
        panic!("{name} does not re-encode to the same JSON at '{pointer}'");
    }
    let second: T = serde_json::from_value(encoded).expect("decoding the re-encoded value");
    assert_eq!(first, second, "{name} changed across a round trip");
}

/// The JSON pointer of the first place two values differ.
fn first_difference(left: &Value, right: &Value, pointer: String) -> Option<String> {
    match (left, right) {
        (Value::Object(a), Value::Object(b)) => {
            let keys: std::collections::BTreeSet<&String> = a.keys().chain(b.keys()).collect();
            keys.into_iter()
                .find_map(|key| match (a.get(key), b.get(key)) {
                    (Some(x), Some(y)) => first_difference(x, y, format!("{pointer}/{key}")),
                    _ => Some(format!("{pointer}/{key}")),
                })
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => a
            .iter()
            .zip(b)
            .enumerate()
            .find_map(|(index, (x, y))| first_difference(x, y, format!("{pointer}/{index}"))),
        _ if left == right => None,
        _ => Some(pointer),
    }
}

/// Objects whose keys are data, not field names.
const MAP_KEYS: [&str; 5] = [
    "fields",
    "slots",
    "env",
    "headers",
    "effective_artifact_hashes",
];

/// Every JSON pointer to an object that is a struct, not a map or free JSON.
fn struct_pointers(
    value: &Value,
    pointer: String,
    parent_key: Option<&str>,
    out: &mut Vec<String>,
) {
    match value {
        Value::Object(map) => {
            let is_map = parent_key.is_some_and(|key| MAP_KEYS.contains(&key));
            let is_free_json = parent_key == Some("body") && pointer.ends_with("/request/body");
            if is_free_json {
                return;
            }
            if !is_map {
                out.push(pointer.clone());
            }
            for (key, child) in map {
                let escaped = key.replace('~', "~0").replace('/', "~1");
                struct_pointers(child, format!("{pointer}/{escaped}"), Some(key), out);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                struct_pointers(child, format!("{pointer}/{index}"), parent_key, out);
            }
        }
        _ => {}
    }
}

/// Inserts an unknown field into every struct of the fixture, one at a time, and checks each
/// insertion makes decoding fail.
fn refuses_unknown_fields<T: DeserializeOwned + std::fmt::Debug>(name: &str) {
    let original = raw(name);
    let mut pointers = Vec::new();
    struct_pointers(&original, String::new(), None, &mut pointers);
    assert!(!pointers.is_empty());
    for pointer in pointers {
        let mut tampered = original.clone();
        let target = tampered
            .pointer_mut(&pointer)
            .and_then(Value::as_object_mut)
            .expect("pointer names an object");
        target.insert("zz_unknown".to_owned(), json!("x"));
        let decoded = serde_json::from_value::<T>(tampered);
        assert!(
            decoded.is_err(),
            "{name} accepted an unknown field at '{pointer}'"
        );
    }
}

/// Replaces the top-level version tag with an unknown one and checks decoding fails.
fn refuses_unknown_version<T: DeserializeOwned + std::fmt::Debug>(name: &str) {
    let mut tampered = raw(name);
    let object = tampered.as_object_mut().expect("an envelope is an object");
    let key = ["version", "schema_version"]
        .into_iter()
        .find(|key| object.contains_key(*key))
        .unwrap_or_else(|| panic!("{name} has no version tag"));
    let current = object[key]
        .as_str()
        .expect("a version is a string")
        .to_owned();
    let (family, _) = current.split_once('/').expect("versions are family/n");
    object.insert(key.to_owned(), json!(format!("{family}/999")));
    let err = serde_json::from_value::<T>(tampered).expect_err("an unknown version decoded");
    assert!(
        err.to_string().contains("unsupported"),
        "{name}: unexpected error {err}"
    );
}

macro_rules! envelopes {
    ($($test:ident: $ty:ty = $file:literal;)+) => {$(
        mod $test {
            use super::*;

            #[test]
            fn round_trips() {
                round_trip::<$ty>($file);
            }

            #[test]
            fn refuses_unknown_fields() {
                super::refuses_unknown_fields::<$ty>($file);
            }

            #[test]
            fn refuses_unknown_version() {
                super::refuses_unknown_version::<$ty>($file);
            }
        }
    )+};
}

envelopes! {
    plant_view: PlantView = "plant-view-v1.json";
    plant_view_plain: PlantView = "plant-view-v1-plain.json";
    scope_draft: ScopeDraft = "scope-draft.json";
    scope: Scope = "scope.json";
    scope_confirmation: ScopeConfirmation = "scope-confirmation.json";
    ledger_decision: LedgerEvent = "ledger-decision.json";
    ledger_decision_supersede: LedgerEvent = "ledger-decision-supersede.json";
    ledger_note: LedgerEvent = "ledger-note.json";
    ledger_probe_retired: LedgerEvent = "ledger-probe-retired.json";
    episode: DiscrepancyEpisode = "episode.json";
    contest: Contest = "contest.json";
    receipt_envelope: Receipt = "receipt.json";
    probe_corpus: ProbeCorpus = "probe-corpus.json";
    app_contract: AppContract = "app-contract.json";
    run: Run = "run.json";
    run_setup_failed: Run = "run-setup-failed.json";
    observation: Observation = "observation.json";
    observation_incomplete: Observation = "observation-incomplete.json";
    mcp_get_scope_request: GetScopeRequest = "mcp-get-scope-request.json";
    mcp_get_scope_response: GetScopeResponse = "mcp-get-scope-response.json";
    mcp_get_rulings_request: GetRulingsRequest = "mcp-get-rulings-request.json";
    mcp_get_rulings_response: GetRulingsResponse = "mcp-get-rulings-response.json";
    mcp_post_readback_request: PostReadbackRequest = "mcp-post-readback-request.json";
    mcp_post_readback_response: PostReadbackResponse = "mcp-post-readback-response.json";
    mcp_contest_request: ContestRequest = "mcp-contest-request.json";
    mcp_contest_response: ContestResponse = "mcp-contest-response.json";
    mcp_get_receipt_request: GetReceiptRequest = "mcp-get-receipt-request.json";
    mcp_get_receipt_response: GetReceiptResponse = "mcp-get-receipt-response.json";
    mcp_error_stale_scope: McpError = "mcp-error-stale-scope.json";
    mcp_error_forbidden: McpError = "mcp-error-forbidden.json";
    mcp_error_unsupported: McpError = "mcp-error-unsupported.json";
    mcp_error_scope_not_confirmed: McpError = "mcp-error-scope-not-confirmed.json";
}

#[test]
fn every_fixture_is_covered() {
    let mut names: Vec<String> = std::fs::read_dir(fixtures())
        .expect("listing fixtures")
        .map(|entry| {
            entry
                .expect("a fixture entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| name.ends_with(".json") && !name.ends_with(".schema.json"))
        .collect();
    names.sort();
    let source = include_str!("contract.rs");
    for name in names {
        assert!(
            source.contains(&format!("\"{name}\"")),
            "{name} has no round-trip test"
        );
    }
}

#[test]
fn only_preserved_and_fulfilled_pass() {
    for class in ItemClass::ALL {
        let expected = matches!(class, ItemClass::Preserved | ItemClass::ObligationFulfilled);
        assert_eq!(class.counts_toward_pass(), expected, "{class:?}");
    }
    assert!(!ItemClass::AuthorizedButUnspecified.counts_toward_pass());
    let names: Vec<Value> = ItemClass::ALL
        .iter()
        .map(|class| serde_json::to_value(class).expect("encoding a class"))
        .collect();
    assert_eq!(
        names,
        [
            "preserved",
            "obligation_fulfilled",
            "obligation_failed",
            "authorized_but_unspecified",
            "out_of_scope",
            "violates_ruling",
            "unstable_base",
            "incomparable",
            "error"
        ]
        .map(Value::from)
    );
}

#[test]
fn plant_view_fixture_shows_every_mark() {
    let view: PlantView = load("plant-view-v1.json");
    for class in ItemClass::ALL {
        assert!(
            view.items.iter().any(|item| item.class == class),
            "no {class:?} item"
        );
        assert_eq!(view.counts.by_class.get(class), 1);
    }
    assert_eq!(view.counts.by_class.total() as usize, view.items.len());
    for state in [
        ItemState::Open,
        ItemState::Bounced,
        ItemState::Fixed,
        ItemState::Residue,
        ItemState::Stopped,
    ] {
        assert!(
            view.items.iter().any(|item| item.state == state),
            "no {state:?} item"
        );
    }
    let tags: Vec<_> = view
        .machines
        .iter()
        .flat_map(|machine| &machine.rulings)
        .collect();
    assert!(tags.iter().any(|tag| tag.enforced));
    assert!(tags.iter().any(|tag| !tag.enforced));
    assert!(
        view.machines
            .iter()
            .any(|machine| machine.coverage == Coverage::Uncovered && machine.probes == 0)
    );
    assert!(
        view.items
            .iter()
            .any(|item| item.route_path.source == RouteSource::Static)
    );
    assert!(view.counts.masks_applied > 0);
    assert!(!view.scope.items.is_empty() && !view.scope.must_keep.is_empty());
    assert!(!view.scope.open.is_empty());
    let machines: Vec<&str> = view
        .machines
        .iter()
        .map(|m| m.machine_id.as_str())
        .collect();
    assert_eq!(
        machines,
        [
            "auth",
            "dashboard",
            "errors",
            "links",
            "middleware",
            "ratelimit",
            "server",
            "store",
            "test"
        ]
    );
    assert!(matches!(view.tour, Some(TourView::Directed { .. })));
}

#[test]
fn plain_fixture_is_the_same_view_without_a_tour() {
    let directed: PlantView = load("plant-view-v1.json");
    let plain: PlantView = load("plant-view-v1-plain.json");
    assert!(plain.tour.is_none());
    assert_eq!(
        PlantView {
            tour: None,
            ..directed
        },
        plain
    );
}

#[test]
fn views_and_tour_bind_the_receipt_fixture() {
    let receipt: Receipt = load("receipt.json");
    let hash = receipt.receipt_hash().expect("hashing the receipt");
    let view: PlantView = load("plant-view-v1.json");
    assert_eq!(view.receipt_hash, hash);
    let Some(TourView::Directed { tour, .. }) = &view.tour else {
        panic!("the fixture has a directed tour");
    };
    assert_eq!(tour.receipt_hash, hash);
    let response: GetReceiptResponse = load("mcp-get-receipt-response.json");
    assert_eq!(response.receipt_hash, hash);
    assert_eq!(response.receipt, receipt);
    assert_eq!(view.merged_sha, *receipt.merged_commit());
    assert_eq!(view.base_sha, *receipt.base_commit());
    assert_eq!(view.gate_state, GateState::Blocked);
}

#[test]
fn plant_view_schema_is_current() {
    let schema = schemars::schema_for!(PlantView);
    let generated = serde_json::to_string_pretty(&schema).expect("encoding the schema") + "\n";
    let path = fixtures().join("plant-view-v1.schema.json");
    if std::env::var_os("ACAN_BLESS").is_some() {
        std::fs::write(&path, &generated).expect("writing the schema");
    }
    let committed = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert_eq!(
        committed, generated,
        "plant-view-v1.schema.json is stale; rerun with ACAN_BLESS=1"
    );
}

#[test]
fn an_unknown_plant_view_version_names_the_problem() {
    let mut value = raw("plant-view-v1.json");
    value["version"] = json!("plant-view/2");
    let err = serde_json::from_value::<PlantView>(value).expect_err("plant-view/2 decoded");
    assert!(err.to_string().contains("plant-view/2"), "{err}");
}

#[test]
fn status_is_a_channel_not_a_body_field() {
    let status: ObservationPath = "status".parse().expect("status");
    let body_status: ObservationPath = "body.status".parse().expect("body.status");
    assert_ne!(status.channel(), body_status.channel());
    assert!("status.code".parse::<ObservationPath>().is_err());
    assert!("body.links[*].code".parse::<ObservationPath>().is_err());
    assert!("status".parse::<MaskablePath>().is_err());
    assert!("body.links[*].code".parse::<MaskablePath>().is_ok());
    let observation: Observation = load("observation.json");
    assert!(observation.fields.contains_key(&status));
    assert!(observation.fields.contains_key(&body_status));
}

#[test]
fn selectors_refuse_inconsistent_field_paths() {
    let base = json!({
        "surface": "http", "method": "POST", "route_template": "/api/login",
        "step_context": { "kind": "any" }, "principal": { "kind": "anonymous" }
    });
    let with = |channel: &str, field: Option<&str>| {
        let mut value = base.clone();
        value["channel"] = json!(channel);
        if let Some(field) = field {
            value["field_path"] = json!(field);
        }
        serde_json::from_value::<Selector>(value)
    };
    assert!(with("status", None).is_ok());
    assert!(with("status", Some("code")).is_err());
    assert!(with("any", Some("error")).is_err());
    assert!(with("header", Some("retry-after")).is_ok());
    assert!(with("header", Some("a.b")).is_err());
    assert!(with("side_effect", Some("store.users[*].name")).is_ok());
    assert!(with("side_effect", Some("users[*].name")).is_err());
    assert!(with("side_effect", None).is_err());
    assert!("links[*].code".parse::<FieldPattern>().is_ok());
}

#[test]
fn a_scope_item_without_acceptance_is_refused() {
    let scope: Scope = load("scope.json");
    let mut item = serde_json::to_value(&scope.body.may[0]).expect("encoding an item");
    item["acceptance"] = json!([]);
    assert!(serde_json::from_value::<ScopeItem>(item).is_err());
}

#[test]
fn evidence_rows_keep_hypotheticals_out_of_run_columns() {
    let row = |slot: &str, provenance: &str| {
        serde_json::from_value::<EvidenceRow>(json!({
            "slot": slot, "run_id": "run-1", "commit": "0123456789abcdef0123456789abcdef01234567",
            "step": "s1", "field": "status", "value": 200, "provenance": provenance
        }))
    };
    assert!(row("candidate", "observed").is_ok());
    assert!(row("candidate", "model_proposal").is_err());
    assert!(row("base_a", "human_spec").is_err());
    assert!(row("prediction", "observed").is_err());
    assert!(row("target", "human_spec").is_ok());
    let unanchored = json!({
        "slot": "candidate", "step": "s1", "field": "status", "value": 200,
        "provenance": "observed"
    });
    assert!(serde_json::from_value::<EvidenceRow>(unanchored).is_err());
}

#[test]
fn bounded_numbers_refuse_out_of_range_values() {
    assert!(serde_json::from_value::<WindowMs>(json!(0)).is_err());
    assert!(serde_json::from_value::<WindowMs>(json!(86_400_001_u64)).is_err());
    assert!(serde_json::from_value::<WindowMs>(json!(60_000)).is_ok());
}

#[test]
fn episode_identity_ignores_probe_spelling() {
    let episode: DiscrepancyEpisode = load("episode.json");
    let respelled = DiscrepancyEpisode {
        probe_hash: episode.evidence_revisions[0].probe_hash,
        step_id: episode.evidence_revisions[0].step_id.clone(),
        ..episode.clone()
    };
    assert_ne!(respelled.probe_hash, episode.probe_hash);
    let id = episode.key().episode_id().expect("hashing the key");
    assert_eq!(respelled.key().episode_id().expect("hashing the key"), id);
}

#[test]
fn hashes_do_not_depend_on_field_order() {
    let scope: Scope = load("scope.json");
    let mut value = serde_json::to_value(&scope).expect("encoding");
    let reordered: serde_json::Map<String, Value> = value
        .as_object_mut()
        .expect("an object")
        .iter()
        .rev()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    let reparsed: Scope = serde_json::from_value(Value::Object(reordered)).expect("decoding");
    assert_eq!(
        scope.scope_hash().expect("hashing"),
        reparsed.scope_hash().expect("hashing")
    );
    let shown = scope.scope_hash().expect("hashing").to_string();
    assert!(shown.starts_with("sha256:") && shown.len() == 71);
}

#[test]
fn the_unsupported_mcp_error_names_the_operation() {
    let built = McpError::unsupported(Unsupported("API01: get_scope"));
    let fixture: McpError = load("mcp-error-unsupported.json");
    assert_eq!(built, fixture);
    assert!(matches!(fixture.error, McpErrorCode::Unsupported { .. }));
}

fn comparison() -> Comparison {
    serde_json::from_value(json!({
        "probe_hash": raw("run.json")["probe"],
        "base_a": [], "base_b": [], "candidate": [],
        "outcome": { "kind": "unchanged" }
    }))
    .expect("decoding a comparison")
}

#[test]
fn every_p01_stub_fails_closed() {
    let corpus: ProbeCorpus = load("probe-corpus.json");
    let run: Run = load("run.json");
    let observations = vec![load::<Observation>("observation.json")];
    let evidence = RunEvidence {
        run: &run,
        observations: &observations,
    };
    let contrast = Contrast {
        probe: &corpus.probes[0],
        base_a: evidence,
        base_b: evidence,
        candidate: evidence,
    };
    let plan = &corpus.probes[0].normalization;
    let comparison = comparison();
    let scope: Scope = load("scope.json");
    let predicate = &scope.body.may[0].acceptance.first().predicate;

    let unsupported = |op: &str| Unsupported(Box::leak(op.to_owned().into_boxed_str()));
    let compare_failed = |result: Result<(), CompareError>, op: &str| {
        assert_eq!(result, Err(CompareError::Unsupported(unsupported(op))));
    };
    compare_failed(
        compare::canonicalize(&observations[0], plan).map(drop),
        "P01: canonicalize",
    );
    compare_failed(
        compare::derive_masks(&contrast).map(drop),
        "P01: derive_masks",
    );
    compare_failed(compare::compare(&contrast, &[]).map(drop), "P01: compare");

    assert_eq!(
        predicate::evaluate(predicate, &contrast, &comparison),
        Err(PredicateError::Unsupported(unsupported("P01: evaluate")))
    );
    assert_eq!(
        predicate::check_examples(predicate, &[(contrast, &comparison)], &[]),
        Err(PredicateError::Unsupported(unsupported(
            "P01: check_examples"
        )))
    );

    let events: Vec<LedgerEvent> = [
        "ledger-decision.json",
        "ledger-note.json",
        "ledger-decision-supersede.json",
        "ledger-probe-retired.json",
    ]
    .into_iter()
    .map(load)
    .collect();
    assert_eq!(
        fold::fold(&events),
        Err(FoldError::Unsupported(unsupported("P01: fold")))
    );

    let receipt_fixture: Receipt = load("receipt.json");
    let confirmation: ScopeConfirmation = load("scope-confirmation.json");
    let ledger: FoldedLedger = serde_json::from_value(json!({
        "head": { "event_seq": 0 }, "expectations": [], "notes": [], "retired_probes": []
    }))
    .expect("decoding a folded ledger");
    let results = [ProbeResult {
        contrast,
        masks: &[],
        comparison: &comparison,
    }];
    let inputs = ReceiptInputs {
        scope: &scope,
        confirmation: &confirmation,
        prepared: &receipt_fixture.prepared,
        ledger: &ledger,
        corpus: &corpus,
        results: &results,
        episodes: &[],
        contests: &[],
        gate_state: GateState::Running,
    };
    assert_eq!(
        receipt::build_receipt(&inputs),
        Err(ReceiptError::Unsupported(unsupported("P01: build_receipt")))
    );
}
