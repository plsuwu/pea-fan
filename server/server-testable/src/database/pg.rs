use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use thiserror::Error;

pub type DbResult<T> = core::result::Result<T, DatabaseError>;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("failed to perform upsert on table '{}'", table)]
    Upsert { table: String },

    #[error("failed to perform `user_channel_scores' update: {}", reason)]
    Update { reason: String },

    #[error("failed to get data from db: '{}'", reason)]
    Get { reason: String },

    #[error("failed to perform batch migration: '{}'", reason)]
    Migrate { reason: String },
}

#[async_trait]
pub trait Database {
    fn new(pool: PgPool) -> Self;

    async fn upsert_user(&self, user_login: &str) -> DbResult<User>;
    async fn upsert_channel(&self, broadcaster_login: &str) -> DbResult<Channel>;
    async fn update_score(
        &self,
        user_login: &str,
        broadcaster_login: &str,
        score: i64,
    ) -> DbResult<()>;

    async fn get_channel_total(&self, broadcaster_login: &str) -> DbResult<Option<i64>>;
    async fn get_user_total(&self, user_login: &str) -> DbResult<Option<i64>>;

    async fn get_channels_global_leaderboard(&self, limit: i64) -> DbResult<Vec<ChannelEntry>>;
    async fn get_channel_internal_leaderboard(
        &self,
        broadcaster_login: &str,
        limit: i64,
    ) -> DbResult<Vec<UserChannelEntry>>;
    async fn get_user_channels_leaderboard(
        &self,
        user_login: &str,
    ) -> DbResult<Vec<UserChannelEntry>>;

    async fn get_user_rank_on_channel(
        &self,
        user_login: &str,
        broadcaster: &str,
    ) -> DbResult<Option<i64>>;

    async fn migrate_redis_batched(&self, migrations: Vec<(String, String, i64)>) -> DbResult<()>;
}

#[async_trait]
impl Database for DatabaseLayer {
    fn new(pool: PgPool) -> Self {
        dotenvy::dotenv().ok();
        Self { pool }
    }

