//! Observations, their normalization, and the masks base-versus-base disagreement derives.
//!
//! An observation keeps what a step produced as structured JSON per channel, so empty
//! containers, key sets, array lengths and the difference between a missing key and a JSON
//! `null` survive into comparison, cards and receipts. [`Observation::lookup`] is the one meaning
//! of an [`ObservationPath`].

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::hash::{
    CanonicalError, ContentHash, EvidenceHash, NormalizationHash, ObservationHash, ProbeHash,
};
use crate::probe::{CaptureKind, Run};
use crate::provenance::{EvidenceAddress, Observed};
use crate::types::{
    ActorName, CaptureName, Channel, ClientName, CommitSha, Evidence, HttpMethod, MaskablePath,
    NonEmpty, NormalizationVersion, ObservationPath, PathPattern, RouteTemplate, RunId, StepId,
    TrustedSink,
};
use crate::version::ObservationV1;

/// Why an observation is incomplete.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "reason", rename_all = "snake_case", deny_unknown_fields)]
pub enum IncompleteReason {
    /// The response exceeded the output bound and was cut.
    OutputTruncated {},
    /// The response body was not JSON. Its bytes are kept in `raw_ref` only.
    BodyNotJson {},
    /// A capture the step needed was missing.
    CaptureMissing {
        /// The binding.
        binding: CaptureName,
    },
    /// A side-effect sink did not reach its completion barrier.
    SinkIncomplete {
        /// The sink.
        sink: TrustedSink,
    },
    /// The step did not run because an earlier one failed.
    StepSkipped {},
    /// The step's deadline passed.
    DeadlineExceeded {},
}

/// Whether an observation holds everything its step should have produced.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum Completeness {
    /// Everything was recorded.
    Complete {},
    /// Something is missing. An incomplete observation never passes.
    Incomplete {
        /// Why.
        reasons: NonEmpty<IncompleteReason>,
    },
}

/// The request an HTTP step sent, as the runner recorded it. `count` and `window` predicates
/// match these.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RequestRecord {
    /// The method.
    pub method: HttpMethod,
    /// The route-table pattern the request path matched.
    pub route_template: RouteTemplate,
    /// The fixture actor whose session it carried, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<ActorName>,
    /// The client identity the runner presented.
    pub client: ClientName,
    /// The controlled clock when it was sent, in milliseconds since the run started.
    pub sent_at_ms: u64,
}

/// What one step produced, per channel, as structured JSON.
///
/// A channel that produced nothing is absent or empty: no response means no `status`, a header
/// the response did not send has no key. A body of JSON `null` is `"body": null`, which is
/// present, and differs from a response with no body, which has no `body` key.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Channels {
    /// The HTTP status, absent when no response arrived or the step is not an HTTP step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    /// Response headers by lowercase name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    /// Cookies the response set, by name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub cookies: BTreeMap<String, String>,
    /// The JSON body, absent when the response had none.
    #[serde(
        default,
        deserialize_with = "crate::types::present_json",
        skip_serializing_if = "Option::is_none"
    )]
    pub body: Option<Value>,
    /// Trusted sink snapshots, by sink.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub side_effects: BTreeMap<TrustedSink, Value>,
}

/// One step of a field path: an object key or an array index.
enum Segment<'a> {
    Key(&'a str),
    Index(&'a str),
}

/// Splits a validated field part (`.links[0].code`, `[2]`, or empty) into its segments.
fn segments(field: &str) -> Vec<Segment<'_>> {
    let mut out = Vec::new();
    let mut rest = field;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix('[') {
            let close = after.find(']').unwrap_or(after.len());
            out.push(Segment::Index(&after[..close]));
            rest = after.get(close + 1..).unwrap_or_default();
        } else {
            let key = rest.strip_prefix('.').unwrap_or(rest);
            let end = key.find(['.', '[']).unwrap_or(key.len());
            out.push(Segment::Key(&key[..end]));
            rest = &key[end..];
        }
    }
    out
}

fn walk<'v>(root: &'v Value, field: &str) -> Option<&'v Value> {
    segments(field)
        .into_iter()
        .try_fold(root, |value, segment| match segment {
            Segment::Key(key) => value.as_object()?.get(key),
            Segment::Index(index) => value.as_array()?.get(index.parse::<usize>().ok()?),
        })
}

