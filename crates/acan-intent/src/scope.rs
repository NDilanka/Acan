//! Scopes: what an order may change, what it must keep, and the human confirmation that binds it.
//!
//! Precedence, highest first: active rulings, then must-keep obligations, then may-change items.
//! A may-change item never licenses a difference that a ruling or a must-keep obligation rejects.
//! Permission is never a pass: an authorized difference passes only through its acceptance
//! obligations.

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::expr::Predicate;
use crate::hash::{CanonicalError, ContentHash, ExampleHash, ScopeHash, SelectorHash};
use crate::provenance::ModelProposal;
use crate::types::{
    ActorName, Channel, ChoiceId, CommitSha, FieldPattern, HumanRef, InvalidValue, ItemId,
    MachineId, MethodSelector, MustKeepId, NonEmpty, ObligationId, ObservationPath, OptionKey,
    OrderRef, RouteTemplate, RulingId, Scalar, ScopeRev, StepId, Surface, UnixMillis,
};
use crate::version::{ScopeConfirmationV1, ScopeDraftV1, ScopeV1};

/// Which step of a probe sequence a selector applies to.
#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StepContext {
    /// Every step.
    Any {},
    /// One step.
    Step {
        /// The step.
        step_id: StepId,
    },
}

/// Whose request a selector applies to.
#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Principal {
    /// Anyone.
    Any {},
    /// Requests without a session.
    Anonymous {},
    /// Requests with any session.
    Authenticated {},
    /// Requests sent as one fixture actor.
    Actor {
        /// The actor.
        name: ActorName,
    },
}

/// Where in the app a scope item, must-keep rule or ruling applies.
///
/// `field_path` is relative to `channel` and absent for `status` and `any`. A header or cookie
/// path is the single lowercase name. A `side_effect` path starts with the sink
/// (`store.users[*].name`, `outbound_http`) and is required. Decoding refuses any other mix.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(try_from = "SelectorWire")]
pub struct Selector {
    /// The interface kind.
    pub surface: Surface,
    /// The method.
    pub method: MethodSelector,
    /// The route pattern.
    pub route_template: RouteTemplate,
    /// Which steps.
    pub step_context: StepContext,
    /// Whose requests.
    pub principal: Principal,
    /// Which part of the response.
    pub channel: Channel,
    /// Which field within the channel, or the whole channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_path: Option<FieldPattern>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "Selector")]
struct SelectorWire {
    surface: Surface,
    method: MethodSelector,
    route_template: RouteTemplate,
    step_context: StepContext,
    principal: Principal,
    channel: Channel,
    #[serde(default)]
    field_path: Option<FieldPattern>,
}

impl TryFrom<SelectorWire> for Selector {
    type Error = InvalidValue;

    fn try_from(wire: SelectorWire) -> Result<Self, Self::Error> {
        let path = wire.field_path.as_ref().map(FieldPattern::as_str);
        let consistent = match wire.channel {
            Channel::Status | Channel::Any => path.is_none(),
            Channel::Header | Channel::Cookie => path.is_none_or(|p| !p.contains(['.', '['])),
            Channel::Body => true,
            Channel::SideEffect => path.is_some_and(|p| {
                let sink = p.split(['.', '[']).next().unwrap_or_default();
                matches!(sink, "store" | "outbound_http")
            }),
        };
        if !consistent {
            return Err(InvalidValue::Malformed {
                what: "selector",
                raw: format!("channel {:?} with field_path {path:?}", wire.channel),
            });
        }
        Ok(Self {
            surface: wire.surface,
            method: wire.method,
            route_template: wire.route_template,
            step_context: wire.step_context,
            principal: wire.principal,
            channel: wire.channel,
            field_path: wire.field_path,
        })
    }
}

impl JsonSchema for Selector {
    fn schema_name() -> Cow<'static, str> {
        "Selector".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        SelectorWire::json_schema(generator)
    }
}

