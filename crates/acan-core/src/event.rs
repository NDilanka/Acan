//! Commands in, events out: the persisted and streamed schema.
//!
//! [`Stamped`] is stored in the host's event table and sent on the WebSocket as the same
//! JSON. Renaming a variant or field breaks stored logs.

use serde::{Deserialize, Serialize};

use crate::clash::{ClashKey, Declaration, DeclarationInput, Explanation, OverlapKind, Settlement};
use crate::gate::{GateVerdict, WitnessOutcome};
use crate::ids::{
    AgentId, ClashId, CodeRev, CommitId, GateRunId, Generation, MachineKey, MemberId, OrderId,
    OrderRev, QuestionId, ReadbackId, Seq, TaskId, Timestamp, WitnessId,
};
use crate::index::Locus;
use crate::lease::{Fence, Lease, LeaseEnd};
use crate::member::{AgentPrincipal, Member, Role};
use crate::order::{
    Question, QuestionDraft, Readback, ReadbackVerdict, TaskSpec, Witness, WitnessSpec,
};
use crate::world::{Step, WorldRef};

/// A command with the principal the host authenticated.
///
/// Not deserializable: the host builds it from an authenticated session, so a request can
/// never choose its own principal, and `System` commands can never arrive over the wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// From the UI, as a member.
    Human {
        /// The authenticated member.
        member: MemberId,
        /// What they asked for.
        cmd: HumanCmd,
    },
    /// From an MCP tool, as an agent.
    Agent {
        /// The authenticated agent.
        agent: AgentId,
        /// What it asked for.
        cmd: AgentCmd,
    },
    /// From the origin's own job loop.
    System(SystemCmd),
}

/// What a member can ask for. This is the HTTP request body a desktop sends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum HumanCmd {
    /// Owner only. Adds or changes a member.
    AddMember {
        /// The member.
        member: MemberId,
        /// Their role.
        role: Role,
    },
    /// Registers an agent owned by the caller.
    RegisterAgent {
        /// The agent's id from the token store.
        agent: AgentId,
    },
    /// Posts an order with its locked witnesses.
    PostOrder {
        /// The machine to pin it on.
        machine: MachineKey,
        /// What to do.
        sentence: String,
        /// The checks that must hold.
        witnesses: Vec<WitnessSpec>,
    },
    /// Answers an open question with option 0 or 1.
    AnswerQuestion {
        /// The question.
        question: QuestionId,
        /// The chosen option index.
        option: usize,
    },
    /// Settles or proposes a choice from the clash's menu.
    SettleClash {
        /// The clash.
        clash: ClashId,
        /// The choice. Must be on [`crate::Clash::menu`].
        choice: Settlement,
    },
    /// Cancels an order that has not landed.
    CancelOrder {
        /// The order.
        order: OrderId,
    },
}

/// What an agent can ask for. These are the MCP tool arguments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum AgentCmd {
    /// Reads back an open task.
    PostReadback {
        /// The task.
        task: TaskId,
        /// What the agent will do.
        summary: String,
        /// Where it expects to work.
        touches: Vec<Locus>,
    },
    /// Claims an open task. Needs a matched readback at the current revision.
    Claim {
        /// The task.
        task: TaskId,
    },
    /// Extends the caller's lease.
    Renew {
        /// The caller's fence.
        fence: Fence,
    },
    /// Gives the task back.
    Release {
        /// The caller's fence.
        fence: Fence,
    },
    /// Declares or redeclares the footprint.
    Declare {
        /// The caller's fence.
        fence: Fence,
        /// What it will change.
        footprint: DeclarationInput,
    },
    /// Reports a built commit and starts the gate.
    ReportBuilt {
        /// The caller's fence.
        fence: Fence,
        /// The commit to test.
        head: CommitId,
    },
}

/// Results from the origin's job loop and timers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemCmd {
    /// The host rescanned; `Ctx::index` is the new index.
    IndexUpdated,
    /// The foreman split an order.
    SplitReady {
        /// The order.
        order: OrderId,
        /// The revision the job was derived at.
        rev: OrderRev,
        /// The tasks.
        tasks: Vec<TaskSpec>,
    },
    /// The foreman reviewed readbacks.
    ReadbacksReviewed {
        /// The order.
        order: OrderId,
        /// The revision the job was derived at.
        rev: OrderRev,
        /// One verdict per reviewed readback.
        verdicts: Vec<(ReadbackId, ReadbackVerdict)>,
        /// A question, when readbacks disagree.
        question: Option<QuestionDraft>,
    },
    /// The foreman explained a clash.
    ClashExplained {
        /// The clash.
        clash: ClashId,
        /// The explanation.
        explanation: Explanation,
    },
    /// A witness finished.
    WitnessResult {
        /// The run.
        run: GateRunId,
        /// The witness.
        witness: WitnessId,
        /// Its outcome.
        outcome: WitnessOutcome,
    },
    /// A merge finished.
    LandResult {
        /// The task.
        task: TaskId,
        /// The commit the job merged.
        head: CommitId,
        /// The merge commit, or why it failed.
        result: Result<CommitId, String>,
    },
    /// A model call finished, for the usage panel.
    ModelUsed {
        /// The model name.
        model: String,
        /// Which step called it.
        step: Step,
        /// Prompt tokens.
        input_tokens: u32,
        /// Completion tokens.
        output_tokens: u32,
    },
    /// A timer tick. Expires due leases.
    Tick,
}

