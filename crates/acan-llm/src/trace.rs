use std::fmt;

use serde::Serialize;

/// The provider's id for one reply, and where Acan read it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequestId {
    pub value: String,
    pub source: RequestIdSource,
}

/// Where a [`RequestId`] was read from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestIdSource {
    /// A response header with this name.
    Header(String),
    /// The `id` field of the chat-completions body.
    Body,
}

impl fmt::Display for RequestIdSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Header(name) => write!(f, "header {name}"),
            Self::Body => f.write_str("body id"),
        }
    }
}

/// A reply with the provider's trace fields for the call that produced it.
#[derive(Debug, Clone, PartialEq)]
pub struct Traced<T> {
    pub value: T,
    /// Absent when the reply has neither a request-id header nor a body `id`.
    pub request_id: Option<RequestId>,
    /// The model the provider echoed in the reply body.
    pub reply_model: Option<String>,
}
