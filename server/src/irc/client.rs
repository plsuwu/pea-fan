use std::collections::HashSet;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use std::time::Duration;

use futures::StreamExt;
use irc::client::{ClientStream, prelude::*};
use irc::proto::CapSubCommand;
use irc::proto::message::Tag;
use thiserror::Error;
use tokio::sync::OnceCell;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::Sender;
use tokio::time::Interval;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::db::prelude::*;
use crate::irc::ReplyReason;
use crate::util::channel::ChannelError;
use crate::util::env::{EnvErr, Var};
use crate::util::helix::Helix;
use crate::var;

#[derive(Debug)]
pub struct MpscChannels {
    pub sender: UnboundedSender<IrcCommand>,
    pub receiver: UnboundedReceiver<IrcMessage>,
}

#[derive(Debug)]
pub enum IrcCommand {
    Privmsg {
        channel: String,
        data: String,
    },
    ReplyPm {
        channel: String,
        message: String,
        reply_id: String,
    },
    Incr,
}

#[derive(Debug)]
pub enum IrcMessage {
    Privmsg { tags: IrcTags, message: String },
}

#[derive(Debug, Clone, Default)]
pub struct IrcTags {
    pub user_id: String,
    pub user_login: String,
    pub color: String,
    pub channel_name: String,
    pub channel_id: String,
    pub msg_id: String,
}

const COUNTER_USER: &str = "pee_liker";
const CHANNEL_WHITELIST: [&str; 7] = [
    "plss",
    "chikogaki",
    "lcolonq",
    "madmad01",
    "aaallycat",
    "gibbbons",
    "sleepiebug",
];
const ID_BLACKLIST: [&str; 3] = [
    "19264788",  // Nightbot
    "100135110", // StreamElements
    "1152307157", // us!!!
                 // ... other bots ...
];

static IGNORED_USER_IDS: LazyLock<OnceCell<HashSet<&str>>> = LazyLock::new(OnceCell::new);

/// Provides constant-time lookups via a `LazyLock`ed HashSet to check for blacklisted users
/// on-the-fly.
///
/// # Note
///
/// Blacklist is only two ids at present, and the overhead associated with instantiation of a
/// hashset is significant compared to checking the two items in an otherwise constant array.
#[allow(dead_code)]
#[instrument]
pub async fn is_ignored_user(user_id: &str) -> bool {
    let blacklist = IGNORED_USER_IDS
        .get_or_try_init(|| async { ignored_hashset().await })
        .await
        .unwrap();

    blacklist.contains(user_id)
}

async fn ignored_hashset() -> Result<HashSet<&'static str>, ()> {
    Ok(HashSet::from_iter(ID_BLACKLIST))
}

pub struct ReplyCooldown {
    can_reply: Arc<RwLock<AtomicBool>>,
}

impl ReplyCooldown {
    pub async fn new() -> Result<Self, ()> {
        Ok(Self {
            can_reply: Arc::new(RwLock::new(AtomicBool::new(true))),
        })
    }
}

static REPLY_TIMER: LazyLock<OnceCell<ReplyCooldown>> = LazyLock::new(OnceCell::new);
async fn get_reply_timer() -> &'static ReplyCooldown {
    REPLY_TIMER
        .get_or_try_init(|| async { ReplyCooldown::new().await })
        .await
        .unwrap()
}
async fn can_reply() -> bool {
    let reply_timer = get_reply_timer().await;
    reply_timer
        .can_reply
        .read()
        .unwrap()
        .load(Ordering::Relaxed)
}

async fn set_can_reply(val: bool) {
    let reply_timer = get_reply_timer().await;

    reply_timer
        .can_reply
        .write()
        .unwrap()
        .store(val, Ordering::Relaxed);
}

// async fn reply_interval(reply_timer: &'static ReplyCooldown) {
// }

