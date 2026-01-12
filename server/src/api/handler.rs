use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::{Json, debug_handler};
use tracing::instrument;

use crate::api::server::{AppState, JsonResult, RouteError};
use crate::db::models::{PaginatedResponse, Pagination};
use crate::db::prelude::{ChannelLeaderboardEntry, Repository};
use crate::db::prelude::{ChatterLeaderboardEntry, ChatterRepository, LeaderboardRepository};
use crate::util::helix::{Helix, HelixUser};

#[instrument(skip(state))]
pub async fn global_channels(
    Query(param): Query<Pagination>,
    State(state): State<Arc<AppState>>,
) -> JsonResult<PaginatedResponse<ChannelLeaderboardEntry>> {
    let limit = param.limit;
    let offset = param.page * limit;

    let lb_repo = LeaderboardRepository::new(state.db_pool);
    let segment = lb_repo.get_channel_leaderboard(limit, offset).await?;

    Ok(Json(segment))
}

#[instrument(skip(state))]
pub async fn channel_by_login(
    State(state): State<Arc<AppState>>,
    Path(login): Path<String>,
) -> JsonResult<ChannelLeaderboardEntry> {
    let (ch_repo, lb_repo) = (
        ChatterRepository::new(state.db_pool),
        LeaderboardRepository::new(state.db_pool),
    );

    let channel = ch_repo.get_by_login(login.clone()).await?;
    match lb_repo
        .get_single_channel_leaderboard(channel.id.into())
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
) -> JsonResult<ChannelLeaderboardEntry> {
    match LeaderboardRepository::new(state.db_pool)
        .get_single_channel_leaderboard(id.clone().into())
        .await?
    {
        Some(ch) => Ok(Json(ch)),
        None => Err(RouteError::InvalidUser(id)),
    }
}

#[instrument(skip(state))]
pub async fn global_chatters(
    Query(param): Query<Pagination>,
    State(state): State<Arc<AppState>>,
) -> JsonResult<PaginatedResponse<ChatterLeaderboardEntry>> {
    let limit = param.limit;
    let offset = param.page * limit;

    let lb_repo = LeaderboardRepository::new(state.db_pool);
    let segment = lb_repo.get_chatter_leaderboard(limit, offset).await?;

    Ok(Json(segment))
}

#[instrument(skip(state))]
pub async fn chatter_by_login(
    State(state): State<Arc<AppState>>,
    Path(login): Path<String>,
) -> JsonResult<ChatterLeaderboardEntry> {
    let (ch_repo, lb_repo) = (
        ChatterRepository::new(state.db_pool),
        LeaderboardRepository::new(state.db_pool),
    );

    let chatter = ch_repo.get_by_login(login.clone()).await?;
    match lb_repo.get_single_chatter_leaderboard(chatter.id).await? {
        Some(ch) => Ok(Json(ch)),
        None => Err(RouteError::InvalidUser(login)),
    }
}

#[instrument(skip(state))]
pub async fn chatter_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> JsonResult<ChatterLeaderboardEntry> {
    match LeaderboardRepository::new(state.db_pool)
        .get_single_chatter_leaderboard(id.clone().into())
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
