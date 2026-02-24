#![allow(dead_code)]

use std::collections::HashMap;

use redis::{AsyncCommands, CopyOptions, from_redis_value};
use serde::{Deserialize, Serialize};
use tracing::{instrument, warn};

use super::redis_pool::RedisResult;
use crate::db::redis::redis_pool::{KeyType, RedisErr, RedisKey, redis_pool};
use crate::db::repositories::Repository;
use crate::redis_key;
use crate::util::helix::{Helix, HelixUser};

use crate::db::prelude::*;

#[derive(Debug)]
pub struct Migrator {
    pub channels: Vec<Channel>,
    pub chatters: Vec<Chatter>,
    pub scores: Vec<i32>,
}

impl Migrator {
    #[instrument]
    pub fn new() -> Self {
        tracing::info!("migrator init");

        let channels = Vec::new();
        let chatters = Vec::new();
        let scores = Vec::new();

        Self {
            channels,
            chatters,
            scores,
        }
    }

    #[instrument(skip(self))]
    pub async fn process(&mut self) -> RedisResult<()> {
        tracing::info!("begin process pipeline");

        let channel_logins = Self::get_channel_keys().await?;
        tracing::debug!(
            cached_channel_count = channel_logins.len(),
            "retrieved channel keys from redis"
        );

        // process chatters with tracked channels ('broadcasters')
        let (channels, broadcasters) = {
            let fetched = Helix::fetch_users_by_login(channel_logins.clone()).await?;
            (
                Self::merge_channels(fetched.clone()).await?,
                fetched.into_iter().map(Chatter::from).collect::<Vec<_>>(),
            )
        };
        tracing::debug!(
            channels_count = channels.len(),
            broadcasters_count = broadcasters.len(),
            "fetched channel and broadcaster data from helix"
        );

        // (re)map redis channel login names onto the new channel structure for database upset
        let mut channel_map = HashMap::new();
        let channels: Vec<_> = channels
            .into_iter()
            .map(|ch| {
                let ch_name = ch.login.clone();
                let as_channel = Channel::from(ch);
                channel_map.insert(ch_name, as_channel.clone());

                as_channel
            })
            .collect();

        tracing::debug!(
            channel_map_size = channel_map.len(),
            "built channel mapping"
        );

        let pool = db_pool().await?;

        let chatter_repo = ChatterRepository::new(pool);
        let channel_repo = ChannelRepository::new(pool);
        // let score_repo = LeaderboardRepository::new(pool);

        chatter_repo.insert_many(&broadcasters).await?;
        tracing::debug!(
            count = broadcasters.len(),
            "upsert broadcasters to database"
        );

        let broadcasters_channels: Vec<Channel> =
            broadcasters.into_iter().map(Channel::from).collect();
        channel_repo.insert_many(&broadcasters_channels).await?;
        tracing::debug!(count = channels.len(), "channels upserted to database");

        // -- end of initial broadcaster data processing --

        // fetch and process the non-broadcaster chatters
        let mut chatter_logins = Self::get_chatter_keys().await?;
        let num_chatters = chatter_logins.len();
        tracing::debug!(num_chatters, "retrieved chatter keys from redis");

        let mut fetched = Helix::fetch_users_by_login(chatter_logins.clone()).await?;

        // filter out 'invalid' chatters
        let existing_logins: Vec<String> = fetched
            .iter()
            .map(|user| user.login.to_lowercase())
            .collect();

        let pre_filter_len = chatter_logins.len();
        chatter_logins.retain(|user| existing_logins.contains(&user.to_lowercase()));
        let removed_count = pre_filter_len - chatter_logins.len();

        if removed_count > 0 {
            tracing::warn!(
                removed_count,
                remaining_count = chatter_logins.len(),
                "filtered invalid chatter logins",
            );

            // TODO:
            //  perhaps we write filtered logins to a file to read this list of users
            //  easily??
            //      e.g:
            //     ```
            //     /var/log/piss-fan-server/[yyyy-mm-dd]_migrator_unknown-userlist.log
            //     ```
            //  .. or something
        } else {
            tracing::debug!("no invalid chatter logins found in cache");
        }

        // TODO: turn this block into a function call i reckon
        // --
        {
            let _span = tracing::debug_span!("sort_and_validate").entered();
            chatter_logins.sort_by_key(|a| a.to_lowercase());
            fetched.sort_by(|a, b| a.login.to_lowercase().cmp(&b.login.to_lowercase()));

            assert_eq!(chatter_logins.len(), fetched.len());

            // only check for complete alignment when debug mode, which will ideally catch any bugs
            // during development.
            //
            // otherwise, we do a quick 3-point index sample to validate this:
            //  * first element
            //  * middle element
            //  * last element
            if cfg!(debug_assertions) {
                for i in 0..chatter_logins.len() {
                    assert_eq!(
                        chatter_logins[i].to_lowercase(),
                        fetched[i].login.to_lowercase(),
                        "(at index {i}) alignment check failed"
                    );
                }
                tracing::debug!("validated chatter-login alignment");
            } else {
                let sample_indices = [0, chatter_logins.len() / 2, chatter_logins.len() - 1];
                for &i in &sample_indices {
                    if i < chatter_logins.len() {
                        assert_eq!(
                            chatter_logins[i].to_lowercase(),
                            fetched[i].login.to_lowercase(),
                            "(at index {i}) sample alignment check failed"
                        );
                    }
                }

                tracing::debug!("validated chatter-login alignment (sampled)");
            }
        }
        // --

        tracing::info!(
            fetched_count = fetched.len(),
            "matched chatters with respective user data"
        );

        // transform chatter structure + create db entries
        let chatters = Self::merge_chatters(&mut fetched, &chatter_logins).await?;

        chatter_repo.insert_many(&chatters).await?;
        tracing::info!(count = chatters.len(), "upsert chatters to database");

        // transform leaderboard structure + update db entries
        let scores = Self::merge_leaderboards(&fetched, &chatter_logins, &channel_map).await?;
        let total_scores: usize = scores.values().map(|s| s.len()).sum();
        tracing::info!(
            score_maps = scores.len(),
            total_scores,
            "merged leaderboard data"
        );

        Tx::with_tx(pool, |mut tx| async move {
            let result = async {
                for (chatter_id, scoremap) in scores.into_iter() {
                    for (channel_id, score) in scoremap.into_iter() {
                        tracing::trace!(
                            channel = channel_id,
                            "updating and recaculating channel score"
                        );
                        tracing::trace!(chatter = chatter_id, "updating chatter scoremap");
                        tx.update_score(
                            &chatter_id.clone().into(),
                            &channel_id.clone().into(),
                            score.into(),
                        )
                        .await?;

                        tx.recalculate_channel_total(&channel_id.into()).await?;
                        tx.recalculate_chatter_total(&chatter_id.clone().into())
                            .await
                            .unwrap();
                    }
                }

                Ok(())
            }
            .await;

            tracing::info!("cache migration pipeline complete");

            (tx, result)
        })
        .await?;

        Ok(())
    }

