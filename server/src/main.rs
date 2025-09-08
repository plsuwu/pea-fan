mod database;
mod hook;
mod parsing;
mod socket;
mod util;

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use crate::database::redis::redis_pool;
use crate::database::redis_migrate::Migrator;
use crate::database::schema::{Channel, Chatter};
use crate::socket::client::{IrcClient, IrcClientConfig};
use crate::socket::core::IrcEvent;
use crate::socket::handlers::{EventRouter, IrcCounter, IrcLogger};
use crate::socket::pool::{IrcConnectionPool, PoolConfig, PooledConnection};
use crate::util::channel::{self, ChannelUtilError};
use crate::util::helix::{Helix, HelixError};

use chrono::Local;
use thiserror::Error;
use tokio::time::sleep;
use tracing::{debug, error, info, instrument, warn};

type MainResult<T> = core::result::Result<T, MainError>;

#[derive(Error, Debug)]
enum MainError {
    #[error("Failed to get tracked channels: {0}")]
    ChannelRetrievalFailure(#[from] reqwest::Error),

    #[error("Failed to fetch data from Helix API: {0}")]
    HelixApiError(#[from] HelixError),

    #[error("Redis error: {0}")]
    RedisHandlerError(#[from] database::redis::RedisPoolError),

    #[error("postgres error: {0}")]
    PostgresError(#[from] database::postgres::PostgresError),

    #[error("channel util error: {0}")]
    ChannelUtilError(#[from] ChannelUtilError),

    #[error("irc websocket client error: {0}")]
    IrcClientError(#[from] socket::core::IrcError),
}

#[tokio::main]
#[instrument]
async fn main() -> MainResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    // let config = IrcClientConfig::default();
    let pool_config = PoolConfig::default();
    let channels = vec!["plss", "cchiko_"];

    let (mut pool, mut _events) = IrcConnectionPool::new(pool_config);

    let mut handler_router = EventRouter::new();
    for ch in &channels {
        handler_router.register("logger", IrcLogger::new(ch));
        handler_router.register("counter", IrcCounter::new(ch, "piss", true));
    }

    pool.start().await?;

    let join_task = {
        let pool = &pool;
        let channels = channels.clone();

        async move {
            // batch-join on socket pool
            for chunk in channels.chunks(10) {
                let mut tasks = Vec::new();
                for &channel in chunk {
                    tasks.push(async move {
                        match pool.join_channel(channel).await {
                            Ok(_) => {
                                info!("{}: join ok", channel);
                                Ok(())
                            }

                            Err(e) => {
                                error!("{}: failed to join: {}", channel, e);
                                Err(e)
                            }
                        }
                    });
                }

                let results = futures::future::join_all(tasks).await;
                let success = results.iter().filter(|r| r.is_ok()).count();
                let fail = results.len() - success;

                warn!("batched join result: {} ok, {} fail", success, fail);
                // sleep(Duration::from_millis(100)).await;
            }
        }
    };

    let event_task = {
        let mut event_rx = pool.event_broadcast.subscribe();

        async move {
            while let Ok(event) = event_rx.recv().await {
                match event {
                    _ => handler_router.route(&event).await,
                }
            }
        }
    };

    join_task.await;
    event_task.await;

    // pool.join_channel("plss").await?;
    //
    // while let Ok(event) = events.recv().await {
    //
    // }

    // let (mut client, mut events_rx) = IrcClient::new(IrcClientConfig::default());
    // client.connect().await?;
    //
    // client.join_channel("plss").await?;

    // while let Some(event) = events_rx.recv().await {
    //     debug!("RX: {:#?}", event);
    // }

    // _ = Migrator::new().preprocess().await?;
    // let mut channels = vec!["plss".to_string()];

    // let mut socket_pool =

    // let channel_user_data = Helix::fetch_user_by_login(&mut channels.clone()).await?;
    // let is_live = Helix::fetch_live_state(&mut channels).await?;

    // let config = socket::pool::SocketPoolConfig::default();
    // let handler = Arc::new(socket::handler::EchoEventHandler::new());
    //
    // let (mut pool, pool_tx) = socket::pool::SocketPool::new(config, handler);
    //
    // let pool_handle = tokio::spawn(async move {
    //     pool.start().await;
    // }).await.unwrap();

    Ok(())
}
