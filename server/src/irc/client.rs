use std::collections::HashSet;
use std::time::Duration;

use futures::StreamExt;
use irc::client::{ClientStream, prelude::*};
use irc::proto::CapSubCommand;
use thiserror::Error;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::db::models::{Chatter, DbUser};
use crate::db::pg::{PgErr, db_pool};
use crate::util::channel::{ChannelError, update_channels};
use crate::util::env::{EnvErr, Var};
use crate::var;

#[derive(Debug)]
pub struct MpscChannels {
    pub sender: UnboundedSender<IrcCommand>,
    pub receiver: UnboundedReceiver<IrcMessage>,
}

#[derive(Debug)]
pub enum IrcCommand {
    Privmsg { channel: String, data: String },
    ReplyPm { channel: String, message: String },
}

#[derive(Debug)]
pub enum IrcMessage {
    Privmsg { tags: IrcTags, message: String },
    GetChannels,
    StreamStart { channel: String },
}

#[derive(Debug, Default)]
pub struct IrcTags {
    pub user_id: String,
    pub user_login: String,
    pub color: String,
    pub channel_name: String,
    pub channel_id: String,
}

#[instrument]
pub async fn irc_runner(channels: Vec<String>) -> IrcResult<Vec<tokio::task::JoinHandle<()>>> {
    let (mut irc_client, channels) = IrcConnection::init(channels).await?;
    let rx_handle = tokio::spawn(async move {
        let mut rx_channel = channels.receiver;
        let mut tx_channel = channels.sender;

        read_channel(&mut rx_channel, &mut tx_channel)
            .await
            .unwrap();
    });

    let client_stream_reader = tokio::spawn(async move {
        irc_client.connect().await.unwrap();
        let mut stream = irc_client.client.stream().unwrap();

        let mut check_interval = tokio::time::interval(Duration::from_secs(30));
        check_interval.tick().await;

        let joined_channels = irc_client.get_joined();
        tracing::info!(
            joined_count = joined_channels.len(),
            joined_names = ?joined_channels,
            "joined channel info"
        );

        loop {
            tokio::select! {
                Some(msg_res) = stream.next() => {
                    if let Ok(msg) = msg_res {
                        command_parser(&msg, &mut irc_client).await.unwrap();
                    }
                }

                Some(cmd) = irc_client.receiver.recv() => {
                    match cmd {
                        IrcCommand::ReplyPm { channel, message } => {
                            irc_client.client.send_privmsg(channel, message).unwrap();
                        }
                        _ => (),
                    }
                }

                _ = check_interval.tick() => {
                    if let Err(e) = rejoin_channels(&mut irc_client).await {
                        tracing::error!(error = ?e, "channel rejoin failure");
                    }
                }
            }
        }
    });

    Ok(vec![client_stream_reader, rx_handle])
}

#[instrument(skip(client))]
async fn rejoin_channels(client: &mut IrcConnection) -> IrcResult<()> {
    let expected: HashSet<String> = client.channels.iter().cloned().collect();
    let joined: HashSet<String> = client.get_joined().into_iter().collect();

    let missing: Vec<String> = expected.difference(&joined).cloned().collect();

    if !missing.is_empty() {
        tracing::warn!(missing_count = missing.len(), missing = ?missing, "trying channel rejoin");
        client.join_channels(missing)?;
    } else {
        tracing::debug!(joined_count = joined.len(), "all channels appear joined");
    }

    Ok(())
}

impl IrcConnection {
    #[instrument(skip(channels))]
    pub async fn init(channels: Vec<String>) -> IrcResult<(Self, MpscChannels)> {
        let channel_rooms: Vec<String> = channels.iter().map(|chan| format!("#{}", chan)).collect();

        tracing::info!(channels = ?channels, "channel list");

        let config = Config {
            use_tls: Some(true),
            nickname: Some(var!(Var::UserLogin).await.unwrap().to_string()),
            password: Some(format!("oauth:{}", var!(Var::UserToken).await.unwrap())),

            server: Some(TTV_IRC_URI.to_string()),
            port: Some(TTV_IRC_PORT),
            ping_time: Some(300),
            ..Config::default()
        };

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<IrcCommand>();
        let (msg_tx, msg_rx) = mpsc::unbounded_channel::<IrcMessage>();

        let connection = Client::from_config(config.clone()).await.unwrap();

        let client = (
            Self {
                config,
                client: connection,
                channels: channel_rooms,
                sender: msg_tx,
                receiver: cmd_rx,
                id: Uuid::new_v4(),
            },
            MpscChannels {
                sender: cmd_tx,
                receiver: msg_rx,
            },
        );

        Ok(client)
    }

