// use super::tests::PgTestFunctions;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Pool, Postgres, Transaction};
use thiserror::Error;

const SQLX_FK_VIOLATION: &str = "23503";

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

impl DatabaseLayer {
    pub(crate) async fn _upsert_channel_fallible<'a>(
        &self,
        tx: &mut Transaction<'static, Postgres>,
        broadcaster: &'a User,
    ) -> Result<Channel, sqlx::Error> {
        let query = sqlx::query_as!(
            Channel,
            r#"
            WITH update AS (
                INSERT INTO channels (id, broadcaster, total)
                VALUES ($1, $2, $3)
                ON CONFLICT (id)
                DO UPDATE SET
                    broadcaster = $2,
                    total = $3,
                    updated_at = NOW()
                returning id, broadcaster, total
            )
            SELECT
                u.id as id,
                u.broadcaster as broadcaster,
                users.color as color,
                users.image as image,
                u.total as total
            FROM update u
            JOIN users ON users.id = u.id
            "#,
            broadcaster.id,
            broadcaster.login,
            broadcaster.total,
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(query)
    }
}

#[async_trait]
pub trait Database {
    async fn new() -> DbResult<DatabaseLayer>;

    async fn upsert_user<'a>(
        &self,
        tx: &mut Transaction<'static, Postgres>,
        user: &'a User,
    ) -> DbResult<User>;
    async fn upsert_channel<'a>(
        &self,
        tx: &mut Transaction<'static, Postgres>,
        broadcaster: &'a User,
    ) -> DbResult<Channel>;
    async fn update_channel_score<'a>(
        &self,
        chatter: &'a User,
        broadcaster: &'a User,
        score: i32,
    ) -> DbResult<()>;

    async fn get_user(&self, user_id: &str) -> DbResult<User>;
    async fn get_channel(&self, channel_id: &str) -> DbResult<Channel>;
    async fn get_score(&self, user_id: &str, channel_id: &str) -> DbResult<Score>;

    async fn get_channel_leaderboard_global(&self, limit: i64) -> DbResult<Vec<ChannelEntry>>;
    async fn get_channel_leaderboard_internal(
        &self,
        channel_id: &str,
        limit: i64,
    ) -> DbResult<Vec<UserChannelEntry>>;

    async fn get_user_leaderboard_global(&self, limit: i64) -> DbResult<Vec<UserEntry>>;
    async fn get_user_leaderboard_internal(
        &self,
        user_id: &str,
        limit: i64,
    ) -> DbResult<Vec<UserChannelEntry>>;

    async fn from_cache(&self, migrations: Vec<(User, User, i32)>) -> DbResult<()>;
    async fn to_cache(&self, broadcaster: &str) -> DbResult<()>;
}

#[async_trait]
impl Database for DatabaseLayer {
    async fn new() -> DbResult<Self> {
        let db_url = dotenvy::var("DATABASE_URL")?;
        let pool = Pool::connect(&db_url).await?;

        Ok(DatabaseLayer { pool })
    }

