//! The plant view, `plant-view/1`: what the UI draws for one receipt snapshot.
//!
//! Trusted code builds every fact here from the receipt. The optional [`Tour`] only orders
//! camera moves over those facts: it refers to items, marks and evidence that exist in the
//! snapshot, binds caption slots to evidence references, and carries no literal values. Trusted
//! code rejects a whole tour on any invalid reference and the UI falls back to the plain card.
//!
//! L0 marks, one per fact: may-change items draw footprints, must-keep rules pins, open choices
//! question pins, machines with `coverage: uncovered` a `no probes` chip, evidence slots the
//! ghost base, `route_path` the belt crates tinted by `sequence`, `state: bounced` the
//! `bounced attempts/budget` chip, `state: residue` the amber card, `class: violates_ruling` the
//! flashing tag of `ruling_id`, enforced rulings brass tags, not-enforced rulings hollow tags,
//! unknown classes a grey mark, and `gate_state` with `merged_sha` the order plate.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::hash::{EpisodeId, EvidenceHash, ProbeHash, ReceiptHash};
use crate::probe::RoutePath;
use crate::provenance::Provenance;
use crate::report::{ClassCounts, EvidenceRow, EvidenceSlot, GateState, SequenceStep};
use crate::scope::{ChoiceOption, ItemProvenance, TargetKind};
use crate::types::{
    CatalogVersion, ChoiceId, CommitSha, ItemId, MachineId, ModelRef, MustKeepId, NonEmpty,
    ObligationId, ObservationPath, OrderRef, RouteTemplate, RulingId, RunId, Scalar, ScopeRev,
    SlotName, StepId, TemplateId, UnixMillis,
};
use crate::verdict::{ItemClass, ItemState};
use crate::version::PlantViewV1;

/// The order on the plate.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlantOrder {
    /// The order.
    pub id: OrderRef,
    /// Its sentence.
    pub sentence: String,
    /// The machine it is pinned on.
    pub machine_id: MachineId,
    /// The confirmed scope revision.
    pub scope_rev: ScopeRev,
    /// When the human confirmed it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmed_at: Option<UnixMillis>,
}

/// How well probes cover a machine.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Coverage {
    /// Every route through it has a probe.
    Covered,
    /// Some do.
    Partial,
    /// None do. Drawn hatched with a `no probes` chip.
    Uncovered,
}

/// Whether a ruling's tag is current.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RulingTagState {
    /// In force.
    Active,
    /// Replaced by a later decision.
    Superseded,
}

/// A ruling's tag on a machine.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RulingTag {
    /// The ruling lineage.
    pub ruling_id: RulingId,
    /// Brass when true, hollow `NOT ENFORCED` when false.
    pub enforced: bool,
    /// Whether it is current.
    pub state: RulingTagState,
    /// The label.
    pub label: String,
}

/// One machine of the plant.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlantMachine {
    /// The machine.
    pub machine_id: MachineId,
    /// Its probe coverage.
    pub coverage: Coverage,
    /// How many corpus probes pass through it.
    pub probes: u32,
    /// The routes whose static path passes through it.
    pub routes: Vec<RouteTemplate>,
    /// Its ruling tags.
    pub rulings: Vec<RulingTag>,
}

/// A scope example as the card shows it: a target, never an observation.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExampleView {
    /// Always `target`.
    pub kind: TargetKind,
    /// The expected base value, if the human gave one. JSON `null` is a value.
    #[serde(
        default,
        deserialize_with = "crate::types::present_scalar",
        skip_serializing_if = "Option::is_none"
    )]
    pub before: Option<Scalar>,
    /// The desired value.
    pub after: Scalar,
}

/// A may-change item's footprint.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScopeItemView {
    /// The item.
    pub item_id: ItemId,
    /// The machines it covers.
    pub machine_ids: NonEmpty<MachineId>,
    /// Its selector in the core's words.
    pub selector_text: String,
    /// Its acceptance obligations.
    pub obligations: NonEmpty<ObligationId>,
    /// Where it came from.
    pub provenance: ItemProvenance,
    /// Its example.
    pub example: ExampleView,
}

