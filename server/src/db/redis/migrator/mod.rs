use std::collections::HashMap;

use redis::AsyncCommands;
use sqlx::{Pool, Postgres};
use tracing::instrument;

use crate::db::prelude::*;
use crate::db::redis::migrator::io::PgHandler;
use crate::db::redis::migrator::util::KeyList;
use crate::db::redis::redis_pool::{KeyType, RedisResult};
use crate::util::helix::Helix;

pub mod io;
pub mod transform;
pub mod util;

pub type LeaderboardRow = HashMap<String, i64>;
pub type LeaderboardMap = HashMap<String, LeaderboardRow>;

const DEFAULT_TIMESTAMP_OFFSET: i64 = 120;

#[instrument(skip(redis_pool, database_pool))]
pub async fn process_initial_migration<R: AsyncCommands + Sync>(
    redis_pool: R,
    database_pool: &'static Pool<Postgres>,
) -> RedisResult<()> {
    let mut migrator = Migrator::new(redis_pool, database_pool);
    migrator.migrate_cached_channels().await?;
    let (cached_chatters, resolved_chatters) = migrator.migrate_cached_chatters().await?;

    let (resolved, rejected) = migrator
        .migrate_cached_leaderboards(cached_chatters, &resolved_chatters)
        .await?;

    tracing::error!(?rejected, "INVALID CHATTERS");

    migrator
        .postgres_handler
        .migrate(resolved, DEFAULT_TIMESTAMP_OFFSET)
        .await?;

    Ok(())
}

#[instrument(skip(redis_pool, database_pool, aliases), fields(aliases_count = aliases.len()))]
pub async fn process_alias_migration<R: AsyncCommands + Sync>(
    redis_pool: R,
    database_pool: &'static Pool<Postgres>,
    current_name: &str,
    aliases: &[String],
) -> RedisResult<()> {
    let mut migrator = Migrator::new(redis_pool, database_pool);
    let (chatter_id, leaderboard) = migrator
        .migrate_aliased_chatter(current_name, aliases)
        .await?;

    migrator
        .postgres_handler
        .migrate_alias(&chatter_id, leaderboard, DEFAULT_TIMESTAMP_OFFSET)
        .await?;

    Ok(())
}

#[derive(Debug)]
pub struct Migrator<'a, R: AsyncCommands + Sync> {
    redis_connection: R,
    database_pool: &'static Pool<Postgres>,
    postgres_handler: PgHandler<'a>,
}

