//! Orders, witnesses, tasks, readbacks and questions.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::clash::Declaration;
use crate::event::{Event, Rejection};
use crate::ids::{
    AgentId, CommitId, GateRunId, MachineKey, MemberId, OrderId, OrderRev, QuestionId, ReadbackId,
    Seq, TaskId, WitnessId,
};
use crate::index::Locus;
use crate::lease::Claim;
use crate::world::{Ctx, World};

/// A check a human pins when posting an order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessSpec {
    /// The executable check.
    pub check: Check,
    /// Why the human pinned it, shown on the witness card.
    pub reason: String,
}

/// An executable check the gate runs against a candidate commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Check {
    /// A test command run in the task's worktree. Passes on exit code 0.
    Test {
        /// The command line.
        command: String,
    },
    /// An HTTP request against the running app.
    Http {
        /// The HTTP method.
        method: String,
        /// The request path.
        path: String,
        /// The expected status code.
        expect_status: u16,
        /// A substring the body must contain.
        expect_body: Option<String>,
    },
    /// A screenshot of a route, compared against the baseline by a vision model.
    Screenshot {
        /// The route to open.
        route: String,
        /// The region to compare. `None` compares the whole page.
        region: Option<Region>,
    },
}

/// A pixel rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Region {
    /// Left edge.
    pub x: u32,
    /// Top edge.
    pub y: u32,
    /// Width.
    pub w: u32,
    /// Height.
    pub h: u32,
}

/// A locked witness. Only humans add witnesses; agents can never edit or remove one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Witness {
    /// The witness id.
    pub id: WitnessId,
    /// What the witness checks.
    pub kind: WitnessKind,
    /// Why it was pinned.
    pub reason: String,
    /// The member who pinned it or answered the question that created it.
    pub pinned_by: MemberId,
}

/// What a witness checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WitnessKind {
    /// An executable check, run at the gate.
    Check {
        /// The check.
        check: Check,
    },
    /// A human's answer to a question. Judged at readback review, skipped at the gate.
    Ruling {
        /// The question it answers.
        question: QuestionId,
        /// The chosen answer text.
        text: String,
    },
}

impl Witness {
    /// Whether the gate runs this witness. Rulings are judged at readback, not at the gate.
    #[must_use]
    pub fn gated(&self) -> bool {
        matches!(self.kind, WitnessKind::Check { .. })
    }
}

/// An order: one sentence pinned on a machine, with its locked witnesses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    /// The order id.
    pub id: OrderId,
    /// The machine the order is pinned on.
    pub machine: MachineKey,
    /// What the human asked for.
    pub sentence: String,
    /// The current revision of the witness set.
    pub rev: OrderRev,
    /// The locked witness set at `rev`.
    pub witnesses: Vec<WitnessId>,
    /// The member who posted it.
    pub posted_by: MemberId,
    /// The stored part of the lifecycle. The rest is derived; see [`OrderPhase`].
    pub state: OrderState,
}

/// What the order itself stores. Everything after the split is derived from its tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderState {
    /// Waiting for the foreman's split.
    Splitting,
    /// Tasks exist.
    Split,
    /// Cancelled by a human. Terminal.
    Cancelled,
}

/// The lifecycle phase the UI shows, derived by [`World::phase`]. Never stored.
///
/// Rules, first match wins: `Cancelled` if cancelled; `Splitting` before the split;
/// `Landed` when every task is landed; `Asking` while a question is open; `Clashing` while
/// any task of the order is party to an unsettled clash; `Landing` when any task is queued
/// and none is open, claimed or gating; `Gating` when any task is gating; `Building` when
/// any task is building; otherwise `ReadingBack`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum OrderPhase {
    /// The foreman is splitting the order.
    Splitting,
    /// Workers are reading back and claiming.
    ReadingBack,
    /// A question is pinned and waits for a human.
    Asking {
        /// The open question.
        question: QuestionId,
    },
    /// A declaration clash waits for settlement.
    Clashing,
    /// At least one task is building.
    Building,
    /// At least one task's witnesses are running.
    Gating,
    /// Every remaining task passed its gate and waits in the land queue.
    Landing,
    /// Every task landed. Terminal.
    Landed,
    /// Cancelled. Terminal.
    Cancelled,
}

/// One unit of work the foreman split from an order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskSpec {
    /// A short title.
    pub title: String,
    /// The worker's brief.
    pub brief: String,
    /// The machines the task is expected to touch.
    pub scope: Vec<MachineKey>,
}

/// A task and its claim state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// The task id.
    pub id: TaskId,
    /// The order it belongs to.
    pub order: OrderId,
    /// What to do.
    pub spec: TaskSpec,
    /// Where the task is in its lifecycle.
    pub state: TaskState,
    /// The highest generation ever granted on this task. The next grant uses the one after it.
    pub last_generation: Option<crate::ids::Generation>,
}