    #[instrument]
    pub async fn get_channel_keys() -> RedisResult<Vec<String>> {
        let mut conn = redis_pool().await?.manager.clone();
        let key_query = redis_key!(channel, score);
        tracing::info!(key = key_query, "built redis key");

        let keys_raw: Vec<String> = from_redis_value(conn.keys(key_query).await?)?;
        tracing::debug!(raw_key_count = keys_raw.len(), "retrieved raw channel keys");
        let mut processed_keys: Vec<_> = keys_raw
            .iter()
            .filter_map(|val| {
                val.split(':')
                    .nth(1)
                    .and_then(|s| s.split('#').nth(1))
                    .map(|s| s.to_owned())
            })
            .collect();

        if processed_keys.len() != keys_raw.len() {
            tracing::warn!(
                raw_count = keys_raw.len(),
                processed_count = processed_keys.len(),
                "partial key parse failure"
            );
        }

        processed_keys.sort_by_key(|a| a.to_lowercase());
        let pre_dedup = processed_keys.len();
        processed_keys.dedup_by(|a, b| a.to_lowercase().eq(&b.to_lowercase()));

        if pre_dedup != processed_keys.len() {
            tracing::debug!(
                removed_duplicates = pre_dedup - processed_keys.len(),
                "removed duplicate channel keys"
            );
        }

        tracing::info!(
            channel_key_count = processed_keys.len(),
            "processed channel keys"
        );
        Ok(processed_keys)
    }

