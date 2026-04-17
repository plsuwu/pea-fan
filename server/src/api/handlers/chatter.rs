//! Route handlers for publicly-accessible chatter-related queries

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use tracing::instrument;

use crate::api::server::{ApiResponse, ApiResult, AppState, RouteError};
use crate::db::models::chatter::ChatterSearchResult;
use crate::db::models::{PaginatedResponse, Pagination};
use crate::db::prelude::{ChatterId, Repository};
use crate::db::prelude::{ChatterLeaderboardEntry, ChatterRepository, LeaderboardRepository};
use crate::util::is_user_id;

/// Query the database for a chatter given their login or ID.
///
/// Uses Postgres trigram matching to fuzzily match user logins to provided queries; if queried by ID,
/// it must match a chatter's ID exactly.
///
/// # Methods
///
/// * GET
///
///     ```http
///     /api/v1/chatter/search/{login_or_id}
///     ```
///
///     Path:
///     - `login_or_id`: chatter `login` or `id`; a `login` does not have to be exact. 9-character, digit-only strings are interpreted as an `id` and must match exactly.
#[instrument(skip(state))]
pub async fn search(
    State(state): State<Arc<AppState>>,
    Path(query): Path<String>,
) -> ApiResult<(Vec<ChatterSearchResult>, i64)> {
    let chatter_repo = ChatterRepository::new(state.database_pool);

    // I don't think Twitch lets you have a number-only login (?)
    let result = if is_user_id(&query) {
        tracing::debug!(?query, "heuristically determined query to be an ID");
        let chatter_id = ChatterId::from(query.clone());

        if let Some(chatter) = chatter_repo.get_by_id(&chatter_id).await?
            && let Some(ranking) = LeaderboardRepository::new(state.database_pool)
                .get_chatter_rank(&chatter_id)
                .await?
        {
            vec![ChatterSearchResult {
                id: chatter.id.to_string(),
                name: chatter.name,
                login: chatter.login,
                color: chatter.color,
                image: chatter.image,
                total: chatter.total,
                ranking: ranking,
                similarity_score: 1.0,
            }]
        } else {
            tracing::debug!(?query, "failed to find matching chatter");
            Vec::new()
        }
    } else {
        tracing::debug!(?query, "query is not an ID");
        chatter_repo.search_by_login(&query).await?
    };

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM chatter
        "#,
    )
    .fetch_one(state.database_pool)
    .await?
    .unwrap_or(0);

    Ok(ApiResponse::ok((result, total)))
}

/// Retrieves the global chatter leaderboard
///
/// # Methods
///
/// * GET
///
///     ```http
///     /api/v1/chatter/leaderboard?score_page=[SCORE_PAGE]&score_limit=[SCORE_LIMIT]&limit=0&page=0
///     ```
///
///     Path:
///     - {ID}:             the ID of a chatter.
///
///     Params:
///
///     - `limit`:          number of items on the retrieved page. valid range is `0 <= limit <= MAX_U64`
///     - `page`:           retrieve items starting with `limit * page`. valid range is `0 <= page <= MAX_U64`
///     - `score_page`:     should be set to 0; consumed downstream by SQL queries but not relevant for this function.
///     - `score_limit`:    should be set to 0; consumed downstream by SQL queries but not relevant for this function.
#[instrument(skip(state))]
pub async fn chatter_leaderboard(
    Query(param): Query<Pagination>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<PaginatedResponse<ChatterLeaderboardEntry>> {
    let limit = param.limit;
    let offset = param.page * limit;

    let lb_repo = LeaderboardRepository::new(state.database_pool);
    let segment = lb_repo.get_chatter_leaderboard(limit, offset).await?;

    Ok(ApiResponse::ok(segment))
}

/// Retrieve a chatter via `login`, along with the associated per-channel leaderboard.
///
/// # Methods
///
/// * GET
///
///     ```http
///     /api/v1/chatter/by-login/{login}
///     ```
///
///     Path:
///     - {login}:             the login of a chatter.
#[instrument(skip(state))]
pub async fn by_login(
    State(state): State<Arc<AppState>>,
    Path(login): Path<String>,
) -> ApiResult<ChatterLeaderboardEntry> {
    let (ch_repo, lb_repo) = (
        ChatterRepository::new(state.database_pool),
        LeaderboardRepository::new(state.database_pool),
    );

    let chatter = ch_repo.get_by_login(&login).await?;
    let ch = lb_repo
        .get_single_chatter_leaderboard(chatter.id.clone())
        .await?
        .ok_or(RouteError::InvalidUser(chatter.id.0))?;

    Ok(ApiResponse::ok(ch))
}

/// Retrieve a chatter via `id`, along with the associated per-channel leaderboard.
///
/// # Methods
///
/// * GET
///
///     ```http
///     /api/v1/chatter/by-id/{id}
///     ```
///
///     Path:
///     - {id}:             the id of a chatter.
#[instrument(skip(state))]
pub async fn by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<ChatterLeaderboardEntry> {
    let ch = LeaderboardRepository::new(state.database_pool)
        .get_single_chatter_leaderboard(id.clone().into())
        .await?
        .ok_or(RouteError::InvalidUser(id))?;

    Ok(ApiResponse::ok(ch))
}
