use std::fmt;
use std::time::Duration;

use serde::Serialize;

use crate::chat::{ChatRequest, ChatResponse, ToolCall};
use crate::client::ModelClient;
use crate::error::LlmError;
use crate::role::Role;
use crate::usage::{CallRecord, Usage, UsageLedger};

type Matcher = Box<dyn Fn(&ChatRequest) -> bool + Send + Sync>;

/// An offline provider. Replies come from a script keyed by role and a request matcher.
#[derive(Default)]
pub struct Stub {
    rules: Vec<Rule>,
    ledger: UsageLedger,
}

struct Rule {
    role: Role,
    matches: Matcher,
    reply: Reply,
}

/// What a scripted call returns.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Reply {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Usage,
}

impl Reply {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            ..Self::default()
        }
    }

    /// A reply whose text is `value` serialized as JSON.
    pub fn json(value: &impl Serialize) -> Self {
        Self::text(serde_json::to_string(value).expect("stub reply serializes to JSON"))
    }

    pub fn with_usage(mut self, usage: Usage) -> Self {
        self.usage = usage;
        self
    }
}

impl Stub {
    /// Adds a reply for `role` when `matches` returns true. The first matching rule wins.
    pub fn when(
        mut self,
        role: Role,
        matches: impl Fn(&ChatRequest) -> bool + Send + Sync + 'static,
        reply: Reply,
    ) -> Self {
        self.rules.push(Rule {
            role,
            matches: Box::new(matches),
            reply,
        });
        self
    }

    pub fn ledger(&self) -> &UsageLedger {
        &self.ledger
    }
}

impl fmt::Debug for Stub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stub")
            .field("rules", &self.rules.len())
            .finish_non_exhaustive()
    }
}

impl ModelClient for Stub {
    async fn chat(&self, role: Role, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        let rule = self
            .rules
            .iter()
            .find(|rule| rule.role == role && (rule.matches)(&request))
            .ok_or(LlmError::NoScriptedReply { role })?;
        let call = CallRecord {
            role,
            provider: "stub".to_owned(),
            model: "stub".to_owned(),
            usage: rule.reply.usage,
            latency: Duration::ZERO,
        };
        self.ledger.record(call.clone());
        Ok(ChatResponse {
            content: rule.reply.content.clone(),
            tool_calls: rule.reply.tool_calls.clone(),
            finish_reason: Some("stop".to_owned()),
            call,
        })
    }
}
