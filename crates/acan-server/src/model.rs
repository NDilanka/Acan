//! Prompts, reply shapes, and the mapping from a model reply to the commands the origin accepts.
//!
//! Ids cross the model boundary in their display form (`t-3`, `w-1`). Counter ids serialize as
//! bare numbers, so replies are parsed from these strings.

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use acan_core::{
    Clash, ClashId, CodeIndex, Explanation, Judgement, Locus, MachineKey, OrderId, OrderRev,
    ParseIdError, QuestionDraft, ReadbackId, ReadbackVerdict, Settlement, Stage, SystemCmd, Task,
    TaskId, TaskSpec, TaskState, Witness, WitnessId, World,
};
use acan_llm::{ChatRequest, Message};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

pub const SPLIT_PROMPT: &str = "You split one order into tasks for AI coding workers. Reply with one JSON object with a tasks list of two to four tasks. Each task has a title of a few words, a brief that a worker can start from without the order, and a scope key holding a list of the machines the task changes, even when there is one machine. Copy every scope name exactly from the machines list in the context. Split by intent: each task is one feature or concern that one worker can finish alone. Tasks may touch shared code when their feature needs it, because Acan detects overlapping changes and resolves them. Every witness must still hold once all the tasks have landed.";

pub const REVIEW_PROMPT: &str = "You review each worker readback against the order's witnesses. Reply with one JSON object with two keys. readbacks is a list with one entry per readback in the context. Each entry has a readback key, holding the readback id, and a witnesses key: a list with one entry per witness of the order. Each witness entry has a witness key, holding the witness id, and a judgement key. A judgement is an object with a judgement key, which is match when the readback respects the witness, disagrees when it contradicts the witness, or unclear when it does not say enough. Give a reason key for every judgement that is not match. Judge each readback on its own. question is null when every judgement is match. Otherwise write one only when the readbacks disagree about what the order means; then it has text and exactly two options.";

pub const EXPLAIN_PROMPT: &str = "You explain a clash between two tasks that declared overlapping changes. Reply with one JSON object with three keys. consequence is one or two plain sentences on what goes wrong if both proceed, naming the declared files and symbols the tasks share. breaks is the id of the witness that would break, such as w-2, or null when none would. suggested is one choice copied exactly from the menu, or null when no choice fits. Never suggest a choice that is not on the menu.";

/// Which job a model request was built for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ask {
    Split,
    Review,
    Explain,
}

/// The job `request` was built for, read from its system prompt.
pub(crate) fn ask_of(request: &ChatRequest) -> Option<Ask> {
    request.messages.iter().find_map(|message| match message {
        Message::System { content } => match content.as_str() {
            SPLIT_PROMPT => Some(Ask::Split),
            REVIEW_PROMPT => Some(Ask::Review),
            EXPLAIN_PROMPT => Some(Ask::Explain),
            _ => None,
        },
        _ => None,
    })
}

