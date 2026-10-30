//! The final gate end to end, against the real acan-demo at its pinned commit and a real Docker.
//!
//! A candidate that changes nothing a probe reads lands on the protected branch. A candidate that
//! seeds a regression in the signup response is blocked, and its receipt names the must-keep rule
//! it breaks. A runtime that cannot isolate a run never installs anything. A ruling that is
//! waiting for a human survives a restart of the ruling log, and land refuses it until its answer
//! is in the log. The gate only reads the repository, so nothing here writes to `ACAN_DEMO_DIR`.

#[path = "../src/ledger.rs"]
#[allow(
    dead_code,
    reason = "the gate reads and appends the ruling log through the ledger module, and this test reaches only some of its functions"
)]
mod ledger;

#[path = "../src/observations.rs"]
#[allow(
    dead_code,
    reason = "the gate's observation store has callers outside this test"
)]
mod observations;

#[path = "../src/observe.rs"]
#[allow(
    dead_code,
    reason = "the gate's observe pass has callers outside this test"
)]
mod observe;

#[path = "../src/land.rs"]
#[allow(
    dead_code,
    reason = "the gate's final check and install have callers outside this test"
)]
mod land;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use acan_git::{InstallOutcome, Preparation, PrepareOutcome, ProtectedPaths, Repo};
use acan_intent::{
    AppContract, CommitSha, ContainerPolicy, CoreDerived, DiscrepancyEpisode, EpisodeId,
    EpisodeKey, FoldedLedger, ItemClass, ItemState, LedgerEvent, Outcome, ProbeCorpus, Scope,
    ScopeConfirmation, canonical_episode_selector, fold,
};
use acan_witness::RunnerConfig;
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::sync::Mutex;

/// The environment variable that names the acan-demo checkout. The tests only read it.
const DEMO_DIR_VAR: &str = "ACAN_DEMO_DIR";
/// The acan-demo commit the protected branch is pinned to.
const DEMO_COMMIT: &str = "f91962156841332d5753d869554839f7c4cdea79";
/// The protected branch of the pinned clone.
const PROTECTED_BRANCH: &str = "main";
/// The port the app listens on inside its own container.
const APP_PORT: u16 = 3000;
/// The order the scope confirms.
const ORDER: &str = "o-demo-signup";
/// The protected probe corpus, which every candidate is judged by and no candidate may change.
const PROTECTED_CORPUS: &str = ".acan/probes/core.json";
/// The must-keep rule that keeps the signup answer as it is.
const SIGNUP_RULE: &str = "k-signup-body";
/// The signup handler's answer, as the pinned demo has it.
const SIGNUP_RESPONSE: &str = "return { id: user.id, username: user.username };";
/// The seeded regression: the answer upper-cases the username, so the answer differs from base.
const REGRESSED_SIGNUP_RESPONSE: &str =
    "return { id: user.id, username: user.username.toUpperCase() };";
/// The obligation of the signup rule that the seeded regression breaks.
const SIGNUP_OBLIGATION: &str = "obl-signup-body";
/// The ruling lineage the human answer belongs to.
const RULING: &str = "r-2";
/// The probe hash the residue episode names. The episode's identity does not include it, so any
/// valid hash will do for this test.
const EPISODE_PROBE: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000000";

/// Serializes the Docker tests, so their containers do not compete for the same CPU budget and
/// their sequences stay inside the 30 s budget.
static DOCKER: Mutex<()> = Mutex::const_new(());

/// A clone of the acan-demo, reset to the pinned commit on the protected branch.
struct Demo {
    _dir: TempDir,
    root: PathBuf,
    repo: Repo,
}

