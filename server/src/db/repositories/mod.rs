#![allow(unused_assignments, dead_code)]

use core::fmt;

use async_trait::async_trait;
use sqlx::{Pool, Postgres, Result as SqlxResult, Transaction};
use tracing::instrument;

use crate::db::models::channel::ChannelId;
use crate::db::models::chatter::ChatterId;
use crate::db::prelude::{Channel, Chatter, ScoreSummary};

pub mod channel;
pub mod chatter;
pub mod leaderboard;

pub struct Tx<'a> {
    inner: Option<Transaction<'a, Postgres>>,
}

impl<'a> Tx<'a> {
    /// "Automatic" transaction handler
    ///
    /// # Usage
    ///
    /// God help me
    #[instrument(skip(pool, f))]
    pub async fn with_tx<F, Fut, T>(pool: &'static Pool<Postgres>, f: F) -> SqlxResult<T>
    where
        F: FnOnce(Tx<'a>) -> Fut,
        Fut: Future<Output = (Tx<'a>, SqlxResult<T>)>,
    {
        let tx = Self::begin(pool).await?;
        let (mut tx, result) = f(tx).await;

        match result {
            Ok(val) => {
                tx.commit().await?;
                Ok(val)
            }
            Err(e) => {
                tracing::trace!(error = ?e, "transacted query failure");
                Err(e)
            }
        }
    }

    #[instrument(skip(self, item))]
    pub async fn insert_chatter(&mut self, item: &Chatter) -> SqlxResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO chatter (
                id,
                login,
                name,
                color,
                image,
                total,
                private,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, false, $7, $8)
            ON CONFLICT (id)
            DO UPDATE SET
                login = $2,
                name = $3,
                color = $4,
                image = $5,
                updated_at = NOW()
            "#,
            &item.id.to_string(),
            item.login,
            item.name,
            item.color,
            item.image,
            item.total,
            item.created_at,
            item.updated_at
        )
        .execute(&mut **self.inner_mut()?)
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn insert_channel(&mut self, item: &Channel) -> SqlxResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO channel (
                id,
                channel_total,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id)
            DO NOTHING
            "#,
            &item.id.to_string(),
            item.channel_total,
            item.created_at,
            item.updated_at,
        )
        .execute(&mut **self.inner_mut()?)
        .await?;

        Ok(())
    }

    #[instrument(skip(pool))]
    pub async fn begin(pool: &'static Pool<Postgres>) -> SqlxResult<Self> {
        let inner = pool.begin().await?;
        Ok(Self { inner: Some(inner) })
    }

    #[instrument(skip(self))]
    pub async fn commit(&mut self) -> SqlxResult<()> {
        if let Some(tx) = self.inner.take() {
            tx.commit().await
        } else {
            Err(sqlx::Error::Protocol(
                "Transaction already completed".into(),
            ))
        }
    }

    fn inner_mut(&mut self) -> SqlxResult<&mut Transaction<'a, Postgres>> {
        self.inner
            .as_mut()
            .ok_or_else(|| sqlx::Error::Protocol("Transaction already completed".into()))
    }

    #[instrument(skip(self))]
    pub async fn rollback(&mut self) -> SqlxResult<()> {
        if let Some(tx) = self.inner.take() {
            tx.rollback().await
        } else {
            Err(sqlx::Error::Protocol(
                "Transaction already completed".into(),
            ))
        }
    }

    #[instrument(skip(self, chatter_id, channel_id))]
    pub async fn increment_score(
        &mut self,
        chatter_id: &ChatterId,
        channel_id: &ChannelId,
    ) -> SqlxResult<ScoreSummary> {
        self.increment_score_by(chatter_id, channel_id, 1).await
    }

    #[instrument(skip(self, chatter_id, channel_id, score))]
    pub async fn increment_score_by(
        &mut self,
        chatter_id: &ChatterId,
        channel_id: &ChannelId,
        score: i64,
    ) -> SqlxResult<ScoreSummary> {
        sqlx::query_as::<_, ScoreSummary>(
            r#"
            INSERT INTO score (
                channel_id,
                chatter_id,
                score,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, NOW(), NOW())
            ON CONFLICT (chatter_id, channel_id)
            DO UPDATE SET
                score = score.score + $3,
                updated_at = NOW()
            RETURNING 
                chatter_id, 
                channel_id,
                score
            "#,
        )
        .bind(channel_id)
        .bind(chatter_id)
        .bind(score)
        .fetch_one(&mut **self.inner_mut()?)
        .await
    }

    #[instrument(skip(self, chatter_id, channel_id, score))]
    /// Alternatively 'set_score' - overwrites the score referenced by the foreign key `(channel_id, chatter_id)`
    pub async fn update_score(
        &mut self,
        chatter_id: &ChatterId,
        channel_id: &ChannelId,
        score: i64,
    ) -> SqlxResult<ScoreSummary> {
        sqlx::query_as::<_, ScoreSummary>(
            r#"
            INSERT INTO score (
                channel_id,
                chatter_id,
                score,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, NOW(), NOW())
            ON CONFLICT (chatter_id, channel_id)
            DO UPDATE SET
                score = $3,
                updated_at = NOW()
            RETURNING 
                chatter_id, 
                channel_id,
                score
            "#,
        )
        .bind(channel_id)
        .bind(chatter_id)
        .bind(score)
        .fetch_one(&mut **self.inner_mut()?)
        .await
    }

    #[instrument(skip(self))]
    pub async fn recalculate_chatter_total(&mut self, chatter_id: &ChatterId) -> SqlxResult<()> {
        let res = sqlx::query(
            r#"
            UPDATE chatter
            SET total = (SELECT COALESCE(SUM(score), 0) FROM score WHERE chatter_id = $1),
                updated_at = NOW()
            WHERE id = $1
            RETURNING total
            "#,
        )
        .bind(chatter_id)
        .fetch_one(&mut **self.inner_mut()?)
        .await?;

        tracing::debug!(result = ?res, "recalculated total for chatter");

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn recalculate_channel_total(&mut self, channel_id: &ChannelId) -> SqlxResult<()> {
        let res = sqlx::query(
            r#"
            UPDATE channel
            SET channel_total = (SELECT COALESCE(SUM(score), 0) FROM score WHERE channel_id = $1),
                updated_at = NOW()
            WHERE id = $1
            RETURNING channel_total
            "#,
        )
        .bind(channel_id)
        .fetch_one(&mut **self.inner_mut()?)
        .await?;

        tracing::debug!(result = ?res, "recalculated total for channel");

        Ok(())
    }
}