/// Why a model reply cannot become a command.
#[derive(Debug, thiserror::Error)]
pub enum ReplyError {
    /// The reply names an id or machine that is not valid.
    #[error("reply names an invalid id: {0}")]
    Id(#[from] ParseIdError),
    /// The reply names a machine that is not on the plant.
    #[error("reply names machine {0:?}, which is not on the plant")]
    UnknownMachine(String),
    /// A split reply has no tasks, so the order would have nothing to do.
    #[error("split reply has no tasks")]
    NoTasks,
    /// A review question must offer exactly two answers.
    #[error("review question needs exactly two options, got {0}")]
    Options(usize),
}

fn request(system: &str, context: &Value) -> ChatRequest {
    ChatRequest {
        messages: vec![Message::system(system), Message::user(context.to_string())],
        ..ChatRequest::default()
    }
}

fn witness_json(witness: &Witness) -> Value {
    json!({
        "id": witness.id.to_string(),
        "kind": witness.kind,
        "reason": witness.reason,
    })
}

fn witnesses_json(world: &World, ids: &[WitnessId]) -> Vec<Value> {
    ids.iter()
        .filter_map(|id| world.witness(*id))
        .map(witness_json)
        .collect()
}

fn task_json(task: &Task) -> Value {
    json!({
        "id": task.id.to_string(),
        "title": task.spec.title,
        "brief": task.spec.brief,
        "scope": task.spec.scope,
    })
}

/// A task with the footprint it declared. A clash only exists between declared tasks, so the
/// footprint is null only for a task that has since lost its declaration.
fn declared_task_json(task: &Task) -> Value {
    let declared = match &task.state {
        TaskState::Claimed {
            stage: Stage::Declared { decl } | Stage::Building { decl } | Stage::Gating { decl, .. },
            ..
        } => json!({
            "files": decl.footprint.files,
            "symbols": decl.footprint.symbols,
            "interfaces": decl.footprint.interfaces,
            "inserts": decl.footprint.inserts,
        }),
        _ => Value::Null,
    };
    let mut value = task_json(task);
    value["declared"] = declared;
    value
}

fn settlement_json(choice: &Settlement) -> Value {
    match choice {
        Settlement::Narrow { task } => json!({ "narrow": { "task": task.to_string() } }),
        Settlement::Serialize { first, then } => json!({
            "serialize": { "first": first.to_string(), "then": then.to_string() }
        }),
        Settlement::SplitAtContract { owner, contract } => json!({
            "split_at_contract": { "owner": owner.to_string(), "contract": contract }
        }),
    }
}

/// Every machine the code index names, in name order: the owner of each file and both ends of
/// each belt. The split prompt lists these, and a split may name only these.
#[must_use]
pub fn plant_machines(index: &CodeIndex) -> Vec<MachineKey> {
    let mut machines: BTreeSet<&MachineKey> =
        index.files.values().map(|file| &file.machine).collect();
    for belt in &index.belts {
        machines.insert(&belt.from);
        machines.insert(&belt.to);
    }
    machines.into_iter().cloned().collect()
}

/// The split prompt for `order`, or `None` when the order is gone. `machines` is the plant's
/// machine list, from [`plant_machines`].
#[must_use]
pub fn split_request(
    world: &World,
    order: OrderId,
    machines: &[MachineKey],
) -> Option<ChatRequest> {
    let order = world.order(order)?;
    Some(request(
        SPLIT_PROMPT,
        &json!({
            "order": order.sentence,
            "machine": order.machine,
            "machines": machines,
            "witnesses": witnesses_json(world, &order.witnesses),
        }),
    ))
}

/// The review prompt for `readbacks` of `order`, or `None` when the order is gone.
#[must_use]
pub fn review_request(
    world: &World,
    order: OrderId,
    readbacks: &BTreeSet<ReadbackId>,
) -> Option<ChatRequest> {
    let order = world.order(order)?;
    let reviewed: Vec<Value> = readbacks
        .iter()
        .filter_map(|id| world.readback(*id))
        .filter_map(|readback| {
            let task = world.task(readback.task)?;
            Some(json!({
                "readback": readback.id.to_string(),
                "task": task_json(task),
                "summary": readback.summary,
                "touches": readback.touches,
            }))
        })
        .collect();
    Some(request(
        REVIEW_PROMPT,
        &json!({
            "order": order.sentence,
            "witnesses": witnesses_json(world, &order.witnesses),
            "readbacks": reviewed,
        }),
    ))
}

/// The explain prompt for `clash`, or `None` when the clash or a party to it is gone. It carries
/// both tasks with their declared footprints, the order's witnesses, and the menu.
#[must_use]
pub fn explain_request(world: &World, clash: ClashId) -> Option<ChatRequest> {
    let clash: &Clash = world.clash(clash)?;
    let first = world.task(clash.key.a())?;
    let second = world.task(clash.key.b())?;
    let order = world.order(first.order)?;
    let menu: Vec<Value> = clash.menu().iter().map(settlement_json).collect();
    Some(request(
        EXPLAIN_PROMPT,
        &json!({
            "clash": clash.id.to_string(),
            "kind": clash.kind,
            "at": clash.key.at(),
            "order": order.sentence,
            "witnesses": witnesses_json(world, &order.witnesses),
            "tasks": [declared_task_json(first), declared_task_json(second)],
            "menu": menu,
        }),
    ))
}

/// A split reply: the tasks the foreman cut the order into.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SplitReply {
    /// Two to four tasks, each one feature or concern that one worker can finish alone. Tasks may
    /// touch shared code when their feature needs it: an overlap becomes a clash, not a refusal.
    pub tasks: Vec<SplitTask>,
}

