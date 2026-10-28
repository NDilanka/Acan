//! The marks the product screen draws on the plant, read from the origin's view.
//!
//! [`Marks::build`] says where each mark stands: an agent's body on its machine, the amber ring
//! round an agent that waits on the user, the clash pin, the footprints the agents tint, and the
//! flags the page labels. [`card`] builds the clash card from the view and the world, so its rows
//! follow the origin. [`Motion`] only eases marks between the positions the view gives them.

use std::collections::{BTreeMap, BTreeSet};

use acan_core::{OrderId, TaskId, World};
use acan_iso::{BODY_HEIGHT, PIN_HEIGHT, PLINTH_TOP};
use acan_render::Instance;
use acan_render::theme::{self, AGENT_SLOTS, TINT_BODY0, TINT_FOOTPRINT0, TINT_LEN, TINT_SHARED};
use serde::Serialize;

use crate::mirror::{AgentView, ClashView, View};
use crate::plan::Plan;

/// Distance between two bodies on one machine, in plan units.
pub const SEAT_STEP: f32 = 16.0;
/// Clearance a body keeps from the side of its machine, in plan units.
const EDGE_CLEAR: f32 = 6.0;
/// Height above a machine's top where its order flag anchors, in plan units.
const ORDER_LIFT: f32 = 4.0;
/// Height above a body's crown where its flag anchors, in plan units.
const FLAG_LIFT: f32 = 2.0;
/// Height above a pin's tip where its label anchors, in plan units.
const PIN_LABEL_LIFT: f32 = 1.0;
/// How far below its place a body starts its ease-in, and a pin its rise, in plan units.
const RISE: f32 = 6.0;
/// Milliseconds a body takes to ease in on arrival.
pub const ARRIVE_MS: f32 = 200.0;
/// Milliseconds a body takes to move between machines.
pub const MOVE_MS: f32 = 400.0;
/// Milliseconds a pin takes to rise in, or to sink out.
pub const PIN_MS: f32 = 250.0;
/// The dot of a declaration whose task has no agent.
const UNCLAIMED: &str = "#7a7a72";

/// The palette slot of an agent's hue: a 32-bit FNV-1a hash of its id, modulo the palette. The
/// hash is fixed, so an agent keeps its hue across reloads and builds.
#[must_use]
pub fn hue_slot(agent: &str) -> usize {
    let hash = agent.bytes().fold(0x811c_9dc5_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    });
    hash as usize % AGENT_SLOTS
}

/// The hue of a palette slot, as `#rrggbb`.
#[must_use]
pub fn hue_css(slot: usize) -> String {
    let [r, g, b, _] = theme::agent_colour(slot).map(|c| (c * 255.0).round() as u32);
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// The tint slot a body draws with: the agent's hue at full strength.
#[must_use]
pub fn body_tint(slot: usize) -> u8 {
    u8::try_from(TINT_BODY0 as usize + slot).expect("tint slots fit in a byte")
}

/// A body on a machine, standing on the plinth top.
#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub agent: String,
    pub slot: usize,
    /// The plan index of the machine it stands on.
    pub machine: usize,
    /// Where the body stands, in world units. The foot sits on this point.
    pub at: [f32; 3],
    pub activity: &'static str,
    /// Whether the agent waits on the user: its task is in an open clash, or its order asks a question.
    pub waiting: bool,
}

/// A clash the pin marks: an unsettled clash with a machine to stand on.
#[derive(Debug, Clone, PartialEq)]
pub struct Pin {
    /// The clash id, as the view writes it. It keys the pin's motion.
    pub id: String,
    /// The index of the clash in the view.
    pub clash: usize,
    /// The plan index of the machine it stands on.
    pub machine: usize,
    /// Where the pin's foot stands, in world units: the machine's centre on its top.
    pub at: [f32; 3],
}

