use core::fmt;
use std::{env, sync::LazyLock};

use crate::ws::client::{CacheCounter, WsClientResult};
use async_trait::async_trait;
use redis::{AsyncCommands, Value, aio::ConnectionManager};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

const CANNOT_DEBUG: &str = "Debug called on ConnectionManager";
static REDIS_CONNECTION_POOL: LazyLock<OnceCell<RedisPool>> = LazyLock::new(OnceCell::new);
pub type RedisPoolResult<T> = Result<T, redis::RedisError>;

/// Retrieve a Redis connection handle from a static client pool
pub async fn redis_pool() -> RedisPoolResult<&'static RedisPool> {
    REDIS_CONNECTION_POOL
        .get_or_try_init(|| async {
            RedisPool::new(
                &env::var("REDIS_URL").unwrap_or_else(|_| "redis:://127.0.0.1:6380".into()),
            )
            .await
        })
        .await
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedisResponse {}

pub trait Key {
    fn new(name: &str) -> Self;
}

#[derive(Debug)]
pub struct ChatterKeys {
    total: String,
    leaderboard: String,
    image: String,
    redact: String,
    prev_helix_fetch: String,
}

impl Key for ChatterKeys {
    fn new(name: &str) -> Self {
        Self {
            total: format!("user:{}:total", name),
            leaderboard: format!("user:{}:leaderboard", name),
            image: format!("user:{}:image", name),
            redact: format!("user:{}:redact", name),
            prev_helix_fetch: format!("user:{}:prev_helix_fetch", name),
        }
    }
}

#[derive(Debug)]
pub struct ChannelKeys {
    total: String,
    leaderboard: String,
    image: String,
}

impl Key for ChannelKeys {
    fn new(name: &str) -> Self {
        Self {
            total: format!("channel:#{}:total", name),
            leaderboard: format!("channel:#{}:leaderboard", name),
            image: format!("channel:#{}:image", name),
        }
    }
}

#[async_trait]
pub trait CacheRead {
    async fn get_channel_data(&self, channel: &str) -> RedisPoolResult<()>;
    async fn get_chatter_data(&self, chatter: &str) -> RedisPoolResult<()>;

    fn format_leadboard(leaderboard: Vec<String>) -> Vec<(String, isize)> {
        leaderboard
            .chunks_exact(2)
            .map(|chunk| (chunk[0].to_string(), chunk[1].parse::<isize>().unwrap_or(0)))
            .collect()
    }
}

#[async_trait]
pub trait CacheWrite {}

#[derive(Clone)]
pub struct RedisPool {
    manager: ConnectionManager,
}

impl fmt::Debug for RedisPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", CANNOT_DEBUG)
    }
}

impl RedisPool {
    pub async fn new(url: &str) -> RedisPoolResult<Self> {
        let client = redis::Client::open(url)?;
        let manager = ConnectionManager::new(client).await?;

        Ok(Self { manager })
    }

    pub async fn pull_from_db(channel: &str) -> RedisPoolResult<()> {
        todo!() 
    }

    pub async fn push_to_db(channel: &str) -> RedisPoolResult<()> {
        todo!()
    }
}

#[async_trait]
impl CacheRead for RedisPool {
    async fn get_chatter_data(&self, chatter: &str) -> RedisPoolResult<()> {
        let mut conn = self.manager.clone();

        let chatter_keys = ChatterKeys::new(chatter);

        let mut pipe = redis::pipe();
        pipe.atomic();

        pipe.get(chatter_keys.total);
        pipe.zrevrange_withscores(chatter_keys.leaderboard, 0, -1);

        let res_outer: Vec<Value> = pipe.query_async(&mut conn).await?;

        Ok(())
    }

    async fn get_channel_data(&self, channel: &str) -> RedisPoolResult<()> {
        Ok(())
    }
}

#[async_trait]
impl CacheCounter for RedisPool {
    async fn increment_counter(&self, channel: &str, chatter: &str) -> WsClientResult<()> {
        let mut conn = self.manager.clone();

        let chatter_keys = ChatterKeys::new(chatter);
        let channel_keys = ChannelKeys::new(channel);

        let mut pipe = redis::pipe();
        pipe.atomic();

        pipe.incr(chatter_keys.total, 1);
        pipe.incr(channel_keys.total, 1);

        pipe.zincr(chatter_keys.leaderboard, chatter, 1);
        pipe.zincr(channel_keys.leaderboard, channel, 1);

        let _: () = pipe.query_async(&mut conn).await?;

        Ok(())
    }
}
