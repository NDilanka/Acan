//! The gate context: everything a candidate's verdict depends on, compared before it lands.
//!
//! Identity rules:
//! - `base_commit` is the protected head the order's scope was confirmed against.
//! - `protected_head` is the protected branch head when the candidate was prepared. Land is a
//!   compare-and-swap against it; a moved head forces a new preparation and a new run.
//! - `merged_commit` is the prepared merged commit that was tested. Only it may land.
//! - `overlay_hash` names the protected inputs (`.acan/`, probes, fixtures) restored from the
//!   protected head on top of the merged tree before every run. It is part of the checked
//!   context, so the final gate compares it.
//! - `ledger_head` is the final ruling log revision materialized before the commit was frozen.
//!
//! Every composite hash is the [`ContentHash`] of one manifest below, whose content and order
//! are frozen here: JCS fixes the encoding, the manifest fixes what is included. Changing any
//! byte of an included input, such as one maskable path of one probe's normalization plan,
//! changes the hash and voids the context.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::hash::{
    ArtifactHash, CanonicalError, ContentHash, EnvHash, FixtureHash, FixtureSetHash, LedgerHash,
    NormalizationHash, OverlayHash, ProbeHash, ProbeSetHash, RunnerHash, SandboxPolicyHash,
    ScopeHash,
};
use crate::probe::{AppContract, ProbeCorpus};
use crate::types::{
    CommitSha, EvaluatorVersion, FixtureId, RepoPath, RunnerBuild, RuntimeId, UnixMillis,
};

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

/// One effective probe: its spec's identity and its normalization plan's content.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ProbeSetEntry {
    /// The probe.
    pub probe_hash: ProbeHash,
    /// Its normalization plan.
    pub normalization_hash: NormalizationHash,
}

/// Every effective probe of the corpus, retired ones excluded, sorted by probe hash and then
/// plan hash, without duplicates. Its hash is the [`ProbeSetHash`].
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProbeSetManifest {
    /// The probes.
    pub probes: Vec<ProbeSetEntry>,
}

impl ProbeSetManifest {
    /// The manifest of `corpus` without the `retired` probes.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if a probe cannot be hashed.
    pub fn of(corpus: &ProbeCorpus, retired: &[ProbeHash]) -> Result<Self, CanonicalError> {
        let mut probes = Vec::with_capacity(corpus.probes.len());
        for probe in &corpus.probes {
            let probe_hash = probe.spec.probe_hash()?;
            if !retired.contains(&probe_hash) {
                probes.push(ProbeSetEntry {
                    probe_hash,
                    normalization_hash: probe.normalization.normalization_hash()?,
                });
            }
        }
        probes.sort_unstable();
        probes.dedup();
        Ok(Self { probes })
    }

    /// The probe set's identity.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn probe_set_hash(&self) -> Result<ProbeSetHash, CanonicalError> {
        ContentHash::of(self).map(ProbeSetHash::from)
    }
}

/// Every fixture of the corpus by id. Its hash is the [`FixtureSetHash`].
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FixtureSetManifest {
    /// Each fixture's hash.
    pub fixtures: BTreeMap<FixtureId, FixtureHash>,
}

impl FixtureSetManifest {
    /// The manifest of `corpus`'s fixtures.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if a fixture cannot be hashed.
    pub fn of(corpus: &ProbeCorpus) -> Result<Self, CanonicalError> {
        let fixtures = corpus
            .fixtures
            .iter()
            .map(|fixture| Ok((fixture.id.clone(), fixture.fixture_hash()?)))
            .collect::<Result<_, CanonicalError>>()?;
        Ok(Self { fixtures })
    }

    /// The fixture set's identity.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn fixture_set_hash(&self) -> Result<FixtureSetHash, CanonicalError> {
        ContentHash::of(self).map(FixtureSetHash::from)
    }
}

/// The runner build and the whole protected app contract. Its hash is the [`RunnerHash`].
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunnerManifest {
    /// The runner build.
    pub build: RunnerBuild,
    /// The app contract.
    pub app: AppContract,
}

