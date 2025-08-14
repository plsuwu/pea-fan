use crate::constants::{STREAM_OFFLINE, STREAM_ONLINE, VERSION};
use serde::{Deserialize, Serialize};

pub trait StreamCommonEvent {
    fn broadcaster_id(&self) -> &str;
    fn broadcaster_name(&self) -> &str;
    fn broadcaster_login(&self) -> &str;
}

pub trait StreamCommonSubscription {
    fn r#type(&self) -> &str;
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

pub enum StreamGenericRequestType {
    Online,
    Offline,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamGenericRequest {
    pub r#type: String,
    pub version: String,
    pub condition: ConditionBroadcasterUID,
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
        let condition = ConditionBroadcasterUID {
            broadcaster_user_id,
        };
        let transport = Transport {
            method: "webhook".to_string(),
            callback: callback.to_string(),
            secret: Some(secret.to_owned()),
        };

        let notify_type = match r#type {
            StreamGenericRequestType::Online => STREAM_ONLINE.to_string(),
            StreamGenericRequestType::Offline => STREAM_OFFLINE.to_string(),
        };

        Self {
            r#type: notify_type,
            version: VERSION.to_string(),
            condition,
            transport,
        }
    }
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscriptionGenericResponse {
    pub data: Vec<SubscriptionGenericData>,
    pub total: usize,
    pub total_cost: usize,
    pub max_total_cost: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConditionMultiUID {
    /// User ID of the channel for which to receive chat message events for
    broadcaster_user_id: String,
    /// User ID to read chat as
    user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConditionBroadcasterUID {
    /// User ID of the channel for which to receive chat message events for
    pub broadcaster_user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Subscription {
    // is this the correct type??
    /// Client ID
    pub id: String,
    /// Notification's subscription type
    pub r#type: String,
    ///
    pub version: String,
    pub status: String,
    pub cost: isize,
    // condition: Condition,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChallengeRequest {
    pub challenge: String,
    pub subscription: SubscriptionGenericData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transport {
    /// Transport method.
    ///
    /// Should be set to "webhook".
    pub method: String,
    /// The callback URL where the notifications are sent. The URL must use the HTTPS
    /// protocol and port 443.
    ///
    /// > Note that redirects are not followed.
    pub callback: String,
    /// Secret used to verify the signature.
    ///
    /// Required during a request, not included in the body of a response.
    ///
    /// Secret must be:
    /// - ASCII string
    /// - at least 10 characters
    /// - at most 100 characters
    pub secret: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscriptionGenericData {
    pub id: String,
    pub status: String,
    pub r#type: String,
    pub version: String,
    pub cost: usize,

    /// Note that this doesn't handle for the case where we need to provide a `user_id` string.
    ///
    /// We don't CURRENTLY need it but this should be kept in mind if we transition functionality
    /// away from IRC over websockets for reading chat
    pub condition: ConditionBroadcasterUID,
    pub transport: Transport,
    pub created_at: String,
}
