//! Woefully incomplete! Make sure this is correctly implemented at some nonspecific stage :))

use std::error::Error;

use thiserror;

#[derive(Debug, Clone)]
pub struct WebhookEnv {
    pub base_url: String,
    pub app_token: String,
    pub callback_url: String,
}

impl WebhookEnv {
    pub fn read_env() -> Result<(), Box<dyn Error>> {
        let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        println!("env: {:?}", env);

        Ok(())
    }
}
