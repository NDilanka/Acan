//! Declarations, pure clash detection, and settlement.
//!
//! [`detect`] is a pure function of declarations and a [`CodeIndex`]. The origin calls it
//! inside `execute` and records its blocking results as `ClashFound` and `ClashDissolved`
//! events, so replicas never re-run it and never need the index.
//!
//! ```text
//! (none) --ClashFound--> Open
//! Open --ClashExplained--> Open (explanation attached; settling never waits for it)
//! Open --SettleClash with Propose right--> Proposed
//! Open|Proposed --SettleClash with Settle right--> Settled
//! Open|Proposed --redetection no longer finds the key--> Dissolved
//! ```

use serde::{Deserialize, Serialize};

use crate::event::{Event, Rejection};
use crate::ids::{AgentId, ClashId, CodeRev, FilePath, Generation, MemberId, TaskId, WitnessId};
use crate::index::{CodeIndex, Locus};
use crate::lease::Fence;
use crate::world::{Ctx, World};

/// File names that are produced by a tool and must have one producer. Matched on the last path segment.
pub(crate) const GENERATED_FILES: &[&str] = &[
    "Cargo.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "bun.lockb",
];

/// A symbol named by file and name. The origin resolves it to byte ranges through the index.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SymbolRef {
    /// The file.
    pub file: FilePath,
    /// The symbol name.
    pub name: String,
}

/// What an agent says it will change, before it writes code.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclarationInput {
    /// Files the agent may rewrite anywhere. Coverage is unknown, so these overlap with anything in the file.
    pub files: Vec<FilePath>,
    /// Symbols whose bodies the agent will change.
    pub symbols: Vec<SymbolRef>,
    /// Exported symbols whose signature or contract the agent will change.
    pub interfaces: Vec<SymbolRef>,
    /// Files the agent will only append to: a route table, a registry, a barrel file.
    pub inserts: Vec<FilePath>,
}

/// An accepted declaration, bound to the claim generation and index revision it was made under.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Declaration {
    /// The declaring task.
    pub task: TaskId,
    /// The claim generation it was declared under.
    pub generation: Generation,
    /// The index revision current when it was accepted.
    pub index: CodeRev,
    /// What the agent will change.
    pub footprint: DeclarationInput,
}

/// How two declarations overlap, after the agent-collision taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlapKind {
    /// Both change the same symbol's byte range. Blocking.
    SharedImplementation,
    /// One changes an exported symbol in machine M; the other's footprint is in a machine that imports M. Blocking.
    Contract,
    /// Both touch a generated file such as a lockfile. Blocking: one producer.
    GeneratedOutput,
    /// Both append to the same insertion file. Blocking, cheap to settle.
    SharedInsertion,
    /// A whole-file write (or an unresolved symbol) meets anything in that file. Blocking.
    WholeFile,
    /// Same file, non-overlapping symbols. Advisory: never recorded as a clash.
    DisjointRegions,
}

impl OverlapKind {
    /// Whether this kind blocks building until settled.
    #[must_use]
    pub fn blocking(self) -> bool {
        !matches!(self, Self::DisjointRegions)
    }
}

/// The identity of an overlap: an ordered task pair and a place.
///
/// The lower task id is always `a`, so detection is deterministic and re-detection of the
/// same overlap maps to the same clash.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClashKey {
    a: TaskId,
    b: TaskId,
    at: Locus,
}

impl ClashKey {
    /// Builds a key for two distinct tasks, ordering them lower first.
    ///
    /// # Panics
    /// Panics if `x == y`; a task never clashes with itself.
    #[must_use]
    pub fn new(x: TaskId, y: TaskId, at: Locus) -> Self {
        assert_ne!(x, y, "a clash needs two different tasks, got {x} twice");
        let (a, b) = if x < y { (x, y) } else { (y, x) };
        Self { a, b, at }
    }

    /// The lower task id.
    #[must_use]
    pub fn a(&self) -> TaskId {
        self.a
    }

    /// The higher task id.
    #[must_use]
    pub fn b(&self) -> TaskId {
        self.b
    }

    /// Where the overlap is.
    #[must_use]
    pub fn at(&self) -> &Locus {
        &self.at
    }

    /// Whether `task` is one of the two parties.
    #[must_use]
    pub fn involves(&self, task: TaskId) -> bool {
        self.a == task || self.b == task
    }
}

/// One overlap found by [`detect`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Overlap {
    /// The tasks and place.
    pub key: ClashKey,
    /// How they overlap.
    pub kind: OverlapKind,
}