impl Demo {
    async fn pinned() -> Self {
        let source = std::env::var_os(DEMO_DIR_VAR).unwrap_or_else(|| {
            panic!("set {DEMO_DIR_VAR} to a checkout of acan-demo at {DEMO_COMMIT}")
        });
        let dir = tempfile::tempdir().expect("a temporary directory for the clone");
        let root = dir.path().join("demo");
        git(
            dir.path(),
            &[
                "clone",
                "--quiet",
                "--no-hardlinks",
                &source.to_string_lossy(),
                &root.to_string_lossy(),
            ],
        );
        git(
            &root,
            &["checkout", "--quiet", "-B", PROTECTED_BRANCH, DEMO_COMMIT],
        );
        let repo = Repo::open_with_protected(&root, PROTECTED_BRANCH)
            .await
            .expect("the clone opens with its protected branch");
        Self {
            _dir: dir,
            root,
            repo,
        }
    }

    /// The protected head.
    fn head(&self) -> CommitSha {
        CommitSha::from_str(&git(&self.root, &["rev-parse", PROTECTED_BRANCH]))
            .expect("the protected head is a commit")
    }

    /// Commits `edit` on a new branch off the protected head, and returns the commit. The checkout
    /// is left on the protected branch, which install needs.
    fn candidate(&self, branch: &str, edit: impl FnOnce(&Path)) -> CommitSha {
        git(
            &self.root,
            &["checkout", "--quiet", "-b", branch, PROTECTED_BRANCH],
        );
        edit(&self.root);
        git(&self.root, &["add", "-A"]);
        git(&self.root, &["commit", "--quiet", "-m", branch]);
        let commit = CommitSha::from_str(&git(&self.root, &["rev-parse", "HEAD"]))
            .expect("the candidate is a commit");
        git(&self.root, &["checkout", "--quiet", PROTECTED_BRANCH]);
        commit
    }

    /// Commits `edit` on the protected branch itself, which moves its head, and returns the new
    /// head. The checkout must be on the protected branch, as [`Demo::candidate`] leaves it.
    fn advance(&self, edit: impl FnOnce(&Path)) -> CommitSha {
        edit(&self.root);
        git(&self.root, &["add", "-A"]);
        git(
            &self.root,
            &["commit", "--quiet", "-m", "advance the protected head"],
        );
        self.head()
    }
}

/// Everything one gate pass reads: the base preparation, the candidate preparation, the protected
/// inputs of the base, the scope and its confirmation, and the ruling log.
struct Gate {
    _dir: TempDir,
    base: Preparation,
    candidate: Preparation,
    snapshot: PathBuf,
    scratch: PathBuf,
    observations: PathBuf,
    app: AppContract,
    corpus: ProbeCorpus,
    policy: ContainerPolicy,
    runner: RunnerConfig,
    scope: Scope,
    confirmation: ScopeConfirmation,
    ledger: CoreDerived<FoldedLedger>,
    episodes: Vec<DiscrepancyEpisode>,
}

impl Gate {
    async fn new(demo: &Demo, candidate: &CommitSha, runner: RunnerConfig) -> Self {
        let dir = tempfile::tempdir().expect("a temporary directory for the gate");
        let base = prepare(&demo.repo, &demo.head()).await;
        let candidate = prepare(&demo.repo, candidate).await;
        let snapshot = dir.path().join("snapshot");
        observe::snapshot(&demo.repo, &base, &snapshot)
            .await
            .expect("the protected snapshot is written");
        let (app, corpus) = observe::read_protected(&snapshot).expect("the protected inputs read");
        let scope = scope(&demo.head());
        let confirmation = confirmation(&scope);
        let ledger = fold::fold(&[]).expect("the empty ruling log folds");
        let policy = observe::runner_policy();
        Self {
            base,
            candidate,
            snapshot,
            scratch: dir.path().join("scratch"),
            observations: dir.path().join("observations"),
            app,
            corpus,
            policy,
            runner,
            scope,
            confirmation,
            ledger,
            episodes: Vec::new(),
            _dir: dir,
        }
    }

    fn plan<'a>(&'a self, repo: &'a Repo) -> observe::Plan<'a> {
        observe::Plan {
            repo,
            base: &self.base,
            candidate: &self.candidate,
            snapshot: &self.snapshot,
            scratch: &self.scratch,
            observations: &self.observations,
            app: &self.app,
            corpus: &self.corpus,
            policy: &self.policy,
            runner: &self.runner,
        }
    }

