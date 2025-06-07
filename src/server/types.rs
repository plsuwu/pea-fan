use serde::{Deserialize, Serialize};

const CHANNEL_CHAT_MESSAGE: &'static str = "channel.chat.message";
const VERSION: &'static str = "1";

#[derive(Serialize, Deserialize)]
pub struct ChannelChatMessageRequest {
    pub r#type: String,
    pub version: String,
    pub condition: Condition,
    pub transport: Transport,
}

impl ChannelChatMessageRequest {
    pub fn new(broadcaster_user_id: &str, user_id: &str, callback: &str, secret: &str) -> Self {
        let broadcaster_user_id = broadcaster_user_id.to_string();
        let user_id = user_id.to_string();

        let condition: Condition = {
            Condition::ChannelChatMessage {
                broadcaster_user_id,
                user_id,
            }
        };
        let transport = Transport {
            method: "webhook".to_string(),
            callback: callback.to_string(),
            secret: secret.to_string(),
        };

        Self {
            r#type: CHANNEL_CHAT_MESSAGE.to_string(),
            version: VERSION.to_string(),
            condition,
            transport,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChannelChatMessagePayload {
    subscription: Subscription,
    event: ChannelChatMessageEvent,
}

#[derive(Serialize, Deserialize)]
pub struct ChannelChatMessageEvent {
    broadcaster_user_id: String,
    broadcaster_user_name: String,
    broadcaster_user_login: String,

    chatter_user_id: String,
    chatter_user_name: String,
    chatter_user_login: String,

    message_id: String,
    message: MessageEventMessage,
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
    message_type: String,
    badges: Vec<Badges>,
    cheer: Option<Cheer>,
    color: String,
    reply: Option<Reply>,
    channel_points_custom_reward_id: Option<String>,

    source_broadcaster_user_id: Option<String>,
    source_broadcaster_user_name: Option<String>,
    source_broadcaster_user_login: Option<String>,
    source_message_id: Option<String>,
    source_badges: Option<Badges>,
    is_source_only: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Reply {
    parent_message_id: String,
    parent_message_body: String,
    parent_user_id: String,
    parent_user_name: String,
    parent_user_login: String,
    thread_message_id: String,
    thread_user_id: String,
    thread_user_login: String,
}

#[derive(Serialize, Deserialize)]
pub struct Badges {
    set_id: String,
    id: String,
    info: String,
}

#[derive(Serialize, Deserialize)]
pub struct Cheer {
    bits: usize,
}

#[derive(Serialize, Deserialize)]
pub struct MessageEventMessage {
    text: String,
    fragments: Vec<Fragments>,
}

#[derive(Serialize, Deserialize)]
pub struct Fragments {
    /// Type of message fragment.
    ///
    /// # Possible values
    ///
    /// - "text"
    /// - "cheermote"
    /// - "emote"
    /// - "mention"
    r#type: String,
    text: String,
    cheermote: Option<Cheermote>,
    emote: Option<Emote>,
    mention: Option<Mention>,
}

/// Metadata pertaining to a cheermote
#[derive(Serialize, Deserialize)]
pub struct Cheermote {
    prefix: String,
    bits: usize,
    tier: usize,
}

/// Metadata pertaining to an emote
#[derive(Serialize, Deserialize)]
pub struct Emote {
    id: String,
    emote_set_id: String,
    owner_id: String,
    format: Vec<String>,
}

/// Metadata pertaining to a mention
#[derive(Serialize, Deserialize)]
pub struct Mention {
    user_id: String,
    user_name: String,
    user_login: String,
}

#[derive(Serialize, Deserialize)]
pub enum Condition {
    /// `Channel Chat Message` condition
    ChannelChatMessage {
        /// User ID of the channel for which to receive chat message events for
        broadcaster_user_id: String,
        /// User ID to read chat as
        user_id: String,
    },

    /// `Channel Subscribe` condition
    ChannelSubscribe {
        /// User ID of the channel for which to receive subscribe notifications
        broadcaster_user_id: String,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Subscription {
    // is this the correct type??
    /// Client ID
    id: String,
    /// Notification's subscription type
    r#type: String,
    ///
    version: String,
    status: String,
    cost: isize,
    // condition: Condition,
    created_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct Transport {
    /// Transport method.
    ///
    /// Should be set to "webhook".
    method: String,
    /// The callback URL where the notifications are sent. The URL must use the HTTPS protocol and port 443.
    ///
    /// > Redirects are NOT followed.
    callback: String,
    /// Secret used to verify the signature.
    ///
    /// Secret must be:
    /// - ASCII string
    /// - at least 10 characters
    /// - at most 100 characters
    secret: String,
}

#[derive(Debug)]
pub struct Challenge {
    challenge: String,

}

pub struct ChallengeSubscription {
    id: String,
    status: String,
    r#type: String,
    version: String,
    cost: String,
    condition: Condition,
    transport: Transport,
    created_at: String,
}
