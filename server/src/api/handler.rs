use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::{Json, debug_handler};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;

use crate::api::middleware::verify_internal::SessionToken;
use crate::api::server::{AppState, JsonResult, RouteError, stream_online_hook_handler};
use crate::db::PgError;
use crate::db::models::channel::ChannelReplies;
use crate::db::models::chatter::ChatterSearchResult;
use crate::db::models::leaderboard::TimeWindow;
use crate::db::models::{PaginatedResponse, Pagination};
use crate::db::prelude::{ChannelId, ChannelRepository, Chatter, ChatterId, Tx};
use crate::db::prelude::{ChannelLeaderboardEntry, Repository};
use crate::db::prelude::{ChatterLeaderboardEntry, ChatterRepository, LeaderboardRepository};
use crate::db::redis::migrator::{self, process_alias_migration};
use crate::db::repositories::leaderboard::ScorePagination;
use crate::util;
use crate::util::helix::{Helix, HelixErr, HelixUser};

#[instrument(skip(state))]
pub async fn global_channels(
    Query(param): Query<Pagination>,
    State(state): State<Arc<AppState>>,
) -> JsonResult<PaginatedResponse<ChannelLeaderboardEntry>> {
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

    Ok(Json(segment))
}

#[instrument(skip(state))]
pub async fn channel_by_login(
    State(state): State<Arc<AppState>>,
    Path(login): Path<String>,
    Query(param): Query<Pagination>,
) -> JsonResult<ChannelLeaderboardEntry> {
    let (ch_repo, lb_repo) = (
        ChatterRepository::new(state.database_pool),
        LeaderboardRepository::new(state.database_pool),
    );

    let channel = ch_repo.get_by_login(&login).await?;
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
    match LeaderboardRepository::new(state.database_pool)
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
pub async fn all_channels(State(state): State<Arc<AppState>>) -> JsonResult<Vec<String>> {
    Ok(Json(state.channels.clone()))
}

#[instrument(skip(state))]
pub async fn bot_enabled_channels(
    State(state): State<Arc<AppState>>,
) -> JsonResult<Vec<ChannelReplies>> {
    let enabled_channels = sqlx::query_as::<_, ChannelReplies>(
        r#"
        SELECT * FROM reply_configuration 
        WHERE enabled = TRUE
        "#,
    )
    .fetch_all(state.database_pool)
    .await?;

    Ok(Json(enabled_channels))
}

#[instrument(skip(state))]
#[axum::debug_handler]
pub async fn live_channels(State(state): State<Arc<AppState>>) -> JsonResult<Vec<Chatter>> {
    let live_ids = crate::db::redis::get_all_live(&mut state.redis_pool.clone())
        .await
        .unwrap_or(Vec::new());

    if live_ids.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let broadcasters = ChatterRepository::new(state.database_pool)
        .get_many_by_id(
            &live_ids
                .into_iter()
                .map(|id| ChatterId::from(id))
                .collect::<Vec<_>>(),
        )
        .await?;

    Ok(Json(broadcasters))
}

#[instrument(skip(state))]
pub async fn irc_joins(
    State(state): State<Arc<AppState>>,
) -> JsonResult<HashMap<&'static str, Vec<String>>> {
    let mut output = HashMap::new();
    let mut missing = Vec::new();
    let joined = state.irc_connection.joined_channels().await?;

    let all: Vec<String> = state
        .channels
        .clone()
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

    Ok(Json(output))
}

#[instrument(skip(state))]
pub async fn global_chatters(
    Query(param): Query<Pagination>,
    State(state): State<Arc<AppState>>,
) -> JsonResult<PaginatedResponse<ChatterLeaderboardEntry>> {
    let limit = param.limit;
    let offset = param.page * limit;

    let lb_repo = LeaderboardRepository::new(state.database_pool);
    let segment = lb_repo.get_chatter_leaderboard(limit, offset).await?;

    Ok(Json(segment))
}

