use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use axum::body::Body;
use axum::debug_handler;
use axum::extract::{MatchedPath, Path, Query, Request, State};
use axum::middleware::{self, Next, from_fn};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::future::Lazy;
use http::{HeaderMap, StatusCode};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use tokio_util::task::LocalPoolHandle;
use tower_http::trace::TraceLayer;
use tracing::{Span, instrument};

use crate::api::middleware as internal_mw;
use crate::api::middleware::verify_external::VerifiedBody;
use crate::db::models::{self, Channel, Chatter, DbUser};
use crate::db::pg::{DisplayableChannel, DisplayableChatter, Pagination, PgErr, db_pool};
use crate::db::redis::redis_pool::redis_pool;
use crate::util::env::Var;
use crate::util::helix::{Helix, HelixErr, HelixUser};
use crate::var;

static LOCAL_POOL: LazyLock<LocalPoolHandle> = LazyLock::new(|| LocalPoolHandle::new(1));

pub type JsonResult<T> = core::result::Result<Json<T>, RouteError>;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: &'static PgPool,
    pub redis_pool: ConnectionManager,
}

async fn webhook_int_handler(headers: HeaderMap) -> Result<&'static str, StatusCode> {
    Ok("not implemented")
}

async fn webhook_ext_handler(
    headers: HeaderMap,
    body: VerifiedBody,
) -> Result<&'static str, StatusCode> {
    Ok("not implemented")
}

async fn irc_get_joined(
) -> JsonResult<Vec<String>> {
    Ok(Json(Vec::new()))
}

#[instrument(skip(tx))]
pub async fn router(tx: tokio::sync::mpsc::UnboundedSender<SocketAddr>) {
    // let cors = internal_mw::cors().await.unwrap();
    let state = Arc::new(AppState {
        db_pool: db_pool().await.unwrap(),
        redis_pool: redis_pool().await.unwrap().manager.clone(),
    });

    let app = Router::new()
        .route("/webhook/external", post(webhook_ext_handler))
        .route_layer(middleware::from_fn(
            internal_mw::verify_external::verify_sender_ident,
        ))
        .route("/webhook/internal", get(webhook_int_handler))
        // .route_layer(middleware::from_fn(internal_mw::verify_internal::verify_sender_ident))
        .route("/", get(|| async { Json("[]") }))
        .route("/channels", get(tracked_channels))
        .route("/channel/by-id/{id}", get(channel_by_id))
        .route("/chatters", get(tracked_chatters))
        .route("/chatter/by-login/{login}", get(chatter_by_login))
        .route("/chatter/by-id/{id}", get(chatter_by_id))
        .route("/helix/by-login/{login}", get(helix_user_by_login))
        .route("/irc/joined", get(irc_get_joined))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &axum::http::Request<_>| {
                    let method = req.method();
                    let uri = req.uri();

                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|matched| matched.as_str());

                    tracing::debug_span!("api_request", ?method, ?uri, ?matched_path)
                })
                .on_failure(()),
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

    _ = tx.send(socket_addr).unwrap();
    axum::serve(listener, app).await.unwrap()
}

#[derive(Debug, Error)]
pub enum RouteError {
    #[error(transparent)]
    QueryError(#[from] PgErr),

    #[error("{0}")]
    AuthError(StatusCode),

    #[error(transparent)]
    HelixError(#[from] HelixErr),
}

impl IntoResponse for RouteError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status, message, err) = match &self {
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

            RouteError::HelixError(helix_err) => {
                match helix_err {
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

#[instrument(skip(request, next), fields(uri = request.uri().to_string()))]
async fn log_route_errors(request: Request, next: Next) -> Response {
    let res = next.run(request).await;
    if let Some(err) = res.extensions().get::<Arc<RouteError>>() {
        tracing::error!(error = ?err, "error occurred inside route handler");
    }

    res
}

#[instrument(fields(max = query.max, offset = query.offset))]
pub async fn tracked_channels(
    Query(query): Query<Pagination>,
) -> JsonResult<Vec<DisplayableChannel>> {
    let max = query.max;
    let offset = query.offset;
    let conn = db_pool().await.unwrap();
    let channels = Channel::get_range(conn, max, offset).await?;

    Ok(Json(channels))
}

pub async fn tracked_chatters(
    Query(query): Query<Pagination>,
) -> Json<(Vec<DisplayableChatter>, i64)> {
    let max = query.max;
    let offset = query.offset;

    let conn = db_pool().await.unwrap();
    let chatters = Chatter::get_range(conn, max, offset).await.unwrap();

    Json(chatters)
}

pub async fn channel_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> JsonResult<DisplayableChannel> {
    let channel = models::Channel::query_from_id(state.db_pool, id.clone())
        .await
        .map_err(|e| RouteError::QueryError(e))?;

    Ok(Json(DisplayableChannel {
        id,
        name: channel.name,
        login: channel.login,
        color: channel.color,
        image: channel.image,
        total_as_chatter: channel.total_as_chatter,
        total_as_broadcaster: channel.total_as_broadcaster,
        chatters: channel.chatters,
    }))
}

pub async fn chatter_by_login(
    State(state): State<Arc<AppState>>,
    Path(login): Path<String>,
) -> JsonResult<DisplayableChatter> {
    let user = models::Chatter::get_by_login(state.db_pool, &login)
        .await
        .unwrap();

    Ok(Json(DisplayableChatter {
        id: user.id,
        name: user.name,
        login: user.login,
        color: user.color,
        image: user.image,
        total: user.total.to_string(),
        channels: Some(sqlx::types::Json(Vec::new())),
    }))
}

pub async fn chatter_by_id(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
) -> JsonResult<DisplayableChatter> {
    let user = models::Chatter::query_by_id(state.db_pool, &user_id.to_string())
        .await
        .unwrap();

    Ok(Json(DisplayableChatter {
        id: user.id,
        name: user.name,
        login: user.login,
        color: user.color,
        image: user.image,
        total: user.total.to_string(),
        channels: user.channels,
    }))
}

#[debug_handler]
pub async fn helix_user_by_login(Path(login): Path<String>) -> JsonResult<Vec<HelixUser>> {
    let logins = vec![login];
    let helix_user = Helix::fetch_users_by_login(logins).await?;

    Ok(Json(helix_user))
}

// pub async fn chatter_list(Query(query): Query<Pagination>) -> Json<i64> {
//     let max = query.max;
//     let offset = query.offset;
//
//     let conn = db_pool().await.unwrap();
//     let (chatters, count) = Chatter::get_range(conn, max, offset).await.unwrap();
//
//     info!("{:#?}", chatters);
//
//     Json(count)
// }

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::tracing as otlp_trace;

    #[tokio::test]
    async fn test_run_server() {
        let provider = otlp_trace::build_subscriber().await.unwrap();

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SocketAddr>();

        tracing::info!("starting server");
        let server_handle = tokio::task::spawn(async move {
            router(tx).await;
        });

        while !rx.is_closed() {
            if let Some(msg) = rx.recv().await {
                tracing::info!(
                    server_url = "127.0.0.1",
                    server_port = msg.port(),
                    "server ready"
                );
                break;
            }
        }

        _ = server_handle.await;
        otlp_trace::destroy_tracer(provider);
    }
}
