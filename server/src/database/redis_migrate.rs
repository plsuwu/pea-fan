//! Some of this needs ironing out but its kind of a one-off to update the schema

use std::collections::HashMap;

use redis::{AsyncCommands, from_redis_value};
use tracing::{debug, info, instrument, trace, warn};

use crate::database::redis::{NOT_PRESENT_IN_CACHE, RedisPoolResult, redis_pool};
use crate::database::schema::{Channel, Chatter, Score};
use crate::util::helix::{Helix, InternalUser};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Migrator {
    pub channels: Vec<Channel>,
    pub chatters: Vec<Chatter>,
    pub scores: Vec<i32>,
}

impl Migrator {
    pub fn new() -> Self {
        let channels = Vec::new();
        let chatters = Vec::new();
        let scores = Vec::new();

        Self {
            channels,
            chatters,
            scores,
        }
    }

    pub async fn preprocess(&mut self) -> RedisPoolResult<()> {
        let mut channel_logins = Self::get_channel_keys().await?;
        let (channels, broadcasters) = {
            let fetched = Helix::fetch_user_by_login(&mut channel_logins).await?;
            (Self::merge_channels(fetched.clone()).await?, fetched)
        };

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

        // insert channels
        Chatter::bulk_upsert(&broadcasters).await?;
        Channel::bulk_upsert(&channels).await?;

        let mut chatter_logins = Self::get_chatter_keys().await?;
        info!("found {} keys for chatters", chatter_logins.len());

        let mut fetched = Helix::fetch_user_by_login(&mut chatter_logins).await?;

        // filter out chatters we couldn't find
        let existing_logins: Vec<String> = fetched.iter().map(|user| user.login.clone()).collect();
        chatter_logins.retain(|user| existing_logins.contains(&user.to_lowercase()));

        // correct ordering of fetched data and verify ordering of redis keys match
        // ordering of chatter data
        chatter_logins.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        fetched.sort_by(|a, b| a.login.cmp(&b.login));
        warn!("{:#?} | {}", fetched[0], chatter_logins[0].to_lowercase());
        warn!("{:#?} | {}", fetched[8], chatter_logins[8].to_lowercase());
        warn!(
            "{:#?} | {}",
            fetched[444],
            chatter_logins[444].to_lowercase()
        );

        info!("got {} chatter results from helix", fetched.len());

        assert_eq!(fetched[0].login, chatter_logins[0].to_lowercase());
        assert_eq!(fetched[8].login, chatter_logins[8].to_lowercase());
        assert_eq!(fetched[182].login, chatter_logins[182].to_lowercase());
        assert_eq!(fetched[444].login, chatter_logins[444].to_lowercase());

        let chatters = { Self::merge_chatters(&mut fetched, &chatter_logins).await? };
        Chatter::bulk_upsert(&chatters).await?;

        let scores = { Self::merge_leaderboards(&fetched, &chatter_logins, &channel_map).await? };
        Score::bulk_update(scores).await?;

        Ok(())
    }

    pub async fn merge_leaderboards(
        chatters: &Vec<InternalUser>,
        redis_keys: &Vec<String>,
        channel_map: &HashMap<String, Channel>,
    ) -> RedisPoolResult<HashMap<String, HashMap<String, i32>>> {
        let mut conn = redis_pool().await?.manager.clone();
        let mut pipeline = redis::pipe();

        for chatter in redis_keys {
            warn!("getting chatter data: {}", chatter);

            let chatter_leaderboard_key = format!("user:{}:leaderboard", chatter);
            pipeline.zrange_withscores(chatter_leaderboard_key, 0, -1);
        }

        let leaderboards: Vec<Vec<String>> = pipeline.query_async(&mut conn).await?;
        let mut user_score_map = HashMap::new();
        for (idx, scores) in leaderboards.into_iter().enumerate() {
            let mut mapped_scores = HashMap::new();

            for score in scores.chunks_exact(2) {
                let channel_id = &channel_map
                    .get(&score[0].split('#').nth(1).unwrap().to_lowercase())
                    .unwrap()
                    .id;

                mapped_scores.insert(channel_id.to_owned(), score[1].parse::<i32>().unwrap());
            }

            user_score_map.insert(chatters[idx].id.to_string(), mapped_scores);
        }

        trace!("LEADERBOARD OUTPUT: {:?}", user_score_map);
        Ok(user_score_map)
    }

