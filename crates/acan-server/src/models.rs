//! The model client the server runs: the providers in a routing file, or the offline replies.

use std::path::Path;

use acan_llm::{
    ApiKeys, ChatRequest, ChatResponse, LlmConfig, LlmError, ModelClient, OpenAiCompatible, Role,
    UsageLedger,
};
use anyhow::Context;

use crate::offline::Offline;

/// The client behind every model call. Routing is configuration; this enum only picks the source.
#[expect(
    clippy::large_enum_variant,
    reason = "one value per server, built once and never moved in a hot path"
)]
pub enum Models {
    /// Calls the configured OpenAI-compatible providers.
    Live(OpenAiCompatible),
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
        Ok(Self::Live(OpenAiCompatible::new(config, keys)?))
    }

    /// The usage ledger that records every call this client makes.
    #[must_use]
    pub fn ledger(&self) -> UsageLedger {
        match self {
            Self::Live(client) => client.ledger().clone(),
            Self::Offline(offline) => offline.ledger().clone(),
        }
    }
}

impl ModelClient for Models {
    async fn chat(&self, role: Role, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        match self {
            Self::Live(client) => client.chat(role, request).await,
            Self::Offline(offline) => offline.chat(role, request).await,
        }
    }
}
