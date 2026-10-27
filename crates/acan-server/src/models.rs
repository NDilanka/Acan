//! The model client the server runs: the providers in a routing file, with the offline replies
//! behind them.
//!
//! A live call that fails answers with the offline reply for that call and logs a warning. The
//! failure can be a missing key, an HTTP or timeout error, or a reply that does not match the
//! schema. Live and offline calls are metered in one ledger, so the usage panel shows which
//! calls were offline.

use std::path::Path;

use acan_llm::{
    ApiKeys, ChatRequest, ChatResponse, ConfigError, LlmConfig, LlmError, ModelClient,
    OpenAiCompatible, Role, Structured, UsageLedger,
};
use anyhow::Context;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

use crate::offline::Offline;

/// The client behind every model call. Routing is configuration; this enum only picks the source.
#[expect(
    clippy::large_enum_variant,
    reason = "one value per server, built once and never moved in a hot path"
)]
pub enum Models {
    /// Calls the configured OpenAI-compatible providers. A call that fails is answered by
    /// `offline`.
    Live {
        client: OpenAiCompatible,
        offline: Offline,
    },
    /// Answers from built-in replies. Used when there is no routing file.
    Offline(Offline),
}

impl Models {
    /// Loads the routing file at `path`. When the file is absent, falls back to the offline
    /// replies.
    ///
    /// # Errors
    ///
    /// Returns an error when the file exists but cannot be read or validated.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            tracing::warn!(
                config = %path.display(),
                "no model routing file; model jobs get built-in offline replies"
            );
            return Ok(Self::Offline(Offline::default()));
        }
        let config =
            LlmConfig::load(path).with_context(|| format!("loading {}", path.display()))?;
        let keys = ApiKeys::from_env(&config);
        Ok(Self::live(config, keys)?)
    }

    /// Builds the live client for `config` with `keys`. Calls that fail are answered offline.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Client`] when the HTTP client cannot be built.
    pub fn live(config: LlmConfig, keys: ApiKeys) -> Result<Self, ConfigError> {
        let ledger = UsageLedger::default();
        let client = OpenAiCompatible::new(config, keys)?.with_ledger(ledger.clone());
        Ok(Self::Live {
            client,
            offline: Offline::with_ledger(ledger),
        })
    }

    /// The usage ledger that records every call this client makes, live or offline.
    #[must_use]
    pub fn ledger(&self) -> UsageLedger {
        match self {
            Self::Live { client, .. } => client.ledger().clone(),
            Self::Offline(offline) => offline.ledger().clone(),
        }
    }
}

/// Logs a live failure. A reply that does not match the schema also logs the parser's detail and
/// the raw reply, since that is what shows why the model drifted.
fn warn_fallback(role: Role, err: &LlmError) {
    if let LlmError::SchemaValidation { detail, raw, .. } = err {
        tracing::warn!(
            %role,
            error = %err,
            %detail,
            %raw,
            "live model call failed; answering with the offline reply"
        );
    } else {
        tracing::warn!(
            %role,
            error = %err,
            "live model call failed; answering with the offline reply"
        );
    }
}

impl ModelClient for Models {
    async fn chat(&self, role: Role, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        match self {
            Self::Live { client, offline } => match client.chat(role, request.clone()).await {
                Ok(response) => Ok(response),
                Err(err) => {
                    warn_fallback(role, &err);
                    offline.chat(role, request).await
                }
            },
            Self::Offline(offline) => offline.chat(role, request).await,
        }
    }

    async fn chat_json<T>(
        &self,
        role: Role,
        request: ChatRequest,
    ) -> Result<Structured<T>, LlmError>
    where
        T: DeserializeOwned + JsonSchema,
    {
        match self {
            Self::Live { client, offline } => {
                // The live result ends its statement before the fallback awaits, so the future
                // stays Send even when `T` is not.
                let err = match client.chat_json::<T>(role, request.clone()).await {
                    Ok(structured) => return Ok(structured),
                    Err(err) => err,
                };
                warn_fallback(role, &err);
                offline.chat_json::<T>(role, request).await
            }
            Self::Offline(offline) => offline.chat_json::<T>(role, request).await,
        }
    }
}