/// One task in a split reply.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SplitTask {
    /// A short title of a few words.
    pub title: String,
    /// What the worker should do, complete enough to start without the order.
    pub brief: String,
    /// The machines this task changes. Copy each name exactly from the machines list in the
    /// context.
    pub scope: Vec<String>,
}

/// Maps a split reply to the `SplitReady` command for `order` at `rev`. Every scope name must be
/// one of `machines`.
///
/// # Errors
///
/// Returns [`ReplyError`] when the reply has no tasks, or names a machine that is not on the
/// plant.
pub fn split_result(
    order: OrderId,
    rev: OrderRev,
    machines: &[MachineKey],
    reply: SplitReply,
) -> Result<SystemCmd, ReplyError> {
    if reply.tasks.is_empty() {
        return Err(ReplyError::NoTasks);
    }
    let tasks = reply
        .tasks
        .into_iter()
        .map(|task| task_spec(task, machines))
        .collect::<Result<Vec<_>, ReplyError>>()?;
    Ok(SystemCmd::SplitReady { order, rev, tasks })
}

fn task_spec(task: SplitTask, machines: &[MachineKey]) -> Result<TaskSpec, ReplyError> {
    let scope = task
        .scope
        .iter()
        .map(|name| -> Result<MachineKey, ReplyError> {
            let machine = MachineKey::from_str(name)?;
            if machines.contains(&machine) {
                Ok(machine)
            } else {
                Err(ReplyError::UnknownMachine(name.clone()))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TaskSpec {
        title: task.title,
        brief: task.brief,
        scope,
    })
}

/// A review reply: one entry per readback the foreman judged, and at most one question.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReviewReply {
    /// One entry per readback in the context.
    pub readbacks: Vec<ReviewedReadback>,
    /// A question for a human, only when the readbacks disagree about what the order means. Null
    /// otherwise.
    pub question: Option<ReviewQuestion>,
}

/// The judgements for one readback.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReviewedReadback {
    /// The readback id from the context, such as `r-4`.
    pub readback: String,
    /// One judgement per witness of the order, each with its witness id.
    pub witnesses: Vec<WitnessJudgement>,
}

/// The judgement of one witness.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WitnessJudgement {
    /// The witness id from the context, such as `w-2`.
    pub witness: String,
    /// The judgement.
    pub judgement: JudgementReply,
}

/// A judgement as the model writes it: `{"judgement": "match"}`, or `disagrees` or `unclear`
/// with a `reason`. The same tag as the origin's judgement.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(tag = "judgement", rename_all = "snake_case")]
pub enum JudgementReply {
    /// The readback respects the witness.
    Match,
    /// The readback contradicts the witness.
    Disagrees {
        /// Why the readback contradicts the witness, in one sentence.
        reason: String,
    },
    /// The readback does not say enough to tell whether it respects the witness.
    Unclear {
        /// What is missing from the readback, in one sentence.
        reason: String,
    },
}

/// A question the foreman drafts for a human.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReviewQuestion {
    /// The question, in plain words.
    pub text: String,
    /// Exactly two answers, each one a complete choice.
    pub options: Vec<String>,
}

