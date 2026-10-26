//! The world: the fold of one log, the origin that extends it, and replicas that follow it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::clash::{Clash, ClashKey, Settlement};
use crate::event::{Command, Rejection, ReplayError, Stamped};
use crate::gate::GateRun;
use crate::ids::{
    AgentId, ClashId, CodeRev, GateRunId, MemberId, OrderId, QuestionId, ReadbackId, Seq, TaskId,
    Timestamp, WitnessId, WorldId,
};
use crate::index::CodeIndex;
use crate::job::Job;
use crate::member::{AgentPrincipal, Member, SettleRight};
use crate::order::{Order, OrderPhase, Question, Readback, Task, Witness};

/// Which repository a world tracks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldRef {
    /// The world id.
    pub id: WorldId,
    /// The repository, as the host names it.
    pub repo: String,
}

/// What the host supplies to every `execute`: the time and the current code index.
#[derive(Debug, Clone, Copy)]
pub struct Ctx<'a> {
    /// The origin's clock.
    pub now: Timestamp,
    /// The latest scan. Detection reads it; replicas never need it.
    pub index: &'a CodeIndex,
}

/// Which loop step made a model call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    /// Splitting an order.
    Split,
    /// Reviewing readbacks.
    Review,
    /// Explaining a clash.
    Explain,
    /// Comparing a screenshot witness.
    Screenshot,
}

/// Token totals.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenCount {
    /// Prompt tokens.
    pub input: u64,
    /// Completion tokens.
    pub output: u64,
    /// Number of calls.
    pub calls: u32,
}

/// Model usage per model and step, for the usage panel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelUsage {
    /// Totals keyed by model name, then step. Nested rather than tuple-keyed so it serializes to JSON.
    pub totals: BTreeMap<String, BTreeMap<Step, TokenCount>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Counters {
    order: u32,
    task: u32,
    witness: u32,
    readback: u32,
    question: u32,
    clash: u32,
    gate: u32,
}

/// The fold of a world log. Read-only outside the crate; only applying events changes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
    pub(crate) world_ref: WorldRef,
    pub(crate) next_seq: Seq,
    // Advanced by apply from the ids it sees, so replay and the origin agree on the next id.
    ids: Counters,
    pub(crate) index_rev: Option<CodeRev>,
    pub(crate) members: BTreeMap<MemberId, Member>,
    pub(crate) agents: BTreeMap<AgentId, AgentPrincipal>,
    pub(crate) witnesses: BTreeMap<WitnessId, Witness>,
    pub(crate) orders: BTreeMap<OrderId, Order>,
    pub(crate) tasks: BTreeMap<TaskId, Task>,
    pub(crate) readbacks: BTreeMap<ReadbackId, Readback>,
    pub(crate) questions: BTreeMap<QuestionId, Question>,
    pub(crate) clashes: BTreeMap<ClashId, Clash>,
    pub(crate) clash_keys: BTreeMap<ClashKey, ClashId>,
    pub(crate) gates: BTreeMap<GateRunId, GateRun>,
    pub(crate) usage: ModelUsage,
}

impl World {
    /// The world this log belongs to.
    #[must_use]
    pub fn world_ref(&self) -> &WorldRef {
        &self.world_ref
    }

    /// The seq the next event will carry. A client resumes its stream from here.
    #[must_use]
    pub fn next_seq(&self) -> Seq {
        self.next_seq
    }

    /// The index revision of the last `IndexUpdated`, if any.
    #[must_use]
    pub fn index_rev(&self) -> Option<&CodeRev> {
        self.index_rev.as_ref()
    }

    /// A member by id.
    #[must_use]
    pub fn member(&self, id: &MemberId) -> Option<&Member> {
        self.members.get(id)
    }

    /// An agent principal by id.
    #[must_use]
    pub fn agent(&self, id: &AgentId) -> Option<&AgentPrincipal> {
        self.agents.get(id)
    }

    /// A witness by id.
    #[must_use]
    pub fn witness(&self, id: WitnessId) -> Option<&Witness> {
        self.witnesses.get(&id)
    }

    /// An order by id.
    #[must_use]
    pub fn order(&self, id: OrderId) -> Option<&Order> {
        self.orders.get(&id)
    }

    /// Every order, in id order.
    pub fn orders(&self) -> impl Iterator<Item = &Order> {
        self.orders.values()
    }

    /// A task by id.
    #[must_use]
    pub fn task(&self, id: TaskId) -> Option<&Task> {
        self.tasks.get(&id)
    }

    /// The tasks of an order, in id order.
    pub fn tasks_of(&self, order: OrderId) -> impl Iterator<Item = &Task> {
        self.tasks.values().filter(move |t| t.order == order)
    }

    /// A readback by id.
    #[must_use]
    pub fn readback(&self, id: ReadbackId) -> Option<&Readback> {
        self.readbacks.get(&id)
    }

    /// The readbacks of a task, in id order.
    pub fn readbacks_of(&self, task: TaskId) -> impl Iterator<Item = &Readback> {
        self.readbacks.values().filter(move |r| r.task == task)
    }

    /// The order's unanswered question, if any. At most one exists.
    #[must_use]
    pub fn open_question(&self, order: OrderId) -> Option<&Question> {
        self.questions
            .values()
            .find(|q| q.order == order && q.answer.is_none())
    }

    /// A clash by id.
    #[must_use]
    pub fn clash(&self, id: ClashId) -> Option<&Clash> {
        self.clashes.get(&id)
    }

    /// Every clash ever found, including settled and dissolved ones.
    pub fn clashes(&self) -> impl Iterator<Item = &Clash> {
        self.clashes.values()
    }

