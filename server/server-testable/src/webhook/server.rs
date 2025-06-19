use crate::ws::client::WsClientError;
use crate::ws::connection::{Connection, WsConnection};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebhookError {
    #[error("invalid webhook type: '{}'", webhook_type)]
    InvalidType { webhook_type: String },

    #[error("missing required field '{}' in payload", field_name)]
    MissingField { field_name: String },

    #[error(
        "invalid event type '{}' for broadcaster '{}'",
        event_type,
        broadcaster
    )]
    InvalidEvent {
        event_type: String,
        broadcaster: String,
    },
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("failed to create websocket: {}", source)]
    WsClient {
        #[source]
        source: WsClientError,
        channel: String,
    },

    #[error("connection error for channel '{}': {}", channel, kind)]
    Dispatch {
        channel: String,
        #[source]
        kind: DispatchError,
    },

    #[error("webhook processing error: {}", source)]
    Webhook {
        #[source]
        source: WebhookError,
    },

    #[error("configuration error: {}", message)]
    Configuration { message: String },

    #[error("JSON processing error: {}", source)]
    Json {
        #[source]
        source: serde_json::Error,
    },
}

#[derive(Debug, Error)]
pub enum DispatchError {
    #[error("connection already exists and is active")]
    Conflict,

    #[error("connection not found")]
    NotFound,

    #[error("connection state is invalid: {}", state)]
    InvalidState { state: String },

    #[error("timeout during shutdown")]
    Timeout,

    #[error("connection task panicked: {}", reason)]
    Panic { reason: String },
}

impl From<serde_json::Error> for ServerError {
    fn from(source: serde_json::Error) -> Self {
        ServerError::Json { source }
    }
}

impl From<WebhookError> for ServerError {
    fn from(source: WebhookError) -> Self {
        ServerError::Webhook { source }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebhookMessageType {
    Verify,
    Notify,
    Revoke,
}

pub type WebhookResult<T> = core::result::Result<T, WebhookError>;
pub type DispatchResult<T> = core::result::Result<T, DispatchError>;

impl WebhookMessageType {
    pub fn parse_from_str(value: &str) -> WebhookResult<Self> {
        match value {
            "webhook_callback_verification" => Ok(Self::Verify),
            "notification" => Ok(Self::Notify),
            "revocation" => Ok(Self::Revoke),
            _ => Err(WebhookError::InvalidType {
                webhook_type: value.to_string(),
            }),
        }
    }
}

impl TryFrom<&str> for WebhookMessageType {
    type Error = WebhookError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse_from_str(value)
    }
}

#[async_trait]
pub trait WsDispatchHandler: Send + Sync {
    async fn open_connection(&self, channel: &str) -> DispatchResult<()>;
    async fn close_connection(&self, channel: &str) -> DispatchResult<bool>;
    fn is_active(&self, channel: &str) -> bool;
    fn get_summary(&self) -> (Vec<String>, Vec<String>);
    fn cleanup(&self);
}