#[instrument(skip(state))]
pub async fn chatter_by_login(
    State(state): State<Arc<AppState>>,
    Path(login): Path<String>,
    Query(param): Query<Pagination>,
) -> JsonResult<ChatterLeaderboardEntry> {
    let (ch_repo, lb_repo) = (
        ChatterRepository::new(state.database_pool),
        LeaderboardRepository::new(state.database_pool),
    );

    let chatter = ch_repo.get_by_login(&login).await?;
    match lb_repo.get_single_chatter_leaderboard(chatter.id).await? {
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
    match LeaderboardRepository::new(state.database_pool)
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

#[instrument]
#[debug_handler]
pub async fn helix_user_by_id(Path(id): Path<String>) -> JsonResult<Vec<HelixUser>> {
    let mut ids = vec![id];
    let helix_user = Helix::fetch_users_by_id(&mut ids).await?;

    Ok(Json(helix_user))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aliases {
    pub current: String,
    pub historic: Vec<String>,
}

pub async fn force_update_channel(
    State(state): State<Arc<AppState>>,
) -> Result<Json<String>, StatusCode> {
    let handle = tokio::spawn(async move {
        let mut channel_ids = sqlx::query_scalar!(
            r#"
            SELECT id FROM channel;
            "#
        )
        .fetch_all(state.database_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|id| ChatterId(id))
        .collect::<Vec<_>>();

        match util::channel::update_stored_channels(&mut channel_ids, true).await {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!(error = ?e, "failed to perform channel refresh");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    });

    match handle.await {
        Ok(Ok(_)) => Ok(Json(String::from("OK"))),
        Ok(Err(e)) => {
            tracing::error!("error in task: {e}");
            Ok(Json(e.to_string()))
        }
        Err(e) => {
            tracing::error!("task panic: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[instrument(skip(payload))]
pub async fn update_chatter_in_cache(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> Result<Json<String>, StatusCode> {
    let json_body: Aliases =
        serde_json::from_value::<Aliases>(payload).map_err(|_| StatusCode::BAD_REQUEST)?;

    let handle = tokio::spawn(async move {
        let current_name = json_body.current;
        let aliases = json_body.historic;

        match process_alias_migration(
            state.redis_pool.clone(),
            state.database_pool,
            &current_name,
            &aliases,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });

    match handle.await {
        Ok(Ok(_)) => Ok(Json(String::from("OK"))),
        Ok(Err(e)) => Ok(Json(e.to_string())),
        Err(e) => {
            tracing::error!("task panic: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[instrument(skip(state))]
pub async fn clear_chatter_scores(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<String>, StatusCode> {
    let database_handler = migrator::io::PgHandler(state.database_pool);
    tracing::warn!(id, "CLEARING CHATTER SCORE");

    let handle = tokio::spawn(async move {
        match database_handler
            .clear_scores_for_chatter(&ChatterId(id))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });

    match handle.await {
        Ok(Ok(_)) => Ok(Json(String::from("OK"))),
        Ok(Err(e)) => {
            tracing::error!("error in task: {e}");
            Ok(Json(e.to_string()))
        }
        Err(e) => {
            tracing::error!("task panic: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// #[instrument(skip(state))]
// pub async fn print_totp(State(state): State<Arc<AppState>>) -> JsonResult<&'static str> {
//     let mut guard = state.totp_handler.lock().await;
//     guard.write_code_out();
//
//     Ok(Json("OK"))
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct TOTPResponse {
    pub valid: bool,
    pub session: String,
}

impl TOTPResponse {
    pub fn new(valid: bool, session: String) -> Json<Self> {
        Json(Self { valid, session })
    }
}

#[instrument(skip(state))]
pub async fn totp_compare(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> Result<Json<TOTPResponse>, StatusCode> {
    #[derive(Debug, Serialize, Deserialize)]
    struct TOTPPayload {
        token: String,
    }

    let json_body: TOTPPayload =
        serde_json::from_value::<TOTPPayload>(payload).map_err(|_| StatusCode::BAD_REQUEST)?;

    tracing::debug!(?json_body, "RECEIVED CODE");
    let mut guard = state.totp_handler.lock().await;
    let valid = guard.totp_cmp(&json_body.token).map_err(|e| {
        tracing::error!(error = ?e, "unknown error during TOTP validation");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if valid {
        // create new session token and store in db
        let session = SessionToken::new_token();
        SessionToken::store_token(state.database_pool, &session)
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "failed to store new session token");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        Ok(TOTPResponse::new(valid, session))
    } else {
        Ok(TOTPResponse::new(valid, String::default()))
    }
}

// #[instrument(skip(payload))]
// pub async fn update_channel_in_cache(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<Value>,
// ) -> Result<Json<String>, StatusCode> {
//     let redis_connection = state.redis_pool.clone();
//     let database_pool = state.database_pool;
//
//     let json_body: Aliases =
//         serde_json::from_value::<Aliases>(payload).map_err(|_| StatusCode::BAD_REQUEST)?;
//
//     let handle = tokio::spawn(async move {
//         let current_name = json_body.current;
//         let historic_names = json_body.historic;
//
//         match update_from_cached_alias(
//             redis_connection,
//             database_pool,
//             &current_name,
//             &historic_names,
//         )
//         .await
//         {
//             Ok(_) => (),
//             Err(e) => match e {
//                 RedisErr::UpdateEmpty => {
//                     tracing::warn!("nothing to update for broadcaster user");
//                 }
//                 _ => {
//                     let msg = e.to_string();
//                     return Err(msg);
//                 }
//             },
//         };
//
//         match update_historic_channel(json_body).await {
//             Ok(_) => Ok(()),
//             Err(e) => Err(e.to_string()),
//         }
//     });
//
//     match handle.await {
//         Ok(Ok(_)) => Ok(Json(String::from("OK"))),
//         Ok(Err(e)) => Ok(Json(e.to_string())),
//         Err(e) => {
//             tracing::error!("task panic: {e}");
//             Err(StatusCode::INTERNAL_SERVER_ERROR)
//         }
//     }
// }

#[instrument(skip(state))]
pub async fn run_cache_migration(
    State(state): State<Arc<AppState>>,
    // _: HeaderMap,
) -> Result<Json<String>, StatusCode> {
    let handle = tokio::spawn(async move {
        match migrator::process_initial_migration(state.redis_pool.clone(), state.database_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });

    match handle.await {
        Ok(Ok(_)) => Ok(Json(String::from("OK"))),
        Ok(Err(e)) => Ok(Json(e.to_string())),
        Err(e) => {
            tracing::error!("task panic: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchByLoginParam {
    login: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchByIdParam {
    id: String,
}

#[instrument(skip(state))]
#[axum::debug_handler]
pub async fn search_by_login(
    State(state): State<Arc<AppState>>,
    Query(param): Query<SearchByLoginParam>,
) -> JsonResult<(Vec<ChatterSearchResult>, usize)> {
    let repo = ChatterRepository::new(state.database_pool);

    let search_res = repo.search_by_login(&param.login).await?;
    let length = search_res.len();

    Ok(Json((search_res, length)))
}

#[instrument(skip(state))]
pub async fn force_irc_reconnect(
    State(state): State<Arc<AppState>>,
) -> Result<Json<String>, StatusCode> {
    let supervisor = &state.irc_connection;
    match supervisor.connection.reset_tx.send(()).await {
        Ok(_) => {
            tracing::info!("force irc reset from API triggered");
            Ok(Json("IRC_RESET_REQUESTED".to_string()))
        }
        Err(e) => {
            tracing::error!(error = ?e, "failure while triggering irc reconnect from API");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowedScores {
    yesterday: i64,
    prev_week: i64,
    prev_month: i64,
    prev_year: i64,
    last_7_days: i64,
    last_30_days: i64,
}

impl WindowedScores {
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
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScoreWindowVariant {
    variant: String,
}

#[instrument(skip(state))]
pub async fn get_channel_scores_window(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(window): Query<ScoreWindowVariant>,
) -> Result<Json<WindowedScores>, StatusCode> {
    let pool = state.database_pool;
    if window.variant != "chatter" && window.variant != "channel" {
        tracing::warn!(
            window.variant,
            "query param for windowed score query not 'channel' || 'chatter'"
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    match tokio::spawn(async move {
        Tx::with_tx(pool, |mut tx| async move {
            let result = async {
                let mut query_results = Vec::new();
                let queries: [String; 6] = [
                    TimeWindow::Yesterday.into_query(&window.variant),
                    TimeWindow::PrevWeek.into_query(&window.variant),
                    TimeWindow::PrevMonth.into_query(&window.variant),
                    TimeWindow::PrevYear.into_query(&window.variant),
                    TimeWindow::Last7Days.into_query(&window.variant),
                    TimeWindow::Last30Days.into_query(&window.variant),
                ];
                for query in queries {
                    let q_res = sqlx::query_scalar::<_, i64>(&query)
                        .bind(&id)
                        .fetch_one(&mut **tx.inner_mut()?)
                        .await
                        .unwrap_or(0);

                    query_results.push(q_res);
                }

                Ok(query_results)
            }
            .await;

            (tx, result)
        })
        .await
    })
    .await
    {
        Ok(Ok(vals)) => Ok(Json(WindowedScores::new(
            vals[0], vals[1], vals[2], vals[3], vals[4], vals[5],
        ))),
        Ok(Err(e)) => {
            tracing::error!(error = ?e, "sqlx inner error");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(e) => {
            tracing::error!(error = ?e, "tokio worker error");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[instrument(skip(state))]
#[axum::debug_handler]
pub async fn channel_configs(
    State(state): State<Arc<AppState>>,
    Query(param): Query<SearchByIdParam>,
) -> JsonResult<Vec<ChannelReplies>> {
    let channel_repo = ChannelRepository::new(state.database_pool);

    if param.id == "all" {
        let all_configs = channel_repo.get_all_reply_configs().await?;
        Ok(Json(all_configs))
    } else {
        let config = channel_repo.get_reply_config(&param.id).await?;
        Ok(Json(vec![config]))
    }
}

#[instrument(skip(state))]
pub async fn update_channel_config(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SearchByIdParam>,
) -> Result<Response<axum::body::Body>, RouteError> {
    let channel_repo = ChannelRepository::new(state.database_pool);
    let id = payload.id;

    channel_repo.update_channel_config(&ChannelId(id)).await?;
    Ok("OK".into_response())
}

#[instrument(skip(state))]
pub async fn force_delete_hooks(
    State(state): State<Arc<AppState>>,
) -> Result<Response<axum::body::Body>, RouteError> {
    let handle = tokio::spawn(async move {
        let ids = state.channel_ids.clone();
        match crate::db::redis::init_stream_states(&mut state.redis_pool.clone(), &ids).await {
            Ok(_) => tracing::info!("removed all channel states from redis cache"),
            Err(e) => tracing::error!(error = ?e, "failed to remove channel states from redis"),
        }

        let active_hooks = Helix::get_active_subscriptions().await?;
        tracing::debug!(?active_hooks, "active_hooks");

        if !active_hooks.is_empty() {
            tracing::debug!("active_hooks populated - deleting...");
            match Helix::delete_subscriptions(&active_hooks).await {
                Ok(_) => {
                    let hooks_count = active_hooks.len();
                    Ok(format!("deleted {hooks_count} hooks").into_response())
                }
                Err(e) => Err(e),
            }
        } else {
            Ok("no hooks to delete".into_response())
        }
    });

    match handle.await {
        Ok(Ok(res)) => Ok(res),
        Ok(Err(e)) => {
            tracing::error!("error in task: {e:?}");
            Err(RouteError::HelixError(e))
        }
        Err(e) => {
            tracing::error!("task panic: {e}");
            Err(RouteError::JoinError(e))
        }
    }
}

#[instrument(skip(state))]
pub async fn force_reset_hooks(
    State(state): State<Arc<AppState>>,
) -> Result<Response<axum::body::Body>, RouteError> {
    let handle = tokio::spawn(async move {
        let ids = state.channel_ids.clone();
        match stream_online_hook_handler(&ids, state.redis_pool.clone()).await {
            Ok(_) => {
                tracing::info!("forced reset ok");
                Ok("OK".into_response())
            }
            Err(e) => {
                tracing::error!(error = ?e, "forced reset fail");
                Err(e)
            }
        }
    });

    handle.await?
}

// #[instrument(skip(state))]
// pub async fn add_new_channel(
//     State(state): State<Arc<AppState>>,
//     Query(param): Query<SearchByLoginParam>,
// ) -> JsonResult<String> {
//     let chatter_repo = ChatterRepository::new(state.database_pool);
//     let channel_repo = ChannelRepository::new(state.database_pool);
//
//     match tokio::spawn(async move {
//         let helix_chatter =
//     });
// }
