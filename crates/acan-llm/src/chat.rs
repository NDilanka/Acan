use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::usage::CallRecord;

/// One message in a conversation. Serializes to the OpenAI chat-completions shape.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum Message {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        tool_calls: Vec<ToolCall>,
    },
    Tool {
        tool_call_id: String,
        content: String,
    },
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self::System {
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::User {
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type", default)]
    pub kind: ToolKind,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolKind {
    #[default]
    Function,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    /// JSON text as the model wrote it. The caller parses it.
    pub arguments: String,
}

/// A tool the model may call. `parameters` is a JSON Schema object.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// A chat-completions request for one role. The provider and model come from routing.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSpec>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    /// Set by `chat_json`. Set it directly to ask `chat` for schema-shaped output.
    pub response_format: Option<JsonSchemaFormat>,
}

impl ChatRequest {
    /// The text of the last user message, if there is one.
    pub fn last_user_text(&self) -> Option<&str> {
        self.messages
            .iter()
            .rev()
            .find_map(|message| match message {
                Message::User { content } => Some(content.as_str()),
                _ => None,
            })
    }
}

/// A named JSON Schema the reply must match.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonSchemaFormat {
    pub name: String,
    pub schema: Value,
}

impl JsonSchemaFormat {
    /// Builds the format for `T` from its schemars schema.
    pub fn for_type<T: JsonSchema>() -> Self {
        let mut schema = serde_json::to_value(schemars::schema_for!(T))
            .expect("schemars schemas serialize to JSON");
        if let Value::Object(map) = &mut schema {
            map.remove("$schema");
        }
        Self {
            name: openai_name(&T::schema_name()),
            schema,
        }
    }
}

/// OpenAI accepts only letters, digits, dashes, and underscores, up to 64 characters.
fn openai_name(raw: &str) -> String {
    let name: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(64)
        .collect();
    if name.is_empty() {
        "output".to_owned()
    } else {
        name
    }
}

/// The reply to one chat call, with its accounting record.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: Option<String>,
    pub call: CallRecord,
}
