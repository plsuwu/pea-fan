use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub message_id: String,
    pub message_type: String,
    pub message_timestamp: String,
    pub subscription_type: Option<String>,
    pub subscription_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub id: String,
    pub status: String,
    pub keepalive_timeout_seconds: usize,
    pub reconnect_url: String,
    pub connected_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SocketSessionPayload {
    pub session: Session,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SocketSubscriptionPayload {
    subscription: SocketSubscription,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConditionUserBroadcasterId {
    broadcaster_user_id: String,
    user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SocketTransport {
    /// Must be 'websocket'
    method: String,
    session_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SocketSubscription {
    id: String,
    status: String,
    r#type: String,
    version: String,
    cost: String,
    condition: Option<ConditionUserBroadcasterId>,
    transport: Option<SocketTransport>,
    created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Empty;

/// Defines the first message that the EventSub WebSocket server sends after client connection to
/// the server.
///
/// > [Read more]
///
/// [Read more]: https://dev.twitch.tv/docs/eventsub/handling-websocket-events#welcome-message
#[derive(Serialize, Deserialize, Debug)]
pub struct SocketWelcome {
    metadata: Metadata,
    payload: SocketSessionPayload,
}

/// Defines the message that the EventSub WebSocket server sends the client to indicate that the
/// WebSocket connection is healthy
///
/// > [Read more]
///
/// [Read more]: https://dev.twitch.tv/docs/eventsub/handling-websocket-events#keepalive-message
#[derive(Serialize, Deserialize, Debug)]
pub struct SocketKeepalive {
    metadata: Metadata,
    payload: Empty,
}

/// Defines a message that the EventSub WebSocket server sends your client when an event that you
/// subcribe to occurs.
///
/// > [Read more]
///
/// [Read more]: https://dev.twitch.tv/docs/eventsub/handling-websocket-events#notification-message
#[derive(Serialize, Deserialize, Debug)]
pub struct SocketChatMessageNotification {
    metadata: Metadata,
    payload: ChannelChatMessagePayload,
}

/// Defines a message that the EventSub WebSocket server sends if the server must drop the
/// connection
///
/// > [Read more]
///
/// [Read more]: https://dev.twitch.tv/docs/eventsub/handling-websocket-events#reconnect-message
#[derive(Serialize, Deserialize, Debug)]
pub struct SocketReconnect {
    metadata: Metadata,
    payload: SocketSessionPayload,
}

/// Defines a message that the EventSub WebSocket server sends if the user no longer exists or they
/// revoked the authorization token that the subscription relied on.
///
/// > [Read more]
///
/// [Read more]: https://dev.twitch.tv/docs/eventsub/handling-websocket-events#revocation-message
#[derive(Serialize, Deserialize, Debug)]
pub struct SocketRevocation {
    metadata: Metadata,
    payload: SocketSubscriptionPayload,
}

/// A standard WebSocket [Close] frame.
///
/// > [Read more]
///
/// [Close]: https://datatracker.ietf.org/doc/html/rfc6455#section-5.5.1
/// [Read more]: https://dev.twitch.tv/docs/eventsub/handling-websocket-events#close-message
pub enum SocketClose {
    /// Indicates a problem with the server (similar to an HTTP 500 status code).
    InternalServerError = 4000,
    /// Sending outgoing messages to the server is prohibited with the exception of pong
    /// messages.
    ClientSentInboundTraffic = 4001,
    /// You must respond to ping messages with a pong message. See [Ping message].
    ///
    /// [Ping message]: https://dev.twitch.tv/docs/eventsub/websocket-reference/#ping-message
    ClientFailedPing = 4002,
    /// When you connect to the server, you must create a subscription within 10 seconds or the
    /// connection is closed. The time limit is subject to change.
    ConnectionUnused = 4003,
    /// When you receive a session_reconnect message, you have 30 seconds to reconnect to the
    /// server and close the old connection. See [Reconnect message].
    ///
    /// [Reconnect message]: https://dev.twitch.tv/docs/eventsub/websocket-reference/#reconnect-message
    ReconnectGracePeriodExpired = 4004,
    /// Transient network timeout.
    NetworkTimeout = 4005,
    /// Transient network error.
    NetworkError = 4006,
    /// The reconnect URL is invalid.
    InvalidReconnect = 4007,
}

/// Payload field for an incoming chat message/chat notification.
#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelChatMessagePayload {
    pub subscription: SocketSubscription,
    pub event: ChannelChatMessageEvent,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Badges {
    pub set_id: String,
    pub id: String,
    pub info: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cheer {
    pub bits: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub text: String,
    pub fragments: Option<Vec<Fragments>>,
}

#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
pub struct Cheermote {
    pub prefix: String,
    pub bits: usize,
    pub tier: usize,
}

/// Metadata pertaining to an emote
#[derive(Serialize, Deserialize, Debug)]
pub struct Emote {
    pub id: String,
    pub emote_set_id: String,
    pub owner_id: String,
    pub format: Vec<String>,
}

/// Metadata pertaining to a mention
#[derive(Serialize, Deserialize, Debug)]
pub struct Mention {
    pub user_id: String,
    pub user_name: String,
    pub user_login: String,
}
