extern crate redis;
use std::sync::LazyLock;

use redis::{AsyncCommands, AsyncConnectionConfig};
use redis::{Client, aio::ConnectionManager};
use tokio::runtime::Handle;
use tokio::sync::OnceCell;

pub type RedisPoolResult<T> = Result<T, redis::RedisError>;

const REDIS_URL: &'static str = "redis://127.0.0.1:6380";
static REDIS_CONNECTION_POOL: LazyLock<OnceCell<RedisPool>> = LazyLock::new(OnceCell::new);

pub async fn redis_pool() -> RedisPoolResult<&'static RedisPool> {
    REDIS_CONNECTION_POOL
        .get_or_try_init(|| async { RedisPool::new(REDIS_URL).await })
        .await
}

/// $: `redis-server --port 6380 --save "300 10" --appendonly yes --appendfsync everysec`
#[derive(Clone)]
pub struct RedisPool {
    manager: ConnectionManager,
}

impl RedisPool {
    pub async fn new(redis_url: &str) -> RedisPoolResult<Self> {
        let client = Client::open(redis_url)?;
        let manager = ConnectionManager::new(client).await?;

        Ok(RedisPool { manager })
    }

    pub async fn increment(&self, channel: &str, chatter: &str) -> RedisPoolResult<()> {
        let mut conn = self.manager.clone();

        let user_total = format!("user:{}:total", chatter);
        let chan_total = format!("channel:{}:total", channel);

        let user_leaderboard = format!("user:{}:leaderboard", chatter);
        let chan_leaderboard = format!("channel:{}:leaderboard", channel);

        // Atomic read-modify-write operations via transaction
        let mut pipe = redis::pipe();
        pipe.atomic();

        pipe.incr(user_total, 1);  
        pipe.incr(chan_total, 1);

        pipe.zincr(chan_leaderboard, chatter, 1);
        pipe.zincr(user_leaderboard, channel, 1);

        let _: () = pipe.query_async(&mut conn).await?;

        Ok(())
    }
}
