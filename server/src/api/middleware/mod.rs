pub mod verify_external;
pub mod verify_internal;

use http::request::Parts as ReqParts;
use http::{HeaderValue, Method};
use thiserror::Error;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::util::env::{EnvErr, Var};
use crate::var;

pub type MiddlewareResult<T> = core::result::Result<T, MiddlewareErr>;

#[derive(Debug, Error)]
pub enum MiddlewareErr {
    #[error(transparent)]
    EnvErr(#[from] EnvErr),

    #[error("ring::error::Unspecified error occurred")]
    UnspecifiedRingErr,
}

pub async fn cors_layer() -> CorsLayer {
    let cors_allowed = var!(Var::CorsAllowOrigins).await.unwrap_or("*");

    let allowed = if cors_allowed == "*" {
        AllowOrigin::any()
    } else {
        AllowOrigin::predicate(|org: &HeaderValue, _: &ReqParts| {
            org.as_bytes().ends_with(cors_allowed.as_bytes())
        })
    };

    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(allowed)
}
