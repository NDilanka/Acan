//! Acan's world model: orders, claims, clashes and gates as one event-sourced fold.
//!
//! One [`Origin`] per world decides commands and appends [`Stamped`] events. Every other
//! holder (the browser, a joined desktop, the hosted replay) keeps a [`Replica`] that folds
//! the same events with the same code. The crate does no I/O and builds for
//! `wasm32-unknown-unknown`.
#![doc = include_str!("../DESIGN.md")]

mod clash;
mod event;
mod gate;
mod ids;
mod index;
mod job;
mod lease;
mod member;
mod order;
mod world;

pub use clash::{
    Barrier, BarrierHold, Clash, ClashKey, ClashState, ContractRef, Declaration, DeclarationInput,
    Explanation, Overlap, OverlapKind, Settlement, SymbolRef, detect,
};
pub use event::{
    Actor, AgentCmd, Command, Event, HumanCmd, Rejection, ReplayError, Stamped, SystemCmd,
};
pub use gate::{GateRun, GateVerdict, WitnessOutcome};
pub use ids::{
    AgentId, ClashId, CodeRev, CommitId, FilePath, GateRunId, Generation, MachineKey, MemberId,
    OrderId, OrderRev, ParseIdError, QuestionId, ReadbackId, Seq, TaskId, Timestamp, WitnessId,
    WorldId,
};
pub use index::{Belt, CodeIndex, IndexedFile, IndexedSymbol, Locus};
pub use job::Job;
pub use lease::{Claim, DECLARE_BARRIER_MS, Fence, LEASE_TTL_MS, Lease, LeaseEnd};
pub use member::{AgentPrincipal, Member, Role, SettleRight};
pub use order::{
    Check, Judgement, Order, OrderPhase, OrderState, Question, QuestionDraft, Readback,
    ReadbackVerdict, Region, Stage, Task, TaskSpec, TaskState, Witness, WitnessKind, WitnessSpec,
};
pub use world::{Ctx, ModelUsage, Origin, Replica, Step, TokenCount, World, WorldRef};
