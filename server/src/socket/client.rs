use core::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::{SplitSink, SplitStream};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub type WsClientResult<T> = std::result::Result<T, SocketClientError>;
pub type SocketWriter = Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>;
pub type SocketReader = Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>;

#[derive(Debug, Error)]
pub enum SocketClientError {}

// #[async_trait]
// pub trait Manager: fmt::Debug {
//     async fn connect(&self, conn: &SocketConnection) -> WsClientResult<Box<dyn Client>>;
// }

#[async_trait]
pub trait EventHandler: Send + Sync + fmt::Debug {
    async fn handle_event(&self, event: SocketEvent) -> WsClientResult<()>;
}

#[async_trait]
pub trait RoomClient: Send + Sync + fmt::Debug {
    async fn send(&mut self, message: &str) -> WsClientResult<()>;
    async fn receive(&mut self) -> WsClientResult<Option<String>>;
    async fn close(&mut self) -> WsClientResult<()>;
    fn is_connected(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct SocketClient {
    joined: Vec<String>,
}

/// Event types for the websocket IRC client
#[derive(Debug, Clone)]
pub enum SocketEvent {
    // url/user probably to be provided on the socket
    // client itself as a field on the enum option
    Connected {
        client: SocketClient,
    },
    Authenticated {
        client: SocketClient,
    },
    Disconnected {
        // TODO: enum that provides predefined
        //      'reasons' for client disconnection
        reason: String,
        client: SocketClient,
    },
    Joined {
        channel: String,
        client: SocketClient,
    },
    Parted {
        channel: String,
        reason: String,
        client: SocketClient,
    },
    Message {
        command: IrcCommand,
        client: SocketClient,
    },
    Error {
        error: String,
        detail: Option<String>,
        client: SocketClient,
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
