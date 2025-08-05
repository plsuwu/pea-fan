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

#[derive(Clone)]
pub struct RedisPool {
    manager: ConnectionManager,
}

impl fmt::Debug for RedisPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // i want to be able to log debug info where alongside this struct but for obvious
        // reasons this isn't really viable to have debug implemented in a simple way, so
        // we have this thing instead
        write!(f, "{}", CANNOT_DEBUG)
    }
}

impl RedisPool {
    pub async fn new(url: &str) -> RedisPoolResult<Self> {
        let client = redis::Client::open(url)?;
        let manager = ConnectionManager::new(client).await?;

        Ok(Self { manager })
    }

    /// When is stream comes online, perform a batched read from the database to facilitate faster
    /// write access
    ///
    /// # Params
    ///
    /// * `channel_id` - The ID of the channel to pull into Redis
    pub async fn from_db(channel_id: &str) -> RedisPoolResult<()> {
        todo!()
    }

    /// When a stream goes offline, perform a batched write to the database to free up memory
    ///
    /// # Params
    ///
    /// * `channel_id` - the ID of the channel to push out of Redis
    pub async fn to_db(channel_id: &str) -> RedisPoolResult<()> {
        todo!()
    }
}

#[async_trait]
pub trait CacheWrite {}

pub enum UserType {
    Chatter,
    Channel,
}

impl ToString for UserType {
    fn to_string(&self) -> String {
        match self {
            UserType::Chatter => String::from("user"),
            UserType::Channel => String::from("channel"),
        }
    }
}

pub trait Key {
    fn leaderboard(&self, id: &str) -> String;

    fn login(&self, id: &str) -> String;
    fn image(&self, id: &str) -> String;
    fn total(&self, id: &str) -> String;

    fn redact(&self, id: &str) -> String;
    fn fetched(&self, id: &str) -> String;
}

#[derive(Debug)]
pub enum QueryKey {
    Chatter,
    Channel,
}

impl QueryKey {
    fn to_str(&self) -> &'static str {
        match self {
            QueryKey::Chatter => "chatter",
            QueryKey::Channel => "channel",
        }
    }
}

impl Key for QueryKey {
    fn login(&self, id: &str) -> String {
        format!("{}:{}:login", self.to_str(), id)
    }

    fn image(&self, id: &str) -> String {
        format!("{}:{}:image", self.to_str(), id)
    }

    fn total(&self, id: &str) -> String {
        format!("{}:{}:total", self.to_str(), id)
    }

    fn leaderboard(&self, id: &str) -> String {
        format!("{}:{}:leaderboard", self.to_str(), id)
    }

    fn redact(&self, id: &str) -> String {
        format!("{}:{}:redact", self.to_str(), id)
    }

    fn fetched(&self, id: &str) -> String {
        format!("{}:{}:fetched", self.to_str(), id)
    }
}

pub struct Region {
    cursor: isize,
    limit: isize,
}

impl Region {
    pub fn increment(&mut self) {
        self.cursor += self.limit;
        self.limit += self.limit;
    }
}

impl Default for Region {
    fn default() -> Self {
        Self {
            cursor: 0,
            limit: 99,
        }
    }
}

#[async_trait]
pub trait CacheRead {
    // async fn get_leaderboard(&self, key: QueryKey, region: Region) -> RedisPoolResult<Vec<User>>;
    async fn get_image(&self, key: QueryKey) -> Option<String>;
    async fn get_total(&self, key: QueryKey) -> i32;
    async fn get_login(&self, key: QueryKey) -> String;
    async fn get_redaction(&self, key: QueryKey) -> bool;
    async fn get_fetched(&self, key: QueryKey) -> bool;
}

#[async_trait]
impl CacheRead for RedisPool {
    // async fn get_leaderboard(&self, key: QueryKey, region: Region) -> RedisPoolResult<Vec<User>> {
    //     // let leaderboard =
    //     //     self.manager
    //     //         .zrevrange_withscores(key.to_str(), region.cursor, region.limit).await?;
    //     todo!();
    // }

    async fn get_image(&self, key: QueryKey) -> Option<String> {
        todo!()
    }

    async fn get_total(&self, key: QueryKey) -> i32 {
        todo!()
    }

    async fn get_login(&self, key: QueryKey) -> String {
        todo!()
    }

    async fn get_redaction(&self, key: QueryKey) -> bool {
        todo!()
    }

    async fn get_fetched(&self, key: QueryKey) -> bool {
        todo!()
    }
}