    #[instrument]
    pub async fn get_chatter_keys() -> RedisResult<Vec<String>> {
        let mut conn = redis_pool().await?.manager.clone();
        let keys_raw: Vec<String> = from_redis_value(conn.keys(redis_key!(user, score)).await?)?;
        tracing::debug!(raw_key_count = keys_raw.len(), "retrieved raw chatter keys");

        let mut processed_keys: Vec<_> = keys_raw
            .iter()
            .filter_map(|val| val.split(':').nth(1).map(|s| s.to_owned()))
            .collect();

        if processed_keys.len() != keys_raw.len() {
            tracing::warn!(
                raw_count = keys_raw.len(),
                processed_count = processed_keys.len(),
                "partial key parse failure"
            );
        }

        processed_keys.sort_by_key(|a| a.to_lowercase());
        let pre_dedup = processed_keys.len();
        processed_keys.dedup_by(|a, b| a.to_lowercase().eq(&b.to_lowercase()));

        if pre_dedup != processed_keys.len() {
            tracing::debug!(
                removed_duplicates = pre_dedup - processed_keys.len(),
                "removed duplicate channel keys",
            );
        }

        tracing::info!(
            channel_key_count = processed_keys.len(),
            "processed chatter keys"
        );
        Ok(processed_keys)
    }

    #[instrument(skip(chatters, channel_map), fields(chatter_count = chatters.len(), channel_count = channel_map.len()))]
    pub async fn merge_leaderboards(
        chatters: &[HelixUser],
        redis_keys: &Vec<String>,
        channel_map: &HashMap<String, Channel>,
    ) -> RedisResult<HashMap<String, HashMap<String, i32>>> {
        let mut conn = redis_pool().await?.manager.clone();
        let mut pipeline = redis::pipe();

        redis_keys.iter().for_each(|chatter| {
            let key = redis_key!(user, leaderboard, chatter);
            pipeline.zrange_withscores(key, 0, -1);
        });

        tracing::debug!(query_count = redis_keys.len(), "built redis query pipeline");
        let leaderboards: Vec<Vec<String>> = pipeline.query_async(&mut conn).await?;
        tracing::debug!(
            result_count = leaderboards.len(),
            "retrieved leaderboard data"
        );

        let mut chatter_scores = HashMap::new();
        let mut total_scores = 0;
        let mut legacy_remaps = 0;
        let mut unknown_channels = 0;
        let mut empty_scoremaps = 0;

        for (i, scores) in leaderboards.into_iter().enumerate() {
            let mut mapped_scores = HashMap::new();
            // let mut should_update = HashSet::new();

            for score in scores.chunks_exact(2) {
                total_scores += 1;
                let channel_key = &score[0];
                let channel_login = channel_key
                    .split('#')
                    .nth(1)
                    .unwrap_or_else(|| {
                        tracing::warn!(channel_key, "invalid channel key format");
                        ""
                    })
                    .to_lowercase();

                // TODO:
                //  this block doesnt make sense what the fuck is going on here
                if let Some(channel_data) = &channel_map.get(&channel_login) {
                    if let Ok(score_value) = score[1].parse::<i32>() {
                        mapped_scores.insert(channel_data.id.to_string(), score_value);
                    } else {
                        tracing::warn!(
                            channel_key,
                            score_value = %score[1],
                            "failed to parse score value"
                        );
                    }
                } else {
                    let remapped_login = match &*channel_login {
                        "cchiko_" => "chikogaki".to_string(),
                        "pekoe_bunny" => "dearpekoe".to_string(),
                        "sheriff_baiken" => "baikenvt".to_string(),
                        "haelpc" => "netaccount".to_string(),

                        // unknown key (realistically should never match this arm!!)
                        _ => {
                            unknown_channels += 1;
                            tracing::error!(
                                chatter = %chatters[i].login,
                                channel_key,
                                "unknown channel in chatter leaderboard"
                            );

                            continue;
                        }
                    };

                    // --
                    if let Some(channel_data) = channel_map.get(&remapped_login) {
                        legacy_remaps += 1;
                        tracing::warn!(
                            chatter = %chatters[i].login,
                            old_key = channel_key,
                            new_login = %remapped_login,
                            "legacy channel in leaderboard"
                        );

                        if let Ok(score_value) = score[1].parse::<i32>() {
                            mapped_scores.insert(channel_data.id.to_string(), score_value);
                        }
                    } else {
                        tracing::error!(
                            chatter = %chatters[i].login,
                            channel_key,
                            attempted_remap = %remapped_login,
                            "legacy channel remap failure"
                        );
                    }
                }
            }

            if mapped_scores.is_empty() {
                empty_scoremaps += 1;
                tracing::warn!(chatter = %chatters[i].login, "chatter has empty scoremap");
            }

            // if !should_update.is_empty() {
            //     let update_count = should_update.len();
            //     tracing::debug!(
            //         chatter = %chatters[i].login,
            //         update_count,
            //         "updating legacy channel names"
            //     );
            //
            //     for (old, new) in should_update {
            //         if let Err(e) = Self::update_historic_channel(&old, &new).await {
            //             tracing::error!(
            //                 chatter = %chatters[i].login,
            //                 old_key = %old,
            //                 new_key = %new,
            //                 error = %e,
            //                 "failed to update legacy channel name"
            //             );
            //         }
            //     }
            // }

            chatter_scores.insert(chatters[i].id.to_string(), mapped_scores);
        }

        tracing::info!(
            chatter_count = chatter_scores.len(),
            total_scores,
            legacy_remaps,
            unknown_channels,
            empty_scoremaps,
            "merged leaderboards"
        );

        Ok(chatter_scores)
    }