/// Who caused an event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum Actor {
    /// A member.
    Member(MemberId),
    /// An agent.
    Agent(AgentId),
    /// The origin itself.
    System,
}

/// An event with its log position. The unit of storage, streaming and replay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stamped {
    /// Position in the world log.
    pub seq: Seq,
    /// When the origin accepted it.
    pub at: Timestamp,
    /// Who caused it.
    pub actor: Actor,
    /// What happened.
    pub event: Event,
}

/// A fact the origin accepted. Applying it never fails, reads a clock, or runs detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// The first event of every log.
    WorldOpened {
        world: WorldRef,
        founder: MemberId,
    },
    MemberAdded {
        member: Member,
    },
    AgentRegistered {
        agent: AgentPrincipal,
    },
    IndexUpdated {
        rev: CodeRev,
    },
    OrderPosted {
        order: OrderId,
        machine: MachineKey,
        sentence: String,
        witnesses: Vec<Witness>,
    },
    OrderSplit {
        order: OrderId,
        rev: OrderRev,
        tasks: Vec<(TaskId, TaskSpec)>,
    },
    ReadbackPosted {
        readback: Readback,
    },
    ReadbacksReviewed {
        order: OrderId,
        rev: OrderRev,
        verdicts: Vec<(ReadbackId, ReadbackVerdict)>,
    },
    QuestionPinned {
        question: Question,
    },
    QuestionAnswered {
        question: QuestionId,
        option: usize,
        witness: Witness,
    },
    OrderRevised {
        order: OrderId,
        rev: OrderRev,
        witnesses: Vec<WitnessId>,
    },
    LeaseGranted {
        task: TaskId,
        lease: Lease,
        readback: ReadbackId,
    },
    LeaseRenewed {
        task: TaskId,
        generation: Generation,
        expires_at: Timestamp,
    },
    LeaseEnded {
        task: TaskId,
        generation: Generation,
        why: LeaseEnd,
    },
    Declared {
        decl: Declaration,
    },
    ClashFound {
        clash: ClashId,
        key: ClashKey,
        kind: OverlapKind,
        found_at: CodeRev,
    },
    ClashDissolved {
        clash: ClashId,
    },
    ClashExplained {
        clash: ClashId,
        explanation: Explanation,
    },
    ClashProposed {
        clash: ClashId,
        choice: Settlement,
        by: MemberId,
    },
    ClashSettled {
        clash: ClashId,
        choice: Settlement,
        by: MemberId,
    },
    BuildCleared {
        task: TaskId,
    },
    BuildReported {
        task: TaskId,
        generation: Generation,
        head: CommitId,
        run: GateRunId,
        rev: OrderRev,
        witnesses: Vec<WitnessId>,
    },
    WitnessReported {
        run: GateRunId,
        witness: WitnessId,
        outcome: WitnessOutcome,
    },
    GateClosed {
        run: GateRunId,
        verdict: GateVerdict,
    },
    Landed {
        task: TaskId,
        commit: CommitId,
    },
    LandFailed {
        task: TaskId,
        reason: String,
    },
    OrderCancelled {
        order: OrderId,
    },
    ModelUsed {
        model: String,
        step: Step,
        input_tokens: u32,
        output_tokens: u32,
    },
}

/// Why the origin refused a command. A refused command appends nothing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, thiserror::Error)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum Rejection {
    /// The principal may not do this.
    #[error("forbidden: {why}")]
    Forbidden { why: &'static str },
    /// The referenced entity does not exist.
    #[error("unknown {what} {id}")]
    Unknown { what: &'static str, id: String },
    /// A job result or command was for an older order revision.
    #[error("stale order revision; current is {current:?}")]
    StaleRevision { current: OrderRev },
    /// A fenced write carried a generation that is not the task's live one.
    #[error("stale generation; current is {current:?}")]
    StaleGeneration { current: Option<Generation> },
    /// The caller's lease has expired.
    #[error("lease expired")]
    LeaseExpired,
    /// The caller's readback does not match the current witnesses.
    #[error("readback does not match witnesses {failing:?}")]
    NotMatched { failing: Vec<WitnessId> },
    /// Another agent holds the task.
    #[error("held by {holder} until {until:?}")]
    HeldBy { holder: AgentId, until: Timestamp },
    /// The order has an open question.
    #[error("order has an open question {question}")]
    OpenQuestion { question: QuestionId },
    /// The entity is in the wrong state for this command.
    #[error("{what} is {found}")]
    WrongState {
        what: &'static str,
        found: &'static str,
    },
    /// The command is malformed.
    #[error("invalid command: {why}")]
    Invalid { why: String },
}

/// Why a log could not be folded.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReplayError {
    /// The log does not start with `WorldOpened` at seq 0.
    #[error("log does not start with WorldOpened at seq 0")]
    NotOpened,
    /// An event arrived out of order. The caller refetches from `expected`.
    #[error("expected seq {expected:?}, got {got:?}")]
    SeqGap { expected: Seq, got: Seq },
}
