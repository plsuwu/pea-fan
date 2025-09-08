use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Local, NaiveDateTime};
use thiserror::Error;
use tracing::debug;

use crate::database::schema::{self, Channel, ChannelBasic, Chatter};
use crate::util::helix::{self, Helix, HelixError};
use crate::util::secrets::ENV_SECRETS;

pub type PostgresResult<T> = core::result::Result<T, PostgresError>;

#[derive(Error, Debug)]
pub enum PostgresError {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("helix error: {0}")]
    HelixError(#[from] HelixError),
}

pub struct PostgresConnection {
    pub pool: sqlx::PgPool,
}

impl PostgresConnection {
    pub async fn get() -> PostgresResult<Self> {
        let db_url = ENV_SECRETS.get().pg_url.clone();
        let pool = sqlx::Pool::connect(&db_url).await?;

        Ok(Self { pool })
    }

    pub async fn transfer_to_redis(channel_id: &str) -> PostgresResult<()> {
        let connection = Self::get().await?;
        let mut tx = connection.pool.begin().await?;

        todo!()
    }
}

impl schema::Chatter {
    pub async fn bulk_upsert(chatters: &Vec<helix::InternalUser>) -> PostgresResult<()> {
        let connection = PostgresConnection::get().await?;
        let mut tx = connection.pool.begin().await?;

        for chatter in chatters {
            sqlx::query_as!(
                schema::Chatter,
                r#"
                INSERT INTO chatters (id, login, name, color, image, total, redact)
                VALUES ($1, $2 ,$3, $4, $5, $6, $7)
                ON CONFLICT (id)
                DO UPDATE SET 
                    login = $2,
                    name = $3, 
                    color = $4,
                    image = $5,
                    total = $6, 
                    redact = $7,
                    updated_at = NOW()
                "#,
                chatter.id,
                chatter.login,
                chatter.name,
                chatter.color,
                chatter.image,
                chatter.total,
                chatter.redact,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn upsert(chatter: &helix::InternalUser) -> PostgresResult<()> {
        let connection = PostgresConnection::get().await?;
        _ = sqlx::query!(
            r#"
            INSERT INTO chatters (id, login, name, color, image, total, redact)
            VALUES ($1, $2 ,$3, $4, $5, $6, $7)
            ON CONFLICT (id)
            DO UPDATE SET 
                login = $2, 
                name = $3,
                color = $4,
                image = $5,
                total = $6, 
                redact = $7,
                updated_at = NOW()
            "#,
            chatter.id,
            chatter.login,
            chatter.name,
            chatter.color,
            chatter.image,
            chatter.total,
            chatter.redact,
        )
        .execute(&connection.pool)
        .await?;

        Ok(())
    }

    pub async fn get_by_id(id: &str) -> PostgresResult<Self> {
        let connection = PostgresConnection::get().await?;
        let query = sqlx::query_as!(
            schema::Chatter,
            r#"
            SELECT 
                id,
                login,
                name,
                color,
                image,
                total,
                redact,
                created_at,
                updated_at
            FROM chatters 
            WHERE id = $1
            "#,
            id,
        )
        .fetch_one(&connection.pool)
        .await?;

        Ok(query)
    }
}

impl schema::Channel {
    pub async fn bulk_upsert(channels: &Vec<schema::Channel>) -> PostgresResult<()> {
        let connection = PostgresConnection::get().await?;
        let mut tx = connection.pool.begin().await?;

        for channel in channels {
            _ = sqlx::query_as!(
                schema::Channel,
                r#"
                INSERT INTO channels (id, total)
                VALUES ($1, $2)
                ON CONFLICT (id)
                DO UPDATE SET
                    total = $2,
                    updated_at = NOW()
                "#,
                channel.id,
                channel.total
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
    pub async fn upsert(channel: &schema::Channel) -> PostgresResult<()> {
        let connection = PostgresConnection::get().await?;
        _ = sqlx::query!(
            r#"
            INSERT INTO channels (id, total)
            VALUES ($1, $2)
            ON CONFLICT (id)
            DO UPDATE SET
                total = $2,
                updated_at = NOW()
            "#,
            channel.id,
            channel.total
        )
        .execute(&connection.pool)
        .await?;

        Ok(())
    }

    pub async fn get_existing_ids_by_login(logins: &Vec<String>) -> PostgresResult<Vec<ChannelBasic>> {
        let connection = PostgresConnection::get().await?;
        let existing = sqlx::query_as!(
            ChannelBasic,
            r#"
            SELECT c.id, u.login FROM channels c
            JOIN chatters u ON c.id = u.id
            ORDER BY u.login ASC
            "#,
        )
        .fetch_all(&connection.pool)
        .await?;

        debug!("existing: {:#?} ({} rows)", existing, existing.len());
        
        Ok(existing)
    }

    pub async fn get_broadcaster(id: &str) -> PostgresResult<schema::Broadcaster> {
        let connection = PostgresConnection::get().await?;
        let mut tx = connection.pool.begin().await?;

        let chatter = sqlx::query_as!(
            schema::Chatter,
            r#"
            SELECT
                id,
                login,
                name,
                color,
                image,
                total,
                redact,
                created_at,
                updated_at
            FROM chatters
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&mut *tx)
        .await?;

        let channel = sqlx::query_as!(
            schema::Channel,
            r#"
            SELECT id, total, created_at, updated_at FROM channels
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(schema::Broadcaster { chatter, channel })
    }
}

impl schema::Score {
    pub async fn get_rank_on_channel(chatter_id: &str, channel_id: &str) -> PostgresResult<()> {
        let connection = PostgresConnection::get().await?;
        let result = sqlx::query!(
            r#"
            WITH channel_leaderboard AS (
                SELECT 
                    *,
                    ROW_NUMBER() OVER (ORDER BY score DESC, created_at ASC)
                AS "channel_rank!"
                FROM scores
                WHERE channel_id = $2
            )
            SELECT * FROM channel_leaderboard
            WHERE chatter_id = $1
            "#,
            chatter_id,
            channel_id,
        )
        .fetch_one(&connection.pool)
        .await?;

        todo!()
    }

    pub async fn bulk_update(scores: HashMap<String, HashMap<String, i32>>) -> PostgresResult<()> {
        let connection = PostgresConnection::get().await?;
        let mut tx = connection.pool.begin().await?;

        for (chatter_id, val) in scores.iter() {
            for (channel_id, score) in val.iter() {
                _ = sqlx::query!(
                    r#"
                    INSERT INTO scores (chatter_id, channel_id, score)
                    VALUES ($1, $2, $3) 
                    ON CONFLICT (chatter_id, channel_id)
                    DO UPDATE SET 
                        score = $3,
                        updated_at = NOW()
                    "#,
                    chatter_id,
                    channel_id,
                    score
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn update(chatter_id: &str, channel_id: &str, score: i32) -> PostgresResult<()> {
        let connection = PostgresConnection::get().await?;
        _ = sqlx::query!(
            r#"
            INSERT INTO scores (chatter_id, channel_id, score)
            VALUES ($1, $2, $3)
            ON CONFLICT (chatter_id, channel_id)
            DO UPDATE SET
                score = $3,
                updated_at = NOW()
            "#,
            chatter_id,
            channel_id,
            score,
        )
        .execute(&connection.pool)
        .await?;

        Ok(())
    }
}