impl Selector {
    /// The selector's identity, bound into a scope confirmation.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails, which a valid selector never does.
    pub fn selector_hash(&self) -> Result<SelectorHash, CanonicalError> {
        ContentHash::of(self).map(SelectorHash::from)
    }

    /// Whether the selector is a broad grant that needs its own acknowledgement: any method,
    /// any step, any principal, or any channel.
    #[must_use]
    pub fn is_broad(&self) -> bool {
        self.method == MethodSelector::Any
            || self.step_context == StepContext::Any {} && self.principal == Principal::Any {}
            || self.channel == Channel::Any
    }
}

/// What a may-change item does to the selected behavior.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Change {
    /// New behavior where there was none.
    Add,
    /// Different behavior.
    Modify,
    /// Behavior that goes away.
    Remove,
}

/// Where a scope item came from. Drawn as a chip on the card.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ItemProvenance {
    /// The user's own requirement.
    User,
    /// An existing ruling.
    Ruling,
    /// The model's interpretation of the order. Needs the human's confirmation like the rest.
    Model,
    /// An open choice the human answered.
    Open,
}

/// One acceptance obligation: a predicate the change must satisfy.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PredicateRef {
    /// The obligation.
    pub obligation_id: ObligationId,
    /// The predicate, evaluated by the core at the gate.
    pub predicate: Predicate,
    /// One line for the card.
    pub text: String,
}

/// Marks an example as a desired target. A pre-code example is never an observation.
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    /// A desired target.
    #[default]
    Target,
}

/// An example of the intended behavior, written before any code exists.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TargetExample {
    /// Always `target`.
    pub kind: TargetKind,
    /// The probe step the example describes.
    pub step_id: StepId,
    /// The field.
    pub path: ObservationPath,
    /// The base value the human expects today, if they gave one. JSON `null` is a value.
    #[serde(
        default,
        deserialize_with = "crate::types::present_scalar",
        skip_serializing_if = "Option::is_none"
    )]
    pub before: Option<Scalar>,
    /// The value the human wants after the change.
    pub after: Scalar,
}

impl TargetExample {
    /// The example's identity, bound into a scope confirmation.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn example_hash(&self) -> Result<ExampleHash, CanonicalError> {
        ContentHash::of(self).map(ExampleHash::from)
    }
}

/// One thing an order may change.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScopeItem {
    /// The item.
    pub id: ItemId,
    /// Where it applies.
    pub selector: Selector,
    /// What kind of change.
    pub change: Change,
    /// What the change must satisfy. Never empty: permission alone is never a pass.
    pub acceptance: NonEmpty<PredicateRef>,
    /// Where the item came from.
    pub provenance: ItemProvenance,
    /// The intended behavior, as a target.
    pub example: TargetExample,
    /// The plant machines it touches.
    pub machine_ids: NonEmpty<MachineId>,
}

/// Behavior an order must keep. Takes precedence over every may-change item.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MustKeep {
    /// The rule.
    pub id: MustKeepId,
    /// Where it applies.
    pub selector: Selector,
    /// The obligations that express it.
    pub obligations: NonEmpty<PredicateRef>,
    /// The rule in words, for the pin and the card.
    pub text: String,
    /// The ruling it restates, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ruling_id: Option<RulingId>,
    /// The plant machines it guards.
    pub machine_ids: NonEmpty<MachineId>,
}

/// One answer to an open choice.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChoiceOption {
    /// The answer key.
    pub key: OptionKey,
    /// The answer in words.
    pub label: String,
}

/// A question the scope cannot settle. A human must answer it before confirming.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenChoice {
    /// The choice.
    pub id: ChoiceId,
    /// Where the question pin goes.
    pub machine_id: MachineId,
    /// The question.
    pub question: String,
    /// The answers, one key each.
    pub options: NonEmpty<ChoiceOption>,
}

/// The body of a scope: may-change items, must-keep rules, and open choices.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScopeBody {
    /// What the order may change.
    pub may: Vec<ScopeItem>,
    /// What it must keep.
    pub must_keep: Vec<MustKeep>,
    /// Questions for the human.
    pub open: Vec<OpenChoice>,
}

