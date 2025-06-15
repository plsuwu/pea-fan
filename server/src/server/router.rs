use super::activity;
use super::midware::verify;
use crate::constants::{CHANNELS, SERVER_PORT, TrackedChannels};
use crate::db::redis::redis_pool;
use crate::server::midware::cors;
use crate::server::webhook::notification::webhook_handler;
use crate::server::{GetChannelQueryParams, GetUserQueryParams, RedisQueryResponse, get_debug};
use axum::extract::Query;
use axum::routing::{get, post};
use axum::{Json, Router, middleware};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::sync::oneshot;

/// Server listener
pub async fn route(tx: oneshot::Sender<(SocketAddr, Option<String>)>) {
    let app = Router::new()
        .route("/webhook-global", post(webhook_handler))
        .route_layer(middleware::from_fn(verify::verify_sender_ident))
        .route(
            "/",
            get(|| async { "root endpoint has no content, leave me be or i will scream" }),
        )
        .route("/active-sockets", get(activity))
        .route("/channels", get(get_tracked_channels))
        .route("/ceilings/channel", get(get_channel))
        .route("/ceilings/user", get(get_user))
        .route("/checkhealth", get(|| async { "SERVER_OK" }))
        .layer(cors::cors_layer());

    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), SERVER_PORT);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();

    _ = tx.send((bind_addr, get_debug()));
    axum::serve(listener, app).await.unwrap();
}

pub async fn get_tracked_channels() -> Json<TrackedChannels> {
    Json(CHANNELS)
}

pub async fn get_channel(Query(query): Query<GetChannelQueryParams>) -> Json<RedisQueryResponse> {
    if !CHANNELS.contains(&query.name.as_str()) {
        Json(RedisQueryResponse {
            err: true,
            err_msg: "NOT_TRACKED".to_string(),
            total: "0".to_string(),
            leaderboard: Vec::new(),
        })
    } else {
        let redis = redis_pool().await.unwrap();
        let res = redis.get_channel_data(&query.name).await;
        match res {
            Ok(r) => Json(r),
            Err(e) => {
                println!("[x] got error from redis: {:?}", e);

                // needs proper handling (e.g if a tracked chanel has no data)
                // but asdljk;ffasjdkl;jlfk;dsjl;kf for now
                Json(RedisQueryResponse {
                    err: true,
                    err_msg: format!("REDIS_ERROR({})", e),
                    total: "0".to_string(),
                    leaderboard: Vec::new(),
                })
            }
        }
    }
}

pub async fn get_user(Query(query): Query<GetUserQueryParams>) -> Json<RedisQueryResponse> {
    let redis = redis_pool().await.unwrap();
    match redis.get_user_data(&query.name).await {
        Err(_) => Json(RedisQueryResponse {
            err: true,
            err_msg: "NOT_TRACKED".to_string(),
            total: "0".to_string(),
            leaderboard: Vec::new(),
        }),
        Ok(val) => Json(val),
    }
}
