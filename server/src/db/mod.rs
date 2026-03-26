use sqlx::PgPool;
use thiserror::Error;
use tokio::sync::OnceCell;

use crate::util::env::Var;
use crate::util::{env, helix};
use crate::var;

pub mod models;
pub mod pg;
pub mod redis;
pub mod repositories;

pub mod prelude {
    pub use crate::db::PgError;
    pub use crate::db::db_pool;

    pub use crate::db::models::channel::ChannelLeaderboardEntry;
    pub use crate::db::models::channel::{Channel, ChannelId};
    pub use crate::db::models::chatter::ChatterLeaderboardEntry;
    pub use crate::db::models::chatter::{Chatter, ChatterId};
    pub use crate::db::models::leaderboard::{Score, ScoreSummary};

    pub use crate::db::repositories::Repository;
    pub use crate::db::repositories::Tx;
    pub use crate::db::repositories::channel::ChannelRepository;
    pub use crate::db::repositories::chatter::ChatterRepository;
    pub use crate::db::repositories::leaderboard::LeaderboardRepository;
}

static DB_POOL: OnceCell<PgPool> = OnceCell::const_new();
pub async fn db_pool() -> PgResult<&'static sqlx::Pool<sqlx::Postgres>> {
    DB_POOL
        .get_or_try_init(|| async {
            let db_url = var!(Var::DatabaseUrl).await?;
            Ok(sqlx::PgPool::connect(db_url).await?)
        })
        .await
}

pub type PgResult<T> = core::result::Result<T, PgError>;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum PgError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error(transparent)]
    HelixError(#[from] helix::HelixErr),

    #[error(transparent)]
    EnvError(#[from] env::EnvErr),
}
