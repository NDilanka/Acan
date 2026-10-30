//! Observe and judge one candidate: every probe runs on the base twice and on the prepared
//! merged commit once, each in a fresh container, and the receipt is a pure function of what
//! those runs recorded.
//!
//! Each run starts a fresh app from the commit it names. Protected inputs (`.acan/` and `test/`)
//! are mounted read-only from a snapshot of the protected head, never from a worktree, so a
//! candidate cannot change the contract it is judged by. Observations are stored by hash. A run
//! that cannot start, times out or is cancelled keeps its outcome: it is never dropped and never
//! counted as a pass. A probe whose three runs cannot be compared stops the pass with an error,
//! and nothing is installed.
//!
//! The probe driver shares the container with the candidate's code. The runner's network is
//! `none`, so nothing outside the container reaches the app, but a candidate that controls its own
//! process could alter what the driver reports. The receipt does not claim the driver is isolated
//! from the candidate.

use std::collections::BTreeMap;
use std::fs;
use std::future::pending;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use acan_git::{Preparation, ProtectedPaths, Repo};
use acan_intent::admit::AdmitError;
use acan_intent::compare::{self, CompareError, Contrast};
use acan_intent::receipt::{self, ProbeResult, ReceiptError, ReceiptInputs};
use acan_intent::{
    AppContract, ArtifactHash, CanonicalError, Comparison, ContainerPolicy, ContentHash,
    CoreDerived, CorpusProbe, DiscrepancyEpisode, EnvHash, EnvManifest, EvaluatorVersion, Fixture,
    FixtureId, FixtureSetHash, FixtureSetManifest, FoldedLedger, GateContext, GateState,
    InvalidValue, Mask, ObservationHash, ObservedRun, PreparedCommit, ProbeCorpus, ProbeHash,
    ProbeSetHash, ProbeSetManifest, Receipt, RepoPath, RunRole, RunnerBuild, RunnerHash,
    RunnerManifest, RuntimeId, SandboxPolicyHash, Scope, ScopeConfirmation, UnixMillis,
};
use acan_witness::{RunnerConfig, SequenceInput, run_sequence};
use serde::de::DeserializeOwned;

use crate::land;
use crate::observations::{self, ObservationError};

