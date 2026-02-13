use std::error::Error;
use std::sync::Arc;

use axum::body::{Body, HttpBody};
use axum::extract::{self, Path, Query, State};
use axum::{Json, debug_handler};
use http::{HeaderMap, StatusCode};
use redis::RedisError;
use serde_json::Value;
use tokio::sync::oneshot;
use tracing::instrument;

use crate::api::middleware::verify_external::VerifiedBody;
use crate::api::server::{AppState, JsonResult, RouteError};
use crate::db::models::{PaginatedResponse, Pagination};
use crate::db::prelude::{ChannelLeaderboardEntry, Repository};
use crate::db::prelude::{ChatterLeaderboardEntry, ChatterRepository, LeaderboardRepository};
use crate::db::redis::migrator::{
    Aliases, Migrator, update_historic_channel, update_historic_user,
};
use crate::db::redis::redis_pool::RedisErr;
use crate::db::repositories::leaderboard::ScorePagination;
use crate::util::helix::{Helix, HelixUser};

#[instrument(skip(state))]
pub async fn global_channels(
    Query(param): Query<Pagination>,
    State(state): State<Arc<AppState>>,
) -> JsonResult<PaginatedResponse<ChannelLeaderboardEntry>> {
    let limit = param.limit;
    let offset = param.page * limit;
    let score_limit = param.score_limit;
    let score_offset = param.score_page * score_limit;

    let lb_repo = LeaderboardRepository::new(state.db_pool);
    let segment = lb_repo
        .get_channel_leaderboard(
            limit,
            offset,
            &ScorePagination::new(score_limit, score_offset),
        )
        .await?;

    Ok(Json(segment))
}

#[instrument(skip(state))]
pub async fn channel_by_login(
    State(state): State<Arc<AppState>>,
    Path(login): Path<String>,
    Query(param): Query<Pagination>,
) -> JsonResult<ChannelLeaderboardEntry> {
    let (ch_repo, lb_repo) = (
        ChatterRepository::new(state.db_pool),
        LeaderboardRepository::new(state.db_pool),
    );

    let channel = ch_repo.get_by_login(login.clone()).await?;
    match lb_repo
        .get_single_channel_leaderboard(
            channel.id.into(),
            ScorePagination::new(param.score_limit, param.score_page * param.score_limit),
        )
        .await?
    {
        Some(ch) => Ok(Json(ch)),
        None => Err(RouteError::InvalidUser(login)),
    }
}

#[instrument(skip(state))]
pub async fn channel_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(param): Query<Pagination>,
) -> JsonResult<ChannelLeaderboardEntry> {
    match LeaderboardRepository::new(state.db_pool)
        .get_single_channel_leaderboard(
            id.clone().into(),
            ScorePagination::new(param.score_limit, param.score_page * param.score_limit),
        )
        .await?
    {
        Some(ch) => Ok(Json(ch)),
        None => Err(RouteError::InvalidUser(id)),
    }
}

#[instrument(skip(state))]
pub async fn irc_joins(State(state): State<Arc<AppState>>) -> JsonResult<Vec<String>> {
    let tx = &state.tx_client;
    let msg = String::from("irc_joins");

    let (tx_oneshot, rx_oneshot) = oneshot::channel::<Vec<String>>();

    tx.send((msg, tx_oneshot))?;
    match rx_oneshot.await {
        Ok(data) => Ok(Json(data)),
        Err(e) => {
            tracing::error!(error = ?e, "failure during irc_joins query");
            Err(e.into())
        }
    }
}

#[instrument(skip(state))]
pub async fn global_chatters(
    Query(param): Query<Pagination>,
    State(state): State<Arc<AppState>>,
) -> JsonResult<PaginatedResponse<ChatterLeaderboardEntry>> {
    let limit = param.limit;
    let offset = param.page * limit;

    let score_limit = param.score_limit;
    let score_offset = param.score_page * score_limit;

    let lb_repo = LeaderboardRepository::new(state.db_pool);
    let segment = lb_repo
        .get_chatter_leaderboard(
            limit,
            offset,
            ScorePagination::new(score_limit, score_offset),
        )
        .await?;

    Ok(Json(segment))
}

