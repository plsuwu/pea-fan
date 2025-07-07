use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventTypeError {
    #[error("unknown EventType: {0}")]
    Conversion(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransportMethod {
    Webhook,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SubscriptionStatus {
    #[serde(rename = "notification")]
    Notify,
    #[serde(rename = "webhook_callback_verification")]
    Verify,
    #[serde(rename = "revocation")]
    Revoked,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Condition {
    pub broadcaster_user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transport {
    pub method: TransportMethod,
    pub callback: String,
    pub secret: String,
}

impl Transport {
    pub fn webhook(callback: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            method: TransportMethod::Webhook,
            callback: callback.into(),
            secret: secret.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebhookRequest {
    pub r#type: String,
    #[serde(default = "version_default")]
    pub version: String,
    pub condition: Condition,
    pub transport: Transport,
}

fn version_default() -> String {
    "1".to_string()
}

impl WebhookRequest {
    pub fn new(
        event_type: impl Into<String>,
        broadcaster_id: impl Into<String>,
        callback: impl Into<String>,
        secret: impl Into<String>,
    ) -> Self {
        Self {
            r#type: event_type.into(),
            version: version_default(),
            condition: Condition {
                broadcaster_user_id: broadcaster_id.into(),
            },
            transport: Transport::webhook(callback, secret),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Subscription {
    pub id: String,
    pub r#type: String,
    #[serde(default = "version_default")]
    pub version: String,
    pub status: SubscriptionStatus,
    pub cost: u32, // maybe isize idk
    pub condition: Condition,
    pub created_at: String, // could use chrono::DateTime<Utc> ??
    pub transport: Transport,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventMetadata {
    // fields that should always be present
    pub broadcaster_user_id: String,
    pub broadcaster_user_login: String,
    pub broadcaster_user_name: String,

    // present only when the event's type is stream online
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>, // could use chrono::DateTime<Utc>
}

// Main webhook payload
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebhookPayload {
    pub subscription: Subscription,
    pub event: EventMetadata,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    #[serde(rename = "stream.online")]
    StreamOnline,
    #[serde(rename = "stream.offline")]
    StreamOffline,
}

impl core::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::StreamOnline => write!(f, "stream.online"),
            EventType::StreamOffline => write!(f, "stream.offline"),
        }
    }
}

impl Into<String> for EventType {
    fn into(self) -> String {
        match self {
            EventType::StreamOnline => "stream.online".to_string(),
            EventType::StreamOffline => "stream.offline".to_string(),
        }
    }
}

impl core::str::FromStr for EventType {
    type Err = EventTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stream.online" => Ok(EventType::StreamOnline),
            "stream.offline" => Ok(EventType::StreamOffline),
            _ => Err(EventTypeError::Conversion(s.to_string())),
        }
    }
}

impl WebhookRequest {
    pub fn stream_online(broadcaster_id: &str, callback: &str, secret: &str) -> Self {
        Self::new(EventType::StreamOnline, broadcaster_id, callback, secret)
    }

    pub fn stream_offline(broadcaster_id: &str, callback: &str, secret: &str) -> Self {
        Self::new(EventType::StreamOffline, broadcaster_id, callback, secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_serialization() {
        assert_eq!(
            serde_json::to_string(&EventType::StreamOnline).unwrap(),
            "\"stream.online\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::StreamOffline).unwrap(),
            "\"stream.offline\""
        );
    }

    #[test]
    fn test_webhook_request_creation() {
        let request =
            WebhookRequest::stream_online("123456789", "https://example.com/webhook", "my_secret");

        assert_eq!(request.r#type, EventType::StreamOnline.to_string());
        assert_eq!(request.version, "1");
        assert_eq!(request.condition.broadcaster_user_id, "123456789");
    }
}
