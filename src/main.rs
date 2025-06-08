use args::parse_cli_args;
use server::subscriber;

mod args;
mod parser;
mod server;
mod socket;

extern crate chrono;
// let args = args::parse_cli_args();

// const CHANNELS: [&'static str; 3] = ["cchiko_", "sleepiebug", "womfyy"];
const CHANNELS: [&'static str; 2] = ["sleepiebug", "plss"];

#[tokio::main]
async fn main() {
    let mut handles = Vec::new(); 

    let args = parse_cli_args();
    for broadcaster in CHANNELS.iter() {
        println!("[+] subscribing to stream event webhook: '{}'", &broadcaster);

        let args_clone = args.clone();
        let handle = tokio::task::spawn(async move {
            subscriber::sub_stream_event_multi(&broadcaster, &args_clone.token)
                .await
                .unwrap()
        });

        handles.push(handle);
    }
    
    // we're sending the subscription requests before the webhook is started here but 
    // i'm going to attempt to fix this momentarily...
    tokio::task::spawn(async move {
        let join_handle = futures_util::future::join_all(handles).await;
        for result in join_handle {
            println!("{:?}", result);
        }
    });

    server::serve().await;
}