/// A must-keep pin.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MustKeepView {
    /// The rule.
    pub id: MustKeepId,
    /// The machines it guards.
    pub machine_ids: NonEmpty<MachineId>,
    /// The rule in words.
    pub text: String,
    /// The ruling it restates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ruling_id: Option<RulingId>,
}

/// A question pin.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenChoiceView {
    /// The choice.
    pub id: ChoiceId,
    /// Where the pin goes.
    pub machine_id: MachineId,
    /// The question.
    pub question: String,
    /// One key per answer.
    pub options: NonEmpty<ChoiceOption>,
}

/// The confirmed scope's marks.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScopeView {
    /// The revision, shown as `MAY CHANGE · rev N`.
    pub rev: ScopeRev,
    /// The may-change items.
    pub items: Vec<ScopeItemView>,
    /// The must-keep rules.
    pub must_keep: Vec<MustKeepView>,
    /// The open choices.
    pub open: Vec<OpenChoiceView>,
}

/// One receipt item on the plant.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlantItem {
    /// The item.
    pub item_id: ItemId,
    /// Its class.
    pub class: ItemClass,
    /// The machine its mark goes on.
    pub machine_id: MachineId,
    /// The probe that checked it.
    pub probe_id: ProbeHash,
    /// Its episode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_id: Option<EpisodeId>,
    /// The ruling it violates or relies on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ruling_id: Option<RulingId>,
    /// Repair attempts used.
    pub attempts: u32,
    /// Repair attempts allowed.
    pub budget: u32,
    /// Where it stands.
    pub state: ItemState,
    /// The machines its request passes.
    pub route_path: RoutePath,
    /// The sequence strip.
    pub sequence: Vec<SequenceStep>,
    /// The values on the card.
    pub evidence: Vec<EvidenceRow>,
}

/// The plant's counts. Each has its denominator beside it.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlantCounts {
    /// Items per class; their total is the item denominator.
    pub by_class: ClassCounts,
    /// Probes run, of `probes_total`.
    pub probes_run: u32,
    /// Probes in the corpus.
    pub probes_total: u32,
    /// Routes hit by a probe, of `routes_total`. A hit is not behavioral coverage.
    pub routes_covered: u32,
    /// Baseline routes.
    pub routes_total: u32,
    /// Masks applied.
    pub masks_applied: u32,
}

/// The view of one receipt snapshot.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlantView {
    /// Always `plant-view/1`.
    pub version: PlantViewV1,
    /// The receipt it shows.
    pub receipt_hash: ReceiptHash,
    /// The order's base commit.
    pub base_sha: CommitSha,
    /// The tested merged commit.
    pub merged_sha: CommitSha,
    /// Where the gate is.
    pub gate_state: GateState,
    /// The order.
    pub order: PlantOrder,
    /// Every machine.
    pub machines: Vec<PlantMachine>,
    /// The confirmed scope.
    pub scope: ScopeView,
    /// Every receipt item.
    pub items: Vec<PlantItem>,
    /// The counts.
    pub counts: PlantCounts,
    /// The tour, if one was made and validated. Absent means the plain card.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tour: Option<TourView>,
}

/// A zoom level of the camera catalog.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ZoomLevel {
    /// The whole plant.
    Fit,
    /// One machine and its neighbours.
    Machine,
    /// One machine up close.
    Close,
}

/// Where the camera goes for one tour step.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Camera {
    /// The machine.
    pub machine_id: MachineId,
    /// The zoom.
    pub zoom: ZoomLevel,
}

