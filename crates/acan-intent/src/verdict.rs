//! Item classes, evaluations and differences: what the core decides about each check.
//!
//! Only [`ItemClass::Preserved`] and [`ItemClass::ObligationFulfilled`] count toward a pass.
//! Permission is never a pass, and unknown, error, unstable base and incomparable never pass.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::hash::{EpisodeId, ProbeHash};
use crate::observation::CanonicalObservation;
use crate::types::{ItemId, MustKeepId, ObligationId, ObservationPath, RulingId, Scalar, StepId};

/// The one class the core assigns to each receipt item.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ItemClass {
    /// No difference from base where none was allowed.
    Preserved,
    /// Every acceptance obligation of the item passed.
    ObligationFulfilled,
    /// An acceptance or must-keep obligation failed.
    ObligationFailed,
    /// A difference inside a may-change item that no obligation checks.
    AuthorizedButUnspecified,
    /// A difference no scope item allows.
    OutOfScope,
    /// An active ruling rejected the candidate.
    ViolatesRuling,
    /// The two base runs disagreed on an unmaskable field.
    UnstableBase,
    /// Base and candidate could not be compared.
    Incomparable,
    /// The run or the evaluation failed.
    Error,
}

impl ItemClass {
    /// Every class, in receipt order.
    pub const ALL: [Self; 9] = [
        Self::Preserved,
        Self::ObligationFulfilled,
        Self::ObligationFailed,
        Self::AuthorizedButUnspecified,
        Self::OutOfScope,
        Self::ViolatesRuling,
        Self::UnstableBase,
        Self::Incomparable,
        Self::Error,
    ];

    /// Whether an item of this class counts toward a pass.
    #[must_use]
    pub fn counts_toward_pass(self) -> bool {
        match self {
            Self::Preserved | Self::ObligationFulfilled => true,
            Self::ObligationFailed
            | Self::AuthorizedButUnspecified
            | Self::OutOfScope
            | Self::ViolatesRuling
            | Self::UnstableBase
            | Self::Incomparable
            | Self::Error => false,
        }
    }
}

/// Where an item stands in the repair loop.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ItemState {
    /// Checked, nothing pending.
    Open,
    /// Sent back to the agent with a reproducer.
    Bounced,
    /// Bounced, then the original obligation passed fresh.
    Fixed,
    /// Waiting for a human ruling.
    Residue,
    /// The repair budget ran out. Shown as stopped, never as passed.
    Stopped,
}

/// What permission covers a check. Rulings and must-keep take precedence over may-change.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Authorization {
    /// An active ruling governs the selector.
    Ruling {
        /// The ruling.
        ruling_id: RulingId,
    },
    /// A must-keep rule governs it.
    MustKeep {
        /// The rule.
        must_keep_id: MustKeepId,
    },
    /// A may-change item allows a difference here.
    MayChange {
        /// The item.
        item_id: ItemId,
    },
    /// Nothing allows a difference: base must be preserved.
    None,
}

/// Why a comparison or a check has no answer. Never a pass.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "reason", rename_all = "snake_case", deny_unknown_fields)]
pub enum UnknownReason {
    /// Base A and base B disagreed where masking is not allowed.
    UnstableBase,
    /// The two sides have different shapes or reached different steps.
    Incomparable {
        /// What differed.
        detail: String,
    },
    /// An observation was incomplete.
    Incomplete,
    /// A run failed.
    RunFailed,
    /// The evidence belongs to an older commit or scope.
    Stale,
    /// The step was skipped.
    Skipped,
    /// The evaluator failed.
    Error {
        /// What happened.
        detail: String,
    },
}

/// The behavioral outcome of a check against base.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Outcome {
    /// The candidate matched base after normalization and masks.
    Unchanged,
    /// The candidate differed.
    Changed {
        /// The differences.
        differences: Vec<Difference>,
    },
    /// No answer.
    Unknown {
        /// Why.
        reason: UnknownReason,
    },
}

/// The result of one obligation.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "result", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckResult {
    /// The predicate held.
    Pass,
    /// It did not.
    Fail {
        /// Why, in words the core wrote.
        detail: String,
    },
    /// It could not be evaluated. Never a pass.
    Unknown {
        /// Why.
        reason: UnknownReason,
    },
}

/// One obligation's result.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObligationCheck {
    /// The obligation.
    pub obligation_id: ObligationId,
    /// Its result.
    pub result: CheckResult,
}

/// Everything the core weighed to classify one item. Derived; never built outside this crate.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct Evaluation {
    /// What permission covers it.
    pub authorization: Authorization,
    /// What happened against base.
    pub outcome: Outcome,
    /// Every obligation that applies, with its result.
    pub obligations: Vec<ObligationCheck>,
}

/// One field where the candidate differed from base. Derived; never built outside this crate.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct Difference {
    /// The probe.
    pub probe_hash: ProbeHash,
    /// The step.
    pub step_id: StepId,
    /// The field.
    pub path: ObservationPath,
    /// Base's value, absent if base had no such field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<Scalar>,
    /// The candidate's value, absent if it had no such field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate: Option<Scalar>,
    /// The episode this difference belongs to, once the core has matched it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_id: Option<EpisodeId>,
}

/// The comparison of one probe's base and candidate runs. Derived; never built outside this
/// crate.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct Comparison {
    /// The probe.
    pub probe_hash: ProbeHash,
    /// Base A, normalized.
    pub base_a: Vec<CanonicalObservation>,
    /// Base B, normalized.
    pub base_b: Vec<CanonicalObservation>,
    /// The candidate, normalized.
    pub candidate: Vec<CanonicalObservation>,
    /// The overall outcome.
    pub outcome: Outcome,
}
