//! MCP request, response and error envelopes of the scope-first lane.
//!
//! Five tools: `get_scope`, `get_rulings`, `post_readback`, `contest`, `get_receipt`. Every
//! request and response carries `version: "acan-mcp/1"`. Every failure is an [`McpError`] with a
//! closed `code`; nothing fails as an empty success.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::episode::{ContestKind, ContestStatus, Prediction};
use crate::expr::Predicate;
use crate::gate::LedgerHead;
use crate::hash::{EpisodeId, ProbeHash, ReceiptHash, ScopeHash};
use crate::ledger::NotEnforcedReason;
use crate::report::Receipt;
use crate::scope::{RepairBudget, Scope, Selector};
use crate::types::{
    ContestId, MachineId, NonEmpty, OrderRef, RulingId, ScopeRev, TaskRef, UnixMillis,
};
use crate::version::McpV1;
use crate::view::RulingTagState;

/// `get_scope`: the confirmed scope of the caller's task.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetScopeRequest {
    /// Always `acan-mcp/1`.
    pub version: McpV1,
    /// The caller's task.
    pub task: TaskRef,
}

/// The reply to `get_scope`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetScopeResponse {
    /// Always `acan-mcp/1`.
    pub version: McpV1,
    /// The task.
    pub task: TaskRef,
    /// The confirmed scope.
    pub scope: Scope,
    /// Its hash, which a readback must echo.
    pub scope_hash: ScopeHash,
    /// When the human confirmed it.
    pub confirmed_at: UnixMillis,
    /// The order's repair budget.
    pub budget: RepairBudget,
}

/// `get_rulings`: the rulings in force on some machines, or on all of them.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetRulingsRequest {
    /// Always `acan-mcp/1`.
    pub version: McpV1,
    /// The caller's task.
    pub task: TaskRef,
    /// The machines, or empty for every machine.
    #[serde(default)]
    pub machine_ids: Vec<MachineId>,
}

/// How a ruling is applied.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Enforcement {
    /// Executed at the gate.
    Enforced {
        /// Where.
        selector: Selector,
        /// What must hold.
        predicate: Box<Predicate>,
    },
    /// A note. Never counted as a check.
    NotEnforced {
        /// The note.
        text: String,
        /// Why it is not enforced.
        reason: NotEnforcedReason,
    },
}

/// One ruling as an agent reads it.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RulingSummary {
    /// The ruling lineage.
    pub ruling_id: RulingId,
    /// The decision in force.
    pub event_seq: u64,
    /// The label.
    pub label: String,
    /// The machines it is fixed to.
    pub machine_ids: NonEmpty<MachineId>,
    /// Whether it is current.
    pub state: RulingTagState,
    /// How it is applied.
    pub enforcement: Enforcement,
}

/// The reply to `get_rulings`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetRulingsResponse {
    /// Always `acan-mcp/1`.
    pub version: McpV1,
    /// The log revision read.
    pub ledger_head: LedgerHead,
    /// The rulings.
    pub rulings: Vec<RulingSummary>,
}

/// `post_readback`: the agent restates the scope it will build against, with predictions.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PostReadbackRequest {
    /// Always `acan-mcp/1`.
    pub version: McpV1,
    /// The caller's task.
    pub task: TaskRef,
    /// The scope revision the agent read. A different revision is refused as stale.
    pub scope_rev: ScopeRev,
    /// The scope hash the agent read.
    pub scope_hash: ScopeHash,
    /// The plan in the agent's words.
    pub summary: String,
    /// What the agent predicts its change does. Recorded as model proposals and scored by the
    /// core against observations; never evidence, never a pass.
    pub predictions: Vec<Prediction>,
}

/// The deterministic acknowledgement of a readback. It authorizes nothing but the claim the
/// confirmed scope already allows.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PostReadbackResponse {
    /// Always `acan-mcp/1`.
    pub version: McpV1,
    /// The task.
    pub task: TaskRef,
    /// The acknowledged revision.
    pub scope_rev: ScopeRev,
    /// The acknowledged hash.
    pub scope_hash: ScopeHash,
    /// How many predictions were recorded.
    pub predictions_recorded: u32,
}

/// `contest`: the agent objects to one discrepancy episode, with a reproducer.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContestRequest {
    /// Always `acan-mcp/1`.
    pub version: McpV1,
    /// The caller's task.
    pub task: TaskRef,
    /// The episode.
    pub episode_id: EpisodeId,
    /// What the agent claims.
    pub kind: ContestKind,
    /// A probe that shows it.
    pub reproducer: ProbeHash,
    /// Why, in the agent's words.
    pub rationale: String,
}

/// The reply to `contest`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContestResponse {
    /// Always `acan-mcp/1`.
    pub version: McpV1,
    /// The filed contest.
    pub contest_id: ContestId,
    /// The episode.
    pub target: EpisodeId,
    /// Always `open` when filed.
    pub status: ContestStatus,
}

/// `get_receipt`: the latest receipt of the caller's task.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetReceiptRequest {
    /// Always `acan-mcp/1`.
    pub version: McpV1,
    /// The caller's task.
    pub task: TaskRef,
}

/// The reply to `get_receipt`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetReceiptResponse {
    /// Always `acan-mcp/1`.
    pub version: McpV1,
    /// The receipt's hash.
    pub receipt_hash: ReceiptHash,
    /// The receipt.
    pub receipt: Receipt,
}

/// Why an MCP call failed.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "code", rename_all = "snake_case", deny_unknown_fields)]
pub enum McpErrorCode {
    /// No valid token.
    Unauthorized {},
    /// The caller may not do this. Agent tokens never confirm scopes, append rulings or retire
    /// probes.
    Forbidden {
        /// The refused action.
        action: String,
    },
    /// The task, order, episode or receipt does not exist.
    NotFound {
        /// What was missing.
        what: String,
    },
    /// The request did not decode or failed validation.
    InvalidRequest {
        /// What was wrong.
        detail: String,
    },
    /// The order's scope has not been confirmed, so no work may be claimed.
    ScopeNotConfirmed {
        /// The order.
        order: OrderRef,
    },
    /// The request named a scope revision or hash other than the confirmed one.
    StaleScope {
        /// The confirmed revision.
        current_rev: ScopeRev,
        /// The confirmed hash.
        current_hash: ScopeHash,
    },
    /// The request relied on observations of an older commit or scope.
    StaleEvidence {
        /// What was stale.
        detail: String,
    },
    /// The episode's or order's repair budget is spent.
    BudgetExhausted {
        /// The episode.
        episode_id: EpisodeId,
    },
    /// The operation is not available in this build.
    Unsupported {
        /// The operation, such as `API01: get_scope`.
        operation: String,
    },
}

/// The error envelope of every MCP tool.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct McpError {
    /// Always `acan-mcp/1`.
    pub version: McpV1,
    /// Why.
    pub error: McpErrorCode,
    /// One line for the agent's transcript.
    pub message: String,
}

impl McpError {
    /// An error with `code` and `message`.
    pub fn new(code: McpErrorCode, message: impl Into<String>) -> Self {
        Self {
            version: McpV1,
            error: code,
            message: message.into(),
        }
    }

    /// The error for an operation the build does not have yet.
    #[must_use]
    pub fn unsupported(operation: crate::Unsupported) -> Self {
        Self::new(
            McpErrorCode::Unsupported {
                operation: operation.0.to_owned(),
            },
            operation.to_string(),
        )
    }
}