    async fn upsert_user(&self, user_login: &str) -> DbResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (user_login) 
            VALUES ($1) 
            ON CONFLICT (user_login)
            DO UPDATE SET updated_at = NOW()
            RETURNING id, user_id, user_login, color, total, redact
            "#,
            user_login
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn upsert_channel(&self, broadcaster_login: &str) -> DbResult<Channel> {
        let channel = sqlx::query_as!(
            Channel,
            r#"
            INSERT INTO channels (broadcaster_login)
            VALUES ($1)
            ON CONFLICT (broadcaster_login)
            DO UPDATE SET updated_at = NOW()
            RETURNING broadcaster_id, broadcaster_login, total, profile_img_url
            "#,
            broadcaster_login,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(channel)
    }

    async fn update_score(
        &self,
        user_login: &str,
        broadcaster_login: &str,
        score: i64,
    ) -> DbResult<()> {
        let mut tx = self.pool.begin().await?;

        let user = self.upsert_user(user_login).await?;
        let channel = self.upsert_channel(broadcaster_login).await?;

        sqlx::query!(
            r#"
            INSERT INTO user_channel_scores (user_login, broadcaster_login, score)
            VALUES ($1, $2, $3) 
            ON CONFLICT (user_login, broadcaster_login)
            DO UPDATE SET score = $3, updated_at = NOW()
            "#,
            user.user_login,
            channel.broadcaster_login,
            score
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Retrieve a user's current total score
    ///
    /// Functionally equivalent to `GET users:[USER_LOGIN]:total`
    async fn get_user_total(&self, user_login: &str) -> DbResult<Option<i64>> {
        let result = sqlx::query!(
            r#"
            SELECT total FROM users WHERE user_login = $1
            "#,
            user_login,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| row.total))
    }

    /// Retrieve a channel's current total score
    ///
    /// Functionally equivalent to `GET channels:[BROADCASTER_LOGIN]:total`
    async fn get_channel_total(&self, broadcaster_login: &str) -> DbResult<Option<i64>> {
        let result = sqlx::query!(
            r#"
            SELECT total FROM channels WHERE broadcaster_login = $1
            "#,
            broadcaster_login,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| row.total))
    }

    /// Retrieve the channels leaderboard
    ///
    /// Functionally equivalent to calling `GET channel:[CHANNEL_LOGIN]:total` for all channels and
    /// sorting them by score in descending order
    async fn get_channels_global_leaderboard(&self, limit: i64) -> DbResult<Vec<ChannelEntry>> {
        // the `rank` output from the below query should never be NULL, but sqlx doesn't appear to
        // recognise this and instead expects to unpack `query_as!().rank` into an `Option<T>`
        // field without the `!` non-null assertion
        let entries = sqlx::query_as!(
            ChannelEntry,
            r#"
            SELECT 
                c.broadcaster_login,
                c.profile_img_url,
                u.color as broadcaster_color,
                c.total,
                ROW_NUMBER() OVER (ORDER BY c.total DESC) as "rank!"
            FROM channels c
            JOIN users u ON c.broadcaster_login = u.user_login
            WHERE c.total > 0
            ORDER BY c.total DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    async fn get_user_channels_leaderboard(
        &self,
        user_login: &str,
    ) -> DbResult<Vec<UserChannelEntry>> {
        let entries = sqlx::query_as!(
            UserChannelEntry,
            r#"
            SELECT
                u.user_login,
                c.broadcaster_login,
                ucs.score as total,
                ROW_NUMBER() OVER (ORDER BY ucs.score DESC) as "rank!"
            FROM user_channel_scores ucs
            JOIN users u ON ucs.user_login = u.user_login
            JOIN channels c ON ucs.broadcaster_login = c.broadcaster_login
            WHERE u.user_login = $1
            ORDER BY ucs.score DESC
            "#,
            user_login
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    async fn get_user_rank_on_channel(
        &self,
        user_login: &str,
        broadcaster_login: &str,
    ) -> DbResult<Option<i64>> {
        let result = sqlx::query!(
            r#"
            WITH ranked_scores AS (
                SELECT 
                    u.user_login,
                    ROW_NUMBER() OVER (ORDER BY ucs.score DESC) as rank
                FROM user_channel_scores ucs
                JOIN users u ON ucs.user_login = u.user_login
                JOIN channels c ON ucs.broadcaster_login = c.broadcaster_login
                WHERE c.broadcaster_login = $2
            )
            SELECT rank FROM ranked_scores WHERE user_login = $1
            "#,
            user_login,
            broadcaster_login,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| row.rank.unwrap_or(0)))
    }

    async fn get_channel_internal_leaderboard(
        &self,
        broadcaster_login: &str,
        limit: i64,
    ) -> DbResult<Vec<UserChannelEntry>> {
        let entries = sqlx::query_as!(
            UserChannelEntry,
            r#"
            SELECT  
                u.user_login,
                c.broadcaster_login,
                ucs.score as total,
                ROW_NUMBER() OVER (ORDER BY ucs.score DESC) as "rank!"
            FROM user_channel_scores ucs
            JOIN users u ON ucs.user_login = u.user_login
            JOIN channels c ON ucs.broadcaster_login = c.broadcaster_login
            WHERE c.broadcaster_login = $1
            ORDER BY ucs.score DESC
            LIMIT $2
            "#,
            broadcaster_login,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    async fn migrate_redis_batched(&self, migrations: Vec<(String, String, i64)>) -> DbResult<()> {
        let mut tx = self.pool.begin().await?;

        for (user, channel, score) in migrations {
            sqlx::query!(
                r#"
                WITH user_upsert AS (
                    INSERT INTO users (user_login)
                    VALUES ($1)
                    ON CONFLICT (user_login) DO UPDATE SET updated_at = NOW()
                    RETURNING user_login
                ),
                channel_upsert AS (
                    INSERT INTO channels (broadcaster_login)
                    VALUES ($2)
                    ON CONFLICT (broadcaster_login) DO UPDATE SET updated_at = NOW()
                    RETURNING broadcaster_login
                )
                INSERT INTO user_channel_scores (user_login, broadcaster_login, score)
                SELECT user_upsert.user_login, channel_upsert.broadcaster_login, $3
                FROM user_upsert, channel_upsert
                ON CONFLICT (user_login, broadcaster_login)
                DO UPDATE SET score = $3, updated_at = NOW()
                "#,
                user,
                channel,
                score
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub user_login: String,
    pub user_id: String,
    pub color: String,
    pub total: i64,
    pub redact: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    pub broadcaster_id: String,
    pub broadcaster_login: String,
    pub profile_img_url: String,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserChannelScore {
    pub user_login: String,
    pub channel_login: String,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelEntry {
    pub broadcaster_login: String,
    pub broadcaster_color: String,
    pub profile_img_url: String,
    pub total: i64,
    pub rank: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserChannelEntry {
    pub user_login: String,
    pub broadcaster_login: String,
    pub total: i64,
    pub rank: i64,
}

pub struct DatabaseLayer {
    pool: PgPool,
}

pub async fn handle_update_score(
    db: &DatabaseLayer,
    user_login: String,
    channel: String,
    score: i64,
) -> DbResult<()> {
    db.update_score(&user_login, &channel, score).await
}

pub async fn handle_get_channels_global_leaderboard(
    db: &DatabaseLayer,
    limit: Option<i64>,
) -> DbResult<Vec<ChannelEntry>> {
    db.get_channels_global_leaderboard(limit.unwrap_or(10))
        .await
}

pub async fn handle_get_channel_internal_leaderboard(
    db: &DatabaseLayer,
    channel: String,
    limit: Option<i64>,
) -> DbResult<Vec<UserChannelEntry>> {
    db.get_channel_internal_leaderboard(&channel, limit.unwrap_or(10))
        .await
}
