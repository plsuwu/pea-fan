use std::sync::Arc;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use http::StatusCode;
use http::header::AUTHORIZATION;
use ring::digest;
use ring::rand::SecureRandom;
use sqlx::{Error, PgPool};

use crate::api::server::AppState;
use crate::db::models::Session;
use crate::db::{PgError, PgResult};
use crate::util::constant_time_cmp;
use crate::util::env::Var;
use crate::var;

// TODO:
//  we probably want to sign the POST body and verify it here, however
//  this should be fine for now...
pub async fn verify_initial_ident(req: Request, next: Next) -> Result<Response, StatusCode> {
    let headers = req.headers().clone();
    let authorized_header = headers
        .get(AUTHORIZATION)
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_str()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let internal_token = var!(Var::InternalToken)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if constant_time_cmp(authorized_header, internal_token) {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn verify_session_ident(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let db = state.database_pool;

    let headers = req.headers().clone();
    let ident = headers
        .get(AUTHORIZATION)
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_str()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let is_valid = SessionToken::cmp_token(db, ident)
        .await
        .map_err(|e| match e {
            SessionError::NoValidSessions => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    tracing::info!(token = ?is_valid, "validated token");

    // i dont think its possible to be a `String::Default()` here??
    if is_valid.token != String::default() {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error("no valid sessions")]
    NoValidSessions,

    #[error(transparent)]
    DatabaseError(#[from] PgError),
}

pub struct SessionToken;

impl SessionToken {
    pub fn new_token() -> String {
        let rng = ring::rand::SystemRandom::new();
        let mut bytes = [0u8; digest::SHA256_OUTPUT_LEN];

        rng.fill(&mut bytes).unwrap();
        let hex = hex::encode(bytes);

        tracing::info!(hex, "BUILT SESSION");

        hex
    }

    pub async fn store_token(db: &'static PgPool, token: &str) -> PgResult<()> {
        let ts_now = chrono::Utc::now();
        let ts_exp = ts_now.checked_add_days(chrono::Days::new(14)).unwrap();

        sqlx::query!(
            r#"
            INSERT INTO session (
                token,
                created_at,
                expires_at
            )
            VALUES ($1, $2, $3)
            "#,
            token,
            ts_now.naive_utc(),
            ts_exp.naive_utc()
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn get_matching_token(
        db: &'static PgPool,
        token: &str,
    ) -> Result<Option<Session>, SessionError> {
        match sqlx::query_as::<_, Session>(
            r#"
            SELECT * FROM session 
            WHERE token = $1 
            AND expires_at > NOW()
            "#,
        )
        .bind(token)
        .fetch_one(db)
        .await
        {
            Ok(val) => Ok(Some(val)),
            Err(e) => match e {
                Error::RowNotFound => {
                    tracing::warn!(error = ?e, "token invalid");
                    return Ok(None);
                }
                _ => {
                    tracing::error!(error = ?e, "failed to retrieve session from database");
                    Err(SessionError::DatabaseError(PgError::SqlxError(e)))
                }
            },
        }
    }

    pub async fn cmp_token(db: &'static PgPool, ident: &str) -> Result<Session, SessionError> {
        if let Some(valid_session_token) = SessionToken::get_matching_token(db, ident).await? {
            tracing::info!(?valid_session_token, "found valid token");
            return Ok(valid_session_token);
        }

        Err(SessionError::NoValidSessions)
    }
}
