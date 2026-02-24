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
use sqlx::PgPool;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{self, Sender};
use tokio::task::JoinHandle;
use tower_http::trace::TraceLayer;
use tracing::instrument;

use crate::api::handler::*;
use crate::api::middleware::verify_external::{get_hmac_key, verify_sender_ident};
use crate::api::middleware::verify_internal::verify_internal_ident;
use crate::api::webhook::webhook_handler;
use crate::db::prelude::*;
use crate::db::redis::redis_pool::redis_pool;
use crate::util::channel::ChannelError;
use crate::util::env::Var;
use crate::util::helix::HelixErr;
use crate::var;

pub type JsonResult<T> = core::result::Result<Json<T>, RouteError>;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct AppState {
    pub db_pool: &'static PgPool,
    pub redis_pool: ConnectionManager,
    pub tx_client: UnboundedSender<(String, Sender<Vec<String>>)>,
}

#[instrument(skip(tx))]
pub async fn router(
    tx: tokio::sync::mpsc::UnboundedSender<SocketAddr>,
    tx_to_client: UnboundedSender<(String, Sender<Vec<String>>)>,
) {
    // let cors = internal_mw::cors().await.unwrap();
    let state = Arc::new(AppState {
        db_pool: db_pool().await.unwrap(),
        redis_pool: redis_pool().await.unwrap().manager.clone(),
        tx_client: tx_to_client,
        // rx_client: Arc::new(rx_from_client),
    });

    let secret_key = get_hmac_key().await.unwrap();
    tracing::info!(secret_key, "HMAC SECRET KEY");

    //
    // twitch hook callback
    let external_post_routes = Router::new()
        .route("/callback", post(webhook_handler))
        .route_layer(middleware::from_fn(verify_sender_ident));

    let internal_post_routes = Router::new()
        .route("/update/channel", post(update_channel_in_cache))
        .route("/update/chatter", post(update_chatter_in_cache))
        .route("/update/migrate", get(run_cache_migration))
        .route_layer(middleware::from_fn(verify_internal_ident));

    let app = Router::new()
        .merge(external_post_routes)
        .merge(internal_post_routes)
        //
        // general
        .route("/", get(|| async { Response::new(Body::empty()) }))
        .route("/search/by-login", get(search_by_login))
        //
        // channel-related routes
        .route("/channel/leaderboard", get(global_channels))
        .route("/channel/by-login/{login}", get(channel_by_login))
        .route("/channel/by-id/{id}", get(channel_by_id))
        .route("/channel/irc-joins", get(irc_joins))
        //
        // chatter-related routes
        .route("/chatter/leaderboard", get(global_chatters))
        .route("/chatter/by-login/{login}", get(chatter_by_login))
        .route("/chatter/by-id/{id}", get(chatter_by_id))
        //
        // proxied helix requests
        .route("/helix/by-login/{login}", get(helix_user_by_login))
        .route("/helix/by-id/{id}", get(helix_user_by_id))
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
    tx_to_irc: UnboundedSender<(String, Sender<Vec<String>>)>,
    mut rx: UnboundedReceiver<SocketAddr>,
) -> Result<Vec<JoinHandle<()>>, RouteError> {
    tracing::info!("starting server");
    let server_handle = tokio::task::spawn(async move {
        router(tx, tx_to_irc).await;
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::telemetry as otlp_trace;
    use futures::future::join_all;
    // use futures_util::future::join_all;
    use tokio::sync::oneshot::Sender;

    #[tokio::test]
    async fn test_run_server() {
        let provider = otlp_trace::Telemetry::new().await.unwrap().register();

        let (tx_server, rx) = tokio::sync::mpsc::unbounded_channel::<SocketAddr>();
        let (tx_from_api, rx_from_api) =
            tokio::sync::mpsc::unbounded_channel::<(String, Sender<Vec<String>>)>();

        let channels = ["vacu0usly", "plss", "chikogaki"]
            .into_iter()
            .map(|ch| ch.to_string())
            .collect();

        let mut handles = start_server(tx_server, tx_from_api, rx).await.unwrap();
        handles.extend(
            crate::irc::client::start_irc_handler(channels, rx_from_api)
                .await
                .unwrap(),
        );

        _ = join_all(handles).await;
        provider.shutdown();
    }
}
