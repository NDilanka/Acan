//! Prompts, reply shapes, and the mapping from a model reply to the commands the origin accepts.
//!
//! Ids cross the model boundary in their display form (`t-3`, `w-1`). Counter ids serialize as
//! bare numbers, so replies are parsed from these strings.

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use acan_core::{
    Clash, ClashId, Explanation, Judgement, Locus, MachineKey, OrderId, OrderRev, ParseIdError,
    QuestionDraft, ReadbackId, ReadbackVerdict, Settlement, SystemCmd, Task, TaskId, TaskSpec,
    Witness, WitnessId, World,
};
use acan_llm::{ChatRequest, Message};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

pub(crate) const SPLIT_PROMPT: &str = "You split one order into tasks for AI coding workers. Each task is one piece of work that one worker can finish alone. Scope lists the machine names a task touches. Every witness must still hold once all the tasks have landed.";

pub(crate) const REVIEW_PROMPT: &str = "You review each worker readback against the order's witnesses. For every witness, answer match when the readback respects it, disagrees when it contradicts it, and unclear when it does not say enough. Give a reason for every answer that is not match. If readbacks disagree about what the order means, ask one question with exactly two options. Otherwise ask nothing.";

pub(crate) const EXPLAIN_PROMPT: &str = "You explain a clash between two tasks that touch the same code. Say what goes wrong if both proceed. Name the witness that would break, if one would. Suggest one choice from the menu, or none.";

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
    /// A split reply has no tasks, so the order would have nothing to do.
    #[error("split reply has no tasks")]
    NoTasks,
    /// A review question must offer exactly two answers.
    #[error("review question needs exactly two options, got {0}")]
    Options(usize),
}

fn request(system: &str, context: Value) -> ChatRequest {
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

/// The split prompt for `order`, or `None` when the order is gone.
#[must_use]
pub fn split_request(world: &World, order: OrderId) -> Option<ChatRequest> {
    let order = world.order(order)?;
    Some(request(
        SPLIT_PROMPT,
        json!({
            "order": order.sentence,
            "machine": order.machine,
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
        json!({
            "order": order.sentence,
            "witnesses": witnesses_json(world, &order.witnesses),
            "readbacks": reviewed,
        }),
    ))
}

/// The explain prompt for `clash`, or `None` when the clash or a party to it is gone.
#[must_use]
pub fn explain_request(world: &World, clash: ClashId) -> Option<ChatRequest> {
    let clash: &Clash = world.clash(clash)?;
    let first = world.task(clash.key.a())?;
    let second = world.task(clash.key.b())?;
    let menu: Vec<Value> = clash.menu().iter().map(settlement_json).collect();
    Some(request(
        EXPLAIN_PROMPT,
        json!({
            "clash": clash.id.to_string(),
            "kind": clash.kind,
            "at": clash.key.at(),
            "tasks": [task_json(first), task_json(second)],
            "menu": menu,
        }),
    ))
}

/// A split reply: the tasks the foreman cut the order into.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SplitReply {
    /// The tasks, in the order they should be worked.
    pub tasks: Vec<SplitTask>,
}

/// One task in a split reply.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SplitTask {
    /// A short title.
    pub title: String,
    /// What the worker should do.
    pub brief: String,
    /// The machine names the task touches.
    pub scope: Vec<String>,
}

/// Maps a split reply to the `SplitReady` command for `order` at `rev`.
///
/// # Errors
///
/// Returns [`ReplyError`] when the reply has no tasks or names an invalid machine.
pub fn split_result(
    order: OrderId,
    rev: OrderRev,
    reply: SplitReply,
) -> Result<SystemCmd, ReplyError> {
    if reply.tasks.is_empty() {
        return Err(ReplyError::NoTasks);
    }
    let tasks = reply
        .tasks
        .into_iter()
        .map(task_spec)
        .collect::<Result<Vec<_>, ParseIdError>>()?;
    Ok(SystemCmd::SplitReady { order, rev, tasks })
}

fn task_spec(task: SplitTask) -> Result<TaskSpec, ParseIdError> {
    let scope: Vec<MachineKey> = task
        .scope
        .iter()
        .map(|name| MachineKey::from_str(name))
        .collect::<Result<_, _>>()?;
    Ok(TaskSpec {
        title: task.title,
        brief: task.brief,
        scope,
    })
}

/// A review reply: one entry per readback the foreman judged, and at most one question.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReviewReply {
    /// One entry per readback.
    pub readbacks: Vec<ReviewedReadback>,
    /// A question, when readbacks disagree about what the order means.
    pub question: Option<ReviewQuestion>,
}

/// The judgements for one readback.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReviewedReadback {
    /// The readback id, such as `r-4`.
    pub readback: String,
    /// One judgement per witness of the order.
    pub witnesses: Vec<WitnessJudgement>,
}

/// The judgement of one witness.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WitnessJudgement {
    /// The witness id, such as `w-2`.
    pub witness: String,
    /// The judgement.
    pub judgement: JudgementReply,
}

/// A judgement as the model writes it.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JudgementReply {
    /// The readback respects the witness.
    Match,
    /// The readback contradicts the witness.
    Disagrees {
        /// Why.
        reason: String,
    },
    /// The readback does not say enough.
    Unclear {
        /// Why.
        reason: String,
    },
}

/// A question the foreman drafts for a human.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReviewQuestion {
    /// The question.
    pub text: String,
    /// Exactly two answers.
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

/// An explain reply: the consequence of letting both tasks proceed.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExplainReply {
    /// What goes wrong if both proceed.
    pub consequence: String,
    /// The witness that would break, such as `w-2`, if one would.
    pub breaks: Option<String>,
    /// A suggested choice from the menu, if one fits.
    pub suggested: Option<SettlementReply>,
}

/// A settlement as the model writes it.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SettlementReply {
    /// Narrow one task's declaration.
    Narrow {
        /// The task that narrows.
        task: String,
    },
    /// Build one task after another.
    Serialize {
        /// The task that goes first.
        first: String,
        /// The task that waits.
        then: String,
    },
    /// Land a contract first, then build against it.
    SplitAtContract {
        /// The task that owns the contract.
        owner: String,
        /// The contract name.
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
        let cmd =
            split_result(order("o-3"), OrderRev::new(1), split(&["ui"])).expect("valid reply");
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
        let err = split_result(order("o-3"), OrderRev::new(1), split(&[""]))
            .expect_err("empty machine name");
        assert!(matches!(err, ReplyError::Id(_)), "got {err:?}");
    }

    #[test]
    fn split_reply_without_tasks_is_refused() {
        let err = split_result(
            order("o-3"),
            OrderRev::new(1),
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
}