/// A task's lifecycle.
///
/// ```text
/// Open --LeaseGranted--> Claimed{Declaring}
/// Claimed --LeaseEnded(any)--> Open           (the gate run stays in the log, inert)
/// Claimed{Declaring|Declared|Building} --Declared--> Claimed{Declared}
/// Claimed{Declared} --BuildCleared--> Claimed{Building}
/// Claimed{Building} --BuildReported--> Claimed{Gating}
/// Claimed{Gating} --GateClosed(Failed)--> Claimed{Building}
/// Claimed{Gating} --GateClosed(Passed)--> Queued     (lease ends as Delivered in the same batch)
/// Queued --Landed--> Landed
/// Queued --LandFailed--> Open
/// any but Landed --OrderCancelled--> Dropped
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum TaskState {
    /// Claimable by an agent whose readback matched the order's current revision.
    Open,
    /// Held under a live lease.
    Claimed {
        /// The lease and the readback it was granted on.
        claim: Claim,
        /// Progress under the claim.
        stage: Stage,
    },
    /// Passed its gate; waits in the land queue. No lease is needed from here.
    Queued {
        /// The commit that passed.
        head: CommitId,
        /// The gate run that passed.
        run: GateRunId,
        /// The log position where it was queued. The land queue is ordered by this.
        queued_at: Seq,
    },
    /// Merged. Terminal.
    Landed {
        /// The merge commit.
        commit: CommitId,
    },
    /// The order was cancelled. Terminal.
    Dropped,
}

/// Progress under a claim. A stage that needs a declaration holds it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum Stage {
    /// Claimed; no footprint declared yet.
    Declaring,
    /// Declared; waiting on a blocking clash or a serialized predecessor.
    Declared {
        /// The current declaration.
        decl: Declaration,
    },
    /// Cleared to write code.
    Building {
        /// The current declaration.
        decl: Declaration,
    },
    /// The agent reported a commit; the gate is running.
    Gating {
        /// The current declaration.
        decl: Declaration,
        /// The gate run.
        run: GateRunId,
        /// The commit under test.
        head: CommitId,
    },
}

/// A worker's restatement of its task before it writes code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Readback {
    /// The readback id.
    pub id: ReadbackId,
    /// The task read back.
    pub task: TaskId,
    /// The agent who wrote it.
    pub agent: AgentId,
    /// The agent's summary of what it will do.
    pub summary: String,
    /// Where the agent expects to work.
    pub touches: Vec<Locus>,
    /// The foreman's review. `None` until reviewed.
    pub verdict: Option<ReadbackVerdict>,
}

/// The foreman's judgement of a readback against one revision of the witness set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadbackVerdict {
    /// The order revision this verdict was judged against.
    pub rev: OrderRev,
    /// One judgement per witness of the order at `rev`.
    pub per_witness: BTreeMap<WitnessId, Judgement>,
}

/// The foreman's judgement of one witness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "judgement", rename_all = "snake_case")]
pub enum Judgement {
    /// The readback respects the witness.
    Match,
    /// The readback contradicts the witness.
    Disagrees {
        /// Why.
        reason: String,
    },
    /// The readback does not say enough. Not a match.
    Unclear {
        /// Why.
        reason: String,
    },
}

impl ReadbackVerdict {
    /// Whether this verdict lets its agent claim: judged at the order's current revision,
    /// and every current witness judged [`Judgement::Match`].
    #[must_use]
    pub fn matches(&self, order: &Order) -> bool {
        todo!()
    }

    /// The current witnesses that block a claim: missing, disagreeing or unclear.
    #[must_use]
    pub fn failing(&self, order: &Order) -> Vec<WitnessId> {
        todo!()
    }
}

/// A question the foreman drafts when readbacks disagree. The origin assigns its id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestionDraft {
    /// Where on the plant to pin it.
    pub at: Locus,
    /// The question.
    pub text: String,
    /// Exactly two answers.
    pub options: [String; 2],
}

/// A question pinned on the plant. At most one is open per order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    /// The question id.
    pub id: QuestionId,
    /// The order it blocks.
    pub order: OrderId,
    /// The order revision it was asked at.
    pub rev: OrderRev,
    /// Where it is pinned.
    pub at: Locus,
    /// The question.
    pub text: String,
    /// Exactly two answers.
    pub options: [String; 2],
    /// The chosen option index, once answered.
    pub answer: Option<usize>,
}

/// `PostOrder`: mints the order and its witnesses. The order starts `Splitting`.
pub(crate) fn decide_post_order(
    world: &World,
    ctx: &Ctx<'_>,
    member: &MemberId,
    machine: MachineKey,
    sentence: String,
    witnesses: Vec<WitnessSpec>,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}

/// `SplitReady`: rejects a result for an old revision or an order that is not `Splitting`.
pub(crate) fn decide_split_ready(
    world: &World,
    order: OrderId,
    rev: OrderRev,
    tasks: Vec<TaskSpec>,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}

/// `PostReadback`: an agent reads back an open task. Allowed while the task is `Open`.
pub(crate) fn decide_readback(
    world: &World,
    agent: &AgentId,
    task: TaskId,
    summary: String,
    touches: Vec<Locus>,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}

/// `ReadbacksReviewed`: records verdicts for the current revision and pins at most one
/// question. A draft is ignored while a question is already open on the order.
pub(crate) fn decide_reviewed(
    world: &World,
    order: OrderId,
    rev: OrderRev,
    verdicts: Vec<(ReadbackId, ReadbackVerdict)>,
    question: Option<QuestionDraft>,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}

/// `AnswerQuestion`: adds a ruling witness, bumps the order revision, and ends every live
/// lease on the order as `Superseded`, so no claim survives on a verdict for an old witness set.
pub(crate) fn decide_answer(
    world: &World,
    member: &MemberId,
    question: QuestionId,
    option: usize,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}

/// `CancelOrder`: any phase but `Landed`. Ends live leases as `Cancelled` and drops tasks.
pub(crate) fn decide_cancel(
    world: &World,
    member: &MemberId,
    order: OrderId,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}
