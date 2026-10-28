//! The receipt: a pure function of observations, masks, the scope and the expectations.
//!
//! Every count carries its denominator. A route counts as covered when at least one probe
//! reached it; that is a route hit, not adequate behavioral coverage, which the obligation
//! counts in [`BehaviorCounts`] report separately. The receipt is attached to the PR after the
//! commit is frozen, never inside it.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::episode::Contest;
use crate::gate::PreparedCommit;
use crate::hash::{CanonicalError, ContentHash, EpisodeId, ProbeHash, ReceiptHash, ScopeHash};
use crate::ledger::NotEnforcedReason;
use crate::observation::Mask;
use crate::probe::{RoutePath, RunRole};
use crate::provenance::Provenance;
use crate::types::{
    CommitSha, HttpMethod, InvalidValue, ItemId, MachineId, NonEmpty, ObservationPath, OrderRef,
    RouteTemplate, RulingId, RunId, Scalar, ScopeRev, StepId,
};
use crate::verdict::{Evaluation, ItemClass, ItemState};
use crate::version::ReceiptV1;

/// Items per class. Core-derived: the fields are private to this crate and no public
/// constructor exists, so a count can only come from the core's classification.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClassCounts {
    pub(crate) preserved: u32,
    pub(crate) obligation_fulfilled: u32,
    pub(crate) obligation_failed: u32,
    pub(crate) authorized_but_unspecified: u32,
    pub(crate) out_of_scope: u32,
    pub(crate) violates_ruling: u32,
    pub(crate) unstable_base: u32,
    pub(crate) incomparable: u32,
    pub(crate) error: u32,
}

impl ClassCounts {
    /// How many items have `class`.
    #[must_use]
    pub fn get(&self, class: ItemClass) -> u32 {
        match class {
            ItemClass::Preserved => self.preserved,
            ItemClass::ObligationFulfilled => self.obligation_fulfilled,
            ItemClass::ObligationFailed => self.obligation_failed,
            ItemClass::AuthorizedButUnspecified => self.authorized_but_unspecified,
            ItemClass::OutOfScope => self.out_of_scope,
            ItemClass::ViolatesRuling => self.violates_ruling,
            ItemClass::UnstableBase => self.unstable_base,
            ItemClass::Incomparable => self.incomparable,
            ItemClass::Error => self.error,
        }
    }

    /// How many items there are: the denominator of every class count.
    #[must_use]
    pub fn total(&self) -> u32 {
        ItemClass::ALL.iter().map(|class| self.get(*class)).sum()
    }
}

/// Where the gate is for the receipt's candidate. Drawn on the order plate.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum GateState {
    /// The merged commit is prepared; no run has started.
    Prepared,
    /// Runs are in progress.
    Running,
    /// An item does not pass, or a required behavior is not enforced. Land is refused.
    Blocked,
    /// The tested merged commit landed.
    Landed,
    /// The repair budget ran out.
    Stopped,
}

/// Which column of the card an evidence value belongs to.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSlot {
    /// The first base run.
    BaseA,
    /// The second base run.
    BaseB,
    /// The candidate run.
    Candidate,
    /// The scope example's desired target.
    Target,
    /// The agent's prediction.
    Prediction,
}

impl EvidenceSlot {
    /// Whether a value in this slot may carry `provenance`. Run columns hold observed or
    /// core-derived values, the target holds a human spec, and the prediction a model proposal,
    /// so nothing hypothetical is shown as observed.
    #[must_use]
    pub fn admits(self, provenance: Provenance) -> bool {
        match self {
            Self::BaseA | Self::BaseB | Self::Candidate => {
                matches!(provenance, Provenance::Observed | Provenance::CoreDerived)
            }
            Self::Target => provenance == Provenance::HumanSpec,
            Self::Prediction => provenance == Provenance::ModelProposal,
        }
    }
}

/// One value on the card, with the slot it belongs to and who authored it.
///
/// Decoding refuses a slot and provenance that do not fit ([`EvidenceSlot::admits`]) and an
/// observed value without its run and commit.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(try_from = "EvidenceRowWire")]
pub struct EvidenceRow {
    /// The column.
    pub slot: EvidenceSlot,
    /// The run, for observed values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<RunId>,
    /// The commit the run tested, for observed values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<CommitSha>,
    /// The step.
    pub step: StepId,
    /// The field.
    pub field: ObservationPath,
    /// The value.
    pub value: Scalar,
    /// Who authored it.
    pub provenance: Provenance,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "EvidenceRow")]
struct EvidenceRowWire {
    slot: EvidenceSlot,
    #[serde(default)]
    run_id: Option<RunId>,
    #[serde(default)]
    commit: Option<CommitSha>,
    step: StepId,
    field: ObservationPath,
    value: Scalar,
    provenance: Provenance,
}

impl TryFrom<EvidenceRowWire> for EvidenceRow {
    type Error = InvalidValue;

    fn try_from(wire: EvidenceRowWire) -> Result<Self, Self::Error> {
        let unanchored = wire.provenance == Provenance::Observed
            && (wire.run_id.is_none() || wire.commit.is_none());
        if !wire.slot.admits(wire.provenance) || unanchored {
            return Err(InvalidValue::Malformed {
                what: "evidence row",
                raw: format!("slot {:?} with provenance {:?}", wire.slot, wire.provenance),
            });
        }
        Ok(Self {
            slot: wire.slot,
            run_id: wire.run_id,
            commit: wire.commit,
            step: wire.step,
            field: wire.field,
            value: wire.value,
            provenance: wire.provenance,
        })
    }
}

