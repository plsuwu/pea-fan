#![allow(dead_code)]

use std::sync::Arc;

use irc::proto::Message;
use irc::proto::message::Tag;
use sqlx::PgPool;
use tokio::sync::Mutex;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::instrument;

use crate::db::prelude::{
    ChannelId, ChannelRepository, Chatter, ChatterId, ChatterRepository, LeaderboardRepository,
    Repository, Tx,
};
use crate::irc::ReplyReason;
use crate::irc::commands::{IncomingMessage, IrcTags, OutgoingCommand};
use crate::irc::error::{ClientResult, ConnectionClientError};
use crate::irc::parse::format_username;
use crate::irc::rate_limit::Bucket;
use crate::util::helix::Helix;

const TRAILER_CHAR: char = '\u{180B}';
pub const COUNTER_USER: &str = "pee_liker";

/// Names of "response-enabled" broadcasters; bot will only respond to command in these
/// chatrooms to avoid saturating the rate limit and polluting the chat of unsuspecting
/// streamers.
///
/// # Note for the future
///
/// The intention here is to ultimately move whitelist state to the database (or perhaps
/// Redis) so we can dynamically update whitelisted channels via the API
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
    "19264788",   // Nightbot
    "100135110",  // StreamElements
    "1152307157", // us
];

const COMMAND: &str = "!pisscount";
const KEYWORD: &str = "piss";

#[derive(Debug, Default)]
pub struct LastMessage {
    pub channel: String,
    pub message: String,
    pub tagged_chatter: String,
    pub has_invisible_char: bool,
}

#[derive(Debug)]
pub struct WorkerPool {
    workers: Vec<JoinHandle<()>>,
    pub last_message: Arc<Mutex<LastMessage>>,
}

impl WorkerPool {
    #[instrument]
    pub fn spawn(
        count: usize,
        msg_rx: async_channel::Receiver<IncomingMessage>,
        cmd_tx: mpsc::Sender<OutgoingCommand>,
        rate_limiter: Arc<Bucket>,
        pool: &'static PgPool,
    ) -> Self {
        let last_message = Arc::new(Mutex::new(LastMessage::default()));

        let workers = (0..count)
            .map(|id| {
                let rx = msg_rx.clone();
                let tx = cmd_tx.clone();
                let rate_limiter = Arc::clone(&rate_limiter);
                let last_message = Arc::clone(&last_message);

                tokio::spawn(async move {
                    tracing::info!(worker_id = id, "worker started");
                    while let Ok(msg) = rx.recv().await {
                        if let Err(e) =
                            handle_message(msg, &tx, &last_message, &rate_limiter, pool).await
                        {
                            tracing::error!(?e, worker_id = id, "worker error");
                        }
                    }
                })
            })
            .collect();

        Self {
            workers,
            last_message,
        }
    }
}

#[instrument]
async fn is_whitelisted_channel(
    pool: &'static PgPool,
    channel_id: &str,
) -> Result<bool, ConnectionClientError> {
    let repo = ChannelRepository::new(pool);

    let row = repo.get_reply_config(channel_id).await?;

    Ok(row.enabled)
}

#[instrument(skip(msg, cmd_tx, last_message, bucket, pool))]
async fn handle_message(
    msg: IncomingMessage,
    cmd_tx: &mpsc::Sender<OutgoingCommand>,
    last_message: &Arc<Mutex<LastMessage>>,
    bucket: &Arc<Bucket>,
    pool: &'static PgPool,
) -> Result<(), ConnectionClientError> {
    let rate_limiter = bucket.clone();
    match msg {
        IncomingMessage::Privmsg { tags, text } => {
            let channel = format!("{}.#{}", &tags.channel_id, &tags.channel_name);
            let chatter = format!("{}.{}", &tags.user_id, &tags.user_login);

            tracing::info!(
                // msg_id = tags.msg_id,
                channel,
                chatter,
                content = text,
                "PRIVMSG"
            );

            // check for command invocation
            if text.starts_with("!pisscount")
                && is_whitelisted_channel(pool, &tags.channel_id).await?
            {
                tracing::debug!("`pisscount` command recv");
                let repo = ChatterRepository::new(pool);
                let mut reply = build_query_response(&repo, &text, &tags).await?;

                // we use a mutex here as we want to do a single read and a single write in order
                // to atomically compare every response to its predecessor
                let mut guard = last_message.lock().await;
                tracing::trace!(
                    prev_msg_content = ?guard.message,
                    prev_in_channel = ?guard.channel,
                    prev_tagged_chatter = ?guard.tagged_chatter,
                    curr_msg_content = ?reply,
                    curr_in_channel = ?tags.channel_name,
                    curr_tagged_chatter = ?tags.user_login,
                );

                if &guard.channel == &tags.channel_name
                    && &guard.message == &reply
                    && &guard.tagged_chatter == &tags.user_login
                {
                    // circumvent duplicate message filter if content matches
                    reply.push(TRAILER_CHAR);
                }

                guard.channel = tags.channel_name.clone();
                guard.message = reply.clone();
                guard.tagged_chatter = tags.user_login.clone();

                let channel = format!("#{0}", tags.channel_name);
                let reply_tag = vec![Tag(
                    String::from("reply-parent-msg-id"),
                    Some(tags.msg_id.clone()),
                )];
                let response = Message {
                    tags: Some(reply_tag),
                    prefix: None,
                    command: irc::proto::Command::PRIVMSG(channel, reply),
                };

                tracing::debug!(message = ?response, "final `irc::proto::Message` for output");

                // ensure we adhere to rate limits - build response, then wait until a permit is
                // available before pushing out to connection
                rate_limiter.acquire_one().await?;
                tracing::debug!(reply_for = tags.msg_id, "reply permit acquired");
                cmd_tx
                    .send(OutgoingCommand::Reply { message: response })
                    .await?;

            // if not invoking a command, check for keyword
            } else if text.to_lowercase().contains(KEYWORD)
                && !ID_BLACKLIST.contains(&tags.user_id.as_str())
            {
                tracing::info!(tags.user_login, tags.channel_name, "incrementing score");
                increment_score(pool, &tags).await?;
            }

            Ok(())
        }
        _ => {
            tracing::info!(message = ?msg, "received_unhandled_message");
            Ok(())
        }
    }
}

