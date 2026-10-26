//! HTTP routes. Every `/api` route except the stream upgrade checks its bearer token here.
//! The stream checks the same token, and also accepts it as a query parameter for browsers.
//!
//! Status codes: a missing or unknown token is 401. A valid token whose principal may not use
//! the route is 403. That is an auth boundary, so an agent token on `POST /api/commands` gets 403
//! and never reaches the origin. A command the origin refuses is 409, with the `Rejection` as the
//! body.

use std::path::PathBuf;

use acan_core::{Command, HumanCmd, MemberId, Rejection, Seq, Stamped};
use acan_llm::{ModelClient, UsageTotals};
use acan_scan::Plant;
use axum::extract::{Query, State};
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use tower_http::services::ServeDir;

use crate::auth::Principal;
use crate::host::{Host, SubmitError};
use crate::stream;

/// Builds the router. A `web` directory is served for every path the API does not claim.
pub fn router<C: ModelClient + 'static>(host: Host<C>, web: Option<PathBuf>) -> Router {
    let router = Router::new()
        .route("/health", get(health))
        .route("/api/commands", post(post_command::<C>))
        .route("/api/events", get(get_events::<C>))
        .route("/api/plant", get(get_plant::<C>))
        .route("/api/scan", post(post_scan::<C>))
        .route("/api/usage", get(get_usage::<C>))
        .route("/api/stream", get(stream::upgrade::<C>));
    let router = match web {
        Some(dir) => router.fallback_service(ServeDir::new(dir)),
        None => router,
    };
    router.with_state(host)
}

/// A failed request, with the status it maps to.
#[derive(Debug)]
pub enum ApiError {
    /// The bearer token is missing or unknown.
    Unauthorized,
    /// The token is valid, but its principal may not use this route.
    Forbidden(&'static str),
    /// The origin refused the command. The body is the [`Rejection`].
    Rejected(Rejection),
    /// Something on the server failed. The detail goes to the log, not the response.
    Internal(anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "unauthorized" })),
            )
                .into_response(),
            Self::Forbidden(why) => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "forbidden", "why": why })),
            )
                .into_response(),
            Self::Rejected(rejection) => (StatusCode::CONFLICT, Json(rejection)).into_response(),
            Self::Internal(err) => {
                tracing::error!(error = ?err, "request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "internal" })),
                )
                    .into_response()
            }
        }
    }
}

impl From<SubmitError> for ApiError {
    fn from(err: SubmitError) -> Self {
        match err {
            SubmitError::Rejected(rejection) => Self::Rejected(rejection),
            SubmitError::Internal(err) => Self::Internal(err),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err)
    }
}

/// The bearer token in an `Authorization: Bearer` header, if there is one.
pub(crate) fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

/// The principal a token acts for.
pub(crate) fn authenticate<C: ModelClient>(
    host: &Host<C>,
    token: Option<&str>,
) -> Result<Principal, ApiError> {
    token
        .and_then(|token| host.tokens().principal(token))
        .cloned()
        .ok_or(ApiError::Unauthorized)
}

fn member<C: ModelClient>(host: &Host<C>, headers: &HeaderMap) -> Result<MemberId, ApiError> {
    match authenticate(host, bearer(headers))? {
        Principal::Member(member) => Ok(member),
        Principal::Agent(_) => Err(ApiError::Forbidden("only members may do this over HTTP")),
    }
}

async fn health() -> &'static str {
    "ok"
}

async fn post_command<C: ModelClient + 'static>(
    State(host): State<Host<C>>,
    headers: HeaderMap,
    Json(cmd): Json<HumanCmd>,
) -> Result<Json<Vec<Stamped>>, ApiError> {
    let member = member(&host, &headers)?;
    let events = host.submit(Command::Human { member, cmd }).await?;
    Ok(Json(events))
}

#[derive(Deserialize)]
struct EventsQuery {
    #[serde(default)]
    from: u64,
}

async fn get_events<C: ModelClient + 'static>(
    State(host): State<Host<C>>,
    headers: HeaderMap,
    Query(query): Query<EventsQuery>,
) -> Result<Json<Vec<Stamped>>, ApiError> {
    authenticate(&host, bearer(&headers))?;
    let events = host.store().load_from(Seq::new(query.from)).await?;
    Ok(Json(events))
}

async fn get_plant<C: ModelClient + 'static>(
    State(host): State<Host<C>>,
    headers: HeaderMap,
) -> Result<Json<Plant>, ApiError> {
    authenticate(&host, bearer(&headers))?;
    let scan = host.scan();
    Ok(Json(scan.plant.clone()))
}

async fn post_scan<C: ModelClient + 'static>(
    State(host): State<Host<C>>,
    headers: HeaderMap,
) -> Result<Json<Vec<Stamped>>, ApiError> {
    member(&host, &headers)?;
    let events = host.rescan().await?;
    Ok(Json(events))
}

async fn get_usage<C: ModelClient + 'static>(
    State(host): State<Host<C>>,
    headers: HeaderMap,
) -> Result<Json<Vec<UsageTotals>>, ApiError> {
    authenticate(&host, bearer(&headers))?;
    Ok(Json(host.ledger().snapshot()))
}
