use std::sync::LazyLock;

use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::OnceCell;
use tracing::{info, instrument};

use crate::db::pg;
use crate::util::env::{EnvErr, Var};
use crate::util::helix::HelixErr;
use crate::var;

static REDIS_POOL: LazyLock<OnceCell<RedisPool>> = LazyLock::new(OnceCell::new);
pub async fn redis_pool() -> RedisResult<&'static RedisPool> {
    REDIS_POOL
        .get_or_try_init(|| async { RedisPool::new().await })
        .await
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

impl RedisPool {
    #[instrument]
    pub async fn new() -> RedisResult<Self> {
        let redis_url = var!(Var::RedisUrl).await?;
        tracing::debug!(redis_url, "connecting to redis server");

        let client = redis::Client::open(redis_url)?;
        let manager = ConnectionManager::new(client).await?;

        Ok(Self { manager })
    }
}

pub type RedisResult<T> = core::result::Result<T, RedisErr>;

#[derive(Debug, Error)]
pub enum RedisErr {
    #[error("{0}")]
    EnvErr(#[from] EnvErr),

    #[error("{0}")]
    PgError(#[from] pg::PgErr),

    #[error("helix fetch error: {0}")]
    HelixError(#[from] HelixErr),

    #[error("redis client error: {0}")]
    RedisClientError(#[from] redis::RedisError),

    #[error("parse error: {0}")]
    ParseError(#[from] redis::ParsingError),

    #[error("unable to parse resulting redis key")]
    BadKey,
}

pub struct RedisPool {
    pub manager: ConnectionManager,
}