    /// The same gate with the given discrepancy episodes, which the receipt reads.
    fn with_episodes(mut self, episodes: Vec<DiscrepancyEpisode>) -> Self {
        self.episodes = episodes;
        self
    }

    fn order(&self) -> observe::Order<'_> {
        observe::Order {
            scope: &self.scope,
            confirmation: &self.confirmation,
            ledger: &self.ledger,
            episodes: &self.episodes,
        }
    }
}

/// Prepares `commit` on the protected head, and fails the test unless it merges cleanly.
async fn prepare(repo: &Repo, commit: &CommitSha) -> Preparation {
    let outcome = repo
        .prepare(commit, &protected())
        .await
        .expect("the commit prepares");
    let PrepareOutcome::Prepared(preparation) = outcome else {
        panic!("the commit conflicts with the protected head");
    };
    preparation
}

fn protected() -> ProtectedPaths {
    observe::protected_paths(&observe::runner_policy())
}

fn runner() -> RunnerConfig {
    RunnerConfig::new("docker", APP_PORT)
}

/// The scope the demo order confirms: one must-keep rule over the signup answer's body. The rule
/// reaches the signup probes at their first step and holds when the answer matches base.
fn scope(base: &CommitSha) -> Scope {
    serde_json::from_value(json!({
        "version": "acan-scope/1",
        "order": ORDER,
        "rev": 1,
        "base_commit": base,
        "body": {
            "may": [],
            "must_keep": [{
                "id": SIGNUP_RULE,
                "selector": {
                    "surface": "http",
                    "method": "POST",
                    "route_template": "/api/signup",
                    "step_context": { "kind": "step", "step_id": "s1" },
                    "principal": { "kind": "anonymous" },
                    "channel": "body"
                },
                "obligations": [{
                    "obligation_id": "obl-signup-body",
                    "predicate": {
                        "predicate": "equals_base",
                        "step_id": "s1",
                        "path": "body",
                        "anchor": { "kind": "fresh" }
                    },
                    "text": "The signup answer's body matches the base run"
                }],
                "text": "Signup answers with the same body as before the change",
                "machine_ids": ["auth", "store"]
            }],
            "open": []
        }
    }))
    .expect("the demo scope decodes")
}

/// The human confirmation of `scope`, binding its hashes and selectors.
fn confirmation(scope: &Scope) -> ScopeConfirmation {
    let selector_hashes: Vec<Value> = scope
        .body
        .must_keep
        .iter()
        .map(|rule| json!(rule.selector.selector_hash().expect("the selector hashes")))
        .collect();
    serde_json::from_value(json!({
        "version": "acan-scope-confirmation/1",
        "order": ORDER,
        "scope_rev": scope.rev,
        "scope_hash": scope.scope_hash().expect("the scope hashes"),
        "semantic_scope_hash": scope.semantic_scope_hash().expect("the scope hashes"),
        "base_commit": scope.base_commit,
        "ruling_versions": [],
        "selector_hashes": selector_hashes,
        "example_hashes": [],
        "answers": [],
        "broad_grants": [],
        "budget": {
            "repair_rounds": 2,
            "wall_time_ms": 600_000,
            "model_cost_micros": 0,
            "max_episodes": 4
        },
        "by": "owner",
        "at": 1_700_000_000_000_u64
    }))
    .expect("the demo confirmation decodes")
}

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn touch_notes(root: &Path) {
    fs::write(root.join("NOTES.md"), "A note the probes never read.\n")
        .expect("the note is written");
}

fn seed_regression(root: &Path) {
    let file = root.join("src").join("auth.ts");
    let source = fs::read_to_string(&file).expect("the demo has src/auth.ts");
    assert_eq!(
        source.matches(SIGNUP_RESPONSE).count(),
        1,
        "the signup response appears once in the pinned demo"
    );
    fs::write(
        &file,
        source.replace(SIGNUP_RESPONSE, REGRESSED_SIGNUP_RESPONSE),
    )
    .expect("the regression is written");
}

