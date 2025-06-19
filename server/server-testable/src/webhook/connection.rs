use thiserror;

#[derive(Debug, Clone)]
pub struct WebhookEnv {
    pub base_url: String,
    pub app_token: String,
    pub callback_url: String,
}

impl WebhookEnv {
    pub fn read_env() -> Result<Self, Box<dyn std::error::Error>> {
        let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        match env.as_str() {
            "production" => Ok(Self {
                base_url: std::env::var("PRODUCTION_WEBHOOK_URL")?,
                app_token: std::env::var("PRODUCTION_APP_TOKEN")?,
                callback_url: std::env::var("PRODUCTION_CALLBACK_URL")?,
            }),

            "testing" => Ok(Self {
                base_url: std::env::var("TESTING_WEBHOOK_URL")?,
                app_token: std::env::var("TESTING_APP_TOKEN")?,
                callback_url: std::env::var("TESTING_CALLBACK_URL")?,
            }),

            _ => Ok(Self {
                base_url: std::env::var("DEVELOPMENT_WEBHOOK_URL")?,
                app_token: std::env::var("DEVELOPMENT_APP_TOKEN")?,
                callback_url: std::env::var("DEVELOPMENT_CALLBACK_URL")?,
            }),
        }
    }
}