/// The protected directory, relative to a snapshot's root.
const PROTECTED_DIR: &str = ".acan";
/// The app contract inside [`PROTECTED_DIR`].
const APP_FILE: &str = "app.json";
/// The probe corpus inside [`PROTECTED_DIR`].
const CORPUS_FILE: &str = "probes/core.json";
/// The lockfile the app was installed from, relative to a worktree's root.
const LOCKFILE: &str = "bun.lock";
/// The directory the candidate's artifact hashes are read from. Nothing runs in it.
const ARTIFACTS_DIR: &str = "artifacts";
/// The runner build name `acan_witness` stamps on every run. The receipt refuses runs whose runner
/// hash differs, so a drift here fails the gate instead of passing silently.
const RUNNER_BUILD: &str = "acan-witness/0.1.0";
/// The evaluator that classifies the runs: the receipt builder in acan-intent.
const EVALUATOR: &str = "acan-intent/0.1.0";
/// Why a contrast could not be observed or judged. Every variant stops the candidate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum ObserveError {
    /// A protected input in the snapshot is missing or does not decode.
    #[error("cannot read the protected input {}: {detail}", path.display())]
    Protected {
        /// The file.
        path: PathBuf,
        /// What went wrong.
        detail: String,
    },
    /// A file or directory the runner needs could not be created or read.
    #[error("cannot access {}: {source}", path.display())]
    Io {
        /// The path.
        path: PathBuf,
        /// The cause.
        #[source]
        source: io::Error,
    },
    /// Git could not write a run's tree or the snapshot.
    #[error(transparent)]
    Git(#[from] acan_git::Error),
    /// A run's record was not admitted. That is a runner bug, never a model's doing.
    #[error(transparent)]
    Admit(#[from] AdmitError),
    /// An observation could not be stored.
    #[error(transparent)]
    Store(#[from] ObservationError),
    /// A stored observation hashed differently from the one the run recorded.
    #[error("observation {recorded} was stored as {stored}")]
    HashMismatch {
        /// The hash the run recorded.
        recorded: ObservationHash,
        /// The hash the stored content has.
        stored: ObservationHash,
    },
    /// The probe names a fixture the corpus does not hold.
    #[error("probe fixture {fixture} is not in the corpus")]
    MissingFixture {
        /// The fixture id.
        fixture: FixtureId,
    },
    /// The three runs of a probe cannot be compared.
    #[error("probe {probe} cannot be compared: {source}")]
    Compare {
        /// The probe.
        probe: ProbeHash,
        /// Why the contrast does not check.
        #[source]
        source: CompareError,
    },
    /// The receipt could not be built from these inputs.
    #[error(transparent)]
    Receipt(#[from] ReceiptError),
    /// A value has no canonical JSON form.
    #[error(transparent)]
    Hash(#[from] CanonicalError),
    /// A built-in name is not a valid value. A build bug.
    #[error(transparent)]
    Name(#[from] InvalidValue),
}

/// One admitted run and the hashes of the observations it stored, in step order.
#[derive(Debug, Clone)]
pub(crate) struct RecordedRun {
    /// The admitted run, with its observations.
    pub(crate) observed: ObservedRun,
    /// The stored observations' hashes, in the run's step order.
    pub(crate) observations: Vec<ObservationHash>,
}

/// The three runs of one probe.
#[derive(Debug, Clone)]
pub(crate) struct ProbeRuns {
    /// The probe, with its normalization plan.
    pub(crate) probe: CorpusProbe,
    /// The first base run.
    pub(crate) base_a: RecordedRun,
    /// The second base run.
    pub(crate) base_b: RecordedRun,
    /// The candidate run.
    pub(crate) candidate: RecordedRun,
}

/// What one observe pass runs against. Every directory must be missing or empty: the runs write
/// into them, so a pass never reuses another pass's worktrees.
#[derive(Debug)]
pub(crate) struct Plan<'a> {
    /// The repository the protected head and the candidate live in.
    pub(crate) repo: &'a Repo,
    /// The preparation whose protected head is the base the runs start from. Its merged commit is
    /// the protected head itself.
    pub(crate) base: &'a Preparation,
    /// The preparation whose merged commit is the candidate under test.
    pub(crate) candidate: &'a Preparation,
    /// The protected-head snapshot the containers mount protected paths from. See [`snapshot`].
    pub(crate) snapshot: &'a Path,
    /// The directory the per-run worktrees are written under.
    pub(crate) scratch: &'a Path,
    /// The directory observations are stored under, by hash.
    pub(crate) observations: &'a Path,
    /// The protected app contract, read from the snapshot.
    pub(crate) app: &'a AppContract,
    /// The protected probe corpus, read from the snapshot.
    pub(crate) corpus: &'a ProbeCorpus,
    /// The container policy the runs apply.
    pub(crate) policy: &'a ContainerPolicy,
    /// How the host reaches docker and the app.
    pub(crate) runner: &'a RunnerConfig,
}

/// The order a candidate is judged against: its confirmed scope, the confirmation of that scope,
/// and the folded ruling log in force.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Order<'a> {
    /// The confirmed scope.
    pub(crate) scope: &'a Scope,
    /// The human confirmation of that scope.
    pub(crate) confirmation: &'a ScopeConfirmation,
    /// The ruling log, folded.
    pub(crate) ledger: &'a CoreDerived<FoldedLedger>,
    /// The discrepancy episodes the receipt tracks. An episode in `residue` or `contested`
    /// state makes its item wait for a human ruling.
    pub(crate) episodes: &'a [DiscrepancyEpisode],
}

/// One probe's contrast with the masks and comparison the core derives from it.
#[derive(Debug)]
pub(crate) struct Judgement<'a> {
    /// The three runs, borrowed from the observe pass.
    pub(crate) contrast: Contrast<'a>,
    /// The masks derived from the base runs.
    pub(crate) masks: Vec<CoreDerived<Mask>>,
    /// The comparison under those masks.
    pub(crate) comparison: CoreDerived<Comparison>,
}

/// The inputs of a context that do not come from one preparation: the protected contract, the
/// corpus and policy hashes, and the candidate's own artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Facts {
    artifacts: BTreeMap<RepoPath, ArtifactHash>,
    probe_set: ProbeSetHash,
    fixture_set: FixtureSetHash,
    runner: RunnerHash,
    runtime: RuntimeId,
    env: EnvHash,
    policy: SandboxPolicyHash,
    evaluator: EvaluatorVersion,
}

/// A judged candidate: the context it was tested under and the receipt the core built from it.
#[derive(Debug, Clone)]
pub(crate) struct Judged {
    /// The tested context. The receipt is void for any other.
    pub(crate) context: GateContext,
    /// The receipt, whose gate state is `Blocked` when it does not pass.
    pub(crate) receipt: Receipt,
}

/// The container policy every run applies: the contract default, which the witness and the
/// fixture `container-policy.json` also use. It pins the image by digest, gives no network, and
/// mounts `.acan` read-only from the snapshot.
pub(crate) fn runner_policy() -> ContainerPolicy {
    ContainerPolicy::default_run()
}

/// The paths the preparation of every candidate restores from the protected head: the policy's
/// protected mounts, which are exactly the paths the containers mount from the snapshot.
pub(crate) fn protected_paths(policy: &ContainerPolicy) -> ProtectedPaths {
    ProtectedPaths::new(policy.mounts.protected.iter().cloned())
}

/// Writes the protected paths of `base`'s protected head into `dir`, the snapshot the containers
/// mount protected paths from. `dir` is created when missing.
///
/// # Errors
///
/// Returns [`ObserveError`] when the directory cannot be created or git cannot write the paths.
pub(crate) async fn snapshot(
    repo: &Repo,
    base: &Preparation,
    dir: &Path,
) -> Result<(), ObserveError> {
    fs::create_dir_all(dir).map_err(|source| io_error(dir, source))?;
    repo.restore_protected(base, dir).await?;
    Ok(())
}

/// Reads the protected app contract and probe corpus from a snapshot.
///
/// # Errors
///
/// Returns [`ObserveError::Protected`] when either file is missing or does not decode.
pub(crate) fn read_protected(snapshot: &Path) -> Result<(AppContract, ProbeCorpus), ObserveError> {
    let root = snapshot.join(PROTECTED_DIR);
    let app = read_json(&root.join(APP_FILE))?;
    let corpus = read_json(&root.join(CORPUS_FILE))?;
    Ok((app, corpus))
}

/// The content hash of the lockfile in a materialized worktree.
///
/// # Errors
///
/// Returns [`ObserveError::Io`] when the lockfile cannot be read.
pub(crate) fn lockfile_hash(worktree: &Path) -> Result<ArtifactHash, ObserveError> {
    let path = worktree.join(LOCKFILE);
    let bytes = fs::read(&path).map_err(|source| io_error(&path, source))?;
    Ok(ArtifactHash::from(ContentHash::of_bytes(&bytes)))
}

/// Runs every probe of the corpus: base A, base B, and the candidate, each in its own container
/// and its own worktree, with the three of one probe at once.
///
/// # Errors
///
/// Returns [`ObserveError`] when a worktree cannot be written or a record does not admit. A run
/// that did not complete is not an error: its outcome is in the record.
pub(crate) async fn observe(plan: &Plan<'_>) -> Result<Vec<ProbeRuns>, ObserveError> {
    let corpus: &ProbeCorpus = plan.corpus;
    let mut runs = Vec::with_capacity(corpus.probes.len());
    for (index, probe) in corpus.probes.iter().enumerate() {
        let fixture = fixture_of(corpus, probe)?;
        let probe_dir = plan.scratch.join(format!("probe-{index}"));
        let base_a = stage(plan.repo, plan.base, &probe_dir.join("base-a")).await?;
        let base_b = stage(plan.repo, plan.base, &probe_dir.join("base-b")).await?;
        let candidate = stage(plan.repo, plan.candidate, &probe_dir.join("candidate")).await?;
        let input_a = sequence(plan, probe, fixture, &base_a, RunRole::BaseA);
        let input_b = sequence(plan, probe, fixture, &base_b, RunRole::BaseB);
        let input_c = sequence(plan, probe, fixture, &candidate, RunRole::Candidate);
        let (a, b, c) = tokio::join!(
            run_sequence(&input_a, pending::<()>()),
            run_sequence(&input_b, pending::<()>()),
            run_sequence(&input_c, pending::<()>()),
        );
        runs.push(ProbeRuns {
            probe: probe.clone(),
            base_a: record(plan.observations, a?)?,
            base_b: record(plan.observations, b?)?,
            candidate: record(plan.observations, c?)?,
        });
    }
    Ok(runs)
}

/// Derives the masks and the comparison of every probe's runs. The first probe whose runs cannot
/// be compared stops the pass.
///
/// # Errors
///
/// Returns [`ObserveError::Compare`] for the first contrast that does not check or compare.
pub(crate) fn compare_runs<'a>(
    runs: &'a [ProbeRuns],
    corpus: &'a ProbeCorpus,
) -> Result<Vec<Judgement<'a>>, ObserveError> {
    runs.iter()
        .map(|runs| -> Result<Judgement<'a>, ObserveError> {
            let fixture = fixture_of(corpus, &runs.probe)?;
            let contrast = Contrast {
                probe: &runs.probe,
                fixture,
                base_a: &runs.base_a.observed,
                base_b: &runs.base_b.observed,
                candidate: &runs.candidate.observed,
            };
            let probe = runs.probe.spec.probe_hash()?;
            let compared = |source: CompareError| ObserveError::Compare { probe, source };
            let masks = compare::derive_masks(&contrast).map_err(&compared)?;
            let comparison = compare::compare(&contrast, &masks).map_err(&compared)?;
            Ok(Judgement {
                contrast,
                masks,
                comparison,
            })
        })
        .collect()
}

