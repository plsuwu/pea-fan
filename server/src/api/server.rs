use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use axum::extract::{MatchedPath, Request};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use axum_prometheus::PrometheusMetricLayer;
use http::StatusCode;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use serde::Serialize;
use sqlx::{PgPool, Pool, Postgres};
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{self, Sender};
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::task::{JoinError, JoinHandle};
use tower_http::trace::TraceLayer;
use tracing::instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::api::middleware::cors_layer;
use crate::api::middleware::verify_external::{get_hmac_key, verify_external_ident};
use crate::api::middleware::verify_internal::verify_session_ident;
use crate::api::webhook::webhook_handler;
use crate::api::{handlers::*, webhook};
use crate::db::prelude::*;
use crate::db::redis::redis_pool::RedisErr;
use crate::irc::{ConnectionClientError, IrcHandle};
use crate::util::channel::ChannelError;
use crate::util::env::Var;
use crate::util::helix::HelixErr;
use crate::util::totp::TOTPHandler;
use crate::{util, var};

pub type ApiResult<T> = Result<Json<ApiResponse<T>>, RouteError>;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub status: u16,
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Json<Self> {
        Json(Self {
            status: 200,
            data: Some(data),
        })
    }

    pub fn empty() -> Json<ApiResponse<()>> {
        Json(ApiResponse {
            status: 204,
            data: None,
        })
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub database_pool: &'static PgPool,
    pub redis_pool: ConnectionManager,
    pub irc_connection: IrcHandle,
    pub channels: Arc<RwLock<Vec<String>>>,
    pub channel_ids: Arc<RwLock<Vec<String>>>,
    pub totp_handler: Arc<Mutex<TOTPHandler>>,
}

#[instrument(skip_all, fields(num_ids = channel_ids.len()))]
pub async fn stream_online_hook_handler<R: AsyncCommands + Sync>(
    channel_ids: &[String],
    mut redis_pool: R,
) -> Result<(), RouteError> {
    match webhook::dispatch::reset_hooks(&channel_ids).await {
        Ok(_) => tracing::debug!("webhook subs reset"),
        Err(e) => tracing::error!(error = ?e, "reset webhook subs failure"),
    }

    match crate::db::redis::init_stream_states(&mut redis_pool, &channel_ids).await {
        Ok(_) => tracing::debug!("initial cache entries created"),
        Err(e) => tracing::error!(error = ?e, "initial cache entry create failure"),
    }

    Ok(())
}

#[instrument(skip(database_pool))]
pub async fn initialize_channels(
    database_pool: &'static Pool<Postgres>,
) -> Result<(Vec<String>, Vec<String>), RouteError> {
    let channel_ids = ChannelRepository::new(database_pool)
        .get_all_channel_ids()
        .await
        .unwrap();

    let as_chatter_ids = channel_ids
        .iter()
        .map(|ch| ChatterId::from(ch.to_owned()))
        .collect::<Vec<ChatterId>>();

    let channel_logins: Vec<String> = util::channel::update_stored_channels(&as_chatter_ids, true)
        .await
        .unwrap()
        .into_keys()
        .collect();

    tracing::info!(?channel_logins, "using this channel list");
    Ok((channel_ids, channel_logins))
}

fn public_channel_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/all", get(channel::channel_name_list))
        .route("/leaderboard", get(channel::channel_leaderboard))
        .route("/live", get(channel::live_channels))
        .route("/irc-joins", get(channel::irc_joins))
        .route("/bot-state", get(channel::bot_enabled))
        .route("/by-id/{id}", get(channel::by_id))
        .route("/by-login/{login}", get(channel::by_login))
        .route("/windowed/{id}", get(channel::channel_score_windows))
}

fn public_chatter_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/leaderboard", get(chatter::chatter_leaderboard))
        .route("/by-login/{login}", get(chatter::by_login))
        .route("/by-id/{id}", get(chatter::by_id))
}

