use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use redis::{AsyncCommands, aio::ConnectionManager, from_redis_value};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::OnceCell;
use tracing::{debug, info, instrument, trace, warn};

use crate::database::postgres::{self, PostgresError};
use crate::database::schema::{self, Channel, Chatter, Score};
use crate::util::helix::{Helix, HelixError, InternalUser};
use crate::util::secrets::ENV_SECRETS;

pub type RedisPoolResult<T> = core::result::Result<T, RedisPoolError>;

pub const NOT_PRESENT_IN_CACHE: &str = "[NOT_PRESENT_IN_CACHE]";
pub const NOT_VALID_HELIX_USER: &str = "[NOT_VALID_HELIX_USER]";

// TODO: should this be a `LazyLock` rather than `OnceCell`?
static REDIS_POOL: LazyLock<OnceCell<RedisPool>> = LazyLock::new(OnceCell::new);
pub async fn redis_pool() -> RedisPoolResult<&'static RedisPool> {
    REDIS_POOL
        .get_or_try_init(|| async { RedisPool::new().await })
        .await
}

#[derive(Debug, Error)]
pub enum RedisPoolError {
    #[error("redis client error: {0}")]
    RedisClientError(#[from] redis::RedisError),

    // #[error("redis value conversion error")]
    // ConversionError,
    #[error("helix fetch error: {0}")]
    HelixFetchError(#[from] HelixError),

    #[error("sqlx-postgres error: {0}")]
    PostgresError(#[from] PostgresError),
}

#[derive(Serialize, Deserialize)]
pub enum ChatterKey {
    Id(String),
    Name(String),
    Score(String),
    Leaderboard(String),
}

#[derive(Serialize, Deserialize)]
pub enum ChannelKey {
    Id(String),
    Name(String),
    Score(String),
    Leaderboard(String),
}

impl From<ChatterKey> for String {
    fn from(value: ChatterKey) -> Self {
        match value {
            ChatterKey::Id(chatter_login) => format!("chatter:{}:id", chatter_login),
            ChatterKey::Name(chatter_id) => format!("chatter:{}:name", chatter_id),
            ChatterKey::Score(chatter_id) => format!("chatter:{}:score", chatter_id),
            ChatterKey::Leaderboard(chatter_id) => format!("chatter:{}:leaderboard", chatter_id),
        }
    }
}

impl From<ChannelKey> for String {
    fn from(value: ChannelKey) -> Self {
        match value {
            ChannelKey::Id(channel_login) => format!("channel:{}:id", channel_login),
            ChannelKey::Name(channel_id) => format!("channel:{}:id", channel_id),
            ChannelKey::Score(channel_id) => format!("channel:{}:id", channel_id),
            ChannelKey::Leaderboard(channel_id) => format!("channel:{}:id", channel_id),
        }
    }
}

pub struct RedisPool {
    pub manager: ConnectionManager,
}

impl RedisPool {
    #[instrument]
    pub async fn new() -> RedisPoolResult<Self> {
        let host = &ENV_SECRETS.get().redis_host;
        let port = &ENV_SECRETS.get().redis_port;
        let url = format!("redis://{}:{}", host, port);

        info!("Redis client connecting to server at '{}'", &url);

        let client = redis::Client::open(url)?;
        let manager = ConnectionManager::new(client).await?;

        Ok(Self { manager })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ActiveChannel {
    pub broadcaster: schema::Chatter,
    pub total_count_lifetime: i32,
    pub chatters_current: Vec<schema::Chatter>,
    pub total_count_current: i32,
}

impl ActiveChannel {
    pub async fn pull(id: &str) -> RedisPoolResult<Self> {
        let broadcaster = schema::Chatter::get_by_id(id).await?;

        Ok(Self {
            total_count_lifetime: broadcaster.total.clone(),
            broadcaster,
            total_count_current: 0,
            chatters_current: Vec::new(),
        })
    }

    pub async fn push(&self) -> RedisPoolResult<()> {
        todo!()
    }

    pub async fn increment(&mut self, chatter_id: &str) -> RedisPoolResult<()> {
        todo!()
    }
}
