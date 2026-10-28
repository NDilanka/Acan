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
//! - Every decision names the retained examples it was checked against by
//!   [`ContrastExampleHash`], including the one it was ruled on, and a fixed `equals_base` anchor
//!   names one of them. Decoding refuses a decision that breaks either rule. The examples and the
//!   observations they name are retained for as long as the decision exists.
//! - Derived state ([`FoldedLedger`]) rebuilds from the log and the retained examples alone,
//!   with no model call. Only [`crate::fold::fold`] makes the [`crate::CoreDerived`] copy the gate
//!   reads; a decoded [`FoldedLedger`] is display data.

use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::expr::Predicate;
use crate::gate::{GateContext, LedgerHead};
use crate::hash::{
    CanonicalError, ContentHash, ContrastExampleHash, EpisodeId, EvidenceHash, LedgerHash,
    NormalizationHash, ObservationHash, ProbeHash, ScopeHash,
};
use crate::probe::{CorpusProbe, Fixture, Run};
use crate::scope::Selector;
use crate::types::{
    HumanRef, InvalidValue, MachineId, NonEmpty, NormalizationVersion, OrderRef, RulingId,
    UnixMillis,
};
use crate::version::{ContrastExampleV1, LedgerV1};

/// One run of a retained example: its whole record, captures included, and the hashes of its
/// stored observations in sequence order.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunManifest {
    /// The run record.
    pub run: Run,
    /// Its observations, recipe steps first.
    pub observations: Vec<ObservationHash>,
}

/// A retained contrast a ruling was checked against: everything needed to rebuild it after the
/// probe, the fixture or the base have moved on. Stored by its [`ContrastExampleHash`].
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContrastExample {
    /// Always `acan-contrast-example/1`.
    pub version: ContrastExampleV1,
    /// The probe as it was, normalization plan included.
    pub probe: CorpusProbe,
    /// The fixture as it was.
    pub fixture: Fixture,
    /// The tested context of the candidate.
    pub context: GateContext,
    /// The first base run.
    pub base_a: RunManifest,
    /// The second base run.
    pub base_b: RunManifest,
    /// The candidate run.
    pub candidate: RunManifest,
}

impl ContrastExample {
    /// The example's identity, which decisions list.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn contrast_example_hash(&self) -> Result<ContrastExampleHash, CanonicalError> {
        ContentHash::of(self).map(ContrastExampleHash::from)
    }
}

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

/// A human ruling the core can execute: a predicate bound to the retained examples it was
/// checked against. Saved only if it accepts every accepted and rejects every rejected example,
/// including the one ruled on ([`crate::predicate::check_examples`]).
///
/// Decoding refuses a decision whose `ruled_on` example is not listed, whose fixed
/// `equals_base` anchors are not listed, or that lists one example as both accepted and
/// rejected.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(try_from = "DecisionConfirmedWire")]
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
    /// The examples it must accept.
    pub accepts: Vec<ContrastExampleHash>,
    /// The examples it must reject.
    pub rejects: Vec<ContrastExampleHash>,
    /// The example the human ruled on, listed in `accepts` or `rejects`.
    pub ruled_on: ContrastExampleHash,
    /// The normalization version label the examples were compared under.
    pub normalization_version: NormalizationVersion,
    /// The normalization plan's content the examples were compared under.
    pub normalization_hash: NormalizationHash,
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

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "DecisionConfirmed")]
struct DecisionConfirmedWire {
    schema_version: LedgerV1,
    event_seq: u64,
    #[serde(default)]
    prev_hash: Option<LedgerHash>,
    actor: HumanRef,
    at: UnixMillis,
    ruling_id: RulingId,
    order: OrderRef,
    scope_hash: ScopeHash,
    #[serde(default)]
    episode_id: Option<EpisodeId>,
    evidence_hashes: Vec<EvidenceHash>,
    selector: Selector,
    predicate_ast: Predicate,
    accepts: Vec<ContrastExampleHash>,
    rejects: Vec<ContrastExampleHash>,
    ruled_on: ContrastExampleHash,
    normalization_version: NormalizationVersion,
    normalization_hash: NormalizationHash,
    applicability: Applicability,
    #[serde(default)]
    supersedes: Option<Supersedes>,
    label: String,
    machine_ids: NonEmpty<MachineId>,
}

impl TryFrom<DecisionConfirmedWire> for DecisionConfirmed {
    type Error = InvalidValue;

