//! The world: the fold of one log, the origin that extends it, and replicas that follow it.

use std::collections::{BTreeMap, BTreeSet};
use std::mem;

use serde::{Deserialize, Serialize};

use crate::clash::{self, Clash, ClashKey, ClashState, Declaration, Settlement};
use crate::event::{
    Actor, AgentCmd, Command, Event, HumanCmd, Rejection, ReplayError, Stamped, SystemCmd,
};
use crate::gate::{self, GateRun, GateVerdict};
use crate::ids::{
    AgentId, ClashId, CodeRev, GateRunId, MemberId, OrderId, OrderRev, QuestionId, ReadbackId, Seq,
    TaskId, Timestamp, WitnessId, WorldId,
};
use crate::index::CodeIndex;
use crate::job::Job;
use crate::lease::{self, Claim};
use crate::member::{self, AgentPrincipal, Member, Role, SettleRight};
use crate::order::{
    self, Order, OrderPhase, OrderState, Question, Readback, Stage, Task, TaskState, Witness,
};

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
        let order = self.orders.get(&order)?;
        let phase = match order.state {
            OrderState::Cancelled => OrderPhase::Cancelled,
            OrderState::Splitting => OrderPhase::Splitting,
            OrderState::Split => self.split_phase(order.id),
        };
        Some(phase)
    }

    fn split_phase(&self, order: OrderId) -> OrderPhase {
        let tasks: Vec<&Task> = self.tasks_of(order).collect();
        let any = |pred: fn(&TaskState) -> bool| tasks.iter().any(|t| pred(&t.state));
        if tasks
            .iter()
            .all(|t| matches!(t.state, TaskState::Landed { .. }))
        {
            return OrderPhase::Landed;
        }
        if let Some(question) = self.open_question(order) {
            return OrderPhase::Asking {
                question: question.id,
            };
        }
        if tasks
            .iter()
            .any(|t| self.clashes_for(t.id).any(Clash::blocking))
        {
            return OrderPhase::Clashing;
        }
        let queued = any(|s| matches!(s, TaskState::Queued { .. }));
        let active = any(|s| matches!(s, TaskState::Open | TaskState::Claimed { .. }));
        if queued && !active {
            OrderPhase::Landing
        } else if any(|s| {
            matches!(
                s,
                TaskState::Claimed {
                    stage: Stage::Gating { .. },
                    ..
                }
            )
        }) {
            OrderPhase::Gating
        } else if any(|s| {
            matches!(
                s,
                TaskState::Claimed {
                    stage: Stage::Building { .. },
                    ..
                }
            )
        }) {
            OrderPhase::Building
        } else {
            OrderPhase::ReadingBack
        }
    }

    /// Queued tasks in merge order: by the seq at which each was queued.
    #[must_use]
    pub fn land_queue(&self) -> Vec<TaskId> {
        let mut queued: Vec<(Seq, TaskId)> = self
            .tasks
            .values()
            .filter_map(|t| match t.state {
                TaskState::Queued { queued_at, .. } => Some((queued_at, t.id)),
                _ => None,
            })
            .collect();
        queued.sort_unstable();
        queued.into_iter().map(|(_, task)| task).collect()
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
        let mut jobs = Vec::new();
        for order in self.orders.values() {
            match order.state {
                OrderState::Splitting => jobs.push(Job::Split {
                    order: order.id,
                    rev: order.rev,
                }),
                OrderState::Split if self.open_question(order.id).is_none() => {
                    let readbacks = self.unreviewed(order);
                    if !readbacks.is_empty() {
                        jobs.push(Job::Review {
                            order: order.id,
                            rev: order.rev,
                            readbacks,
                        });
                    }
                }
                OrderState::Split | OrderState::Cancelled => {}
            }
        }
        jobs.extend(
            self.clashes
                .values()
                .filter(|c| c.blocking() && c.explanation.is_none())
                .map(|c| Job::Explain { clash: c.id }),
        );
        for task in self.tasks.values() {
            let TaskState::Claimed {
                stage: Stage::Gating { run, .. },
                ..
            } = &task.state
            else {
                continue;
            };
            let Some(gate) = self.gates.get(run) else {
                continue;
            };
            jobs.extend(
                gate.witnesses
                    .iter()
                    .filter(|w| !gate.results.contains_key(w))
                    .map(|&witness| Job::RunWitness { run: *run, witness }),
            );
        }
        let front = self
            .land_queue()
            .first()
            .and_then(|&id| self.tasks.get(&id));
        if let Some(Task {
            id,
            state: TaskState::Queued { head, .. },
            ..
        }) = front
        {
            jobs.push(Job::Land {
                task: *id,
                head: head.clone(),
            });
        }
        jobs
    }

    fn unreviewed(&self, order: &Order) -> BTreeSet<ReadbackId> {
        self.readbacks
            .values()
            .filter(|r| {
                self.tasks
                    .get(&r.task)
                    .is_some_and(|t| t.order == order.id && t.state == TaskState::Open)
            })
            .filter(|r| r.verdict.as_ref().is_none_or(|v| v.rev != order.rev))
            .map(|r| r.id)
            .collect()
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
        let role = self.members.get(member)?.role;
        if !clash.menu().contains(choice) {
            return None;
        }
        if role == Role::Owner {
            return Some(SettleRight::Settle);
        }
        let own = choice
            .yielder(&clash.key)
            .and_then(|task| self.holder(task))
            .and_then(|agent| self.agents.get(agent))
            .is_some_and(|principal| &principal.owner == member);
        Some(if own {
            SettleRight::Settle
        } else {
            SettleRight::Propose
        })
    }

    /// The agent holding `task` under a live claim.
    pub(crate) fn holder(&self, task: TaskId) -> Option<&AgentId> {
        match &self.tasks.get(&task)?.state {
            TaskState::Claimed { claim, .. } => Some(&claim.lease.holder),
            _ => None,
        }
    }

    /// Declarations of every claimed task that has one. Detection runs over these.
    pub(crate) fn live_declarations(&self) -> impl Iterator<Item = &Declaration> {
        self.tasks.values().filter_map(|t| match &t.state {
            TaskState::Claimed { stage, .. } => stage.declaration(),
            _ => None,
        })
    }

    pub(crate) fn next_order_id(&self) -> OrderId {
        OrderId::new(self.ids.order)
    }

    pub(crate) fn next_task_id(&self) -> TaskId {
        TaskId::new(self.ids.task)
    }

    pub(crate) fn next_witness_id(&self) -> WitnessId {
        WitnessId::new(self.ids.witness)
    }

    pub(crate) fn next_readback_id(&self) -> ReadbackId {
        ReadbackId::new(self.ids.readback)
    }

    pub(crate) fn next_question_id(&self) -> QuestionId {
        QuestionId::new(self.ids.question)
    }

    pub(crate) fn next_clash_id(&self) -> ClashId {
        ClashId::new(self.ids.clash)
    }

    pub(crate) fn next_gate_id(&self) -> GateRunId {
        GateRunId::new(self.ids.gate)
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
        let mut log = log.into_iter();
        let first = log.next().ok_or(ReplayError::NotOpened)?;
        let Event::WorldOpened { world, .. } = &first.event else {
            return Err(ReplayError::NotOpened);
        };
        if first.seq != Seq::ZERO {
            return Err(ReplayError::NotOpened);
        }
        let mut folded = Self::opened(world.clone());
        folded.apply(&first);
        for stamped in log {
            if stamped.seq != folded.next_seq {
                return Err(ReplayError::SeqGap {
                    expected: folded.next_seq,
                    got: stamped.seq,
                });
            }
            folded.apply(&stamped);
        }
        Ok(folded)
    }

    /// Stamps `events` at the next seqs, applies each, and appends it to `batch`.
    fn record(
        &mut self,
        at: Timestamp,
        actor: &Actor,
        events: Vec<Event>,
        batch: &mut Vec<Stamped>,
    ) {
        for event in events {
            let stamped = Stamped {
                seq: self.next_seq,
                at,
                actor: actor.clone(),
                event,
            };
            self.apply(&stamped);
            batch.push(stamped);
        }
    }

    /// The only writer of world state. Never fails, reads a clock, or runs detection;
    /// `seq` continuity is checked by the caller.
    fn apply(&mut self, stamped: &Stamped) {
        match &stamped.event {
            Event::WorldOpened { founder, .. } => {
                let owner = Member {
                    id: founder.clone(),
                    role: Role::Owner,
                };
                self.members.insert(founder.clone(), owner);
            }
            Event::MemberAdded { member } => {
                self.members.insert(member.id.clone(), member.clone());
            }
            Event::AgentRegistered { agent } => {
                self.agents.insert(agent.id.clone(), agent.clone());
            }
            Event::IndexUpdated { rev } => self.index_rev = Some(rev.clone()),
            Event::OrderPosted {
                order,
                machine,
                sentence,
                witnesses,
                posted_by,
            } => {
                bump(&mut self.ids.order, order.get());
                for witness in witnesses {
                    bump(&mut self.ids.witness, witness.id.get());
                    self.witnesses.insert(witness.id, witness.clone());
                }
                let posted = Order {
                    id: *order,
                    machine: machine.clone(),
                    sentence: sentence.clone(),
                    rev: OrderRev::ZERO,
                    witnesses: witnesses.iter().map(|w| w.id).collect(),
                    posted_by: posted_by.clone(),
                    state: OrderState::Splitting,
                };
                self.orders.insert(*order, posted);
            }
            Event::OrderSplit { order, tasks, .. } => {
                if let Some(o) = self.orders.get_mut(order) {
                    o.state = OrderState::Split;
                }
                for (id, spec) in tasks {
                    bump(&mut self.ids.task, id.get());
                    let task = Task {
                        id: *id,
                        order: *order,
                        spec: spec.clone(),
                        state: TaskState::Open,
                        last_generation: None,
                    };
                    self.tasks.insert(*id, task);
                }
            }
            Event::ReadbackPosted { readback } => {
                bump(&mut self.ids.readback, readback.id.get());
                self.readbacks.insert(readback.id, readback.clone());
            }
            Event::ReadbacksReviewed { verdicts, .. } => {
                for (id, verdict) in verdicts {
                    if let Some(readback) = self.readbacks.get_mut(id) {
                        readback.verdict = Some(verdict.clone());
                    }
                }
            }
            Event::QuestionPinned { question } => {
                bump(&mut self.ids.question, question.id.get());
                self.questions.insert(question.id, question.clone());
            }
            Event::QuestionAnswered {
                question,
                option,
                witness,
            } => {
                if let Some(q) = self.questions.get_mut(question) {
                    q.answer = Some(*option);
                }
                bump(&mut self.ids.witness, witness.id.get());
                self.witnesses.insert(witness.id, witness.clone());
            }
            Event::OrderRevised {
                order,
                rev,
                witnesses,
            } => {
                if let Some(o) = self.orders.get_mut(order) {
                    o.rev = *rev;
                    o.witnesses.clone_from(witnesses);
                }
            }
            Event::LeaseGranted {
                task,
                lease,
                readback,
            } => {
                if let Some(t) = self.tasks.get_mut(task) {
                    t.last_generation = Some(lease.generation);
                    t.state = TaskState::Claimed {
                        claim: Claim {
                            lease: lease.clone(),
                            readback: *readback,
                        },
                        stage: Stage::Declaring,
                    };
                }
            }
            Event::LeaseRenewed {
                task,
                generation,
                expires_at,
            } => {
                if let Some(TaskState::Claimed { claim, .. }) =
                    self.tasks.get_mut(task).map(|t| &mut t.state)
                    && claim.lease.generation == *generation
                {
                    claim.lease.expires_at = *expires_at;
                }
            }
            Event::LeaseEnded {
                task, generation, ..
            } => {
                if let Some(t) = self.tasks.get_mut(task)
                    && matches!(&t.state, TaskState::Claimed { claim, .. } if claim.lease.generation == *generation)
                {
                    t.state = TaskState::Open;
                }
            }
            Event::Declared { decl } => {
                self.restage(decl.task, |_| Stage::Declared { decl: decl.clone() });
            }
            Event::ClashFound {
                clash,
                key,
                kind,
                found_at,
                contract,
            } => {
                bump(&mut self.ids.clash, clash.get());
                let found = Clash {
                    id: *clash,
                    key: key.clone(),
                    kind: *kind,
                    found_at: found_at.clone(),
                    contract: contract.clone(),
                    explanation: None,
                    state: ClashState::Open,
                };
                self.clashes.insert(*clash, found);
                self.clash_keys.insert(key.clone(), *clash);
                for party in [key.a(), key.b()] {
                    self.restage(party, |stage| match stage {
                        Stage::Building { decl } => Stage::Declared { decl },
                        other => other,
                    });
                }
            }
            Event::ClashDissolved { clash } => {
                if let Some(c) = self.clashes.get_mut(clash) {
                    c.state = ClashState::Dissolved;
                }
            }
            Event::ClashExplained { clash, explanation } => {
                if let Some(c) = self.clashes.get_mut(clash) {
                    c.explanation = Some(explanation.clone());
                }
            }
            Event::ClashProposed { clash, choice, by } => {
                if let Some(c) = self.clashes.get_mut(clash) {
                    c.state = ClashState::Proposed {
                        choice: choice.clone(),
                        by: by.clone(),
                    };
                }
            }
            Event::ClashSettled { clash, choice, by } => {
                if let Some(c) = self.clashes.get_mut(clash) {
                    c.state = ClashState::Settled {
                        choice: choice.clone(),
                        by: by.clone(),
                    };
                }
                if let Settlement::Narrow { task } = choice {
                    self.restage(*task, |stage| match stage {
                        Stage::Declared { .. } | Stage::Building { .. } => Stage::Declaring,
                        other => other,
                    });
                }
            }
            Event::BuildCleared { task } => {
                self.restage(*task, |stage| match stage {
                    Stage::Declared { decl } => Stage::Building { decl },
                    other => other,
                });
            }
            Event::BuildReported {
                task,
                generation,
                head,
                run,
                rev,
                witnesses,
            } => {
                bump(&mut self.ids.gate, run.get());
                let started = GateRun {
                    id: *run,
                    task: *task,
                    generation: *generation,
                    rev: *rev,
                    head: head.clone(),
                    witnesses: witnesses.clone(),
                    results: BTreeMap::new(),
                };
                self.gates.insert(*run, started);
                self.restage(*task, |stage| match stage {
                    Stage::Building { decl } => Stage::Gating {
                        decl,
                        run: *run,
                        head: head.clone(),
                    },
                    other => other,
                });
            }
            Event::WitnessReported {
                run,
                witness,
                outcome,
            } => {
                if let Some(gate) = self.gates.get_mut(run) {
                    gate.results.insert(*witness, outcome.clone());
                }
            }
            Event::GateClosed { run, verdict } => self.close_gate(*run, verdict, stamped.seq),
            Event::Landed { task, commit } => {
                if let Some(t) = self.tasks.get_mut(task) {
                    t.state = TaskState::Landed {
                        commit: commit.clone(),
                    };
                }
            }
            Event::LandFailed { task, .. } => {
                if let Some(t) = self.tasks.get_mut(task) {
                    t.state = TaskState::Open;
                }
            }
            Event::OrderCancelled { order } => {
                if let Some(o) = self.orders.get_mut(order) {
                    o.state = OrderState::Cancelled;
                }
                for t in self.tasks.values_mut().filter(|t| t.order == *order) {
                    if !matches!(t.state, TaskState::Landed { .. }) {
                        t.state = TaskState::Dropped;
                    }
                }
            }
            Event::ModelUsed {
                model,
                step,
                input_tokens,
                output_tokens,
            } => {
                let count = self
                    .usage
                    .totals
                    .entry(model.clone())
                    .or_default()
                    .entry(*step)
                    .or_default();
                count.input += u64::from(*input_tokens);
                count.output += u64::from(*output_tokens);
                count.calls = count.calls.saturating_add(1);
            }
        }
        self.next_seq = stamped.seq.next();
    }

    /// Replaces a claimed task's stage with `f(stage)`. Does nothing for an unclaimed task.
    fn restage(&mut self, task: TaskId, f: impl FnOnce(Stage) -> Stage) {
        if let Some(TaskState::Claimed { stage, .. }) =
            self.tasks.get_mut(&task).map(|t| &mut t.state)
        {
            let old = mem::replace(stage, Stage::Declaring);
            *stage = f(old);
        }
    }

    fn close_gate(&mut self, run: GateRunId, verdict: &GateVerdict, seq: Seq) {
        let Some(task) = self.gates.get(&run).map(|g| g.task) else {
            return;
        };
        let Some(t) = self.tasks.get_mut(&task) else {
            return;
        };
        t.state = match mem::replace(&mut t.state, TaskState::Open) {
            TaskState::Claimed {
                claim,
                stage:
                    Stage::Gating {
                        decl,
                        run: gating,
                        head,
                    },
            } if gating == run => match verdict {
                GateVerdict::Passed => TaskState::Queued {
                    head,
                    run,
                    queued_at: seq,
                },
                GateVerdict::Failed { .. } => TaskState::Claimed {
                    claim,
                    stage: Stage::Building { decl },
                },
                GateVerdict::Running => TaskState::Claimed {
                    claim,
                    stage: Stage::Gating { decl, run, head },
                },
            },
            other => other,
        };
    }
}

