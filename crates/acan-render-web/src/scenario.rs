//! The product screen's demo script: one order, two agents, and an overlapping declaration.
//!
//! The server has no agent route on this branch, so the demo log is written before `acan serve`
//! starts. Every event comes from the origin's own `execute`, so the log replays like any other.

use std::str::FromStr;

use acan_core::{
    AgentCmd, AgentId, Check, ClashId, CodeIndex, Command, Ctx, DeclarationInput, Event, FilePath,
    HumanCmd, Judgement, Locus, MachineKey, MemberId, OrderId, Origin, ReadbackId, ReadbackVerdict,
    Rejection, Stamped, SymbolRef, SystemCmd, TaskId, TaskSpec, TaskState, Timestamp, WitnessSpec,
};

/// The two agents the script registers, in registration order.
pub const AGENTS: [&str; 2] = ["agent-a", "agent-b"];

/// Where the script pins its order, and the symbol both agents then declare.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The machine the order is pinned on.
    pub machine: MachineKey,
    /// The file that holds the shared symbol.
    pub file: FilePath,
    /// The symbol both tasks declare.
    pub symbol: String,
    /// What the human asks for.
    pub sentence: String,
}

impl Target {
    /// The first file in path order that holds a symbol, with that symbol.
    ///
    /// Returns `None` when no file in the index holds a symbol.
    #[must_use]
    pub fn first_symbol(index: &CodeIndex, sentence: &str) -> Option<Self> {
        index.files.iter().find_map(|(path, file)| {
            let symbol = file.symbols.first()?;
            Some(Self {
                machine: file.machine.clone(),
                file: path.clone(),
                symbol: symbol.name.clone(),
                sentence: sentence.to_owned(),
            })
        })
    }
}

/// Runs the script on `origin` and returns every event it accepted, in order.
///
/// The order is posted by `founder`, split into two tasks, read back by both agents, reviewed
/// as matching, claimed, and then both agents declare `target.symbol`. The second declaration
/// leaves a blocking clash open, which [`clash_id`] returns.
///
/// # Errors
///
/// Returns the origin's [`Rejection`] for the first command it refuses.
pub fn clash_script(
    origin: &mut Origin,
    index: &CodeIndex,
    founder: &MemberId,
    target: &Target,
    now: Timestamp,
) -> Result<Vec<Stamped>, Rejection> {
    let ctx = Ctx { now, index };
    let mut log = Vec::new();
    let mut run = |origin: &mut Origin, cmd: Command| -> Result<Vec<Stamped>, Rejection> {
        let events = origin.execute(&ctx, cmd)?;
        log.extend(events.iter().cloned());
        Ok(events)
    };

    let posted = run(
        origin,
        Command::Human {
            member: founder.clone(),
            cmd: HumanCmd::PostOrder {
                machine: target.machine.clone(),
                sentence: target.sentence.clone(),
                witnesses: vec![WitnessSpec {
                    check: Check::Test {
                        command: "npm test".to_owned(),
                    },
                    reason: "the existing tests keep passing".to_owned(),
                }],
            },
        },
    )?;
    let order = posted
        .iter()
        .find_map(|stamped| match &stamped.event {
            Event::OrderPosted { order, .. } => Some(*order),
            _ => None,
        })
        .ok_or_else(|| invalid("the order post"))?;

    let rev = origin
        .world()
        .order(order)
        .map(|o| o.rev)
        .ok_or_else(|| unknown("order", order))?;
    let tasks = vec![
        TaskSpec {
            title: format!("Change {} for the order", target.symbol),
            brief: format!("Change {} and keep the tests green.", target.symbol),
            scope: vec![target.machine.clone()],
        },
        TaskSpec {
            title: format!("Change {} for the other caller", target.symbol),
            brief: format!("Change {} for the second caller.", target.symbol),
            scope: vec![target.machine.clone()],
        },
    ];
    run(
        origin,
        Command::System(SystemCmd::SplitReady { order, rev, tasks }),
    )?;
    let task_ids: Vec<TaskId> = origin.world().tasks_of(order).map(|task| task.id).collect();

    let agents = AGENTS
        .iter()
        .map(|name| AgentId::from_str(name).map_err(|_| invalid("an agent id")))
        .collect::<Result<Vec<_>, _>>()?;
    for agent in &agents {
        run(
            origin,
            Command::Human {
                member: founder.clone(),
                cmd: HumanCmd::RegisterAgent {
                    agent: agent.clone(),
                },
            },
        )?;
    }

    let mut readbacks: Vec<ReadbackId> = Vec::new();
    for (agent, task) in agents.iter().zip(&task_ids) {
        let events = run(
            origin,
            Command::Agent {
                agent: agent.clone(),
                cmd: AgentCmd::PostReadback {
                    task: *task,
                    summary: format!("Change {} and run the tests.", target.symbol),
                    touches: vec![Locus::Machine {
                        machine: target.machine.clone(),
                    }],
                },
            },
        )?;
        readbacks.extend(events.iter().filter_map(|stamped| match &stamped.event {
            Event::ReadbackPosted { readback } => Some(readback.id),
            _ => None,
        }));
    }

    let witnesses = origin
        .world()
        .order(order)
        .map(|o| o.witnesses.clone())
        .ok_or_else(|| unknown("order", order))?;
    let verdicts: Vec<(ReadbackId, ReadbackVerdict)> = readbacks
        .into_iter()
        .map(|readback| {
            let per_witness = witnesses
                .iter()
                .map(|witness| (*witness, Judgement::Match))
                .collect();
            (readback, ReadbackVerdict { rev, per_witness })
        })
        .collect();
    run(
        origin,
        Command::System(SystemCmd::ReadbacksReviewed {
            order,
            rev,
            verdicts,
            question: None,
        }),
    )?;

    for (agent, task) in agents.iter().zip(&task_ids) {
        run(
            origin,
            Command::Agent {
                agent: agent.clone(),
                cmd: AgentCmd::Claim { task: *task },
            },
        )?;
    }

    let symbol = SymbolRef {
        file: target.file.clone(),
        name: target.symbol.clone(),
    };
    for (agent, task) in agents.iter().zip(&task_ids) {
        let claimed = origin.world().task(*task).map(|t| t.state.clone());
        let fence = match claimed {
            Some(TaskState::Claimed { claim, .. }) => claim.fence(*task),
            _ => return Err(invalid("a claim")),
        };
        run(
            origin,
            Command::Agent {
                agent: agent.clone(),
                cmd: AgentCmd::Declare {
                    fence,
                    footprint: DeclarationInput {
                        symbols: vec![symbol.clone()],
                        ..DeclarationInput::default()
                    },
                },
            },
        )?;
    }

    Ok(log)
}

/// The clash the script left open: the first one the origin recorded.
#[must_use]
pub fn clash_id(origin: &Origin) -> Option<ClashId> {
    origin.world().clashes().next().map(|clash| clash.id)
}

/// The order the script posted: the first one the origin recorded.
#[must_use]
pub fn order_id(origin: &Origin) -> Option<OrderId> {
    origin.world().orders().next().map(|order| order.id)
}

fn unknown(what: &'static str, id: impl std::fmt::Display) -> Rejection {
    Rejection::Unknown {
        what: what.into(),
        id: id.to_string(),
    }
}

fn invalid(what: &'static str) -> Rejection {
    Rejection::Invalid {
        why: format!("{what} is not valid"),
    }
}
