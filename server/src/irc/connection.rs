#![allow(dead_code)]

use std::time::Duration;
use std::time::Instant;

use futures::StreamExt;
use irc::client::{Client, data};
use tokio::sync::mpsc;
use tokio::sync::watch;
use tracing::instrument;

use crate::irc::IrcQuery;
use crate::irc::channels::ChannelAction;
use crate::irc::channels::ChannelEvent;
use crate::irc::channels::ChannelManager;
use crate::irc::commands::TwitchCapability;
use crate::irc::error::ClientResult;
use crate::irc::error::ConnectionClientError;
use crate::irc::parse::is_counter_user;
use crate::irc::parse::is_pong;
use crate::irc::parse::parse_incoming;
use crate::irc::worker::COUNTER_USER;
use crate::util::env;

use super::commands::{IncomingMessage, OutgoingCommand};

const KEEPALIVE_INTERVAL: u64 = 180;

#[derive(Debug)]
pub struct ConnectionSupervisor {
    channels: Vec<String>,
    reset_rx: mpsc::Receiver<()>,
    generation_tx: watch::Sender<u64>,
    generation: u64,
}

/// Signals that can be used by any task to request or observe a reconnect
#[derive(Clone, Debug)]
pub struct ConnectionHandle {
    /// Triggers a connection reset
    pub reset_tx: mpsc::Sender<()>,
    /// Reflects the current connection generation, updates whenever a connection is reset.

    #[allow(dead_code)]
    pub generation_rx: watch::Receiver<u64>,
}

impl ConnectionSupervisor {
    #[instrument]
    pub fn new(channels: Vec<String>) -> (Self, ConnectionHandle) {
        let (reset_tx, reset_rx) = mpsc::channel(4);
        let (generation_tx, generation_rx) = watch::channel(0u64);

        let handle = ConnectionHandle {
            reset_tx,
            generation_rx,
        };

        let supervisor = Self {
            channels,
            reset_rx,
            generation_tx,
            generation: 0,
        };

        (supervisor, handle)
    }