/// The receipt inputs of each judgement, in the same order.
pub(crate) fn results<'j, 'a>(judgements: &'j [Judgement<'a>]) -> Vec<ProbeResult<'j>> {
    judgements
        .iter()
        .map(|judgement| ProbeResult {
            contrast: judgement.contrast,
            masks: &judgement.masks,
            comparison: &judgement.comparison,
        })
        .collect()
}

impl Facts {
    /// The facts of the protected inputs and the candidate's artifacts.
    ///
    /// # Errors
    ///
    /// Returns [`ObserveError`] when a manifest cannot be hashed or a built-in name is invalid.
    fn of(
        plan: &Plan<'_>,
        order: &Order<'_>,
        artifacts: BTreeMap<RepoPath, ArtifactHash>,
    ) -> Result<Self, ObserveError> {
        let retired = &order.ledger.value().retired_probes;
        let probe_set = ProbeSetManifest::of(plan.corpus, retired)?.probe_set_hash()?;
        let fixture_set = FixtureSetManifest::of(plan.corpus)?.fixture_set_hash()?;
        let runner = RunnerManifest {
            build: RunnerBuild::from_str(RUNNER_BUILD)?,
            app: plan.app.clone(),
        }
        .runner_hash()?;
        // The controlled environment is the policy's variables, with the app's port variable set
        // to the port the driver gives the app, as the witness does when it starts a run.
        let mut vars = plan.policy.environment.clone();
        vars.insert(plan.app.port_env.clone(), plan.runner.port.to_string());
        let env = EnvManifest {
            vars,
            clock_start: UnixMillis(0),
            seed: 0,
        }
        .env_hash()?;
        Ok(Self {
            artifacts,
            probe_set,
            fixture_set,
            runner,
            runtime: plan.app.runtime.clone(),
            env,
            policy: plan.policy.sandbox_policy_hash()?,
            evaluator: EvaluatorVersion::from_str(EVALUATOR)?,
        })
    }
}