#[instrument(skip(state))]
pub async fn chatter_by_login(
    State(state): State<Arc<AppState>>,
    Path(login): Path<String>,
    Query(param): Query<Pagination>,
) -> JsonResult<ChatterLeaderboardEntry> {
    let (ch_repo, lb_repo) = (
        ChatterRepository::new(state.db_pool),
        LeaderboardRepository::new(state.db_pool),
    );

    let chatter = ch_repo.get_by_login(login.clone()).await?;
    match lb_repo
        .get_single_chatter_leaderboard(
            chatter.id,
            ScorePagination::new(param.score_limit, param.score_page * param.score_limit),
        )
        .await?
    {
        Some(ch) => Ok(Json(ch)),
        None => Err(RouteError::InvalidUser(login)),
    }
}

#[instrument(skip(state))]
pub async fn chatter_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(param): Query<Pagination>,
) -> JsonResult<ChatterLeaderboardEntry> {
    match LeaderboardRepository::new(state.db_pool)
        .get_single_chatter_leaderboard(
            id.clone().into(),
            ScorePagination::new(param.score_limit, param.score_page * param.score_limit),
        )
        .await?
    {
        Some(ch) => Ok(Json(ch)),
        None => Err(RouteError::InvalidUser(id)),
    }
}

#[instrument]
#[debug_handler]
pub async fn helix_user_by_login(Path(login): Path<String>) -> JsonResult<Vec<HelixUser>> {
    let logins = vec![login];
    let helix_user = Helix::fetch_users_by_login(logins).await?;

    Ok(Json(helix_user))
}

#[instrument]
#[debug_handler]
pub async fn helix_user_by_id(Path(id): Path<String>) -> JsonResult<Vec<HelixUser>> {
    let mut ids = vec![id];
    let helix_user = Helix::fetch_users_by_id(&mut ids).await?;

    Ok(Json(helix_user))
}

#[instrument(skip(payload))]
pub async fn update_chatter_in_cache(
    Json(payload): Json<Value>,
) -> Result<Json<String>, StatusCode> {
    let json_body: Aliases =
        serde_json::from_value::<Aliases>(payload).map_err(|_| StatusCode::BAD_REQUEST)?;

    match update_historic_user(json_body).await {
        Ok(_) => Ok(Json(String::from("OK"))),
        Err(e) => return Ok(Json(e.to_string())),
    }
}

#[instrument(skip(payload))]
pub async fn update_channel_in_cache(
    Json(payload): Json<Value>,
) -> Result<Json<String>, StatusCode> {
    tracing::debug!(?payload, "RX post");
    let json_body: Aliases =
        serde_json::from_value::<Aliases>(payload).map_err(|_| StatusCode::BAD_REQUEST)?;

    match update_historic_user(json_body.clone()).await {
        Ok(_) => (),
        Err(e) => match e {
            RedisErr::UpdateEmpty => {
                // avoid bailing out if the broadcaster's total as a user/chatter is 0
                tracing::warn!("nothing to update for broadcaster user");
            }
            _ => {
                return Ok(Json(e.to_string()));
            }
        },
    };

    match update_historic_channel(json_body).await {
        Ok(_) => Ok(Json(String::from("OK"))),
        Err(e) => return Ok(Json(e.to_string())),
    }
}

#[instrument(skip(_headers))]
pub async fn run_cache_migration(_headers: HeaderMap) -> Result<Json<String>, StatusCode> {
    // this blocks for ages so maybe we run these updater functions on a separate thread
    match Migrator::new().process().await {
        Ok(_) => Ok(Json(String::from("OK"))),
        Err(e) => return Ok(Json(e.to_string())),
    }
}
