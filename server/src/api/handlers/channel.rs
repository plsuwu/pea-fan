//! Route handlers for publicly-accessible channel-related queries

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use serde::Serialize;
use tracing::instrument;

use crate::api::extractors::{ScoreVariant, ScoreWindowQuery};
use crate::api::server::{ApiResponse, ApiResult, AppState, RouteError};
use crate::db::models::channel::ChannelReplies;
use crate::db::models::leaderboard::TimeWindow;
use crate::db::models::{PaginatedResponse, Pagination};
use crate::db::prelude::LeaderboardRepository;
use crate::db::prelude::Repository;
use crate::db::prelude::{ChannelLeaderboardEntry, Chatter};
use crate::db::prelude::{ChatterId, ChatterRepository};
use crate::db::repositories::leaderboard::ScorePagination;

#[derive(Debug, Serialize)]
pub struct WindowedScores {
    yesterday: i64,
    prev_week: i64,
    prev_month: i64,
    prev_year: i64,
    last_7_days: i64,
    last_30_days: i64,
}

impl WindowedScores {
    const YESTERDAY_IDX: usize = 0;
    const PREV_WEEK_IDX: usize = 1;
    const PREV_MONTH_IDX: usize = 2;
    const PREV_YEAR_IDX: usize = 3;
    const LAST_7_DAYS_IDX: usize = 4;
    const LAST_30_DAYS_IDX: usize = 5;
    // const LAST_24_HOURS_IDX: usize = 6;

    pub fn new(
        yesterday: i64,
        prev_week: i64,
        prev_month: i64,
        prev_year: i64,
        last_7_days: i64,
        last_30_days: i64,
    ) -> Self {
        Self {
            yesterday,
            prev_week,
            prev_month,
            prev_year,
            last_7_days,
            last_30_days,
        }
    }

    /// This method assumes the slice is the correct length.
    pub fn new_from_vec(win: &[i64]) -> Self {
        Self {
            yesterday: win[Self::YESTERDAY_IDX],
            prev_week: win[Self::PREV_WEEK_IDX],
            prev_month: win[Self::PREV_MONTH_IDX],
            prev_year: win[Self::PREV_YEAR_IDX],
            last_7_days: win[Self::LAST_7_DAYS_IDX],
            last_30_days: win[Self::LAST_30_DAYS_IDX],
        }
    }
}

/// Retrieve a list of channel names.
///
/// # Methods
///
/// * GET
///
///     ```http
///     /api/v1/channels/all
///     ```
#[instrument(skip(state))]
pub async fn channel_name_list(State(state): State<Arc<AppState>>) -> ApiResult<Vec<String>> {
    let _guard = state.channels.read().await;
    let channels = _guard.clone();
    
    drop(_guard);
    Ok(ApiResponse::ok(channels))
}

