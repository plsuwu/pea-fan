use args::parse_cli_args;
use server::subscriber::{self, get_active_hooks};
use std::process::exit;
use tokio::io;

mod args;
mod db;
mod parser;
mod server;
mod socket;

extern crate chrono;
// let args = args::parse_cli_args();

pub const CHANNELS: [&'static str; 13] = [
    "cchiko_",
    "sleepiebug",
    "myrmidon",
    "lcolonq",
    "liljuju",
    "parasi",
    "snoozy",
    "vacu0usly",
    "womfyy",
    "kyoharuvt",
    "myramors",
    "batatvideogames",
    "chocojax",
];

// pub const CHANNELS: [&'static str; 1] = [
//     "sleepiebug",
// ];

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = parse_cli_args();

    let (tx, rx) = tokio::sync::oneshot::channel();
    let server_handle = tokio::task::spawn(async move {
        server::serve(tx).await;
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

    // nuke all active subscriptions on startup - kind of 'resets' our subscription state;
    // we realistically shouldn't have to do this very often.
    if let Some(active_subscriptions) = get_active_hooks(&args.app_token).await {
        _ = futures_util::future::join_all(
            active_subscriptions
                .iter()
                .map(async |sub_val: &serde_json::Value| {
                    let subscription_id: &str = sub_val["id"].as_str().unwrap();
                    println!("[+] deleting subscription with id '{}'", subscription_id);

                    subscriber::delete_subscription_multi(subscription_id, &args.app_token)
                        .await
                        .unwrap()
                })
                .collect::<Vec<_>>(),
        )
        .await;
    };

    let mut handles = Vec::new();
    for broadcaster in CHANNELS.iter() {
        println!(
            "[+] subscribing to 'stream.online'/'stream.offline' event webhooks for '{}'",
            &broadcaster
        );

        let args_clone = args.clone();
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