/// What a flag's anchor follows while the marks move.
#[derive(Debug, Clone, PartialEq)]
pub enum Follow {
    /// Stays at the flag's own point.
    Still,
    /// Rides the body of the named agent.
    Body(String),
    /// Rides the pin of the named clash.
    Pin(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlagKind {
    Agent,
    Order,
    Clash,
    Question,
}

/// A label the page draws over the plant, with its anchor in world units.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Flag {
    pub kind: FlagKind,
    pub text: String,
    /// The agent's hue as `#rrggbb`, for the dot.
    pub hue: Option<String>,
    /// Whether the agent waits on the user. The flag then carries an amber dot.
    pub waiting: bool,
    /// The clash index a clash flag belongs to, so a click opens its card.
    pub clash: Option<usize>,
    #[serde(skip)]
    pub at: [f32; 3],
    #[serde(skip)]
    pub follow: Follow,
}

/// One row of the clash card.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CardRow {
    /// A declaration: the agent that holds the task, the task, and its hue.
    Declaration {
        agent: String,
        hue: String,
        text: String,
    },
    /// The witness the clash would break, in red.
    Witness { text: String },
    /// The settlement the card suggests, always the last row. The key settles with it.
    Settle {
        key: usize,
        text: String,
        suggested: bool,
    },
}

/// The clash card: its title, its rows, and what each key does.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Card {
    /// The index of the clash in the view.
    pub clash: usize,
    pub title: String,
    pub rows: Vec<CardRow>,
    /// What each key does, such as `1 narrow t-0`. Empty once the clash is resolved.
    pub keys: Vec<String>,
    /// How the clash was resolved, once it has been.
    pub resolved: Option<String>,
}

/// Every mark the view calls for, placed on the plan.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Marks {
    pub bodies: Vec<Body>,
    pub pins: Vec<Pin>,
    pub flags: Vec<Flag>,
    /// For each plan machine, the hue slots of the agents whose footprints cover it, sorted.
    pub footprints: Vec<Vec<usize>>,
}

impl Marks {
    /// Places the marks for `view` on `plan`. `tops` gives each plan machine's top height, in
    /// world units, in plan order. A pin stands on it.
    #[must_use]
    pub fn build(view: &View, plan: &Plan, tops: &[f32]) -> Self {
        let waiting = waiting_agents(view);
        let bodies = bodies(view, plan, &waiting);
        let mut footprints = vec![Vec::new(); plan.machines.len()];
        for body in &bodies {
            let slots: &mut Vec<usize> = &mut footprints[body.machine];
            if !slots.contains(&body.slot) {
                slots.push(body.slot);
                slots.sort_unstable();
            }
        }
        let pins = pins(view, plan, tops);
        let flags = flags(view, plan, tops, &bodies, &pins);
        Self {
            bodies,
            pins,
            flags,
            footprints,
        }
    }

    /// The tint slot of a plan machine's footprint: none, one agent's mix, or the shared mix.
    #[must_use]
    pub fn footprint_tint(&self, machine: usize) -> u8 {
        let tint = match self.footprints.get(machine).map(Vec::as_slice) {
            None | Some([]) => 0,
            Some([slot]) => TINT_FOOTPRINT0 + *slot as u32,
            Some(_) => TINT_SHARED,
        };
        u8::try_from(tint).expect("tint slots fit in a byte")
    }

    /// The renderer's tint table: the defaults, with the shared slot holding the mix of every
    /// hue that shares a machine.
    #[must_use]
    pub fn tint_table(&self) -> [[f32; 4]; TINT_LEN] {
        let mut table = theme::tints();
        let shared: BTreeSet<usize> = self
            .footprints
            .iter()
            .filter(|slots| slots.len() > 1)
            .flatten()
            .copied()
            .collect();
        if !shared.is_empty() {
            let count = shared.len() as f32;
            let mut mix = [0.0_f32; 4];
            for &slot in &shared {
                for (total, channel) in mix.iter_mut().zip(theme::agent_colour(slot)) {
                    *total += channel / count;
                }
            }
            table[TINT_SHARED as usize] = theme::footprint_mix(mix);
        }
        table
    }
}

/// The clash card for the clash at `index` in the view, or `None` when there is no such clash.
#[must_use]
pub fn card(view: &View, world: &World, index: usize) -> Option<Card> {
    let clash = view.clashes.get(index)?;
    let mut rows: Vec<CardRow> = [clash.a, clash.b]
        .into_iter()
        .map(|task| declaration(view, task))
        .collect();
    if let Some(witness) = &clash.breaks {
        rows.push(CardRow::Witness {
            text: format!("breaks {witness}"),
        });
    }
    let mut keys = Vec::new();
    if is_open(clash) {
        keys = clash
            .options
            .iter()
            .map(|option| format!("{} {}", option.key, option.label))
            .collect();
        let suggested = world
            .clashes()
            .find(|candidate| candidate.id == clash.id)
            .and_then(|candidate| candidate.explanation.as_ref())
            .and_then(|explanation| explanation.suggested.clone());
        let chosen = clash
            .options
            .iter()
            .position(|option| suggested.as_ref() == Some(&option.choice));
        let (option, from_explanation) = match chosen {
            Some(i) => (clash.options.get(i), true),
            None => (clash.options.first(), false),
        };
        if let Some(option) = option {
            rows.push(CardRow::Settle {
                key: option.key,
                text: option.label.clone(),
                suggested: from_explanation,
            });
        }
    }
    Some(Card {
        clash: index,
        title: clash.label.clone(),
        rows,
        keys,
        resolved: clash.resolved.clone(),
    })
}

