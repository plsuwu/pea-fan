use std::sync::Arc;

use clap::Parser;

const TWITCH_OAUTH_LENGTH: usize = 30;

#[derive(Parser, Debug)]
pub struct Cli {
    /// TTV user/bot login/username
    #[arg(short, long)]
    pub login: String,

    /// Webhook OAuth (app access token)
    #[arg(short, long)]
    pub app_token: String,

    /// User OAuth (user access token)
    #[arg(short, long)]
    pub user_token: String,
    // /// TTV broadcaster login/username
    // #[arg(short, long)]
    // pub broadcaster: String,
}

pub fn parse_cli_args() -> Arc<Cli> {
    let args = Arc::new(Cli::parse());

    // debug printer
    println!("[+] COMMAND LINE: {:?}", args);
    
    let args_clone = args.clone();
    for token in [&args_clone.app_token, &args_clone.user_token].iter() {
        match token.len() {
            TWITCH_OAUTH_LENGTH => continue,
            _ => {
                panic!(
                    "[x] EXPECTED OAUTH TOKEN LENGTH OF {} (got {})",
                    TWITCH_OAUTH_LENGTH,
                    token.len()
                );
            }
        }
    }

    args
}
