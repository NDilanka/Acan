//! The final gate end to end, against the real acan-demo at its pinned commit and a real Docker.
//!
//! A candidate that changes nothing a probe reads lands on the protected branch. A candidate that
//! seeds a regression in the signup response is blocked, and its receipt names the must-keep rule
//! it breaks. A runtime that cannot isolate a run never installs anything. The gate only reads the
//! repository, so nothing here writes to `ACAN_DEMO_DIR`.

#[path = "../src/ledger.rs"]
#[allow(
    dead_code,
    reason = "the gate reads the ruling log through the ledger module, which this test only reaches for its atomic replace"
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

use acan_git::{Preparation, PrepareOutcome, ProtectedPaths, Repo};
use acan_intent::{
    AppContract, CommitSha, ContainerPolicy, CoreDerived, FoldedLedger, ItemClass, Outcome,
    ProbeCorpus, Scope, ScopeConfirmation, fold,
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
/// The must-keep rule that keeps the signup answer as it is.
const SIGNUP_RULE: &str = "k-signup-body";
/// The signup handler's answer, as the pinned demo has it.
const SIGNUP_RESPONSE: &str = "return { id: user.id, username: user.username };";
/// The seeded regression: the answer upper-cases the username, so the answer differs from base.
const REGRESSED_SIGNUP_RESPONSE: &str =
    "return { id: user.id, username: user.username.toUpperCase() };";

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

    fn order(&self) -> observe::Order<'_> {
        observe::Order {
            scope: &self.scope,
            confirmation: &self.confirmation,
            ledger: &self.ledger,
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
