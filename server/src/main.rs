use std::net::SocketAddr;

use futures::future::join_all;
use tokio::sync::oneshot::Sender;

mod api;
mod db;
mod irc;
mod util;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let provider = crate::util::tracing::build_subscriber().await.unwrap();
    tracing::info!("starting main application");

    let channels_updated = util::channel::update_channels(None).await.unwrap();
    let channel_names: Vec<String> = channels_updated.into_iter().map(|(chan, _)| chan).collect();

    let (tx_server_ready, rx_server_ready) = tokio::sync::mpsc::unbounded_channel::<SocketAddr>();
    let (tx_to_client, rx_from_api) =
        tokio::sync::mpsc::unbounded_channel::<(String, Sender<Vec<String>>)>();

    let mut handles = Vec::new();

    let irc_handles = irc::client::start_irc_handler(channel_names, rx_from_api)
        .await
        .unwrap();

    let server_handles = api::server::start_server(tx_server_ready, tx_to_client, rx_server_ready)
        .await
        .unwrap();

    handles.extend(irc_handles);
    handles.extend(server_handles);

    _ = join_all(handles).await;

    crate::util::tracing::destroy_tracer(provider);

    Ok(())
}