pub mod sql_fragment {
    pub const CHATTER_FIELDS: &str = r#"
        id,
        login,
        name, 
        color,
        image,
        total,
        private,
        created_at,
        updated_at
    "#;

    pub const CHANNEL_FIELDS: &str = r#"
        id,
        channel_total, 
        created_at,
        updated_at
    "#;
}

#[async_trait]
pub trait Repository {
    type Ident: for<'q> sqlx::Encode<'q, Postgres> + sqlx::Type<Postgres> + Send + Sync + fmt::Debug;
    type Output: for<'r> sqlx::FromRow<'r, <Postgres as sqlx::Database>::Row>
        + Sized
        + Unpin
        + Send
        + fmt::Debug;

    const BASE_FIELDS: &'static str;
    const TABLE_NAME: &'static str;

    fn new(pool: &'static Pool<Postgres>) -> Self
    where
        Self: Sized;

    fn pool(&self) -> &'static Pool<Postgres>;

    async fn exists(&self, id: &Self::Ident) -> SqlxResult<bool> {
        Ok(
            match sqlx::query_scalar::<_, bool>(&format!(
                "SELECT EXISTS (SELECT 1 FROM {} WHERE id = $1)",
                Self::TABLE_NAME
            ))
            .bind(id)
            .fetch_one(self.pool())
            .await
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(error = ?e, table = ?Self::TABLE_NAME, "failed to check chatter existence");
                    false
                }
            },
        )
    }

    #[instrument(skip(self, id))]
    async fn get_by_id(&self, id: &Self::Ident) -> SqlxResult<Option<Self::Output>> {
        sqlx::query_as::<_, Self::Output>(&format!(
            "SELECT {} FROM {} WHERE id = $1",
            Self::BASE_FIELDS,
            Self::TABLE_NAME
        ))
        .bind(id)
        .fetch_optional(self.pool())
        .await
    }

    #[instrument(skip(self, ids))]
    async fn get_many_by_id(&self, ids: &[Self::Ident]) -> SqlxResult<Vec<Self::Output>> {
        let tx_result = Tx::with_tx(self.pool(), |tx| async move {
            let result = async {
                let mut output = Vec::new();
                for id in ids {
                    match sqlx::query_as::<_, Self::Output>(&format!(
                        "SELECT {} FROM {} WHERE id = $1",
                        Self::BASE_FIELDS,
                        Self::TABLE_NAME
                    ))
                    .bind(id)
                    .fetch_optional(self.pool())
                    .await
                    {
                        Ok(Some(ch)) => output.push(ch),
                        Ok(None) => (),
                        Err(e) => {
                            tracing::error!(error = ?e, "error while retrieving ids from db");
                        }
                    }
                }

                output
            }
            .await;

            (tx, Ok(result))
        })
        .await?;

        Ok(tx_result)
    }

    #[instrument(skip(self, login))]
    async fn get_by_login(&self, login: String) -> SqlxResult<Self::Output> {
        sqlx::query_as::<_, Self::Output>(&format!(
            "SELECT {} FROM {} WHERE login = $1",
            Self::BASE_FIELDS,
            Self::TABLE_NAME
        ))
        .bind(login)
        .fetch_one(self.pool())
        .await
    }

    #[instrument(skip(self, limit, offset))]
    async fn get_by_range(&self, limit: i64, offset: i64) -> SqlxResult<Vec<Self::Output>> {
        sqlx::query_as::<_, Self::Output>(&format!(
            "SELECT {} FROM {} ORDER BY total DESC, created_at ASC LIMIT $1 OFFSET $2",
            Self::BASE_FIELDS,
            Self::TABLE_NAME,
        ))
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool())
        .await
    }

    async fn insert(&self, item: &Self::Output) -> SqlxResult<()>;
    async fn insert_many(&self, items: &[Self::Output]) -> SqlxResult<()>;

    /// Increments a `total` count field for the implementing struct, returning `Ok(new_total)` if
    /// successful.
    async fn increment_score(&self, s: &Self::Output) -> SqlxResult<i64>;
}
