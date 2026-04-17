pub mod dispatch;

use std::sync::Arc;

use axum::{body::Body, extract::State};
use http::{HeaderMap, StatusCode};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tracing::instrument;

use crate::api::middleware::verify_external::{TWITCH_MESSAGE_TYPE_HEADER, VerifiedBody};
use crate::api::server::AppState;
use crate::db::{prelude::ChannelId, redis::set_stream_state};
use crate::util::helix::HelixErr;

pub trait StreamCommonEvent {
    fn broadcaster_id(&self) -> &str;
    fn broadcaster_name(&self) -> &str;
    fn broadcaster_login(&self) -> &str;
}

pub trait StreamCommonSubscription {
    fn r#type(&self) -> &str;
}

#[instrument(skip(state, headers, body))]
pub async fn webhook_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: VerifiedBody,
) -> Result<Body, StatusCode> {
    tracing::debug!("parsing incoming webhook");

    let notification: serde_json::Value = body.as_json().map_err(|_| StatusCode::BAD_REQUEST)?;
    let msg_type: WebhookMessageType = headers
        .get(TWITCH_MESSAGE_TYPE_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?
        .try_into()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!(msg_type = ?msg_type, notification = %notification, "recv webhook notification");

    match msg_type {
        WebhookMessageType::Verify => {
            tracing::warn!("verify webhook");
            handle_verify(notification).await
        }
        WebhookMessageType::Notify => {
            tracing::warn!("notify webhook");
            handle_notify(&mut state.redis_pool.clone(), notification).await
        }
        WebhookMessageType::Revoke => {
            tracing::warn!("revoke webhook");
            todo!()
        }
    }
}

#[instrument(skip(redis_pool, body))]
pub async fn stream_event_notify<R: AsyncCommands + Sync, T>(
    redis_pool: &mut R,
    body: Value,
) -> Result<Body, StatusCode>
where
    T: StreamCommonEvent + StreamCommonSubscription + serde::de::DeserializeOwned + Clone + 'static,
{
    let payload: T = serde_json::from_value(body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let channel = if payload.broadcaster_login() == "testBroadcaster" {
        String::from("103033809")
    } else {
        payload.broadcaster_id().to_string()
    };

    let notif_type = format!("{}", payload.r#type());
    tracing::debug!(channel, notif_type, "recv event notification");

    if notif_type == "stream.online" {
        set_stream_state(redis_pool, &ChannelId(channel.clone()), true)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else if notif_type == "stream.offline" {
        set_stream_state(redis_pool, &ChannelId(channel.clone()), false)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(channel.into())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChallengeRequest {
    pub challenge: String,
    pub subscription: SubscriptionGenericData,
}

#[instrument]
pub async fn handle_verify(raw_json: Value) -> Result<Body, StatusCode> {
    let challenge: ChallengeRequest =
        serde_json::from_value(raw_json).map_err(|_| StatusCode::BAD_REQUEST)?;

    // let broadcaster_id = &challenge.subscription.condition.broadcaster_user_id;
    // if challenge.subscription.r#type == "stream.offline" {
    //     crate::db::redis::set_stream_state(&mut redis_pool().await?.clone(), broadcaster_id, ).await?;

    tracing::debug!(challenge_str = challenge.challenge, "recv challenge string");
    Ok(challenge.challenge.into())
}

#[instrument(skip(redis_pool))]
pub async fn handle_notify<R: AsyncCommands + Sync>(
    redis_pool: &mut R,
    raw_json: Value,
) -> Result<Body, StatusCode> {
    tracing::info!(?raw_json, "raw json body");
    match &raw_json["subscription"]["type"].as_str() {
        Some("stream.online") => {
            stream_event_notify::<R, StreamOnlinePayload>(redis_pool, raw_json).await
        }
        Some("stream.offline") => {
            stream_event_notify::<R, StreamOfflinePayload>(redis_pool, raw_json).await
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

pub type WebhookResult<T> = core::result::Result<T, WebhookError>;

#[derive(Debug, Error)]
pub enum WebhookError {
    #[error("failed to parse webhook message type '{0}'")]
    MessageTypeParseError(String),

    #[error(transparent)]
    HelixError(#[from] HelixErr),
}

#[derive(Debug)]
pub enum WebhookMessageType {
    Verify,
    Notify,
    Revoke,
}

impl WebhookMessageType {
    pub fn try_from_str(value: &str) -> WebhookResult<Self> {
        match value {
            "webhook_callback_verification" => Ok(Self::Verify),
            "notification" => Ok(Self::Notify),
            "revocation" => Ok(Self::Revoke),
            _ => Err(WebhookError::MessageTypeParseError(value.to_string())),
        }
    }
}

impl TryFrom<&str> for WebhookMessageType {
    type Error = WebhookError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        WebhookMessageType::try_from_str(value)
    }
}

#[derive(Debug)]
pub enum StreamGenericRequestType {
    Online,
    Offline,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamOnlinePayload {
    pub subscription: SubscriptionGenericData,
    pub event: StreamOnlineEvent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamOfflinePayload {
    pub subscription: SubscriptionGenericData,
    pub event: StreamOfflineEvent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamGenericRequest {
    pub r#type: String,
    pub version: String,
    pub condition: BroadcasterUserId,
    pub transport: Transport,
}

impl StreamGenericRequest {
    pub fn new(
        broadcaster_user_id: &str,
        callback: &str,
        secret: &str,
        r#type: StreamGenericRequestType,
    ) -> Self {
        let broadcaster_user_id = broadcaster_user_id.to_string();
        let condition = BroadcasterUserId {
            broadcaster_user_id,
        };
        let transport = Transport {
            method: "webhook".to_string(),
            callback: callback.to_string(),
            secret: Some(secret.to_owned()),
        };

        let notify_type = match r#type {
            StreamGenericRequestType::Online => String::from("stream.online"),
            StreamGenericRequestType::Offline => String::from("stream.offline"),
        };

        Self {
            r#type: notify_type,
            version: String::from("1"),
            condition,
            transport,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BroadcasterUserId {
    pub broadcaster_user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transport {
    /// Transport method; should be set to "webhook".
    pub method: String,
    pub callback: String,
    pub secret: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct HelixDataGeneric<T> {
    pub data: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscriptionGenericData {
    pub id: String,
    pub status: String,
    pub r#type: String,
    pub version: String,
    pub cost: usize,

    /// NOTE:
    ///  this doesn't handle for the case where we are required to
    ///  provide a `user_id` over a `broadcaster_user_id`
    pub condition: BroadcasterUserId,
    pub transport: Transport,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamOnlineEvent {
    pub id: String,
    pub broadcaster_user_id: String,
    pub broadcaster_user_login: String,
    pub broadcaster_user_name: String,
    pub r#type: String,
    pub started_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamOfflineEvent {
    pub broadcaster_user_id: String,
    pub broadcaster_user_login: String,
    pub broadcaster_user_name: String,
}

macro_rules! impl_stream_event {
    (
        $struct:ty,
        id: $id:ident,
        name: $name:ident,
        login: $login:ident
    ) => {
        impl StreamCommonEvent for $struct {
            fn broadcaster_id(&self) -> &str {
                &self.$id
            }

            fn broadcaster_name(&self) -> &str {
                &self.$name
            }

            fn broadcaster_login(&self) -> &str {
                &self.$login
            }
        }
    };
}

macro_rules! delegate_stream_common {
    ($struct:ty, $event_field:ident, $subscript_field:ident) => {
        impl StreamCommonEvent for $struct {
            fn broadcaster_id(&self) -> &str {
                self.$event_field.broadcaster_id()
            }

            fn broadcaster_name(&self) -> &str {
                self.$event_field.broadcaster_name()
            }

            fn broadcaster_login(&self) -> &str {
                self.$event_field.broadcaster_login()
            }
        }

        impl StreamCommonSubscription for $struct {
            fn r#type(&self) -> &str {
                &self.$subscript_field.r#type
            }
        }
    };
}

impl_stream_event!(
    StreamOnlineEvent,
    id: broadcaster_user_id,
    name: broadcaster_user_name,
    login: broadcaster_user_login
);

impl_stream_event!(
    StreamOfflineEvent,
    id: broadcaster_user_id,
    name: broadcaster_user_name,
    login: broadcaster_user_login
);

delegate_stream_common!(StreamOnlinePayload, event, subscription);
delegate_stream_common!(StreamOfflinePayload, event, subscription);
