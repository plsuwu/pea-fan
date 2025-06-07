use axum::{
    Router,
    extract::Json,
    http::HeaderMap,
    routing::{get, post},
};

use serde_json::Value;

const HMAC_PREFIX: &'static str = "sha256=";
const TWITCH_MESSAGE_ID: &'static str = "Twitch-Eventsub-Message-Id";
const TWITCH_MESSAGE_TIMESTAMP: &'static str = "Twitch-Eventsub-Message-Timestamp";
const TWITCH_MESSAGE_SIGNATURE: &'static str = "Twitch-Eventsub-Message-Signature";
const TWITCH_MESSAGE_TYPE: &'static str = "Twitch-Eventsub-Message-Type";

pub async fn server_main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/api/callback", get(get_callback).post(callback));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub async fn root() -> &'static str {
    "://"
}

pub async fn callback(headers: HeaderMap, Json(body): Json<Value>) -> String {
    println!("{:#?}", headers);
    println!("{:#?}", body);

    let mut result: String = "".to_string();

    if let Some(message_type) = headers.get(TWITCH_MESSAGE_TYPE) {
        match message_type.to_str().unwrap() {
            "webhook_callback_verification" => result = get_challenge_res(body),
            _ => result = "".to_string(),
        }
    }

    result
}

pub fn get_challenge_res(body: Value) -> String {
    let challenge_str = body["challenge"].as_str();
    if let Some(string) = challenge_str {
        return string.to_owned();
    } else {
        return "".to_string();
    }
}

pub async fn get_callback() -> &'static str {
    "method not supported :("
}

