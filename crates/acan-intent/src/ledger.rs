//! The ruling log, `.acan/rulings.log`: one JSON line per human decision, append-only.
//!
//! Log rules, binding on every reader and writer:
//! - Only a human member authors an event. Agents and models never do; `actor` is a
//!   [`HumanRef`] and nothing else.
//! - Events carry `event_seq` 1, 2, 3 and so on with no gaps. Each names the chain hash of the
//!   one before it in `prev_hash` (absent only on event 1), and its own chain hash is the
//!   [`LedgerHash`] of its canonical JSON.
//! - Re-appending an event byte-identical to the current head is a duplicate and changes
//!   nothing. A different event with a sequence number already used is a conflict and is
//!   refused.
//! - A truncated or undecodable line is refused, never skipped or repaired. Replay stops there
//!   with an error.
//! - Supersession is a new decision in the same `ruling_id` lineage naming the event it replaces,
//!   which must be the lineage's current head and have a lower sequence number. A second event
//!   superseding the same event is a conflict and is refused; cycles cannot be written.
//! - Derived state ([`FoldedLedger`]) rebuilds from the log alone, with no model call.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::expr::Predicate;
use crate::gate::LedgerHead;
use crate::hash::{
    CanonicalError, ContentHash, EpisodeId, EvidenceHash, LedgerHash, ObservationHash, ProbeHash,
    ScopeHash,
};
use crate::scope::Selector;
use crate::types::{
    HumanRef, MachineId, NonEmpty, NormalizationVersion, OrderRef, RulingId, UnixMillis,
};
use crate::version::LedgerV1;

/// How far a confirmed decision reaches.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Applicability {
    /// Approves exactly the accepted observations, by hash. An anchored `equals_base` never
    /// rolls forward to a new base on its own.
    ExactObservationApproval,
    /// A predicate the gate evaluates fresh on every later candidate.
    StandingPredicateApproval,
}

/// The earlier decision a new one replaces.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Supersedes {
    /// The replaced event, in the same ruling lineage.
    pub event_seq: u64,
}

/// A human ruling the core can execute: a predicate bound to the observations it was checked
/// against. Saved only if it accepts every accepted and rejects every rejected observation,
/// including the one ruled on.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecisionConfirmed {
    /// Always `acan-ledger/1`.
    pub schema_version: LedgerV1,
    /// This event's position in the log.
    pub event_seq: u64,
    /// The chain hash of the previous event. Absent only on event 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_hash: Option<LedgerHash>,
    /// The human who confirmed it.
    pub actor: HumanRef,
    /// When.
    pub at: UnixMillis,
    /// The ruling lineage.
    pub ruling_id: RulingId,
    /// The order it was ruled in.
    pub order: OrderRef,
    /// The confirmed scope in force when it was ruled.
    pub scope_hash: ScopeHash,
    /// The discrepancy episode it settles, if it settles one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_id: Option<EpisodeId>,
    /// The raw evidence the decision rests on. Retained for as long as the decision exists.
    pub evidence_hashes: Vec<EvidenceHash>,
    /// Where it applies.
    pub selector: Selector,
    /// What must hold there.
    pub predicate_ast: Predicate,
    /// The observations it must accept.
    pub accepts_hashes: Vec<ObservationHash>,
    /// The observations it must reject.
    pub rejects_hashes: Vec<ObservationHash>,
    /// The normalization version the observations were compared under.
    pub normalization_version: NormalizationVersion,
    /// How far it reaches.
    pub applicability: Applicability,
    /// The decision it replaces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<Supersedes>,
    /// The brass tag's label.
    pub label: String,
    /// The machines the tag is fixed to.
    pub machine_ids: NonEmpty<MachineId>,
}

/// Why a ruling stays a note.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NotEnforcedReason {
    /// No predicate of the contract expresses it.
    Unexpressible {
        /// What could not be expressed.
        detail: String,
    },
    /// The human deferred the decision. The residue stays open; defer never counts as fixed.
    Deferred,
}

/// A human ruling the core cannot execute. Shown with a hollow `NOT ENFORCED` tag and never
/// counted as a check. If it was required behavior, automatic land stays blocked.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NoteRecorded {
    /// Always `acan-ledger/1`.
    pub schema_version: LedgerV1,
    /// This event's position in the log.
    pub event_seq: u64,
    /// The chain hash of the previous event. Absent only on event 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_hash: Option<LedgerHash>,
    /// The human who wrote it.
    pub actor: HumanRef,
    /// When.
    pub at: UnixMillis,
    /// The ruling lineage.
    pub ruling_id: RulingId,
    /// The order it was written in.
    pub order: OrderRef,
    /// The discrepancy episode it concerns, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_id: Option<EpisodeId>,
    /// The note.
    pub text: String,
    /// Why it is not enforced.
    pub reason: NotEnforcedReason,
    /// The decision it replaces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<Supersedes>,
    /// The machines the hollow tag hangs on.
    pub machine_ids: NonEmpty<MachineId>,
}