/// A body's glide, from where it was to where the view puts it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Glide {
    from: [f32; 3],
    to: [f32; 3],
    elapsed: f32,
    duration: f32,
}

impl Glide {
    fn new(from: [f32; 3], to: [f32; 3], duration: f32) -> Self {
        Self {
            from,
            to,
            elapsed: 0.0,
            duration,
        }
    }

    fn at(&self) -> [f32; 3] {
        let t = ease(self.elapsed / self.duration);
        std::array::from_fn(|axis| self.from[axis] + (self.to[axis] - self.from[axis]) * t)
    }

    fn running(&self) -> bool {
        self.elapsed < self.duration
    }
}

/// A pin's rise in, or its sink out once the view no longer shows it.
#[derive(Debug, Clone)]
struct Rise {
    pin: Pin,
    elapsed: f32,
    leaving: bool,
}

impl Rise {
    fn at(&self) -> [f32; 3] {
        let t = ease(self.elapsed / PIN_MS);
        let lift = if self.leaving { t } else { 1.0 - t };
        [self.pin.at[0], self.pin.at[1], self.pin.at[2] - RISE * lift]
    }

    fn running(&self) -> bool {
        self.elapsed < PIN_MS
    }
}

/// The eases the marks run between the view's positions. Nothing here is read back into the view.
#[derive(Debug, Default)]
pub struct Motion {
    bodies: BTreeMap<String, Glide>,
    pins: BTreeMap<String, Rise>,
}

impl Motion {
    /// Points each ease at the marks the view now gives. A body that moves glides from where it
    /// is, and a pin that leaves sinks out.
    pub fn sync(&mut self, marks: &Marks) {
        for body in &marks.bodies {
            let glide = match self.bodies.get(&body.agent) {
                Some(glide) if glide.to == body.at => continue,
                Some(glide) => Glide::new(glide.at(), body.at, MOVE_MS),
                None => Glide::new(
                    [body.at[0], body.at[1], body.at[2] - RISE],
                    body.at,
                    ARRIVE_MS,
                ),
            };
            self.bodies.insert(body.agent.clone(), glide);
        }
        let live: BTreeSet<&str> = marks
            .bodies
            .iter()
            .map(|body| body.agent.as_str())
            .collect();
        self.bodies.retain(|agent, _| live.contains(agent.as_str()));

        for pin in &marks.pins {
            self.pins
                .entry(pin.id.clone())
                .and_modify(|rise| {
                    rise.leaving = false;
                    rise.pin = pin.clone();
                })
                .or_insert_with(|| Rise {
                    pin: pin.clone(),
                    elapsed: 0.0,
                    leaving: false,
                });
        }
        for (id, rise) in &mut self.pins {
            let shown = marks.pins.iter().any(|pin| &pin.id == id);
            if !shown && !rise.leaving {
                rise.leaving = true;
                rise.elapsed = 0.0;
            }
        }
        self.pins.retain(|_, rise| !rise.leaving || rise.running());
    }

    /// Moves every ease on by `dt_ms`. Returns true while any of them is still running.
    pub fn advance(&mut self, dt_ms: f32) -> bool {
        let dt = dt_ms.max(0.0);
        for glide in self.bodies.values_mut() {
            glide.elapsed += dt;
        }
        for rise in self.pins.values_mut() {
            rise.elapsed += dt;
        }
        let bodies = self.bodies.values().any(Glide::running);
        let pins = self.pins.values().any(Rise::running);
        bodies || pins
    }

    /// Where an agent's body stands now.
    #[must_use]
    pub fn body_at(&self, agent: &str) -> Option<[f32; 3]> {
        self.bodies.get(agent).map(Glide::at)
    }

