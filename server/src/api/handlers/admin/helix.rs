use std::sync::Arc;

use axum::extract::{Path, State};
use tracing::instrument;

use crate::api::handlers::spawn_protected;
use crate::api::server::stream_online_hook_handler;
use crate::api::server::{ApiResponse, ApiResult, AppState, RouteError};
use crate::api::webhook::{HelixDataGeneric, SubscriptionGenericData};
use crate::util::helix::{Helix, HelixUser};

/// GET
#[instrument]
pub async fn user_by_login(Path(login): Path<String>) -> ApiResult<Vec<HelixUser>> {
    let logins = vec![login];
    let user = Helix::fetch_users_by_login(logins).await?;

    Ok(ApiResponse::ok(user))
}

/// GET
#[instrument]
pub async fn user_by_id(Path(id): Path<String>) -> ApiResult<Vec<HelixUser>> {
    let mut ids = vec![id];
    let user = Helix::fetch_users_by_id(&mut ids).await?;

    Ok(ApiResponse::ok(user))
}

/// DELETE
#[instrument(skip(state))]
pub async fn delete_hooks(State(state): State<Arc<AppState>>) -> ApiResult<usize> {
    let result = spawn_protected(async move {
        let ids = state.channel_ids.read().await.clone();
        crate::db::redis::clear_stream_states(&mut state.redis_pool.clone(), &ids)
            .await
            .map_err(RouteError::from)?;

        tracing::info!("removed all channel states from redis cache");

        let active_hooks = Helix::get_active_subscriptions().await?;
        tracing::debug!(?active_hooks, "active_hooks");

        if !active_hooks.is_empty() {
            tracing::debug!("active_hooks populated - deleting...");
            Helix::delete_subscriptions(&active_hooks)
                .await
                .map_err(RouteError::from)?;

            let hooks_count = active_hooks.len();
            Ok(hooks_count)
        } else {
            Ok(0usize)
        }
    })
    .await?;

    Ok(ApiResponse::ok(result))
}

/// PUT
#[instrument(skip(state))]
pub async fn reset_hooks(State(state): State<Arc<AppState>>) -> ApiResult<()> {
    spawn_protected(async move {
        let ids = state.channel_ids.read().await.clone();
        stream_online_hook_handler(&ids, state.redis_pool.clone()).await
    })
    .await?;

    Ok(ApiResponse::<()>::empty())
}

/// GET
#[instrument(skip(state))]
pub async fn active_hooks(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Vec<(String, SubscriptionGenericData)>> {
    let channel_data = spawn_protected(async move {
        let active_data: HelixDataGeneric<SubscriptionGenericData> =
            serde_json::from_value(Helix::get_active_subscriptions_raw().await?)
                .map_err(RouteError::from)?;

        let mut channel_data = Vec::new();
        let mut tx = state.database_pool.begin().await?;

        for sub in active_data.data.into_iter() {
            let broadcaster_id = &sub.condition.broadcaster_user_id;
            let login: String = sqlx::query_scalar!(
                r#"
                SELECT login FROM chatter
                WHERE id = $1
                "#,
                broadcaster_id,
            )
            .fetch_one(&mut *tx)
            .await?;

            channel_data.push((login, sub));
        }

        tx.commit().await?;
        Ok(channel_data)
    })
    .await?;

    Ok(ApiResponse::ok(channel_data))
}
