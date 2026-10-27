//! Folds the origin's event log into the state the product screen draws.
//!
//! [`Mirror`] wraps an [`acan_core::Replica`], so the screen shows exactly what the origin
//! decided. [`Mirror::view`] reads that state into a [`View`], the one shape the page renders.

use std::collections::{BTreeMap, BTreeSet};

use acan_core::{
    AgentId, Clash, ClashId, ClashState, Declaration, DeclarationInput, Event, FilePath, Locus,
    OrderId, OrderPhase, OverlapKind, Question, QuestionId, ReplayError, Replica, Seq, Settlement,
    Stage, Stamped, Task, TaskId, TaskState, World,
};
use serde::Serialize;

use crate::layout::Plan;

/// Agent body colors, as `#rrggbb`. The first agent gets the first color.
pub const PALETTE: [&str; 6] = [
    "#2f6fed", "#e0662b", "#1f9d6b", "#8a4fd6", "#d6336c", "#0f8ea3",
];

/// Where the first agent's body sits inside its machine, in world units from the corner.
const BODY_OFFSET: [f32; 2] = [14.0, 44.0];
/// Horizontal step between two agent bodies on one machine.
const BODY_STEP: f32 = 40.0;
/// Size of an agent body, in world units.
const BODY_SIZE: [f32; 2] = [30.0, 20.0];

/// The origin's state, folded from its log.
#[derive(Debug, Clone)]
pub struct Mirror {
    replica: Replica,
    /// Registered agents, in registration order. Their position fixes their color.
    agents: Vec<AgentId>,
    /// The agent holding each task's lease, from the last grant.
    holders: BTreeMap<TaskId, AgentId>,
}

impl Mirror {
    /// Folds a log that starts with `WorldOpened` at seq 0.
    ///
    /// # Errors
    ///
    /// Returns the replay error when the log does not replay.
    pub fn from_log(log: Vec<Stamped>) -> Result<Self, ReplayError> {
        let replica = Replica::from_log(log.iter().cloned())?;
        let mut mirror = Self {
            replica,
            agents: Vec::new(),
            holders: BTreeMap::new(),
        };
        for stamped in &log {
            mirror.note(stamped);
        }
        Ok(mirror)
    }

    /// Applies one streamed event. An event already applied is ignored.
    ///
    /// # Errors
    ///
    /// Returns [`ReplayError::SeqGap`] when events were missed, and the replica's error for an
    /// event that refers to something the log never created. Nothing is applied in either case.
    pub fn ingest(&mut self, stamped: &Stamped) -> Result<(), ReplayError> {
        let fresh = stamped.seq == self.replica.next_seq();
        self.replica.ingest(stamped)?;
        if fresh {
            self.note(stamped);
        }
        Ok(())
    }

    /// The seq the next event will carry. The stream resumes from here.
    #[must_use]
    pub fn next_seq(&self) -> Seq {
        self.replica.next_seq()
    }

    /// The origin's world, read-only.
    #[must_use]
    pub fn world(&self) -> &World {
        self.replica.world()
    }

    /// Records what the world does not keep: which agents are registered and who holds each
    /// lease. A released lease keeps its holder; the task's state says whether it still counts.
    fn note(&mut self, stamped: &Stamped) {
        match &stamped.event {
            Event::AgentRegistered { agent } => self.agents.push(agent.id.clone()),
            Event::LeaseGranted { task, lease, .. } => {
                self.holders.insert(*task, lease.holder.clone());
            }
            _ => {}
        }
    }