impl RunnerManifest {
    /// The runner's identity.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn runner_hash(&self) -> Result<RunnerHash, CanonicalError> {
        ContentHash::of(self).map(RunnerHash::from)
    }
}

/// Every protected file restored from the protected head onto the merged tree, by path. Its
/// hash is the [`OverlayHash`].
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OverlayManifest {
    /// The content hash of each file.
    pub files: BTreeMap<RepoPath, ArtifactHash>,
}

impl OverlayManifest {
    /// The overlay's identity.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn overlay_hash(&self) -> Result<OverlayHash, CanonicalError> {
        ContentHash::of(self).map(OverlayHash::from)
    }
}

/// The controlled environment of every run. Its hash is the [`EnvHash`].
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnvManifest {
    /// The variables the runner sets, the app contract's `env` included.
    pub vars: BTreeMap<String, String>,
    /// Where the controlled clock starts.
    pub clock_start: UnixMillis,
    /// The seed the runner hands the app.
    pub seed: u64,
}

impl EnvManifest {
    /// The environment's identity.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn env_hash(&self) -> Result<EnvHash, CanonicalError> {
        ContentHash::of(self).map(EnvHash::from)
    }
}

/// What a run's container may reach.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SandboxNetwork {
    /// Only the runner, on loopback. The outbound stub sink is the runner's.
    LoopbackOnly,
}

/// The container policy every run uses. Its hash is the [`SandboxPolicyHash`].
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SandboxPolicy {
    /// Network reach.
    pub network: SandboxNetwork,
    /// Memory cap.
    pub memory_bytes: u64,
    /// CPU cap, in thousandths of a core.
    pub cpu_millis: u32,
    /// Process cap.
    pub max_pids: u32,
    /// The worktree paths the app may write; everything else is read-only.
    pub writable: Vec<RepoPath>,
}

impl SandboxPolicy {
    /// The policy's identity.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn sandbox_policy_hash(&self) -> Result<SandboxPolicyHash, CanonicalError> {
        ContentHash::of(self).map(SandboxPolicyHash::from)
    }
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
    /// The protected inputs materialized onto the merged tree ([`OverlayManifest`]).
    pub overlay_hash: OverlayHash,
    /// The hashes of the tested artifact's own inputs, such as `bun.lock`, by path.
    pub effective_artifact_hashes: BTreeMap<RepoPath, ArtifactHash>,
    /// The ruling log revision in force.
    pub ledger_head: LedgerHead,
    /// The confirmed scope.
    pub scope_hash: ScopeHash,
    /// The effective probes with their normalization plans ([`ProbeSetManifest`]).
    pub probe_set_hash: ProbeSetHash,
    /// The fixtures ([`FixtureSetManifest`]).
    pub fixture_set_hash: FixtureSetHash,
    /// The runner build and app contract ([`RunnerManifest`]).
    pub runner_hash: RunnerHash,
    /// The evaluator that classified the runs.
    pub evaluator_version: EvaluatorVersion,
    /// The app runtime.
    pub runtime: RuntimeId,
    /// The controlled environment ([`EnvManifest`]).
    pub controlled_env_hash: EnvHash,
    /// The container policy the runs used ([`SandboxPolicy`]).
    pub sandbox_policy_hash: SandboxPolicyHash,
}

