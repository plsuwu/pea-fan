use server::subscriber;

mod args;
mod socket;
mod parser;
mod server;

extern crate chrono;

#[tokio::main]
async fn main() {
    let args = args::parse_cli_args();

    // let broadcaster_id = subscriber::get_user_id(&args.broadcaster).await.unwrap();
    // println!("{}", broadcaster_id);

    server::webhook::server_main().await;
}