impl JsonSchema for EvidenceRow {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "EvidenceRow".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        EvidenceRowWire::json_schema(generator)
    }
}

/// How one step of a probe sequence went, for the crate tint and the sequence strip.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum StepTint {
    /// The step matched base.
    Same,
    /// The step differed from base.
    Differed,
    /// The step has no answer. Drawn grey, never green.
    Unknown,
}

/// One request of a probe sequence in one run.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SequenceStep {
    /// The step.
    pub step_id: StepId,
    /// The request's position in the sequence, from 1.
    pub request: u32,
    /// The run.
    pub run_id: RunId,
    /// The run's role.
    pub role: RunRole,
    /// The observed HTTP status, if the request completed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    /// The tint.
    pub tint: StepTint,
}

/// One classified receipt item.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReceiptItem {
    /// A scope item, or an item the core minted for a check outside the scope.
    pub item_id: ItemId,
    /// Its class.
    pub class: ItemClass,
    /// Where it stands in the repair loop.
    pub state: ItemState,
    /// The machine its mark goes on.
    pub machine_id: MachineId,
    /// The probe that checked it.
    pub probe_hash: ProbeHash,
    /// Its episode, if it failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_id: Option<EpisodeId>,
    /// The ruling it violates or relies on, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ruling_id: Option<RulingId>,
    /// Repair attempts used.
    pub attempts: u32,
    /// Repair attempts allowed.
    pub budget: u32,
    /// The machines its request passes.
    pub route_path: RoutePath,
    /// The sequence strip.
    pub sequence: Vec<SequenceStep>,
    /// The values on the card.
    pub evidence: Vec<EvidenceRow>,
    /// What the core weighed.
    pub evaluation: Evaluation,
}

/// The probe denominator.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProbeCounts {
    /// Probes in the corpus, retired ones excluded.
    pub total: u32,
    /// Probes run on base A, base B and the candidate.
    pub run: u32,
    /// Probes whose runs did not complete.
    pub errored: u32,
}

/// A baseline route no probe exercises.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UncoveredRoute {
    /// The method.
    pub method: HttpMethod,
    /// The route.
    pub route_template: RouteTemplate,
    /// Why, from the corpus.
    pub reason: String,
}

/// The route denominator. `covered` counts route hits only.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouteCounts {
    /// Every method-and-route pattern of the baseline route table.
    pub total: u32,
    /// Patterns at least one run probe reached.
    pub covered: u32,
    /// Patterns no probe exercises.
    pub uncovered: Vec<UncoveredRoute>,
}

/// The behavioral denominator: obligations, not route hits.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BehaviorCounts {
    /// Acceptance, must-keep and ruling obligations that apply.
    pub obligations_total: u32,
    /// Those that passed.
    pub passed: u32,
    /// Those that failed.
    pub failed: u32,
    /// Those with no answer. Never a pass.
    pub unknown: u32,
}

/// How one ruling fared at the gate.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum RulingStatus {
    /// Executed, and every probe that reached its selector satisfied it.
    Held {},
    /// Executed, and the candidate violated it.
    Violated {
        /// The violating items.
        item_ids: NonEmpty<ItemId>,
    },
    /// Executed, but no probe reached its selector. Not counted as held.
    NotReached {},
    /// Its evidence or normalization version no longer matches. Never a pass.
    Stale {},
    /// A note. Never counted as a check.
    NotEnforced {
        /// Why.
        reason: NotEnforcedReason,
    },
}

/// One ruling's line on the receipt.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RulingCheck {
    /// The ruling lineage.
    pub ruling_id: RulingId,
    /// The decision in force.
    pub event_seq: u64,
    /// The tag label.
    pub label: String,
    /// The machines it is fixed to.
    pub machine_ids: NonEmpty<MachineId>,
    /// How it fared.
    pub status: RulingStatus,
}

/// The receipt of one candidate. Core-derived: every class, count and denominator is a pure
/// function of observations, masks, the scope and the expectations.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    /// Always `acan-receipt/1`.
    pub version: ReceiptV1,
    /// The order.
    pub order: OrderRef,
    /// The confirmed scope revision.
    pub scope_rev: ScopeRev,
    /// Its hash.
    pub scope_hash: ScopeHash,
    /// The base, overlay and tested context. The receipt is void for any other merged commit.
    pub prepared: PreparedCommit,
    /// Where the gate is.
    pub gate_state: GateState,
    /// Every classified item.
    pub items: Vec<ReceiptItem>,
    /// Items per class, with the total as denominator.
    pub counts: ClassCounts,
    /// The probe denominator.
    pub probes: ProbeCounts,
    /// The route denominator.
    pub routes: RouteCounts,
    /// The behavioral denominator.
    pub behavior: BehaviorCounts,
    /// Every mask applied, each bound to the two base observations that derived it.
    pub masks: Vec<Mask>,
    /// Every ruling, enforced or not.
    pub rulings: Vec<RulingCheck>,
    /// Every contest filed in the order.
    pub contests: Vec<Contest>,
}

impl Receipt {
    /// The receipt's identity, which a plant view and a tour bind to.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn receipt_hash(&self) -> Result<ReceiptHash, CanonicalError> {
        ContentHash::of(self).map(ReceiptHash::from)
    }

    /// The tested merged commit.
    #[must_use]
    pub fn merged_commit(&self) -> &CommitSha {
        &self.prepared.context.merged_commit
    }

    /// The order's base commit.
    #[must_use]
    pub fn base_commit(&self) -> &CommitSha {
        &self.prepared.base_commit
    }
}
