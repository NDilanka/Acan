//! The offline model: built-in replies so a server with no routing file still runs its loop.
//!
//! Each job kind gets the plainest reply that moves it forward: a split into one task on the
//! order's machine, a review that matches every witness for every readback, and a generic
//! clash explanation with no suggestion. Token counts are estimates, so the usage panel shows
//! the calls a live provider would have taken.

use std::time::Duration;

use acan_llm::{
    CallRecord, ChatRequest, ChatResponse, LlmError, ModelClient, Role, Usage, UsageLedger,
};
use serde_json::{Value, json};

use crate::model::{self, Ask};

/// Characters per token for the usage estimate, the usual rule of thumb for English text.
const CHARS_PER_TOKEN: usize = 4;

/// Answers model jobs from built-in replies and records each call in its ledger.
#[derive(Debug, Default)]
pub struct Offline {
    ledger: UsageLedger,
}

impl Offline {
    /// Records every answered call in `ledger`, so the live client and the fallback share one.
    #[must_use]
    pub fn with_ledger(ledger: UsageLedger) -> Self {
        Self { ledger }
    }

    /// The ledger every answered call is recorded in.
    #[must_use]
    pub fn ledger(&self) -> &UsageLedger {
        &self.ledger
    }
}

impl ModelClient for Offline {
    async fn chat(&self, role: Role, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        let context: Value = request
            .last_user_text()
            .and_then(|text| serde_json::from_str(text).ok())
            .ok_or(LlmError::NoScriptedReply { role })?;
        let reply = match model::ask_of(&request).ok_or(LlmError::NoScriptedReply { role })? {
            Ask::Split => split(&context),
            Ask::Review => review(&context),
            Ask::Explain => explain(),
        }
        .to_string();
        let prompt_chars: usize = request.last_user_text().map_or(0, str::len);
        let call = CallRecord {
            role,
            provider: "offline".to_owned(),
            model: "offline".to_owned(),
            usage: Usage {
                prompt_tokens: estimate(prompt_chars),
                completion_tokens: estimate(reply.len()),
            },
            latency: Duration::ZERO,
        };
        self.ledger.record(call.clone());
        Ok(ChatResponse {
            content: Some(reply),
            tool_calls: Vec::new(),
            finish_reason: Some("stop".to_owned()),
            call,
        })
    }
}

fn estimate(chars: usize) -> u32 {
    u32::try_from(chars.div_ceil(CHARS_PER_TOKEN)).unwrap_or(u32::MAX)
}

fn split(context: &Value) -> Value {
    let sentence = context["order"].as_str().unwrap_or("the order");
    json!({
        "tasks": [{
            "title": sentence,
            "brief": format!("Make this true: {sentence}"),
            "scope": [context["machine"]],
        }]
    })
}

fn review(context: &Value) -> Value {
    let witnesses: Vec<Value> = ids(&context["witnesses"], "id")
        .map(|witness| json!({ "witness": witness, "judgement": { "judgement": "match" } }))
        .collect();
    let readbacks: Vec<Value> = ids(&context["readbacks"], "readback")
        .map(|readback| json!({ "readback": readback, "witnesses": witnesses }))
        .collect();
    json!({ "readbacks": readbacks, "question": null })
}

fn explain() -> Value {
    json!({
        "consequence": "Both tasks change the same code, so whichever lands second would build on code that has moved under it.",
        "breaks": null,
        "suggested": null,
    })
}

/// The string `field` of every object in the array `list`.
fn ids<'v>(list: &'v Value, field: &'v str) -> impl Iterator<Item = &'v str> {
    list.as_array()
        .into_iter()
        .flatten()
        .filter_map(move |item| item[field].as_str())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::str::FromStr;

    use acan_core::{Judgement, MachineKey, OrderId, OrderRev, ReadbackId, SystemCmd, WitnessId};
    use acan_llm::Message;

    use super::*;

    fn request(system: &str, context: &Value) -> ChatRequest {
        ChatRequest {
            messages: vec![Message::system(system), Message::user(context.to_string())],
            ..ChatRequest::default()
        }
    }

    #[tokio::test]
    async fn a_review_matches_every_witness_for_every_readback() {
        let offline = Offline::default();
        let context = json!({
            "order": "keep the store fast",
            "witnesses": [{ "id": "w-1" }, { "id": "w-2" }],
            "readbacks": [{ "readback": "r-1" }, { "readback": "r-2" }],
        });
        let reply = offline
            .chat_json::<model::ReviewReply>(Role::Foreman, request(model::REVIEW_PROMPT, &context))
            .await
            .expect("the offline review parses");
        let readbacks = |ids: &[&str]| -> BTreeSet<ReadbackId> {
            ids.iter()
                .map(|id| ReadbackId::from_str(id).expect("valid readback id"))
                .collect()
        };
        let machine = MachineKey::from_str("model").expect("valid machine key");
        let order = OrderId::from_str("o-0").expect("valid order id");

        let cmd = model::review_result(
            order,
            &machine,
            OrderRev::new(1),
            &readbacks(&["r-1", "r-2"]),
            reply.value,
        )
        .expect("the reply becomes a command");

        let SystemCmd::ReadbacksReviewed {
            verdicts, question, ..
        } = cmd
        else {
            panic!("a review becomes ReadbacksReviewed, got {cmd:?}");
        };
        let judged: BTreeSet<ReadbackId> = verdicts.iter().map(|(id, _)| *id).collect();
        assert_eq!(
            judged,
            readbacks(&["r-1", "r-2"]),
            "every readback is judged"
        );
        let witnesses: BTreeSet<WitnessId> = ["w-1", "w-2"]
            .iter()
            .map(|id| WitnessId::from_str(id).expect("valid witness id"))
            .collect();
        for (readback, verdict) in &verdicts {
            assert_eq!(
                verdict.per_witness.keys().copied().collect::<BTreeSet<_>>(),
                witnesses,
                "{readback} is judged against every witness"
            );
            assert!(
                verdict.per_witness.values().all(|j| *j == Judgement::Match),
                "{readback} matches"
            );
        }
        assert!(question.is_none(), "matched readbacks raise no question");
        assert_eq!(offline.ledger().snapshot().len(), 1, "the call is recorded");
    }

    #[tokio::test]
    async fn an_explain_reply_becomes_an_explanation() {
        let reply = Offline::default()
            .chat_json::<model::ExplainReply>(
                Role::Fast,
                request(model::EXPLAIN_PROMPT, &json!({ "clash": "c-1" })),
            )
            .await
            .expect("the offline explanation parses");
        let explanation = model::explanation(reply.value).expect("a valid explanation");
        assert!(
            !explanation.consequence.is_empty(),
            "it says what goes wrong"
        );
    }

    #[tokio::test]
    async fn an_unknown_request_gets_no_reply() {
        let err = Offline::default()
            .chat(Role::Foreman, request("something else", &json!({})))
            .await
            .expect_err("only the server's own prompts are answered");
        assert!(
            matches!(err, LlmError::NoScriptedReply { .. }),
            "got {err:?}"
        );
    }
}
