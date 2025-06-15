pub mod midware;
pub mod router;
pub mod types;
pub mod webhook;

use crate::server::midware::verify::KEY_DIGEST;
use crate::server::webhook::dispatch::IRC_HANDLES;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RootSitemap {
    endpoints: Vec<String>,
}

// pub async fn root() -> Json<RootSitemap> {
// }

#[derive(Serialize, Deserialize)]
pub struct ActivitySummary {
    active_count: usize,
    active_broadcasters: Vec<String>,
}

// #[allow(unused_variables)]
pub async fn activity() -> Json<ActivitySummary> {
    let handles_guard = IRC_HANDLES.lock().unwrap();
    let (active, _) = handles_guard.get_connection_summary();
    let summary = ActivitySummary {
        active_count: active.len(),
        active_broadcasters: active,
    };

    Json(summary)
}

#[derive(Serialize, Deserialize)]
pub struct GetChannelQueryParams {
    name: String,
}

#[derive(Deserialize)]
pub struct GetUserQueryParams {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedisQueryResponse {
    pub err: bool,
    pub err_msg: String,
    pub total: String,
    pub leaderboard: Vec<(String, isize)>,
}

/// Log server port and secret key string to stdout
///
/// Intended for debugging; this would be automatically sent on subscription to a topic in
/// production.
fn get_debug() -> Option<String> {
    let digest_lock = &*KEY_DIGEST;
    if let Ok(digest) = digest_lock.read() {
        Some(digest._hex.clone())
    } else {
        None
    }
}
