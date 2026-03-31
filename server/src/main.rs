// #![warn(unused_crate_dependencies)]

use std::net::SocketAddr;
use std::sync::Arc;

use futures::future::join_all;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::api::server::RouteError;
use crate::db::redis::redis_pool::{RedisErr, redis_pool};
use crate::db::{PgError, db_pool};
use crate::irc::ConnectionClientError;
use crate::util::channel::ChannelError;
use crate::util::env::Var;
use crate::util::telemetry::Telemetry;
use crate::util::totp;

mod api;
mod db;
mod irc;
mod util;

#[derive(Debug, Error)]
enum RunnerErr {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Redis(#[from] RedisErr),

    #[error(transparent)]
    Pg(#[from] PgError),

    #[error(transparent)]
    Std(#[from] Box<dyn std::error::Error>),

    #[error(transparent)]
    Irc(#[from] ConnectionClientError),

    #[error(transparent)]
    Channel(#[from] ChannelError),

    #[error(transparent)]
    Router(#[from] RouteError),
}

type Result<T> = core::result::Result<T, RunnerErr>;

#[tokio::main]
async fn main() -> Result<()> {
    let telemetry_registry = Telemetry::new().await?.register();
    log_startup_init();
    
    let database_pool = db_pool().await?;
    let redis_pool = redis_pool().await?;

    let totp_handler = {
        let totp_key = var!(Var::TOTPKey).await.unwrap();
        Arc::new(Mutex::new(totp::TOTPHandler::new(totp_key)))
    };

    let (tx_server_ready, rx_server_ready) = tokio::sync::mpsc::unbounded_channel::<SocketAddr>();
    let mut handles = Vec::new();

    let server_handles = api::server::start_server(
        tx_server_ready,
        rx_server_ready,
        database_pool,
        redis_pool.clone(),
        totp_handler,
    )
    .await?;

    handles.extend(server_handles);

    _ = join_all(handles).await;
    telemetry_registry.shutdown();
    Ok(())
}

#[inline]
fn log_startup_init() {
    tracing::info!("=======================");
    tracing::info!("      STARTING UP      ");
    tracing::info!("=======================");
}
