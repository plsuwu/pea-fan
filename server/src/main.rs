#![warn(unused_crate_dependencies)]

use std::net::SocketAddr;

use futures::future::join_all;
use thiserror::Error;
use tokio::sync::oneshot::Sender;

use crate::{api::server::RouteError, irc::client::IrcClientErr, util::telemetry};

mod api;
mod db;
mod irc;
mod util;

#[derive(Debug, Error)]
enum RunnerErr {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Std(#[from] Box<dyn std::error::Error>),

    #[error(transparent)]
    Irc(#[from] IrcClientErr),

    #[error(transparent)]
    Router(#[from] RouteError),
}

type Result<T> = core::result::Result<T, RunnerErr>;

#[tokio::main]
async fn main() -> Result<()> {
    let telemetry_registry = telemetry::Telemetry::new().await?.register();

    tracing::info!("starting main application");

    let channels_updated = util::channel::update_channels(None).await.unwrap();
    let channel_names: Vec<String> = channels_updated.into_keys().collect();
    let (tx_server_ready, rx_server_ready) = tokio::sync::mpsc::unbounded_channel::<SocketAddr>();
    let (tx_to_client, rx_from_api) =
        tokio::sync::mpsc::unbounded_channel::<(String, Sender<Vec<String>>)>();

    let mut handles = Vec::new();
    let irc_handles = irc::client::start_irc_handler(channel_names, rx_from_api).await?;
    let server_handles =
        api::server::start_server(tx_server_ready, tx_to_client, rx_server_ready).await?;

    handles.extend(irc_handles);
    handles.extend(server_handles);
    _ = join_all(handles).await;

    telemetry_registry.shutdown();
    Ok(())
}
