//! The gate context: everything a candidate's verdict depends on, compared before it lands.
//!
//! Identity rules:
//! - `base_commit` is the protected head the order's scope was confirmed against.
//! - `protected_head` is the protected branch head when the candidate was prepared. Land is a
//!   compare-and-swap against it; a moved head forces a new preparation and a new run.
//! - `merged_commit` is the prepared merged commit that was tested. Only it may land.
//! - `overlay_hash` names the protected inputs (`.acan/`, probes, fixtures) restored from the
//!   protected head on top of the merged tree before every run.
//! - `ledger_head` is the final ruling log revision materialized before the commit was frozen.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::hash::{
    ArtifactHash, EnvHash, FixtureHash, LedgerHash, OverlayHash, ProbeSetHash, RunnerHash,
    SandboxPolicyHash, ScopeHash,
};
use crate::types::{CommitSha, EvaluatorVersion, NormalizationVersion, RepoPath, RuntimeId};

/// One revision of the ruling log: the sequence number of its last event and the hash chain up
/// to and including it. Revision zero, the empty log, has no hash.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct LedgerHead {
    /// The last event's sequence number. Zero for the empty log.
    pub event_seq: u64,
    /// The chain hash through that event. Absent for the empty log.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<LedgerHash>,
}

/// The complete tested context of one candidate. Land compares every field.
///
/// Artifact-owned inputs (the candidate's own lockfile) are recorded in
/// `effective_artifact_hashes`, apart from the controlled environment in `controlled_env_hash`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GateContext {
    /// The protected branch head the candidate was prepared on.
    pub protected_head: CommitSha,
    /// The prepared merged commit that was tested.
    pub merged_commit: CommitSha,
    /// The hashes of the tested artifact's own inputs, such as `bun.lock`, by path.
    pub effective_artifact_hashes: BTreeMap<RepoPath, ArtifactHash>,
    /// The ruling log revision in force.
    pub ledger_head: LedgerHead,
    /// The confirmed scope.
    pub scope_hash: ScopeHash,
    /// The probe corpus.
    pub probe_set_hash: ProbeSetHash,
    /// The fixtures, hashed together in corpus order.
    pub fixture_hash: FixtureHash,
    /// The runner build and app contract.
    pub runner_hash: RunnerHash,
    /// The evaluator that classified the runs.
    pub evaluator_version: EvaluatorVersion,
    /// The normalization version in force.
    pub normalization_version: NormalizationVersion,
    /// The app runtime.
    pub runtime: RuntimeId,
    /// The controlled environment: variables, clock and seed.
    pub controlled_env_hash: EnvHash,
    /// The container policy the runs used.
    pub sandbox_policy_hash: SandboxPolicyHash,
}

/// A prepared candidate: the order's base, the protected overlay, and the tested context.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreparedCommit {
    /// The commit the order's scope was confirmed against.
    pub base_commit: CommitSha,
    /// The protected inputs materialized onto the merged tree.
    pub overlay_hash: OverlayHash,
    /// The tested context.
    pub context: GateContext,
}
