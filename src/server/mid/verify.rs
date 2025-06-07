use axum::{
    body::{Body, Bytes},
    extract::{FromRequest, Request},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use ring::hmac;

use crate::server::subscriber::KEY_DIGEST;

const HMAC_PREFIX: &'static str = "sha256=";
const TWITCH_MESSAGE_ID: &'static str = "Twitch-Eventsub-Message-Id";
const TWITCH_MESSAGE_TIMESTAMP: &'static str = "Twitch-Eventsub-Message-Timestamp";
const TWITCH_MESSAGE_SIGNATURE: &'static str = "Twitch-Eventsub-Message-Signature";
const TWITCH_MESSAGE_TYPE: &'static str = "Twitch-Eventsub-Message-Type";

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

pub async fn verify_sender_ident(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
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

    // println!("[+] signature ok");

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
    // println!("\t - calculated: \t'{}'", left);
    // println!("\t - actual: \t'{}'", right);

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
