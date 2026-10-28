//! The scope-first MCP tools: get_scope, get_rulings, post_readback, contest and get_receipt.
//!
//! Each tool authorizes the principal, checks the scope revision and evidence it names against
//! the confirmed ones, and answers with the `acan-mcp/1` envelopes. An agent token may read and
//! contest; it never confirms a scope, appends a ruling or retires a probe. No route or MCP
//! tool reaches this module until its owner wires it.
//!
//! Every function here returns [`McpError`] with code `unsupported` until its owner implements
//! it.

use acan_intent::{
    ContestRequest, ContestResponse, GetReceiptRequest, GetReceiptResponse, GetRulingsRequest,
    GetRulingsResponse, GetScopeRequest, GetScopeResponse, McpError, PostReadbackRequest,
    PostReadbackResponse, Unsupported,
};
use acan_llm::ModelClient;

use crate::auth::Principal;
use crate::host::Host;

/// The confirmed scope of the caller's order.
///
/// # Errors
///
/// Returns [`McpError`] when the caller may not read it or the scope is not confirmed.
pub(crate) async fn get_scope<C: ModelClient>(
    _host: &Host<C>,
    _principal: &Principal,
    _request: &GetScopeRequest,
) -> Result<GetScopeResponse, McpError> {
    Err(McpError::unsupported(Unsupported("API01: get_scope")))
}

/// The rulings that apply to the caller's order, enforced or not.
///
/// # Errors
///
/// Returns [`McpError`] when the caller may not read them.
pub(crate) async fn get_rulings<C: ModelClient>(
    _host: &Host<C>,
    _principal: &Principal,
    _request: &GetRulingsRequest,
) -> Result<GetRulingsResponse, McpError> {
    Err(McpError::unsupported(Unsupported("API01: get_rulings")))
}

/// Records the agent's readback and predictions against the confirmed scope revision.
///
/// # Errors
///
/// Returns [`McpError`] when the scope revision is stale or the caller may not post.
pub(crate) async fn post_readback<C: ModelClient>(
    _host: &Host<C>,
    _principal: &Principal,
    _request: &PostReadbackRequest,
) -> Result<PostReadbackResponse, McpError> {
    Err(McpError::unsupported(Unsupported("API01: post_readback")))
}

/// Files a contest against one discrepancy episode.
///
/// # Errors
///
/// Returns [`McpError`] when the episode does not exist, the evidence is stale or the caller may
/// not contest.
pub(crate) async fn contest<C: ModelClient>(
    _host: &Host<C>,
    _principal: &Principal,
    _request: &ContestRequest,
) -> Result<ContestResponse, McpError> {
    Err(McpError::unsupported(Unsupported("API01: contest")))
}

/// The receipt of the caller's latest prepared candidate.
///
/// # Errors
///
/// Returns [`McpError`] when there is no receipt or the caller may not read it.
pub(crate) async fn get_receipt<C: ModelClient>(
    _host: &Host<C>,
    _principal: &Principal,
    _request: &GetReceiptRequest,
) -> Result<GetReceiptResponse, McpError> {
    Err(McpError::unsupported(Unsupported("API01: get_receipt")))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::str::FromStr;

    use acan_core::{MemberId, Origin, Timestamp, WorldId, WorldRef};
    use acan_intent::McpErrorCode;
    use acan_llm::{Stub, UsageLedger};
    use serde::de::DeserializeOwned;

    use super::*;
    use crate::auth::Tokens;
    use crate::host::HostParts;
    use crate::store::EventStore;

    fn fixture<T: DeserializeOwned>(json: &str) -> T {
        serde_json::from_str(json).expect("the request fixture decodes")
    }

    fn assert_unsupported<T: std::fmt::Debug>(result: Result<T, McpError>, operation: &str) {
        match result {
            Err(McpError {
                error: McpErrorCode::Unsupported { operation: name },
                ..
            }) => assert_eq!(name, operation),
            other => panic!("{operation} must fail closed, got {other:?}"),
        }
    }

    async fn host(dir: &tempfile::TempDir) -> Host<Stub> {
        let state = dir.path().join(".acan");
        fs::create_dir_all(&state).expect("state directory");
        let owner = MemberId::from_str("owner").expect("valid member id");
        let store = EventStore::open(&state.join("world.db"))
            .await
            .expect("event store");
        let world = WorldRef {
            id: WorldId::from_str("test").expect("valid world id"),
            repo: dir.path().display().to_string(),
        };
        let (origin, opened) = Origin::open(world, owner.clone(), Timestamp::from_millis(1));
        store
            .append(&[opened])
            .await
            .expect("the open event stores");
        let (tokens, _) = Tokens::load_or_create(&state, &owner).expect("tokens");
        let (_shutdown, shutdown) = tokio::sync::watch::channel(false);
        Host::new(HostParts {
            origin,
            store,
            tokens,
            scan: crate::index::scan(dir.path()).expect("scan"),
            ledger: UsageLedger::default(),
            client: Stub::default(),
            repo: dir.path().to_path_buf(),
            shutdown,
            git: None,
            witness_http_base_url: None,
            witness_concurrency: crate::DEFAULT_WITNESS_CONCURRENCY,
        })
    }

    #[tokio::test]
    async fn every_intent_tool_fails_closed() {
        let dir = tempfile::tempdir().expect("temporary repository");
        let host = host(&dir).await;
        let owner = Principal::Member(MemberId::from_str("owner").expect("valid member id"));
        let scope: GetScopeRequest = fixture(include_str!(
            "../../acan-intent/fixtures/mcp-get-scope-request.json"
        ));
        assert_unsupported(get_scope(&host, &owner, &scope).await, "API01: get_scope");
        let rulings: GetRulingsRequest = fixture(include_str!(
            "../../acan-intent/fixtures/mcp-get-rulings-request.json"
        ));
        assert_unsupported(
            get_rulings(&host, &owner, &rulings).await,
            "API01: get_rulings",
        );
        let readback: PostReadbackRequest = fixture(include_str!(
            "../../acan-intent/fixtures/mcp-post-readback-request.json"
        ));
        assert_unsupported(
            post_readback(&host, &owner, &readback).await,
            "API01: post_readback",
        );
        let filed: ContestRequest = fixture(include_str!(
            "../../acan-intent/fixtures/mcp-contest-request.json"
        ));
        assert_unsupported(contest(&host, &owner, &filed).await, "API01: contest");
        let receipt: GetReceiptRequest = fixture(include_str!(
            "../../acan-intent/fixtures/mcp-get-receipt-request.json"
        ));
        assert_unsupported(
            get_receipt(&host, &owner, &receipt).await,
            "API01: get_receipt",
        );

        host.shutdown().await.expect("the writer stops");
    }
}