    /// Where a clash's pin stands now. Pins that are sinking out are included until they are gone.
    #[must_use]
    pub fn pin_at(&self, clash: &str) -> Option<[f32; 3]> {
        self.pins.get(clash).map(Rise::at)
    }

    /// Where a flag's anchor stands now. A flag that follows a body or a pin rides it.
    #[must_use]
    pub fn flag_at(&self, flag: &Flag) -> [f32; 3] {
        match &flag.follow {
            Follow::Still => flag.at,
            Follow::Body(agent) => self
                .body_at(agent)
                .map_or(flag.at, |[x, y, z]| [x, y, z + body_flag_lift()]),
            Follow::Pin(clash) => self
                .pin_at(clash)
                .map_or(flag.at, |[x, y, z]| [x, y, z + pin_flag_lift()]),
        }
    }

    /// Every pin on the plant now, with its height. A pin that has sunk out is gone.
    #[must_use]
    pub fn pins_now(&self) -> Vec<[f32; 3]> {
        self.pins
            .values()
            .filter(|rise| !rise.leaving || rise.running())
            .map(Rise::at)
            .collect()
    }
}

/// The model kinds of the marks, after the machine kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkKinds {
    pub body: u16,
    pub ring: u16,
    pub pin: u16,
}

/// The instances of the marks now: every body, the ring round each waiting body, and every pin.
#[must_use]
pub fn mark_instances(motion: &Motion, marks: &Marks, kinds: MarkKinds) -> Vec<Instance> {
    let mut out = Vec::new();
    for body in &marks.bodies {
        let Some(at) = motion.body_at(&body.agent) else {
            continue;
        };
        out.push(Instance {
            offset: at,
            kind: kinds.body,
            tint: body_tint(body.slot),
        });
        if body.waiting {
            out.push(Instance {
                offset: [at[0], at[1], PLINTH_TOP],
                kind: kinds.ring,
                tint: 0,
            });
        }
    }
    for at in motion.pins_now() {
        out.push(Instance {
            offset: at,
            kind: kinds.pin,
            tint: 0,
        });
    }
    out
}

/// The height a flag's anchor sits above its body's crown.
#[must_use]
pub fn body_flag_lift() -> f32 {
    BODY_HEIGHT + FLAG_LIFT
}