fn restricted_routes() -> Router<Arc<AppState>> {
    let update_routes = Router::new()
        .route(
            "/channels",
            post(admin::channel::new_channel).put(admin::channel::update_channel_data),
        )
        .route("/live", put(admin::channel::refresh_channel_state))
        .route(
            "/bot-config",
            get(admin::channel::get_reply_config).put(admin::channel::update_channel_config),
        );

    let helix_routes = Router::new()
        .route("/by-login/{login}", get(admin::helix::user_by_login))
        .route("/by-id/{id}", get(admin::helix::user_by_id))
        .route(
            "/hooks",
            get(admin::helix::active_hooks)
                .put(admin::helix::reset_hooks)
                .delete(admin::helix::delete_hooks),
        );

    let irc_routes = Router::new().route("/reset", put(admin::reset_irc));

    Router::new()
        .route("/session", get(admin::validate_session))
        .nest("/update", update_routes)
        .nest("/helix", helix_routes)
        .nest("/irc", irc_routes)
}

#[instrument]
async fn check_health() -> ApiResult<&'static str> {
    Ok(ApiResponse::ok("healthy"))
}

pub async fn router(
    tx: tokio::sync::mpsc::UnboundedSender<SocketAddr>,
    database_pool: &'static Pool<Postgres>,
    redis_pool: ConnectionManager,
    totp_handler: Arc<Mutex<TOTPHandler>>,
) {
    let secret_key = get_hmac_key().await.unwrap();
    tracing::info!(secret_key, "HMAC SECRET KEY");

    let (channel_ids, channel_logins) = initialize_channels(database_pool).await.unwrap();
    let irc_connection = crate::irc::start(channel_logins.clone(), database_pool, 10)
        .await
        .unwrap();

    let state = Arc::new(AppState {
        database_pool,
        irc_connection,
        redis_pool: redis_pool.clone(),
        channels: Arc::new(RwLock::new(channel_logins)),
        channel_ids: Arc::new(RwLock::new(
            channel_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>(),
        )),
        totp_handler,
    });

    let server_state_clone = Arc::clone(&state);

    let external_post_routes = Router::new()
        .route("/callback", post(webhook_handler))
        .route_layer(middleware::from_fn(verify_external_ident));

    let init_auth_routes = Router::new().route("/new-session", post(admin::new_session));

    let admin_routes = restricted_routes().route_layer(middleware::from_fn_with_state(
        state.clone(),
        verify_session_ident,
    ));

    let main_api_routes = Router::new()
        .route("/checkhealth", get(check_health))
        .route("/search/{user}", get(chatter::search))
        .layer(cors_layer().await);

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let routes = Router::new()
        .merge(main_api_routes)
        .nest("/chatter", public_chatter_routes())
        .nest("/channel", public_channel_routes())
        .nest("/auth", init_auth_routes)
        .nest("/_extern", external_post_routes)
        .nest("/_admin", admin_routes);

    let app = Router::new()
        .nest("/api/v1", routes)
        .route("/metrics", get(|| async move { metric_handle.render() }))
        // setting on outermost-ish layer provides prometheus metrics on all routes
        .layer(prometheus_layer)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &axum::http::Request<_>| {
                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|matched| matched.as_str());

                    let headers = req.headers();
                    let cfconnecting = headers
                        .get("cf-connecting-ip")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("MISSING");

                    let country = headers
                        .get("cf-ipcountry")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("MISSING");

                    tracing::info_span!(
                        "http_request",
                        method = %req.method(),
                        path = ?matched_path,
                        cfconnecting,
                        country,
                        user_agent = tracing::field::Empty,
                    )
                })
                .on_response(
                    |response: &Response, latency: std::time::Duration, _span: &tracing::Span| {
                        let status = response.status();
                        if let Some(err) = response.extensions().get::<Arc<RouteError>>() {
                            tracing::error!(
                                %status,
                                latency_ms = latency.as_millis(),
                                error = ?err,
                                "processing_failure",
                            );
                            
                        } else if status.is_server_error() || status.is_client_error() {
                            tracing::warn!(%status, latency_ms = latency.as_millis(), "request_result_non2xx");
                        } else {
                            tracing::info!(%status, latency_ms = latency.as_millis(), "request_result_2xx");
                        }
                    },
                ).on_request(|req: &Request, span: &tracing::Span| {
                    let user_agent = req.headers()
                        .get(http::header::USER_AGENT)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("MISSING");

                    span.record("user-agent", user_agent);
                    tracing::debug!(request_uri = ?req.uri(), "receive_request");
                }),
        )
        .with_state(state);

    let port = var!(Var::ServerApiPort)
        .await
        .unwrap()
        .parse::<u16>()
        .unwrap();

    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
    let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();

    // we want to trigger these every time we run as each new run uses a unique HMAC secret
    if !cfg!(debug_assertions) {
        tokio::spawn(async move {
            let _guard = server_state_clone.channel_ids.read().await;
            let channel_ids = _guard.clone();

            drop(_guard);
            match stream_online_hook_handler(&channel_ids, server_state_clone.redis_pool.clone())
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!(error = ?e, "error while initialising stream states");
                }
            }
        })
        .await
        .unwrap();
    }

    tx.send(socket_addr).unwrap();
    axum::serve(listener, app).await.unwrap()
}

