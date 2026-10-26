use std::future::Future;

use schemars::JsonSchema;
use serde::de::DeserializeOwned;

use crate::chat::{ChatRequest, ChatResponse, JsonSchemaFormat};
use crate::error::LlmError;
use crate::role::Role;
use crate::usage::CallRecord;

/// The one interface the rest of Acan uses for model calls.
pub trait ModelClient: Send + Sync {
    /// Runs one chat completion for `role`. The provider and model come from routing.
    fn chat(
        &self,
        role: Role,
        request: ChatRequest,
    ) -> impl Future<Output = Result<ChatResponse, LlmError>> + Send;

    /// Runs a completion whose reply must parse as `T`.
    fn chat_json<T>(
        &self,
        role: Role,
        mut request: ChatRequest,
    ) -> impl Future<Output = Result<Structured<T>, LlmError>> + Send
    where
        T: DeserializeOwned + JsonSchema,
    {
        async move {
            request.response_format = Some(JsonSchemaFormat::for_type::<T>());
            let response = self.chat(role, request).await?;
            let value = parse_reply::<T>(&response)?;
            Ok(Structured {
                value,
                call: response.call,
            })
        }
    }
}

/// A typed reply and the accounting record for the call that produced it.
#[derive(Debug, Clone)]
pub struct Structured<T> {
    pub value: T,
    pub call: CallRecord,
}

fn parse_reply<T: DeserializeOwned>(response: &ChatResponse) -> Result<T, LlmError> {
    let raw = response.content.as_deref().unwrap_or_default();
    serde_json::from_str(strip_code_fence(raw.trim())).map_err(|err| LlmError::SchemaValidation {
        provider: response.call.provider.clone(),
        detail: err.to_string(),
        raw: raw.to_owned(),
    })
}

/// Models often wrap JSON in a Markdown fence even when asked not to.
fn strip_code_fence(text: &str) -> &str {
    let Some(rest) = text.strip_prefix("```") else {
        return text;
    };
    let rest = rest.strip_prefix("json").unwrap_or(rest);
    rest.trim().strip_suffix("```").unwrap_or(rest).trim()
}

#[cfg(test)]
mod tests {
    use super::strip_code_fence;

    #[test]
    fn fenced_json_is_unwrapped() {
        assert_eq!(strip_code_fence("```json\n{\"a\":1}\n```"), "{\"a\":1}");
    }

    #[test]
    fn bare_json_is_left_alone() {
        assert_eq!(strip_code_fence("{\"a\":1}"), "{\"a\":1}");
    }
}
