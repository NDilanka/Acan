//! Observations, their normalization, and the masks base-versus-base disagreement derives.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::hash::{CanonicalError, ContentHash, EvidenceHash, ObservationHash, ProbeHash};
use crate::probe::CaptureKind;
use crate::types::{
    ActorName, CaptureName, ClientName, HttpMethod, MaskablePath, NonEmpty, NormalizationVersion,
    ObservationPath, PathPattern, RouteTemplate, RunId, Scalar, StepId, TrustedSink,
};
use crate::version::ObservationV1;

/// Why an observation is incomplete.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "reason", rename_all = "snake_case", deny_unknown_fields)]
pub enum IncompleteReason {
    /// The response exceeded the output bound and was cut.
    OutputTruncated {},
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

/// What one step of one run produced, as recorded by the runner.
///
/// Field keys carry their channel, so `status` (the HTTP status) is never confused with a body
/// field called `status` (`body.status`).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    /// Always `acan-observation/1`.
    pub version: ObservationV1,
    /// The run.
    pub run_id: RunId,
    /// The probe.
    pub probe: ProbeHash,
    /// The step.
    pub step_id: StepId,
    /// Whether it is complete.
    pub completeness: Completeness,
    /// The request the step sent, for HTTP steps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<RequestRecord>,
    /// The observed scalars by path.
    pub fields: BTreeMap<ObservationPath, Scalar>,
    /// The raw evidence in the local store, if kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_ref: Option<EvidenceHash>,
}

impl Observation {
    /// The observation's identity, which masks and decisions bind to.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn observation_hash(&self) -> Result<ObservationHash, CanonicalError> {
        ContentHash::of(self).map(ObservationHash::from)
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

/// One projection over matching fields.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Projection {
    /// The fields.
    pub path: PathPattern,
    /// The rewrite.
    pub kind: ProjectionKind,
}

/// A relation that masked fields keep: masking a value never masks its relations.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Relation {
    /// The fields hold the same value.
    Equal {
        /// The fields.
        paths: NonEmpty<PathPattern>,
    },
    /// The fields hold pairwise distinct values.
    Unique {
        /// The fields.
        paths: NonEmpty<PathPattern>,
    },
    /// Each value at `from` appears at `to`.
    References {
        /// The referring fields.
        from: PathPattern,
        /// The referred fields.
        to: PathPattern,
    },
}

/// How a probe's observations are normalized before comparison.
///
/// Status, control flow, cardinality and step reachability are never maskable:
/// [`MaskablePath`] refuses the status channel at decode.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NormalizationPlan {
    /// The plan's version, bound into masks and the gate context.
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
/// Derived by the core from base A and base B, never from a candidate observation, and never
/// constructed outside this crate. Format, uniqueness and relation checks still apply.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct Mask {
    /// The probe.
    pub probe_hash: ProbeHash,
    /// The masked field.
    pub selector: MaskSelector,
    /// Base A's observation of that step.
    pub base_a_observation_hash: ObservationHash,
    /// Base B's observation of that step.
    pub base_b_observation_hash: ObservationHash,
    /// The normalization plan version.
    pub normalization_version: NormalizationVersion,
}

/// An observation after normalization, with the hash of the original.
///
/// Derived by the core; never constructed outside this crate.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct CanonicalObservation {
    /// The original observation.
    pub observation_hash: ObservationHash,
    /// The step.
    pub step_id: StepId,
    /// The normalization plan version.
    pub normalization_version: NormalizationVersion,
    /// The normalized scalars.
    pub fields: BTreeMap<ObservationPath, Scalar>,
}
