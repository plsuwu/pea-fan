use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use sqlx::pool::PoolConnection;
use sqlx::postgres::PgDatabaseError;
use sqlx::prelude::FromRow;
use sqlx::{PgPool, Postgres};
use thiserror::Error;
use tokio::sync::{OnceCell, RwLock};

use crate::util::env::Var;
use crate::util::{env, helix};
use crate::var;

static DB_POOL: LazyLock<OnceCell<Db>> = LazyLock::new(OnceCell::new);
pub async fn db_pool() -> PgResult<&'static PgPool> {
    Ok(&DB_POOL
        .get_or_try_init(|| async { Db::new_pool().await })
        .await?
        .pool)
}

pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn new_pool() -> PgResult<Self> {
        let db_url = var!(Var::DatabaseUrl).await?;
        let pool = sqlx::PgPool::connect(&db_url).await?;

        Ok(Self { pool })
    }
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct DisplayableChatter {
    pub id: String,
    pub name: String,
    pub login: String,
    pub color: String,
    pub image: String,
    pub total: String,
    pub channels: Option<sqlx::types::Json<Vec<DisplayableScore>>>,
}

#[derive(Debug, Deserialize, Serialize, FromRow, Default)]
pub struct DisplayableChannel {
    pub id: String,
    pub name: String,
    pub login: String,
    pub color: String,
    pub image: String,
    pub total_as_chatter: i64,
    pub total_as_broadcaster: i64,
    pub chatters: Option<sqlx::types::Json<Vec<DisplayableChatter>>>,
}

#[derive(Debug, Deserialize, Serialize, FromRow, Default)]
pub struct DisplayableScore {
    pub channel: DisplayableChannel,
    pub chatter_id: String,
    pub score: i64,
    pub rank: i64,
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    #[serde(default = "default_page_offset")]
    pub offset: i64,
    #[serde(default = "default_page_max")]
    pub max: i64,
}

#[inline]
const fn default_page_offset() -> i64 {
    0
}

#[inline]
const fn default_page_max() -> i64 {
    15
}

pub type PgResult<T> = core::result::Result<T, PgErr>;

#[derive(Debug, Error)]
pub enum PgErr {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error(transparent)]
    SqlxDbError {
        #[from]
        source: sqlx::postgres::PgDatabaseError,
    },

    #[error("{0}")]
    HelixError(#[from] helix::HelixErr),

    #[error("{0}")]
    EnvError(#[from] env::EnvErr),
}