/// The height a flag's anchor sits above its pin's tip.
#[must_use]
pub fn pin_flag_lift() -> f32 {
    PIN_HEIGHT + PIN_LABEL_LIFT
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn is_open(clash: &ClashView) -> bool {
    matches!(clash.state, "open" | "proposed")
}

/// The agents that wait on the user: their task is in an open clash, or their order asks a question.
fn waiting_agents(view: &View) -> BTreeSet<String> {
    let open_tasks: Vec<TaskId> = view
        .clashes
        .iter()
        .filter(|clash| is_open(clash))
        .flat_map(|clash| [clash.a, clash.b])
        .collect();
    let asked: Vec<OrderId> = view
        .questions
        .iter()
        .map(|question| question.order)
        .collect();
    view.agents
        .iter()
        .filter(|agent| {
            agent.task.is_some_and(|task| {
                open_tasks.contains(&task)
                    || view
                        .tasks
                        .iter()
                        .find(|candidate| candidate.id == task)
                        .is_some_and(|candidate| asked.contains(&candidate.order))
            })
        })
        .map(|agent| agent.id.to_string())
        .collect()
}

fn bodies(view: &View, plan: &Plan, waiting: &BTreeSet<String>) -> Vec<Body> {
    let homes: Vec<Option<usize>> = view
        .agents
        .iter()
        .map(|agent| {
            agent
                .machine
                .as_deref()
                .and_then(|name| plan.index_of(name))
        })
        .collect();
    let mut crowd: BTreeMap<usize, usize> = BTreeMap::new();
    for home in homes.iter().flatten() {
        *crowd.entry(*home).or_insert(0) += 1;
    }
    let mut seated: BTreeMap<usize, usize> = BTreeMap::new();
    let mut out = Vec::new();
    for (agent, home) in view.agents.iter().zip(homes) {
        let Some(machine) = home else {
            continue;
        };
        let seat = seated.entry(machine).or_insert(0);
        let offset = (*seat as f32 - (crowd[&machine] as f32 - 1.0) * 0.5) * SEAT_STEP;
        *seat += 1;
        out.push(body(agent, machine, offset, plan, waiting));
    }
    out
}

fn body(
    agent: &AgentView,
    machine: usize,
    offset: f32,
    plan: &Plan,
    waiting: &BTreeSet<String>,
) -> Body {
    let id = agent.id.to_string();
    let centre = plan.machines[machine].center();
    let half_width = plan.machines[machine].size[0] * 0.5;
    let limit = (half_width - EDGE_CLEAR).max(0.0);
    Body {
        slot: hue_slot(&id),
        waiting: waiting.contains(&id),
        agent: id,
        machine,
        at: [
            centre[0] + offset.clamp(-limit, limit),
            centre[1],
            PLINTH_TOP,
        ],
        activity: agent.activity,
    }
}

fn pins(view: &View, plan: &Plan, tops: &[f32]) -> Vec<Pin> {
    view.clashes
        .iter()
        .enumerate()
        .filter(|(_, clash)| is_open(clash))
        .filter_map(|(index, clash)| {
            let machine = nearest(plan, clash.at?)?;
            let centre = plan.machines[machine].center();
            Some(Pin {
                id: clash.id.to_string(),
                clash: index,
                machine,
                at: [
                    centre[0],
                    centre[1],
                    tops.get(machine).copied().unwrap_or(0.0),
                ],
            })
        })
        .collect()
}

fn flags(view: &View, plan: &Plan, tops: &[f32], bodies: &[Body], pins: &[Pin]) -> Vec<Flag> {
    let mut out = Vec::new();
    for body in bodies {
        out.push(Flag {
            kind: FlagKind::Agent,
            text: format!("{} · {}", body.agent, body.activity),
            hue: Some(hue_css(body.slot)),
            waiting: body.waiting,
            clash: None,
            at: [body.at[0], body.at[1], body.at[2] + body_flag_lift()],
            follow: Follow::Body(body.agent.clone()),
        });
    }
    // One flag for the order in progress, the same one the page's order strip names.
    let active = view
        .orders
        .iter()
        .find(|order| !matches!(order.phase, "landed" | "cancelled"));
    if let Some(order) = active
        && let Some(machine) = plan.index_of(&order.machine)
    {
        let centre = plan.machines[machine].center();
        out.push(Flag {
            kind: FlagKind::Order,
            text: format!(
                "{} · {} · {}",
                order.id,
                short_name(&order.machine),
                order.phase.replace('_', " ")
            ),
            hue: None,
            waiting: false,
            clash: None,
            at: [
                centre[0],
                centre[1],
                tops.get(machine).copied().unwrap_or(0.0) + ORDER_LIFT,
            ],
            follow: Follow::Still,
        });
    }
    for pin in pins {
        out.push(Flag {
            kind: FlagKind::Clash,
            text: view.clashes[pin.clash].label.clone(),
            hue: None,
            waiting: false,
            clash: Some(pin.clash),
            at: [pin.at[0], pin.at[1], pin.at[2] + pin_flag_lift()],
            follow: Follow::Pin(pin.id.clone()),
        });
    }
    for question in &view.questions {
        let Some(at) = question.at else {
            continue;
        };
        out.push(Flag {
            kind: FlagKind::Question,
            text: question.text.clone(),
            hue: None,
            waiting: true,
            clash: None,
            at: [at[0], at[1], PLINTH_TOP + body_flag_lift()],
            follow: Follow::Still,
        });
    }
    out
}

fn declaration(view: &View, task: TaskId) -> CardRow {
    let found = view.tasks.iter().find(|candidate| candidate.id == task);
    let title = found.map_or_else(|| task.to_string(), |candidate| candidate.title.clone());
    match found.and_then(|candidate| candidate.agent.clone()) {
        Some(agent) => {
            let id = agent.to_string();
            CardRow::Declaration {
                hue: hue_css(hue_slot(&id)),
                agent: id,
                text: format!("{task} · {title}"),
            }
        }
        None => CardRow::Declaration {
            agent: "unclaimed".to_owned(),
            hue: UNCLAIMED.to_owned(),
            text: format!("{task} · {title}"),
        },
    }
}

fn nearest(plan: &Plan, at: [f32; 2]) -> Option<usize> {
    let distance = |i: usize| {
        let c = plan.machines[i].center();
        (c[0] - at[0]).powi(2) + (c[1] - at[1]).powi(2)
    };
    (0..plan.machines.len()).min_by(|&a, &b| distance(a).total_cmp(&distance(b)))
}

fn short_name(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::str::FromStr;

    use acan_core::{
        Belt, CodeIndex, CodeRev, Command, Ctx, FilePath, HumanCmd, IndexedFile, IndexedSymbol,
        MachineKey, MemberId, Origin, Stamped, Timestamp, WorldId, WorldRef,
    };
    use serde_json::json;

    use super::*;
    use crate::mirror::Mirror;
    use crate::plant::from_scan_json;
    use crate::scenario::{self, Target};

    const PLANT: &str = r#"{
        "root": "/repo",
        "machines": [
            {"id": 1, "name": "src/api", "dir": "src/api", "files": [1]},
            {"id": 2, "name": "src/ui", "dir": "src/ui", "files": [2]}
        ],
        "belts": [{"from": 2, "to": 1, "imports": 1}],
        "files": [
            {"id": 1, "path": "src/api/auth.ts"},
            {"id": 2, "path": "src/ui/dashboard.ts"}
        ]
    }"#;

    fn plan() -> Plan {
        let plant = from_scan_json(PLANT).expect("the plant parses");
        Plan::new(&plant, |name| {
            if name == "src/api" {
                [0.0, 0.0, 96.0, 86.0]
            } else {
                [200.0, 0.0, 296.0, 86.0]
            }
        })
    }

    /// Every machine's top, as the scene gives it: the plinth top for each.
    fn tops(plan: &Plan) -> Vec<f32> {
        vec![PLINTH_TOP; plan.machines.len()]
    }

    fn founder() -> MemberId {
        MemberId::from_str("owner").expect("a member id")
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

    /// Runs the demo script on a fresh origin: one order, two agents on src/api, one clash.
    fn scripted() -> (Origin, Vec<Stamped>, CodeIndex) {
        let index = index();
        let world = WorldRef {
            id: WorldId::from_str("demo").expect("a world id"),
            repo: "demo".to_owned(),
        };
        let (mut origin, opened) = Origin::open(world, founder(), Timestamp::from_millis(1_000));
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

    fn machine(plan: &Plan, name: &str) -> usize {
        plan.index_of(name).expect("the plan has the machine")
    }

    #[test]
    fn a_hue_depends_only_on_the_agent_id() {
        let a = hue_slot("agent-a");
        assert_eq!(
            a,
            hue_slot("agent-a"),
            "the same id always gets the same hue"
        );
        assert!(a < AGENT_SLOTS);
        assert_eq!(hue_css(a).len(), 7);
        assert!(hue_css(a).starts_with('#'));
        assert_ne!(
            hue_slot("agent-a"),
            hue_slot("agent-b"),
            "the two scenario agents get different hues"
        );
    }

    #[test]
    fn two_agents_on_one_machine_stand_apart_on_its_top() {
        let plan = plan();
        let (_origin, log, _index) = scripted();
        let view = Mirror::from_log(log).expect("the log replays").view(&plan);
        let marks = Marks::build(&view, &plan, &tops(&plan));

        let api = machine(&plan, "src/api");
        let on_api: Vec<&Body> = marks
            .bodies
            .iter()
            .filter(|body| body.machine == api)
            .collect();
        let [first, second] = on_api.as_slice() else {
            panic!("both agents stand on src/api");
        };
        assert!(
            (first.at[0] - second.at[0]).abs() >= SEAT_STEP,
            "the bodies are a seat apart"
        );
        assert_eq!(first.at[1], second.at[1]);
        let centre = plan.machines[api].center();
        for body in [first, second] {
            assert!(
                (body.at[0] - centre[0]).abs() <= plan.machines[api].size[0] * 0.5,
                "a body stands on its machine"
            );
            assert_eq!(body.at[2], PLINTH_TOP);
        }
        assert_ne!(first.slot, second.slot, "the two agents have distinct hues");
    }

    #[test]
    fn the_scripted_clash_gets_a_pin_on_its_machine_with_its_label() {
        let plan = plan();
        let (_origin, log, _index) = scripted();
        let view = Mirror::from_log(log).expect("the log replays").view(&plan);
        let marks = Marks::build(&view, &plan, &tops(&plan));

        let api = machine(&plan, "src/api");
        let [pin] = marks.pins.as_slice() else {
            panic!("the open clash has one pin");
        };
        assert_eq!(pin.machine, api);
        assert_eq!(pin.clash, 0);
        assert_eq!(pin.id, view.clashes[0].id.to_string());
        assert_eq!(pin.at[2], PLINTH_TOP);

        let clash_flag = marks
            .flags
            .iter()
            .find(|flag| flag.kind == FlagKind::Clash)
            .expect("the pin has a flag");
        assert_eq!(clash_flag.text, view.clashes[0].label);
        assert_eq!(clash_flag.clash, Some(0));

        assert_eq!(
            marks.bodies.iter().filter(|body| body.waiting).count(),
            2,
            "both agents wait on the open clash"
        );
        assert_eq!(
            marks.footprint_tint(api),
            u8::try_from(TINT_SHARED).expect("fits")
        );
    }

    #[test]
    fn the_card_rows_come_from_the_view() {
        let plan = plan();
        let (_origin, log, _index) = scripted();
        let mirror = Mirror::from_log(log).expect("the log replays");
        let view = mirror.view(&plan);

        let card = card(&view, mirror.world(), 0).expect("clash 0 has a card");
        assert_eq!(card.title, view.clashes[0].label);
        let [first, second, settle] = card.rows.as_slice() else {
            panic!("two declarations and the settle row, with no witness");
        };
        assert!(matches!(first, CardRow::Declaration { hue, .. } if hue.starts_with('#')));
        assert!(matches!(second, CardRow::Declaration { .. }));
        assert!(matches!(
            settle,
            CardRow::Settle {
                key: 1,
                suggested: false,
                ..
            }
        ));
        assert_eq!(card.keys.len(), view.clashes[0].options.len());
        assert!(card.keys[0].starts_with("1 "));
        assert_eq!(card.resolved, None);
    }

    #[test]
    fn a_broken_witness_is_a_row_before_the_settle() {
        let plan = plan();
        let (_origin, log, _index) = scripted();
        let mirror = Mirror::from_log(log).expect("the log replays");
        let mut view = mirror.view(&plan);
        view.clashes[0].breaks = Some("w-2".to_owned());

        let card = card(&view, mirror.world(), 0).expect("clash 0 has a card");
        let [_, _, witness, settle] = card.rows.as_slice() else {
            panic!("two declarations, the witness and the settle row");
        };
        assert_eq!(
            witness,
            &CardRow::Witness {
                text: "breaks w-2".to_owned()
            }
        );
        assert!(matches!(settle, CardRow::Settle { .. }));
    }

    #[test]
    fn a_settled_clash_loses_its_pin_once_the_pin_has_sunk() {
        let plan = plan();
        let tops = tops(&plan);
        let (mut origin, log, index) = scripted();
        let mut mirror = Mirror::from_log(log).expect("the log replays");

        let before = mirror.view(&plan);
        let mut motion = Motion::default();
        motion.sync(&Marks::build(&before, &plan, &tops));
        assert_eq!(motion.pins_now().len(), 1, "the open clash shows its pin");

        let clash = before.clashes[0].id;
        let option = before.clashes[0]
            .options
            .first()
            .expect("the clash offers a settlement");
        let payload = json!({"cmd": "settle_clash", "clash": clash, "choice": option.choice});
        let cmd: HumanCmd = serde_json::from_value(payload).expect("a human command");
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
        let marks = Marks::build(&after, &plan, &tops);
        assert!(marks.pins.is_empty(), "a settled clash has no pin");
        motion.sync(&marks);
        assert_eq!(
            motion.pins_now().len(),
            1,
            "the pin is still there while it sinks out"
        );
        motion.advance(PIN_MS);
        assert!(
            motion.pins_now().is_empty(),
            "the pin is gone after the ease"
        );
    }

    #[test]
    fn a_waiting_body_gets_a_ring_and_each_pin_an_instance() {
        let plan = plan();
        let (_origin, log, _index) = scripted();
        let view = Mirror::from_log(log).expect("the log replays").view(&plan);
        let marks = Marks::build(&view, &plan, &tops(&plan));
        let mut motion = Motion::default();
        motion.sync(&marks);
        motion.advance(ARRIVE_MS);

        let kinds = MarkKinds {
            body: 40,
            ring: 41,
            pin: 42,
        };
        let instances = mark_instances(&motion, &marks, kinds);
        let count = |kind: u16| instances.iter().filter(|i| i.kind == kind).count();
        assert_eq!(count(kinds.body), 2);
        assert_eq!(count(kinds.ring), 2, "each waiting agent has a ring");
        assert_eq!(count(kinds.pin), 1);
    }
}
