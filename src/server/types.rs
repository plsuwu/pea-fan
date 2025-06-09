use serde::{Deserialize, Serialize};

const CHANNEL_CHAT_MESSAGE: &'static str = "channel.chat.message";
const STREAM_ONLINE: &'static str = "stream.online";
const STREAM_OFFLINE: &'static str = "stream.offline";
const VERSION: &'static str = "1";

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

//
// ---------------------------------------------------------------------------------------------------
// --- idk how many of the structs below are actually still required (its definitely some of them) ---
// ---------------------------------------------------------------------------------------------------
//

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscriptionGenericResponse {
    pub data: Vec<SubscriptionGenericData>,
    pub total: usize,
    pub total_cost: usize,
    pub max_total_cost: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelChatMessagePayload {
    pub subscription: Subscription,
    pub event: ChannelChatMessageEvent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelSubscriptionMessagePayload {
    pub subscription: Subscription,
    pub event: ChannelSubscriptionMessageEvent,
}

pub trait ChatMessageCommon {
    fn user_id(&self) -> &str;
    fn user_name(&self) -> &str;
    fn user_login(&self) -> &str;
    fn broadcaster_user_id(&self) -> &str;
    fn broadcaster_user_name(&self) -> &str;
    fn broadcaster_user_login(&self) -> &str;
    fn message(&self) -> &Message;
}

macro_rules! impl_chat_common {
    (
        $struct:ty,
        user_id: $user_id_field:ident,
        user_name: $user_name_field:ident,
        user_login: $user_login_field:ident,
        broadcaster_user_id: $broadcaster_user_id_field:ident,
        broadcaster_user_name: $broadcaster_user_name_field:ident,
        broadcaster_user_login: $broadcaster_user_login_field:ident,
        message: $message_field:ident,
    ) => {
        impl ChatMessageCommon for $struct {
            fn user_id(&self) -> &str {
                &self.$user_id_field
            }

            fn user_name(&self) -> &str {
                &self.$user_name_field
            }

            fn user_login(&self) -> &str {
                &self.$user_login_field
            }

            fn broadcaster_user_id(&self) -> &str {
                &self.$broadcaster_user_id_field
            }

            fn broadcaster_user_name(&self) -> &str {
                &self.$broadcaster_user_name_field
            }

            fn broadcaster_user_login(&self) -> &str {
                &self.$broadcaster_user_login_field
            }

            fn message(&self) -> &Message {
                &self.$message_field
            }
        }
    };
}

impl_chat_common!(
    ChannelSubscriptionMessageEvent,
    user_id: user_id,
    user_name: user_name,
    user_login: user_login,
    broadcaster_user_id: broadcaster_user_id,
    broadcaster_user_name: broadcaster_user_name,
    broadcaster_user_login: broadcaster_user_login,
    message: message,
);

impl_chat_common!(
    ChannelChatMessageEvent,
    user_id: chatter_user_id,
    user_name: chatter_user_name,
    user_login: chatter_user_login,
    broadcaster_user_id: broadcaster_user_id,
    broadcaster_user_name: broadcaster_user_name,
    broadcaster_user_login: broadcaster_user_login,
    message: message,
);

macro_rules! delegate_common {
    ($struct:ty, $field:ident) => {
        impl ChatMessageCommon for $struct {
            fn user_id(&self) -> &str {
                self.$field.user_id()
            }
            fn user_name(&self) -> &str {
                self.$field.user_name()
            }
            fn user_login(&self) -> &str {
                self.$field.user_login()
            }
            fn broadcaster_user_id(&self) -> &str {
                self.$field.broadcaster_user_id()
            }
            fn broadcaster_user_name(&self) -> &str {
                self.$field.broadcaster_user_name()
            }
            fn broadcaster_user_login(&self) -> &str {
                self.$field.broadcaster_user_login()
            }
            fn message(&self) -> &Message {
                self.$field.message()
            }
        }
    };
}

delegate_common!(ChannelChatMessagePayload, event);
delegate_common!(ChannelSubscriptionMessagePayload, event);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelSubscriptionMessageEvent {
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub broadcaster_user_id: String,
    pub broadcaster_user_login: String,
    pub broadcaster_user_name: String,
    pub tier: String,
    pub message: Message,
    pub cumulative_months: usize,
    pub streak_months: Option<usize>,
    pub duration_months: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelChatMessageEvent {
    pub broadcaster_user_id: String,
    pub broadcaster_user_name: String,
    pub broadcaster_user_login: String,

    pub chatter_user_id: String,
    pub chatter_user_name: String,
    pub chatter_user_login: String,

    pub message_id: String,
    pub message: Message,
    /// Type of the message
    ///
    /// # Possible values
    ///
    /// - "text"
    /// - "channel_points_highlighted"
    /// - "channel_points_sub_only"
    /// - "user_intro"
    /// - "power_ups_message_effect"
    /// - "power_ups_gigantified_emote"
    pub message_type: String,
    pub badges: Vec<Badges>,
    pub cheer: Option<Cheer>,
    pub color: String,
    pub reply: Option<Reply>,
    pub channel_points_custom_reward_id: Option<String>,

    pub source_broadcaster_user_id: Option<String>,
    pub source_broadcaster_user_name: Option<String>,
    pub source_broadcaster_user_login: Option<String>,
    pub source_message_id: Option<String>,
    pub source_badges: Option<Badges>,
    pub is_source_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reply {
    pub parent_message_id: String,
    pub parent_message_body: String,
    pub parent_user_id: String,
    pub parent_user_name: String,
    pub parent_user_login: String,
    pub thread_message_id: String,
    pub thread_user_id: String,
    pub thread_user_login: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Badges {
    pub set_id: String,
    pub id: String,
    pub info: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cheer {
    pub bits: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub text: String,
    pub fragments: Option<Vec<Fragments>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fragments {
    /// Type of message fragment.
    ///
    /// # Possible values
    ///
    /// - "text"
    /// - "cheermote"
    /// - "emote"
    /// - "mention"
    pub r#type: String,
    pub text: String,
    pub cheermote: Option<Cheermote>,
    pub emote: Option<Emote>,
    pub mention: Option<Mention>,
}

/// Metadata pertaining to a cheermote
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cheermote {
    pub prefix: String,
    pub bits: usize,
    pub tier: usize,
}

/// Metadata pertaining to an emote
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Emote {
    pub id: String,
    pub emote_set_id: String,
    pub owner_id: String,
    pub format: Vec<String>,
}

/// Metadata pertaining to a mention
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mention {
    pub user_id: String,
    pub user_name: String,
    pub user_login: String,
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
    broadcaster_user_id: String,
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
    pub condition: ConditionBroadcasterUID,
    pub transport: Transport,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelChatMessageRequest {
    pub r#type: String,
    pub version: String,
    pub condition: ConditionMultiUID,
    pub transport: Transport,
}

#[allow(dead_code)]
impl ChannelChatMessageRequest {
    pub fn new(broadcaster_user_id: &str, user_id: &str, callback: &str, secret: &str) -> Self {
        let broadcaster_user_id = broadcaster_user_id.to_string();
        let user_id = user_id.to_string();

        let condition = ConditionMultiUID {
            broadcaster_user_id,
            user_id,
        };

        let transport = Transport {
            method: "webhook".to_string(),
            callback: callback.to_string(),
            secret: Some(secret.to_string()),
        };

        Self {
            r#type: CHANNEL_CHAT_MESSAGE.to_string(),
            version: VERSION.to_string(),
            condition,
            transport,
        }
    }
}
