use sqlx::{Pool, Postgres, Result as SqlxResult};
use tracing::instrument;

use super::sql_fragment;
use crate::db::PgError;
use crate::db::models::channel::{Channel, ChannelId, ChannelReplies};
use crate::db::prelude::Tx;
use crate::db::repositories::Repository;

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
            }
            .await;

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

impl ChannelRepository {
    #[instrument(skip(self))]
    pub async fn get_all_channel_ids(&self) -> SqlxResult<Vec<String>> {
        match sqlx::query_scalar!(
            r#"
            SELECT id FROM channel
            "#,
        )
        .fetch_all(self.pool)
        .await
        {
            Ok(channel_ids) => Ok(channel_ids),
            Err(e) => {
                tracing::error!(error = ?e, "failed to retrieve channels");
                Err(e)
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn new_channel_config(&self, channel: &ChannelId) -> SqlxResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO reply (id)
            SELECT channel.id FROM channel
            WHERE channel.id = $1
            "#,
            &channel.0,
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn update_channel_config(&self, channel: &ChannelId) -> SqlxResult<()> {
        match sqlx::query!(
            r#"
            UPDATE reply SET
                enabled = NOT enabled,
                updated_at = NOW()
            WHERE id = $1
            "#,
            channel.0,
        )
        .execute(self.pool)
        .await
        {
            Ok(_) => {
                tracing::info!(channel = channel.0, "update ok");
                Ok(())
            }
            Err(e) => {
                tracing::error!(error = ?e, channel = channel.0, "failed to update");
                Err(e)
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn get_reply_config(&self, channel: &str) -> SqlxResult<ChannelReplies> {
        let result = sqlx::query_as::<_, ChannelReplies>(
            r#"
            SELECT * FROM reply_configuration
            WHERE id = $1
            "#,
        )
        .bind(channel)
        .fetch_one(self.pool)
        .await?;

        Ok(result)
    }

    #[instrument(skip(self))]
    pub async fn get_all_reply_configs(&self) -> SqlxResult<Vec<ChannelReplies>> {
        // TODO rename this view if ever bothered
        let configs = sqlx::query_as::<_, ChannelReplies>(
            r#"
            SELECT * FROM reply_configuration
            "#,
        )
        .fetch_all(self.pool)
        .await?;

        Ok(configs)
    }

    // pub async fn get_all_reply_configs()
}
