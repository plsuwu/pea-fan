use super::DatabaseLayer;
use crate::{database::pg::DbResult, webhook::connection};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub login: String,
    pub color: String,
    pub image: Option<String>,
    pub total: i32,
    pub redact: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub id: String,
    pub total: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Broadcaster {
    pub user: User,
    pub channel: Channel,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Score {
    pub chatter_id: String,
    pub channel_id: String,
    pub score: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ScoreWithRank {
    pub chatter_id: String,
    pub channel_id: String,
    pub score: i32,
    pub rank: i64,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Score {
    pub async fn get_rank_on_channel(chatter_id: &str, channel_id: &str) -> DbResult<()> {
        let connection = DatabaseLayer::get().await?;

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
            channel_id
        )
        .fetch_one(&connection.pool)
        .await?;

        println!("{:#?}", result);

        Ok(())
    }

    pub async fn update(chatter_id: &str, channel_id: &str, score: i32) -> DbResult<()> {
        let connection = DatabaseLayer::get().await?;
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

impl Channel {
    pub async fn get_broadcaster(channel_id: &str) -> DbResult<Broadcaster> {
        let connection = DatabaseLayer::get().await?;

        let mut tx = connection.pool.begin().await?;
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id,
                login,
                color,
                image,
                total,
                redact,
                created_at,
                updated_at
            FROM users
            WHERE id = $1
            "#,
            channel_id,
        )
        .fetch_one(&mut *tx)
        .await?;

        let channel = sqlx::query_as!(
            Channel,
            r#"
            SELECT id, total, created_at, updated_at FROM channels
            WHERE id = $1
            "#,
            channel_id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(Broadcaster { user, channel })
    }

    pub async fn upsert(channel: &Channel) -> DbResult<()> {
        let connection = DatabaseLayer::get().await?;
        let _ = sqlx::query!(
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
}

impl User {
    pub async fn upsert(chatter: &User) -> DbResult<()> {
        let connection = DatabaseLayer::get().await?;
        let _ = sqlx::query!(
            r#"
            INSERT INTO users (id, login, color, image, total, redact)
            VALUES ($1, $2 ,$3, $4, $5, $6)
            ON CONFLICT (id)
            DO UPDATE SET 
                login = $2, 
                color = $3,
                image = $4,
                total = $5, 
                redact = $6,
                updated_at = NOW()
            "#,
            chatter.id,
            chatter.login,
            chatter.color,
            chatter.image,
            chatter.total,
            chatter.redact,
        )
        .execute(&connection.pool)
        .await?;

        Ok(())
    }

    pub async fn get_by_id(id: &str) -> DbResult<Self> {
        let connection = DatabaseLayer::get().await?;
        let query = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id,
                login,
                color,
                image,
                total,
                redact,
                created_at,
                updated_at
            FROM users
            WHERE id = $1
            "#,
            id,
        )
        .fetch_one(&connection.pool)
        .await?;

        Ok(query)
    }
}
