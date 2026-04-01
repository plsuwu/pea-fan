use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{MatchedPath, Request};
use axum::middleware::{self, Next, from_fn};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use http::StatusCode;
use redis::aio::ConnectionManager;
use serde::Serialize;
use sqlx::{PgPool, Pool, Postgres};
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{self, Sender};
use tokio::task::JoinHandle;
use tower_http::trace::TraceLayer;
use tracing::instrument;

use crate::api::middleware::cors_layer;
use crate::api::middleware::verify_external::{get_hmac_key, verify_external_ident};
use crate::api::middleware::verify_internal::verify_session_ident;
use crate::api::webhook::webhook_handler;
use crate::api::{handler::*, webhook};
use crate::db::prelude::*;
use crate::irc::{ConnectionClientError, IrcHandle};
use crate::util::channel::ChannelError;
use crate::util::env::Var;
use crate::util::helix::HelixErr;
use crate::util::totp::TOTPHandler;
use crate::{util, var};

pub type JsonResult<T> = core::result::Result<Json<T>, RouteError>;

#[derive(Clone, Debug)]
pub struct AppState {
    pub database_pool: &'static PgPool,
    pub redis_pool: ConnectionManager,
    pub irc_connection: IrcHandle,
    pub channels: Vec<String>,
    pub totp_handler: Arc<Mutex<TOTPHandler>>,
}

#[instrument(skip(tx, database_pool, redis_pool, totp_handler))]
pub async fn router(
    tx: tokio::sync::mpsc::UnboundedSender<SocketAddr>,
    database_pool: &'static Pool<Postgres>,
    redis_pool: ConnectionManager,
    totp_handler: Arc<Mutex<TOTPHandler>>,
) {
    let secret_key = get_hmac_key().await.unwrap();
    tracing::info!(secret_key, "HMAC SECRET KEY");

    let channels_updated = util::channel::update_channels(None)
        .await
        .unwrap_or_default();

    let channel_logins: Vec<String> = channels_updated.keys().cloned().collect();
    let channel_ids: Vec<String> = channels_updated.into_values().map(|ch| ch.id.0).collect();
    
    match webhook::dispatch::reset_hooks(&channel_ids).await {
        Ok(_) => tracing::debug!("webhook subs reset"),
        Err(e) => tracing::error!(error = ?e, "reset webhook subs failure"),
    }

    match crate::db::redis::init_stream_states(&mut redis_pool.clone(), &channel_ids).await {
        Ok(_) => tracing::debug!("initial cache entries created"),
        Err(e) => tracing::error!(error = ?e, "initial cache entry create failure"),
    }

    let irc_connection = crate::irc::start(channel_logins.clone(), database_pool, 10)
        .await
        .unwrap();

    let state = Arc::new(AppState {
        database_pool,
        irc_connection,
        redis_pool,
        channels: channel_logins,
        totp_handler,
    });

    // twitch hook callback
    let external_post_routes = Router::new()
        .route("/callback", post(webhook_handler))
        .route_layer(middleware::from_fn(verify_external_ident));

    let init_auth_routes = Router::new().route("/auth/totp-session", post(totp_compare));

    // runtime administration
    let internal_post_routes = Router::new()
        //
        // internal stuff
        //
        .route(
            "/auth/validate-session",
            get(|| async { "OK".into_response() }),
        )
        .route("/channel/reply-configs", get(channel_configs))
        .route("/update/reply-configs", post(update_channel_config))
        // .route("/update/channel/add", post(add_new_channel))
        // .route("/update/channel/remove", post(remove_channel))
        .route("/update/chatter", post(update_chatter_in_cache))
        .route("/update/channel", get(force_update_channel))
        .route(
            "/update/clear-scores/chatter/{id}",
            get(clear_chatter_scores),
        )
        .route("/update/migrate", get(run_cache_migration))
        .route("/update/db-entries", get(irc_joins))
        .route("/irc/force-reconnect", get(force_irc_reconnect))
        //
        // helix proxying
        //
        .route("/helix/by-login/{login}", get(helix_user_by_login))
        .route("/helix/by-id/{id}", get(helix_user_by_id))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            verify_session_ident,
        ));

    let main_api_routes = Router::new()
        //
        // general
        //
        .route("/", get(|| async { Response::new(Body::empty()) }))
        .route("/search/by-login", get(search_by_login))
        //
        // channel-related routes
        //
        .route("/channel/leaderboard", get(global_channels))
        .route("/channel/windowed/{id}", get(get_channel_scores_window))
        .route("/channel/by-login/{login}", get(channel_by_login))
        .route("/channel/by-id/{id}", get(channel_by_id))
        .route("/channel/irc-joins", get(irc_joins))
        .route("/channel/live", get(live_channels))
        .route("/channel/all", get(all_channels))
        .route("/channel/bot-enabled", get(bot_enabled_channels))
        //
        // chatter-related routes
        //
        .route("/chatter/leaderboard", get(global_chatters))
        .route("/chatter/by-login/{login}", get(chatter_by_login))
        .route("/chatter/by-id/{id}", get(chatter_by_id))
        .layer(cors_layer().await);

    let app = Router::new()
        .merge(external_post_routes)
        .merge(init_auth_routes)
        .merge(internal_post_routes)
        .merge(main_api_routes)
        .layer(
            TraceLayer::new_for_http().make_span_with(|req: &axum::http::Request<_>| {
                let method = req.method();
                let uri = req.uri();

                let matched_path = req
                    .extensions()
                    .get::<MatchedPath>()
                    .map(|matched| matched.as_str());

                tracing::debug_span!("api_request", ?method, ?uri, ?matched_path)
            }),
        )
        .layer(from_fn(log_route_errors))
        .with_state(state);

    let port = var!(Var::ServerApiPort)
        .await
        .unwrap()
        .parse::<u16>()
        .unwrap();

    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
    let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();

    tx.send(socket_addr).unwrap();
    axum::serve(listener, app).await.unwrap()
}