/// Retrieves the global channel leaderboard
///
/// # Methods
///
/// * GET
///
///     ```http
///     /api/v1/channels/leaderboard?limit=[LIMIT]&page=[PAGE]&score_page=0&score_limit=0
///     ```
///
///     Params:
///
///     - `limit`:          number of items on the retrieved page. valid range is `0 <= limit <= MAX_U64`
///     - `page`:           retrieve items starting with `limit * page`. valid range is `0 <= page <= BROADCASTER_COUNT`
///     - `score_page`:     should be set to 0; consumed downstream by SQL queries but not relevant for this function.
///     - `score_limit`:    should be set to 0; consumed downstream by SQL queries but not relevant for this function.
#[instrument(skip(state))]
pub async fn channel_leaderboard(
    Query(param): Query<Pagination>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<PaginatedResponse<ChannelLeaderboardEntry>> {
    let limit = param.limit;
    let offset = param.page * limit;
    let score_limit = param.score_limit;
    let score_offset = param.score_page * score_limit;

    let lb_repo = LeaderboardRepository::new(state.database_pool);
    let segment = lb_repo
        .get_channel_leaderboard(
            limit,
            offset,
            &ScorePagination::new(score_limit, score_offset),
        )
        .await?;

    Ok(ApiResponse::ok(segment))
}

/// Retrieve a channel via `login` along with their associated per-channel leaderboard.
///
/// # Methods
///
/// * GET
///
///     ```http
///     /api/v1/channels/by-login/[LOGIN]?score_page=[SCORE_PAGE]&score_limit=[SCORE_LIMIT]&limit=0&page=0
///     ```
///
///     Params:
///
///     - `score_limit`:    number of items on the retrieved page. valid range is `0 <= score_limit <= MAX_U64`
///     - `score_page`:     retrieve items starting with `score_limit * score_page`. valid range is `0 <= page <= BROADCASTER_COUNT`
///     - `page`:           can be set to 0; consumed downstream by SQL queries but not relevant for this function.
///     - `limit`:          can be set to 0; consumed downstream by SQL queries but not relevant for this function.
#[instrument(skip(state))]
pub async fn by_login(
    State(state): State<Arc<AppState>>,
    Path(login): Path<String>,
    Query(param): Query<Pagination>,
) -> ApiResult<ChannelLeaderboardEntry> {
    let (ch_repo, lb_repo) = (
        ChatterRepository::new(state.database_pool),
        LeaderboardRepository::new(state.database_pool),
    );

    let channel = ch_repo.get_by_login(&login).await?;
    let ch = lb_repo
        .get_single_channel_leaderboard(
            channel.id.into(),
            ScorePagination::new(param.score_limit, param.score_page * param.score_limit),
        )
        .await?
        .ok_or(RouteError::InvalidUser(login))?;

    Ok(ApiResponse::ok(ch))
}

/// Retrieve a channel via its `id`, along with their associated per-channel leaderboard.
///
/// # Methods
///
/// * GET
///
///     ```http
///     /api/v1/channels/by-id/{ID}?score_page=[SCORE_PAGE]&score_limit=[SCORE_LIMIT]&limit=0&page=0
///     ```
///
///     Path:
///     - {ID}:             the ID of a broadcaster.
///
///     Params:
///
///     - `limit`:          number of items on the retrieved page. valid range is `0 <= limit <= MAX_U64`
///     - `page`:           retrieve items starting with `limit * page`. valid range is `0 <= page <= BROADCASTER_COUNT`
///     - `score_page`:     should be set to 0; consumed downstream by SQL queries but not relevant for this function.
///     - `score_limit`:    should be set to 0; consumed downstream by SQL queries but not relevant for this function.
#[instrument(skip(state))]
pub async fn by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(param): Query<Pagination>,
) -> ApiResult<ChannelLeaderboardEntry> {
    let ch = LeaderboardRepository::new(state.database_pool)
        .get_single_channel_leaderboard(
            id.clone().into(),
            ScorePagination::new(param.score_limit, param.score_page * param.score_limit),
        )
        .await?
        .ok_or(RouteError::InvalidUser(id))?;

    Ok(ApiResponse::ok(ch))
}

/// Retrieves a list of those broadcasters where bot responses are enabled.
///
/// # Methods
///
/// * GET
///
///     ```http
///     /api/v1/channels/bot-enabled
///     ```
#[instrument(skip(state))]
pub async fn bot_enabled(State(state): State<Arc<AppState>>) -> ApiResult<Vec<ChannelReplies>> {
    let enabled_channels = sqlx::query_as::<_, ChannelReplies>(
        r#"
        SELECT * FROM reply_configuration 
        WHERE enabled = TRUE
        "#,
    )
    .fetch_all(state.database_pool)
    .await?;

    Ok(ApiResponse::ok(enabled_channels))
}

