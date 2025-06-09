use args::parse_cli_args;
use parser::{lexer, test_parser_function};
use server::{close_websocket, open_websocket, subscriber};
use socket::{client::Client, settings::ConnectionSettings};
use std::{
    process::exit,
    sync::{Arc, RwLock},
};
use tokio::{io, time::{sleep, Sleep}};

mod args;
mod parser;
mod server;
mod socket;

extern crate chrono;
// let args = args::parse_cli_args();

const CHANNELS: [&'static str; 5] = ["cchiko_", "sleepiebug", "womfyy", "snoozy", "vacu0usly"];
// const CHANNELS: [&'static str; 2] = ["sleepiebug", "plss"];

#[tokio::main]
async fn main() -> io::Result<()> {
    // parser::test_parser_function();
    let args = parse_cli_args();
    
    // let test_socket_handle = open_websocket("plss");
    // sleep(tokio::time::Duration::from_secs(10)).await;
    // let close_test_socket = close_websocket("plss").await;



    // let mut irc_handles = Vec::new();
    //
    //
    //
    // let irc_join_handle = futures_util::future::join_all(irc_handles).await;
    // for res in irc_join_handle {
    //     println!("{:?}", res);
    // }

    let (tx, rx) = tokio::sync::oneshot::channel();
    let server_handle = tokio::task::spawn(async move {
        server::serve(tx).await;
    });
    //
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

    // let mut handles = Vec::new();
    // for broadcaster in CHANNELS.iter() {
    //     println!(
    //         "[+] subscribing to 'stream.online'/'stream.offline' event webhooks for '{}'",
    //         &broadcaster
    //     );
    //
    //     let args_clone = args.clone();
    //     let handle = tokio::task::spawn(async move {
    //         subscriber::sub_stream_event_multi(&broadcaster, &args_clone.app_token)
    //             .await
    //             .unwrap()
    //     });
    //
    //     handles.push(handle);
    // }
    //
    


    // let result = test_socket_handle.await.unwrap();
    // println!("{:?}", result);
    // let join_handle = futures_util::future::join_all(handles).await;
    // for result in join_handle {
    //     println!("{:?}", result);
    // }
    //
    server_handle.await?;
    Ok(())
}