impl Order<'_> {
    /// The tested context of `preparation` under these facts and this order's ruling log.
    fn context(&self, facts: &Facts, preparation: &Preparation) -> GateContext {
        preparation.bind(GateContext {
            protected_head: preparation.protected_head().clone(),
            merged_commit: preparation.merged_commit().clone(),
            overlay_hash: preparation.overlay_hash(),
            effective_artifact_hashes: facts.artifacts.clone(),
            ledger_head: self.ledger.value().head,
            scope_hash: self.confirmation.scope_hash,
            probe_set_hash: facts.probe_set,
            fixture_set_hash: facts.fixture_set,
            runner_hash: facts.runner,
            evaluator_version: facts.evaluator.clone(),
            runtime: facts.runtime.clone(),
            controlled_env_hash: facts.env,
            sandbox_policy_hash: facts.policy,
        })
    }
}

/// Observes every probe, judges the contrasts and builds the receipt of the candidate.
///
/// # Errors
///
/// Returns [`ObserveError`] when a run cannot be recorded or a contrast does not check. Such a
/// candidate has no receipt and nothing may install it.
pub(crate) async fn judge(plan: &Plan<'_>, order: &Order<'_>) -> Result<Judged, ObserveError> {
    let facts = Facts::of(plan, order, artifacts(plan).await?)?;
    let runs = observe(plan).await?;
    let judgements = compare_runs(&runs, plan.corpus)?;
    let results = results(&judgements);
    let context = order.context(&facts, plan.candidate);
    let mut receipt = build(order, plan, &results, &context, GateState::Prepared)?;
    if !land::blockers(&receipt).is_empty() {
        receipt = build(order, plan, &results, &context, GateState::Blocked)?;
    }
    Ok(Judged { context, receipt })
}

