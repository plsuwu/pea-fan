use clap::Parser;

const TWITCH_OAUTH_LENGTH: usize = 30;

#[derive(Parser, Debug)]
pub struct Cli {
    /// TTV user/bot login/username
    #[arg(short, long)]
    pub login: String,

    /// IRC oauth (this might be implemented via a bot idk)
    #[arg(short, long)]
    pub oauth_token: String,

    /// TTV broadcaster login/username
    #[arg(short, long)]
    pub broadcaster: String,
}

pub fn parse_cli_args() -> Cli {
    let args = Cli::parse();

    // debug printer
    println!("[+] COMMAND LINE: {:?}", args);

    match args.oauth_token.len() {
        TWITCH_OAUTH_LENGTH => return args,
        _ => {
            panic!(
                "[x] EXPECTED OAUTH TOKEN LENGTH OF {} (got {})",
                TWITCH_OAUTH_LENGTH,
                args.oauth_token.len()
            );
        }
    }
}
