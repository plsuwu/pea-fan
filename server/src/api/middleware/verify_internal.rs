use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use http::header::AUTHORIZATION;
use http::{HeaderMap, StatusCode};

use crate::util::constant_time_cmp;
use crate::util::env::Var;
use crate::var;

// TODO:
//  we probably want to sign the POST body and verify it here, however
//  this should be fine for now...
pub async fn verify_internal_ident(req: Request, next: Next) -> Result<Response, StatusCode> {
    let headers = req.headers().clone();
    let authorized_header = headers
        .get(AUTHORIZATION)
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_str()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let internal_token = var!(Var::InternalToken)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !constant_time_cmp(authorized_header, internal_token) {
        Err(StatusCode::UNAUTHORIZED)
    } else {
        Ok(next.run(req).await)
    }
}