    #[instrument(skip(broadcasters), fields(count = broadcasters.len()))]
    pub async fn merge_channels(broadcasters: Vec<HelixUser>) -> RedisResult<Vec<HelixUser>> {
        let num_keys = broadcasters.len();
        tracing::debug!("building redis pipeline for channel totals");

        let mut conn = redis_pool().await?.manager.clone();
        let mut pipeline = redis::pipe();
        broadcasters.iter().for_each(|ch| {
            let total_key = redis_key!(channel, score, &ch.login);
            pipeline.get(total_key);
        });

        let res: Vec<String> = pipeline.query_async(&mut conn).await?;
        tracing::debug!(
            retrieved_count = res.len(),
            "retrieved cached channel totals"
        );

        let mut parse_failures = 0;
        let processed: Vec<_> = broadcasters
            .into_iter()
            .enumerate()
            .map(|(i, mut chan)| {
                match res[i].parse::<i64>() {
                    Ok(total) => chan.total = total,
                    Err(e) => {
                        parse_failures += 1;
                        tracing::warn!(
                            channel  =%chan.login,
                            value = %res[i],
                            error = %e,
                            "failed to parse channel_total, falling back to '0'"
                        );
                        chan.total = 0;
                    }
                }

                chan
            })
            .collect();

        tracing::info!(
            processed_count = processed.len(),
            total_requested = num_keys,
            parse_failures,
            "merged channel data"
        );

        Ok(processed)
    }

