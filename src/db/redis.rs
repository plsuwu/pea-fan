extern crate redis;
use std::collections::HashMap;
use std::sync::LazyLock;

use redis::{AsyncCommands, AsyncConnectionConfig, Value, from_redis_value};
use redis::{Client, aio::ConnectionManager};
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;
use tokio::sync::OnceCell;

use crate::server::RedisQueryResponse;

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

    pub async fn get_channel_data(&self, channel: &str) -> RedisPoolResult<RedisQueryResponse> {
        let mut conn = self.manager.clone();

        let chan_total = format!("channel:#{}:total", channel);
        let chan_leaderboard = format!("channel:#{}:leaderboard", channel);

        let mut pipe = redis::pipe();
        pipe.atomic();

        pipe.get(chan_total);
        pipe.zrevrange_withscores(chan_leaderboard, 0, 5);

        let res_outer: Vec<Value> = pipe.query_async(&mut conn).await?;
        let total: String = from_redis_value(&res_outer[0])?;
        let leaderboard_vec: Vec<String> = from_redis_value(&res_outer[1])?;
        let leaderboard = Self::pair_score_with_user(leaderboard_vec);

        Ok(RedisQueryResponse {
            total,
            err_msg: "",
            leaderboard,
            err: false,
        })
    }

    pub async fn get_user_data(&self, user: &str) -> RedisPoolResult<RedisQueryResponse> {
        let mut conn = self.manager.clone();

        let user_total = format!("user:{}:total", user);
        let user_leaderboard = format!("user:{}:leaderboard", user);

        let mut pipe = redis::pipe();
        pipe.atomic();

        pipe.get(user_total);
        pipe.zrevrange_withscores(user_leaderboard, 0, -1); // all channel data

        let res_outer: Vec<Value> = pipe.query_async(&mut conn).await?;
        let total: String = from_redis_value(&res_outer[0])?;
        let leaderboard_vec: Vec<String> = from_redis_value(&res_outer[1])?;
        let leaderboard = Self::pair_score_with_user(leaderboard_vec);

        Ok(RedisQueryResponse {
            total,
            err_msg: "",
            leaderboard,
            err: false,
        })
    }

    fn pair_score_with_user(data: Vec<String>) -> Vec<(String, isize)> {
        data.chunks_exact(2)
            .map(|chunk| (chunk[0].to_string(), chunk[1].parse::<isize>().unwrap()))
            .collect()
    }
}

#[derive(Serialize, Deserialize)]
pub struct CounterData {
    total: String,
    leaderboard: Vec<String>,
}
