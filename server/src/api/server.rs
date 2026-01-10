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
use futures::future::{Lazy, join_all};
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
// use crate::db::{PgError, db_pool};
// use crate::db::redis::redis_pool::redis_pool;
// use crate::db::repositories::Repository;
use crate::db::prelude::*;
use crate::db::redis::redis_pool::redis_pool;
use crate::util::env::Var;
use crate::util::helix::{Helix, HelixErr, HelixUser};
use crate::var;

pub type JsonResult<T> = core::result::Result<Json<T>, RouteError>;

#[derive(Clone, Debug)]
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

async fn irc_get_joined() -> JsonResult<Vec<String>> {
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
        .route("/", get(|| async { "OK".into_response() }))
        .route("/helix/by-login/{login}", get(helix_user_by_login))
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
    QueryError(#[from] PgError),

    #[error("{0}")]
    AuthError(StatusCode),

    #[error(transparent)]
    HelixError(#[from] HelixErr),

    #[error(transparent)]
    SqlxError(#[from] sqlx::error::Error),
}

impl IntoResponse for RouteError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status, message, err) = match &self {
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
/// Custom error trace handler for `RouteError`-type responses (which all routes should resolve
/// their errors to)
async fn log_route_errors(request: Request, next: Next) -> Response {
    let res = next.run(request).await;
    if let Some(err) = res.extensions().get::<Arc<RouteError>>() {
        tracing::error!(error = ?err, "error occurred inside route handler");
    }

    res
}

// #[instrument(fields(max = query.limit, offset = query.offset))]
// pub async fn tracked_channels(
//     State(state): State<Arc<AppState>>,
//     Query(query): Query<Pagination>,
// ) -> JsonResult<Vec<ChannelResponse>> {
//     let limit = query.limit;
//     let offset = query.offset;
//
//     // let channels = ChannelRepository::new(state.db_pool).get_
//     let channels = Vec::new();
//     Ok(Json(channels))
// }

#[instrument]
#[debug_handler]
pub async fn helix_user_by_login(Path(login): Path<String>) -> JsonResult<Vec<HelixUser>> {
    let logins = vec![login];
    let helix_user = Helix::fetch_users_by_login(logins).await?;

    Ok(Json(helix_user))
}

#[instrument]
pub async fn main_server_handler() -> Result<(), RouteError> {

    // TODO:
    //  perhaps these should be passed into the function as args!
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SocketAddr>();

    tracing::info!("starting server");
    let server_handle = tokio::task::spawn(async move {
        router(tx).await;
    });

    let logging_handle = tokio::task::spawn(async move {
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
    });

    _ = join_all([server_handle, logging_handle]).await;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::tracing as otlp_trace;

    #[tokio::test]
    async fn test_run_server() {
        let provider = otlp_trace::build_subscriber().await.unwrap();

        // let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SocketAddr>();

        // tracing::info!("starting server");
        // let server_handle = tokio::task::spawn(async move {
        //     router(tx).await;
        // });
        //
        // while !rx.is_closed() {
        //     if let Some(msg) = rx.recv().await {
        //         tracing::info!(
        //             server_url = "127.0.0.1",
        //             server_port = msg.port(),
        //             "server ready"
        //         );
        //         break;
        //     }
        // }
        
        let server_handle = main_server_handler().await;
        server_handle.unwrap();

        // _ = server_handle.await;
        otlp_trace::destroy_tracer(provider);
    }
}
