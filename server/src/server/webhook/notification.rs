use super::super::midware::verify::VerifiedBody;
use super::dispatch::WebhookMessageType;
use super::dispatch::stream_event_notify;
use crate::args::get_cli_args;
use crate::constants::TWITCH_MESSAGE_TYPE_HEADER;
use crate::constants::{STREAM_OFFLINE, STREAM_ONLINE};
use crate::server::types::{ChallengeRequest, StreamOfflinePayload, StreamOnlinePayload};
use crate::server::webhook::dispatch::open_websocket;
use crate::server::webhook::subscriber::{UsersQueryData, check_stream_state, get_user_data};
use crate::socket::client::get_current_time;
use axum::body::Body;
use http::{HeaderMap, StatusCode};
use serde_json::Value;

/// Handles webhook callbacks on the `<ROOT_URL>:<PORT>/webhook-global` endpoint
pub async fn webhook_handler(headers: HeaderMap, body: VerifiedBody) -> Result<Body, StatusCode> {
    let notification = get_notification_body(body)?;
    let webhook_msg_type = get_notification_type(headers)?;

    match webhook_msg_type {
        WebhookMessageType::Verify => handle_verify(notification).await,
        WebhookMessageType::Notify => handle_message(notification).await,
        WebhookMessageType::Revoke => handle_revoke(notification).await,
    }
}

pub fn get_notification_body(body: VerifiedBody) -> Result<Value, StatusCode> {
    body.as_json().map_err(|_| StatusCode::BAD_REQUEST)
}

pub fn get_notification_type(headers: HeaderMap) -> Result<WebhookMessageType, StatusCode> {
    let msg_type = headers
        .get(TWITCH_MESSAGE_TYPE_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    msg_type
        .try_into()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_broadcaster_data(broadcaster_id: &str) -> Result<UsersQueryData, StatusCode> {
    let token = get_cli_args();
    get_user_data(&token.app_token, broadcaster_id)
        .await
        .map_err(|e| {
            eprintln!(
                "[{}] failed to get data for {}: {:?}",
                get_current_time(),
                broadcaster_id,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn handle_verify(notification: Value) -> Result<Body, StatusCode> {
    let challenge: ChallengeRequest =
        serde_json::from_value(notification).map_err(|_| StatusCode::BAD_REQUEST)?;

    let broadcaster_id = &challenge.subscription.condition.broadcaster_user_id;
    let broadcaster_data = get_broadcaster_data(broadcaster_id).await?;

    if challenge.subscription.r#type == STREAM_OFFLINE {
        handle_open_on_rx_offline(broadcaster_id, &broadcaster_data.login).await;
    }

    log_challenge_res(&broadcaster_data.login, &challenge);
    Ok(challenge.challenge.into())
}

pub async fn handle_open_on_rx_offline(broadcaster_id: &str, broadcaster_login: &str) {
    let token = get_cli_args();
    match check_stream_state(&token.app_token, broadcaster_id).await {
        Ok(true) => spawn_websocket(broadcaster_login),
        Err(e) => {
            println!(
                "[x] failed to retrieve broadcaster '{}' online state: {:?}",
                broadcaster_login, e
            );
        }
        _ => (),
    }
}

pub fn spawn_websocket(broadcaster_login: &str) {
    let login = broadcaster_login.to_owned();

    tokio::task::spawn(async move {
        if let Err(e) = open_websocket(&login).await {
            eprintln!("[x] failed during websocket open for '{}': {:?}", login, e);
        } else {
            println!("[x] opened socket for '{}'", login);
        }
    });
}

pub fn log_challenge_res(broadcaster_login: &str, challenge: &ChallengeRequest) {
    println!(
        "[+] responding to challenge for '{}-{}' with {}",
        broadcaster_login, challenge.subscription.r#type, challenge.challenge,
    );
}

pub async fn handle_message(body: Value) -> Result<Body, StatusCode> {
    match &body["subscription"]["type"].as_str() {
        Some(STREAM_ONLINE) => {
            stream_event_notify::<StreamOnlinePayload>(body).map_err(|_| StatusCode::BAD_REQUEST)
        }
        Some(STREAM_OFFLINE) => {
            stream_event_notify::<StreamOfflinePayload>(body).map_err(|_| StatusCode::BAD_REQUEST)
        }
        _ => Ok(format!("TYPE({})_NOT_IMPLEMENTED", body["subscription"]["type"]).into()),
    }
}

pub async fn handle_revoke(notification: Value) -> Result<Body, StatusCode> {
    let rev = format!("[x] rx REVOCATION: {:#?}", notification);
    println!("{}", rev);

    Ok(rev.into())
}
