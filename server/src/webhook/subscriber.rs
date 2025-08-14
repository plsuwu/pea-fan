use async_trait::async_trait;
use axum::response::sse::Event;
use http::{
    HeaderMap, StatusCode,
    header::{AUTHORIZATION, InvalidHeaderValue},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tracing::{debug, info, instrument, warn};

use crate::webhook::{
    middleware::verify::{SESSION_KEY, SessionKey},
    types::{EventType, WebhookRequest},
};

const HELIX_BASE: &str = "https://api.twitch.tv/helix";
// const CALLBACK_ROUTE: &str = "https://ff4be2d2d5e6.ngrok-free.app";

// const CALLBACK_ROUTE: &str = "http://localhost/webhook-global"; // <-- get something proper for this :))
// const CALLBACK_ROUTE: &str = "https://api.piss.fan/webhook-global";

#[derive(Error, Debug)]
pub enum HookHandlerError {
    #[error("Failed to fetch an updated channel list: {0}")]
    ChannelError(#[from] reqwest::Error),

    #[error("Failed to read .env: {0}")]
    EnvError(#[from] dotenvy::Error),

    #[error("Environment variable not a valid header value: {0}")]
    InvalidHeaderValid(#[from] InvalidHeaderValue),

    #[error("Error during deserialization: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("There appears to be no active webhook subscriptions.")]
    EmptyWebHookArray,

    #[error("Error response code from subscription creation endpoint: {0}")]
    SubscriptionCreateError(Value),
}

pub type HookHandlerResult<T> = core::result::Result<T, HookHandlerError>;

#[async_trait]
pub trait Subscriber {
    // async fn init_hooks(&self, broadcasters: &Vec<String>) -> HookHandlerResult<()>;
    async fn startup(&self) -> HookHandlerResult<()>;
    async fn create(
        &self,
        broadcaster: &str,
        notification: EventType,
        // key: &str,
    ) -> HookHandlerResult<Value>;
    async fn delete(&self, subscription_id: &str) -> HookHandlerResult<()>;
    async fn get_current(&self) -> Option<Vec<Value>>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HookHandler {
    pub channels: Vec<String>,
    pub secrets: Env,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct Env {
    app_token: String,
    user_token: String,
    client_id: String,
    client_secret: String,
}

impl Env {
    fn get() -> HookHandlerResult<Self> {
        match dotenvy::var("ENVIRONMENT")?.as_str() {
            "PRODUCTION" => {
                let app_token = dotenvy::var("APP_TOKEN")?;
                let user_token = dotenvy::var("USER_TOKEN")?;
                let client_id = dotenvy::var("CLIENT_ID")?;
                let client_secret = dotenvy::var("CLIENT_SECRET")?;

                Ok(Self {
                    app_token,
                    user_token,
                    client_id,
                    client_secret,
                })
            }
            _ => {
                let app_token = dotenvy::var("STAGING_APP_TOKEN")?;
                let user_token = dotenvy::var("STAGING_USER_TOKEN")?;
                let client_id = dotenvy::var("STAGING_CLIENT_ID")?;
                let client_secret = dotenvy::var("STAGING_CLIENT_SECRET")?;

                Ok(Self {
                    app_token,
                    user_token,
                    client_id,
                    client_secret,
                })
            }
        }
    }

    fn build_headers(&self) -> HookHandlerResult<HeaderMap> {
        let bearer = format!("Bearer {}", self.app_token);
        let client_id = self.client_id.clone();

        let mut headers = HeaderMap::new();
        headers.insert("client-id", client_id.try_into()?);
        headers.insert(AUTHORIZATION, bearer.try_into()?);

        Ok(headers)
    }
}

impl HookHandler {
    pub async fn new() -> HookHandlerResult<Self> {
        let mut handler = Self {
            channels: Vec::new(),
            secrets: Env::get()?,
        };

        handler.refresh_channels().await?;
        Ok(handler)
    }

    pub async fn refresh_channels(&mut self) -> HookHandlerResult<()> {
        self.channels = super::get_tracked_channels().await?;
        Ok(())
    }
}

#[async_trait]
impl Subscriber for HookHandler {
    #[instrument(skip(self))]
    async fn startup(&self) -> HookHandlerResult<()> {
        if let Some(active) = self.get_current().await {
            debug!("ACTIVE: {:?}", active);
            _ = futures_util::future::join_all(
                active
                    .iter()
                    .map(async |sub_val: &serde_json::Value| {
                        let sub_id = sub_val["id"].as_str().unwrap();
                        info!("Deleting subscription '{}'", sub_id);

                        self.delete(sub_id).await.unwrap();
                    })
                    .collect::<Vec<_>>(),
            )
            .await;
        };

        let key = SESSION_KEY.get_hex_key();
        let mut handles = Vec::new();

        for brd in self.channels.iter() {
            let on = self.create(&brd, EventType::StreamOnline).await?;
            let off = self.create(&brd, EventType::StreamOffline).await?;

            handles.push(on);   
            handles.push(off);
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn create(
        &self,
        broadcaster: &str,
        notification: EventType,
        // key: &str,
    ) -> HookHandlerResult<Value> {
        let client = reqwest::Client::new();
        let headers = self.secrets.build_headers()?;
        let subs_uri = format!("{}/eventsub/subscriptions", HELIX_BASE);

        let body = match notification {
            EventType::StreamOnline => WebhookRequest::stream_online(
                broadcaster,
                CALLBACK_ROUTE,
                &self.secrets.client_secret,
            ),
            EventType::StreamOffline => WebhookRequest::stream_offline(
                broadcaster,
                CALLBACK_ROUTE,
                &self.secrets.client_secret,
            ),
        };

        // let body = WebhookRequest::new(
        //     notification,
        //     broadcaster,
        //     CALLBACK_ROUTE,
        //     self.secrets.client_secret.clone(),
        // );

        let req = client.post(subs_uri).json(&body).headers(headers);
        let res = req.send().await?;

        if res.status() != 200 && res.status() != 202 {
            match res.status() {
                // StatusCode::CONFLICT => {
                //     // TODO: revoke and retry + implement like a backoff or something
                //     //       > will i ever bother doing this? probably not who knows :3
                // }
                _ => {
                    let err: Value = serde_json::from_str(&res.text().await?)?;
                    return Err(HookHandlerError::SubscriptionCreateError(err));
                }
            }
        } else {
            let deser: Value = serde_json::from_str(&res.text().await?)?;
            let status = &deser["data"][0]["status"].as_str().unwrap();
            let sub_type = &deser["data"][0]["type"].as_str().unwrap();

            let broadcaster_id = &deser["data"][0]["condition"]["broadcaster_user_id"]
                .as_str()
                .unwrap();

            info!(
                "Got status '{}': {} (for uid '{}')",
                status, sub_type, broadcaster_id
            );
            Ok(deser)
        }
    }

    #[instrument(skip(self))]
    async fn delete(&self, subscription_id: &str) -> HookHandlerResult<()> {
        let client = reqwest::Client::new();
        let headers = self.secrets.build_headers()?;
        let subs_uri = format!(
            "{}/eventsub/subscriptions?id={}",
            HELIX_BASE, subscription_id
        );

        let res = client.delete(subs_uri).headers(headers).send().await;
        match res {
            Ok(_) => info!("Subscription '{}' deletion ok", subscription_id),
            Err(e) => warn!("Subscription '{}' deletion failure: {e}", subscription_id),
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_current(&self) -> Option<Vec<Value>> {
        let client = reqwest::Client::new();
        let subs_uri = format!("{}/eventsub/subscriptions?status=enabled", HELIX_BASE);
        let headers = self.secrets.build_headers().ok()?;

        let req = client.get(subs_uri).headers(headers);
        let res = req.send().await.ok()?;

        let mut deser: Value = serde_json::from_str(&res.text().await.ok()?).ok()?;
        if let Some(_) = deser["total"].take().as_u64() {
            let maybe_data: Result<Vec<Value>, serde_json::Error> =
                serde_json::from_value(deser["data"].clone());

            if let Ok(data_array) = maybe_data {
                return Some(data_array);
            }
        }

        None
    }
}