/// The foreman's scope draft for one order. Every part of it is a model proposal.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScopeDraft {
    /// Always `acan-scope-draft/1`.
    pub version: ScopeDraftV1,
    /// The order.
    pub order: OrderRef,
    /// The revision this draft proposes.
    pub rev: ScopeRev,
    /// The commit the draft was written against.
    pub base_commit: CommitSha,
    /// The order's sentence.
    pub sentence: String,
    /// The machine the order is pinned on.
    pub machine_id: MachineId,
    /// The proposed body.
    pub draft: ModelProposal<ScopeBody>,
    /// When the foreman returned it.
    pub drafted_at: UnixMillis,
}

/// A scope revision as the human will confirm it. Its hash is the [`ScopeHash`].
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    /// Always `acan-scope/1`.
    pub version: ScopeV1,
    /// The order. A confirmed scope is per order; later orders inherit the resulting
    /// contract, never the permission.
    pub order: OrderRef,
    /// The revision.
    pub rev: ScopeRev,
    /// The commit the scope was confirmed against.
    pub base_commit: CommitSha,
    /// The body.
    pub body: ScopeBody,
}

impl Scope {
    /// The scope's identity, bound into its confirmation.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn scope_hash(&self) -> Result<ScopeHash, CanonicalError> {
        ContentHash::of(self).map(ScopeHash::from)
    }
}

/// The ruling version a confirmation was made against.
#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct RulingVersion {
    /// The ruling.
    pub ruling_id: RulingId,
    /// Its latest decision's ledger sequence number.
    pub event_seq: u64,
}

/// The human's answer to one open choice.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChoiceAnswer {
    /// The choice.
    pub choice_id: ChoiceId,
    /// The chosen answer.
    pub option: OptionKey,
}

/// A separate acknowledgement of one broad may-change item (see [`Selector::is_broad`]).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BroadGrantAck {
    /// The broad item.
    pub item_id: ItemId,
    /// Its selector, so widening it voids the acknowledgement.
    pub selector_hash: SelectorHash,
    /// Who acknowledged it.
    pub by: HumanRef,
    /// When.
    pub at: UnixMillis,
}

/// The repair budget of one order, fixed before the first run.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RepairBudget {
    /// Repair rounds per episode. Two in v1.
    pub repair_rounds: u32,
    /// Wall time cap for the whole order.
    pub wall_time_ms: u64,
    /// Model cost cap for the whole order, in millionths of a US dollar.
    pub model_cost_micros: u64,
    /// Cap on new discrepancy episodes the order may open, so new items cannot dodge the budget.
    pub max_episodes: u32,
}

/// One human confirmation of one scope revision. Any widening voids it.
///
/// It binds the scope revision and hash, the base commit, the ruling versions, every selector
/// and example hash, the answers to open choices, the broad-grant acknowledgements, and the
/// repair budget.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScopeConfirmation {
    /// Always `acan-scope-confirmation/1`.
    pub version: ScopeConfirmationV1,
    /// The order.
    pub order: OrderRef,
    /// The confirmed revision.
    pub scope_rev: ScopeRev,
    /// The hash of the confirmed [`Scope`].
    pub scope_hash: ScopeHash,
    /// The commit the scope was confirmed against.
    pub base_commit: CommitSha,
    /// The ruling versions in force.
    pub ruling_versions: Vec<RulingVersion>,
    /// The hash of every selector in the scope, in scope order.
    pub selector_hashes: Vec<SelectorHash>,
    /// The hash of every example in the scope, in scope order.
    pub example_hashes: Vec<ExampleHash>,
    /// One answer per open choice.
    pub answers: Vec<ChoiceAnswer>,
    /// One acknowledgement per broad item.
    pub broad_grants: Vec<BroadGrantAck>,
    /// The repair budget.
    pub budget: RepairBudget,
    /// The confirming human. Never an agent or a model.
    pub by: HumanRef,
    /// When.
    pub at: UnixMillis,
}