impl<'a, R> Migrator<'a, R>
where
    R: AsyncCommands + Sync,
{
    pub fn new(redis_connection: R, database_pool: &'static Pool<Postgres>) -> Self {
        Self {
            redis_connection,
            database_pool,
            postgres_handler: io::PgHandler(database_pool),
        }
    }

    pub async fn migrate_cached_channels(&mut self) -> RedisResult<()> {
        let chatter_repo = ChatterRepository::new(self.database_pool);
        let mut redis_handler = io::RedisHandler(&mut self.redis_connection);

        let cached_channels_raw = redis_handler.fetch_keys(KeyType::Channel).await?;
        let mut parsed_channel_ids = cached_channels_raw
            .parse(|name| {
                name.split(':')
                    .nth(1)?
                    .split('#')
                    .nth(1)
                    .map(|ch| util::resolve_channel_id(&ch.to_lowercase()))
            })
            .dedup();

        tracing::debug!(channel_ids = ?parsed_channel_ids, "retrieved and parsed channel_id list from redis keys");

        let helix_channels = Helix::fetch_users_by_id(&mut parsed_channel_ids).await?;
        let broadcaster_chatters: Vec<Chatter> =
            helix_channels.into_iter().map(Chatter::from).collect();

        chatter_repo.insert_many(&broadcaster_chatters).await?;

        let channels: Vec<Channel> = broadcaster_chatters
            .into_iter()
            .map(Channel::from)
            .collect();
        ChannelRepository::new(self.database_pool)
            .insert_many(&channels)
            .await?;

        self.postgres_handler.insert_reply_config(&channels).await?;

        Ok(())
    }

    pub async fn migrate_cached_chatters(&mut self) -> RedisResult<(Vec<String>, Vec<Chatter>)> {
        let chatter_repo = ChatterRepository::new(self.database_pool);
        let mut redis_handler = io::RedisHandler(&mut self.redis_connection);

        let cached_chatters_raw = redis_handler.fetch_keys(KeyType::Chatter).await?;
        let parsed_chatters =
            cached_chatters_raw.parse(|name| name.split(':').nth(1).map(str::to_owned));

        let helix_users = Helix::fetch_users_by_login(parsed_chatters.dedup().lowercase()).await?;
        let resolved_chatters: Vec<Chatter> = helix_users.into_iter().map(Chatter::from).collect();
        chatter_repo.insert_many(&resolved_chatters).await?;

        Ok((parsed_chatters, resolved_chatters))
    }

    #[instrument(skip(self, current_name, aliases))]
    async fn migrate_aliased_chatter(
        &mut self,
        current_name: &str,
        aliases: &[String],
    ) -> RedisResult<(ChatterId, LeaderboardRow)> {
        let helix_user_vec = Helix::fetch_users_by_login(vec![current_name.to_string()]).await?;
        let chatter = Chatter::from(helix_user_vec[0].clone());
        ChatterRepository::new(self.database_pool)
            .insert(&chatter)
            .await?;

        tracing::debug!(?chatter, "found valid helix data");

        let mut redis_handler = io::RedisHandler(&mut self.redis_connection);
        let leaderboard_map = redis_handler.fetch_alias_leaderboards(aliases).await?;

        tracing::debug!(?leaderboard_map, "mapped alias leaderboard");

        Ok((chatter.id, leaderboard_map))
    }

    async fn migrate_cached_leaderboards(
        &mut self,
        keylist: Vec<String>,
        resolved_chatters: &[Chatter],
    ) -> RedisResult<(LeaderboardMap, LeaderboardMap)> {
        let mut redis_handler = io::RedisHandler(&mut self.redis_connection);
        let chatter_logins: HashMap<String, String> = resolved_chatters
            .iter()
            .map(|chatter| (chatter.login.to_string(), chatter.id.to_string()))
            .collect();

        let mut resolved: LeaderboardMap = HashMap::new();
        let mut rejected: LeaderboardMap = HashMap::new();

        for chatter_name in keylist {
            let chatter_leaderboard_raw = redis_handler
                .fetch_leaderboard(&chatter_name, KeyType::Chatter)
                .await?;

            let parsed_leaderboard: HashMap<String, i64> = chatter_leaderboard_raw
                .iter()
                .map(|(channel, score)| {
                    let trimmed_channel_name = transform::trim_octo(&channel);
                    let channel = util::resolve_channel_id(&trimmed_channel_name);

                    (channel, *score)
                })
                .collect();

            if let Some(resolved_chatter_id) = chatter_logins.get(&chatter_name.to_lowercase()) {
                resolved
                    .entry(resolved_chatter_id.to_owned())
                    .and_modify(|curr_board| {
                        for (channel_id, score) in &parsed_leaderboard {
                            tracing::info!(
                                ?curr_board,
                                ?parsed_leaderboard,
                                chatter_name,
                                "CURRENT_LEADERBOARD_PRE"
                            );
                            curr_board
                                .entry(channel_id.to_string())
                                .and_modify(|curr_score| *curr_score += score)
                                .or_insert(*score);
                            tracing::info!(
                                ?curr_board,
                                ?parsed_leaderboard,
                                chatter_name,
                                "CURRENT_LEADERBOARD_POST"
                            );
                        }
                    })
                    .or_insert(parsed_leaderboard);
            } else {
                rejected
                    .entry(chatter_name.to_lowercase())
                    .and_modify(|curr_board| {
                        for (channel_id, score) in &parsed_leaderboard {
                            curr_board
                                .entry(channel_id.to_string())
                                .and_modify(|curr_score| *curr_score += score)
                                .or_insert(*score);
                        }
                    })
                    .or_insert(parsed_leaderboard);
            }
        }

        Ok((resolved, rejected))
    }
}

// /// "fetchable" Helix user data.
// ///
// /// Returns `(resolved, rejected)`, where `resolved` is those chatters with associated Helix user
// /// data, and `rejected` is those chatters whose logins could not be retrieved from the Helix API.
// ///
// /// # Additional Remarks
// ///
// /// The `cached_logins` argument should be a mostly unmodified list of chatter logins; these were
// /// originally fetched and stored as the chatter's `display_name`, and a chatter may be stored
// /// under two different "logins" if they altered the capitalisation of their name during the
// /// lifetime of V1.
// ///
// /// Internally, this function will perform the following steps:
// ///     1. retrieve each user's channel leaderboard through their case-sensitive `display_name`,
// ///     2. the helix usermap is checked for their lowercase `login`,
// ///     3. if found, their leaderboard is added to another HashMap keyed with their lowercase
// ///        `login`, merging leaderboards for any duplicate chatter `login` entries
// #[instrument(skip(conn, helix_users, cached_logins), fields(chatter_count = cached_logins.len(), helix_count = helix_users.len()))]
// pub async fn parse_valid_chatter_leaderboards(
//     conn: &mut impl AsyncCommands,
//     cached_logins: &[String],
//     helix_users: &[Chatter],
// ) -> RedisResult<(LeaderboardMap, LeaderboardMap)> {
//     let mut resolved: LeaderboardMap = HashMap::new();
//     let mut rejected: LeaderboardMap = HashMap::new();
//
//     let helix_usermap = transform::map_login_to_helix_user(helix_users);
//
//     for login in cached_logins {
//         // doing it one-by-one like this appears to be the easiest way to keep continuity between the
//         // current user's login and leaderboard
//         let result: Vec<(String, i64)> = {
//             let raw = io::fetch_leaderboard(conn, KeyType::Chatter, login).await?;
//             raw.into_iter()
//                 .map(|(channel, score)| {
//                     (
//                         util::resolve_channel_id(&transform::trim_octo(&channel)),
//                         score,
//                     )
//                 })
//                 .collect()
//         };
//
//         if let Some(user) = helix_usermap.get(&login.to_lowercase()) {
//             transform::map_leaderboard(&mut resolved, &user.id.0, &result);
//         } else {
//             transform::map_leaderboard(&mut rejected, &login, &result);
//         }
//     }
//
//     Ok((resolved, rejected))
// }