#[tokio::test]
async fn a_passing_order_lands_on_the_protected_branch() {
    let _docker = DOCKER.lock().await;
    let demo = Demo::pinned().await;
    let before = demo.head();
    let candidate = demo.candidate("pass", touch_notes);
    let gate = Gate::new(&demo, &candidate, runner()).await;

    let judged = observe::judge(&gate.plan(&demo.repo), &gate.order())
        .await
        .expect("the candidate is judged");
    assert!(
        land::blockers(&judged.receipt).is_empty(),
        "a candidate that changes no probed behavior passes: {:?}",
        land::blockers(&judged.receipt)
    );

    let current = observe::current(&gate.plan(&demo.repo), &gate.order())
        .await
        .expect("the current context computes");
    let clearance = land::final_gate(&judged.receipt, &current, &gate.candidate)
        .expect("the gate clears the passing candidate");
    let landed = land::install(&demo.repo, clearance)
        .await
        .expect("the protected branch moves to the candidate");

    assert_eq!(landed, candidate, "the merged commit is the candidate");
    assert_eq!(
        demo.head(),
        candidate,
        "the protected branch now holds the candidate"
    );
    assert_ne!(before, candidate);
}

#[tokio::test]
async fn a_seeded_regression_is_blocked_and_the_receipt_names_it() {
    let _docker = DOCKER.lock().await;
    let demo = Demo::pinned().await;
    let before = demo.head();
    let candidate = demo.candidate("regress", seed_regression);
    let gate = Gate::new(&demo, &candidate, runner()).await;

    let judged = observe::judge(&gate.plan(&demo.repo), &gate.order())
        .await
        .expect("the regressed candidate is judged");
    let item = judged
        .receipt
        .items
        .iter()
        .find(|item| item.item_id.as_str() == SIGNUP_RULE)
        .unwrap_or_else(|| {
            panic!(
                "the receipt names the signup rule: {:?}",
                judged.receipt.items
            )
        });
    assert_eq!(item.class, ItemClass::ObligationFailed);
    assert!(
        matches!(item.evaluation.outcome, Outcome::Changed { .. }),
        "the answer differs from base: {:?}",
        item.evaluation.outcome
    );

    let current = observe::current(&gate.plan(&demo.repo), &gate.order())
        .await
        .expect("the current context computes");
    let error = land::final_gate(&judged.receipt, &current, &gate.candidate)
        .expect_err("the regressed candidate is blocked");
    let land::LandError::Blocked(blockers) = error else {
        panic!("the regression is refused by the receipt, not by context: {error:?}");
    };
    assert!(
        blockers.iter().any(|blocker| blocker
            .item
            .as_ref()
            .is_some_and(|id| id.as_str() == SIGNUP_RULE)),
        "the blockers name the signup rule: {blockers:?}"
    );
    assert_eq!(demo.head(), before, "nothing was installed");
}

#[tokio::test]
async fn a_runtime_that_cannot_isolate_a_run_never_installs() {
    let _docker = DOCKER.lock().await;
    let demo = Demo::pinned().await;
    let before = demo.head();
    let candidate = demo.candidate("unisolated", touch_notes);
    let runner = RunnerConfig::new("acan-no-such-container-runtime", APP_PORT);
    let gate = Gate::new(&demo, &candidate, runner).await;

    let judged = observe::judge(&gate.plan(&demo.repo), &gate.order())
        .await
        .expect("the unisolated candidate is judged as unknown, not as a pass");
    assert!(
        !land::blockers(&judged.receipt).is_empty(),
        "a run that never started cannot pass: {:?}",
        judged.receipt.items
    );
    let current = observe::current(&gate.plan(&demo.repo), &gate.order())
        .await
        .expect("the current context computes");
    assert!(
        land::final_gate(&judged.receipt, &current, &gate.candidate).is_err(),
        "the gate refuses an unisolated candidate"
    );
    assert_eq!(demo.head(), before, "nothing was installed");
}