/// The context the candidate would be tested under right now, from the current ruling log and the
/// protected inputs of `plan`. The final gate compares it with the receipt's context.
///
/// # Errors
///
/// Returns [`ObserveError`] when the candidate's lockfile cannot be read or a hash fails.
pub(crate) async fn current(
    plan: &Plan<'_>,
    order: &Order<'_>,
) -> Result<GateContext, ObserveError> {
    let facts = Facts::of(plan, order, artifacts(plan).await?)?;
    Ok(order.context(&facts, plan.candidate))
}

/// The receipt of `results` under `context`, in `state`.
fn build(
    order: &Order<'_>,
    plan: &Plan<'_>,
    results: &[ProbeResult<'_>],
    context: &GateContext,
    state: GateState,
) -> Result<Receipt, ObserveError> {
    let prepared = PreparedCommit {
        base_commit: plan.base.protected_head().clone(),
        context: context.clone(),
    };
    let inputs = ReceiptInputs {
        scope: order.scope,
        confirmation: order.confirmation,
        prepared: &prepared,
        ledger: order.ledger,
        corpus: plan.corpus,
        results,
        anchors: &[],
        episodes: order.episodes,
        contests: &[],
        gate_state: state,
    };
    Ok(receipt::build_receipt(&inputs)?.value().clone())
}

/// The candidate's own artifact hashes: the lockfile of its merged tree.
async fn artifacts(plan: &Plan<'_>) -> Result<BTreeMap<RepoPath, ArtifactHash>, ObserveError> {
    let dir = plan.scratch.join(ARTIFACTS_DIR);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|source| io_error(&dir, source))?;
    }
    let lock = materialize(plan.repo, plan.candidate, &dir).await?;
    Ok(BTreeMap::from([(LOCKFILE.parse()?, lock)]))
}

