use std::process::exit;
use thiserror::Error;

use tracing::{error, info, instrument};

use crate::webhook::router;

mod database;
mod parser;
mod webhook;
mod ws;

type MainResult<T> = core::result::Result<T, MainError>;

#[derive(Error, Debug)]
enum MainError {
    #[error("Failed to get tracked channels: {0}")]
    ChannelRetrievalFailure(#[from] reqwest::Error),
}

#[tokio::main]
#[instrument]
async fn main() -> MainResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let (tx, rx) = tokio::sync::oneshot::channel();
    let server_handle = tokio::task::spawn(async move {
        router::route(tx).await;
    });

    match rx.await {
        Ok((bind_addr, key)) => {
            info!("Webhook server listening on {}", bind_addr);
            info!("Using verification key: '{}'", key);
        }

        Err(_) => {
            error!("Failed to start webhook server; exiting...");
            exit(1);
        }
    }

    Ok(())
}
