// pub mod schema;

use async_trait::async_trait;
use sqlx::{PgPool, Pool};
use thiserror::Error;

pub type DbResult<T> = core::result::Result<T, DatabaseError>;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("dotenvy error: {0}")]
    DotenvyError(#[from] dotenvy::Error),
}

pub struct DatabaseLayer {
    pub pool: PgPool,
}

// #[async_trait]
impl DatabaseLayer {
    pub async fn get() -> DbResult<Self> {
        let db_url = match dotenvy::var("ENVIRONMENT")?.as_str() {
            "DEVELOPMENT" => dotenvy::var("TEST_DATABASE_URL")?,
            _ => dotenvy::var("DATABASE_URL")?,
        };

        let pool = Pool::connect(&db_url).await?;

        Ok(Self { pool })
    }
}