fn bump(counter: &mut u32, seen: u32) {
    *counter = (*counter).max(seen.saturating_add(1));
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
        let opened = Stamped {
            seq: Seq::ZERO,
            at: now,
            actor: Actor::Member(founder.clone()),
            event: Event::WorldOpened {
                world: world.clone(),
                founder,
            },
        };
        let mut state = World::opened(world);
        state.apply(&opened);
        (Self { world: state }, opened)
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
        let mut next = self.world.clone();
        let mut batch = Vec::new();
        let expired = lease::expire_due(&next, ctx);
        next.record(ctx.now, &Actor::System, expired, &mut batch);
        let (actor, decided) = decide(&next, ctx, cmd)?;
        next.record(ctx.now, &actor, decided, &mut batch);
        let orphaned = clash::dissolve_orphans(&next);
        next.record(ctx.now, &Actor::System, orphaned, &mut batch);
        let cleared = clash::build_clear(&next);
        next.record(ctx.now, &Actor::System, cleared, &mut batch);
        self.world = next;
        Ok(batch)
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
        let expected = self.world.next_seq;
        if stamped.seq < expected {
            return Ok(());
        }
        if stamped.seq > expected {
            return Err(ReplayError::SeqGap {
                expected,
                got: stamped.seq,
            });
        }
        self.world.apply(stamped);
        Ok(())
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

fn decide(world: &World, ctx: &Ctx<'_>, cmd: Command) -> Result<(Actor, Vec<Event>), Rejection> {
    match cmd {
        Command::Human { member, cmd } => {
            if world.member(&member).is_none() {
                return Err(Rejection::forbidden("not a member of this world"));
            }
            let events = decide_human(world, &member, cmd)?;
            Ok((Actor::Member(member), events))
        }
        Command::Agent { agent, cmd } => {
            if world.agent(&agent).is_none() {
                return Err(Rejection::forbidden("agent is not registered"));
            }
            let events = decide_agent(world, ctx, &agent, cmd)?;
            Ok((Actor::Agent(agent), events))
        }
        Command::System(cmd) => Ok((Actor::System, decide_system(world, ctx, cmd)?)),
    }
}

fn decide_human(world: &World, member: &MemberId, cmd: HumanCmd) -> Result<Vec<Event>, Rejection> {
    match cmd {
        HumanCmd::AddMember {
            member: added,
            role,
        } => member::decide_add_member(world, member, added, role),
        HumanCmd::RegisterAgent { agent } => member::decide_register_agent(world, member, agent),
        HumanCmd::PostOrder {
            machine,
            sentence,
            witnesses,
        } => order::decide_post_order(world, member, machine, sentence, witnesses),
        HumanCmd::AnswerQuestion { question, option } => {
            order::decide_answer(world, member, question, option)
        }
        HumanCmd::SettleClash { clash, choice } => {
            clash::decide_settle(world, member, clash, choice)
        }
        HumanCmd::CancelOrder { order } => order::decide_cancel(world, member, order),
    }
}

fn decide_agent(
    world: &World,
    ctx: &Ctx<'_>,
    agent: &AgentId,
    cmd: AgentCmd,
) -> Result<Vec<Event>, Rejection> {
    match cmd {
        AgentCmd::PostReadback {
            task,
            summary,
            touches,
        } => order::decide_readback(world, agent, task, summary, touches),
        AgentCmd::Claim { task } => lease::decide_claim(world, ctx, agent, task),
        AgentCmd::Renew { fence } => lease::decide_renew(world, ctx, agent, fence),
        AgentCmd::Release { fence } => lease::decide_release(world, ctx, agent, fence),
        AgentCmd::Declare { fence, footprint } => {
            clash::decide_declare(world, ctx, agent, fence, footprint)
        }
        AgentCmd::ReportBuilt { fence, head } => {
            gate::decide_report_built(world, ctx, agent, fence, head)
        }
    }
}

fn decide_system(world: &World, ctx: &Ctx<'_>, cmd: SystemCmd) -> Result<Vec<Event>, Rejection> {
    match cmd {
        SystemCmd::IndexUpdated => Ok(clash::decide_index_updated(world, ctx)),
        SystemCmd::SplitReady { order, rev, tasks } => {
            order::decide_split_ready(world, order, rev, tasks)
        }
        SystemCmd::ReadbacksReviewed {
            order,
            rev,
            verdicts,
            question,
        } => order::decide_reviewed(world, order, rev, verdicts, question),
        SystemCmd::ClashExplained { clash, explanation } => {
            clash::decide_explained(world, clash, explanation)
        }
        SystemCmd::WitnessResult {
            run,
            witness,
            outcome,
        } => gate::decide_witness_result(world, ctx, run, witness, outcome),
        SystemCmd::LandResult { task, head, result } => {
            gate::decide_land_result(world, task, &head, result)
        }
        SystemCmd::ModelUsed {
            model,
            step,
            input_tokens,
            output_tokens,
        } => Ok(vec![Event::ModelUsed {
            model,
            step,
            input_tokens,
            output_tokens,
        }]),
        SystemCmd::Tick => Ok(Vec::new()),
    }
}