    #[instrument(skip(self))]
    pub async fn connect(&mut self) -> IrcResult<()> {
        self.client.identify()?; // authenticate
        self.client.send_cap_req(&[
            TtvCap::Commands.into(),
            TtvCap::Membership.into(),
            TtvCap::Tags.into(),
        ])?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn join_all_channels(&mut self) -> IrcResult<()> {
        let channels = self.channels.clone();
        self.join_channels(channels)
    }

    #[instrument(skip(self))]
    pub fn join_channels(&mut self, channels: Vec<String>) -> IrcResult<()> {
        let join_str = channels.join(",");

        tracing::info!("sending join");
        self.client.send_join(join_str)?;

        Ok(())
    }

    #[instrument(skip(self), fields(id = %self.id))]
    pub fn get_joined(&mut self) -> Vec<String> {
        if let Some(channels) = self.client.list_channels() {
            return channels;
        }

        Vec::new()
    }
}

#[instrument(skip(msg, client))]
pub async fn command_parser(msg: &Message, client: &mut IrcConnection) -> IrcResult<()> {
    match &msg.command {
        // this is the only command we REALLY care about, but the others
        // are nice to have
        Command::PRIVMSG(channel, msg_content) => {
            let data = IrcMessage::Privmsg {
                tags: parse_tags(msg, channel),
                message: msg_content.to_string(),
            };

            tracing::info!(data = ?data, "RX PRIVMSG");
            send_to_reader(&client.sender, data).await;
        }

        Command::PONG(_, _) | Command::PING(_, _) => {
            let joined = client.get_joined();
            tracing::info!(
                current_joined_channel_count = joined.len(),
                total_tracked_channel_count = client.channels.len(),
                "IRC join stats (RX PING)",
            );

            tracing::debug!("all channels: {:#?}", client.channels);
            tracing::debug!("joined: {:#?}", joined);
        }

        Command::CAP(_, result, caps, _) => match result {
            CapSubCommand::ACK => {
                if let Some(caps) = caps {
                    tracing::info!("CAP REQ {} ok", caps);
                }

                if client.get_joined().len() == 0 {
                    client.join_all_channels()?;
                }
            }

            CapSubCommand::NAK => {
                tracing::warn!("CAP REQ {:?} invalid", caps)
            }

            _ => tracing::error!("unknown CAP REQ res {:?} (raw msg={:?})", result, msg),
        },

        Command::NOTICE(msg_id, target) => {
            tracing::warn!("{}: RECV NOTICE: {}", target, msg_id);
        }

        Command::JOIN(channel, _, _) => {
            if let Some(Prefix::Nickname(user, _, _)) = &msg.prefix {
                tracing::debug!("{}: JOIN {}", user, channel);
            }
        }

        Command::PART(channel, _) => {
            if let Some(Prefix::Nickname(user, _, _)) = &msg.prefix {
                tracing::info!("{}: PART {}", user, channel);
            }
        }

        Command::Raw(ttv_command, channels) => {
            parse_ttv_command(ttv_command, channels, msg);
        }

        Command::Response(response, parts) => {
            parse_ttv_response(response, parts, msg);
        }

        _ => {
            tracing::debug!(command = ?msg.command, message = ?msg, "IRC received generic cmd");
        }
    }

    Ok(())
}

#[instrument(skip(id))]
pub async fn chatter_by_id(id: &str) -> IrcResult<Chatter> {
    let conn = db_pool().await.unwrap();
    Chatter::get_by_id(conn, id)
        .await
        .map_err(|e| IrcClientErr::PgErr(e))
}

#[instrument(skip(login))]
pub async fn chatter_by_login(login: &str) -> IrcResult<Chatter> {
    let conn = db_pool().await.unwrap();
    Chatter::get_by_login(conn, login)
        .await
        .map_err(|e| IrcClientErr::PgErr(e))
}

#[instrument(skip(rx, tx))]
pub async fn read_channel(
    rx: &mut UnboundedReceiver<IrcMessage>,
    tx: &mut UnboundedSender<IrcCommand>,
) -> IrcResult<()> {
    tracing::debug!("IRC mpsc reader started");
    loop {
        if let Some(msg) = rx.recv().await {
            match msg {
                IrcMessage::Privmsg { tags, message } => {
                    if message.starts_with("!pisscount")
                        && (["plss", "sleepiebug", "gibbbons"]
                            .contains(&tags.channel_name.as_str()))
                    {
                        let message = parse_message_for_response(&message, &tags).await;

                        tracing::info!(message = ?message, "responding to query");

                        tx.send(IrcCommand::ReplyPm {
                            channel: format!("#{}", tags.channel_name),
                            message,
                        })?;
                    }
                }
                _ => {}
            }
        }
    }
}

#[instrument]
pub async fn parse_message_for_response(message: &str, tags: &IrcTags) -> String {
    let target;
    let parts = message.split(' ').collect::<Vec<_>>();

    if parts.len() != 1 {
        target = chatter_by_login(&parts[1].to_lowercase()).await;
    } else {
        target = chatter_by_id(&tags.user_id).await;
    }

    match target {
        Ok(ch) => {
            let requested_user = if parts.len() != 1 {
                format!("{}'s", ch.name)
            } else {
                "your".to_string()
            };

            format!(
                "@{} {} of {} messages have mentioned piss :3",
                tags.user_login, ch.total, requested_user,
            )
        }
        Err(IrcClientErr::PgErr(err)) => {
            tracing::error!(error = ?err, "IRC-based query failed to find a chatter");
            format!("@{} literally hwo ://", tags.user_login)
        }
        Err(err) => {
            tracing::error!(error = ?err, "IRC-based query failed in an unexpected way");
            String::default()
        }
    }
}

#[instrument(skip(tx, data))]
pub async fn send_to_reader(tx: &UnboundedSender<IrcMessage>, data: IrcMessage) {
    match tx.send(data) {
        Ok(_) => (),
        Err(err) => tracing::error!(error = ?err, "failed to send to handler channel"),
    }
}

#[instrument(skip(rx))]
pub async fn read_commands_channel(rx: &mut UnboundedReceiver<IrcCommand>) -> IrcResult<()> {
    if let Some(msg) = rx.recv().await {
        warn!(msg = ?msg, "RX (IN CLIENT)");
    }

    Ok(())
}

#[instrument(skip(msg, channel))]
pub fn parse_tags(msg: &Message, channel: &str) -> IrcTags {
    let mut result = IrcTags::default();

    result.channel_name = channel.rsplit('#').next().unwrap_or("UNKNOWN").to_string();
    for tag in msg.tags.clone().unwrap_or(Vec::new()) {
        match (tag.0.as_str(), tag.1) {
            ("room-id", Some(room_id)) => result.channel_id = room_id,
            ("display-name", Some(name)) => result.user_login = name.to_lowercase(),
            ("user-id", Some(user_id)) => result.user_id = user_id,
            ("color", Some(color)) => result.color = color,
            _ => (),
        }
    }

    result
}

// #[instrument(skip(command, channels, msg))]
#[inline]
pub fn parse_ttv_command(command: &str, channels: &Vec<String>, msg: &Message) {
    match command {
        _ => (),
    }
}

// #[instrument(skip(response, parts, msg))]
#[inline]
pub fn parse_ttv_response(response: &Response, parts: &Vec<String>, msg: &Message) {
    match response {
        Response::RPL_MOTD => {
            info!(username = parts[0], motd = parts[1], "MOTD RX");
        }

        _ => (),
    }
}

#[instrument(skip(stream))]
pub async fn read_incoming(stream: &mut ClientStream) -> Option<Message> {
    if let Ok(incoming) = stream.select_next_some().await {
        return Some(incoming);
    }

    None
}

const TTV_IRC_URI: &str = "irc.chat.twitch.tv";
const TTV_IRC_PORT: u16 = 6697;

pub type IrcResult<T> = core::result::Result<T, IrcClientErr>;

#[derive(Debug, Error)]
pub enum IrcClientErr {
    #[error(transparent)]
    EnvError(#[from] EnvErr),

    #[error(transparent)]
    ClientError(#[from] irc::error::Error),

    #[error(transparent)]
    ChannelError(#[from] ChannelError),

    #[error(transparent)]
    PgErr(#[from] PgErr),

    #[error(transparent)]
    MpscSendCommandErr(#[from] mpsc::error::SendError<IrcCommand>),
}

#[derive(Debug)]
pub enum TtvCap {
    Tags,
    Commands,
    Membership,
}

impl From<TtvCap> for Capability {
    fn from(value: TtvCap) -> Self {
        match value {
            TtvCap::Tags => Capability::Custom("twitch.tv/tags"),
            TtvCap::Commands => Capability::Custom("twitch.tv/commands"),
            TtvCap::Membership => Capability::Custom("twitch.tv/membership"),
        }
    }
}

#[derive(Debug)]
pub struct IrcConnection {
    pub config: Config,
    pub client: Client,
    pub channels: Vec<String>,
    pub sender: UnboundedSender<IrcMessage>,
    pub receiver: UnboundedReceiver<IrcCommand>,
    pub id: uuid::Uuid,
}

#[cfg(test)]
mod test {
    use futures::{future::join_all, stream};

    use super::*;

    #[tokio::test]
    async fn test_create_handler() {
        let provider = crate::util::tracing::build_subscriber().await.unwrap();
        let handles = irc_runner(vec!["plss".to_string()]).await.unwrap();

        _ = join_all(handles).await;

        crate::util::tracing::destroy_tracer(provider);
    }

    #[tokio::test]
    async fn test_all_channels_handler() {
        let provider = crate::util::tracing::build_subscriber().await.unwrap();

        let tracked_channels = update_channels(None).await.unwrap();
        let channel_names = tracked_channels.into_iter().map(|(chan, _)| chan).collect();

        let handles = irc_runner(channel_names).await.unwrap();

        let res = join_all(handles).await;

        tracing::info!(fut_result = ?res, "awaited result");

        crate::util::tracing::destroy_tracer(provider);
    }
}