#[instrument]
pub async fn start_irc_handler(
    channels: Vec<String>,
    mut rx_from_api: UnboundedReceiver<(String, Sender<Vec<String>>)>,
) -> IrcResult<Vec<tokio::task::JoinHandle<()>>> {
    let (mut irc_client, channels) = IrcConnection::init(channels).await?;

    let rx_handle = tokio::spawn(async move {
        let mut rx_channel = channels.receiver;
        let mut tx_channel = channels.sender;
        loop {
            match read_channel(&mut rx_channel, &mut tx_channel).await {
                Ok(r) => tracing::warn!(result = ?r, "reader thread returned early"),
                Err(e) => tracing::error!(error = ?e, "error in reader thread"),
            }

            tracing::warn!("reader thread restarting");
        }
    });

    const REPLY_TIMER_DURATION: Duration = Duration::from_millis(2350);
    let reply_cooldown_handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(REPLY_TIMER_DURATION).await;
            if !can_reply().await {
                set_can_reply(true).await;
            }
        }
    });

    let client_stream_reader = tokio::spawn(async move {
        irc_client.connect().await.unwrap();
        let mut stream = irc_client.client.stream().unwrap();

        const MIN_CHECK_DURATION: Duration = Duration::from_secs(25);
        const MAX_CHECK_DURATION: Duration = Duration::from_secs(480);
        let mut check_interval = MIN_CHECK_DURATION;

        let joined_channels = irc_client.get_joined();
        tracing::warn!(
            joined_count = joined_channels.len(),
            joined_names = ?joined_channels,
            "CHANNELS::JOIN_INFO"
        );

        let mut check_timer = Box::pin(tokio::time::sleep(check_interval));
        loop {
            tokio::select! {
                Some(msg_res) = stream.next() => {
                    if let Ok(msg) = msg_res {
                        command_parser(&msg, &mut irc_client).await.unwrap();
                    }
                }

                Some(cmd) = irc_client.receiver.recv() => {
                    match cmd {
                        IrcCommand::ReplyPm { channel, message, reply_id } => {
                            let reply_tag = vec![Tag(String::from("reply-parent-msg-id"), Some(reply_id))];
                            let fmt_channel = format!("{}", channel);
                            let tagged_message =
                                Message::with_tags(Some(reply_tag), None, "PRIVMSG", vec![&fmt_channel, &message])
                                        .unwrap();
                            match irc_client.client.send(tagged_message) {
                                Ok(_) => tracing::debug!("send ok"),
                                Err(e) => tracing::error!(error = ?e, "error while trying to send reply to IRC"),
                            }
                        },
                        _ => (),
                    }
                }

                Some((msg, tx_to_api)) = rx_from_api.recv() => {
                    tracing::debug!(msg, "CHANNEL_INTL_RX::FROM_API");
                    match msg.as_str() {
                        "irc_joins" => {
                            let joined_channels = irc_client.get_joined();
                            tx_to_api.send(joined_channels.clone()).unwrap();
                        },
                        _ => continue,
                    }
                }

                _ = check_timer.as_mut() => {
                    tracing::debug!("timer interval elapsed");
                    match rejoin_channels(&mut irc_client).await {
                        Ok(all_joined) => {
                            if all_joined {
                                if check_interval < MAX_CHECK_DURATION {
                                    let new_interval = check_interval.saturating_mul(2).min(MAX_CHECK_DURATION);
                                    if new_interval != check_interval {
                                        check_interval = new_interval;
                                        tracing::info!(next_interval = ?check_interval, "IRC_JOINS::INTERVAL_INC");
                                    }
                                }
                            } else {
                               check_interval = MIN_CHECK_DURATION;
                               tracing::warn!(check_interval = ?check_interval, "IRC_JOINS::INTERVAL_RST");
                            }
                        },
                        Err(err) => {
                            tracing::error!(error = ?err, "channel rejoin failure");
                            check_interval = MIN_CHECK_DURATION;
                        }
                    }

                    check_timer.set(tokio::time::sleep(check_interval));
                }
            }
        }
    });

    Ok(vec![client_stream_reader, rx_handle, reply_cooldown_handle])
}

/// Checks whether any tracked channels are *not* currently joined and attempts to join them
///
/// # Returns
///
/// If all channels are joined and accounted for, returns `Ok(true)`.
///
/// Otherwise, if one or more channels are found to be unjoined (but there are otherwise no errors),
/// this function will return `Ok(false)`
#[instrument(skip(client))]
async fn rejoin_channels(client: &mut IrcConnection) -> IrcResult<bool> {
    tracing::warn!("checking for unjoined channels");
    let expected: HashSet<String> = client.channels.iter().cloned().collect();
    let joined: HashSet<String> = client.get_joined().into_iter().collect();

    let missing: Vec<String> = expected.difference(&joined).cloned().collect();

    if !missing.is_empty() {
        tracing::warn!(missing_count = missing.len(), missing = ?missing, "trying channel rejoin");
        client.join_channels(missing)?;

        Ok(false)
    } else {
        tracing::debug!(joined_count = joined.len(), "all channels appear joined");

        Ok(true)
    }
}

