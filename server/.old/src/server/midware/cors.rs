use http::Method;
use http::{HeaderValue, request::Parts as RequestParts};
use tower_http::cors::{AllowOrigin, CorsLayer};

#[cfg(feature = "production")]
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(AllowOrigin::predicate(
            |org: &HeaderValue, _rq_pts: &RequestParts| {
                org.as_bytes()
                    .ends_with(crate::constants::ORIGIN_URL_ENDSWITH)
            },
        ))
}

#[cfg(not(feature = "production"))]
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(AllowOrigin::any())
}