/// Custom error trace handler for `RouteError`-type responses
///
/// # Notes
///
/// Currently using this as a replacement for default axum route error handling, but perhaps this
/// is better if implemented in a complementary manner?
#[instrument(skip(request, next), fields(uri = request.uri().to_string()))]
async fn log_route_errors(request: Request, next: Next) -> Response {
    let res = next.run(request).await;
    if let Some(err) = res.extensions().get::<Arc<RouteError>>() {
        tracing::error!(error = ?err, "error occurred inside route handler");
    }

    res
}

#[instrument]
pub async fn start_server(
    tx: UnboundedSender<SocketAddr>,
    mut rx: UnboundedReceiver<SocketAddr>,
    database_pool: &'static Pool<Postgres>,
    redis_pool: ConnectionManager,
    totp_handler: Arc<Mutex<TOTPHandler>>,
) -> Result<Vec<JoinHandle<()>>, RouteError> {
    tracing::info!("starting server...");

    let server_handle = tokio::task::spawn(async move {
        router(tx, database_pool, redis_pool, totp_handler).await;
    });

    let logging_handle = tokio::task::spawn(async move {
        while !rx.is_closed() {
            if let Some(msg) = rx.recv().await {
                tracing::info!(
                    server_url = &format!("http://127.0.0.1:{}", msg.port()),
                    "server ready"
                );
                break;
            }
        }
    });

    let handles = vec![server_handle, logging_handle];
    Ok(handles)
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum RouteError {
    #[error(transparent)]
    IrcClientError(#[from] ConnectionClientError),

    #[error(transparent)]
    QueryError(#[from] PgError),

    #[error(transparent)]
    ChannelFetch(#[from] ChannelError),

    #[error("{0}")]
    AuthError(StatusCode),

    #[error(transparent)]
    HelixError(#[from] HelixErr),

    #[error(transparent)]
    SqlxError(#[from] sqlx::error::Error),

    #[error("invalid login or id '{0}'")]
    InvalidUser(String),

    #[error(transparent)]
    TryRecvError(#[from] oneshot::error::TryRecvError),

    #[error(transparent)]
    ChannelRecvError(#[from] oneshot::error::RecvError),

    #[error(transparent)]
    ChannelSendError(#[from] SendError<(String, Sender<Vec<String>>)>),
}

impl IntoResponse for RouteError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status, message, err) = match &self {
            RouteError::IrcClientError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                error.to_string(),
                Some(self),
            ),

            RouteError::TryRecvError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                error.to_string(),
                Some(self),
            ),

            RouteError::ChannelSendError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                error.to_string(),
                Some(self),
            ),

            RouteError::ChannelRecvError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                error.to_string(),
                Some(self),
            ),

            RouteError::InvalidUser(ident) => (
                StatusCode::BAD_REQUEST,
                format!("invalid login or id '{ident}'"),
                Some(self),
            ),

            RouteError::SqlxError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                err.to_string(),
                Some(self),
            ),

            RouteError::QueryError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                err.to_string(),
                Some(self),
            ),

            RouteError::AuthError(status) => (
                status.to_owned(),
                String::from("invalid authorization header"),
                Some(self),
            ),

            RouteError::ChannelFetch(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("error during channel fetch: {err}"),
                Some(self),
            ),

            RouteError::HelixError(helix_err) => {
                match helix_err {
                    HelixErr::MiddlewareError(error) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error.to_string(),
                        Some(self),
                    ),
                    HelixErr::SerdeError(error) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error.to_string(),
                        Some(self),
                    ),
                    HelixErr::ReqwestError(error) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error.to_string(),
                        Some(self),
                    ),
                    HelixErr::FetchErr(error) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error.to_string(),
                        Some(self),
                    ),
                    HelixErr::EnvError(error) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error.to_string(),
                        Some(self),
                    ),
                    HelixErr::HeaderError(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        String::from("helix reported a malformed request from our server"),
                        Some(self),
                    ),
                    HelixErr::InvalidUsername => (
                        StatusCode::BAD_REQUEST,
                        String::from("invalid username queried"),
                        None, // not necessarily an error for our server to care about
                    ),
                    HelixErr::EmptyDataField => (
                        StatusCode::BAD_REQUEST,
                        String::from("received empty data array from helix api (malformed login?)"),
                        // this also probably isnt our concern, but im still not 100%
                        // on why this occurs and its probably good to have information about
                        Some(self),
                    ),
                    HelixErr::FetchErrWithBody { body } => {
                        (StatusCode::BAD_REQUEST, body.to_string(), Some(self))
                    }
                }
            }
        };

        let mut response = (status, Json(ErrorResponse { message })).into_response();
        if let Some(err) = err {
            response.extensions_mut().insert(Arc::new(err));
        }

        response
    }
}