    #[instrument(skip(users, redis_keys), fields(count = users.len()))]
    pub async fn merge_chatters(
        users: &mut [HelixUser],
        redis_keys: &[String],
    ) -> RedisResult<Vec<Chatter>> {
        tracing::debug!("building redis pipeline for chatter totals");

        let mut conn = redis_pool().await?.manager.clone();
        let mut pipeline = redis::pipe();
        redis_keys.iter().for_each(|user| {
            let total_key = redis_key!(user, total, user); // format!("user:{}:total", user);
            pipeline.get(total_key);
        });

        let res: Vec<redis::Value> = pipeline.query_async(&mut conn).await?;
        tracing::debug!(
            retrieved_count = res.len(),
            "retrieved cached chatter totals"
        );

        let mut parse_failures = Vec::new();
        let processed: Vec<_> = users
            .iter_mut()
            .enumerate()
            .map(|(i, user)| {
                match from_redis_value::<String>(res[i].clone()) {
                    Ok(s) => match s.parse::<i64>() {
                        Ok(total) => user.total = total,
                        Err(e) => {
                            tracing::warn!(
                                user = %user.login,
                                value = %s,
                                error = %e,
                                "chatter parse failure on total"
                            );

                            parse_failures.push(user.login.clone());
                            user.total = 0;
                        }
                    },
                    Err(e) => {
                        tracing::warn!(
                            user = %user.login,
                            error = ?e,
                            "cached chatter deserialization failure on total",
                        );
                        parse_failures.push(user.login.clone());
                        user.total = 0;
                    }
                };
                user.to_owned()
            })
            .map(Chatter::from)
            .collect();

        tracing::info!(
            processed_count = processed.len(),
            parse_failures = parse_failures.len(),
            "merged chatter data",
        );

        if !parse_failures.is_empty() && parse_failures.len() < 100 {
            tracing::warn!(failed_users = ?parse_failures, "failed to parse totals for some users");
        } else if !parse_failures.is_empty() {
            tracing::warn!(
                failed_count = parse_failures.len(),
                failed_sample = ?&parse_failures[..5.min(parse_failures.len())],
                "failed to parse totals for a signficant number of users"
            );
        }

        Ok(processed)
    }