/// A prepared candidate: the order's base and the tested context.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreparedCommit {
    /// The commit the order's scope was confirmed against.
    pub base_commit: CommitSha,
    /// The tested context.
    pub context: GateContext,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::Relation;
    use crate::types::MaskablePath;

    fn corpus() -> ProbeCorpus {
        serde_json::from_str(include_str!("../fixtures/probe-corpus.json")).expect("corpus")
    }

    fn probe_set(corpus: &ProbeCorpus) -> ProbeSetHash {
        ProbeSetManifest::of(corpus, &[])
            .and_then(|manifest| manifest.probe_set_hash())
            .expect("hashing")
    }

    #[test]
    fn a_mask_path_change_under_the_same_version_changes_the_probe_set() {
        let original = corpus();
        let mut widened = original.clone();
        let plan = &mut widened.probes[0].normalization;
        let label = plan.version.clone();
        plan.maskable_paths
            .push("body.everything".parse::<MaskablePath>().expect("a path"));
        assert_eq!(widened.probes[0].normalization.version, label);
        assert_ne!(probe_set(&original), probe_set(&widened));
    }

    #[test]
    fn a_relation_change_changes_the_probe_set() {
        let original = corpus();
        let mut loosened = original.clone();
        let index = original
            .probes
            .iter()
            .position(|probe| !probe.normalization.relations.is_empty())
            .expect("a probe with relations");
        let relations: &mut Vec<Relation> = &mut loosened.probes[index].normalization.relations;
        relations.clear();
        assert_ne!(probe_set(&original), probe_set(&loosened));
    }

    #[test]
    fn corpus_order_does_not_change_the_probe_set_but_retirement_does() {
        let original = corpus();
        let mut reversed = original.clone();
        reversed.probes.reverse();
        assert_eq!(probe_set(&original), probe_set(&reversed));
        let retired = original.probes[0].spec.probe_hash().expect("hashing");
        let without = ProbeSetManifest::of(&original, &[retired])
            .and_then(|manifest| manifest.probe_set_hash())
            .expect("hashing");
        assert_ne!(probe_set(&original), without);
    }

    #[test]
    fn a_fixture_recipe_change_changes_the_fixture_set() {
        let original = corpus();
        let mut edited = original.clone();
        edited.fixtures[0].recipe.pop();
        let hash = |corpus: &ProbeCorpus| {
            FixtureSetManifest::of(corpus)
                .and_then(|manifest| manifest.fixture_set_hash())
                .expect("hashing")
        };
        assert_ne!(hash(&original), hash(&edited));
    }

    #[test]
    fn an_app_contract_change_changes_the_runner() {
        let app: AppContract =
            serde_json::from_str(include_str!("../fixtures/app-contract.json")).expect("app");
        let manifest = RunnerManifest {
            build: "acan-witness/0.1.0".parse().expect("a build"),
            app,
        };
        let mut edited = manifest.clone();
        edited.app.limits.max_requests += 1;
        assert_ne!(
            manifest.runner_hash().expect("hashing"),
            edited.runner_hash().expect("hashing")
        );
    }

    #[test]
    fn an_overlay_change_voids_the_context() {
        let receipt: crate::Receipt =
            serde_json::from_str(include_str!("../fixtures/receipt.json")).expect("receipt");
        let overlay = |probes: &[u8]| OverlayManifest {
            files: BTreeMap::from([(
                ".acan/probes/login.json".parse().expect("a path"),
                ContentHash::of_bytes(probes).into(),
            )]),
        };
        let tested = overlay(b"protected");
        let swapped = overlay(b"edited by the candidate");
        assert_ne!(
            tested.overlay_hash().expect("hashing"),
            swapped.overlay_hash().expect("hashing")
        );
        let mut current = receipt.prepared.context.clone();
        current.overlay_hash = swapped.overlay_hash().expect("hashing");
        assert_ne!(current, receipt.prepared.context);
    }

    #[test]
    fn env_and_sandbox_changes_change_their_hashes() {
        let env = EnvManifest {
            vars: BTreeMap::from([("NODE_ENV".to_owned(), "test".to_owned())]),
            clock_start: UnixMillis(0),
            seed: 7,
        };
        let reseeded = EnvManifest {
            seed: 8,
            ..env.clone()
        };
        assert_ne!(
            env.env_hash().expect("hashing"),
            reseeded.env_hash().expect("hashing")
        );
        let policy = SandboxPolicy {
            network: SandboxNetwork::LoopbackOnly,
            memory_bytes: 512 << 20,
            cpu_millis: 1000,
            max_pids: 64,
            writable: vec![".acan/tmp".parse().expect("a path")],
        };
        let roomier = SandboxPolicy {
            memory_bytes: 1 << 30,
            ..policy.clone()
        };
        assert_ne!(
            policy.sandbox_policy_hash().expect("hashing"),
            roomier.sandbox_policy_hash().expect("hashing")
        );
    }
}
