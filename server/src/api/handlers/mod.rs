use crate::api::server::RouteError;

pub mod admin;
// pub mod admin_old;
pub mod channel;
pub mod chatter;

/// Wraps a Tokio task with the `RouteError::JoinError` return type.
///
/// Intended for use in handler tasks that should always execute to completion (regardless of
/// whether the client remains connected).
pub async fn spawn_protected<F, T>(f: F) -> Result<T, RouteError>
where
    F: Future<Output = Result<T, RouteError>> + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(f).await.map_err(RouteError::JoinError)?
}
