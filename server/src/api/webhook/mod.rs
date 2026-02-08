#![allow(dead_code)]

pub mod dispatch;

use axum::body::Body;
use http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::instrument;

use crate::{
    api::middleware::verify_external::{TWITCH_MESSAGE_TYPE_HEADER, VerifiedBody},
    util::helix::HelixErr,
};

pub trait StreamCommonEvent {
    fn broadcaster_id(&self) -> &str;
    fn broadcaster_name(&self) -> &str;
    fn broadcaster_login(&self) -> &str;
}

pub trait StreamCommonSubscription {
    fn r#type(&self) -> &str;
}

#[instrument(skip(headers, body))]
pub async fn webhook_handler(headers: HeaderMap, body: VerifiedBody) -> Result<Body, StatusCode> {
    tracing::debug!("parsing incoming webhook");

    let notification: serde_json::Value = body.as_json().map_err(|_| StatusCode::BAD_REQUEST)?;
    let msg_type: WebhookMessageType = headers
        .get(TWITCH_MESSAGE_TYPE_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?
        .try_into()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!(msg_type = ?msg_type, notification = %notification, "WEBHOOK::INCOMING");

    match msg_type {
        WebhookMessageType::Verify => {
            tracing::warn!("verify webhook");
            todo!()
        }
        WebhookMessageType::Notify => {
            tracing::warn!("notify webhook");
            todo!()
        }
        WebhookMessageType::Revoke => {
            tracing::warn!("revoke webhook");
            todo!()
        }
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
