use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use chrono::Utc;
use http::StatusCode;
use tracing::instrument;

use crate::api::extractors::{UserIdRequest, UserRequest};
use crate::api::handlers::spawn_protected;
use crate::api::server::{ApiResponse, ApiResult, AppState, RouteError};
use crate::api::webhook::StreamGenericRequestType;
use crate::db::models::channel::ChannelReplies;
use crate::db::prelude::{Channel, ChannelId, ChannelRepository};
use crate::db::prelude::{Chatter, ChatterId, ChatterRepository, Repository};
use crate::db::{self, redis};
use crate::util::helix::Helix;
use crate::util::{self, is_user_id};

/// PUT
#[instrument(skip(state))]
pub async fn update_channel_data(State(state): State<Arc<AppState>>) -> ApiResult<()> {
    let _guard = state.channel_ids.read().await;
    let channel_ids = _guard.clone();

    drop(_guard);

    spawn_protected(async move {
        let mut channel_ids = channel_ids
            .clone()
            .into_iter()
            .map(ChatterId::from)
            .collect::<Vec<ChatterId>>();

        // let mut channel_ids = sqlx::query_scalar!(
        //     r#"
        //     SELECT id FROM channel
        //     "#
        // )
        // .fetch_all(state.database_pool)
        // .await?
        // .into_iter()
        // .map(|id| ChatterId(id))
        // .collect::<Vec<_>>();

        util::channel::update_stored_channels(&mut channel_ids, true)
            .await
            .map_err(RouteError::from)
    })
    .await?;

    Ok(ApiResponse::<()>::empty())
}

/// POST
// #[instrument(skip(state))]
pub async fn new_channel(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserRequest>,
) -> ApiResult<String> {
    spawn_protected(async move {
        let helix_user = if is_user_id(&payload.user) {
            crate::util::helix::Helix::fetch_users_by_id(&mut [payload.user]).await?
        } else {
            crate::util::helix::Helix::fetch_users_by_login([payload.user].to_vec()).await?
        }
        .into_iter()
        .nth(0)
        .ok_or(RouteError::GenericStatusCode(
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;

        let chatter = Chatter::from(helix_user);
        ChatterRepository::new(state.database_pool)
            .insert(&chatter)
            .await?;

        let now = Utc::now().naive_utc();
        let channel = Channel {
            id: ChannelId(chatter.id.0.clone()),
            channel_total: 0,
            created_at: now,
            updated_at: now,
        };

        let chan_repo = ChannelRepository::new(state.database_pool);
        chan_repo.insert(&channel).await?;
        chan_repo.new_channel_config(&channel.id).await?;

        tracing::debug!("acquiring write locks");
        let mut _ids = state.channel_ids.write().await;
        let mut _logins = state.channels.write().await;

        tracing::debug!("acquired - writing new data");
        _ids.push(chatter.id.0.clone());
        _logins.push(chatter.login.clone());

        tracing::debug!("dropping locks");
        drop(_logins);
        drop(_ids);

        let res = state
            .irc_connection
            // we prefer to clone in the non-blocking task than in the connection handler i imagine
            .insert_channel(chatter.login.clone())
            .await?;

        tracing::info!(?res, "creating stream state context");

        let channel_id = ChannelId::from(chatter.id.clone());
        Helix::create_subscription(channel_id.clone(), StreamGenericRequestType::Online).await?;
        Helix::create_subscription(channel_id, StreamGenericRequestType::Offline).await?;
        redis::init_stream_states(&mut state.redis_pool.clone(), &vec![chatter.id.0]).await?;

        tracing::info!("channel addition pipeline completed");
        Ok(ApiResponse::ok(chatter.login))
    })
    .await
}

/// GET
#[instrument(skip(state))]
pub async fn get_reply_config(
    State(state): State<Arc<AppState>>,
    Query(param): Query<UserIdRequest>,
) -> ApiResult<Vec<ChannelReplies>> {
    let channel_repo = ChannelRepository::new(state.database_pool);

    if param.id == "all" {
        let all_configs = channel_repo.get_all_reply_configs().await?;
        Ok(ApiResponse::ok(all_configs))
    } else {
        let config = channel_repo.get_reply_config(&param.id).await?;
        Ok(ApiResponse::ok(vec![config]))
    }
}

/// PUT
#[instrument(skip(state))]
pub async fn update_channel_config(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserIdRequest>,
) -> ApiResult<()> {
    spawn_protected(async move {
        let channel_repo = ChannelRepository::new(state.database_pool);
        let id = ChannelId(payload.id);

        channel_repo
            .update_channel_config(&id)
            .await
            .map_err(RouteError::from)
    })
    .await?;

    Ok(ApiResponse::<()>::empty())
}

/// PUT
#[instrument(skip(state))]
pub async fn refresh_channel_state(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserIdRequest>,
) -> ApiResult<()> {
    let ids = if payload.id == "all" {
        ChannelRepository::new(state.database_pool)
            .get_all_channel_ids()
            .await?
    } else {
        vec![payload.id]
    };

    db::redis::init_stream_states(&mut state.redis_pool.clone(), &ids).await?;

    Ok(ApiResponse::<()>::empty())
}