impl Channels {
    /// What `path` holds: the value with its structure, or [`Evidence::Missing`] when the
    /// channel, key or element does not exist. Never guesses a default.
    #[must_use]
    pub fn lookup(&self, path: &ObservationPath) -> Evidence {
        let raw = path.as_str();
        let text = |map: &BTreeMap<String, String>, prefix: &str| {
            raw.strip_prefix(prefix)
                .and_then(|name| map.get(name))
                .map(|value| Value::String(value.clone()))
        };
        let found = match path.channel() {
            Channel::Status => self.status.map(Value::from),
            Channel::Header => text(&self.headers, "header."),
            Channel::Cookie => text(&self.cookies, "cookie."),
            Channel::Body => {
                let field = raw.strip_prefix("body").unwrap_or_default();
                self.body
                    .as_ref()
                    .and_then(|body| walk(body, field))
                    .cloned()
            }
            Channel::SideEffect => {
                let rest = raw.strip_prefix("side_effect.").unwrap_or_default();
                let (sink, field) = rest.split_at(rest.find(['.', '[']).unwrap_or(rest.len()));
                let sink = match sink {
                    "store" => Some(TrustedSink::Store),
                    "outbound_http" => Some(TrustedSink::OutboundHttp),
                    _ => None,
                };
                sink.and_then(|sink| self.side_effects.get(&sink))
                    .and_then(|snapshot| walk(snapshot, field))
                    .cloned()
            }
            Channel::Any => None,
        };
        found.map_or(Evidence::Missing {}, |value| Evidence::Present { value })
    }
}

/// What one step of one run produced, as recorded by the runner. Raw storage data: it decodes
/// freely, and only [`crate::admit::observed_run`] turns stored observations into evidence.
///
/// Channels are separate, so `status` (the HTTP status) is never confused with a body field
/// called `status` (`body.status`).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    /// Always `acan-observation/1`.
    pub version: ObservationV1,
    /// The run.
    pub run_id: RunId,
    /// The commit the run tested.
    pub commit: CommitSha,
    /// The probe.
    pub probe: ProbeHash,
    /// The step, recipe steps included.
    pub step_id: StepId,
    /// Whether it is complete.
    pub completeness: Completeness,
    /// The request the step sent, for HTTP steps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<RequestRecord>,
    /// What it produced.
    pub channels: Channels,
    /// The raw response or sink dump in the local store, if kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_ref: Option<EvidenceHash>,
}

impl Observation {
    /// The observation's identity, which masks, examples and evidence rows bind to.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn observation_hash(&self) -> Result<ObservationHash, CanonicalError> {
        ContentHash::of(self).map(ObservationHash::from)
    }

    /// What `path` holds in this observation. See [`Channels::lookup`].
    #[must_use]
    pub fn lookup(&self, path: &ObservationPath) -> Evidence {
        self.channels.lookup(path)
    }
}

/// One run admitted as evidence: its record and one observation per HTTP and side-effect step
/// it reached, recipe steps first, each with the hash it was stored under.
///
/// Built only by [`crate::admit::observed_run`], which checks the observations against the
/// stored hashes and the run record. It does not decode, so stored or model-written JSON can
/// never stand in for one.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ObservedRun {
    run: Run,
    observations: Vec<(ObservationHash, Observation)>,
}

impl ObservedRun {
    pub(crate) fn admitted(run: Run, observations: Vec<(ObservationHash, Observation)>) -> Self {
        Self { run, observations }
    }

    /// The run record.
    #[must_use]
    pub fn run(&self) -> &Run {
        &self.run
    }

    /// Every observation with its stored hash, in sequence order.
    pub fn observations(&self) -> impl Iterator<Item = (&ObservationHash, &Observation)> {
        self.observations
            .iter()
            .map(|(hash, observation)| (hash, observation))
    }

    /// The observed value at `path` in `step_id`'s observation, addressed by run, commit, step,
    /// path and observation hash. `None` when the step produced no observation; a path the
    /// observation lacks is `Some` with [`Evidence::Missing`].
    #[must_use]
    pub fn observed(&self, step_id: &StepId, path: &ObservationPath) -> Option<Observed> {
        let (hash, observation) = self
            .observations
            .iter()
            .find(|(_, observation)| &observation.step_id == step_id)?;
        let address = EvidenceAddress {
            run_id: self.run.id.clone(),
            commit: self.run.commit.clone(),
            step_id: step_id.clone(),
            path: path.clone(),
            observation_hash: *hash,
        };
        Some(Observed::new(observation.lookup(path), address))
    }
}

/// A capture binding and its kind, canonicalized consistently across steps.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TypedCapture {
    /// The binding.
    pub binding: CaptureName,
    /// Its kind.
    pub kind: CaptureKind,
}

/// How a projection rewrites a field before comparison. Format and structure are kept.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectionKind {
    /// Replace occurrences of a captured identity with a stable placeholder per run.
    CanonicalIdentity {
        /// The capture.
        binding: CaptureName,
    },
    /// Replace a secret-like value with its hash, keeping its length and alphabet class.
    RedactSecret {},
}

/// One projection over matching fields of every step.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Projection {
    /// The fields.
    pub path: PathPattern,
    /// The rewrite.
    pub kind: ProjectionKind,
}

