mod args;
mod socket;

extern crate chrono;

use crate::socket::settings::CONNECTION_SETTINGS;
use socket::client::Client;
use tokio_tungstenite::tungstenite::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let settings_rw_lock = &*CONNECTION_SETTINGS;

    let client = Client::new(settings_rw_lock).await?;
    client.open(settings_rw_lock).await?;

    client.loop_read().await;

    Ok(())
}
