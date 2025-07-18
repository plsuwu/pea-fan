use crate::constants::HMAC_PREFIX;
use crate::constants::{TWITCH_MESSAGE_ID, TWITCH_MESSAGE_SIGNATURE, TWITCH_MESSAGE_TIMESTAMP};
use axum::body::{Body, Bytes};
use axum::extract::{FromRequest, Request};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use ring::digest;
use ring::hmac::{self, Key};
use ring::rand;
use std::fmt;
use std::sync::{LazyLock, RwLock};

/// Struct for HMAC key storage and generation methods.
///
/// Key is stored in-memory for the duration of the server's uptime; restarting the server should
/// reset this key (this mechanism is, at present, by design).
///
/// # Security
///
/// In a vacuum, `ring::rand::SystemRandom` is cryptographically secure (as far as I am aware
/// smile).
#[derive(Debug)]
pub struct Secret {
    pub key: Key,
    _digest: [u8; digest::SHA256_OUTPUT_LEN],
    pub _hex: String,
}

pub static KEY_DIGEST: LazyLock<RwLock<Secret>> = LazyLock::new(|| RwLock::new(Secret::new()));

impl Secret {
    #[cfg(feature = "production")]
    pub fn new() -> Self {
        let rng = rand::SystemRandom::new();
        let _digest: [u8; digest::SHA256_OUTPUT_LEN] = rand::generate(&rng).unwrap().expose();
        let _hex = hex::encode(_digest);

        let key = Key::new(hmac::HMAC_SHA256, _hex.as_bytes());

        Self { _digest, _hex, key }
    }

    #[cfg(not(feature = "production"))]
    pub fn new() -> Self {
        let rng = rand::SystemRandom::new();
        let _digest: [u8; digest::SHA256_OUTPUT_LEN] = rand::generate(&rng).unwrap().expose();
        let _hex = "f2ffb7656e27b3076c57add06e58621668ab497e8992a9ccbd6f18eb400db094".to_string();

        let key = Key::new(hmac::HMAC_SHA256, _hex.as_bytes());
        Self { _digest, _hex, key }
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self._hex)
    }
}

#[derive(Clone)]
pub struct VerifiedBody(pub Bytes);

#[allow(dead_code)]
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

pub async fn verify_sender_ident(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    // we need a copy of the headers as the body extraction directly
    // following this statment consumes the request's content
    let headers = request.headers().clone();
    let body = match extract_body(&mut request).await {
        Ok(bytes) => bytes,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    if let Err(status) = verify_signature(&headers, &body) {
        eprintln!("[x] unable to verify webhook signature!");
        return Err(status);
    };

    request.extensions_mut().insert(VerifiedBody(body));
    Ok(next.run(request).await)
}

async fn extract_body(request: &mut Request) -> Result<Bytes, ()> {
    let body = std::mem::replace(request.body_mut(), Body::empty());
    axum::body::to_bytes(body, usize::MAX).await.map_err(|_| ())
}

fn verify_signature(headers: &HeaderMap, body: &Bytes) -> Result<(), StatusCode> {
    let (id, ts, received) = get_message_parts(headers)?;

    let message = build_message(id, ts, body);
    let calculated = {
        let guard = &*KEY_DIGEST
            .read()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let signature = hmac::sign(&guard.key, &message);
        format!("{}{}", HMAC_PREFIX, hex::encode(signature.as_ref()))
    };

    if timing_safe_eq(&calculated, &received) {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

type MessageParts<'a> = (&'a str, &'a str, &'a str);
fn get_message_parts<'a>(headers: &'a HeaderMap) -> Result<MessageParts<'a>, StatusCode> {
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

fn build_message(id: &str, ts: &str, body: &Bytes) -> Vec<u8> {
    let mut m = Vec::new();
    m.extend_from_slice(id.as_bytes());
    m.extend_from_slice(ts.as_bytes());
    m.extend_from_slice(body);

    m
}

fn timing_safe_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }

    left.as_bytes()
        .iter()
        .zip(right.as_bytes().iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

impl<S> FromRequest<S> for VerifiedBody
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request(req: Request<Body>, _state: &S) -> Result<Self, Self::Rejection> {
        req.extensions()
            .get::<VerifiedBody>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