/// Retrieves a list of the current live channels
///
/// # Methods
///
/// * GET
///
///     ```http
///     /api/v1/channels/live
///     ```
#[instrument(skip(state))]
#[axum::debug_handler]
pub async fn live_channels(State(state): State<Arc<AppState>>) -> ApiResult<Vec<Chatter>> {
    let cached_live_ids = crate::db::redis::get_all_live(&mut state.redis_pool.clone())
        .await
        .unwrap_or_default();

    if let Some(live_ids) = cached_live_ids {
        let broadcasters = ChatterRepository::new(state.database_pool)
            .get_many_by_id(
                &live_ids
                    .into_iter()
                    .map(|id| ChatterId::from(id))
                    .collect::<Vec<_>>(),
            )
            .await?;

        return Ok(ApiResponse::ok(broadcasters));
    };

    Ok(ApiResponse::ok(Vec::new()))
}

/// Retrieves a list of joined IRC channels (as reported by the IRC connection manager; has the
/// potential to be inaccurate but this does is a reasonable vector for helping to diagnose
/// issues).
///
/// Response data is a HashMap where the `key` is the channel status (`missing`/`joined`/`all`)
/// to the channel name in IRC format (`#channel_name`), e.g.:
///
/// ```
/// {
///     "joined": [
///         "#sleepiebug",
///         "#chikogaki",
///         "#hempie",
///         ...
///     ],
///
///     "missing": [
///         "#b0barley",
///         "#kyoharuvt",
///         ...
///     ],
///
///     "all": [
///         "#sleepiebug",
///         "#chikogaki",
///         "#hempie",
///         "#b0barley",
///         "#kyoharuvt",
///         ...
///     ]
/// }
/// ```
///
/// # Methods
///
/// * GET
///
///     ```http
///     /api/v1/channels/irc-joins
///     ```
#[instrument(skip(state))]
pub async fn irc_joins(
    State(state): State<Arc<AppState>>,
) -> ApiResult<HashMap<&'static str, Vec<String>>> {
    let mut output = HashMap::new();
    let mut missing = Vec::new();
    let joined = state.irc_connection.joined_channels().await?;

    let channels = state.channels.read().await.clone();

    let all: Vec<String> = 
        channels
        .iter()
        .map(|name| {
            let ch_name = format!("#{name}");
            if !joined.contains(&ch_name) {
                missing.push(ch_name.clone());
            }

            ch_name
        })
        .collect();

    output.insert("missing", missing);
    output.insert("joined", joined);
    output.insert("all", all);

    Ok(ApiResponse::ok(output))
}

/// Retrieves channel score windows for preset time periods
///
/// # Methods
///
/// * GET
///
///     ```http
///     /api/v1/channels/windows/{id}
///     ```
///
///     Path:
///     - {id}:     the ID of the broadcaster to perform the fetch for
#[instrument(skip(state))]
pub async fn channel_score_windows(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(window): Query<ScoreWindowQuery>,
) -> ApiResult<WindowedScores> {
    let pool = state.database_pool;
    let variant_str = match window.variant {
        ScoreVariant::Channel => "channel",
        ScoreVariant::Chatter => "chatter",
    };

    let res: Result<Vec<i64>, sqlx::Error> = tokio::spawn(async move {
        let mut tx = pool.begin().await?;
        let mut query_results = Vec::new();

        let queries: [String; 6] = [
            TimeWindow::Yesterday.into_query(&variant_str),
            TimeWindow::PrevWeek.into_query(&variant_str),
            TimeWindow::PrevMonth.into_query(&variant_str),
            TimeWindow::PrevYear.into_query(&variant_str),
            TimeWindow::Last7Days.into_query(&variant_str),
            TimeWindow::Last30Days.into_query(&variant_str),
        ];

        for query in queries {
            let q_res = sqlx::query_scalar::<_, i64>(&query)
                .bind(&id)
                .fetch_one(&mut *tx)
                .await
                .unwrap_or(0);

            query_results.push(q_res);
        }

        tx.commit().await?;
        Ok(query_results)
    })
    .await?;
    let windows = WindowedScores::new_from_vec(&res?);

    Ok(ApiResponse::ok(windows))
}