    async fn upsert_user<'a>(
        &self,
        tx: &mut Transaction<'static, Postgres>,
        user: &'a User,
    ) -> DbResult<User> {
        let query = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, login, color, image, redact, total)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id)
            DO UPDATE SET 
                login = $2,
                color = $3, 
                image = $4,
                redact = $5,
                total = $6,
                updated_at = NOW()
            RETURNING id, login, color, image, redact, total
            "#,
            user.id,
            user.login,
            user.color,
            user.image,
            user.redact,
            user.total
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(query)
    }

    async fn upsert_channel<'a>(
        &self,
        tx: &mut Transaction<'static, Postgres>,
        broadcaster: &'a User,
    ) -> DbResult<Channel> {
        loop {
            match self._upsert_channel_fallible(tx, &broadcaster).await {
                Ok(result) => return Ok(result),
                Err(sqlx::Error::Database(query_err))
                    if query_err.code() == Some(SQLX_FK_VIOLATION.into()) =>
                {
                    println!("[PG_FK_VIOLATION] => {:#?}", query_err);
                    self.upsert_user(tx, &broadcaster).await?;
                }

                Err(e) => {
                    println!("[PG_FALLIBLE_UNHANDLED] => {:#?}", e);
                    return Err(DatabaseError::SqlxError(e));
                }
            }
        }
    }

    async fn update_channel_score<'a>(
        &self,
        user: &'a User,
        broadcaster: &'a User,
        score: i32,
    ) -> DbResult<()> {
        let mut tx = self.pool.begin().await?;

        let chatter = self.upsert_user(&mut tx, &user).await?;
        let channel = self.upsert_channel(&mut tx, &broadcaster).await?;

        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            r#"
            INSERT INTO scores (chatter_id, broadcaster_id, score)
            VALUES ($1, $2, $3)
            ON CONFLICT (chatter_id, broadcaster_id)
            DO UPDATE SET 
                score = $3,
                updated_at = NOW()
            "#,
            chatter.id,
            channel.id,
            score,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn get_score(&self, user_id: &str, channel_id: &str) -> DbResult<Score> {
        let mut tx = self.pool.begin().await?;
        let query = sqlx::query_as!(
            Score,
            r#"
            SELECT 
               chatter_id,
               broadcaster_id,
               u.login as chatter,
               c.broadcaster,
               score
            FROM scores
            JOIN users u ON chatter_id = $1
            JOIN channels c ON broadcaster_id = $2
            WHERE chatter_id = u.id
            AND broadcaster_id = c.id
            "#,
            user_id,
            channel_id,
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(query)
    }

    async fn get_user(&self, user_id: &str) -> DbResult<User> {
        let query = sqlx::query_as!(
            User,
            r#"
            SELECT id, login, color, image, redact, total FROM users WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(query)
    }

    async fn get_channel(&self, channel_id: &str) -> DbResult<Channel> {
        let mut tx = self.pool.begin().await?;
        let query = sqlx::query_as!(
            Channel,
            r#"
            SELECT 
                ch.id, 
                ch.broadcaster, 
                u.color,
                u.image,
                ch.total 
            FROM channels ch 
            JOIN users u ON u.id = ch.id
            WHERE ch.id = $1
            "#,
            channel_id,
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(query)
    }

    async fn get_channel_leaderboard_global(&self, limit: i64) -> DbResult<Vec<ChannelEntry>> {
        let mut tx = self.pool.begin().await?;
        let entries = sqlx::query_as!(
            ChannelEntry,
            r#"
            SELECT 
                c.id as broadcaster_id,
                c.broadcaster,
                u.color,
                u.image,
                c.total,
                ROW_NUMBER() OVER (ORDER BY c.total DESC) as "rank!" 
            FROM channels c
            JOIN users u ON c.id = u.id
            WHERE c.total > 0
            ORDER BY c.total DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(entries)
    }

    async fn get_channel_leaderboard_internal(
        &self,
        channel_id: &str,
        limit: i64,
    ) -> DbResult<Vec<UserChannelEntry>> {
        let mut tx = self.pool.begin().await?;
        let entries = sqlx::query_as!(
            UserChannelEntry,
            r#"
            WITH ranked_scores AS (
                SELECT 
                    s.chatter_id,
                    s.broadcaster_id,
                    u.login,
                    u.color,
                    u.image,
                    s.score as total,
                    ROW_NUMBER() OVER (ORDER BY s.score DESC) as "rank!"
                FROM scores s
                JOIN users u ON s.chatter_id = u.id
                JOIN channels c ON s.broadcaster_id = c.id
                WHERE c.broadcaster = $1
            )
            SELECT * FROM ranked_scores
            LIMIT $2
            "#,
            channel_id,
            limit
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(entries)
    }

    async fn get_user_leaderboard_global(&self, limit: i64) -> DbResult<Vec<UserEntry>> {
        let mut tx = self.pool.begin().await?;
        let entries = sqlx::query_as!(
            UserEntry,
            r#"
            SELECT
                id, 
                login, 
                color, 
                image,
                total,
                ROW_NUMBER() OVER (ORDER BY total DESC) as "rank!"
            FROM users 
            WHERE total > 0 
            ORDER BY total DESC
            LIMIT $1 
            "#,
            limit
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(entries)
    }

    async fn get_user_leaderboard_internal(
        &self,
        user_id: &str,
        limit: i64,
    ) -> DbResult<Vec<UserChannelEntry>> {
        let mut tx = self.pool.begin().await?;
        let entries = sqlx::query_as!(
            UserChannelEntry,
            r#"
            WITH ranked_scores AS (
                SELECT 
                    s.chatter_id,
                    s.broadcaster_id,
                    u.login,
                    u.color,
                    u.image,
                    s.score as total,
                    ROW_NUMBER() OVER (ORDER BY s.score DESC) as "rank!"
                FROM scores s
                JOIN users u ON s.broadcaster_id = u.id
                JOIN users c ON s.chatter_id = c.id
                WHERE u.id = $1
            )
            SELECT * FROM ranked_scores
            LIMIT $2
            "#,
            user_id,
            limit,
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(entries)
    }

    async fn from_cache(&self, migrations: Vec<(User, User, i32)>) -> DbResult<()> {
        for (user, broadcaster, score) in migrations.iter() {
            let mut tx = self.pool.begin().await?;
            self.upsert_user(&mut tx, &user).await?;
            self.upsert_channel(&mut tx, &broadcaster).await?;
            self.update_channel_score(&user, &broadcaster, *score)
                .await?;
        }

        Ok(())
    }

    async fn to_cache(&self, channel: &str) -> DbResult<()> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct User {
    pub id: String,
    pub login: String,
    pub color: String,
    pub image: Option<String>,
    pub redact: bool,
    pub total: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Channel {
    pub id: String,
    pub broadcaster: String,
    pub color: String,
    pub image: Option<String>,
    pub total: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Score {
    pub broadcaster_id: String,
    pub chatter_id: String,
    pub broadcaster: String,
    pub chatter: String,
    pub score: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelEntry {
    pub broadcaster_id: String,
    pub broadcaster: String,
    pub color: String,
    pub image: Option<String>,
    pub total: i32,
    pub rank: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserEntry {
    pub id: String,
    pub login: String,
    pub color: String,
    pub image: Option<String>,
    pub total: i32,
    pub rank: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserChannelEntry {
    pub broadcaster_id: String,
    pub chatter_id: String,
    pub login: String,
    pub color: String,
    pub image: Option<String>,
    pub total: i32,
    pub rank: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::tests::PgTestFunctions;

    #[async_trait]
    trait TestConnection {
        async fn new_test() -> DbResult<DatabaseLayer>;
    }

    #[async_trait]
    impl TestConnection for DatabaseLayer {
        async fn new_test() -> DbResult<DatabaseLayer> {
            todo!()
        }
    }

    #[sqlx::test]
    async fn init_test() {
        clear_test_data().await;
    }

    #[sqlx::test]
    async fn test_upsert_user() {
        let conn = DatabaseLayer::new().await.unwrap();
        let test_user = DatabaseLayer::new_user();

        let mut tx = conn.pool.begin().await.unwrap();
        let result = conn.upsert_user(&mut tx, &test_user).await.unwrap();

        tx.commit().await.unwrap();
        assert_eq!(test_user, result);
    }

    #[sqlx::test]
    async fn test_upsert_multiple_users() {
        // clear_test_data().await;
        let conn = DatabaseLayer::new().await.unwrap();
        let mut output = Vec::new();
        let test_users = DatabaseLayer::new_users_vec();

        let mut tx = conn.pool.begin().await.unwrap();

        for user in &test_users {
            let res = conn.upsert_user(&mut tx, &user).await.unwrap();
            println!("[TX::UPSERT_MULTIUSER] => {:#?}", res);
            output.push(res);
        }

        tx.commit().await.unwrap();

        assert_eq!(test_users, output);
    }

    #[sqlx::test]
    async fn test_upsert_channel() {
        let conn = DatabaseLayer::new().await.unwrap();
        let test_user = DatabaseLayer::new_user();
        let test_channel = DatabaseLayer::new_channel();

        let mut tx = conn.pool.begin().await.unwrap();

        let result = conn.upsert_channel(&mut tx, &test_user).await.unwrap();

        println!("[TX::UPSERT_CHANNEL] => {:#?}", result);

        tx.commit().await.unwrap();

        assert_eq!(test_channel, result);

        // clear_test_data().await;
    }

    #[sqlx::test]
    async fn test_update_channel_score() {
        // clear_test_data().await;

        let conn = DatabaseLayer::new().await.unwrap();
        // let conn = SqlitePoolOptions::new()
        //     .max_connections(1)
        //     .connect("sqlite::memory:")
        //     .await
        //     .unwrap();

        let test_channel = DatabaseLayer::new_channel();

        let test_users = DatabaseLayer::new_users_vec();

        let mut retrieved_scores = Vec::new();
        let expected_scores = vec![
            Score {
                broadcaster_id: "999999999".to_string(),
                chatter_id: "999999998".to_string(),
                broadcaster: "fake_user".to_string(),
                chatter: "test_user".to_string(),
                score: 2,
            },
            Score {
                broadcaster_id: "999999999".to_string(),
                chatter_id: "999999997".to_string(),
                broadcaster: "fake_user".to_string(),
                chatter: "test_user_2".to_string(),
                score: 3,
            },
            Score {
                broadcaster_id: "999999999".to_string(),
                chatter_id: "999999996".to_string(),
                broadcaster: "fake_user".to_string(),
                chatter: "test_user_3".to_string(),
                score: 4,
            },
            Score {
                broadcaster_id: "999999999".to_string(),
                chatter_id: "999999995".to_string(),
                broadcaster: "fake_user".to_string(),
                chatter: "test_user_4".to_string(),
                score: 5,
            },
        ];

        for user in &test_users {
            conn.update_channel_score(&user, &test_channel_user, user.total)
                .await
                .unwrap();
        }

        for user in &test_users {
            let score = conn
                .get_score(&user.id, &test_channel_user.id)
                .await
                .unwrap();

            retrieved_scores.push(score);
        }

        assert_eq!(expected_scores, retrieved_scores);
        // clear_test_data().await;
    }

    async fn clear_test_data() {
        let conn = DatabaseLayer::new().await.unwrap();
        let users_vec = DatabaseLayer::new_users_vec();

        for user in users_vec {
            conn.del_test_data(&user).await.unwrap();
        }

        conn.del_test_data(&DatabaseLayer::new_user())
            .await
            .unwrap();
    }
}

// pub async fn handle_update_score(
//     db: &DatabaseLayer,
//     user_login: String,
//     channel: String,
//     score: i64,
// ) -> DbResult<()> {
//     db.update_score(&user_login, &channel, score).await
// }
//
// pub async fn handle_get_channels_global_leaderboard(
//     db: &DatabaseLayer,
//     limit: Option<i64>,
// ) -> DbResult<Vec<ChannelEntry>> {
//     db.get_channels_global_leaderboard(limit.unwrap_or(10))
//         .await
// }
//
// pub async fn handle_get_channel_internal_leaderboard(
//     db: &DatabaseLayer,
//     channel: String,
//     limit: Option<i64>,
// ) -> DbResult<Vec<UserChannelEntry>> {
//     db.get_channel_internal_leaderboard(&channel, limit.unwrap_or(10))
//         .await
// }
