use std::time::Instant;

use thiserror::Error;
use tokio::sync::oneshot;

use crate::database::schema::Channel;
use crate::parsing;
use crate::parsing::commands::UserInfo;
use crate::parsing::parser::IrcAst;
use crate::socket::pool::{DEFAULT_CAPS, DEFAULT_IRC, IrcConnectionPool};
use crate::util::secrets::ENV_SECRETS;

pub struct PooledIrcClient {
    pool: IrcConnectionPool,
    event_rx: tokio::sync::broadcast::Receiver<IrcEvent>,
}

#[derive(Debug, Error, Clone)]
pub enum IrcError {
    #[error("websocket client failed to connect: {0}")]
    ConnectionFailed(String),

    #[error("error while parsing message: {0}")]
    ParseError(#[from] parsing::parser::ParseError),

    #[error("websocket client error: {0}")]
    WebsocketClientError(String),

    #[error("channel limit reached")]
    ChannelLimitReached,

    #[error("connection timed out")]
    Timeout,
}

#[derive(Debug, Clone)]
pub enum IrcEvent {
    Connected,
    Disconnected,
    RawMsg(IrcAst),

    PrivMsgRx {
        channel: String,
        user_id: String,
        message: String,
        user_info: Option<UserInfo>,
    },

    NoticeRx {
        target: String,
        message: String,
    },

    UserNoticeRx {
        channel: String,
        message: Option<String>,
        msg_id: Option<String>,
        user_info: Option<UserInfo>,
    },

    UserStateRx {
        channel: String,
        message: Option<String>,
        msg_id: Option<String>,
        user_info: Option<UserInfo>,
    },

    ClearChat {
        channel: String,
        target_user: Option<String>,
        duration: Option<u64>,
    },

    ClearMsg {
        channel: String,
        target_msg_id: String,
    },

    Numeric {
        code: u16,
        params: Vec<String>,
    },

    Unknown {
        command: String,
        params: Vec<String>,
    },

    PingRx(String),
    PongRx(String),
    Error(IrcError),
}

impl IrcEvent {
    pub fn is_privmsg(&self) -> bool {
        match self {
            IrcEvent::PrivMsgRx { .. } => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum IrcCommand {
    JoinChannel(String, oneshot::Sender<Result<(), IrcError>>),
    LeaveChannel(String, oneshot::Sender<Result<(), IrcError>>),
    SendMessage(String, String, oneshot::Sender<Result<(), IrcError>>),
    GetChannels(oneshot::Sender<Vec<String>>),
    Disconnect(oneshot::Sender<()>),
}

#[derive(Debug, Clone)]
pub struct IrcChannel {
    pub channel: String,
    pub broadcaster_id: String,
    pub channel_internal: Channel,
    pub joined: Instant,
}

#[derive(Debug, Clone)]
pub struct IrcAuthentication {
    pub caps: String,
    pub pass: String,
    pub nick: String,
    pub user: String,
}

impl IrcAuthentication {
    pub fn new(caps: Option<&str>) -> Self {
        let token = ENV_SECRETS.user_token();
        let login = ENV_SECRETS.user_login();

        let caps = caps.unwrap_or(DEFAULT_CAPS).to_string();
        let pass = format!("PASS oauth:{}", token);
        let nick = format!("NICK {}", login);
        let user = format!("USER {} 8 * :{}", login, login);

        Self {
            caps,
            pass,
            nick,
            user,
        }
    }
}

#[derive(Debug)]
pub struct TwitchAuth {
    pub caps: String,
    pub pass: String,
    pub nick: String,
    pub user: String,
}

impl TwitchAuth {
    pub fn new(caps: Option<&str>) -> Self {
        let token = ENV_SECRETS.user_token();
        let login = ENV_SECRETS.user_login();

        let caps = caps.unwrap_or(DEFAULT_CAPS).to_string();
        let pass = format!("PASS oauth:{}", token);
        let nick = format!("NICK {}", login);
        let user = format!("USER {} 8 * :{}", login, login);

        Self {
            caps,
            pass,
            nick,
            user,
        }
    }
}
