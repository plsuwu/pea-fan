use std::process::exit;
use thiserror::Error;

use tokio::task::JoinError;
use tracing::{debug, error, info, instrument};

use crate::webhook::router;
use crate::webhook::subscriber::{HookHandler, HookHandlerError, Subscriber};

mod database;
mod parser;
mod webhook;
mod socket;

type MainResult<T> = core::result::Result<T, MainError>;

#[derive(Error, Debug)]
enum MainError {
    #[error("Failed to get tracked channels: {0}")]
    ChannelRetrievalFailure(#[from] reqwest::Error),

    #[error("HookHandlerError: {0}")]
    HookHandlerError(#[from] HookHandlerError),

    #[error("Error awaiting server handle future: {0}")]
    ServerFutureError(#[from] JoinError)
}

#[tokio::main]
#[instrument]
async fn main() -> MainResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    info!("LOGGING STARTED\n");

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

    let hook_handler = HookHandler::new().await?;
    let channel_sub_handles = hook_handler.startup().await;

    server_handle.await?;

    Ok(())
}
