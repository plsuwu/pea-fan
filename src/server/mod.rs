pub mod mid;
pub mod subscriber;
pub mod types;

use crate::server::mid::verify;
use crate::server::mid::verify::VerifiedBody;
use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware;
use axum::routing::{get, post};
use serde_json::Value;
use subscriber::KEY_DIGEST;

const TWITCH_MESSAGE_TYPE_HEADER: &'static str = "Twitch-Eventsub-Message-Type";
pub enum WebhookMessageType {
    Verify,
    Notify,
    Revoke,
}

impl WebhookMessageType {
    pub fn parse_to_str(&self) -> &'static str {
        match self {
            Self::Verify => "webhook_callback_verification",
            Self::Notify => "notification",
            Self::Revoke => "revocation",
        }
    }

    pub fn parse_from_str(rx: &'static str) -> Result<Self, &'static str> {
        match rx {
            "webhook_callback_verification" => Ok(Self::Verify),
            "notification" => Ok(Self::Notify),
            "revocation" => Ok(Self::Revoke),
            _ => Err("Invalid messasge type header"),
        }
    }
}

impl TryFrom<&'static str> for WebhookMessageType {
    type Error = &'static str;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        WebhookMessageType::parse_from_str(value)
    }
}

impl Into<&'static str> for WebhookMessageType {
    fn into(self) -> &'static str {
        self.parse_to_str()
    }
}

const TYPE_CALLBACK_VERIFICATION: &str = "webhook_callback_verification";
const SERVER_PORT: &'static str = "3000";

/// Server listener
pub async fn serve() {
    let app = Router::new()
        .route("/", get(root))
        .route("/webhook-global", post(webhook_handler))
        .route_layer(middleware::from_fn(verify::verify_sender_ident));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", SERVER_PORT))
        .await
        .unwrap();

    print_debug();
    axum::serve(listener, app).await.unwrap();
}

pub async fn root() -> &'static str {
    "://"
}

/// Handles webhook callbacks on the `<ROOT_URL>:<PORT>/webhook-global` endpoint
pub async fn webhook_handler(headers: HeaderMap, body: VerifiedBody) -> Result<Body, StatusCode> {
    let notification: serde_json::Value = body.as_json().map_err(|_| StatusCode::BAD_REQUEST)?;
    let msg_type = headers
        .get(TWITCH_MESSAGE_TYPE_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    println!("type: {}", msg_type);
    println!("content: {:#?}", notification);

    match msg_type {
        TYPE_CALLBACK_VERIFICATION => Ok(get_challenge_res(notification)
            .unwrap_or("".to_string())
            .into()),
        _ => {
            println!("");
            Ok("".into())
        }
    }
}

/// Returns the challenge string from the remote verification request
pub fn get_challenge_res(body: Value) -> Option<String> {
    let challenge_str = body["challenge"].as_str();
    if let Some(string) = challenge_str {
        Some(string.to_owned())
    } else {
        None
    }
}

/// Log server port and secret key string to stdout
///
/// Intended for debugging; this would be automatically sent on subscription to a topic in
/// production.
fn print_debug() {
    println!(
        "[+] Starting global webhook API running on port {}.",
        SERVER_PORT
    );

    // Remove these statements in prod??
    let digest_lock = &*KEY_DIGEST;
    if let Ok(digest) = digest_lock.read() {
        println!("[+] (DEBUG): Using secret '{}'", digest);
    }
}