/// One run's worktree, the commit it is at, and the hash of its lockfile.
struct Staged {
    worktree: PathBuf,
    commit: acan_intent::CommitSha,
    lockfile: ArtifactHash,
}

/// Writes `preparation`'s merged tree into `dir` and records what a run needs to know about it.
async fn stage(repo: &Repo, preparation: &Preparation, dir: &Path) -> Result<Staged, ObserveError> {
    let lockfile = materialize(repo, preparation, dir).await?;
    Ok(Staged {
        worktree: dir.to_path_buf(),
        commit: preparation.merged_commit().clone(),
        lockfile,
    })
}

/// Writes `preparation`'s merged tree into `dir` and returns the hash of its lockfile.
async fn materialize(
    repo: &Repo,
    preparation: &Preparation,
    dir: &Path,
) -> Result<ArtifactHash, ObserveError> {
    repo.materialize(preparation, dir).await?;
    lockfile_hash(dir)
}

/// Stores each observation of `run` under `dir` and keeps the run with their hashes.
fn record(dir: &Path, run: ObservedRun) -> Result<RecordedRun, ObserveError> {
    let mut observations = Vec::new();
    for (recorded, observation) in run.observations() {
        let stored = observations::put(dir, observation)?;
        if stored != *recorded {
            return Err(ObserveError::HashMismatch {
                recorded: *recorded,
                stored,
            });
        }
        observations.push(stored);
    }
    Ok(RecordedRun {
        observed: run,
        observations,
    })
}

/// One run's input: the shared protected inputs, plus the worktree and commit it runs on.
fn sequence<'a>(
    plan: &Plan<'a>,
    probe: &'a CorpusProbe,
    fixture: &'a Fixture,
    staged: &'a Staged,
    role: RunRole,
) -> SequenceInput<'a> {
    let corpus: &'a ProbeCorpus = plan.corpus;
    SequenceInput {
        probe,
        fixture,
        routes: &corpus.routes,
        app: plan.app,
        policy: plan.policy,
        commit: staged.commit.clone(),
        role,
        lockfile_hash: staged.lockfile,
        worktree: &staged.worktree,
        snapshot: plan.snapshot,
        config: plan.runner,
    }
}

fn fixture_of<'c>(
    corpus: &'c ProbeCorpus,
    probe: &CorpusProbe,
) -> Result<&'c Fixture, ObserveError> {
    corpus
        .fixtures
        .iter()
        .find(|fixture| fixture.id == probe.spec.fixture)
        .ok_or_else(|| ObserveError::MissingFixture {
            fixture: probe.spec.fixture.clone(),
        })
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, ObserveError> {
    let text = fs::read_to_string(path).map_err(|source| ObserveError::Protected {
        path: path.to_path_buf(),
        detail: source.to_string(),
    })?;
    serde_json::from_str(&text).map_err(|source| ObserveError::Protected {
        path: path.to_path_buf(),
        detail: source.to_string(),
    })
}

fn io_error(path: &Path, source: io::Error) -> ObserveError {
    ObserveError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use acan_intent::ContainerNetwork;

    use super::*;

    #[test]
    fn the_runner_policy_is_offline_and_pinned_by_digest() {
        let policy = runner_policy();
        assert!(
            matches!(policy.network, ContainerNetwork::None {}),
            "runs have no network: {:?}",
            policy.network
        );
        assert!(
            policy.image.digest.is_some(),
            "the image is pinned by digest"
        );
    }

    #[test]
    fn the_preparation_protects_every_path_the_containers_mount() {
        let policy = runner_policy();
        let protected = protected_paths(&policy);
        for path in &policy.mounts.protected {
            assert!(
                protected.iter().any(|kept| kept == path),
                "{path} is restored from the protected head"
            );
        }
        assert!(
            protected.covers(".acan/probes/core.json"),
            "the protected inputs are always protected"
        );
    }
}