    /// Main event loop, where each iteration reflects one full connection lifecycle.
    #[instrument(skip(self, msg_tx, cmd_rx, query_rx))]
    pub async fn run(
        &mut self,
        msg_tx: async_channel::Sender<IncomingMessage>,
        mut cmd_rx: mpsc::Receiver<OutgoingCommand>,
        mut query_rx: mpsc::Receiver<IrcQuery>,
    ) {
        loop {
            self.generation += 1;
            _ = self.generation_tx.send(self.generation);

            match self
                .run_single_connection(&msg_tx, &mut cmd_rx, &mut query_rx)
                .await
            {
                Ok(reason) => {
                    tracing::warn!(?reason, gen = self.generation, "connection ended");
                }
                Err(e) => {
                    tracing::error!(error = ?e, gen = self.generation, "connection error");
                }
            }

            // momentary wait prior to attempting reconnection
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    #[instrument(skip(self, msg_tx, cmd_rx, query_rx))]
    async fn run_single_connection(
        &mut self,
        msg_tx: &async_channel::Sender<IncomingMessage>,
        cmd_rx: &mut mpsc::Receiver<OutgoingCommand>,
        query_rx: &mut mpsc::Receiver<IrcQuery>,
    ) -> Result<DisconnectReason, ConnectionClientError> {
        let mut ping_interval = tokio::time::interval(Duration::from_secs(KEEPALIVE_INTERVAL));
        let ack_deadline = Duration::from_secs(15);

        let (event_tx, event_rx) = mpsc::channel(64);
        let (action_tx, mut action_rx) = mpsc::channel(16);

        let channel_mgr = ChannelManager::new(
            self.channels.clone(),
            COUNTER_USER.to_string(),
            event_rx,
            action_tx,
        );

        let mgr_handle = tokio::spawn(channel_mgr.run());
        let mut client = ConnectionClient::init(&self.channels).await?;

        client.connect().await?;

        let mut stream = client.inner.stream()?;
        let mut last_ack = Instant::now();

        loop {
            tokio::select! {
                // Handle PRIVMSG command
                Some(msg_result) = stream.next() => {
                        let msg = msg_result?;

                        // Handle PONG commands first
                        if is_pong(&msg) {
                            tracing::info!(
                                command = ?msg.command,
                                time_since_last_ack = ?last_ack.elapsed(),
                                "keepalive_acknowledged"
                            );
                            last_ack = Instant::now();
                        }

                        // If we aren't handling a PONG, handle JOIN/PART commands for our user,
                        // otherwise send message to a worker thread for further parsing to avoid
                        // blocking the connection thread.
                        match &msg.command {
                            irc::proto::Command::JOIN(channel, _, _) => {
                                // handle JOIN
                                if is_counter_user(&msg, COUNTER_USER) {
                                    _ = event_tx.try_send(ChannelEvent::Joined(channel.clone()));
                                }
                            }

                            irc::proto::Command::PART(channel, _) => {
                                // handle PART
                                if is_counter_user(&msg, COUNTER_USER) {
                                    _ = event_tx.try_send(ChannelEvent::Parted(channel.clone()));
                                }
                            }

                            _ => {
                                // offload to worker
                                if let Some(parsed) = parse_incoming(&msg) {
                                    _ = msg_tx.send(parsed).await;
                                }
                            }
                        }

                    }

                // Send a PING if we haven't seen one recently to make sure we are still connected
                _ = ping_interval.tick() => {
                    if last_ack.elapsed() > ack_deadline + Duration::from_secs(KEEPALIVE_INTERVAL) {
                        tracing::warn!(
                            time_since_last_ack = ?last_ack.elapsed(),
                            "keepalive_timeout",
                        );

                        return Ok(DisconnectReason::KeepaliveTimeout);
                    }

                    client.inner.send("PING :tmi.twitch.tv")?;
                }

                // Handle a query from an API request
                Some(query) = query_rx.recv() => {
                    match query {
                        IrcQuery::GetJoinedChannels { reply } => {
                            tracing::info!("api_query_joined_channels");
                            if let Err(e) = reply.send(client.joined.clone()) {
                                tracing::error!(data = ?e, "api_query_response_fail");
                            }
                        }

                        IrcQuery::InsertNewChannel { channel, reply } => {
                            tracing::info!("api_insert_new_channel");
                            if let Err(e) = reply.send(client.insert_channel(&channel).await?) {
                                tracing::error!(data = ?e, "failed while inserting and joining new channel");
                            }
                        }
                    }
                }

                // Worker action (internal)
                Some(action) = action_rx.recv() => {
                    match action {
                        ChannelAction::Join(channels) => {
                            let join_str = channels.join(",");

                            tracing::info!(%join_str, "executing JOIN");
                            client.inner.send_join(&join_str)?;
                        }
                    }
                }

                // Worker command (external)
                Some(command) = cmd_rx.recv() => {
                    match command {
                        OutgoingCommand::Reply { message } => {
                            if let Err(e) = client.inner.send(message) {
                                tracing::error!(error = ?e, "send failed");
                            }
                        }
                    }
                }

                // Handle a reset request
                Some(()) = self.reset_rx.recv() => {
                    _ = event_tx.send(ChannelEvent::Disconnected).await;
                    mgr_handle.abort();

                    return Ok(DisconnectReason::ResetRequested);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ConnectionClient {
    pub inner: irc::client::Client,
    #[allow(dead_code)]
    pub channels: Vec<String>,
    pub joined: Vec<String>,
}

impl ConnectionClient {
    #[instrument]
    pub async fn init(channels: &Vec<String>) -> ClientResult<Self> {
        let channels: Vec<String> = channels.iter().map(|chan| format!("#{chan}")).collect();
        tracing::trace!(?channels, "reformatted channel list");

        let config = data::Config {
            use_tls: Some(true),
            nickname: Some(crate::var!(env::Var::UserLogin).await?.to_string()),
            password: Some(format!("oauth:{}", crate::var!(env::Var::UserToken).await?)),

            server: Some(TTV_IRC_URI.to_string()),
            port: Some(TTV_IRC_PORT),
            ping_time: Some(280),
            ..data::Config::default()
        };

        let connection = Client::from_config(config.clone()).await.unwrap();

        Ok(Self {
            channels,
            joined: Vec::new(),
            inner: connection,
        })
    }

    pub async fn insert_channel(&mut self, channel: &str) -> ClientResult<String> {
        let channel = format!("#{channel}");
        self.channels.push(channel.clone());

        tracing::info!(channel, "joining new channel");
        self.join_channel(&channel).await?;

        Ok(channel)
    }

    #[instrument(skip(self))]
    pub async fn connect(&mut self) -> ClientResult<()> {
        tracing::debug!("connecting to IRC: authorizing + requesting capabilities");

        // `identify()` authenticates the user with the server
        self.inner.identify()?;
        self.inner.send_cap_req(&[
            TwitchCapability::Commands.into(),
            TwitchCapability::Membership.into(),
            TwitchCapability::Tags.into(),
        ])?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn join_channel(&mut self, ch: &str) -> ClientResult<bool> {
        if let Err(e) = self.inner.send_join(ch) {
            tracing::warn!(error = ?e, channel = ch, "JOIN failure");
            Ok(false)
        } else {
            tracing::info!(channel = ch, "JOIN success");
            self.joined.push(ch.to_string());
            Ok(true)
        }
    }

    #[instrument(skip(self))]
    pub async fn join_channels(&mut self, channels: Vec<String>) -> ClientResult<()> {
        let join_string = channels.join(",");

        tracing::info!(join_string, "trying JOIN");
        if let Err(e) = self.inner.send_join(join_string.clone()) {
            tracing::error!(error = ?e, join_string, "JOIN failure");
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn join_all_channels(&mut self) -> ClientResult<()> {
        let channels = self.channels.clone();
        self.join_channels(channels).await
    }
}

const TTV_IRC_URI: &str = "irc.chat.twitch.tv";
const TTV_IRC_PORT: u16 = 6697;

#[derive(Debug)]
enum DisconnectReason {
    KeepaliveTimeout,
    ResetRequested,
    StreamEnded,
}