    /// Reads the folded state into the view the page draws, laid out on `plan`.
    #[must_use]
    pub fn view(&self, plan: &Plan) -> View {
        let world = self.world();
        let tasks: Vec<&Task> = world
            .orders()
            .flat_map(|order| world.tasks_of(order.id))
            .collect();

        let abnormal: BTreeSet<usize> = world
            .clashes()
            .filter(|clash| unsettled(clash))
            .flat_map(|clash| plan.machines_at(clash.key.at()))
            .collect();
        let machines = plan
            .machines
            .iter()
            .enumerate()
            .map(|(i, m)| MachineView {
                name: m.name.clone(),
                min: m.min,
                size: m.size,
                abnormal: abnormal.contains(&i),
            })
            .collect();

        let orders = world
            .orders()
            .map(|order| OrderView {
                id: order.id,
                machine: order.machine.as_str().to_owned(),
                sentence: order.sentence.clone(),
                phase: match world.phase(order.id) {
                    Some(phase) => phase_key(&phase),
                    None => "unknown",
                },
                tasks: world.tasks_of(order.id).count(),
                at: plan
                    .index_of(order.machine.as_str())
                    .map(|i| plan.machines[i].center()),
            })
            .collect();

        let task_views = tasks
            .iter()
            .map(|task| TaskView {
                id: task.id,
                order: task.order,
                title: task.spec.title.clone(),
                state: task_state_key(world, task),
                agent: match task.state {
                    TaskState::Open | TaskState::Dropped => None,
                    _ => self.holders.get(&task.id).cloned(),
                },
            })
            .collect();

        let agents = self.agent_views(plan, world, &tasks);

        let questions = world
            .orders()
            .filter_map(|order| world.open_question(order.id))
            .map(|question| question_view(plan, question))
            .collect();

        let clashes = world
            .clashes()
            .map(|clash| clash_view(plan, clash))
            .collect();

        View {
            next_seq: self.next_seq(),
            machines,
            orders,
            tasks: task_views,
            agents,
            questions,
            clashes,
        }
    }

    fn agent_views(&self, plan: &Plan, world: &World, tasks: &[&Task]) -> Vec<AgentView> {
        let mut seats: BTreeMap<usize, usize> = BTreeMap::new();
        self.agents
            .iter()
            .enumerate()
            .map(|(i, agent)| {
                let focus = self.focus(world, tasks, agent);
                let machine = focus.and_then(|(task, _)| task_machine(world, plan, task));
                let body = machine.map(|m| {
                    let seat = seats.entry(m).or_insert(0);
                    let at = *seat;
                    *seat += 1;
                    let corner = plan.machines[m].min;
                    let x = corner[0] + BODY_OFFSET[0] + at as f32 * BODY_STEP;
                    let y = corner[1] + BODY_OFFSET[1];
                    [x, y, BODY_SIZE[0], BODY_SIZE[1]]
                });
                AgentView {
                    id: agent.clone(),
                    color: PALETTE[i % PALETTE.len()],
                    task: focus.map(|(task, _)| task.id),
                    activity: focus.map_or("idle", |(_, activity)| activity),
                    machine: machine.map(|m| plan.machines[m].name.clone()),
                    body,
                }
            })
            .collect()
    }

    /// What `agent` is doing now: the task it holds, else the open task it read back.
    fn focus<'a>(
        &self,
        world: &'a World,
        tasks: &[&'a Task],
        agent: &AgentId,
    ) -> Option<(&'a Task, &'static str)> {
        let held = tasks.iter().copied().find(|task| {
            self.holders.get(&task.id) == Some(agent)
                && !matches!(task.state, TaskState::Open | TaskState::Dropped)
        });
        if let Some(task) = held {
            return Some((task, task_state_key(world, task)));
        }
        tasks
            .iter()
            .copied()
            .find(|task| {
                matches!(task.state, TaskState::Open)
                    && world.readbacks_of(task.id).any(|r| &r.agent == agent)
            })
            .map(|task| (task, "readback"))
    }
}

/// The screen's view of the origin's state. Ids keep their origin shape, so a click can send
/// them back unchanged.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct View {
    /// The seq the mirror has applied up to.
    pub next_seq: Seq,
    /// Every machine of the plan, in plan order.
    pub machines: Vec<MachineView>,
    /// Every order.
    pub orders: Vec<OrderView>,
    /// Every task of every order.
    pub tasks: Vec<TaskView>,
    /// Every registered agent.
    pub agents: Vec<AgentView>,
    /// Open questions, one per order at most.
    pub questions: Vec<QuestionView>,
    /// Every clash the origin recorded, including resolved ones.
    pub clashes: Vec<ClashView>,
}

