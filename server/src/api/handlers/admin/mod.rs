pub mod channel;

pub mod helix;

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use http::StatusCode;
use sqlx::{Pool, Postgres};
use tracing::instrument;

use crate::api::extractors::{TOTPRequest, TOTPResponse};
use crate::api::middleware::verify_internal::SessionToken;
use crate::api::server::{ApiResponse, ApiResult, AppState, RouteError};

/// Create a new admin session token and store it in the database. Return the token to the caller
async fn create_session(database_pool: &'static Pool<Postgres>) -> Result<String, RouteError> {
    let session = SessionToken::new_token();
    SessionToken::store_token(database_pool, &session)
        .await
        .map_err(RouteError::from)?;

    Ok(session)
}


/// This handler should be used from a middleware-protected route; by returning a response, the
/// given token matches some session token where the token is not yet expired
#[instrument]
pub async fn validate_session() -> ApiResult<&'static str> {
    Ok(ApiResponse::ok("ok"))
}

/// POST
#[instrument(skip(state))]
pub async fn new_session(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TOTPRequest>,
) -> ApiResult<TOTPResponse> {
    tracing::debug!(?payload.token, "RECEIVED TOTP TOKEN");
    let mut guard = state.totp_handler.lock().await;
    let is_valid = guard.totp_cmp(&payload.token).map_err(|e| {
        tracing::error!(error = ?e, "unknown error during TOTP validation");
        RouteError::GenericStatusCode(StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    if is_valid {
        let session = create_session(state.database_pool).await?;
        return Ok(ApiResponse::ok(TOTPResponse::new(true, session)));
    };

    Ok(ApiResponse::ok(TOTPResponse::new(false, String::default())))
}

/// PUT
#[instrument(skip(state))]
pub async fn reset_irc(State(state): State<Arc<AppState>>) -> ApiResult<()> {
    let supervisor = &state.irc_connection;

    supervisor
        .connection
        .reset_tx
        .send(())
        .await
        .map_err(RouteError::from)?;

    Ok(ApiResponse::<()>::empty())
}
