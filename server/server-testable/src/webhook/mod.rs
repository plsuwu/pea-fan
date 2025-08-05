pub mod connection;
pub mod helix;
pub mod middleware;
pub mod router;
pub mod server;
pub mod subscriber;
pub mod types;

use tracing::{info, instrument};

const CHANNELS_LIST: &str =
    "https://raw.githubusercontent.com/plsuwu/pea-fan/refs/heads/static/channels";

#[instrument]
pub async fn get_tracked_channels() -> reqwest::Result<Vec<String>> {
    let channel_list = reqwest::get(CHANNELS_LIST)
        .await?
        .text()
        .await?
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();

    info!(
        "Using channel list ({} items): {:#?}",
        channel_list.len(),
        channel_list
    );

    Ok(channel_list)
}