/// Maps a review reply to the `ReadbacksReviewed` command for `order` at `rev`.
///
/// Only readbacks in `awaiting` get a verdict. A readback the model names twice keeps its first
/// verdict. Both rules keep the batch to what the origin still accepts, since the origin refuses
/// the whole batch when any readback in it is not awaiting review. The question, if any, pins on
/// `machine`, the order's machine.
///
/// # Errors
///
/// Returns [`ReplyError`] when a readback or witness id is invalid, or the question does not
/// have two options.
pub fn review_result(
    order: OrderId,
    machine: &MachineKey,
    rev: OrderRev,
    awaiting: &BTreeSet<ReadbackId>,
    reply: ReviewReply,
) -> Result<SystemCmd, ReplyError> {
    let mut verdicts: BTreeMap<ReadbackId, ReadbackVerdict> = BTreeMap::new();
    for reviewed in reply.readbacks {
        let readback = ReadbackId::from_str(&reviewed.readback)?;
        if !awaiting.contains(&readback) || verdicts.contains_key(&readback) {
            continue;
        }
        verdicts.insert(readback, verdict(rev, reviewed.witnesses)?);
    }
    let question = reply
        .question
        .map(|question| question_draft(machine, question))
        .transpose()?;
    Ok(SystemCmd::ReadbacksReviewed {
        order,
        rev,
        verdicts: verdicts.into_iter().collect(),
        question,
    })
}

fn verdict(rev: OrderRev, witnesses: Vec<WitnessJudgement>) -> Result<ReadbackVerdict, ReplyError> {
    let per_witness = witnesses
        .into_iter()
        .map(|judged| {
            let witness = WitnessId::from_str(&judged.witness)?;
            Ok((witness, judgement(judged.judgement)))
        })
        .collect::<Result<BTreeMap<_, _>, ReplyError>>()?;
    Ok(ReadbackVerdict { rev, per_witness })
}

fn judgement(reply: JudgementReply) -> Judgement {
    match reply {
        JudgementReply::Match => Judgement::Match,
        JudgementReply::Disagrees { reason } => Judgement::Disagrees { reason },
        JudgementReply::Unclear { reason } => Judgement::Unclear { reason },
    }
}

fn question_draft(
    machine: &MachineKey,
    question: ReviewQuestion,
) -> Result<QuestionDraft, ReplyError> {
    let [first, second]: [String; 2] = question
        .options
        .try_into()
        .map_err(|options: Vec<String>| ReplyError::Options(options.len()))?;
    Ok(QuestionDraft {
        at: Locus::Machine {
            machine: machine.clone(),
        },
        text: question.text,
        options: [first, second],
    })
}

/// An explain reply: what goes wrong when both tasks proceed, and one choice if one fits.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExplainReply {
    /// What goes wrong if both tasks proceed, in one or two plain sentences.
    pub consequence: String,
    /// The witness id, such as `w-2`, that would break. Null when no witness would break.
    pub breaks: Option<String>,
    /// One choice from the menu, copied exactly. Null when no choice fits.
    pub suggested: Option<SettlementReply>,
}

/// A settlement as the model writes it. Task ids are copied from the context, such as `t-2`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SettlementReply {
    /// One task narrows its declaration so the overlap goes away.
    Narrow {
        /// The task that narrows.
        task: String,
    },
    /// One task builds after the other lands.
    Serialize {
        /// The task that goes first.
        first: String,
        /// The task that waits.
        then: String,
    },
    /// The owner lands the named contract first, and the other task builds against it.
    SplitAtContract {
        /// The task that owns the contract.
        owner: String,
        /// The contract name, copied from the menu.
        contract: String,
    },
}

/// Maps an explain reply to an [`Explanation`].
///
/// # Errors
///
/// Returns [`ReplyError`] when a named id is invalid.
pub fn explanation(reply: ExplainReply) -> Result<Explanation, ReplyError> {
    let breaks = reply
        .breaks
        .as_deref()
        .map(WitnessId::from_str)
        .transpose()?;
    let suggested = reply.suggested.map(settlement).transpose()?;
    Ok(Explanation {
        consequence: reply.consequence,
        breaks,
        suggested,
    })
}

