use std::sync::{Arc, LazyLock};

use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum EnvError {
    #[error("error while fetching .env variables: {0}")]
    DotenvyError(#[from] dotenvy::Error),
}

pub type EnvResult<T> = core::result::Result<T, EnvError>;

pub static ENV_SECRETS: LazyLock<EnvLock> = LazyLock::new(|| EnvLock::new());

#[derive(Debug)]
pub struct EnvLock {
    pub inner: Arc<Env>,
}

impl EnvLock {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Env::init().unwrap()),
        }
    }

    pub fn get(&self) -> Arc<Env> {
        Arc::clone(&self.inner)
    }

    pub fn app_token(&self) -> &str {
        &self.inner.app_token()
    }

    pub fn user_token(&self) -> &str {
        &self.inner.user_token()
    }

    pub fn client_id(&self) -> &str {
        &self.inner.client_id()
    }

    pub fn client_secret(&self) -> &str {
        &self.inner.client_secret()
    }

    pub fn user_login(&self) -> &str {
        &self.inner.user_login()
    }

    pub fn global_client_id(&self) -> &str {
        &self.inner.global_client_id()
    }

    pub fn redis_host(&self) -> &str {
        &self.inner.redis_host()
    }

    pub fn redis_port(&self) -> &str {
        &self.inner.redis_port()
    }

    pub fn pg_url(&self) -> &str {
        &self.inner.pg_url()
    }
}

#[derive(Debug, Clone)]
pub struct Env {
    pub app_token: String,
    pub user_token: String,
    pub client_id: String,
    pub global_client_id: String,
    pub user_login: String,
    pub client_secret: String,
    pub redis_host: String,
    pub redis_port: String,
    pub pg_url: String,
}

impl Env {
    pub fn init() -> EnvResult<Self> {
        let env = match dotenvy::var("ENVIRONMENT")?.as_str() {
            "PRODUCTION" => Ok(Self {
                app_token: dotenvy::var("APP_TOKEN")?,
                user_token: dotenvy::var("USER_TOKEN")?,
                client_id: dotenvy::var("CLIENT_ID")?,
                client_secret: dotenvy::var("CLIENT_SECRET")?,
                user_login: dotenvy::var("USER_LOGIN")?,
                global_client_id: dotenvy::var("GLOBAL_CLIENT_ID")?,
                redis_host: dotenvy::var("REDIS_HOST")?,
                redis_port: dotenvy::var("REDIS_PORT")?,
                pg_url: dotenvy::var("DATABASE_URL")?,
            }),
            _ => Ok(Self {
                app_token: dotenvy::var("STAGING_APP_TOKEN")?,
                user_token: dotenvy::var("STAGING_USER_TOKEN")?,
                client_id: dotenvy::var("STAGING_CLIENT_ID")?,
                client_secret: dotenvy::var("STAGING_CLIENT_SECRET")?,
                user_login: dotenvy::var("STAGING_USER_LOGIN")?,
                global_client_id: dotenvy::var("GLOBAL_CLIENT_ID")?,
                redis_host: dotenvy::var("STAGING_REDIS_HOST")?,
                redis_port: dotenvy::var("STAGING_REDIS_PORT")?,
                pg_url: dotenvy::var("STAGING_DATABASE_URL")?,
            }),
        };

        info!("env: {:?}", env);

        env
    }

    pub fn app_token(&self) -> &str {
        &self.app_token
    }

    pub fn user_token(&self) -> &str {
        &self.user_token
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }

    pub fn user_login(&self) -> &str {
        &self.user_login
    }

    pub fn global_client_id(&self) -> &str {
        &self.global_client_id
    }

    pub fn redis_host(&self) -> &str {
        &self.redis_host
    }

    pub fn redis_port(&self) -> &str {
        &self.redis_port
    }

    pub fn pg_url(&self) -> &str {
        &self.pg_url
    }
}