#[instrument(skip(repo))]
pub async fn build_query_response(
    repo: &ChatterRepository,
    message: &str,
    tags: &IrcTags,
) -> ClientResult<String> {
    let mut parts = message.split(' ').collect::<Vec<_>>();
    let target = if parts.len() > 1 {
        parts[1] = parts[1].trim_start_matches('@');

        // our own count is always going to be 0
        if parts[1].to_lowercase() == COUNTER_USER {
            return Ok(ReplyReason::BotCountQueried.get_reply().to_string());
        }

        let chatter_login = parts[1].to_lowercase();
        repo.get_by_login(chatter_login)
            .await
            .map_err(ConnectionClientError::SqlxError)
    } else {
        let chatter_id = ChatterId::from(tags.user_id.to_string());
        repo.get_by_id(&chatter_id)
            .await?
            .ok_or_else(|| ConnectionClientError::SqlxError(sqlx::Error::RowNotFound))
    };

    let requested_user = format_username(parts);
    match target {
        Ok(ch) => Ok(format!(
            "{0} of {requested_user} messages mentioned {KEYWORD}",
            ch.total
        )),
        Err(ConnectionClientError::SqlxError(err)) => {
            tracing::warn!(error = ?err, "IRC-based query failed due to non-existant user");
            Ok(format!(
                "0 of {requested_user} messages mentioned {KEYWORD}"
            ))
        }
        Err(err) => {
            tracing::error!(error = ?err, "IRC-based query failed in an unexpected way");
            // twitch should filter the empty message here
            Ok(String::default())
        }
    }
}

#[instrument(skip(pool))]
pub async fn increment_score(pool: &'static sqlx::PgPool, tags: &IrcTags) -> ClientResult<()> {
    let chatter_repo = ChatterRepository::new(pool);
    let score_repo = LeaderboardRepository::new(pool);

    let chatter = chatter_repo.get_by_id(&tags.user_id.clone().into()).await?;
    let exists = chatter.is_some();

    if !exists {
        let mut target_id = vec![tags.user_id.clone()];
        let helix_chatter = Helix::fetch_users_by_id(&mut target_id).await?;
        let chatter = Chatter::from(helix_chatter[0].clone());
        chatter_repo.insert(&chatter).await?;
    }

    match score_repo
        .record_score_event(
            &tags.user_id.clone().into(),
            &tags.channel_id.clone().into(),
        )
        .await
    {
        Ok(_) => {
            tracing::debug!(
                channel = tags.channel_id,
                chatter = tags.user_id,
                channel_name = tags.channel_name,
                login = tags.user_login,
                "score event recorded"
            );
            Ok(())
        }
        Err(e) => {
            tracing::error!(
                error = ?e,
                channel = tags.channel_id,
                chatter = tags.user_id,
                "score event insert failure"
            );
            Err(ConnectionClientError::SqlxError(e))
        }
    }
}

// #[instrument(skip(pool))]
// pub async fn increment_score(pool: &'static sqlx::PgPool, tags: &IrcTags) -> ClientResult<()> {
//     let chatter_repo = ChatterRepository::new(pool);
//
//     let chatter = chatter_repo.get_by_id(&tags.user_id.clone().into()).await?;
//     let exists = chatter.is_some();
//     if !exists {
//         let mut target_id = vec![tags.user_id.clone()];
//         let helix_chatter = Helix::fetch_users_by_id(&mut target_id).await?;
//
//         let chatter = Chatter::from(helix_chatter[0].clone());
//         chatter_repo.insert(&chatter).await?;
//     }
//
//     // custom transaction handler to auto-commit on an Ok result
//     match Tx::with_tx(pool, |mut tx| async move {
//         let chatter_id = tags.user_id.clone().into();
//         let channel_id = tags.channel_id.clone().into();
//
//         let result = async {
//             tx.increment_score_by(&chatter_id, &channel_id, 1).await?;
//             tx.recalculate_channel_total(&channel_id).await?;
//             tx.recalculate_chatter_total(&chatter_id).await?;
//
//             Ok(())
//         }
//         .await;
//
//         (tx, result)
//     })
//     .await
//     {
//         Err(e) => {
//             tracing::error!(
//                 error = ?e,
//                 channel = tags.channel_id,
//                 chatter = tags.user_id,
//                 "score increment via transaction failure"
//             );
//
//             return Err(ConnectionClientError::SqlxError(e));
//         }
//         _ => tracing::trace!(
//             channel = tags.channel_id,
//             chatter = tags.user_id,
//             channel_name = tags.channel_name,
//             login = tags.user_login,
//             "increment ok"
//         ),
//     };
//
//     Ok(())
// }