    /// The clashes a task is party to.
    pub fn clashes_for(&self, task: TaskId) -> impl Iterator<Item = &Clash> {
        self.clashes.values().filter(move |c| c.key.involves(task))
    }

    /// A gate run by id.
    #[must_use]
    pub fn gate(&self, id: GateRunId) -> Option<&GateRun> {
        self.gates.get(&id)
    }

    /// Model usage so far.
    #[must_use]
    pub fn usage(&self) -> &ModelUsage {
        &self.usage
    }

    /// The order's lifecycle phase, derived from its tasks, questions and clashes.
    /// See [`OrderPhase`] for the rules. `None` for an unknown order.
    #[must_use]
    pub fn phase(&self, order: OrderId) -> Option<OrderPhase> {
        todo!()
    }

    /// Queued tasks in merge order: by the seq at which each was queued.
    #[must_use]
    pub fn land_queue(&self) -> Vec<TaskId> {
        todo!()
    }

    /// The jobs the host should be running now. See [`Job`] for the contract.
    ///
    /// `Split` for each splitting order; `Review` for each split order without an open
    /// question whose open tasks have readbacks lacking a verdict at the current revision;
    /// `Explain` for each blocking clash without an explanation; `RunWitness` for each
    /// frozen witness of a current gate run without a result; `Land` for the front of the
    /// land queue only.
    #[must_use]
    pub fn jobs(&self) -> Vec<Job> {
        todo!()
    }

    /// What `member` sending `SettleClash { clash, choice }` would do. `None` when the member
    /// is unknown or the choice is not on the clash's menu.
    ///
    /// Owners settle. A member settles when the choice's yielding task is held by one of
    /// their own agents. Otherwise the member may only propose.
    #[must_use]
    pub fn settle_right(
        &self,
        member: &MemberId,
        clash: &Clash,
        choice: &Settlement,
    ) -> Option<SettleRight> {
        todo!()
    }

    fn opened(world_ref: WorldRef) -> Self {
        Self {
            world_ref,
            next_seq: Seq::ZERO,
            ids: Counters::default(),
            index_rev: None,
            members: BTreeMap::new(),
            agents: BTreeMap::new(),
            witnesses: BTreeMap::new(),
            orders: BTreeMap::new(),
            tasks: BTreeMap::new(),
            readbacks: BTreeMap::new(),
            questions: BTreeMap::new(),
            clashes: BTreeMap::new(),
            clash_keys: BTreeMap::new(),
            gates: BTreeMap::new(),
            usage: ModelUsage::default(),
        }
    }

    /// Folds a whole log. The first event must be `WorldOpened` at seq 0.
    fn from_log(log: impl IntoIterator<Item = Stamped>) -> Result<Self, ReplayError> {
        todo!()
    }

    /// The only writer of world state. Never fails, reads a clock, or runs detection;
    /// `seq` continuity is checked by the caller.
    fn apply(&mut self, stamped: &Stamped) {
        todo!()
    }
}

/// The single decider for a world. Only the origin appends events.
///
/// A solo desktop runs it in-process; a team runs it in the shared `acan` server. The host
/// holds it behind one lock, persists the returned events before acknowledging, then
/// broadcasts them.
#[derive(Debug, Clone)]
pub struct Origin {
    world: World,
}

impl Origin {
    /// Starts a new world. Returns the `WorldOpened` event (seq 0, founder as owner) to persist.
    #[must_use]
    pub fn open(world: WorldRef, founder: MemberId, now: Timestamp) -> (Self, Stamped) {
        todo!()
    }

    /// Rebuilds the origin from its stored log.
    ///
    /// # Errors
    /// Returns [`ReplayError`] if the log does not start with `WorldOpened` or has a gap.
    pub fn replay(log: impl IntoIterator<Item = Stamped>) -> Result<Self, ReplayError> {
        Ok(Self {
            world: World::from_log(log)?,
        })
    }

    /// Decides `cmd`, applies the resulting events, and returns them stamped.
    ///
    /// Order of work: expire due leases, decide the command, then clear any declared task
    /// whose blockers are gone. All of it is one batch: either the command is accepted and
    /// every event is returned, or it is rejected and nothing changes (not even expiry).
    /// An already-satisfied command returns an empty batch.
    ///
    /// # Errors
    /// Returns [`Rejection`] when the command is not allowed in the current state.
    pub fn execute(&mut self, ctx: &Ctx<'_>, cmd: Command) -> Result<Vec<Stamped>, Rejection> {
        todo!()
    }

    /// The current state.
    #[must_use]
    pub fn world(&self) -> &World {
        &self.world
    }
}

/// A read-only follower of a world log: the browser, a joined desktop, the hosted replay.
///
/// It has no `execute`; a joined desktop sends commands to the origin over HTTP.
#[derive(Debug, Clone)]
pub struct Replica {
    world: World,
}

impl Replica {
    /// Folds a log fetched from the origin. The first event must be `WorldOpened` at seq 0.
    ///
    /// # Errors
    /// Returns [`ReplayError`] if the log does not start with `WorldOpened` or has a gap.
    pub fn from_log(log: impl IntoIterator<Item = Stamped>) -> Result<Self, ReplayError> {
        Ok(Self {
            world: World::from_log(log)?,
        })
    }

    /// Applies one streamed event. An event already applied (seq below `next_seq`) is ignored.
    ///
    /// # Errors
    /// Returns [`ReplayError::SeqGap`] if events were missed; refetch from `next_seq`.
    pub fn ingest(&mut self, stamped: &Stamped) -> Result<(), ReplayError> {
        todo!()
    }

    /// The seq to resume the stream from.
    #[must_use]
    pub fn next_seq(&self) -> Seq {
        self.world.next_seq
    }

    /// The current state.
    #[must_use]
    pub fn world(&self) -> &World {
        &self.world
    }
}