impl IrcConnection {
    /// `channels` should be a `Vec<String>` containing the login names for the channels we want to
    /// join (i.e. no leading '#' - this is formatted internally):
    ///
    /// ```ignore
    /// // e.g.:
    ///
    /// let channel_names = vec!["plss".to_string()];
    /// let (connection, _) = IrcConnection::init(channel_names).await?;
    ///
    /// assert_eq!(connection.channels, vec!["#plss".to_string()]);
    /// ```
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
            ping_time: Some(280),
            ..Config::default()
        };

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<IrcCommand>();
        let (msg_tx, msg_rx) = mpsc::unbounded_channel::<IrcMessage>();

        let connection = Client::from_config(config.clone()).await.unwrap();

        let client = (
            Self {
                config,
                curr_jitter: 0,
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
        tracing::debug!("connecting to IRC: authorizing + requesting capabilities");

        // `identify()` authenticates the user with the server
        self.client.identify()?;
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
    let command = &msg.command;
    let tags = &msg.tags;
    let prefix = &msg.prefix;

    tracing::trace!(
        command = ?command,
        tags = ?tags,
        prefix = ?prefix,
        response_target = ?msg.response_target(),
        "trace message parts"
    );

    match &msg.command {
        // this is the only command we REALLY care about, but the others
        // are nice to have, particularly for logging purposes
        Command::PRIVMSG(channel, msg_content) => {
            let tags = parse_tags(msg, channel);
            let message = msg_content.to_string();
            tracing::debug!(
                channel_name = tags.channel_name,
                channel_id = tags.channel_id,
                user_login = tags.user_login,
                user_id = tags.user_id,
                content = msg_content,
                "RX::PRIVMSG"
            );

            let data = IrcMessage::Privmsg { tags, message };
            send_to_reader(&client.sender, data).await;
        }

        Command::PONG(data, _) | Command::PING(data, _) => {
            let joined = client.get_joined();
            tracing::debug!(
                info = data,
                current_joined_count = joined.len(),
                total_tracked_count = client.channels.len(),
                "RX::PING",
            );
        }

        Command::CAP(_, result, caps, _) => match result {
            CapSubCommand::ACK => {
                if let Some(caps) = caps {
                    tracing::info!(capabilities = ?caps, "RX::CAP_ACK");
                }

                if client.get_joined().len() == 0 {
                    client.join_all_channels()?;
                }
            }

            CapSubCommand::NAK => {
                tracing::error!(capabilities = ?caps, "RX::CAP_NAK")
            }

            _ => tracing::warn!(result = ?result, msg = ?msg, "RX::CAP_UNKNOWN"),
        },

        Command::NOTICE(msg_id, target) => {
            tracing::warn!(target, msg_id, ?msg, "RX::NOTICE");

            // TODO:
            //  'duplicate message' NOTICE; we circumvent this by appending invisible
            //  character(s) to the end of our last message but its annoying to set up
            //  and i cant be bothered currently
            if msg_id.contains("less than 30 seconds ago") {
                tracing::error!("RX::DUPLICATE_MSG_NOTICE");
            }
        }

        Command::JOIN(channel, _, _) => {
            if let Some(Prefix::Nickname(user, _, _)) = &msg.prefix {
                tracing::debug!(user, channel, "RX::JOIN");
            }
        }

        Command::PART(channel, _) => {
            if let Some(Prefix::Nickname(user, _, _)) = &msg.prefix {
                tracing::debug!(user, channel, "RX::PART");
            }
        }

        Command::Raw(ttv_command, channels) => {
            parse_ttv_command(ttv_command, channels, msg);
        }

        Command::Response(response, parts) => {
            parse_ttv_response(response, parts, msg);
        }

        _ => {
            tracing::debug!(command = ?msg.command, message = ?msg, "RX::OTHER_UNHANDLED");
            // tracing::debug!(command = ?msg.command, message = ?msg, "IRC received generic cmd");
        }
    }

    Ok(())
}

#[instrument(skip(repo, chatter_id))]
pub async fn chatter_by_id(repo: &ChatterRepository, chatter_id: &str) -> IrcResult<Chatter> {
    Ok(repo
        .get_by_id(&ChatterId(chatter_id.to_string()))
        .await?
        .ok_or_else(|| IrcClientErr::SqlxError(sqlx::Error::RowNotFound))?)
}

#[instrument(skip(login))]
pub async fn chatter_by_login(repo: &ChatterRepository, login: &str) -> IrcResult<Chatter> {
    Ok(repo
        .get_by_login(login.to_string())
        .await
        .map_err(|err| IrcClientErr::SqlxError(err))?)
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
                    let pool = db_pool().await?;
                    // first, we check to see if we should reply to a chatter's message with a
                    // counter query (only doing so for "whitelisted" channels)
                    if message.starts_with("!pisscount")
                        && CHANNEL_WHITELIST.contains(&tags.channel_name.as_str())
                    {
                        if !can_reply().await {
                            tracing::warn!("reply cooldown not yet elapsed");
                            continue;
                        }

                        let chatter_repo = ChatterRepository::new(pool);
                        let message = make_query_response(&chatter_repo, &message, &tags).await?;
                        let channel = format!("#{}", tags.channel_name);
                        let reply_id = tags.msg_id;

                        tracing::info!(
                            message = message,
                            parent_msg = reply_id,
                            channel,
                            "responding to query"
                        );

                        set_can_reply(false).await;
                        tx.send(IrcCommand::ReplyPm {
                            channel,
                            reply_id,
                            message,
                        })?;
                    }
                    // otherwise, check whether we should increment a counter if the message isn't
                    // a `!pisscount` query
                    else if message.contains("piss")
                        && !ID_BLACKLIST.contains(&tags.user_id.as_str())
                    {
                        let res = increment_score(pool, &tags).await?;
                        tracing::info!(
                            increment_result = ?res,
                            chatter = tags.user_login,
                            channel = tags.channel_name,
                            "incremented counter"
                        );

                        tx.send(IrcCommand::Incr)?;
                    }
                }
            }
        }
    }
}

