use std::{sync::Arc, time::Duration};

use thiserror::Error;
use tokio::sync::{RwLock, mpsc, oneshot};

use crate::{
    socket::{
        client::{ChannelConfig, SocketClient},
        core::EventHandler,
    },
    util::secrets::ENV_SECRETS,
};

pub const DEFAULT_CAPS: &str = "CAP REQ :twitch.tv/tags twitch.tv/commands";
pub const DEFAULT_IRC: &str = "wss://irc-ws.chat.twitch.tv/";

pub type SocketPoolResult<T> = core::result::Result<T, SocketPoolError>;

#[derive(Error, Debug)]
pub enum SocketPoolError {
    #[error("MPSC Send error (ClientCommand): {0}")]
    MpscSend(#[from] mpsc::error::SendError<ClientCommand>),

    #[error("No clients available to handle join request")]
    NoClients,
}

#[derive(Debug)]
pub struct SocketPool {
    config: SocketPoolConfig,
    connections: Vec<mpsc::UnboundedSender<ClientCommand>>,
    command_rx: mpsc::UnboundedReceiver<PoolCommand>,
    handler: Arc<dyn EventHandler>,
    client_states: Arc<RwLock<Vec<ClientState>>>,
}

impl SocketPool {
    pub fn new(
        config: SocketPoolConfig,
        handler: Arc<dyn EventHandler>,
    ) -> (Self, mpsc::UnboundedSender<PoolCommand>) {
        let (tx, rx) = mpsc::unbounded_channel();

        let pool = Self {
            config,
            connections: Vec::new(),
            command_rx: rx,
            handler,
            client_states: Arc::new(RwLock::new(Vec::new())),
        };

        (pool, tx)
    }

    pub async fn start(&mut self) {
        *self.client_states.write().await =
            vec![ClientState::Disconnected; self.config.max_clients];

        for i in 0..self.config.max_clients {
            let (tx, rx) = mpsc::unbounded_channel();
            self.connections.push(tx);

            let mut client = SocketClient::new(i, self.config.clone(), rx, self.handler.clone());
            let states = self.client_states.clone();
            tokio::spawn(async move {
                client.run().await;
            });
        }

        while let Some(cmd) = self.command_rx.recv().await {
            match cmd {
                PoolCommand::JoinChannel { config, response } => {
                    let res = self.join(config).await;
                    _ = response.send(res);
                }
                _ => (),
            }
        }
    }

    async fn join(&self, config: ChannelConfig) -> SocketPoolResult<()> {
        let mut best = 0;
        let mut min_channels = usize::MAX;

        for (i, client) in self.connections.iter().enumerate() {
            if i < min_channels {
                min_channels = i;
                best = i;
            }
        }

        if let Some(conn) = self.connections.get(best) {
            conn.send(ClientCommand::JoinChannel(config))?;
            Ok(())
        } else {
            Err(SocketPoolError::NoClients)
        }
    }
}

#[derive(Debug, Clone)]
pub struct SocketPoolConfig {
    pub irc_url: &'static str,
    pub auth: IrcAuthInfo,
    pub max_joins: usize,
    pub max_clients: usize,
    pub reconnect_delay: std::time::Duration,
    pub ping_interval: std::time::Duration,
}

impl Default for SocketPoolConfig {
    fn default() -> Self {
        Self {
            irc_url: DEFAULT_IRC,
            auth: IrcAuthInfo::default(),
            max_joins: 100,
            max_clients: 3,
            reconnect_delay: Duration::from_secs(5),
            ping_interval: Duration::from_secs(240),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolHealth {
    pub active_clients: usize,
    pub total_channels: usize,
    pub channels_per_client: Vec<usize>,
    pub client_states: Vec<ClientState>,
}

#[derive(Debug, Clone)]
pub enum ClientState {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

#[derive(Debug)]
pub enum ClientCommand {
    JoinChannel(ChannelConfig),
    Part(String),
    SendMessage { channel: String, message: String },
    Disconnect,
}

#[derive(Debug)]
pub enum PoolCommand {
    JoinChannel {
        config: ChannelConfig,
        response: oneshot::Sender<SocketPoolResult<()>>,
    },
    LeaveChannel {
        channel: String,
        response: oneshot::Sender<SocketPoolResult<()>>,
    },
    SendMessage {
        channel: String,
        message: String,
        response: oneshot::Sender<SocketPoolResult<()>>,
    },
    CheckHealth {
        response: oneshot::Sender<PoolHealth>,
    },
}

#[derive(Debug, Clone)]
pub struct IrcAuthInfo {
    pub caps: String,
    pub pass: String,
    pub nick: String,
    pub user: String,
}

impl IrcAuthInfo {
    pub fn new(user_token: &str, user_login: &str, caps: Option<&str>) -> Self {
        let caps = match caps {
            Some(c) => c.to_string(),
            None => DEFAULT_CAPS.to_string(),
        };

        let pass = format!("PASS oauth:{}", user_token);
        let nick = format!("NICK {}", user_login);
        let user = format!("USER {} 8 * :{}", user_login, user_login);

        // info!("IRC authentication: {}, {}, {}, {}", caps, pass, nick, user);

        Self {
            caps,
            pass,
            nick,
            user,
        }
    }
}

impl Default for IrcAuthInfo {
    fn default() -> Self {
        let token = ENV_SECRETS.get().user_token.clone();
        let login = ENV_SECRETS.get().user_login.clone();

        Self::new(&token, &login, None)
    }
}
