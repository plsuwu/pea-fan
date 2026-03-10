use std::{sync::Arc, time::Duration};

pub mod bridge;
pub mod channels;
pub mod commands;
pub mod connection;
pub mod error;
pub mod parse;
pub mod rate_limit;
pub mod worker;

pub use bridge::IrcHandle;
pub use commands::*;
pub use error::*;

use sqlx::PgPool;
use tinyrand::{Rand, RandRange, Seeded, StdRand};
use tinyrand_std::ClockSeed;
use tokio::sync::mpsc;
use tracing::instrument;

use crate::irc::{connection::ConnectionSupervisor, rate_limit::Bucket, worker::WorkerPool};

#[instrument]
pub async fn start(
    channels: Vec<String>,
    pool: &'static PgPool,
    worker_count: usize,
) -> ClientResult<IrcHandle> {
    let (mut supervisor, conn_handle) = ConnectionSupervisor::new(channels);

    let (msg_tx, msg_rx) = async_channel::bounded(256);
    let (cmd_tx, cmd_rx) = mpsc::channel(64);
    let (query_tx, query_rx) = mpsc::channel(32);
    
    // one permit per bucket, polls for an empty bucket every 1.15 secons and if the bucket is
    // empty, waits an additional 1.15 seconds before refilling
    let rate_limiter = Arc::new(Bucket::new(Duration::from_millis(1150), 1));
    let _workers = WorkerPool::spawn(worker_count, msg_rx, cmd_tx.clone(), rate_limiter, pool);

    tokio::spawn(async move {
        supervisor.run(msg_tx, cmd_rx, query_rx).await;
    });

    Ok(IrcHandle {
        cmd_tx,
        query_tx,
        connection: conn_handle,
    })
}

#[instrument]
pub fn idx(max: usize) -> usize {
    let seed = ClockSeed.next_u64();
    let mut rng = StdRand::seed(seed);

    rng.next_range(0..max)
}

#[derive(Debug)]
pub enum ReplyReason {
    BotCountQueried,
}

impl ReplyReason {
    #[instrument(skip(self))]
    pub fn get_reply(&self) -> &'static str {
        let reasons = match self {
            ReplyReason::BotCountQueried => Self::BOT_COUNT_QUERY,
        };

        reasons[idx(reasons.len() - 1)]
    }

    const BOT_COUNT_QUERY: [&'static str; 6] = [
        "why would i tell you that. so you can mock me. typical.",
        "do you think im stupid. do you actually think that i am dumb.",
        "why dont you worry about your own count instead huh.",
        "do you also ask the mailman to open their letters?",
        "you think youre clever dont you but you arent.",
        "dont you dare ask me for that information ever again.",
    ];
}
