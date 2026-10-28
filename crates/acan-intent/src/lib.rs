//! The contract of Acan's scope-first behavior gate.
//!
//! An order's scope says what an agent may change, what it must keep, and what the change must
//! do. Probes run the base twice and the candidate once on the prepared merged commit. The core
//! classifies every item into one [`ItemClass`], and the [`Receipt`] is a pure function of
//! observations, masks, the scope and the human rulings in `.acan/rulings.log`. The
//! [`PlantView`] is what the UI draws for one receipt.
//!
//! This crate holds the shared types and the pure functions over them. Values carry their
//! [`Provenance`] in their types; see [`provenance`] for what a model may and may not author.
//!
//! Every envelope serializes as `snake_case` JSON with a version tag, refuses unknown fields, and
//! fails to decode on an unknown version. Internally tagged enums spell their fieldless variants
//! `Name {}` rather than `Name`: serde ignores `deny_unknown_fields` on unit variants of an
//! internally tagged enum, so `{"kind": "any", "extra": 1}` would otherwise decode.

pub mod compare;
mod episode;
mod error;
mod expr;
pub mod fold;
mod gate;
mod hash;
mod ledger;
mod mcp;
mod observation;
pub mod predicate;
mod probe;
pub mod provenance;
pub mod receipt;
mod report;
mod scope;
mod types;
mod verdict;
mod version;
mod view;

pub use episode::{
    CandidateAttempt, Contest, ContestKind, ContestStatus, DiscrepancyEpisode, EpisodeKey,
    EpisodeState, EvidenceRevision, Predicted, Prediction, ProposedValue, ScopeRevision,
};
pub use error::Unsupported;
pub use expr::{
    AllowedCount, Boundary, CompareOp, EventFilter, GroupBy, Operand, Predicate, PredicateProposal,
    QuotaCounting, RejectedAttempts, RunSelector, SchemaType, StringFormat, ValueRef, ValueSchema,
    WindowAnchor, WindowMs, WindowPolicy,
};
pub use gate::{GateContext, LedgerHead, PreparedCommit};
pub use hash::{
    ArtifactHash, CanonicalError, ContentHash, EnvHash, EpisodeId, EvidenceHash, ExampleHash,
    FixtureHash, LedgerHash, ObservationHash, OverlayHash, PredicateHash, ProbeHash, ProbeSetHash,
    ReceiptHash, RunnerHash, SandboxPolicyHash, ScopeHash, SelectorHash, canonical_json,
};
pub use ledger::{
    Applicability, DecisionConfirmed, Expectation, ExpectationStatus, FoldedLedger, LedgerEvent,
    NotEnforcedReason, NoteRecorded, ProbeRetired, RulingNote, Supersedes,
};
pub use mcp::{
    ContestRequest, ContestResponse, Enforcement, GetReceiptRequest, GetReceiptResponse,
    GetRulingsRequest, GetRulingsResponse, GetScopeRequest, GetScopeResponse, McpError,
    McpErrorCode, PostReadbackRequest, PostReadbackResponse, RulingSummary,
};
pub use observation::{
    CanonicalObservation, Completeness, IncompleteReason, Mask, MaskSelector, NormalizationPlan,
    Observation, Projection, ProjectionKind, Relation, RequestRecord, TypedCapture,
};
pub use probe::{
    AdvanceTimeStep, AppContract, CaptureKind, CaptureOutcome, CaptureSource, CaptureStep,
    ClockControl, CompletionBarrier, CorpusProbe, Fixture, HttpStep, Isolation, MissingCapture,
    ProbeCorpus, ProbeOrigin, ProbeProposal, ProbeSpec, ReadyCheck, RedirectPolicy,
    RequestTemplate, RouteEntry, RoutePath, RouteSource, RouteStatus, Run, RunEnv, RunLimits,
    RunOutcome, RunRole, SideEffectStep, Step, StoreIsolation, Template,
};
pub use provenance::{
    CoreDerived, EvidenceAddress, HumanSpec, ModelProposal, Observed, Proposable, Provenance,
};
pub use report::{
    BehaviorCounts, ClassCounts, EvidenceRow, EvidenceSlot, GateState, ProbeCounts, Receipt,
    ReceiptItem, RouteCounts, RulingCheck, RulingStatus, SequenceStep, StepTint, UncoveredRoute,
};
pub use scope::{
    BroadGrantAck, Change, ChoiceAnswer, ChoiceOption, ItemProvenance, MustKeep, OpenChoice,
    PredicateRef, Principal, RepairBudget, RulingVersion, Scope, ScopeBody, ScopeConfirmation,
    ScopeDraft, ScopeItem, Selector, StepContext, TargetExample, TargetKind,
};
pub use types::{
    ActorName, AgentRef, CaptureName, CatalogVersion, Channel, ChoiceId, ClientName, CommitSha,
    ContestId, EvaluatorVersion, FieldPath, FieldPattern, FixtureId, HttpMethod, HumanRef,
    InvalidValue, ItemId, MachineId, MaskablePath, MethodSelector, ModelRef, MustKeepId, NonEmpty,
    NormalizationVersion, ObligationId, ObservationPath, OptionKey, OrderRef, PathPattern,
    RepoPath, RouteTemplate, RulingId, RunId, RuntimeId, Scalar, ScopeRev, SlotName, StepId,
    Surface, TaskRef, TemplateId, TrustedSink, UnixMillis,
};
pub use verdict::{
    Authorization, CheckResult, Comparison, Difference, Evaluation, ItemClass, ItemState,
    ObligationCheck, Outcome, UnknownReason,
};
pub use version::{
    AppContractV1, ContestV1, EpisodeV1, LedgerV1, McpV1, ObservationV1, PlantViewV1,
    ProbeCorpusV1, ReceiptV1, RunV1, RunnerContractV1, ScopeConfirmationV1, ScopeDraftV1, ScopeV1,
};
pub use view::{
    BindableValue, BoundItemView, Camera, Caption, Coverage, EvidenceRef, ExampleView, MarkRef,
    MustKeepView, OpenChoiceView, PlantCounts, PlantItem, PlantMachine, PlantOrder, PlantView,
    RulingTag, RulingTagState, ScopeItemView, ScopeView, Tour, TourStep, TourView, ValueType,
    ValueUnit, ZoomLevel,
};