/// A machine footprint and whether a clash stands on it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MachineView {
    /// The machine name.
    pub name: String,
    /// Top-left corner, in world units.
    pub min: [f32; 2],
    /// Width and height, in world units.
    pub size: [f32; 2],
    /// Whether an unsettled clash touches this machine.
    pub abnormal: bool,
}

/// An order pinned on a machine.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OrderView {
    /// The order id.
    pub id: OrderId,
    /// The machine it is pinned on.
    pub machine: String,
    /// What the human asked for.
    pub sentence: String,
    /// The derived phase, in snake case.
    pub phase: &'static str,
    /// How many tasks the foreman split it into.
    pub tasks: usize,
    /// Where the pin sits, in world units.
    pub at: Option<[f32; 2]>,
}

/// A task and what it is doing.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskView {
    /// The task id.
    pub id: TaskId,
    /// The order it belongs to.
    pub order: OrderId,
    /// The task title.
    pub title: String,
    /// The task's activity, in snake case.
    pub state: &'static str,
    /// The agent holding the task's lease, if any.
    pub agent: Option<AgentId>,
}

/// A registered agent, with its body on the plan when it has a task.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AgentView {
    /// The agent id.
    pub id: AgentId,
    /// The body color, as `#rrggbb`.
    pub color: &'static str,
    /// The task the agent is on.
    pub task: Option<TaskId>,
    /// What the agent is doing. `idle` when it has no task.
    pub activity: &'static str,
    /// The machine the agent works on.
    pub machine: Option<String>,
    /// The body rectangle `[x, y, w, h]`, in world units.
    pub body: Option<[f32; 4]>,
}

/// An open question with its two answers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuestionView {
    /// The question id.
    pub id: QuestionId,
    /// The order it belongs to.
    pub order: OrderId,
    /// What is being asked.
    pub text: String,
    /// The two answers.
    pub options: [String; 2],
    /// Where the pin sits, in world units.
    pub at: Option<[f32; 2]>,
}

/// A clash: its pin, its explanation, and the choices it offers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClashView {
    /// The clash id.
    pub id: ClashId,
    /// `open`, `proposed`, `settled` or `dissolved`.
    pub state: &'static str,
    /// The overlap kind, in snake case.
    pub kind: &'static str,
    /// The lower task of the pair.
    pub a: TaskId,
    /// The higher task of the pair.
    pub b: TaskId,
    /// The pin label, such as `c-1 · breaks w-3`.
    pub label: String,
    /// Where the pin sits, in world units.
    pub at: Option<[f32; 2]>,
    /// What goes wrong if both proceed, once the explain job has run.
    pub consequence: Option<String>,
    /// The witness that would break, if the explanation names one.
    pub breaks: Option<String>,
    /// The legal choices, numbered from 1. Empty once the clash is resolved.
    pub options: Vec<OptionView>,
    /// How the clash was resolved, when it has been.
    pub resolved: Option<String>,
}

/// One numbered choice on a clash card.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OptionView {
    /// The key that picks it, 1 for the first option.
    pub key: usize,
    /// The choice in words.
    pub label: String,
    /// The command payload for the choice.
    pub choice: Settlement,
}

fn unsettled(clash: &Clash) -> bool {
    matches!(clash.state, ClashState::Open | ClashState::Proposed { .. })
}

fn clash_view(plan: &Plan, clash: &Clash) -> ClashView {
    let (state, options, resolved) = match &clash.state {
        ClashState::Open => ("open", menu(clash), None),
        ClashState::Proposed { .. } => ("proposed", menu(clash), None),
        ClashState::Settled { choice, by } => (
            "settled",
            Vec::new(),
            Some(format!("{} by {by}", option_label(choice))),
        ),
        ClashState::Dissolved => ("dissolved", Vec::new(), Some("dissolved".to_owned())),
    };
    let explanation = clash.explanation.as_ref();
    let breaks = explanation.and_then(|e| e.breaks);
    let label = match breaks {
        Some(witness) => format!("{} · breaks {witness}", clash.id),
        None => format!("{} · overlap", clash.id),
    };
    ClashView {
        id: clash.id,
        state,
        kind: overlap_key(&clash.kind),
        a: clash.key.a(),
        b: clash.key.b(),
        label,
        at: plan.point_at(clash.key.at()),
        consequence: explanation.map(|e| e.consequence.clone()),
        breaks: breaks.map(|witness| witness.to_string()),
        options,
        resolved,
    }
}