#[instrument]
pub async fn make_query_response(
    repo: &ChatterRepository,
    message: &str,
    tags: &IrcTags,
) -> IrcResult<String> {
    let mut parts = message.split(' ').collect::<Vec<_>>();
    let target = if parts.len() > 1 {
        parts[1] = parts[1].trim_start_matches('@');

        // our count is always going to be 0 but we have fun around here
        if parts[1].to_lowercase() == COUNTER_USER {
            return Ok(ReplyReason::BotCountQueried.get_reply().to_string());
        } else {
            chatter_by_login(repo, &parts[1].to_lowercase()).await
        }
    } else {
        chatter_by_id(repo, &tags.user_id).await
    };

    match target {
        Ok(ch) => {
            let requested_user = if parts.len() != 1 {
                format!("{}'s", ch.name)
            } else {
                "your".to_string()
            };

            Ok(format!(
                "{} of {} messages have mentioned piss",
                ch.total, requested_user,
            ))
        }
        Err(IrcClientErr::SqlxError(err)) => {
            tracing::warn!(error = ?err, "IRC-based query failed due to non-existant user");
            Ok(ReplyReason::RowNotFound.get_reply().to_string())
        }
        Err(err) => {
            tracing::error!(error = ?err, "IRC-based query failed in an unexpected way");
            // twitch should filter the empty message here
            Ok(String::default())
        }
    }
}