    fn try_from(wire: DecisionConfirmedWire) -> Result<Self, Self::Error> {
        let accepts: BTreeSet<_> = wire.accepts.iter().collect();
        let rejects: BTreeSet<_> = wire.rejects.iter().collect();
        let listed = |hash: &ContrastExampleHash| accepts.contains(hash) || rejects.contains(hash);
        let problem = if accepts.intersection(&rejects).next().is_some() {
            Some("an example is both accepted and rejected".to_owned())
        } else if !listed(&wire.ruled_on) {
            Some(format!("ruled_on {} is not listed", wire.ruled_on))
        } else {
            wire.predicate_ast
                .fixed_anchors()
                .into_iter()
                .find(|anchor| !listed(anchor))
                .map(|anchor| format!("fixed anchor {anchor} is not listed"))
        };
        if let Some(raw) = problem {
            return Err(InvalidValue::Malformed {
                what: "decision",
                raw,
            });
        }
        Ok(Self {
            schema_version: wire.schema_version,
            event_seq: wire.event_seq,
            prev_hash: wire.prev_hash,
            actor: wire.actor,
            at: wire.at,
            ruling_id: wire.ruling_id,
            order: wire.order,
            scope_hash: wire.scope_hash,
            episode_id: wire.episode_id,
            evidence_hashes: wire.evidence_hashes,
            selector: wire.selector,
            predicate_ast: wire.predicate_ast,
            accepts: wire.accepts,
            rejects: wire.rejects,
            ruled_on: wire.ruled_on,
            normalization_version: wire.normalization_version,
            normalization_hash: wire.normalization_hash,
            applicability: wire.applicability,
            supersedes: wire.supersedes,
            label: wire.label,
            machine_ids: wire.machine_ids,
        })
    }
}

impl JsonSchema for DecisionConfirmed {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "DecisionConfirmed".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        DecisionConfirmedWire::json_schema(generator)
    }
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
    Deferred {},
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
    /// Its retained examples or normalization plan no longer match the corpus. Never a pass;
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
    /// The examples it accepts.
    pub accepts: Vec<ContrastExampleHash>,
    /// The examples it rejects.
    pub rejects: Vec<ContrastExampleHash>,
    /// The example it was ruled on.
    pub ruled_on: ContrastExampleHash,
    /// How far it reaches.
    pub applicability: Applicability,
    /// The normalization version label it was confirmed under.
    pub normalization_version: NormalizationVersion,
    /// The normalization plan's content it was confirmed under.
    pub normalization_hash: NormalizationHash,
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

/// The state derived from replaying the whole log. The gate reads only the
/// [`crate::CoreDerived`] copy [`crate::fold::fold`] returns.
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

impl FoldedLedger {
    #[cfg(test)]
    pub(crate) fn empty() -> Self {
        Self {
            head: LedgerHead {
                event_seq: 0,
                hash: None,
            },
            expectations: Vec::new(),
            notes: Vec::new(),
            retired_probes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn decision() -> Value {
        serde_json::from_str(include_str!("../fixtures/ledger-decision.json")).expect("fixture")
    }

    fn decodes(value: Value) -> bool {
        serde_json::from_value::<LedgerEvent>(value).is_ok()
    }

    #[test]
    fn the_ruled_on_example_must_be_listed() {
        assert!(decodes(decision()));
        let mut unlisted = decision();
        unlisted["ruled_on"] = json!(ContentHash::of_bytes(b"elsewhere").to_string());
        assert!(!decodes(unlisted));
    }

    #[test]
    fn an_example_is_never_both_accepted_and_rejected() {
        let mut both = decision();
        both["rejects"] = both["accepts"].clone();
        assert!(!decodes(both));
    }

    #[test]
    fn a_fixed_anchor_must_name_a_listed_example() {
        let mut fixed = decision();
        let listed = fixed["accepts"][0].clone();
        fixed["predicate_ast"]["anchor"] = json!({ "kind": "fixed", "example": listed });
        assert!(decodes(fixed.clone()));
        fixed["predicate_ast"]["anchor"]["example"] =
            json!(ContentHash::of_bytes(b"newer base").to_string());
        assert!(!decodes(fixed));
    }

    #[test]
    fn equals_base_always_names_its_anchor() {
        let mut anchorless = decision();
        anchorless["predicate_ast"]
            .as_object_mut()
            .expect("a predicate")
            .remove("anchor");
        assert!(!decodes(anchorless));
    }

    #[test]
    fn the_decision_fixture_rules_on_the_retained_example() {
        let example: ContrastExample =
            serde_json::from_str(include_str!("../fixtures/contrast-example.json"))
                .expect("the example fixture");
        let LedgerEvent::DecisionConfirmed(decision) =
            serde_json::from_value(decision()).expect("the decision fixture")
        else {
            panic!("a decision");
        };
        let hash = example.contrast_example_hash().expect("hashing");
        assert_eq!(decision.ruled_on, hash);
        assert_eq!(
            decision.normalization_hash,
            example
                .probe
                .normalization
                .normalization_hash()
                .expect("hashing")
        );
    }
}