fn menu(clash: &Clash) -> Vec<OptionView> {
    clash
        .menu()
        .into_iter()
        .enumerate()
        .map(|(i, choice)| OptionView {
            key: i + 1,
            label: option_label(&choice),
            choice,
        })
        .collect()
}

fn option_label(choice: &Settlement) -> String {
    match choice {
        Settlement::Narrow { task } => format!("narrow {task}"),
        Settlement::Serialize { first, then } => format!("{first} lands first, {then} waits"),
        Settlement::SplitAtContract { owner, contract } => {
            format!("split at {contract}, {owner} lands it first")
        }
    }
}

fn question_view(plan: &Plan, question: &Question) -> QuestionView {
    QuestionView {
        id: question.id,
        order: question.order,
        text: question.text.clone(),
        options: question.options.clone(),
        at: plan.point_at(&question.at),
    }
}

fn phase_key(phase: &OrderPhase) -> &'static str {
    match phase {
        OrderPhase::Splitting => "splitting",
        OrderPhase::ReadingBack => "reading_back",
        OrderPhase::Asking { .. } => "asking",
        OrderPhase::Clashing => "clashing",
        OrderPhase::Building => "building",
        OrderPhase::Gating => "gating",
        OrderPhase::Landing => "landing",
        OrderPhase::Landed => "landed",
        OrderPhase::Cancelled => "cancelled",
    }
}

fn overlap_key(kind: &OverlapKind) -> &'static str {
    match kind {
        OverlapKind::SharedImplementation => "shared_implementation",
        OverlapKind::Contract => "contract",
        OverlapKind::GeneratedOutput => "generated_output",
        OverlapKind::SharedInsertion => "shared_insertion",
        OverlapKind::WholeFile => "whole_file",
        OverlapKind::DisjointRegions => "disjoint_regions",
    }
}

/// The activity a task shows: its readback, its claim stage, or its land state.
fn task_state_key(world: &World, task: &Task) -> &'static str {
    match &task.state {
        TaskState::Open => {
            if world.readbacks_of(task.id).next().is_some() {
                "readback"
            } else {
                "open"
            }
        }
        TaskState::Claimed { stage, .. } => stage_key(stage),
        TaskState::Queued { .. } => "queued",
        TaskState::Landed { .. } => "landed",
        TaskState::Dropped => "dropped",
    }
}

fn stage_key(stage: &Stage) -> &'static str {
    match stage {
        Stage::Declaring => "claimed",
        Stage::Declared { .. } => "declared",
        Stage::Building { .. } => "building",
        Stage::Gating { .. } => "gating",
    }
}

fn stage_declaration(stage: &Stage) -> Option<&Declaration> {
    match stage {
        Stage::Declaring => None,
        Stage::Declared { decl } | Stage::Building { decl } | Stage::Gating { decl, .. } => {
            Some(decl)
        }
    }
}

/// The machine a task works on: its declared files, else where its readbacks say, else the
/// machine its order is pinned on.
fn task_machine(world: &World, plan: &Plan, task: &Task) -> Option<usize> {
    if let TaskState::Claimed { stage, .. } = &task.state
        && let Some(decl) = stage_declaration(stage)
        && let Some(machine) =
            footprint_paths(&decl.footprint).find_map(|path| plan.machine_of_path(path.as_str()))
    {
        return Some(machine);
    }
    let touched = world
        .readbacks_of(task.id)
        .flat_map(|readback| readback.touches.iter())
        .find_map(|locus: &Locus| plan.machines_at(locus).into_iter().next());
    touched.or_else(|| {
        world
            .order(task.order)
            .and_then(|order| plan.index_of(order.machine.as_str()))
    })
}