/// A field pattern in one step of the sequence, so a relation can join steps.
#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct StepPath {
    /// The step, recipe steps included.
    pub step_id: StepId,
    /// The fields.
    pub path: PathPattern,
}

/// A relation that masked fields keep: masking a value never masks its relations. Relations
/// are step-qualified, so a later step can be related to an earlier one.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Relation {
    /// The fields hold the same value.
    Equal {
        /// The fields.
        paths: NonEmpty<StepPath>,
    },
    /// The fields hold pairwise distinct values.
    Unique {
        /// The fields.
        paths: NonEmpty<StepPath>,
    },
    /// Each value at `from` appears at `to`.
    References {
        /// The referring fields.
        from: StepPath,
        /// The referred fields.
        to: StepPath,
    },
}

/// How a probe's observations are normalized before comparison.
///
/// Normalization is run-wide: it reads the whole sequence of one run, recipe steps included,
/// and the run's capture fingerprints ([`crate::CaptureOutcome::Captured`]), so the same later
/// observation canonicalizes differently when earlier steps bound different identities.
/// Status, control flow, cardinality and step reachability are never maskable:
/// [`MaskablePath`] refuses the status channel at decode. Every byte of the plan is bound into
/// masks, decisions and the gate context through its [`NormalizationHash`].
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NormalizationPlan {
    /// The plan's version label.
    pub version: NormalizationVersion,
    /// The captures and their kinds.
    pub typed_captures: Vec<TypedCapture>,
    /// The projections.
    pub projections: Vec<Projection>,
    /// The relations masked fields keep.
    pub relations: Vec<Relation>,
    /// The only fields base-versus-base disagreement may mask.
    pub maskable_paths: Vec<MaskablePath>,
}

impl NormalizationPlan {
    /// The plan's identity: the hash of its whole content, not just its version label.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn normalization_hash(&self) -> Result<NormalizationHash, CanonicalError> {
        ContentHash::of(self).map(NormalizationHash::from)
    }
}

/// The field a mask covers.
#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct MaskSelector {
    /// The step.
    pub step_id: StepId,
    /// The field.
    pub path: ObservationPath,
}

/// A field the two base runs disagreed on, which comparison ignores for its value only.
///
/// This is the record a receipt lists. Only a [`crate::CoreDerived`] mask from
/// [`crate::compare::derive_masks`] reaches comparison; a decoded record is display data.
/// Format, uniqueness and relation checks still apply to a masked field.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mask {
    /// The probe.
    pub probe_hash: ProbeHash,
    /// The masked field.
    pub selector: MaskSelector,
    /// Base A's observation of that step.
    pub base_a_observation_hash: ObservationHash,
    /// Base B's observation of that step.
    pub base_b_observation_hash: ObservationHash,
    /// The normalization plan's version label.
    pub normalization_version: NormalizationVersion,
    /// The normalization plan's content.
    pub normalization_hash: NormalizationHash,
}

/// One observation after normalization, with the hash of the original.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalObservation {
    /// The original observation. Format checks read the original, not the placeholder.
    pub observation_hash: ObservationHash,
    /// The step.
    pub step_id: StepId,
    /// The normalized channels. Structure, ordering and cardinality are the original's.
    pub channels: Channels,
}