/// Finds every overlap between declarations of different tasks.
///
/// Pure: no I/O, no model, no clock. The output is sorted by key, then kind, and does not
/// depend on the order of `decls`. Rules for each pair of declarations from different tasks:
///
/// - A symbol that does not resolve in `index` counts as a whole-file write of its file.
/// - Same symbol range (ranges intersect) in one file: `SharedImplementation` at that symbol.
/// - Both list the file in `inserts`: `SharedInsertion` at the file.
/// - Both touch a file in the generated list: `GeneratedOutput` at the file.
/// - One side writes the whole file and the other touches it at all: `WholeFile` at the file.
/// - Same file, only disjoint symbols: `DisjointRegions` at the file.
/// - A lists an exported interface symbol in machine M, and any of B's files or symbols is
///   in a machine N with a belt `N -> M`: `Contract` at that belt.
///
/// At most one overlap is reported per key; when several rules hit the same key, the first
/// kind in [`OverlapKind`] order wins.
#[must_use]
pub fn detect(decls: &[&Declaration], index: &CodeIndex) -> Vec<Overlap> {
    todo!()
}

/// A recorded blocking overlap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clash {
    /// The clash id.
    pub id: ClashId,
    /// The tasks and place.
    pub key: ClashKey,
    /// How they overlap.
    pub kind: OverlapKind,
    /// The index revision detection ran against.
    pub found_at: CodeRev,
    /// The foreman's explanation, once the explain job has run.
    pub explanation: Option<Explanation>,
    /// Where the clash is in its lifecycle.
    pub state: ClashState,
}

/// A clash's lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ClashState {
    /// Blocking; no choice yet.
    Open,
    /// Blocking; a member without the right to settle proposed a choice.
    Proposed {
        /// The proposed choice.
        choice: Settlement,
        /// Who proposed it.
        by: MemberId,
    },
    /// Settled. Its effect is applied.
    Settled {
        /// The chosen settlement.
        choice: Settlement,
        /// Who settled it.
        by: MemberId,
    },
    /// A later declaration or rescan removed the overlap.
    Dissolved,
}

/// The foreman's account of a clash, written by a model. It never changes the legal menu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Explanation {
    /// What goes wrong if both proceed.
    pub consequence: String,
    /// The witness that would break, if any.
    pub breaks: Option<WitnessId>,
    /// The suggested choice. Ignored unless it is on [`Clash::menu`].
    pub suggested: Option<Settlement>,
}

/// A way to settle a clash.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "choice", rename_all = "snake_case")]
pub enum Settlement {
    /// `task` redeclares without the overlapping place. Its task returns to `Declaring`.
    Narrow {
        /// The task that narrows.
        task: TaskId,
    },
    /// `then` may not build until `first` has landed.
    Serialize {
        /// The task that goes first.
        first: TaskId,
        /// The task that waits.
        then: TaskId,
    },
    /// `owner` lands the named contract first; the other task builds against it.
    SplitAtContract {
        /// The task that owns the contract.
        owner: TaskId,
        /// The contract, named after the interface symbol.
        contract: String,
    },
}

impl Settlement {
    /// The task that gives way under this choice. `None` if the choice does not fit `key`.
    #[must_use]
    pub fn yielder(&self, key: &ClashKey) -> Option<TaskId> {
        todo!()
    }
}

impl Clash {
    /// The legal choices for this clash, two or three, derived from its kind and key.
    ///
    /// `SharedImplementation`, `GeneratedOutput`, `SharedInsertion`, `WholeFile`: narrow `a`,
    /// narrow `b`, serialize `a` then `b`. `Contract`: split at the contract owned by the
    /// task that changes the interface, or serialize that task first.
    #[must_use]
    pub fn menu(&self) -> Vec<Settlement> {
        todo!()
    }

    /// Whether this clash still blocks its tasks from building.
    #[must_use]
    pub fn blocking(&self) -> bool {
        matches!(self.state, ClashState::Open | ClashState::Proposed { .. })
    }
}

/// `Declare`: checks the fence, stores the declaration, runs [`detect`] over every live
/// declaration, and emits `ClashFound` for new keys and `ClashDissolved` for vanished ones.
/// Allowed in `Declaring`, `Declared` and `Building`; the task moves to `Declared`.
pub(crate) fn decide_declare(
    world: &World,
    ctx: &Ctx<'_>,
    agent: &AgentId,
    fence: Fence,
    footprint: DeclarationInput,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}

/// `IndexUpdated`: re-runs detection over live declarations against the new index.
pub(crate) fn decide_index_updated(world: &World, ctx: &Ctx<'_>) -> Vec<Event> {
    todo!()
}

/// `SettleClash`: the choice must be on the menu. Emits `ClashSettled` or `ClashProposed`
/// according to [`World::settle_right`].
pub(crate) fn decide_settle(
    world: &World,
    member: &MemberId,
    clash: ClashId,
    choice: Settlement,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}

/// `ClashExplained`: attaches the model's explanation to an open or proposed clash.
pub(crate) fn decide_explained(
    world: &World,
    clash: ClashId,
    explanation: Explanation,
) -> Result<Vec<Event>, Rejection> {
    todo!()
}

/// `BuildCleared` for every `Declared` task with no blocking clash and no unlanded
/// `Serialize` predecessor. The origin runs this after every accepted command.
pub(crate) fn build_clear(world: &World) -> Vec<Event> {
    todo!()
}
