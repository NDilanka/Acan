use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::chat::{ChatRequest, ChatResponse, Message, ToolCall, ToolKind, ToolSpec};
use crate::client::ModelClient;
use crate::config::{ConfigError, LlmConfig, ProviderConfig, StructuredOutput};
use crate::error::LlmError;
use crate::key::{ApiKey, ApiKeys};
use crate::role::Role;
use crate::usage::{CallRecord, Usage, UsageLedger};

const ERROR_BODY_LIMIT: usize = 512;

/// Retries for 429 and 5xx replies. Waits `base_delay`, doubling it before each retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(500),
        }
    }
}

/// The one client for every provider. Each provider speaks the OpenAI chat-completions format.
#[derive(Debug)]
pub struct OpenAiCompatible {
    config: LlmConfig,
    keys: ApiKeys,
    http: reqwest::Client,
    retry: RetryPolicy,
    ledger: UsageLedger,
}

impl OpenAiCompatible {
    pub fn new(config: LlmConfig, keys: ApiKeys) -> Result<Self, ConfigError> {
        Ok(Self {
            config,
            keys,
            http: reqwest::Client::builder().build()?,
            retry: RetryPolicy::default(),
            ledger: UsageLedger::default(),
        })
    }

    pub fn with_retry_policy(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    pub fn ledger(&self) -> &UsageLedger {
        &self.ledger
    }

    async fn post_with_retries(
        &self,
        provider: &ProviderConfig,
        key: &ApiKey,
        body: &WireRequest<'_>,
    ) -> Result<WireCompletion, LlmError> {
        let url = format!(
            "{}/chat/completions",
            provider.base_url.trim_end_matches('/')
        );
        let timeout = Duration::from_secs(provider.timeout_secs);
        let mut attempt: u32 = 1;
        loop {
            let response = self
                .http
                .post(&url)
                .bearer_auth(key.expose())
                .timeout(timeout)
                .json(body)
                .send()
                .await
                .map_err(|err| transport_error(&provider.name, err))?;
            let status = response.status();

            if status.is_success() {
                let text = response
                    .text()
                    .await
                    .map_err(|err| transport_error(&provider.name, err))?;
                return serde_json::from_str(&text).map_err(|err| LlmError::MalformedResponse {
                    provider: provider.name.clone(),
                    detail: err.to_string(),
                });
            }

            let retryable = status.as_u16() == 429 || status.is_server_error();
            if retryable && attempt <= self.retry.max_retries {
                let delay = self
                    .retry
                    .base_delay
                    .saturating_mul(2u32.saturating_pow(attempt - 1));
                tracing::warn!(
                    provider = %provider.name,
                    status = status.as_u16(),
                    attempt,
                    ?delay,
                    "retrying model call"
                );
                tokio::time::sleep(delay).await;
                attempt += 1;
                continue;
            }

            let body = response.text().await.unwrap_or_default();
            return Err(status_error(&provider.name, status.as_u16(), attempt, body));
        }
    }
}

impl ModelClient for OpenAiCompatible {
    async fn chat(&self, role: Role, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        let route = self.config.resolve(role)?;
        let provider = route.provider;
        let key = self
            .keys
            .get(&provider.name)
            .ok_or_else(|| LlmError::MissingKey {
                provider: provider.name.clone(),
                env_var: provider.api_key_env.clone(),
            })?;

        let body = wire_request(route.model, &request, provider.structured_output);
        let started = Instant::now();
        let completion = self.post_with_retries(provider, key, &body).await?;
        let latency = started.elapsed();

        let usage = completion.usage.unwrap_or_else(|| {
            tracing::warn!(
                provider = %provider.name,
                model = route.model,
                "reply carried no usage, recording zero tokens"
            );
            Usage::default()
        });
        let call = CallRecord {
            role,
            provider: provider.name.clone(),
            model: route.model.to_owned(),
            usage,
            latency,
        };
        self.ledger.record(call.clone());

        let choice =
            completion
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| LlmError::MalformedResponse {
                    provider: provider.name.clone(),
                    detail: "reply has no choices".to_owned(),
                })?;
        Ok(ChatResponse {
            content: choice.message.content,
            tool_calls: choice.message.tool_calls.unwrap_or_default(),
            finish_reason: choice.finish_reason,
            call,
        })
    }
}

fn transport_error(provider: &str, err: reqwest::Error) -> LlmError {
    if err.is_timeout() {
        LlmError::Timeout {
            provider: provider.to_owned(),
        }
    } else {
        LlmError::Transport {
            provider: provider.to_owned(),
            source: err,
        }
    }
}

fn status_error(provider: &str, status: u16, attempts: u32, body: String) -> LlmError {
    let provider = provider.to_owned();
    match status {
        402 => LlmError::PaymentRequired {
            provider,
            body: body.chars().take(ERROR_BODY_LIMIT).collect(),
        },
        429 => LlmError::RateLimited { provider, attempts },
        500..=599 => LlmError::ServerError {
            provider,
            status,
            attempts,
        },
        _ => LlmError::Status {
            provider,
            status,
            body: body.chars().take(ERROR_BODY_LIMIT).collect(),
        },
    }
}

/// Without `response_format`, the prompt fallback asks for JSON and `chat_json` validates it.
fn wire_request<'a>(
    model: &'a str,
    request: &'a ChatRequest,
    mode: StructuredOutput,
) -> WireRequest<'a> {
    let (messages, response_format) = match (&request.response_format, mode) {
        (None, _) => (request.messages.clone(), None),
        (Some(format), StructuredOutput::JsonSchema) => (
            request.messages.clone(),
            Some(WireResponseFormat {
                kind: "json_schema",
                json_schema: WireJsonSchema {
                    name: &format.name,
                    schema: &format.schema,
                    strict: false,
                },
            }),
        ),
        (Some(format), StructuredOutput::Prompt) => {
            let mut messages = Vec::with_capacity(request.messages.len() + 1);
            messages.push(Message::system(schema_prompt(&format.schema)));
            messages.extend(request.messages.iter().cloned());
            (messages, None)
        }
    };

    WireRequest {
        model,
        messages,
        tools: request
            .tools
            .iter()
            .map(|function| WireTool {
                kind: ToolKind::Function,
                function,
            })
            .collect(),
        temperature: request.temperature,
        max_tokens: request.max_tokens,
        response_format,
    }
}

fn schema_prompt(schema: &Value) -> String {
    format!(
        "Reply with one JSON object that matches this JSON Schema. Write no prose and no code fences.\n{schema}"
    )
}

#[derive(Serialize)]
struct WireRequest<'a> {
    model: &'a str,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<WireTool<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<WireResponseFormat<'a>>,
}

#[derive(Serialize)]
struct WireTool<'a> {
    #[serde(rename = "type")]
    kind: ToolKind,
    function: &'a ToolSpec,
}

#[derive(Serialize)]
struct WireResponseFormat<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    json_schema: WireJsonSchema<'a>,
}

#[derive(Serialize)]
struct WireJsonSchema<'a> {
    name: &'a str,
    schema: &'a Value,
    strict: bool,
}

#[derive(Deserialize)]
struct WireCompletion {
    choices: Vec<WireChoice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct WireChoice {
    message: WireReply,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct WireReply {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}
