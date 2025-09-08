use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tracing::{debug, error, info, warn};

use crate::parsing::commands::IrcCommand as ParsedCommand;
use crate::parsing::parser::{IrcAst, IrcParser, Parser};
use crate::socket::core::{IrcAuthentication, IrcChannel, IrcCommand, IrcError, IrcEvent};
use crate::socket::pool::DEFAULT_IRC;

pub type IrcResult<T> = core::result::Result<T, IrcError>;

#[derive(Debug, Clone)]
pub struct IrcClientConfig {
    pub irc_url: &'static str,
    pub auth: IrcAuthentication,
    pub max_joins: usize,
    pub max_clients: usize,
    pub reconnect_delay: std::time::Duration,
    pub ping_interval: std::time::Duration,
    pub timeout: std::time::Duration,
}

impl Default for IrcClientConfig {
    fn default() -> Self {
        Self {
            irc_url: DEFAULT_IRC,
            auth: IrcAuthentication::new(None),
            max_joins: Default::default(),
            max_clients: Default::default(),
            reconnect_delay: Duration::from_secs(10),
            ping_interval: Duration::from_secs(300),
            timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Debug)]
pub struct IrcClient {
    pub config: IrcClientConfig,
    pub parser: Arc<dyn Parser>,
    pub channels: Arc<Mutex<HashMap<String, IrcChannel>>>,
    pub joined_count: usize,
    pub event_tx: mpsc::UnboundedSender<IrcEvent>,
    pub command_tx: mpsc::UnboundedSender<IrcCommand>,
    pub connected: Arc<Mutex<bool>>,
}

impl IrcClient {
    pub fn new(config: IrcClientConfig) -> (Self, mpsc::UnboundedReceiver<IrcEvent>) {
        Self::new_with_parser(config, Arc::new(IrcParser::new()))
    }

    pub fn new_with_parser(
        config: IrcClientConfig,
        parser: Arc<dyn Parser>,
    ) -> (Self, mpsc::UnboundedReceiver<IrcEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (command_tx, _) = mpsc::unbounded_channel();

        let client = Self {
            config,
            parser,
            channels: Arc::new(Mutex::new(HashMap::new())),
            joined_count: 0,
            event_tx,
            command_tx,
            connected: Arc::new(Mutex::new(false)),
        };

        (client, event_rx)
    }

    pub async fn connect(&mut self) -> IrcResult<()> {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        self.command_tx = command_tx;

        let config = self.config.clone();
        let parser = self.parser.clone();
        let channels = self.channels.clone();
        let event_tx = self.event_tx.clone();
        let connected = self.connected.clone();

        tokio::spawn(async move {
            Self::main_loop(config, parser, channels, event_tx, connected, command_rx).await;
        });

        Ok(())
    }

    async fn main_loop(
        config: IrcClientConfig,
        parser: Arc<dyn Parser>,
        channels: Arc<Mutex<HashMap<String, IrcChannel>>>,
        event_tx: mpsc::UnboundedSender<IrcEvent>,
        connected: Arc<Mutex<bool>>,
        mut command_rx: mpsc::UnboundedReceiver<IrcCommand>,
    ) {
        loop {
            match Self::establish(&config).await {
                Ok(ws_stream) => {
                    info!("connected to irc server '{}'", config.irc_url);
                    *connected.lock().await = true;
                    _ = event_tx.send(IrcEvent::Connected);

                    if let Err(e) = Self::handler(
                        ws_stream,
                        &config,
                        &parser,
                        &channels,
                        &event_tx,
                        &mut command_rx,
                    )
                    .await
                    {
                        error!("connection handler error: {:?}", e);
                        _ = event_tx.send(IrcEvent::Error(e));
                    }
                }
                Err(e) => {
                    error!("failed to connect: {:?}", e);
                    _ = event_tx.send(IrcEvent::Error(e));
                }
            }

            info!("reconnecting in {:?}..", config.reconnect_delay);
            sleep(config.reconnect_delay).await;
        }
    }

    async fn establish(
        config: &IrcClientConfig,
    ) -> IrcResult<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let (ws_stream, _) = connect_async(config.irc_url)
            .await
            .map_err(|e| IrcError::ConnectionFailed(e.to_string()))?;

        Ok(ws_stream)
    }