#[tokio::test]
async fn a_candidate_cannot_change_the_protected_inputs() {
    let demo = Demo::pinned().await;
    let candidate = demo.candidate("edit-corpus", |root| {
        let file = root.join(PROTECTED_CORPUS);
        let text = fs::read_to_string(&file).expect("the corpus is read");
        fs::write(&file, format!("{text}\n")).expect("the edit is written");
    });

    let prepared = prepare(&demo.repo, &candidate).await;
    let discarded: Vec<&str> = prepared
        .discarded()
        .iter()
        .map(|path| path.as_str())
        .collect();
    assert!(
        discarded.contains(&PROTECTED_CORPUS),
        "the edit to the corpus is discarded: {discarded:?}"
    );
    let head_corpus = git(
        &demo.root,
        &["show", &format!("{}:{PROTECTED_CORPUS}", demo.head())],
    );
    let merged_corpus = git(
        &demo.root,
        &[
            "show",
            &format!("{}:{PROTECTED_CORPUS}", prepared.merged_commit()),
        ],
    );
    assert_eq!(
        merged_corpus, head_corpus,
        "the merged commit holds the protected head's corpus"
    );
}

#[tokio::test]
async fn a_head_move_forces_a_fresh_preparation_and_run() {
    let _docker = DOCKER.lock().await;
    let demo = Demo::pinned().await;
    let candidate = demo.candidate("moved-head", touch_notes);
    let gate = Gate::new(&demo, &candidate, runner()).await;
    let judged = observe::judge(&gate.plan(&demo.repo), &gate.order())
        .await
        .expect("the candidate is judged");
    assert!(
        land::blockers(&judged.receipt).is_empty(),
        "the candidate passes before the head moves: {:?}",
        land::blockers(&judged.receipt)
    );
    let current = observe::current(&gate.plan(&demo.repo), &gate.order())
        .await
        .expect("the current context computes");
    let stale = land::final_gate(&judged.receipt, &current, &gate.candidate)
        .expect("the gate clears the candidate before the head moves");

    let moved = demo.advance(|root| {
        fs::write(root.join("MOVED.md"), "The protected head moved.\n")
            .expect("the note is written");
    });
    let refused = land::install(&demo.repo, stale)
        .await
        .expect_err("a clearance for the old head does not install over the new one");
    assert!(
        matches!(
            &refused,
            land::LandError::NotInstalled(InstallOutcome::HeadMoved { current: head })
                if *head == moved
        ),
        "the protected branch moved, so nothing installs: {refused:?}"
    );
    assert_eq!(demo.head(), moved, "the moved head is untouched");

    let fresh = Gate::new(&demo, &candidate, runner()).await;
    let fresh_current = observe::current(&fresh.plan(&demo.repo), &fresh.order())
        .await
        .expect("the fresh context computes");
    let not_cleared = land::final_gate(&judged.receipt, &fresh_current, &fresh.candidate)
        .expect_err("the run against the old head does not clear the new head");
    assert!(
        matches!(not_cleared, land::LandError::ContextChanged),
        "the old receipt names the old head: {not_cleared:?}"
    );

    let rerun = observe::judge(&fresh.plan(&demo.repo), &fresh.order())
        .await
        .expect("the candidate is run again on the moved head");
    assert_eq!(
        rerun.context.protected_head, moved,
        "the new run is tested on the moved head"
    );
    assert!(
        land::blockers(&rerun.receipt).is_empty(),
        "the fresh run passes: {:?}",
        land::blockers(&rerun.receipt)
    );
    let cleared = land::final_gate(&rerun.receipt, &fresh_current, &fresh.candidate)
        .expect("the fresh run clears the gate");
    let landed = land::install(&demo.repo, cleared)
        .await
        .expect("the fresh candidate installs");
    assert_eq!(
        landed,
        *fresh.candidate.merged_commit(),
        "the tested merged commit is what lands"
    );
    assert_eq!(demo.head(), landed, "the protected branch holds it");
}

