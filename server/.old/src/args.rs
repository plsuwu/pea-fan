use clap::Parser;
use std::sync::{Arc, LazyLock};

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
}

pub static COMMAND_LINE_ARGS: LazyLock<ReadableArgs> = LazyLock::new(|| ReadableArgs::new());

impl Cli {
    pub fn new() -> Arc<Cli> {
        Arc::new(Cli::parse())
    }
}

pub struct ReadableArgs {
    inner: Arc<Cli>,
}

impl ReadableArgs {
    pub fn new() -> Self {
        Self { inner: Cli::new() }
    }

    pub fn read(&self) -> Arc<Cli> {
        Arc::clone(&self.inner)
    }
}

pub fn get_cli_args() -> Arc<Cli> {
    COMMAND_LINE_ARGS.read()
}