/// A mark of the snapshot a tour step stages. Every reference must exist in the snapshot.
#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "mark", rename_all = "snake_case", deny_unknown_fields)]
pub enum MarkRef {
    /// A may-change footprint.
    Footprint {
        /// The scope item.
        item_id: ItemId,
    },
    /// A must-keep pin.
    MustKeepPin {
        /// The rule.
        must_keep_id: MustKeepId,
    },
    /// A question pin.
    QuestionPin {
        /// The choice.
        choice_id: ChoiceId,
    },
    /// A `no probes` chip.
    NoProbes {
        /// The machine.
        machine_id: MachineId,
    },
    /// The ghost base machine beside an item's machine.
    Ghost {
        /// The receipt item.
        item_id: ItemId,
    },
    /// The crates of an item's sequence on its route path.
    Crates {
        /// The receipt item.
        item_id: ItemId,
    },
    /// An item's `bounced` chip.
    BouncedChip {
        /// The receipt item.
        item_id: ItemId,
    },
    /// An item's amber residue card.
    ResidueCard {
        /// The receipt item.
        item_id: ItemId,
    },
    /// A ruling's tag.
    RulingTag {
        /// The ruling.
        ruling_id: RulingId,
    },
    /// An item's grey unknown mark.
    Unknown {
        /// The receipt item.
        item_id: ItemId,
    },
    /// The order plate.
    OrderPlate {},
}

/// A reference to one evidence row of one item in the snapshot.
#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRef {
    /// The receipt item.
    pub item_id: ItemId,
    /// The column.
    pub slot: EvidenceSlot,
    /// The step.
    pub step: StepId,
    /// The field.
    pub field: ObservationPath,
}

/// A caption from the trusted template catalog, its slots bound to evidence references.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Caption {
    /// The template.
    pub template_id: TemplateId,
    /// Each slot's evidence. Never a literal.
    pub slots: BTreeMap<SlotName, EvidenceRef>,
}

/// One step of a tour.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TourStep {
    /// The receipt item the step is about.
    pub item_id: ItemId,
    /// Where the camera goes.
    pub camera: Camera,
    /// The marks it stages.
    pub stage: Vec<MarkRef>,
    /// The evidence it points at.
    pub evidence_refs: Vec<EvidenceRef>,
    /// The caption.
    pub caption: Caption,
}

/// An ordered walk over one immutable receipt snapshot.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Tour {
    /// The snapshot. A tour for any other receipt is rejected.
    pub receipt_hash: ReceiptHash,
    /// The steps. Every mandatory item must be reachable.
    pub steps: NonEmpty<TourStep>,
}

/// A validated tour and who ordered it. The UI shows a `directed` or `plain` toggle.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum TourView {
    /// Ordered by the director model and validated by trusted code.
    Directed {
        /// The director.
        model: ModelRef,
        /// The tour.
        tour: Tour,
    },
    /// The deterministic order: residue, then violations, then bounced, then fulfilled.
    Deterministic {
        /// The tour.
        tour: Tour,
    },
}

/// What kind of value a bound slot holds.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    /// An HTTP status code.
    Status,
    /// A count.
    Count,
    /// A duration.
    Duration,
    /// A commit.
    Commit,
    /// An identifier.
    Identifier,
    /// Free text.
    Text,
    /// A number.
    Number,
    /// A boolean.
    Boolean,
}

/// The unit a bound value is shown in.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ValueUnit {
    /// Milliseconds.
    Ms,
    /// Requests.
    Requests,
    /// Bytes.
    Bytes,
}

/// A value trusted code resolved from a snapshot for a caption slot, with its provenance.
/// Model layout cannot substitute a proposal for an observation or swap base and candidate:
/// the binding carries the provenance and run of the value it names.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BindableValue {
    /// What kind of value.
    pub value_type: ValueType,
    /// Who authored it.
    pub provenance: Provenance,
    /// Its unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<ValueUnit>,
    /// The snapshot.
    pub receipt_hash: ReceiptHash,
    /// The receipt item.
    pub item_id: ItemId,
    /// The run, for observed values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<RunId>,
    /// The step.
    pub step_id: StepId,
    /// The raw evidence, for observed values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_hash: Option<EvidenceHash>,
    /// The value.
    pub value: Scalar,
}

/// One item's caption slots bound by trusted code. Rendered by the trusted card; any invalid
/// binding rejects the whole view.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BoundItemView {
    /// The snapshot.
    pub receipt_hash: ReceiptHash,
    /// The receipt item.
    pub item_id: ItemId,
    /// Each slot's value.
    pub typed_slot_bindings: BTreeMap<SlotName, BindableValue>,
    /// The template catalog version.
    pub catalog_version: CatalogVersion,
}
