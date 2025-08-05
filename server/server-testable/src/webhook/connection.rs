//! Woefully incomplete! Make sure this is correctly implemented at some nonspecific stage :))

use std::error::Error;

use thiserror;

#[derive(Debug, Clone)]
pub struct WebhookEnv {
    pub app_token: String,
    pub url: String,
}


