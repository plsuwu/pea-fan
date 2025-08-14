use axum::body::{Body, Bytes};
use axum::extract::{FromRequest, Request};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use ring::digest;
use ring::hmac::{self, Key};
use ring::rand;
use std::fmt;
use std::sync::{Arc, LazyLock, Mutex};

pub type MessageParts<'a> = (&'a str, &'a str, &'a str);
pub type VerifiedResult<T> = core::result::Result<T, axum::http::StatusCode>;

pub const HMAC_PREFIX: &'static str = "sha256=";
pub const TWITCH_MESSAGE_ID: &'static str = "Twitch-Eventsub-Message-Id";
pub const TWITCH_MESSAGE_TIMESTAMP: &'static str = "Twitch-Eventsub-Message-Timestamp";
pub const TWITCH_MESSAGE_SIGNATURE: &'static str = "Twitch-Eventsub-Message-Signature";
pub const TWITCH_MESSAGE_TYPE_HEADER: &str = "Twitch-Eventsub-Message-Type";

pub static SESSION_KEY: LazyLock<SessionKey> = LazyLock::new(|| SessionKey::new());

#[derive(Debug)]
pub struct SessionKeyInner {
    pub key: Key,
    pub hex: String,
    _digest: [u8; digest::SHA256_OUTPUT_LEN],
}

impl SessionKeyInner {
    pub fn new() -> Self {
        let rng = rand::SystemRandom::new();
        let _digest: [u8; digest::SHA256_OUTPUT_LEN] = rand::generate(&rng).unwrap().expose();
        let hex = hex::encode(_digest);
        let key = Key::new(hmac::HMAC_SHA256, hex.as_bytes());

        Self { key, hex, _digest }
    }
}

pub struct SessionKey {
    inner: Arc<Mutex<SessionKeyInner>>,
}

impl SessionKey {
    pub fn new() -> Self {
        let inner = Arc::new(Mutex::new(SessionKeyInner::new()));
        Self { inner }
    }

    pub fn sign(&self, message: &Vec<u8>) -> ring::hmac::Tag {
        let inner_lock = self.inner.lock().unwrap();
        hmac::sign(&inner_lock.key, message)
    }

    pub fn get_hex_key<'a>(&'a self) -> String {
        let inner_lock = self.inner.lock().unwrap();
        inner_lock.hex.clone()
    }
}

impl fmt::Display for SessionKeyInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hex)
    }
}

#[derive(Clone)]
pub struct VerifiedBody(pub Bytes);

impl VerifiedBody {
    pub fn as_bytes(&self) -> &Bytes {
        &self.0
    }

    pub fn as_json<T>(&self) -> Result<T, serde_json::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_slice(&self.0)
    }
}

impl<S> FromRequest<S> for VerifiedBody
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        req.extensions()
            .get::<VerifiedBody>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub async fn sender_ident(mut request: Request, next: Next) -> VerifiedResult<Response> {
    let headers = request.headers().clone();
    let body = extract_body(&mut request).await?;

    verify_signature(&headers, &body)?;

    request.extensions_mut().insert(VerifiedBody(body));
    Ok(next.run(request).await)
}

pub async fn extract_body(request: &mut Request) -> VerifiedResult<Bytes> {
    let body = std::mem::replace(request.body_mut(), Body::empty());
    axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)
}

fn get_parts<'a>(headers: &'a HeaderMap) -> VerifiedResult<MessageParts<'a>> {
    let id = headers
        .get(TWITCH_MESSAGE_ID)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let ts = headers
        .get(TWITCH_MESSAGE_TIMESTAMP)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let received = headers
        .get(TWITCH_MESSAGE_SIGNATURE)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    Ok((id, ts, received))
}

fn get_unsigned_message(id: &str, ts: &str, body: &Bytes) -> Vec<u8> {
    let mut msg = Vec::new();
    msg.extend_from_slice(id.as_bytes());
    msg.extend_from_slice(ts.as_bytes());
    msg.extend_from_slice(body);

    msg
}

fn verify_signature(headers: &HeaderMap, body: &Bytes) -> VerifiedResult<()> {
    let (id, ts, rx) = get_parts(headers)?;
    let message = get_unsigned_message(id, ts, body);
    let calculated_hash = {
        let signature = SESSION_KEY.sign(&message);
        format!("{}{}", HMAC_PREFIX, hex::encode(&signature))
    };

    const_equal(&calculated_hash, &rx)
}

fn const_equal(left: &str, right: &str) -> VerifiedResult<()> {
    if left.len() != right.len() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if left
        .as_bytes()
        .iter()
        .zip(right.as_bytes().iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
    {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
