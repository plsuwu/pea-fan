use crate::parser::{IrcMessage, IrcParser, Parser, ParserError};
use crate::ws::connection::{Connection, WsConnection};
use async_trait::async_trait;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::fmt;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, warn};

#[derive(Debug, Error)]
pub enum WsClientError {
    #[error("Websocket connection error: {0}")]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Redis client error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Authentication failure: {0}")]
    Authentication(String),

    #[error("Parser error: {0}")]
    Parser(#[from] ParserError),

    #[error("Channel error: {0}")]
    Channel(String),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Timeout: {0}")]
    Timeout(String),
}

pub type WsClientResult<T> = std::result::Result<T, WsClientError>;
pub type SocketWriter = Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>;
pub type SocketReader = Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>;

#[derive(Debug, Clone)]
pub enum WsEvent {
    Connected,
    Disconnected {
        channel: String,
        reason: String,
    },
    Authenticated,
    Joined {
        channel: String,
    },
    ChatMessage {
        channel: String,
        user_login: String,
        user_id: String,
        color: Option<String>,
        message: String,
    },
    Ping,
    Error {
        error: String,
    },
    Unknown {
        command: String,
        raw: String,
    },
}

#[async_trait]
pub trait EventHandler: Send + Sync + fmt::Debug {
    async fn handle_event(&self, event: WsEvent) -> WsClientResult<()>;
}

#[async_trait]
pub trait Manager: fmt::Debug {
    async fn connect(&self, conn: &WsConnection) -> WsClientResult<Box<dyn Client>>;
}

#[async_trait]
pub trait Client: Send + Sync + fmt::Debug {
    async fn send(&mut self, message: &str) -> WsClientResult<()>;
    async fn receive(&mut self) -> WsClientResult<Option<String>>;
    async fn close(&mut self) -> WsClientResult<()>;
    fn is_connected(&self) -> bool;
}

#[async_trait]
pub trait CacheCounter: Send + Sync + fmt::Debug {
    async fn increment_counter(&self, channel: &str, user: &str) -> WsClientResult<()>;
}

#[derive(Debug)]
pub struct WsClient {
    writer: SocketWriter,
    reader: SocketReader,
    connected: Arc<Mutex<bool>>,
}

#[async_trait]
impl Client for WsClient {
    #[instrument(skip(self))]
    async fn send(&mut self, message: &str) -> WsClientResult<()> {
        let msg = Message::text(message);
        self.writer
            .lock()
            .await
            .send(msg)
            .await
            .map_err(WsClientError::Websocket)?;

        if !message.contains("PASS oauth:") {
            debug!("Sent: {}", message);
        } else {
            debug!("Sent: [AUTHENTICATION MESSAGE]");
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn receive(&mut self) -> WsClientResult<Option<String>> {
        let mut reader = self.reader.lock().await;
        match reader.next().await {
            Some(Ok(message)) => {
                if let Ok(text) = message.to_text() {
                    debug!("Received: {}", text);
                    Ok(Some(text.to_string()))
                } else {
                    warn!("Received non-text message: {:?}", message);
                    Ok(None)
                }
            }
            Some(Err(e)) => {
                error!("Websocket error: {:?}", e);
                *self.connected.lock().await = false;
                Err(WsClientError::ConnectionClosed)
            }
            None => {
                info!("Websocket connection closed");
                *self.connected.lock().await = false;
                Err(WsClientError::ConnectionClosed)
            }
        }
    }

    async fn close(&mut self) -> WsClientResult<()> {
        *self.connected.lock().await = false;
        self.writer
            .lock()
            .await
            .close()
            .await
            .map_err(WsClientError::Websocket)
    }

    fn is_connected(&self) -> bool {
        futures::executor::block_on(self.connected.lock()).clone()
    }
}

#[derive(Debug)]
pub struct WsManager;

#[async_trait]
impl Manager for WsManager {
    #[instrument(skip(self, conn))]
    async fn connect(&self, conn: &WsConnection) -> WsClientResult<Box<dyn Client>> {
        let url = conn.url();
        info!("Connecting to {}", &url);

        let (stream, _) = connect_async(url).await.map_err(WsClientError::Websocket)?;
        let (w, r) = stream.split();

        Ok(Box::new(WsClient {
            writer: Arc::new(Mutex::new(w)),
            reader: Arc::new(Mutex::new(r)),
            connected: Arc::new(Mutex::new(true)),
        }))
    }
}

#[derive(Debug)]
pub struct WsEventHandler<T>
where
    T: Connection,
{
    connection: T,
    data_store: Arc<dyn CacheCounter>,
}

impl<T> WsEventHandler<T>
where
    T: Connection,
{
    pub fn new(connection: T, data_store: Arc<dyn CacheCounter>) -> Self {
        Self {
            connection,
            data_store,
        }
    }
}

#[async_trait]
impl<T> EventHandler for WsEventHandler<T>
where
    T: Connection + Send + Sync,
{
    async fn handle_event(&self, event: WsEvent) -> WsClientResult<()> {
        match event {
            WsEvent::Connected => {
                info!("Connected to IRC");
            }
            WsEvent::ChatMessage {
                channel,
                user_login,
                user_id,
                color,
                message,
            } => {
                if message.to_lowercase().contains(&self.connection.needle()) {
                    info!(
                        channel = %channel,
                        user = %user_login,
                        user_id = %user_id,
                        color = ?color,
                        message_len = message.len(),
                        "found target string"
                    );

                    self.data_store
                        .increment_counter(&channel, &user_login)
                        .await?;
                }
            }
            WsEvent::Joined { channel } => {
                info!("Joined channel '{}'", channel);
            }
            WsEvent::Ping => {
                debug!("Received PING");
            }
            WsEvent::Error { error } => {
                error!("IRC Error: {}", error);
            }
            WsEvent::Disconnected { channel, reason } => {
                warn!("Disconnected from '{}': {}", channel, reason);
            }
            WsEvent::Unknown { command, raw } => {
                debug!("Unknown IRC command '{}': {}", command, raw);
            }
            WsEvent::Authenticated => {
                info!("Authentication OK");
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct IrcClient {
    pub connection: WsConnection,
    pub manager: Arc<dyn Manager>,
    pub parser: Arc<dyn Parser>,
    pub handler: Arc<dyn EventHandler>,
    pub event_tx: mpsc::UnboundedSender<WsEvent>,
    pub event_rx: Option<mpsc::UnboundedReceiver<WsEvent>>,
}

const IRC_CAPABILITIES_IDX: usize = 0;
const IRC_OAUTH_IDX: usize = 1;
const IRC_NICK_IDX: usize = 2;
const IRC_LOGIN_IDX: usize = 3;
const IRC_CHANNEL_IDX: usize = 4;

impl IrcClient {
    pub fn new(
        connection: WsConnection,
        manager: Arc<dyn Manager>,
        parser: Arc<dyn Parser>,
        handler: Arc<dyn EventHandler>,
    ) -> WsClientResult<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Ok(Self {
            connection,
            manager,
            parser,
            handler,
            event_tx,
            event_rx: Some(event_rx),
        })
    }

    async fn emit_event(&self, event: WsEvent) {
        if let Err(_) = self.event_tx.send(event) {
            error!("Failed to send event, receiver dropped");
        }
    }

    pub async fn authenticate(&self, connection: &mut Box<dyn Client>) -> WsClientResult<()> {
        for cmd in self.connection.auth_commands() {
            connection.send(cmd).await?;
        }

        Ok(())
    }

    async fn respond_ping(&self, client: &mut Box<dyn Client>) -> WsClientResult<()> {
        client.send("PONG :tmi.twitch.tv").await?;
        self.emit_event(WsEvent::Ping).await;

        Ok(())
    }

    async fn respond_join(&self, parsed: &IrcMessage<'_>) {
        if let Ok(channel) = self.parser.extract_channel(&parsed) {
            self.emit_event(WsEvent::Joined {
                channel: channel.to_string(),
            })
            .await;
        }
    }

    async fn respond_privmsg(&self, parsed: &IrcMessage<'_>) {
        println!("{:?}", parsed);

        match self.parser.extract_chat_data(&parsed) {
            Ok(data) => {
                self.emit_event(WsEvent::ChatMessage {
                    channel: data.channel.to_string(),
                    user_login: data.user_login.to_string(),
                    user_id: data.user_id.to_string(),
                    color: data.color.map(|c| c.to_string()),
                    message: data.message.to_string(),
                })
                .await;
            }

            Err(e) => {
                warn!("Failed to extract chat data: {:?}", e);
                self.emit_event(WsEvent::Error {
                    error: format!("Chat parsing error: {}", e),
                })
                .await;
            }
        }
    }

    async fn respond_unhandled(&self, parsed: &IrcMessage<'_>, raw_message: &str) {
        debug!("Unhandled IRC command: {}", parsed.command);
        self.emit_event(WsEvent::Unknown {
            command: parsed.command.to_string(),
            raw: raw_message.to_string(),
        })
        .await;
    }

    async fn process_message(
        &self,
        client: &mut Box<dyn Client>,
        raw_message: &str,
    ) -> WsClientResult<()> {
        let parsed = self.parser.parse(raw_message)?;

        match parsed.command {
            "PING" => self.respond_ping(client).await?,
            "JOIN" => self.respond_join(&parsed).await,
            "PRIVMSG" => self.respond_privmsg(&parsed).await,
            _ => self.respond_unhandled(&parsed, raw_message).await,
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn run(&mut self, cancel_token: CancellationToken) -> WsClientResult<()> {
        let mut conn = self.manager.connect(&self.connection).await?;
        let mut event_rx = self.event_rx.take().unwrap();

        self.authenticate(&mut conn).await?;

        conn.send(&format!("JOIN #{}", self.connection.channel()))
            .await?;

        self.emit_event(WsEvent::Connected).await;
        loop {
            tokio::select! {
                message_result = conn.receive() => {
                    match message_result {
                        Ok(Some(raw_msg)) => {
                            if let Err(e) = self.process_message(&mut conn, &raw_msg).await {
                                error!("Error while processing message: {:?}", e);
                                self.emit_event(WsEvent::Error { error: e.to_string() }).await;
                            }
                        }

                        Ok(None) => continue,
                        Err(e) => {
                            error!("Connection error: {:?}", e);
                            self.emit_event(WsEvent::Disconnected {
                                reason: e.to_string(),
                                channel: self.connection.channel().to_string(),
                            }).await;
                            break;
                        }
                    }
                }

                Some(event) = event_rx.recv() => {
                    if let Err(e) = self.handler.handle_event(event).await {
                        error!("Error while handling event: {:?}", e);
                    }
                }

                _ = cancel_token.cancelled() => {
                    info!("Client shutdown requested");
                    _ = conn.send(&format!("PART #{}", self.connection.channel())).await;
                    _ = conn.close().await;
                    break;
                }
            }
        }

        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct WsClientBuilder {
    connection: Option<WsConnection>,
    manager: Option<Arc<dyn Manager>>,
    parser: Option<Arc<dyn Parser>>,
    handler: Option<Arc<dyn EventHandler>>,
}

impl WsClientBuilder {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    pub fn with_connection(mut self, connection: WsConnection) -> Self {
        self.connection = Some(connection);
        self
    }

    pub fn with_manager(mut self, manager: Arc<dyn Manager>) -> Self {
        self.manager = Some(manager);
        self
    }

    pub fn with_parser(mut self, parser: Arc<dyn Parser>) -> Self {
        self.parser = Some(parser);
        self
    }

    pub fn with_handler(mut self, handler: Arc<dyn EventHandler>) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn build(self) -> WsClientResult<IrcClient> {
        let connection = self.connection.ok_or_else(|| {
            WsClientError::Authentication("Connection configuration required".into())
        })?;
        let event_handler = self.handler.ok_or_else(|| {
            WsClientError::Authentication("Event handler configuration required".into())
        })?;
        let manager = self.manager.unwrap_or_else(|| Arc::new(WsManager));
        let parser = self.parser.unwrap_or_else(|| Arc::new(IrcParser));

        IrcClient::new(connection, manager, parser, event_handler)
    }
}

#[cfg(test)]
mod tests {
    use crate::ws::client::*;
    use crate::ws::connection::*;
    use crate::ws::tests;
    use crate::ws::tests::MockRedisLayer;
    use std::future::IntoFuture;
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    struct MockClient {
        connection_config: WsConnection,
    }

    impl MockClient {
        pub async fn new(endpoint: &str) -> Self {
            let (listener, addr) = tests::listener().await;
            tokio::spawn(axum::serve(listener, tests::router()).into_future());
            let url = format!("ws://{}{}", addr, endpoint);

            let connection_config =
                WsConnection::new(&url, "hello", "test_user_token", "testuser", "testchannel");
            Self { connection_config }
        }

        async fn build_base_client(&self) -> WsClientResult<IrcClient> {
            let store = Arc::new(MockRedisLayer::new("redis://127.0.0.1:6380").await.unwrap());
            let handler = Arc::new(WsEventHandler::new(self.connection_config.clone(), store));

            let client = WsClientBuilder::new()
                .with_connection(self.connection_config.clone())
                .with_handler(handler)
                .build();

            client
        }
    }

    async fn get_connected_socket(endpoint: &str) -> WsClientResult<Box<dyn Client>> {
        let config = MockClient::new(endpoint).await;
        let client = config.build_base_client().await?;
        let connection = client.manager.connect(&config.connection_config).await?;

        Ok(connection)
    }

    #[tokio::test]
    async fn test_send_recv_close() {
        let mut socket = get_connected_socket("/test-client-send")
            .await
            .expect("failed to build and connect to the socket");
        socket
            .send("hello 123")
            .await
            .expect("failed while sending message to socket");

        let result = socket
            .receive()
            .await
            .expect("failed while reading rx result");

        socket
            .close()
            .await
            .expect("failed while closing the socket");
        assert_eq!(result, Some("hello 123".to_string()));
        assert_eq!(socket.is_connected(), false);
    }

    #[tokio::test]
    async fn test_send_recv_wait_close() {
        let mut socket = get_connected_socket("/test-client-send")
            .await
            .expect("failed to build and connect to the socket");

        socket
            .send("hello 123")
            .await
            .expect("failed while sending message to socket");

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        assert_eq!(socket.is_connected(), true);

        let result = socket
            .receive()
            .await
            .expect("failed while reading rx result");
        assert_eq!(result, Some("hello 123".to_string()));

        socket
            .close()
            .await
            .expect("failed while closing the socket");
        assert_eq!(socket.is_connected(), false);
    }
}