#[instrument(skip(pool, tags))]
pub async fn increment_score<'a>(pool: &'static sqlx::PgPool, tags: &'a IrcTags) -> IrcResult<()> {
    let chatter_repo = ChatterRepository::new(pool);
    let chatter = chatter_repo.get_by_id(&tags.user_id.clone().into()).await?;
    let exists = chatter.is_some();

    // i kind of dont want to do this for channels for efficiency reasons - seems better to make sure
    // all channels are present when we read in the channel list and then assume they are present (right??)
    if !exists {
        let mut target_id = vec![tags.user_id.clone()];
        let helix_chatter = Helix::fetch_users_by_id(&mut target_id).await?;

        // TODO: why does this get moved if we dont clone? is it because we return a `Vec` of `T`
        // rather than just the `T`??
        let chatter = Chatter::from(helix_chatter[0].clone());
        chatter_repo.insert(&chatter).await?;
    }

    // let score_repo = LeaderboardRepository::new(pool);
    // let pre_incr = score_repo
    //     .get_relational_score(
    //         &tags.user_id.clone().into(),
    //         &tags.channel_id.clone().into(),
    //     )
    //     .await?;
    // tracing::debug!(pre_incr = ?pre_incr, "score prior to incrementing");

    // do transaction
    match Tx::with_tx(&pool, |mut tx| async move {
        let chatter_id = tags.user_id.clone().into();
        let channel_id = tags.channel_id.clone().into();

        let result = async {
            tx.increment_score_by(&chatter_id, &channel_id, 1).await?;
            tx.recalculate_channel_total(&channel_id).await?;
            tx.recalculate_chatter_total(&chatter_id).await?;

            Ok(())
        }
        .await;

        (tx, result)
    })
    .await
    {
        Err(e) => {
            tracing::error!(
                error = ?e,
                channel = tags.channel_id,
                chatter = tags.user_id,
                "score increment via transaction failure"
            );

            return Err(IrcClientErr::SqlxError(e));
        }
        _ => tracing::info!(
            channel = tags.channel_id,
            chatter = tags.user_id,
            channel_name = tags.channel_name,
            login = tags.user_login,
            "increment ok"
        ),
    };

    // let post_incr = score_repo
    //     .get_relational_score(
    //         &tags.user_id.clone().into(),
    //         &tags.channel_id.clone().into(),
    //     )
    //     .await?;
    // tracing::debug!(post_incr = ?post_incr, "score after incrementing");

    Ok(())
}

#[instrument(skip(tx, data))]
pub async fn send_to_reader(tx: &UnboundedSender<IrcMessage>, data: IrcMessage) {
    match tx.send(data) {
        Ok(_) => (),
        Err(err) => {
            tracing::error!(error = ?err, "failed to send to handler channel");
            return;
        }
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
            ("id", Some(msg_id)) => result.msg_id = msg_id,
            _ => (),
        }
    }

    result
}

#[instrument(skip(command, channels, msg))]
#[inline]
pub fn parse_ttv_command(command: &str, channels: &Vec<String>, msg: &Message) {
    match command {
        _ => (),
    }
}

#[instrument(skip(response, parts, msg))]
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
    PgErr(#[from] PgError),

    #[error("{0:?}")]
    MpscSendCommandErr(#[from] mpsc::error::SendError<IrcCommand>),

    #[error(transparent)]
    SqlxError(#[from] sqlx::error::Error),

    #[error(transparent)]
    HelixError(#[from] crate::util::helix::HelixErr),
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
    pub curr_jitter: u8,
    pub client: Client,
    pub channels: Vec<String>,
    pub sender: UnboundedSender<IrcMessage>,
    pub receiver: UnboundedReceiver<IrcCommand>,
    pub id: uuid::Uuid,
}

#[cfg(test)]
mod test {
    use std::net::SocketAddr;

    use futures::future::join_all;

    use crate::{api::server::start_server, util::channel::update_channels};

    use super::*;

    #[tokio::test]
    async fn test_channel_handler_small() {
        let provider = crate::util::tracing::build_subscriber().await.unwrap();

        let (tx_server, rx) = tokio::sync::mpsc::unbounded_channel::<SocketAddr>();
        let (tx_from_api, rx_from_api) =
            tokio::sync::mpsc::unbounded_channel::<(String, Sender<Vec<String>>)>();

        let channels = ["plss", "gibbbons", "chikogaki"]
            .into_iter()
            .map(|ch| ch.to_string())
            .collect();
        let mut handles = start_server(tx_server, tx_from_api, rx).await.unwrap();
        handles.extend(start_irc_handler(channels, rx_from_api).await.unwrap());

        _ = join_all(handles).await;
        crate::util::tracing::destroy_tracer(provider);
    }

    #[tokio::test]
    async fn test_channel_handler_all() {
        let provider = crate::util::tracing::build_subscriber().await.unwrap();
        let (tx_server, rx) = tokio::sync::mpsc::unbounded_channel::<SocketAddr>();
        let (tx_from_api, rx_from_api) =
            tokio::sync::mpsc::unbounded_channel::<(String, Sender<Vec<String>>)>();

        let tracked_channels = update_channels(None).await.unwrap();
        let channels = tracked_channels.into_iter().map(|(chan, _)| chan).collect();

        let mut handles = start_server(tx_server, tx_from_api, rx).await.unwrap();
        handles.extend(start_irc_handler(channels, rx_from_api).await.unwrap());

        _ = join_all(handles).await;

        crate::util::tracing::destroy_tracer(provider);
    }
}
