use core::fmt;
use std::sync::LazyLock;

use axum::body::{Body, Bytes};
use axum::extract::{FromRequest, Request};
use axum::middleware::Next;
use axum::response::Response;
use http::{HeaderMap, StatusCode};
use ring::digest;
use ring::hmac::{self, Key};
use ring::rand::SecureRandom;
use tokio::sync::OnceCell;

use super::{MiddlewareErr, MiddlewareResult};
use crate::util::constant_time_cmp;

static KEY: LazyLock<OnceCell<Hmac>> = LazyLock::new(OnceCell::new);
async fn get_hmac_struct() -> MiddlewareResult<&'static Hmac> {
    KEY.get_or_try_init(|| async { Hmac::new() }).await
}

pub async fn get_hmac_key() -> MiddlewareResult<String> {
    Ok(get_hmac_struct().await?.hex.clone())
}

#[allow(dead_code)]
pub struct Hmac {
    pub key: Key,
    pub digest: [u8; digest::SHA256_OUTPUT_LEN],
    pub hex: String,
}

impl Hmac {
    fn new() -> MiddlewareResult<Self> {
        let rng = ring::rand::SystemRandom::new();
        let mut digest = [0u8; digest::SHA256_OUTPUT_LEN];

        rng.fill(&mut digest)
            .map_err(|_| MiddlewareErr::UnspecifiedRingErr)?;

        let hex = hex::encode(digest);
        let key = Key::new(hmac::HMAC_SHA256, hex.as_bytes());

        Ok(Self { key, digest, hex })
    }
}

impl fmt::Display for Hmac {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.hex)
    }
}

#[derive(Clone)]
pub struct VerifiedBody(pub Bytes);

impl VerifiedBody {
    // pub fn as_bytes(&self) -> &Bytes {
    //     &self.0
    // }

    pub fn as_json<T>(&self) -> Result<T, serde_json::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_slice(&self.0)
    }
}

pub async fn verify_sender_ident(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let headers = req.headers().clone();
    let body = match extract_body(&mut req).await {
        Ok(bytes) => bytes,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    if let Err(status) = verify_signature(&headers, &body).await {
        tracing::error!(%status, "unable to verify external webhook signature");
        return Err(status);
    }

    req.extensions_mut().insert(VerifiedBody(body));
    Ok(next.run(req).await)
}

async fn extract_body(request: &mut Request) -> Result<Bytes, ()> {
    let body = std::mem::replace(request.body_mut(), Body::empty());
    axum::body::to_bytes(body, usize::MAX).await.map_err(|_| ())
}

async fn verify_signature(headers: &HeaderMap, body: &Bytes) -> Result<(), StatusCode> {
    let (id, timestamp, extern_signature) = get_message_parts(headers)?;
    let rebuilt_message = rebuild_message(id, timestamp, body);

    let expected_signature = {
        let key = &get_hmac_struct()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .key;

        let signed = hmac::sign(key, &rebuilt_message);
        format!("{}{}", HMAC_PREFIX, hex::encode(signed))
    };

    if constant_time_cmp(extern_signature, &expected_signature) {
        return Ok(());
    }

    Err(StatusCode::FORBIDDEN)
}

fn rebuild_message(id: &str, ts: &str, body: &Bytes) -> Vec<u8> {
    let mut m = Vec::new();
    m.extend_from_slice(id.as_bytes());
    m.extend_from_slice(ts.as_bytes());
    m.extend_from_slice(body);

    m
}

type MessageParts<'a> = (&'a str, &'a str, &'a str);
fn get_message_parts<'a>(headers: &'a HeaderMap) -> Result<MessageParts<'a>, StatusCode> {
    let id = headers
        .get(TWITCH_MESSAGE_ID)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let timestamp = headers
        .get(TWITCH_MESSAGE_TIMESTAMP)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let identifier = headers
        .get(TWITCH_MESSAGE_SIGNATURE)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    Ok((id, timestamp, identifier))
}

impl<S> FromRequest<S> for VerifiedBody
where
    S: Send + Sync,
{
    type Rejection = StatusCode;
    async fn from_request(req: Request, _: &S) -> Result<Self, Self::Rejection> {
        req.extensions()
            .get::<VerifiedBody>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub const HMAC_PREFIX: &str = "sha256=";
pub const TWITCH_MESSAGE_ID: &str = "Twitch-Eventsub-Message-Id";
pub const TWITCH_MESSAGE_TIMESTAMP: &str = "Twitch-Eventsub-Message-Timestamp";
pub const TWITCH_MESSAGE_SIGNATURE: &str = "Twitch-Eventsub-Message-Signature";
pub const TWITCH_MESSAGE_TYPE_HEADER: &str = "Twitch-Eventsub-Message-Type";