    #[instrument(skip(old_login, new_login))]
    /// Pipeline for copying "stale" cached data from old keys to new keys on a user's cached information
    ///
    /// # Redis
    ///
    /// * channel & chatter totals:
    /// ```
    /// COPY [source_key] [dest_key]
    /// ```
    ///
    /// * channel & chatter leaderboards
    /// ```
    /// ZUNIONSTORE [dest_key] 1 [source_key]
    /// ```
    ///
    /// # Additional note
    ///
    /// Unsure whether we actually care about this even slightly if we are
    ///  - migrating storage from Redis to Postgres,
    ///  - using the user's ID over their login
    pub async fn update_historic_channel(old_login: &str, new_login: &str) -> RedisResult<()> {
        tracing::debug!(
            old_login,
            new_login,
            "updating cached redis keys for legacy channel"
        );

        let old_channel_total = redis_key!(channel, score, old_login);
        let old_channel_lb = redis_key!(channel, leaderboard, old_login);
        let old_user_total = redis_key!(user, score, old_login);
        let old_user_lb = redis_key!(user, leaderboard, old_login);

        let new_channel_total = redis_key!(channel, score, new_login);
        let new_channel_lb = redis_key!(channel, leaderboard, new_login);
        let new_user_total = redis_key!(user, score, new_login);
        let new_user_lb = redis_key!(user, leaderboard, new_login);

        let mut conn = redis_pool().await?.manager.clone();
        let mut pipeline = redis::pipe();
        let copy_opts = CopyOptions::default().replace(false);

        pipeline.copy(old_channel_total, new_channel_total, copy_opts);
        pipeline.copy(old_user_total, new_user_total, copy_opts);

        pipeline.zinterstore(new_channel_lb, old_channel_lb);
        pipeline.zinterstore(new_user_lb, old_user_lb);

        let () = pipeline.query_async(&mut conn).await?;

        tracing::info!("updated cached keys");
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aliases {
    pub current: String,
    pub historic: Vec<String>,
}

impl Aliases {
    pub fn new(current: String, historic: Vec<String>) -> Self {
        Self { current, historic }
    }
}

// TODO:
//  this should be implemtned with `update_historic_user` as a single function,
//  however i am too lazy and probably wont ever do this.

#[instrument(skip(aliases))]
/// Updates a channel's score with historic data (required when a channel changes their name).
///
/// The aliases and current login strings for this function should NOT contain the leading `#` as user data
/// must be updated before the channel data is updated.
///
///     // e.g.:
///     //  * [x] `#sleepiebug`   (incorrect)
///     //  * [o]  `sleepiebug`    (correct)
pub async fn update_historic_channel(aliases: Aliases) -> RedisResult<()> {
    tracing::info!(
        current_login = aliases.current,
        historic_keys_count = aliases.historic.len(),
        "merging values for channel with historic data"
    );

    // `redis_key!(channel, ...)` should auto-prepend the `#` for us
    let current_total_key = redis_key!(channel, score, &aliases.current);
    let current_leaderboard_key = redis_key!(channel, leaderboard, &aliases.current);
    let mut updated_leaderboard_map: HashMap<String, isize> = HashMap::new();
    let mut conn = redis_pool().await?.manager.clone();

    let mut current_total: isize = conn
        .get::<_, Option<isize>>(&current_total_key)
        .await?
        .unwrap_or_default();

    let initial_total = current_total;
    let current_leaderboard: Vec<(String, isize)> = conn
        .zrange_withscores::<_, Option<Vec<(String, isize)>>>(&current_leaderboard_key, 0, -1)
        .await?
        .unwrap_or_default();

    current_leaderboard.iter().for_each(|(chatter, score)| {
        updated_leaderboard_map.insert(chatter.clone(), *score);
    });

    tracing::debug!(current_total, ?current_leaderboard, "found current data");

    for (i, alias) in aliases.historic.iter().enumerate() {
        let historic_total_key = redis_key!(channel, score, &alias);
        let historic_leaderboard_key = redis_key!(channel, leaderboard, &alias);
        if let Some(total) = conn.get::<_, Option<isize>>(&historic_total_key).await? {
            tracing::debug!(
                prev = current_total,
                updated = (current_total + total),
                "adding score"
            );
            current_total += total;
        } else {
            tracing::warn!(index = i, alias, "skipping uncached alias");
            continue;
        }

        let historic_leaderboard: Vec<(String, isize)> = conn
            .zrange_withscores::<_, Option<Vec<(String, isize)>>>(&historic_leaderboard_key, 0, -1)
            .await?
            .unwrap_or_default();

        tracing::debug!(
            historic_name = alias,
            ?historic_leaderboard,
            "merging historic leaderboard data"
        );

        historic_leaderboard
            .into_iter()
            .for_each(|(chatter, score)| {
                updated_leaderboard_map
                    .entry(chatter)
                    .and_modify(|total| *total += score)
                    .or_insert(score);
            });

        tracing::warn!(alias, "removing historic channel keys");
        conn.del::<_, ()>(&historic_total_key).await?;
        conn.del::<_, ()>(&historic_leaderboard_key).await?;
    }

    if current_total == 0 {
        tracing::error!(aliases.current, historic = ?aliases.historic, "empty dataset");
        return Err(RedisErr::UpdateEmpty);
    }

    tracing::info!(
        ?updated_leaderboard_map,
        initial_total,
        current_total,
        current_login = aliases.current,
        "writing merged data"
    );

    conn.set::<&String, isize, ()>(&current_total_key, current_total)
        .await?;
    let mut pipeline = redis::pipe();
    updated_leaderboard_map
        .into_iter()
        .for_each(|(chatter, score)| {
            pipeline.zadd(&current_leaderboard_key, chatter, score);
        });

    let () = pipeline.query_async(&mut conn).await?;
    tracing::info!(current_login = aliases.current, "channel merge complete");

    Ok(())
}

#[instrument(skip(aliases))]
pub async fn update_historic_user(aliases: Aliases) -> RedisResult<()> {
    tracing::info!(
        current_login = aliases.current,
        historic_keys_count = aliases.historic.len(),
        "merging values for chatter with historic data"
    );

    let current_total_key = redis_key!(user, score, &aliases.current);
    let current_leaderboard_key = redis_key!(user, leaderboard, &aliases.current);
    let mut updated_leaderboard_map: HashMap<String, isize> = HashMap::new();
    let mut conn = redis_pool().await?.manager.clone();

    let mut current_total: isize = conn
        .get::<_, Option<isize>>(&current_total_key)
        .await?
        .unwrap_or_default();

    let initial_total = current_total;
    let current_leaderboard: Vec<(String, isize)> = conn
        .zrange_withscores::<_, Option<Vec<(String, isize)>>>(&current_leaderboard_key, 0, -1)
        .await?
        .unwrap_or_default();

    current_leaderboard.iter().for_each(|(channel, score)| {
        updated_leaderboard_map.insert(channel.clone(), *score);
    });

    tracing::debug!(current_total, ?current_leaderboard, "found current data");

    for (i, alias) in aliases.historic.iter().enumerate() {
        let historic_total_key = redis_key!(user, score, &alias);
        let historic_leaderboard_key = redis_key!(user, leaderboard, &alias);
        if let Some(total) = conn.get::<_, Option<isize>>(&historic_total_key).await? {
            tracing::debug!(
                prev = current_total,
                additional = total,
                updated = (current_total + total),
                "adding score"
            );
            current_total += total;
        } else {
            tracing::warn!(index = i, alias, "skipping uncached alias");
            continue;
        }

        let historic_leaderboard: Vec<(String, isize)> = conn
            .zrange_withscores::<_, Option<Vec<(String, isize)>>>(&historic_leaderboard_key, 0, -1)
            .await?
            .unwrap_or_default();

        tracing::debug!(
            historic_name = alias,
            ?historic_leaderboard,
            "merging historic leaderboard data"
        );

        historic_leaderboard
            .into_iter()
            .for_each(|(channel, score)| {
                updated_leaderboard_map
                    .entry(channel)
                    .and_modify(|total| *total += score)
                    .or_insert(score);
            });

        tracing::warn!(alias, "removing historic user keys");
        conn.del::<_, ()>(&historic_total_key).await?;
        conn.del::<_, ()>(&historic_leaderboard_key).await?;
    }

    if current_total == 0 {
        tracing::error!(aliases.current, historic = ?aliases.historic, "empty dataset");
        return Err(RedisErr::UpdateEmpty);
    }

    tracing::info!(
        ?updated_leaderboard_map,
        initial_total,
        current_total,
        current_login = aliases.current,
        "writing merged data"
    );

    conn.set::<&String, isize, ()>(&current_total_key, current_total)
        .await?;
    let mut pipeline = redis::pipe();
    updated_leaderboard_map
        .into_iter()
        .for_each(|(channel, score)| {
            pipeline.zadd(&current_leaderboard_key, channel, score);
        });

    let () = pipeline.query_async(&mut conn).await?;
    tracing::info!(current_login = aliases.current, "user merge complete");

    Ok(())
}

#[instrument]
pub async fn migrate_redis_into_pg() -> RedisResult<()> {
    Migrator::new().process().await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::telemetry;

    #[tokio::test]
    async fn test_historic_updater() {
        let provider = telemetry::Telemetry::new().await.unwrap().register();

        let user = String::from("cowtitties__");
        let historic = vec![String::from("cowtitties_"), String::from("cowtitties")];

        let aliases = Aliases::new(user, historic);

        update_historic_user(aliases).await.unwrap();

        provider.shutdown();
    }

    #[tokio::test]
    /// Technically this is not a test, but rather can be run manually as a one-off to fix
    /// out-of-date channel or chatter names:
    /// ```
    /// cargo test run_updater -- --show-output
    /// ```
    ///
    /// # Note
    ///
    /// This would be good to serve as an endpoint so we can just update without rebuilding,
    /// but it needs to be more robust before this should be implemented.
    async fn run_updater() {
        let provider = telemetry::Telemetry::new().await.unwrap().register();

        // [
        //      "old_name_1", "new_name_1",
        //      "old_name_2", "new_name_2",
        //      ...
        //  ];

        let names_map = Vec::new();
        for update in names_map.chunks_exact(2) {
            tracing::info!("processing: {} -> {}", update[0], update[1]);
            Migrator::update_historic_channel(update[0], update[1])
                .await
                .unwrap();
        }

        Migrator::new().process().await.unwrap();

        provider.shutdown();

        // let mut conn = redis_pool().await.unwrap().manager.clone();
        // let mut pipeline = redis::pipe();
        //
        // for pairs in names_map.chunks_exact(2) {
        //     pipeline.del(&format!("user:{}:total", pairs[0]));
        //     pipeline.del(&format!("user:{}:leaderboard", pairs[0]));
        //     pipeline.del(&format!("channel:#{}:total", pairs[0]));
        //     pipeline.del(&format!("channel:#{}:leaderboard", pairs[0]));
        // }
        //
        // let res: redis::Value = pipeline.query_async(&mut conn).await.unwrap();

        // info!(
        //     "successfully updated {:?} names and deleted corresponding old keys",
        //     res
        // );
    }
}
