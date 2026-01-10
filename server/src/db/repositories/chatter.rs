use sqlx::{Pool, Postgres, Result as SqlxResult};
use tracing::instrument;

use super::sql_fragment;
use crate::db::{
    models::chatter::{Chatter, ChatterId},
    prelude::Tx,
    repositories::Repository,
};

#[derive(Debug)]
pub struct ChatterRepository {
    pool: &'static Pool<Postgres>,
}

#[async_trait::async_trait]
impl Repository for ChatterRepository {
    type Ident = ChatterId;
    type Output = Chatter;

    const BASE_FIELDS: &'static str = sql_fragment::CHATTER_FIELDS;
    const TABLE_NAME: &'static str = "chatter";

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
            VALUES ($1, $2, $3, $4, $5, 0, false, $6, $7)
            ON CONFLICT (id)
            DO NOTHING
            "#,
            &item.id.to_string(),
            item.login,
            item.name,
            item.color,
            item.image,
            item.created_at,
            item.updated_at
        )
        .execute(self.pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!(error = ?e, "failure during chatter insertion");
                Err(e)
            }
        }
    }

    #[instrument(skip(self, items))]
    async fn insert_many(&self, items: &[Self::Output]) -> SqlxResult<()> {
        Tx::with_tx(self.pool, |mut tx| async move {
            let result = async {
                for item in items {
                    match tx.insert_chatter(item).await {
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
    async fn increment_score(&self, chatter: &Self::Output) -> SqlxResult<i64> {
        match sqlx::query_scalar!(
            r#"
            INSERT INTO chatter (
                id,
                name,
                login,
                color,
                image,
                total,
                private,
                created_at, 
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, 1, false, $6, $7)
            ON CONFLICT (id)
            DO UPDATE SET
                name = EXCLUDED.name,
                login = EXCLUDED.login,
                color = EXCLUDED.color,
                image = EXCLUDED.image,
                total = chatter.total + 1,
                created_at = EXCLUDED.created_at,
                updated_at = NOW()
            RETURNING total
            "#,
            &chatter.id.to_string(),
            chatter.name,
            chatter.login,
            chatter.color,
            chatter.image,
            chatter.created_at,
            chatter.updated_at
        )
        .fetch_one(self.pool)
        .await
        {
            Ok(total) => Ok(total),
            Err(e) => {
                tracing::error!(error = ?e, "failure during chatter total update");
                return Err(e);
            }
        }
    }
}