fn footprint_paths(footprint: &DeclarationInput) -> impl Iterator<Item = &FilePath> {
    footprint
        .files
        .iter()
        .chain(&footprint.inserts)
        .chain(footprint.symbols.iter().map(|symbol| &symbol.file))
        .chain(footprint.interfaces.iter().map(|symbol| &symbol.file))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::str::FromStr;

    use acan_core::{
        Belt, CodeIndex, CodeRev, Command, Ctx, FilePath, HumanCmd, IndexedFile, IndexedSymbol,
        MachineKey, MemberId, Origin, Seq, Timestamp, WorldId, WorldRef,
    };
    use serde_json::json;

    use super::*;
    use crate::layout::PlantJson;
    use crate::scenario::{self, Target};

    const PLANT: &str = r#"{
        "machines": [
            {"id": 1, "name": "src/api", "files": [1]},
            {"id": 2, "name": "src/ui", "files": [2]}
        ],
        "belts": [{"from": 2, "to": 1}],
        "files": [
            {"id": 1, "path": "src/api/auth.ts"},
            {"id": 2, "path": "src/ui/dashboard.ts"}
        ]
    }"#;

    fn plan() -> Plan {
        let plant: PlantJson = serde_json::from_str(PLANT).expect("the plant parses");
        Plan::new(&plant)
    }

    fn machine_index(view: &View, name: &str) -> usize {
        view.machines
            .iter()
            .position(|machine| machine.name == name)
            .expect("the plan has the machine")
    }

    fn founder() -> MemberId {
        MemberId::from_str("owner").expect("a member id")
    }

    fn open_world() -> (Origin, Stamped) {
        let world = WorldRef {
            id: WorldId::from_str("demo").expect("a world id"),
            repo: "demo".to_owned(),
        };
        Origin::open(world, founder(), Timestamp::from_millis(1_000))
    }

    fn index() -> CodeIndex {
        let api = MachineKey::from_str("src/api").expect("a machine key");
        let ui = MachineKey::from_str("src/ui").expect("a machine key");
        let symbol = |name: &str| IndexedSymbol {
            name: name.to_owned(),
            bytes: 0..40,
            exported: true,
        };
        let files = BTreeMap::from([
            (
                FilePath::from_str("src/api/auth.ts").expect("a file path"),
                IndexedFile {
                    machine: api.clone(),
                    symbols: vec![symbol("login")],
                },
            ),
            (
                FilePath::from_str("src/ui/dashboard.ts").expect("a file path"),
                IndexedFile {
                    machine: ui.clone(),
                    symbols: vec![symbol("render")],
                },
            ),
        ]);
        CodeIndex {
            rev: CodeRev::from_str("demo-rev").expect("a code rev"),
            files,
            belts: BTreeSet::from([Belt { from: ui, to: api }]),
        }
    }

    /// Runs the demo script on a fresh origin. Returns the origin, the whole log, and the index.
    fn scripted() -> (Origin, Vec<Stamped>, CodeIndex) {
        let index = index();
        let (mut origin, opened) = open_world();
        let target = Target::first_symbol(&index, "Make login return a session")
            .expect("the index holds a symbol");
        let events = scenario::clash_script(
            &mut origin,
            &index,
            &founder(),
            &target,
            Timestamp::from_millis(2_000),
        )
        .expect("the script runs");
        let log = std::iter::once(opened).chain(events).collect();
        (origin, log, index)
    }

    #[test]
    fn a_new_world_shows_the_plan_and_nothing_else() {
        let (_origin, opened) = open_world();
        let view = Mirror::from_log(vec![opened])
            .expect("the log replays")
            .view(&plan());

        assert_eq!(view.machines.len(), 2);
        assert!(view.machines.iter().all(|machine| !machine.abnormal));
        assert!(view.orders.is_empty());
        assert!(view.agents.is_empty());
        assert!(view.questions.is_empty());
        assert!(view.clashes.is_empty());
        assert_eq!(view.next_seq, Seq::new(1));
    }

    #[test]
    fn the_scripted_clash_is_pinned_on_its_machine_with_both_agents() {
        let plan = plan();
        let (_origin, log, _index) = scripted();
        let view = Mirror::from_log(log).expect("the log replays").view(&plan);

        let api = machine_index(&view, "src/api");
        assert!(view.machines[api].abnormal, "the clash stands on src/api");
        assert!(!view.machines[machine_index(&view, "src/ui")].abnormal);

        let [order] = view.orders.as_slice() else {
            panic!("the script posts one order");
        };
        assert_eq!(order.machine, "src/api");
        assert_eq!(order.phase, "clashing");
        assert_eq!(order.tasks, 2);
        assert_eq!(order.at, Some(plan.machines[api].center()));

        let [first, second] = view.agents.as_slice() else {
            panic!("the script registers two agents");
        };
        assert_eq!(first.color, PALETTE[0]);
        assert_eq!(second.color, PALETTE[1]);
        for agent in [first, second] {
            assert_eq!(agent.machine.as_deref(), Some("src/api"));
            assert_eq!(agent.activity, "declared");
            assert!(agent.body.is_some());
        }
        assert_ne!(first.body, second.body, "two bodies sit side by side");

        let [clash] = view.clashes.as_slice() else {
            panic!("the overlap is one clash");
        };
        assert_eq!(clash.state, "open");
        assert_eq!(clash.kind, "shared_implementation");
        assert_eq!(clash.label, format!("{} · overlap", clash.id));
        assert!(clash.at.is_some());
        assert!(
            clash.consequence.is_none(),
            "no explain job has run on the script"
        );
        assert!(
            (2..=3).contains(&clash.options.len()),
            "a clash offers two or three settlements"
        );
        let keys: Vec<usize> = clash.options.iter().map(|option| option.key).collect();
        assert_eq!(keys, (1..=clash.options.len()).collect::<Vec<_>>());
    }

    #[test]
    fn a_settle_event_resolves_the_pin_as_it_arrives() {
        let (mut origin, log, index) = scripted();
        let plan = plan();
        let mut mirror = Mirror::from_log(log).expect("the log replays");

        let before = mirror.view(&plan);
        let [clash] = before.clashes.as_slice() else {
            panic!("the script leaves one open clash");
        };
        let option = clash
            .options
            .first()
            .expect("the clash offers a settlement");
        let payload = json!({"cmd": "settle_clash", "clash": clash.id, "choice": option.choice});
        let cmd: HumanCmd =
            serde_json::from_value(payload).expect("the page's settle payload is a human command");

        let ctx = Ctx {
            now: Timestamp::from_millis(3_000),
            index: &index,
        };
        let events = origin
            .execute(
                &ctx,
                Command::Human {
                    member: founder(),
                    cmd,
                },
            )
            .expect("the settle is accepted");
        for stamped in &events {
            mirror
                .ingest(stamped)
                .expect("a live event applies in order");
        }

        let after = mirror.view(&plan);
        let [clash] = after.clashes.as_slice() else {
            panic!("the settled clash stays listed");
        };
        assert_eq!(clash.state, "settled");
        assert!(clash.options.is_empty());
        assert!(clash.resolved.is_some());
        assert!(!after.machines[machine_index(&after, "src/api")].abnormal);
    }

    #[test]
    fn ingest_ignores_a_replayed_event_and_refuses_a_gap() {
        let (_origin, log, _index) = scripted();
        let last = log.last().cloned().expect("the log has events");
        let mut mirror = Mirror::from_log(log).expect("the log replays");
        let next = mirror.next_seq();

        mirror
            .ingest(&last)
            .expect("an event already applied is ignored");
        assert_eq!(mirror.next_seq(), next);

        let mut ahead = last;
        ahead.seq = Seq::new(next.get() + 1);
        assert!(matches!(
            mirror.ingest(&ahead),
            Err(ReplayError::SeqGap { .. })
        ));
        assert_eq!(mirror.next_seq(), next);
    }
}