#[tokio::test]
async fn a_missing_protected_path_is_masked_and_the_run_is_judged() {
    let _docker = DOCKER.lock().await;
    let demo = Demo::pinned().await;
    demo.advance(|root| {
        fs::remove_dir_all(root.join("test")).expect("the protected test directory is removed");
    });
    let candidate = demo.candidate("no-protected-test", touch_notes);
    let gate = Gate::new(&demo, &candidate, runner()).await;
    let judged = observe::judge(&gate.plan(&demo.repo), &gate.order())
        .await
        .expect("the candidate is judged");
    assert!(
        land::blockers(&judged.receipt).is_empty(),
        "a missing protected path is masked by an empty mount, so the run is judged on its behavior: {:?}",
        land::blockers(&judged.receipt)
    );
    let item = judged
        .receipt
        .items
        .iter()
        .find(|item| item.item_id.as_str() == SIGNUP_RULE)
        .expect("the receipt names the signup rule");
    assert!(
        item.sequence.iter().all(|step| step.status.is_some()),
        "every role answered, so no run failed to start: {:?}",
        item.sequence
    );
}

/// The episode the seeded regression opens, as the repair loop leaves it once its budget runs
/// out: in residue, waiting for a human ruling. The receipt keys the signup item by it.
fn residue_episode(scope: &Scope) -> DiscrepancyEpisode {
    let selector = canonical_episode_selector(&scope.body.must_keep[0].selector);
    let key: EpisodeKey = serde_json::from_value(json!({
        "order": ORDER,
        "selector": selector,
        "obligation_id": SIGNUP_OBLIGATION,
    }))
    .expect("the episode key decodes");
    let id = key.episode_id().expect("the episode key hashes");
    serde_json::from_value(json!({
        "version": "acan-episode/1",
        "id": id,
        "order": ORDER,
        "scope_lineage": [{
            "rev": scope.rev,
            "scope_hash": scope.scope_hash().expect("the scope hashes"),
        }],
        "probe_hash": EPISODE_PROBE,
        "step_id": "s1",
        "selector": selector,
        "obligation_id": SIGNUP_OBLIGATION,
        "evidence_revisions": [],
        "candidate_attempts": [],
        "bounce_count": 2,
        "state": "residue",
    }))
    .expect("the residue episode decodes")
}

/// The human's answer to `episode`: a note that records the ruling without enforcing it. A note
/// is never counted as a check, so the item stays a blocker until a run on the answered head
/// carries it.
fn answer(episode: &EpisodeId) -> LedgerEvent {
    serde_json::from_value(json!({
        "event": "note_recorded",
        "schema_version": "acan-ledger/1",
        "event_seq": 1,
        "actor": "owner",
        "at": 1_700_000_100_000_u64,
        "ruling_id": RULING,
        "order": ORDER,
        "episode_id": episode,
        "text": "The signup answer changes on purpose; a later decision will settle it",
        "reason": {
            "kind": "unexpressible",
            "detail": "the change is a product call, not a predicate",
        },
        "machine_ids": ["auth"],
    }))
    .expect("the answer decodes")
}

/// The gate's order with `ledger` as its folded ruling state, so the context it computes binds
/// that ledger's head.
fn order_on<'a>(gate: &'a Gate, ledger: &'a CoreDerived<FoldedLedger>) -> observe::Order<'a> {
    observe::Order {
        scope: &gate.scope,
        confirmation: &gate.confirmation,
        ledger,
        episodes: &gate.episodes,
    }
}

