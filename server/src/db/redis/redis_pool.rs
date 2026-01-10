use std::sync::LazyLock;

use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::OnceCell;
use tracing::instrument;

use crate::util::env::{EnvErr, Var};
use crate::util::helix::HelixErr;
use crate::var;

static REDIS_POOL: LazyLock<OnceCell<RedisPool>> = LazyLock::new(OnceCell::new);
pub async fn redis_pool() -> RedisResult<&'static RedisPool> {
    REDIS_POOL
        .get_or_try_init(|| async { RedisPool::new().await })
        .await
}

#[macro_export]
macro_rules! redis_key {
    ($prefix:ident, $keytype:ident) => {{
        let key_type = match stringify!($prefix) {
            "channel" => KeyType::Channel,
            "user" => KeyType::Chatter,
            _ => panic!("invalid key type: {}", stringify!($prefix)),
        };

        let key = match stringify!($keytype) {
            "total" | "score" => RedisKey::Score(key_type),
            "leaderboard" => RedisKey::Leaderboard(key_type),
            _ => panic!("invalid key prefix: '{}'", stringify!($keytype)),
        }
        .wildcard();
        tracing::info!(key = ?key, "built wildcard redis key");

        key
    }};

    ($prefix:ident, $keytype:ident, $name:expr) => {{
        let key_type = match stringify!($prefix) {
            "channel" => KeyType::Channel,
            "user" => KeyType::Chatter,
            _ => panic!("invalid key type: {}", stringify!($prefix)),
        };

        let key = match stringify!($keytype) {
            "total" | "score" => RedisKey::Score(key_type),
            "leaderboard" => RedisKey::Leaderboard(key_type),
            _ => panic!("invalid key prefix: '{}'", stringify!($keytype)),
        }
        .with_name($name);

        tracing::info!(key = ?key, "built named redis key");

        key
    }};
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RedisKey {
    Score(KeyType),
    Leaderboard(KeyType),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum KeyType {
    Chatter,
    Channel,
}

impl core::fmt::Display for KeyType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

impl From<KeyType> for String {
    fn from(value: KeyType) -> Self {
        match value {
            KeyType::Chatter => String::from("user:"),
            KeyType::Channel => String::from("channel:#"),
        }
    }
}

impl RedisKey {
    #[instrument]
    pub fn with_name(&self, name: &str) -> String {
        match self {
            RedisKey::Score(prefix) => format!("{}{}:total", prefix, name),
            RedisKey::Leaderboard(prefix) => format!("{}{}:leaderboard", prefix, name),
        }
    }

    #[instrument]
    pub fn wildcard(&self) -> String {
        match self {
            RedisKey::Score(prefix) => format!("{}*:total", prefix),
            RedisKey::Leaderboard(prefix) => format!("{}*:leaderboard", prefix),
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
    #[error(transparent)]
    EnvErr(#[from] EnvErr),

    #[error(transparent)]
    PgError(#[from] crate::db::PgError),

    #[error(transparent)]
    HelixError(#[from] HelixErr),

    #[error(transparent)]
    RedisClientError(#[from] redis::RedisError),

    #[error(transparent)]
    ParseError(#[from] redis::ParsingError),

    #[error("unable to parse resulting redis key")]
    BadKey,

    #[error(transparent)]
    SqlxError(#[from] sqlx::error::Error),
}

pub struct RedisPool {
    pub manager: ConnectionManager,
}
