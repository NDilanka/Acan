//! Who authored a value: an observation, the core, a human, or a model.
//!
//! The four provenances are distinct types. A model may propose scope drafts, probe inputs,
//! predictions, predicates and tours, and nothing else: [`ModelProposal`] only wraps a
//! [`Proposable`] payload, and that trait is sealed. Observations, item classes, counts and masks
//! are not proposable, and no public constructor turns a proposal into one of them.
//!
//! ```compile_fail
//! # use acan_intent::{ModelProposal, Observation};
//! // An observation cannot even be wrapped as a model proposal.
//! fn launder(_: ModelProposal<Observation>) {}
//! ```
//!
//! ```compile_fail
//! # use acan_intent::{ItemClass, ModelProposal};
//! fn launder(_: ModelProposal<ItemClass>) {}
//! ```
//!
//! ```compile_fail
//! # use acan_intent::{ClassCounts, ModelProposal};
//! fn launder(_: ModelProposal<ClassCounts>) {}
//! ```
//!
//! ```compile_fail
//! # use acan_intent::{Mask, ModelProposal};
//! fn launder(_: ModelProposal<Mask>) {}
//! ```
//!
//! ```compile_fail
//! # use acan_intent::{ModelProposal, Observation, Prediction};
//! // No conversion from a proposal to an observation exists.
//! fn launder(p: ModelProposal<Prediction>) -> Observation { p.into() }
//! ```
//!
//! ```compile_fail
//! # use acan_intent::{ModelProposal, Prediction};
//! // A proposal is read by reference only; it has no by-value accessor.
//! fn launder(p: ModelProposal<Prediction>) -> Prediction { p.into_inner() }
//! ```
//!
//! ```compile_fail
//! # use acan_intent::ClassCounts;
//! // Core-derived counts cannot be built outside the core.
//! let counts = ClassCounts { preserved: 3 };
//! ```
//!
//! Reading a proposal is fine:
//!
//! ```
//! # use acan_intent::{ModelProposal, Prediction};
//! fn read(p: &ModelProposal<Prediction>) -> &Prediction { p.proposal() }
//! ```

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::hash::EvidenceHash;
use crate::types::{
    CommitSha, HumanRef, ModelRef, ObservationPath, RunId, Scalar, StepId, UnixMillis,
};
use crate::version::literal_tag;

literal_tag!(
    /// The `provenance` field of an [`Observed`] value.
    ObservedTag = "observed", "provenance"
);
literal_tag!(
    /// The `provenance` field of a [`CoreDerived`] value.
    CoreDerivedTag = "core_derived", "provenance"
);
literal_tag!(
    /// The `provenance` field of a [`HumanSpec`] value.
    HumanSpecTag = "human_spec", "provenance"
);
literal_tag!(
    /// The `provenance` field of a [`ModelProposal`].
    ModelProposalTag = "model_proposal", "provenance"
);

/// The provenance label the plant and the card draw next to a value.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    /// Recorded by the runner from one run, commit, step and field.
    Observed,
    /// Computed by the core from observations, masks, scope and expectations.
    CoreDerived,
    /// Written or confirmed by a human.
    HumanSpec,
    /// Proposed by a model. Never evidence.
    ModelProposal,
}

mod sealed {
    pub trait Sealed {}
}

/// A payload a model may propose. Sealed: implemented only for scope drafts, probe inputs,
/// predictions, predicate proposals, proposed values and tours.
pub trait Proposable: sealed::Sealed {}

macro_rules! proposable {
    ($($ty:path),+ $(,)?) => {$(
        impl sealed::Sealed for $ty {}
        impl Proposable for $ty {}
    )+};
}

proposable!(
    crate::ScopeBody,
    crate::ProbeProposal,
    crate::Prediction,
    crate::PredicateProposal,
    crate::ProposedValue,
    crate::Tour,
);

/// A value a model proposed, labelled with the model that proposed it.
///
/// It gains authority only through a human: [`HumanSpec::adopt`].
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, bound(deserialize = "T: Deserialize<'de>"))]
pub struct ModelProposal<T: Proposable> {
    provenance: ModelProposalTag,
    model: ModelRef,
    proposal: T,
}

impl<T: Proposable> ModelProposal<T> {
    /// Labels `proposal` as proposed by `model`.
    pub fn new(proposal: T, model: ModelRef) -> Self {
        Self {
            provenance: ModelProposalTag,
            model,
            proposal,
        }
    }

    /// The proposed payload.
    pub fn proposal(&self) -> &T {
        &self.proposal
    }

    /// The model that proposed it.
    pub fn model(&self) -> &ModelRef {
        &self.model
    }
}

/// A value a human wrote or confirmed.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HumanSpec<T> {
    provenance: HumanSpecTag,
    by: HumanRef,
    at: UnixMillis,
    value: T,
}

impl<T> HumanSpec<T> {
    /// A value the human `by` wrote at `at`.
    pub fn new(value: T, by: HumanRef, at: UnixMillis) -> Self {
        Self {
            provenance: HumanSpecTag,
            by,
            at,
            value,
        }
    }

    /// The human confirmation of a model proposal. The proposal's payload becomes the human's.
    pub fn adopt(proposal: ModelProposal<T>, by: HumanRef, at: UnixMillis) -> Self
    where
        T: Proposable,
    {
        Self::new(proposal.proposal, by, at)
    }

    /// The value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Who wrote or confirmed it.
    pub fn by(&self) -> &HumanRef {
        &self.by
    }

    /// When.
    pub fn at(&self) -> UnixMillis {
        self.at
    }
}

/// A value computed by the core. Only this crate's pure functions construct one.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoreDerived<T> {
    pub(crate) provenance: CoreDerivedTag,
    pub(crate) value: T,
}

impl<T> CoreDerived<T> {
    /// The value.
    pub fn value(&self) -> &T {
        &self.value
    }
}

/// One observed scalar with the address of the evidence it came from.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Observed {
    provenance: ObservedTag,
    value: Scalar,
    run_id: RunId,
    commit: CommitSha,
    step_id: StepId,
    path: ObservationPath,
    evidence_hash: EvidenceHash,
}

/// Where an observed value was recorded.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EvidenceAddress {
    /// The run.
    pub run_id: RunId,
    /// The commit the run tested.
    pub commit: CommitSha,
    /// The step that produced it.
    pub step_id: StepId,
    /// The field.
    pub path: ObservationPath,
    /// The raw evidence it was read from.
    pub evidence_hash: EvidenceHash,
}

impl Observed {
    /// An observed value at `address`.
    #[must_use]
    pub fn new(value: Scalar, address: EvidenceAddress) -> Self {
        Self {
            provenance: ObservedTag,
            value,
            run_id: address.run_id,
            commit: address.commit,
            step_id: address.step_id,
            path: address.path,
            evidence_hash: address.evidence_hash,
        }
    }

    /// The value.
    #[must_use]
    pub fn value(&self) -> &Scalar {
        &self.value
    }

    /// Where it was recorded.
    #[must_use]
    pub fn address(&self) -> EvidenceAddress {
        EvidenceAddress {
            run_id: self.run_id.clone(),
            commit: self.commit.clone(),
            step_id: self.step_id.clone(),
            path: self.path.clone(),
            evidence_hash: self.evidence_hash,
        }
    }
}
