use sqlx::{Pool, Postgres, Result as SqlxResult};
use tracing::instrument;

use super::sql_fragment;
use crate::db::{
    models::channel::{Channel, ChannelId},
    prelude::Tx,
    repositories::Repository,
};

#[derive(Debug)]
pub struct ChannelRepository {
    pool: &'static Pool<Postgres>,
}

#[async_trait::async_trait]
impl Repository for ChannelRepository {
    type Ident = ChannelId;
    type Output = Channel;

    const BASE_FIELDS: &'static str = sql_fragment::CHANNEL_FIELDS;
    const TABLE_NAME: &'static str = "channel";

    #[instrument(skip(pool))]
    fn new(pool: &'static Pool<Postgres>) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    fn pool(&self) -> &'static Pool<Postgres> {
        self.pool
    }

    #[instrument(skip(self, item))]
    async fn insert(&self, item: &Self::Output) -> SqlxResult<()> {
        match sqlx::query!(
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
        .execute(self.pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!(error = ?e, "failure during channel insertion");
                Err(e)
            }
        }
    }

    #[instrument(skip(self, items))]
    async fn insert_many(&self, items: &[Self::Output]) -> SqlxResult<()> {
        Tx::with_tx(self.pool, |mut tx| async move {
            let result = async {
                for item in items {
                    match tx.insert_channel(item).await {
                        Ok(_) => (),
                        Err(e) => {
                            tracing::error!(error = ?e, "insert many failure");
                            return Err(e);
                        }
                    }
                }

                Ok(())
            }.await;

            (tx, result)
        })
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn increment_score(&self, channel: &Self::Output) -> SqlxResult<i64> {
        match sqlx::query_scalar!(
            r#"
            INSERT INTO channel (
                id,
                channel_total,
                created_at, 
                updated_at
            )
            VALUES ($1, 1, NOW(), NOW())
            ON CONFLICT (id)
            DO UPDATE SET
                channel_total = channel.channel_total + 1,
                created_at = EXCLUDED.created_at,
                updated_at = NOW()
            RETURNING channel_total
            "#,
            &channel.id.to_string()
        )
        .fetch_one(self.pool)
        .await
        {
            Ok(total) => Ok(total),
            Err(e) => {
                tracing::error!(error = ?e, "failure during channel total update");
                return Err(e);
            }
        }
    }
}
