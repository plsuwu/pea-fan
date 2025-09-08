use async_trait::async_trait;
use axum::extract::FromRequest;
use axum::http::StatusCode;
use axum::middleware;
use axum::{extract, response};
use core::fmt;
use ring::hmac;
use std::arch::asm;
use std::hint::black_box;
use std::sync::{Arc, LazyLock, RwLock};
use thiserror::Error;
use tracing::error;

use crate::hook::middleware::SHA256_PREFIX;
use crate::hook::middleware::TWITCH_MESSAGE_ID;
use crate::hook::middleware::TWITCH_MESSAGE_SIGNATURE;
use crate::hook::middleware::TWITCH_MESSAGE_TIMESTAMP;

pub static KEY: LazyLock<Arc<RwLock<Hmac>>> = LazyLock::new(|| Arc::new(RwLock::new(Hmac::new())));

pub type MiddlewareResult<T> = core::result::Result<T, StatusCode>;
type MessageParts<'a> = (&'a str, &'a str, &'a str);

#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("unable to verify webhook signature")]
    InvalidSignature,
}

pub trait Key: Send + Sync + fmt::Debug {
    fn new() -> Self;
}

#[derive(Debug)]
pub struct Hmac {
    pub key: ring::hmac::Key,
    pub digest: [u8; ring::digest::SHA256_OUTPUT_LEN],
    pub hex: String,
}

impl Key for Hmac {
    fn new() -> Self {
        let rng = ring::rand::SystemRandom::new();
        let digest: [u8; ring::digest::SHA256_OUTPUT_LEN] =
            ring::rand::generate(&rng).unwrap().expose();

        let key = ring::hmac::Key::new(hmac::HMAC_SHA256, &digest);
        let hex = hex::encode(digest);

        Self { key, digest, hex }
    }
}

#[derive(Clone)]
pub struct VerifiedBody(pub axum::body::Bytes);

impl VerifiedBody {
    pub fn as_bytes(&self) -> &axum::body::Bytes {
        &self.0
    }

    pub fn as_json<T>(&self) -> Result<T, serde_json::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_slice(&self.0)
    }
}

pub async fn verify(
    mut request: extract::Request,
    next: middleware::Next,
) -> MiddlewareResult<response::Response> {
    let headers = request.headers().clone();
    let body = extract_body(&mut request)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    verify_signature(&headers, &body)?;
    request.extensions_mut().insert(VerifiedBody(body));

    Ok(next.run(request).await)
}

async fn extract_body(request: &mut extract::Request) -> Result<axum::body::Bytes, ()> {
    let body = std::mem::replace(request.body_mut(), axum::body::Body::empty());
    axum::body::to_bytes(body, usize::MAX).await.map_err(|_| ())
}

fn verify_signature(headers: &http::HeaderMap, body: &axum::body::Bytes) -> MiddlewareResult<()> {
    let (id, ts, rx) = get_message_parts(headers)?;

    let message = build_message(id, ts, body);
    let calc = {
        let guard = &*KEY.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let signature = hmac::sign(&guard.key, &message);

        format!("{}{}", SHA256_PREFIX, hex::encode(&signature))
    };

    match constant_time_cmp_asm(&calc, &rx) {
        true => Ok(()),
        false => Err(StatusCode::FORBIDDEN),
    }
}

fn get_message_parts<'a>(headers: &'a http::HeaderMap) -> MiddlewareResult<MessageParts<'a>> {
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

fn build_message(id: &str, ts: &str, body: &axum::body::Bytes) -> Vec<u8> {
    let mut m = Vec::new();

    m.extend_from_slice(id.as_bytes());
    m.extend_from_slice(ts.as_bytes());
    m.extend_from_slice(body);

    m
}

fn constant_time_cmp_asm(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let a = a.as_bytes();
    let b = b.as_bytes();
    let mut res = 0u8;

    for i in 0..a.len() {
        let left = black_box(&a[i]) as *const u8;
        let right = black_box(&b[i]) as *const u8;

        unsafe {
            asm!(
                "mov {tmp}, [{a_ptr}]",
                "xor {tmp}, [{b_ptr}]",
                "or {res}, {tmp}",
                a_ptr = in(reg) left,
                b_ptr = in(reg) right,
                tmp = out(reg_byte) _,
                res = inout(reg_byte) res,
                options(pure, nomem, nostack)
            );
        }
    }

    res == 0
}

impl<S> FromRequest<S> for VerifiedBody
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request(
        req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        req.extensions()
            .get::<VerifiedBody>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_cmp() {
        let expect = "test_secret_string";
        let passing = "test_secret_string";

        let bad_at_start = "-est_secret_string";
        let bad_at_end = "test_secret_strin-";

        let shorter = "test_s";
        let longer = "test_secret_string_test";

        assert!(constant_time_cmp_asm(expect, passing));
        assert!(!constant_time_cmp_asm(expect, bad_at_start));
        assert!(!constant_time_cmp_asm(expect, bad_at_end));
        assert!(!constant_time_cmp_asm(expect, shorter));
        assert!(!constant_time_cmp_asm(expect, longer));
    }
}