/// One run after run-wide normalization.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalRun {
    /// The run.
    pub run_id: RunId,
    /// The plan applied.
    pub normalization_hash: NormalizationHash,
    /// Every observation of the sequence, in order.
    pub observations: Vec<CanonicalObservation>,
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::compare::RunContext;

    fn observation() -> Observation {
        serde_json::from_str(include_str!("../fixtures/observation.json")).expect("the fixture")
    }

    fn at(path: &str) -> ObservationPath {
        path.parse().expect("a path")
    }

    fn present(value: Value) -> Evidence {
        Evidence::Present { value }
    }

    #[test]
    fn lookup_keeps_structure_null_and_absence_apart() {
        let mut observation = observation();
        observation.channels.body = Some(json!({
            "links": [], "meta": {}, "next": null, "items": [{ "code": "a" }]
        }));
        let lookup = |path: &str| observation.lookup(&at(path));
        assert_eq!(lookup("body.links"), present(json!([])));
        assert_eq!(lookup("body.meta"), present(json!({})));
        assert_eq!(lookup("body.next"), present(Value::Null));
        assert_eq!(lookup("body.missing"), Evidence::Missing {});
        assert_eq!(lookup("body.items[0].code"), present(json!("a")));
        assert_eq!(lookup("body.items[1]"), Evidence::Missing {});
        assert_eq!(lookup("header.retry-after"), Evidence::Missing {});
        assert_eq!(lookup("status"), present(json!(429)));
    }

    #[test]
    fn array_roots_have_one_spelling_and_keep_null_elements() {
        for refused in ["body.[0]", "side_effect.store.[0]", "body.items.[0]"] {
            assert!(refused.parse::<ObservationPath>().is_err(), "{refused}");
            assert!(refused.parse::<PathPattern>().is_err(), "{refused}");
            assert!(refused.parse::<MaskablePath>().is_err(), "{refused}");
        }
        assert!("body.[*]".parse::<PathPattern>().is_err());
        assert!("body[*].code".parse::<PathPattern>().is_ok());
        let mut observation = observation();
        observation.channels.body = Some(json!([null, { "code": "a" }]));
        observation
            .channels
            .side_effects
            .insert(TrustedSink::Store, json!([null, 7]));
        let lookup = |path: &str| observation.lookup(&at(path));
        assert_eq!(lookup("body[0]"), present(Value::Null));
        assert_eq!(lookup("body[1].code"), present(json!("a")));
        assert_eq!(lookup("body[2]"), Evidence::Missing {});
        assert_eq!(lookup("side_effect.store[0]"), present(Value::Null));
        assert_eq!(lookup("side_effect.store[1]"), present(json!(7)));
        assert_eq!(lookup("side_effect.store[2]"), Evidence::Missing {});
        assert_eq!(lookup("side_effect.outbound_http[0]"), Evidence::Missing {});
    }

    #[test]
    fn a_null_body_is_present_and_no_body_is_missing() {
        let decode = |body: Option<Value>| {
            let mut value = serde_json::to_value(observation()).expect("encoding");
            match body {
                Some(body) => value["channels"]["body"] = body,
                None => {
                    value["channels"]
                        .as_object_mut()
                        .expect("channels")
                        .remove("body");
                }
            }
            serde_json::from_value::<Observation>(value).expect("decoding")
        };
        let null = decode(Some(Value::Null));
        let none = decode(None);
        assert_eq!(null.lookup(&at("body")), present(Value::Null));
        assert_eq!(none.lookup(&at("body")), Evidence::Missing {});
        assert_ne!(
            null.observation_hash().expect("hashing"),
            none.observation_hash().expect("hashing")
        );
    }

    #[test]
    fn observed_values_carry_their_full_address() {
        let run = crate::compare::tests::admitted_run();
        let (hash, observation) = run.observations().next().expect("one observation");
        let observed = run
            .observed(&observation.step_id, &at("header.retry-after"))
            .expect("the step was observed");
        assert_eq!(observed.value(), &Evidence::Missing {});
        let address = observed.address();
        assert_eq!(&address.observation_hash, hash);
        assert_eq!(address.run_id, run.run().id);
        assert_eq!(address.commit, run.run().commit);
        assert!(
            run.observed(&"nope".parse().expect("a step"), &at("status"))
                .is_none()
        );
    }

    #[test]
    fn normalization_sees_the_whole_run_not_one_observation() {
        let corpus = crate::compare::tests::corpus();
        let last = observation();
        let earlier = |code: &str| {
            let mut first = last.clone();
            first.step_id = "s1".parse().expect("a step");
            first.channels.body = Some(json!({ "code": code }));
            first
        };
        let run = |code: &str| {
            let base = crate::compare::tests::admitted_run();
            let observations = [earlier(code), last.clone()]
                .into_iter()
                .map(|o| (o.observation_hash().expect("hashing"), o))
                .collect();
            ObservedRun::admitted(base.run().clone(), observations)
        };
        let (one, two) = (run("abc"), run("xyz"));
        let context = |run| RunContext {
            probe: &corpus.probes[0],
            fixture: &corpus.fixtures[0],
            run,
        };
        let last_of = |run: &ObservedRun| run.observations().last().map(|(h, _)| *h);
        assert_eq!(last_of(&one), last_of(&two));
        assert_ne!(context(&one).run, context(&two).run);
    }

    #[test]
    fn relations_join_steps() {
        let relation: Relation = serde_json::from_value(json!({
            "kind": "references",
            "from": { "step_id": "s4", "path": "body.links[*].code" },
            "to": { "step_id": "s2", "path": "body.code" }
        }))
        .expect("a relation");
        let Relation::References { from, to } = relation else {
            panic!("a reference");
        };
        assert_ne!(from.step_id, to.step_id);
    }

    #[test]
    fn a_plan_hash_covers_more_than_its_version_label() {
        let plan = crate::compare::tests::corpus().probes[0]
            .normalization
            .clone();
        let mut edited = plan.clone();
        edited.relations.clear();
        edited.maskable_paths.clear();
        edited
            .maskable_paths
            .push("body.anything".parse().expect("a path"));
        assert_eq!(plan.version, edited.version);
        assert_ne!(
            plan.normalization_hash().expect("hashing"),
            edited.normalization_hash().expect("hashing")
        );
    }
}
