use std::{collections::HashMap, sync::Arc};

use futures::SinkExt;
use futures::StreamExt;
use thiserror::Error;
use tokio::sync::{RwLock, mpsc};
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;
use tracing::info;

use crate::parsing::parser;
use crate::parsing::parser::IrcMessage;
use crate::parsing::parser::Parser;
use crate::socket::{
    core::{EventHandler, MessageHandler},
    pool::{ClientCommand, ClientState, SocketPoolConfig},
};

pub type SocketClientResult<T> = core::result::Result<T, SocketClientError>;

#[derive(Error, Debug)]
pub enum SocketClientError {
    #[error("tungstenite error: {0}")]
    TungsteniteError(#[from] tungstenite::Error),
}

#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub name: String,
    pub handler: Arc<dyn MessageHandler>,
    pub auto_rejoin: bool,
    pub rate_limit: bool,
    pub rate_limit_interval: u32,
}

pub struct SocketClient {
    id: usize,
    config: SocketPoolConfig,
    channels: Arc<RwLock<HashMap<String, ChannelConfig>>>,
    command_rx: mpsc::UnboundedReceiver<ClientCommand>,
    state: Arc<RwLock<ClientState>>,
    handler: Arc<dyn EventHandler>,
}

impl SocketClient {
    pub fn new(
        id: usize,
        config: SocketPoolConfig,
        command_rx: mpsc::UnboundedReceiver<ClientCommand>,
        handler: Arc<dyn EventHandler>,
    ) -> Self {
        Self {
            id,
            config,
            channels: Arc::new(RwLock::new(HashMap::new())),
            command_rx,
            state: Arc::new(RwLock::new(ClientState::Disconnected)),
            handler,
        }
    }

    pub async fn run(&mut self) {
        loop {
            *self.state.write().await = ClientState::Connecting;
            match self.connect().await {
                Ok(_) => {
                    info!("terminated gracefully: {}", self.id);
                    break;
                }
                Err(e) => {
                    info!("connection error: {}: {}", self.id, e);

                    *self.state.write().await = ClientState::Error(e.to_string());
                    self.handler.on_error(self.id, &e.to_string()).await;

                    sleep(self.config.reconnect_delay).await;
                }
            }
            todo!()
        }
    }

    pub async fn connect(&mut self) -> SocketClientResult<()> {
        let url = self.config.irc_url;

        let (stream, _) = connect_async(url).await?;
        let (mut writer, mut reader) = stream.split();

        *self.state.write().await = ClientState::Connected;
        self.handler.on_connect(self.id).await;

        let auth = &self.config.auth;

        writer.send(Message::text(format!("{}", auth.caps))).await?;
        writer.send(Message::text(format!("{}", auth.pass))).await?;
        writer.send(Message::text(format!("{}", auth.nick))).await?;
        writer.send(Message::text(format!("{}", auth.user))).await?;

        let prev_channels = self.channels.read().await;
        if !prev_channels.is_empty() {
            for con in prev_channels.values() {
                writer
                    .send(Message::text(format!("JOIN #{}", con.name)))
                    .await?;
                con.handler.on_join(&con.name).await;
            }
        }

        loop {
            tokio::select! {
                msg = reader.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            self.handle_socket_msg(&text).await;
                        },
                        Some(Ok(Message::Ping(data))) => {
                            writer.send(Message::Pong(data)).await?;
                        }
                        _ => {}
                    }
                }

                cmd = self.command_rx.recv() => {
                    match cmd {
                        Some(ClientCommand::JoinChannel(config)) => {
                            writer.send(Message::text(format!("JOIN #{}", config.name))).await?;
                            config.handler.on_join(&config.name).await;
                            self.channels.write().await.insert(config.name.clone(), config);
                        }
                        _ => {}
                    }
                }
            }

            todo!()
        }
    }

    async fn handle_socket_msg(&self, data: &str) {
        let parser = parser::IrcParser::new();
        info!("raw msg data: {:#?}", data);
        if let Ok(message) = parser.parse(data) {
            info!("parsed message: {:#?}", message);
            match message.command {
                "PRIVMSG" => {
                    if let Ok(chat_msg) = parser.extract_chat_data(&message) {
                        info!("channel: {:?}", chat_msg.channel);
                        info!("chatter: {:?}", chat_msg.user_login);
                        info!("chatter id: {:?}", chat_msg.user_id);
                        info!("chat msg: {:?}", chat_msg.message);
                    }
                }
                _ => {}
            }
        }
    }

    pub async fn get_state(&self) -> ClientState {
        self.state.read().await.clone()
    }
}

/// Event types for the websocket IRC client
#[derive(Debug, Clone)]
pub enum SocketEvent {
    Connected {
        as_user: String,
        client_id: usize,
    },
    Authenticated {
        as_user: String,
        client_id: usize,
    },
    Disconnected {
        // TODO: enum that provides predefined
        //      'reasons' for client disconnection (maybe)
        reason: String,
        channel: String,
        as_user: String,
        client_id: usize,
    },
    Joined {
        channel: String,
        as_user: String,
        client_id: usize,
    },
    Parted {
        reason: String,
        channel: String,
        as_user: String,
        client_id: usize,
    },
    Message {
        command: IrcCommand,
        as_user: String,
        client_id: usize,
    },
    Error {
        detail: Option<String>,
        channel: Option<String>,
        error: String,
        as_user: String,
        client_id: usize,
    },
}
/// IRC Command Reference enum
///
/// # IRC Reference
///
/// Twitch documents its chatroom IRC on its [IRC Reference page].
/// [Reference page]: https://dev.twitch.tv/docs/chat/irc/
#[derive(Debug, Clone)]
pub enum IrcCommand {
    PrivMsg {
        channel: String,
        chatter_login: String,
        chatter_id: String,
        message: String,
    },
    UserNotice {
        chatter_login: String,
        chatter_id: String,
        message: String,
        channel: String,
    },
    Notice {
        /// Sent to indicate the outcome of an action (e.g banning a user).
        ///
        /// # Tags
        ///
        /// If we have the tags capability, a `NOTICE` message
        /// includes a ['msg-id' tag] to describe what the notice represents.
        ///
        /// ['msg-id' tag]: https://dev.twitch.tv/docs/chat/irc/#notice-reference
        message: String,
        msg_id: String,
        channel: String,
    },
    Ping {
        channel: String,
    },
    /// Generic (i.e, server-based) - sent when the IRC server needs to terminate
    /// a connection for maintenance reasons
    ///
    /// Intended to provide a chance for our bot to perform minimal cleanup & state
    /// management before the server terminates the connection.
    Reconnect,
    Other {
        command: String,
        channel: String,
    },
}
