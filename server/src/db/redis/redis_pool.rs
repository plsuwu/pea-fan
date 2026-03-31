use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::OnceCell;
use tracing::instrument;

use crate::util::env::{EnvErr, Var};
use crate::util::helix::HelixErr;
use crate::var;

static REDIS_POOL: OnceCell<ConnectionManager> = OnceCell::const_new();

/// Retrieves a reference to a `redis::aio::ConnectionManager`.
///
/// As the `redis::aio::ConnectionManager` is an `Arc<_>` under the hood, this reference can be
/// cloned to produce an owned instance of a Redis connection.
pub async fn redis_pool() -> RedisResult<&'static ConnectionManager> {
    REDIS_POOL
        .get_or_try_init(|| async {
            let redis_url = var!(Var::RedisUrl).await?;
            let client = redis::Client::open(redis_url)?;

            Ok(ConnectionManager::new(client).await?)
        })
        .await
}

#[macro_export]
/// Usage:
/// ```no_run
/// redis_key!(
///     channel | user,
///     total | score | leaderboard,
///     "USER_LOGIN"
/// );
/// ```
macro_rules! redis_key {
    ($prefix:ident, $keytype:ident) => {{
        use crate::db::redis::redis_pool::KeyType;
        use crate::db::redis::redis_pool::RedisKey;

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
        tracing::trace!(key = ?key, "built wildcard redis key");

        key
    }};

    ($prefix:ident, $keytype:ident, $name:expr) => {{
        use crate::db::redis::redis_pool::KeyType;
        use crate::db::redis::redis_pool::RedisKey;

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

        tracing::trace!(key = ?key, "built named redis key");

        key
    }};
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RedisKey {
    Score(KeyType),
    Leaderboard(KeyType),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, PartialOrd)]
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
            RedisKey::Score(prefix) => format!("{prefix}{name}:total"),
            RedisKey::Leaderboard(prefix) => format!("{prefix}{name}:leaderboard"),
        }
    }

    #[instrument]
    pub fn wildcard(&self) -> String {
        match self {
            RedisKey::Score(prefix) => format!("{prefix}*:total"),
            RedisKey::Leaderboard(prefix) => format!("{prefix}*:leaderboard"),
        }
    }
}

pub type RedisResult<T> = core::result::Result<T, RedisErr>;

#[derive(Debug, Error)]
pub enum RedisErr {
    #[error(transparent)]
    Std(#[from] Box<dyn std::error::Error>),

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

    #[error(transparent)]
    SqlxError(#[from] sqlx::error::Error),
}

unsafe impl Send for RedisErr {}
unsafe impl Sync for RedisErr {}
