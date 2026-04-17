use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use tracing::instrument;

use crate::api::extractors::{AliasUpdateRequest, UserIdRequest};
use crate::api::handlers::spawn_protected;
use crate::api::server::{ApiResponse, ApiResult, AppState, RouteError};
use crate::db::prelude::ChatterId;
use crate::db::redis::migrator::{self, process_alias_migration};

/// PUT
#[instrument(skip(payload))]
pub async fn update_chatter_from_cache(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AliasUpdateRequest>,
) -> ApiResult<()> {
    spawn_protected(async move {
        let current_name = payload.current;
        let aliases = payload.historic;

        process_alias_migration(
            state.redis_pool.clone(),
            state.database_pool,
            &current_name,
            &aliases,
        )
        .await
        .map_err(RouteError::from)
    })
    .await?;

    Ok(ApiResponse::<()>::empty())
}

/// DELETE
#[instrument(skip(state))]
pub async fn clear_chatter_scores(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserIdRequest>,
) -> ApiResult<()> {
    spawn_protected(async move {
        let database_handler = migrator::io::PgHandler(state.database_pool);
        let payload_id_clone = payload.id.clone();

        database_handler
            .clear_scores_for_chatter(&ChatterId(payload_id_clone))
            .await
            .map_err(RouteError::from)
    })
    .await?;
    
    Ok(ApiResponse::<()>::empty())
}
