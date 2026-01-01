use std::collections::{HashMap, HashSet};
use std::ops::AddAssign;

use redis::{AsyncCommands, CopyOptions, from_redis_value};
use tracing::{error, info, instrument, warn};

use super::redis_pool::RedisResult;
use crate::db::models::{Channel, Chatter, DbUser, Score};
use crate::db::pg::db_pool;
use crate::db::redis::redis_pool::redis_pool;
use crate::util::helix::{Helix, HelixUser};

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
    pub async fn preprocess(&mut self) -> RedisResult<()> {
        tracing::info!("begin preprocess pipeline");

        let mut channel_logins = {
            let _span = tracing::info_span!("fetch_channel_keys").entered();
            Self::get_channel_keys().await?
        };

        tracing::info!(
            channel_count = channel_logins.len(),
            "retrieved channel keys from redis"
        );

        // process chatters with tracked channels ('broadcasters')
        let (channels, broadcasters) = {
            let _span =
                tracing::info_span!("fetch_channel_data", channel_count = channel_logins.len())
                    .entered();

            let fetched = Helix::fetch_users_by_login(channel_logins.clone()).await?;
            tracing::info!(
                fetched_count = fetched.len(),
                requested_count = channel_logins.len(),
                "fetched channel data from helix"
            );

            (
                Self::merge_channels(fetched.clone()).await?,
                fetched.into_iter().map(Chatter::from).collect::<Vec<_>>(),
            )
        };

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

        {
            // chatter upsert nested span
            let _span =
                tracing::info_span!("upsert_broadcasters", count = channels.len()).entered();

            Chatter::upsert_multi(pool, &broadcasters).await?;
            tracing::info!(
                count = broadcasters.len(),
                "upsert broadcasters to database"
            );
        }

        {
            // channel upsert nested span
            let _span = tracing::info_span!("upsert_channels", count = channels.len()).entered();

            Channel::upsert_multi(pool, &channels).await?;
            tracing::info!(count = channels.len(), "upsert channels to database");
        }

        // fetch and process the non-broadcaster chatters
        let mut chatter_logins = {
            let _span = tracing::info_span!("fetch_chatter_keys").entered();
            Self::get_chatter_keys().await?
        };

        let num_chatters = chatter_logins.len();
        tracing::info!(num_chatters, "retrieved chatter keys from redis");

        let mut fetched = {
            let _span =
                tracing::info_span!("fetch_chatter_data", chatter_count = num_chatters).entered();
            Helix::fetch_users_by_login(chatter_logins.clone()).await?
        };

        // filter out 'invalid' chatters
        let existing_logins: Vec<String> = fetched
            .iter()
            .map(|user| user.login.to_lowercase())
            .collect();

        let pre_filter = chatter_logins.len();
        chatter_logins.retain(|user| existing_logins.contains(&user.to_lowercase()));
        let removed_count = pre_filter - chatter_logins.len();

        if removed_count > 0 {
            tracing::warn!(
                removed_count,
                remaining_count = chatter_logins.len(),
                "filtered invalid chatter logins",
            );
        } else {
            tracing::debug!("no invalid chatter logins found in cache");
        }

        {
            let _span = tracing::debug_span!("sort_and_validate").entered();
            chatter_logins.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
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
                        "(at index {}) alignment check failed",
                        i
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
                            "(at index {}) sample alignment check failed",
                            i
                        );
                    }
                }

                tracing::debug!("validated chatter-login alignment (sampled)");
            }
        }

        tracing::info!(
            fetched_count = fetched.len(),
            "matched chatters with respective user data"
        );

        // transform chatter structure + create db entries
        let chatters = {
            let _span = tracing::info_span!("merge_chatters", count = fetched.len()).entered();
            Self::merge_chatters(&mut fetched, &chatter_logins).await?
        };

        {
            let _span = tracing::info_span!("upsert_chatters", count = chatters.len()).entered();
            Chatter::upsert_multi(pool, &chatters).await?;
            tracing::info!(count = chatters.len(), "upsert chatters to database");
        }

        // transform leaderboard structure + update db entries
        let scores = {
            let _span = tracing::info_span!(
                "merge_leaderboards",
                chatter_count = fetched.len(),
                channel_count = channel_map.len()
            )
            .entered();
            Self::merge_leaderboards(&fetched, &chatter_logins, &channel_map).await?
        };

        let total_scores: usize = scores.values().map(|s| s.len()).sum();
        tracing::info!(
            score_maps = scores.len(),
            total_scores,
            "merged leaderboard data"
        );

        {
            let _span =
                tracing::info_span!("update_scores", score_maps = scores.len(), total_scores)
                    .entered();

            Score::update_multi(pool, scores).await?;
            tracing::info!("updated scores in database");
        }

        tracing::info!("preprocessing pipeline completed successfully");
        Ok(())
    }

    #[instrument]
    pub async fn get_channel_keys() -> RedisResult<Vec<String>> {
        let mut conn = redis_pool().await?.manager.clone();
        let keys_raw: Vec<String> = {
            let _span = tracing::debug_span!("redis_keys_query").entered();
            from_redis_value(conn.keys("channel:*:total").await?)?
        };

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

        // let mut processed_keys: Vec<_> = keys_raw
        //     .iter()
        //     .map(|val| {
        //         let b = val
        //             .split(':')
        //             .nth(1)
        //             .expect("key format error (on ':')")
        //             .split('#')
        //             .nth(1)
        //             .expect("key format error (on '#')");
        //         b.to_owned()
        //     })
        //     .collect();

        if processed_keys.len() != keys_raw.len() {
            tracing::warn!(
                raw_count = keys_raw.len(),
                processed_count = processed_keys.len(),
                "partial key parse failure"
            );
        }

        processed_keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
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
        let keys_raw: Vec<String> = {
            let _span = tracing::debug_span!("redis_keys_query").entered();
            from_redis_value(conn.keys("user:*:total").await?)?
        };

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

        processed_keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
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
        chatters: &Vec<HelixUser>,
        redis_keys: &Vec<String>,
        channel_map: &HashMap<String, Channel>,
    ) -> RedisResult<HashMap<String, HashMap<String, i32>>> {
        let mut conn = redis_pool().await?.manager.clone();
        let mut pipeline = redis::pipe();

        {
            let _span = tracing::debug_span!("build_pipeline").entered();
            for chatter in redis_keys {
                let chatter_lb_key = format!("user:{}:leaderboard", chatter);
                pipeline.zrange_withscores(chatter_lb_key, 0, -1);
            }

            tracing::debug!(query_count = redis_keys.len(), "built redis pipeline");
        }

        let lbs: Vec<Vec<String>> = {
            let _span = tracing::debug_span!("execute_pipeline").entered();
            pipeline.query_async(&mut conn).await?
        };

        tracing::debug!(result_count = lbs.len(), "retrieved leaderboard data");

        let mut chatter_scores = HashMap::new();
        let mut total_scores = 0;
        let mut legacy_remaps = 0;
        let mut unknown_channels = 0;
        let mut empty_scoremaps = 0;

        for (i, scores) in lbs.into_iter().enumerate() {
            let mut mapped_scores = HashMap::new();
            let mut should_update = HashSet::new();

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

                if let Some(channel_data) = &channel_map.get(&channel_login) {
                    if let Ok(score_value) = score[1].parse::<i32>() {
                        mapped_scores.insert(channel_data.id.to_owned(), score_value);
                    } else {
                        tracing::warn!(
                            channel_key,
                            score_value = %score[1],
                            "failed to parse score value"
                        );
                    }
                } else {
                    let mut remapped_login = channel_key.clone();
                    match channel_login.as_str() {
                        "#cchiko_" => remapped_login = "chikogaki".to_string(),
                        "#pekoe_bunny" => remapped_login = "dearpekoe".to_string(),
                        "#sheriff_baiken" => remapped_login = "baikenvt".to_string(),
                        _ => {
                            unknown_channels += 1;
                            tracing::error!(
                                chatter = %chatters[i].login,
                                channel_key,
                                "unknown channel in leaderboard"
                            );

                            continue;
                        }
                    };

                    if let Some(channel_data) = channel_map.get(&channel_login) {
                        legacy_remaps += 1;
                        tracing::warn!(
                            chatter = %chatters[i].login,
                            old_key = channel_key,
                            new_login = %remapped_login,
                            "legacy channel in leaderboard"
                        );
                        should_update.insert((score[0].clone(), channel_login.clone()));

                        if let Ok(score_value) = score[1].parse::<i32>() {
                            mapped_scores.insert(channel_data.id.to_owned(), score_value);
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

            if !should_update.is_empty() {
                let update_count = should_update.len();
                tracing::debug!(
                    chatter = %chatters[i].login,
                    update_count,
                    "updating legacy channel names"
                );

                for (old, new) in should_update {
                    if let Err(e) = Self::update_cached_name(&old, &new).await {
                        tracing::error!(
                            chatter = %chatters[i].login,
                            old_key = %old,
                            new_key = %new,
                            error = %e,
                            "failed to update legacy channel name"
                        );
                    }
                }
            }

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

        for ch in &broadcasters {
            let total_key = format!("channel:#{}:total", ch.login);
            pipeline.get(total_key);
        }

        let res: Vec<String> = {
            let _span = tracing::debug_span!("execute_pipeline").entered();
            pipeline.query_async(&mut conn).await?
        };

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
                            "failed to parse channel total (using default=0)"
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
        users: &mut Vec<HelixUser>,
        redis_keys: &Vec<String>,
    ) -> RedisResult<Vec<Chatter>> {
        tracing::debug!("building redis pipeline for chatter totals");

        let mut conn = redis_pool().await?.manager.clone();
        let mut pipeline = redis::pipe();
        redis_keys.iter().for_each(|user| {
            let total_key = format!("user:{}:total", user);
            pipeline.get(total_key);
        });

        let res: Vec<redis::Value> = {
            let _span = tracing::debug_span!("execute_pipeline").entered();
            pipeline.query_async(&mut conn).await?
        };

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
                                "chatter total parse failure"
                            );

                            parse_failures.push(user.login.clone());
                            user.total = 0;
                        }
                    },
                    Err(e) => {
                        tracing::warn!(
                            user = %user.login,
                            error = ?e,
                            "cached chatter total deserialization failure",
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

        if !parse_failures.is_empty() && parse_failures.len() < 10 {
            tracing::debug!(failed_users = ?parse_failures, "failed to parse totals for users");
        } else if !parse_failures.is_empty() {
            tracing::debug!(
                failed_count = parse_failures.len(),
                sample = ?&parse_failures[..5.min(parse_failures.len())],
                "failed to parse large number of users"
            );
        }

        Ok(processed)
    }

    #[instrument(skip(old_login, new_login))]
    /// pipeline for redis COPY operations on a user's cached information:
    ///
    /// * channel & chatter totals:
    /// ```redis
    /// COPY [source_key] [dest_key]
    /// ```
    ///
    /// * channel & chatter leaderboards
    /// ```redis
    /// ZUNIONSTORE [dest_key] 1 [source_key]
    /// ```
    pub async fn update_cached_name(old_login: &str, new_login: &str) -> RedisResult<()> {
        tracing::debug!("updating cached redis keys for legacy channel");

        let old_channel_total = format!("channel:#{}:total", old_login);
        let old_channel_lb = format!("channel:#{}:leaderboard", old_login);
        let old_user_total = format!("user:{}:total", old_login);
        let old_user_lb = format!("user:{}:leaderboard", old_login);

        let new_channel_total = format!("channel:#{}:total", new_login);
        let new_channel_lb = format!("channel:#{}:leaderboard", new_login);
        let new_user_total = format!("user:{}:total", new_login);
        let new_user_lb = format!("user:{}:leaderboard", new_login);

        let mut conn = redis_pool().await?.manager.clone();
        let mut pipeline = redis::pipe();
        let copy_opts = CopyOptions::default().replace(true);

        pipeline.copy(old_channel_total, new_channel_total, copy_opts);
        pipeline.copy(old_user_total, new_user_total, copy_opts);

        pipeline.zinterstore(new_channel_lb, old_channel_lb);
        pipeline.zinterstore(new_user_lb, old_user_lb);

        let (): _ = {
            let _span = tracing::debug_span!("execute_pipeline").entered();
            pipeline.query_async(&mut conn).await?
        };

        tracing::info!("updated cached keys");
        Ok(())
    }
}

// TODO:
//  the below is really only needed as a one-off.

#[cfg(test)]
mod test {
    use opentelemetry::trace::Tracer;

    use super::*;

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
        let provider = crate::util::tracing::build_subscriber().await.unwrap();

        // [
        //      "old_name_1", "new_name_1",
        //      "old_name_2", "new_name_2",
        //      ...
        //  ];

        let names_map = Vec::new();
        for update in names_map.chunks_exact(2) {
            info!("processing: {} -> {}", update[0], update[1]);
            Migrator::update_cached_name(update[0], update[1])
                .await
                .unwrap();
        }

        let _migrator = Migrator::new().preprocess().await.unwrap();

        crate::util::tracing::destroy_tracer(provider);

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
