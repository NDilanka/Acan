//! The MCP client the harness uses. It speaks JSON-RPC 2.0 over one POST per message to the
//! origin's `/mcp` endpoint, with the agent's bearer token.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, bail};
use reqwest::StatusCode;
use serde_json::{Value, json};

/// The MCP revision the harness asks for. The origin echoes it back when it lists it.
const PROTOCOL_VERSION: &str = "2025-06-18";

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// The answer to one tool call. A command the origin refuses is an answer, not a transport
/// failure, so it comes back as [`Reply::Refused`] with the origin's rejection text.
#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    /// The tool succeeded. The value is the JSON the tool returned.
    Done(Value),
    /// The origin refused the command. The text is its rejection.
    Refused(String),
}

/// A connection to one origin's MCP endpoint, acting as one agent. Cloning shares the HTTP pool.
#[derive(Clone)]
pub struct McpClient {
    http: reqwest::Client,
    url: String,
    token: String,
}

impl fmt::Debug for McpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("McpClient")
            .field("url", &self.url)
            .finish_non_exhaustive()
    }
}

impl McpClient {
    /// A client for the origin at `server`, such as `http://127.0.0.1:7311`.
    #[must_use]
    pub fn new(server: &str, token: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            url: format!("{}/mcp", server.trim_end_matches('/')),
            token,
        }
    }

    /// Opens the session: `initialize`, then the `initialized` notification.
    ///
    /// # Errors
    ///
    /// Returns an error when the token is refused, the origin does not speak the revision, or
    /// the origin cannot be reached.
    pub async fn connect(&self) -> anyhow::Result<()> {
        let result = self
            .request(
                "initialize",
                json!({
                    "protocolVersion": PROTOCOL_VERSION,
                    "capabilities": {},
                    "clientInfo": { "name": "acan-worker", "version": env!("CARGO_PKG_VERSION") },
                }),
            )
            .await?;
        let version = result["protocolVersion"].as_str().unwrap_or_default();
        if version != PROTOCOL_VERSION {
            bail!("the origin speaks MCP revision {version:?}, not {PROTOCOL_VERSION}");
        }
        let response = self
            .post(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }))
            .await?;
        if !response.status().is_success() {
            bail!(
                "the origin did not accept the initialized notification: {}",
                response.status()
            );
        }
        Ok(())
    }

    /// Calls one tool and returns its answer.
    ///
    /// # Errors
    ///
    /// Returns an error only when the call did not complete as a tool call: the transport
    /// failed, the origin answered with a protocol error, or the token was refused.
    pub async fn call(&self, tool: &str, arguments: Value) -> anyhow::Result<Reply> {
        let result = self
            .request(
                "tools/call",
                json!({ "name": tool, "arguments": arguments }),
            )
            .await?;
        let text = result["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        if result["isError"].as_bool().unwrap_or(false) {
            return Ok(Reply::Refused(text));
        }
        let value = serde_json::from_str(&text).unwrap_or(Value::String(text));
        Ok(Reply::Done(value))
    }

    async fn request(&self, method: &str, params: Value) -> anyhow::Result<Value> {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let response = self
            .post(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))
            .await?;
        let status = response.status();
        if !status.is_success() {
            bail!("{method} answered HTTP {status}");
        }
        let body: Value = response
            .json()
            .await
            .with_context(|| format!("{method} answered with no JSON"))?;
        if let Some(error) = body.get("error") {
            let message = error["message"].as_str().unwrap_or("unknown error");
            bail!("{method} failed: {message}");
        }
        Ok(body["result"].clone())
    }

    async fn post(&self, body: &Value) -> anyhow::Result<reqwest::Response> {
        let response = self
            .http
            .post(&self.url)
            .bearer_auth(&self.token)
            .json(body)
            .send()
            .await
            .with_context(|| format!("posting to {}", self.url))?;
        if response.status() == StatusCode::UNAUTHORIZED {
            bail!("the origin refused the agent token");
        }
        Ok(response)
    }
}