#[tokio::test]
async fn a_pending_ruling_survives_a_restart_and_is_answered_in_the_log() {
    let _docker = DOCKER.lock().await;
    let demo = Demo::pinned().await;
    let before = demo.head();
    let candidate = demo.candidate("residue", seed_regression);
    let episode = residue_episode(&scope(&before));
    let gate = Gate::new(&demo, &candidate, runner())
        .await
        .with_episodes(vec![episode.clone()]);
    let judged = observe::judge(&gate.plan(&demo.repo), &gate.order())
        .await
        .expect("the regressed candidate is judged");
    let item = judged
        .receipt
        .items
        .iter()
        .find(|item| item.item_id.as_str() == SIGNUP_RULE)
        .expect("the receipt names the signup rule");
    assert_eq!(
        item.state,
        ItemState::Residue,
        "the episode waits for a human ruling"
    );
    assert_eq!(
        item.episode_id.as_ref(),
        Some(&episode.id),
        "the item is keyed by the episode"
    );
    let waiting = item.item_id.clone();

    // The state dir does not exist yet. An append into it is refused, so the first open has to
    // create the directory before the first append can land.
    let state = tempfile::tempdir().expect("a temporary state root");
    let state_dir = state.path().join("state");
    let log = state_dir.join(ledger::RULINGS_LOG);
    let answer_event = answer(&episode.id);
    assert!(
        !state_dir.exists(),
        "the state dir is absent before the first open"
    );
    let empty = ledger::head(&log).expect("an absent log reads as empty");
    let refused = ledger::append(&log, &empty, &answer_event)
        .expect_err("an append into an absent state dir is refused");
    assert!(
        matches!(refused, ledger::LedgerError::Io { .. }),
        "the refusal is the missing directory: {refused:?}"
    );
    assert!(!state_dir.exists(), "a refused append creates nothing");

    // The first start creates the state dir and reads the log, then the host stops.
    let first = land::RulingLog::open(&state_dir).expect("the first start creates the state dir");
    assert!(state_dir.is_dir(), "the state dir exists before any append");
    assert!(
        first.answered().expect("the log reads").is_empty(),
        "nothing is answered yet"
    );
    drop(first);

    // A restart rebuilds the host from the same state dir. Nothing from the first start is kept.
    let restarted = land::RulingLog::open(&state_dir).expect("the restart opens the same log");
    let answered = restarted.answered().expect("the log reads back");
    assert_eq!(
        land::pending(&judged.receipt, &answered),
        vec![waiting.clone()],
        "the ruling is still pending after the restart"
    );
    let folded = restarted.folded().expect("the log folds");
    let current = observe::current(&gate.plan(&demo.repo), &order_on(&gate, &folded))
        .await
        .expect("the current context computes");
    let error = land::final_gate(&judged.receipt, &current, &gate.candidate)
        .expect_err("land refuses a pending ruling");
    let land::LandError::Blocked(blockers) = error else {
        panic!("the pending ruling is refused by the receipt: {error:?}");
    };
    assert!(
        blockers
            .iter()
            .any(|blocker| blocker.item.as_ref() == Some(&waiting)),
        "the blockers name the pending item: {blockers:?}"
    );
    assert_eq!(demo.head(), before, "nothing was installed");

    // The answer is appended after the open that created its directory.
    let answered_head = restarted
        .answer(&answer_event)
        .expect("the answer appends after the head it read");
    assert_eq!(
        answered_head.event_seq, 1,
        "the answer is the log's first event"
    );
    drop(restarted);

    // A second restart folds the answer, and nothing is pending.
    let again = land::RulingLog::open(&state_dir)
        .expect("the second restart opens the log with the answer");
    let answered = again.answered().expect("the log reads back");
    assert!(
        answered.contains(&episode.id),
        "the answer names the episode"
    );
    assert!(
        land::pending(&judged.receipt, &answered).is_empty(),
        "no ruling is pending after the answer"
    );
    let folded = again.folded().expect("the log folds");
    assert_eq!(
        folded.value().head,
        answered_head,
        "the folded head is the answer"
    );
    assert_eq!(
        folded.value().notes.len(),
        1,
        "the answer folds as one note"
    );

    // The answer moved the ledger head, so the receipt from before it names another context.
    // Only a run on the answered head can clear the gate.
    let current = observe::current(&gate.plan(&demo.repo), &order_on(&gate, &folded))
        .await
        .expect("the current context computes");
    let stale = land::final_gate(&judged.receipt, &current, &gate.candidate)
        .expect_err("the receipt from before the answer is refused");
    assert!(
        matches!(stale, land::LandError::ContextChanged),
        "the old receipt is for another ledger head: {stale:?}"
    );
    assert_eq!(demo.head(), before, "nothing was installed");
}
