mod args;
mod socket;
mod parser;
mod server;

extern crate chrono;
// let args = args::parse_cli_args();    

#[tokio::main]
async fn main() {
    server::serve().await;
}