#[instrument(skip_all, err)]
pub async fn start_server(
    tx: UnboundedSender<SocketAddr>,
    mut rx: UnboundedReceiver<SocketAddr>,
    database_pool: &'static Pool<Postgres>,
    redis_pool: ConnectionManager,
    totp_handler: Arc<Mutex<TOTPHandler>>,
) -> Result<Vec<JoinHandle<()>>, RouteError> {
    tracing::info!("starting server");

    let server_handle = tokio::task::spawn(async move {
        router(tx, database_pool, redis_pool, totp_handler).await;
    });

    let logging_handle = tokio::task::spawn(async move {
        while !rx.is_closed() {
            if let Some(msg) = rx.recv().await {
                tracing::info!(
                    server_url = &format!("http://{}:{}", msg.ip(), msg.port()),
                    "server ready"
                );

                break;
            }
        }
    });

    let handles = vec![server_handle, logging_handle];
    Ok(handles)
}

#[derive(Debug, Error)]
pub enum RouteError {
    #[error(transparent)]
    JoinError(#[from] JoinError),

    #[error(transparent)]
    IrcClientError(#[from] ConnectionClientError),

    #[error(transparent)]
    QueryError(#[from] PgError),

    #[error(transparent)]
    ChannelFetch(#[from] ChannelError),

    #[error("{0}")]
    GenericStatusCode(StatusCode),

    #[allow(dead_code)]
    #[error("TOTP validation failure: {0}")]
    ValidationError(String),

    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error(transparent)]
    HelixError(#[from] HelixErr),

    #[error(transparent)]
    SignalError(#[from] mpsc::error::SendError<()>),

    #[error(transparent)]
    RedisError(#[from] RedisErr),

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

impl RouteError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidUser(_) => StatusCode::NOT_FOUND,
            Self::GenericStatusCode(s) => *s,
            Self::HelixError(e) => e.status_code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn client_message(&self) -> String {
        match self {
            Self::InvalidUser(id) => format!("unknown user '{id}'"),
            Self::GenericStatusCode(_) => "unauthorized".into(),
            Self::HelixError(e) => e.client_message(),
            _ => "internal server error".into(),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    status: u16,
    error: String,
}

impl IntoResponse for RouteError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.client_message();

        let mut response = (
            status,
            Json(ErrorBody {
                status: status.as_u16(),
                error: message,
            }),
        )
            .into_response();

        response.extensions_mut().insert(Arc::new(self));
        response
    }
}