    pub async fn get_channel_keys() -> RedisPoolResult<Vec<String>> {
        let mut conn = redis_pool().await?.manager.clone();
        let channel_keys_raw: Vec<String> = from_redis_value(&conn.keys("channel:*:total").await?)?;

        Ok(channel_keys_raw
            .iter()
            .map(|val| {
                let broadcaster = val.split(':').nth(1).unwrap().split('#').nth(1).unwrap();
                broadcaster.to_string()
            })
            .collect())
    }

    pub async fn get_chatter_keys() -> RedisPoolResult<Vec<String>> {
        let mut conn = redis_pool().await?.manager.clone();
        let chatter_keys_raw: Vec<String> = from_redis_value(&conn.keys("user:*:total").await?)?;

        Ok(chatter_keys_raw
            .iter()
            .map(|val| {
                let chatter = val.split(':').nth(1).unwrap();
                chatter.to_string()
            })
            .collect())
    }

    #[instrument(skip(broadcasters))]
    pub async fn merge_channels(
        broadcasters: Vec<InternalUser>,
    ) -> RedisPoolResult<Vec<InternalUser>> {
        let num_keys = broadcasters.len();
        info!("preprocessing: {} channels to transform", num_keys);

        let channels = Self::remap_channels(broadcasters).await?;

        info!(
            "preprocessing: transformed {} of {} keyed channels",
            channels.len(),
            num_keys
        );

        Ok(channels)
    }

    #[instrument(skip(users))]
    pub async fn merge_chatters(
        users: &mut Vec<InternalUser>,
        redis_keys: &Vec<String>,
    ) -> RedisPoolResult<Vec<InternalUser>> {
        let mut users = users.to_owned();
        let num_keys = users.len();

        info!("preprocessing: {} chatters to transform", num_keys);

        let chatters = Self::remap_chatters(&mut users, redis_keys).await?;

        info!(
            "preprocessing: transformed {} of {} keyed chatters",
            chatters.len(),
            num_keys
        );

        Ok(chatters)
    }

    #[instrument(skip(users))]
    pub async fn remap_chatters(
        users: &mut Vec<InternalUser>,
        redis_keys: &Vec<String>,
    ) -> RedisPoolResult<Vec<InternalUser>> {
        let mut conn = redis_pool().await?.manager.clone();
        let users_len = users.len();

        let mut pipeline = redis::pipe();
        redis_keys.iter().for_each(|user| {
            let user_total_key = format!("user:{}:total", user);
            pipeline.get(user_total_key);
        });

        let user_query_value: Vec<redis::Value> = pipeline.query_async(&mut conn).await?;

        let mut failed = Vec::new();
        let fmt: Vec<_> = users
            .iter_mut()
            .enumerate()
            .map(|(idx, user)| {
                match from_redis_value::<String>(&user_query_value[idx]) {
                    Ok(s) => user.total = s.parse::<i32>().unwrap_or(0),
                    _ => {
                        failed.push(user.clone());
                    }
                };

                user.to_owned()
            })
            .collect();

        tracing::error!(NOT_PRESENT_IN_CACHE);
        warn!("{:#?}\n", failed);

        debug!(
            "retrieved {} of {} chatters from redis cache",
            fmt.len(),
            users_len,
        );

        trace!("{:?}", users);
        Ok(fmt)
    }

    pub async fn remap_channels(channels: Vec<InternalUser>) -> RedisPoolResult<Vec<InternalUser>> {
        let mut conn = redis_pool().await?.manager.clone();
        let mut pipeline = redis::pipe();

        for ch in &channels {
            let channel_total_key = format!("channel:#{}:total", ch.login);
            pipeline.get(channel_total_key);
        }

        let res_channel: Vec<String> = pipeline.query_async(&mut conn).await?;
        trace!("channels from redis: {:?}", res_channel);
        info!("retrieved {} channels from redis", res_channel.len());

        let fmt: Vec<_> = channels
            .into_iter()
            .enumerate()
            .map(|(idx, mut chan)| {
                // i dont think the `or` condition for unwrapping here
                // should ever be hit (?)
                chan.total = res_channel[idx].parse().unwrap_or(0);
                chan
            })
            .collect();

        trace!("transformed channels: {:?}", fmt);
        info!(
            "transformed {} (of {}) channels to the new format",
            res_channel.len(),
            fmt.len()
        );
        Ok(fmt)
    }
}
