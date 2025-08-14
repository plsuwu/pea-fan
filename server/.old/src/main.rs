use crate::constants::CHANNELS;
use crate::server::router;
use crate::server::webhook::subscriber::{self, reset_all_hooks};
use args::get_cli_args;
use std::process::exit;
use std::sync::Arc;
use tokio::io;

mod args;
mod constants;
mod db;
mod parser;
mod server;
mod socket;

extern crate chrono;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = get_cli_args();

    let (tx, rx) = tokio::sync::oneshot::channel();
    let server_handle = tokio::task::spawn(async move {
        router::route(tx).await;
    });

    match rx.await {
        Ok((bind_addr, key_opt)) => {
            println!("[+] server listening on {}", bind_addr);
            if let Some(key) = key_opt {
                println!("[+] using key '{}'", key);
            };
        }

        Err(_) => {
            eprintln!("[x] Failed to start webhook server.");
            exit(1);
        }
    }

    reset_all_hooks().await;

    let mut handles = Vec::new();
    for broadcaster in CHANNELS.iter() {
        println!(
            "[+] subscribing to 'stream.online'+'stream.offline' event webhooks for '{}'",
            &broadcaster
        );

        let args_clone = Arc::clone(&args);
        let handle = tokio::task::spawn(async move {
            match subscriber::sub_stream_event_multi(&broadcaster, &args_clone.app_token).await {
                Ok(res) => res,
                Err(e) => {
                    println!(
                        "[x] Subscription attempt for '{}' - error: {:?}",
                        broadcaster, e
                    );
                }
            }
        });

        handles.push(handle);
    }

    _ = futures_util::future::join_all(handles).await;
    server_handle.await?;
    Ok(())
}