/// A human retires a probe from the corpus. The probe stays in history.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProbeRetired {
    /// Always `acan-ledger/1`.
    pub schema_version: LedgerV1,
    /// This event's position in the log.
    pub event_seq: u64,
    /// The chain hash of the previous event. Absent only on event 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_hash: Option<LedgerHash>,
    /// The human who retired it.
    pub actor: HumanRef,
    /// When.
    pub at: UnixMillis,
    /// The probe.
    pub probe_hash: ProbeHash,
    /// Why. Required: a probe that ever exposed a difference needs a human reason.
    pub reason: String,
}

/// One line of the ruling log.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum LedgerEvent {
    /// An executable ruling.
    DecisionConfirmed(Box<DecisionConfirmed>),
    /// A ruling kept as a note.
    NoteRecorded(NoteRecorded),
    /// A retired probe.
    ProbeRetired(ProbeRetired),
}

impl LedgerEvent {
    /// The event's position in the log.
    #[must_use]
    pub fn event_seq(&self) -> u64 {
        match self {
            Self::DecisionConfirmed(event) => event.event_seq,
            Self::NoteRecorded(event) => event.event_seq,
            Self::ProbeRetired(event) => event.event_seq,
        }
    }

    /// The chain hash of the event before it.
    #[must_use]
    pub fn prev_hash(&self) -> Option<LedgerHash> {
        match self {
            Self::DecisionConfirmed(event) => event.prev_hash,
            Self::NoteRecorded(event) => event.prev_hash,
            Self::ProbeRetired(event) => event.prev_hash,
        }
    }

    /// The human who authored it.
    #[must_use]
    pub fn actor(&self) -> &HumanRef {
        match self {
            Self::DecisionConfirmed(event) => &event.actor,
            Self::NoteRecorded(event) => &event.actor,
            Self::ProbeRetired(event) => &event.actor,
        }
    }

    /// The event's chain hash: the hash of its canonical JSON, which includes `prev_hash`.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn chain_hash(&self) -> Result<LedgerHash, CanonicalError> {
        ContentHash::of(self).map(LedgerHash::from)
    }
}

/// Whether a derived expectation is in force.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ExpectationStatus {
    /// Evaluated at the gate.
    Active,
    /// Replaced by a later decision of the same lineage. Shown in its history.
    Superseded,
    /// Its evidence or normalization version no longer matches the corpus. Never a pass;
    /// the receipt lists it.
    Stale,
}

/// An executable ruling as the gate applies it. Derived from the log; never built outside this
/// crate.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct Expectation {
    /// The ruling lineage.
    pub ruling_id: RulingId,
    /// The decision it comes from.
    pub event_seq: u64,
    /// Where it applies.
    pub selector: Selector,
    /// What must hold.
    pub predicate: Predicate,
    /// The observations it accepts.
    pub accepts: Vec<ObservationHash>,
    /// The observations it rejects.
    pub rejects: Vec<ObservationHash>,
    /// How far it reaches.
    pub applicability: Applicability,
    /// The normalization version it was confirmed under.
    pub normalization_version: NormalizationVersion,
    /// Whether it is in force.
    pub status: ExpectationStatus,
    /// The tag label.
    pub label: String,
    /// The machines it is fixed to.
    pub machine_ids: NonEmpty<MachineId>,
}

/// A note as the plant shows it. Derived from the log; never built outside this crate.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RulingNote {
    /// The ruling lineage.
    pub ruling_id: RulingId,
    /// The note's event.
    pub event_seq: u64,
    /// The text.
    pub text: String,
    /// Why it is not enforced.
    pub reason: NotEnforcedReason,
    /// Whether a later event replaced it.
    pub superseded: bool,
    /// The machines it hangs on.
    pub machine_ids: NonEmpty<MachineId>,
}

/// The state derived from replaying the whole log. Derived; never built outside this crate.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct FoldedLedger {
    /// The log revision folded.
    pub head: LedgerHead,
    /// Every executable ruling, superseded ones included.
    pub expectations: Vec<Expectation>,
    /// Every note.
    pub notes: Vec<RulingNote>,
    /// Every retired probe.
    pub retired_probes: Vec<ProbeHash>,
}