    async fn handler(
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        config: &IrcClientConfig,
        parser: &Arc<dyn Parser>,
        channels: &Arc<Mutex<HashMap<String, IrcChannel>>>,
        event_tx: &mpsc::UnboundedSender<IrcEvent>,
        command_rx: &mut mpsc::UnboundedReceiver<IrcCommand>,
    ) -> IrcResult<()> {
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        Self::send_handshake(&mut ws_sender, config).await?;

        let _ping_tx = event_tx.clone();
        let ping_interval = config.ping_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(ping_interval);
            loop {
                interval.tick().await;
                debug!("ping interval elapsed...");
            }
        });

        loop {
            tokio::select! {
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            Self::handle_raw(&text, parser, channels, event_tx).await;
                        }
                        Some(Ok(Message::Close(_))) => {
                            warn!("socket connection closed");
                            break;
                        }
                        Some(Err(e)) => {
                            return Err(IrcError::WebsocketClientError(e.to_string()));
                        }
                        None => {
                            info!("websocket stream ended");
                            break;
                        }
                        _ => {}
                    }
                }

                cmd = command_rx.recv() => {
                    match cmd {
                        Some(cmd) => {
                            if let Err(e) = Self::handle_command(cmd, &mut ws_sender, channels).await {
                                error!("command handler failure: {}", e);
                            }
                        }
                        None => {
                            info!("command channel closed");
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn send_handshake(
        ws_sender: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        config: &IrcClientConfig,
    ) -> IrcResult<()> {
        let auth_commands = [
            &config.auth.caps,
            &config.auth.pass,
            &config.auth.nick,
            &config.auth.user,
        ];
        for cmd in auth_commands {
            debug!("sending auth frame: {cmd}");

            ws_sender
                .send(Message::Text(cmd.into()))
                .await
                .map_err(|e| IrcError::WebsocketClientError(e.to_string()))?;
        }

        Ok(())
    }

    async fn handle_raw(
        raw_message: &str,
        parser: &Arc<dyn Parser>,
        channels: &Arc<Mutex<HashMap<String, IrcChannel>>>,
        event_tx: &mpsc::UnboundedSender<IrcEvent>,
    ) {
        // debug!("received raw msg: {}", raw_message);
        match parser.parse(raw_message) {
            Ok(ast) => {
                _ = event_tx.send(IrcEvent::RawMsg(ast.clone()));
                Self::handle_parsed_command(&ast, channels, event_tx).await;
            }
            Err(e) => {
                warn!("failed to parse message '{}': {}", raw_message, e);
                _ = event_tx.send(IrcEvent::Error(IrcError::ParseError(e)));
            }
        }
    }

    async fn handle_parsed_command(
        ast: &IrcAst,
        channels: &Arc<Mutex<HashMap<String, IrcChannel>>>,
        event_tx: &mpsc::UnboundedSender<IrcEvent>,
    ) {
        match &ast.command {
            ParsedCommand::PrivMsg {
                channel,
                message,
                user_info,
            } => {
                let user_id = ast
                    .source
                    .as_ref()
                    .map(|s| s.nick.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                _ = event_tx.send(IrcEvent::PrivMsgRx {
                    channel: channel.clone(),
                    user_id,
                    message: message.clone(),
                    user_info: user_info.clone(),
                });
            }

            ParsedCommand::Notice { target, message } => {
                _ = event_tx.send(IrcEvent::NoticeRx {
                    target: target.clone(),
                    message: message.clone(),
                })
            }

            ParsedCommand::Ping { server } => {
                _ = event_tx.send(IrcEvent::PingRx(server.clone()));
                // TODO: respond with PONG
            }

            ParsedCommand::Pong { server } => {
                _ = event_tx.send(IrcEvent::PongRx(server.clone()));
            }

            ParsedCommand::UserNotice {
                channel,
                message,
                msg_id,
                user_info,
            } => {
                _ = event_tx.send(IrcEvent::UserNoticeRx {
                    channel: channel.clone(),
                    message: message.clone(),
                    msg_id: msg_id.clone(),
                    user_info: user_info.clone(),
                })
            }

            ParsedCommand::UserState {
                channel,
                message,
                msg_id,
                user_info,
            } => {
                _ = event_tx.send(IrcEvent::UserStateRx {
                    channel: channel.clone(),
                    message: message.clone(),
                    msg_id: msg_id.clone(),
                    user_info: user_info.clone(),
                })
            }

            ParsedCommand::ClearChat {
                channel,
                target_user,
                duration,
            } => {
                _ = event_tx.send(IrcEvent::ClearChat {
                    channel: channel.clone(),
                    target_user: target_user.clone(),
                    duration: *duration,
                })
            }

            ParsedCommand::ClearMsg {
                channel,
                target_msg_id,
            } => {
                _ = event_tx.send(IrcEvent::ClearMsg {
                    channel: channel.clone(),
                    target_msg_id: target_msg_id.clone(),
                })
            }

            ParsedCommand::Numeric { code, params } => {
                _ = event_tx.send(IrcEvent::Numeric {
                    code: *code,
                    params: params.clone(),
                })
            }

            ParsedCommand::Unknown { command, params } => {
                _ = event_tx.send(IrcEvent::Unknown {
                    command: command.clone(),
                    params: params.clone(),
                })
            }
        }
    }

    async fn handle_command(
        cmd: IrcCommand,
        ws_sender: &mut futures_util::stream::SplitSink<
            WebSocketStream<MaybeTlsStream<TcpStream>>,
            Message,
        >,
        channels: &Arc<Mutex<HashMap<String, IrcChannel>>>,
    ) -> IrcResult<()> {
        match cmd {
            IrcCommand::JoinChannel(channel, response) => {
                let guard = channels.lock().await;
                if guard.len() >= 100 {
                    _ = response.send(Err(IrcError::ChannelLimitReached));
                    return Ok(());
                }

                drop(guard);
                let join_msg = format!("JOIN #{}", channel);

                // these could probably be broken out into a sender function but
                // iajsdkjfhkask;fhj
                match ws_sender.send(Message::Text(join_msg.into())).await {
                    Ok(_) => {
                        _ = response.send(Ok(()));
                    }
                    Err(e) => {
                        _ = response.send(Err(IrcError::WebsocketClientError(e.to_string())));
                    }
                }
            }
            IrcCommand::LeaveChannel(channel, response) => {
                let part_msg = format!("PART #{}", channel);
                match ws_sender.send(Message::Text(part_msg.into())).await {
                    Ok(_) => {
                        _ = response.send(Ok(()));
                    }
                    Err(e) => {
                        _ = response.send(Err(IrcError::WebsocketClientError(e.to_string())));
                    }
                }
            }
            IrcCommand::SendMessage(channel, message, response) => {
                let privmsg = format!("PRIVMSG #{} :{}", channel, message);
                match ws_sender.send(Message::Text(privmsg.into())).await {
                    Ok(_) => {
                        _ = response.send(Ok(()));
                    }
                    Err(e) => {
                        _ = response.send(Err(IrcError::WebsocketClientError(e.to_string())));
                    }
                }
            }
            IrcCommand::GetChannels(response) => {
                let guard = channels.lock().await;
                let channel_names: Vec<String> = guard.keys().cloned().collect();
                _ = response.send(channel_names);
            }

            // IrcCommand::Disconnect(sender) => {}
            _ => {}
        }

        Ok(())
    }

    pub async fn join_channel(&self, channel: &str) -> IrcResult<()> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(IrcCommand::JoinChannel(channel.to_string(), tx))
            .map_err(|_| IrcError::ConnectionFailed("command channel closed".to_string()))?;

        tokio::time::timeout(self.config.timeout, rx)
            .await
            .map_err(|_| IrcError::Timeout)?
            .map_err(|_| IrcError::ConnectionFailed("response channel closed".to_string()))?
    }

    pub async fn leave_channel(&self, channel: &str) -> IrcResult<()> {
        let (tx, rx) = oneshot::channel();

        self.command_tx
            .send(IrcCommand::LeaveChannel(channel.to_string(), tx))
            .map_err(|_| IrcError::ConnectionFailed("command channel closed".to_string()))?;

        tokio::time::timeout(self.config.timeout, rx)
            .await
            .map_err(|_| IrcError::Timeout)?
            .map_err(|_| IrcError::ConnectionFailed("response channel closed".to_string()))?
    }

    pub async fn send_message(&self, channel: &str, message: &str) -> IrcResult<()> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(IrcCommand::SendMessage(
                channel.to_string(),
                message.to_string(),
                tx,
            ))
            .map_err(|_| IrcError::ConnectionFailed("command channel closed".to_string()))?;

        tokio::time::timeout(self.config.timeout, rx)
            .await
            .map_err(|_| IrcError::Timeout)?
            .map_err(|_| IrcError::ConnectionFailed("response channel closed".to_string()))?
    }

    pub async fn get_joined_channels(&self) -> Vec<String> {
        let (tx, rx) = oneshot::channel();

        if self.command_tx.send(IrcCommand::GetChannels(tx)).is_ok() {
            rx.await.unwrap_or_default()
        } else {
            vec![]
        }
    }

    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }
}
