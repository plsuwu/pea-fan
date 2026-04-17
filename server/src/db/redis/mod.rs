use redis::AsyncCommands;
use tracing::instrument;

use super::redis::redis_pool::RedisResult;
use crate::{db::prelude::ChannelId, util::helix::Helix};

pub mod migrator;
pub mod redis_pool;

#[instrument(skip(redis_pool, ids))]
pub async fn clear_stream_states<R: AsyncCommands + Sync>(
    redis_pool: &mut R,
    ids: &[String],
) -> RedisResult<()> {
    let mut pipeline = redis::pipe();
    for id in ids {
        let key = format!("{}:online", id);
        pipeline.del(key);
    }

    () = pipeline.query_async(redis_pool).await?;
    Ok(())
}

#[instrument(skip(redis_pool, ids))]
pub async fn init_stream_states<R: AsyncCommands + Sync>(
    redis_pool: &mut R,
    ids: &[String],
) -> RedisResult<()> {
    let mut pipeline = redis::pipe();
    let live = Helix::get_streams(ids).await?;
    tracing::debug!(live_broadcasters = ?live, "retrieved stream states");

    for id in ids {
        let key = format!("{}:online", id);
        match live.iter().any(|br| &br.user_id == id) {
            true => {
                tracing::debug!(id, "caching stream.online");
                pipeline.set(key, 1);
            }
            false => {
                tracing::debug!(id, "removing cached broadcast");
                pipeline.del(key);
            }
        };
    }

    let _: () = pipeline.query_async(redis_pool).await?;
    Ok(())
}

#[instrument(skip(redis_pool))]
pub async fn get_all_live<R: AsyncCommands + Sync>(
    redis_pool: &mut R,
) -> RedisResult<Option<Vec<String>>> {
    let keys: Vec<String> = redis_pool.keys("*:online").await.unwrap_or_default();

    // avoid allocating up to like three or four Vecs by optioning and having the caller handle the
    // empty/None case; though I imagine extra allocations are optimized away by the compiler, this
    // seems more explicit and reliable.
    if keys.is_empty() {
        return Ok(None);
    }

    let result: Vec<String> = keys
        .into_iter()
        .filter_map(|item| item.split(':').nth(0).map(str::to_owned))
        .collect();

    Ok(Some(result))
}

#[instrument(skip(redis_pool))]
pub async fn set_stream_state<R: AsyncCommands + Sync>(
    redis_pool: &mut R,
    channel_id: &ChannelId,
    online: bool,
) -> RedisResult<()> {
    tracing::info!(channel_id = channel_id.0, online, "setting stream state");
    let key = format!("{}:online", channel_id.0);

    if !online {
        let _: () = redis_pool.del(key).await.unwrap_or_default();
        return Ok(());
    }

    let _: () = redis_pool.set(key, online as i8).await?;

    Ok(())
}

#[instrument(skip(redis_pool))]
pub async fn get_stream_state<R: AsyncCommands + Sync>(
    redis_pool: &mut R,
    channel_id: &ChannelId,
) -> bool {
    let key = format!("{}:online", channel_id.0);
    let state: bool = match redis_pool.get(key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::error!(
                error = ?e,
                channel_id = channel_id.0,
                "failed to retrieve stream state, assume online=true as fallback"
            );

            true
        }
    };

    state
}