fn settlement(reply: SettlementReply) -> Result<Settlement, ParseIdError> {
    Ok(match reply {
        SettlementReply::Narrow { task } => Settlement::Narrow {
            task: TaskId::from_str(&task)?,
        },
        SettlementReply::Serialize { first, then } => Settlement::Serialize {
            first: TaskId::from_str(&first)?,
            then: TaskId::from_str(&then)?,
        },
        SettlementReply::SplitAtContract { owner, contract } => Settlement::SplitAtContract {
            owner: TaskId::from_str(&owner)?,
            contract,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order(id: &str) -> OrderId {
        OrderId::from_str(id).expect("valid order id")
    }

    fn plant() -> Vec<MachineKey> {
        vec![MachineKey::from_str("ui").expect("valid key")]
    }

    fn split(machines: &[&str]) -> SplitReply {
        SplitReply {
            tasks: vec![SplitTask {
                title: "Rename the handler".to_owned(),
                brief: "Rename it and update callers".to_owned(),
                scope: machines.iter().map(|m| (*m).to_owned()).collect(),
            }],
        }
    }

    #[test]
    fn split_reply_becomes_split_ready_with_machine_keys() {
        let cmd = split_result(order("o-3"), OrderRev::new(1), &plant(), split(&["ui"]))
            .expect("valid reply");
        assert_eq!(
            cmd,
            SystemCmd::SplitReady {
                order: order("o-3"),
                rev: OrderRev::new(1),
                tasks: vec![TaskSpec {
                    title: "Rename the handler".to_owned(),
                    brief: "Rename it and update callers".to_owned(),
                    scope: vec![MachineKey::from_str("ui").expect("valid key")],
                }],
            }
        );
    }

    #[test]
    fn split_reply_with_an_empty_machine_name_is_refused() {
        let err = split_result(order("o-3"), OrderRev::new(1), &plant(), split(&[""]))
            .expect_err("empty machine name");
        assert!(matches!(err, ReplyError::Id(_)), "got {err:?}");
    }

    #[test]
    fn split_reply_naming_a_machine_off_the_plant_is_refused() {
        let err = split_result(order("o-3"), OrderRev::new(1), &plant(), split(&["db"]))
            .expect_err("db is not on the plant");
        assert!(
            matches!(&err, ReplyError::UnknownMachine(name) if name == "db"),
            "got {err:?}"
        );
    }

    #[test]
    fn split_reply_without_tasks_is_refused() {
        let err = split_result(
            order("o-3"),
            OrderRev::new(1),
            &plant(),
            SplitReply { tasks: Vec::new() },
        )
        .expect_err("no tasks");
        assert!(matches!(err, ReplyError::NoTasks), "got {err:?}");
    }

    fn readback(id: &str) -> ReadbackId {
        ReadbackId::from_str(id).expect("valid readback id")
    }

    fn witness_match(witness: &str) -> WitnessJudgement {
        WitnessJudgement {
            witness: witness.to_owned(),
            judgement: JudgementReply::Match,
        }
    }

    fn reviewed(id: &str, witnesses: Vec<WitnessJudgement>) -> ReviewedReadback {
        ReviewedReadback {
            readback: id.to_owned(),
            witnesses,
        }
    }

    #[test]
    fn review_question_must_offer_exactly_two_answers() {
        let reply = ReviewReply {
            readbacks: Vec::new(),
            question: Some(ReviewQuestion {
                text: "Which?".to_owned(),
                options: vec!["a".into(), "b".into(), "c".into()],
            }),
        };
        let machine = MachineKey::from_str("ui").expect("valid key");
        let err = review_result(
            order("o-1"),
            &machine,
            OrderRev::new(1),
            &BTreeSet::new(),
            reply,
        )
        .expect_err("three options");
        assert!(matches!(err, ReplyError::Options(3)), "got {err:?}");
    }

    #[test]
    fn review_keeps_only_awaiting_readbacks_and_each_one_once() {
        let rev = OrderRev::new(2);
        let awaiting = BTreeSet::from([readback("r-1"), readback("r-3")]);
        let reply = ReviewReply {
            readbacks: vec![
                reviewed("r-3", vec![witness_match("w-1")]),
                reviewed("r-2", vec![witness_match("w-1")]),
                reviewed("r-1", vec![witness_match("w-1")]),
                reviewed(
                    "r-3",
                    vec![WitnessJudgement {
                        witness: "w-1".to_owned(),
                        judgement: JudgementReply::Disagrees {
                            reason: "second answer".to_owned(),
                        },
                    }],
                ),
            ],
            question: None,
        };
        let machine = MachineKey::from_str("ui").expect("valid key");

        let cmd =
            review_result(order("o-1"), &machine, rev, &awaiting, reply).expect("valid reply");

        let matched = ReadbackVerdict {
            rev,
            per_witness: BTreeMap::from([(
                WitnessId::from_str("w-1").expect("valid witness id"),
                Judgement::Match,
            )]),
        };
        assert_eq!(
            cmd,
            SystemCmd::ReadbacksReviewed {
                order: order("o-1"),
                rev,
                verdicts: vec![
                    (readback("r-1"), matched.clone()),
                    (readback("r-3"), matched),
                ],
                question: None,
            },
            "only awaiting readbacks are judged, each once, keeping the first verdict"
        );
    }

    #[test]
    fn explanation_maps_a_suggested_narrow_to_a_task_id() {
        let reply = ExplainReply {
            consequence: "Both edit the same handler.".to_owned(),
            breaks: Some("w-2".to_owned()),
            suggested: Some(SettlementReply::Narrow {
                task: "t-2".to_owned(),
            }),
        };
        let explanation = explanation(reply).expect("valid reply");
        assert_eq!(
            explanation.suggested,
            Some(Settlement::Narrow {
                task: TaskId::from_str("t-2").expect("valid task id"),
            })
        );
        assert_eq!(
            explanation.breaks,
            Some(WitnessId::from_str("w-2").expect("valid id"))
        );
    }

    #[test]
    fn explanation_refuses_a_settlement_with_a_bad_task_id() {
        let reply = ExplainReply {
            consequence: String::new(),
            breaks: None,
            suggested: Some(SettlementReply::Serialize {
                first: "task-2".to_owned(),
                then: "t-3".to_owned(),
            }),
        };
        assert!(explanation(reply).is_err(), "task-2 is not a task id");
    }

    #[test]
    fn review_reply_in_the_documented_shape_parses() {
        let reply: ReviewReply = serde_json::from_value(json!({
            "readbacks": [{
                "readback": "r-0",
                "witnesses": [
                    { "witness": "w-0", "judgement": { "judgement": "match" } },
                    { "witness": "w-1", "judgement": {
                        "judgement": "disagrees", "reason": "the default is dropped"
                    } }
                ]
            }],
            "question": null
        }))
        .expect("the documented shape parses");

        let judged = &reply.readbacks[0].witnesses;
        assert!(matches!(judged[0].judgement, JudgementReply::Match));
        assert!(matches!(
            &judged[1].judgement,
            JudgementReply::Disagrees { reason } if reason == "the default is dropped"
        ));
    }

    #[test]
    fn explain_reply_in_the_documented_shape_parses() {
        let reply: ExplainReply = serde_json::from_value(json!({
            "consequence": "Both edit the greet function.",
            "breaks": "w-0",
            "suggested": { "narrow": { "task": "t-2" } }
        }))
        .expect("the documented shape parses");

        assert!(matches!(
            reply.suggested,
            Some(SettlementReply::Narrow { task }) if task == "t-2"
        ));
    }
}
