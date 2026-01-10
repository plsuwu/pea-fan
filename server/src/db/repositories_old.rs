use async_trait::async_trait;
use serde::Deserialize;
use sqlx::{Pool, Postgres, Result as SqlxResult, Transaction};
use tracing::instrument;

pub use crate::db::Repository;
use crate::db::models::*;

pub struct Tx<'a> {
    inner: Transaction<'a, Postgres>,
}

impl<'a> Tx<'a> {
    #[instrument(skip(pool))]
    pub async fn begin(pool: &'a Pool<Postgres>) -> SqlxResult<Self> {
        Ok(Self {
            inner: pool.begin().await?,
        })
    }

    #[instrument(skip(self))]
    pub async fn commit(self) -> SqlxResult<()> {
        self.inner.commit().await
    }

    #[instrument(skip(self))]
    pub async fn rollback(self) -> SqlxResult<()> {
        self.inner.rollback().await
    }

    #[instrument(skip(self, chatter_id, channel_id))]
    pub async fn increment_score(
        &mut self,
        chatter_id: &ChatterId,
        channel_id: &ChannelId,
    ) -> SqlxResult<()> {
        sqlx::query(
            r#"
            INSERT INTO score (
                channel_id,
                chatter_id,
                score,
                created_at,
                updated_at,
            )
            VALUES ($1, $2, 1, NOW(), NOW())
            ON CONFLICT (chatter_id, channel_id)
            DO UPDATE SET
                score = score.score + 1,
                updated_at = NOW()
            "#,
        )
        .bind(channel_id)
        .bind(chatter_id)
        .execute(&mut *self.inner)
        .await?;

        Ok(())
    }

    #[instrument(skip(self, chatter_id, channel_id, score))]
    pub async fn update_score(
        &mut self,
        chatter_id: &ChatterId,
        channel_id: &ChannelId,
        score: i64,
    ) -> SqlxResult<Score> {
        sqlx::query_as::<_, Score>(
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
                score,
                created_at,
                updated_at
            "#,
        )
        .bind(channel_id)
        .bind(chatter_id)
        .bind(score as i32)
        .fetch_one(&mut *self.inner)
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
        .fetch_one(&mut *self.inner)
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
        .fetch_one(&mut *self.inner)
        .await?;

        tracing::debug!(result = ?res, "recalculated total for channel");

        Ok(())
    }

    #[instrument(skip(self, name, color, total))]
    pub async fn insert_chatter(
        &mut self,
        id: &ChatterId,
        name: &str,
        login: &str,
        color: &str,
        image: &str,
        total: i64,
    ) -> SqlxResult<()> {
        sqlx::query(
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
            VALUES ($1, $2, $3, $4, $5, $6, false, NOW(), NOW())
            ON CONFLICT (id)
            DO UPDATE SET 
                login = $2,
                name = $3, 
                color = $4,
                image = $5,
                updated_at = NOW()
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(login)
        .bind(color)
        .bind(image)
        .bind(total)
        .execute(&mut *self.inner)
        .await?;

        Ok(())
    }

    pub async fn insert_channel(&mut self, id: &ChannelId, total: i64) -> SqlxResult<()> {
        sqlx::query(
            r#"
            INSERT INTO channel (
                id,
                channel_total,
                created_at,
                updated_at
            )
            VALUES ($1, $2, NOW(), NOW())
            ON CONFLICT (id)
            DO NOTHING
            "#,
        )
        .bind(id)
        .bind(total)
        .execute(&mut *self.inner)
        .await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct ChatterRepository {
    pool: &'static Pool<Postgres>,
}

#[async_trait]
impl Repository<Postgres> for ChatterRepository {
    type Ident = ChatterId;
    type Output = Chatter;

    const BASE_QUERY: &'static str = sql::SELECT_CHATTER_BASE;
    fn new(pool: &'static Pool<Postgres>) -> Self {
        Self { pool }
    }

    fn pool(&self) -> &'static Pool<Postgres> {
        self.pool
    }
}

impl ChatterRepository {
    #[instrument(skip(self, id))]
    pub async fn exists(&self, id: &str) -> SqlxResult<bool> {
        Ok(sqlx::query(
            r#"
            SELECT EXISTS (
                SELECT id FROM chatter WHERE id = $1
            )
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?
        .is_some())
    }

    #[instrument(skip(self, ids))]
    pub async fn get_many_by_id(&self, ids: &Vec<String>) -> SqlxResult<Vec<Chatter>> {
        let mut tx = Tx::begin(self.pool()).await?;
        let mut retrieved = Vec::new();

        for id in ids {
            match sqlx::query_as::<_, Chatter>(
                r#"
                SELECT * FROM chatter WHERE id = $1
                "#,
            )
            .bind(id.clone())
            .fetch_optional(&mut *tx.inner)
            .await?
            {
                Some(val) => retrieved.push(val),
                None => continue,
            }
        }

        Ok(retrieved)
    }

    #[instrument(skip(self, chatters), fields(chatter_count = chatters.len()))]
    pub async fn insert_many(&self, chatters: &Vec<Chatter>) -> SqlxResult<()> {
        let mut tx = Tx::begin(self.pool).await?;
        for chatter in chatters {
            tx.insert_chatter(
                &chatter.id,
                &chatter.name,
                &chatter.login,
                &chatter.color,
                &chatter.image,
                chatter.total,
            )
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    #[instrument(skip(self, chatter), fields(chatter = chatter.id.0))]
    pub async fn insert(&self, chatter: Chatter) -> SqlxResult<ChatterResponse> {
        let mut tx = Tx::begin(self.pool).await?;

        match tx
            .insert_chatter(
                &chatter.id,
                &chatter.name,
                &chatter.login,
                &chatter.color,
                &chatter.image,
                chatter.total,
            )
            .await
        {
            Ok(_) => {
                tx.commit().await?;
                Ok(self.get_with_score(&chatter.id).await?)
            }
            Err(e) => {
                tracing::error!(error = ?e, "error during chatter upsertion");
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn get_with_score(&self, id: &ChatterId) -> SqlxResult<ChatterResponse> {
        let chatter = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;
        let scores = self.get_scores_for(id).await?;

        Ok(ChatterResponse {
            id: chatter.id.0,
            name: chatter.name,
            login: chatter.login,
            color: chatter.color,
            image: chatter.image,
            total: chatter.total,
            scores,
        })
    }

    #[instrument(skip(self))]
    pub async fn get_scores_for(&self, chatter_id: &ChatterId) -> SqlxResult<Vec<ScoreResponse>> {
        #[derive(sqlx::FromRow)]
        struct ScoreWithChannel {
            channel_id: ChannelId,
            chatter_id: ChatterId,
            score: i64,
            ranking: i64,
            channel_name: String,
            channel_login: String,
            channel_color: String,
            channel_image: String,
            channel_total_chatter: i64,
            channel_total: i64,
        }

        let rows = sqlx::query_as::<_, ScoreWithChannel>(&format!(
            r#"
                SELECT
                    rs.channel_id,
                    rs.chatter_id,
                    rs.ranking,
                    ch_chatter.name as channel_name,
                    ch_chatter.login as channel_login,
                    ch_chatter.color as channel_color,
                    ch_chatter.image as channel_image,
                    ch_chatter.total as channel_total_chatter,
                    ch.channel_total as channel_total
                FROM ranked_scores_view rs
                JOIN channel ch ON rs.channel_id = ch.id
                JOIN chatter ch_chatter ON ch.id = ch_chatter.id
                WHERE rs.chatter_id = $1
                ORDER BY rs.rank ASC
                "#
        ))
        .bind(chatter_id)
        .fetch_all(self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| ScoreResponse {
                channel: ChannelSummary {
                    id: row.channel_id.0.clone(),
                    name: row.channel_name,
                    login: row.channel_login,
                    color: row.channel_color,
                    image: row.channel_image,
                    total_chatter: row.channel_total_chatter,
                    total_channel: row.channel_total,
                },
                chatter: ChatterSummary {
                    id: row.chatter_id.0,
                    ..Default::default()
                },
                score: row.score,
                ranking: row.ranking,
            })
            .collect())
    }

    #[instrument(skip(self))]
    pub async fn get_leaderboard(&self, limit: i64, offset: i64) -> SqlxResult<Vec<ScoreResponse>> {
        #[derive(sqlx::FromRow)]
        struct LeaderboardRow {
            chatter_id: ChatterId,
            chatter_name: String,
            chatter_login: String,
            chatter_color: String,
            chatter_image: String,
            chatter_total: i64,
            score: i64,
            ranking: i64,
        }

        let rows = sqlx::query_as::<_, LeaderboardRow>(&format!(
            r#"
            SELECT 
                rs.chatter_id,
                c.name as chatter_name,
                c.login as chatter_login,
                c.color as chatter_color,
                c.image as chatter_image,
                c.total as chatter_total,
                rs.score, 
                rs.ranking
            FROM ranked_scores_view rs
            JOIN chatter c ON rs.chatter_id = c.id
            ORDER BY rs.rank ASC
            LIMIT $1 OFFSET $2
            "#
        ))
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| ScoreResponse {
                channel: channel_summary.clone(),
                chatter: ChatterSummary {
                    id: row.chatter_id.0,
                    name: row.chatter_name,
                    login: row.chatter_login,
                    color: row.chatter_color,
                    image: row.chatter_image,
                    total: row.chatter_total,
                },
                score: row.score,
                ranking: row.ranking,
            })
            .collect())
    }

    #[instrument(skip(self))]
    pub async fn get_leaderboard_on_channel(
        &self,
        channel_id: &ChannelId,
        limit: i64,
        offset: i64,
    ) -> SqlxResult<Vec<ScoreResponse>> {
        #[derive(sqlx::FromRow)]
        struct LeaderboardRow {
            chatter_id: ChatterId,
            chatter_name: String,
            chatter_login: String,
            chatter_color: String,
            chatter_image: String,
            chatter_total: i64,
            score: i64,
            ranking: i64,
        }

        let rows = sqlx::query_as::<_, LeaderboardRow>(&format!(
            r#"
            SELECT 
                rs.chatter_id,
                c.name as chatter_name,
                c.login as chatter_login,
                c.color as chatter_color,
                c.image as chatter_image,
                c.total as chatter_total,
                rs.score, 
                rs.ranking
            FROM ranked_scores_view rs
            JOIN chatter c ON rs.chatter_id = c.id
            WHERE rs.channel_id = $1
            ORDER BY rs.rank ASC
            LIMIT $2 OFFSET $3
            "#
        ))
        .bind(channel_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        let channel = ChannelRepository::new(self.pool)
            .get_by_id(channel_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let channel_chatter = self
            .get_by_id(&ChatterId(channel_id.0.clone()))
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let channel_summary = ChannelSummary {
            id: channel.id.0,
            name: channel_chatter.name,
            login: channel_chatter.login,
            color: channel_chatter.color,
            image: channel_chatter.image,
            total_chatter: channel_chatter.total,
            total_channel: channel.channel_total,
        };

        Ok(rows
            .into_iter()
            .map(|row| ScoreResponse {
                channel: channel_summary.clone(),
                chatter: ChatterSummary {
                    id: row.chatter_id.0,
                    name: row.chatter_name,
                    login: row.chatter_login,
                    color: row.chatter_color,
                    image: row.chatter_image,
                    total: row.chatter_total,
                },
                score: row.score,
                ranking: row.ranking,
            })
            .collect())
    }
}

pub struct ChannelRepository {
    pool: &'static Pool<Postgres>,
}

#[async_trait]
impl Repository<Postgres> for ChannelRepository {
    type Ident = ChannelId;
    type Output = Channel;

    const BASE_QUERY: &'static str = sql::SELECT_CHANNEL_BASE;
    fn new(pool: &'static Pool<Postgres>) -> Self {
        Self { pool }
    }

    fn pool(&self) -> &'static Pool<Postgres> {
        self.pool
    }
}

impl ChannelRepository {
    pub async fn insert_many(&self, broadcasters: &Vec<Chatter>) -> SqlxResult<()> {
        let mut tx = Tx::begin(self.pool).await?;
        for broadcaster in broadcasters {
            if sqlx::query(
                r#"
                SELECT EXISTS (
                    SELECT id FROM chatter WHERE id = $1
                )
                "#,
            )
            .bind(broadcaster.id.clone())
            .fetch_optional(&mut *tx.inner)
            .await?
            .is_none()
            {
                tx.insert_chatter(
                    &broadcaster.id,
                    &broadcaster.name,
                    &broadcaster.login,
                    &broadcaster.color,
                    &broadcaster.image,
                    broadcaster.total,
                )
                .await?;
            }

            let id = broadcaster.id.clone();
            tx.insert_channel(&id.into(), 0).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    #[instrument(skip(self, broadcaster), fields(channel = broadcaster.id.0))]
    pub async fn insert(&self, broadcaster: Chatter) -> SqlxResult<()> {
        let mut tx = Tx::begin(self.pool).await?;
        if sqlx::query(
            r#"
            SELECT EXISTS (
                SELECT id FROM chatter WHERE id = $1
            )
            "#,
        )
        .bind(broadcaster.id.clone())
        .fetch_optional(&mut *tx.inner)
        .await?
        .is_none()
        {
            tx.insert_chatter(
                &broadcaster.id,
                &broadcaster.name,
                &broadcaster.login,
                &broadcaster.color,
                &broadcaster.image,
                broadcaster.total,
            )
            .await?;
        }

        tx.insert_channel(&broadcaster.id.into(), 0).await?;
        tx.commit().await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_with_score(&self, id: &ChannelId) -> SqlxResult<ChannelResponse> {
        let channel = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let chatter = ChatterRepository::new(self.pool)
            .get_by_id(&ChatterId(id.0.clone()))
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let scores = self.get_scores_for(id).await?;

        Ok(ChannelResponse {
            id: channel.id.0,
            name: chatter.name,
            login: chatter.login,
            color: chatter.color,
            image: chatter.image,
            total_chatter: chatter.total,
            total_channel: channel.channel_total,
            scores,
        })
    }

    #[instrument(skip(self))]
    async fn get_scores_for(&self, channel_id: &ChannelId) -> SqlxResult<Vec<ScoreResponse>> {
        #[derive(sqlx::FromRow)]
        struct ScoreWithChatter {
            channel_id: ChannelId,
            chatter_id: ChatterId,
            score: i64,
            ranking: i64,
            chatter_name: String,
            chatter_login: String,
            chatter_color: String,
            chatter_image: String,
            chatter_total: i64,
        }

        let rows = sqlx::query_as::<_, ScoreWithChatter>(
            r#"
            SELECT 
                rs.channel_id,
                rs.chatter_id,
                rs.score,
                rs.ranking,
                c.name as chatter_name,
                c.login as chatter_login, 
                c.color as chatter_color,
                c.image as chatter_image,
                c.total as chatter_total,
            FROM ranked_scores_view rs
            JOIN chatter c ON rs.chatter_id = c.id
            WHERE rs.channel_id = $1
            ORDER BY rs.ranking ASC
            "#,
        )
        .bind(channel_id)
        .fetch_all(self.pool)
        .await?;

        let channel = self
            .get_by_id(channel_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let channel_chatter = ChatterRepository::new(self.pool)
            .get_by_id(&ChatterId(channel_id.0.clone()))
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let channel_summary = ChannelSummary {
            id: channel.id.0,
            name: channel_chatter.name,
            login: channel_chatter.login,
            color: channel_chatter.color,
            image: channel_chatter.image,
            total_chatter: channel_chatter.total,
            total_channel: channel.channel_total,
        };

        Ok(rows
            .into_iter()
            .map(|row| ScoreResponse {
                channel: channel_summary.clone(),
                chatter: ChatterSummary {
                    id: row.chatter_id.0,
                    name: row.chatter_name,
                    login: row.chatter_login,
                    color: row.chatter_color,
                    image: row.chatter_image,
                    total: row.chatter_total,
                },
                score: row.score,
                ranking: row.ranking,
            })
            .collect())
    }
}

#[derive(Debug)]
pub struct ScoreRepository {
    pool: &'static Pool<Postgres>,
}

impl ScoreRepository {
    pub fn new(pool: &'static Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &Pool<Postgres> {
        self.pool
    }

    pub async fn for_chatter(&self, id: &ChatterId) -> SqlxResult<(String, i64)> {
        #[derive(sqlx::FromRow, Deserialize)]
        struct IntWrapper {
            name: String,
            total: i64,
        }

        match sqlx::query_as::<_, IntWrapper>(
            r#"
            SELECT name, total FROM chatter WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?
        {
            Some(v) => Ok((v.name, v.total)),
            None => Err(sqlx::Error::RowNotFound),
        }
    }

    #[instrument(skip(self, channel, chatter))]
    pub async fn increment(&self, channel: &ChannelId, chatter: &ChatterId) -> SqlxResult<()> {
        let res = sqlx::query(
            r#"
            INSERT INTO score (
                channel_id,
                chatter_id,
                score,
                created_at,
                updated_at
            )
            VALUES ($1, $2, 1, NOW(), NOW())
            ON CONFLICT (chatter_id, channel_id)
            DO UPDATE SET
                score = score.score + 1,
                updated_at = NOW()
            "#,
        )
        .bind(channel.clone())
        .bind(chatter.clone())
        .execute(self.pool)
        .await?;

        let mut tx = Tx::begin(self.pool()).await?;
        tx.recalculate_channel_total(channel).await?;
        tx.recalculate_chatter_total(chatter).await?;
        tx.commit().await?;

        tracing::debug!(
            increment_result = ?res,
            channel_id = %channel,
            chatter_id = %chatter,
            "incremented score and refreshed counts"
        );

        Ok(())
    }

    #[instrument(skip(self, channel, chatter))]
    pub async fn get_relation_for(
        &self,
        channel: ChannelId,
        chatter: ChatterId,
    ) -> SqlxResult<ScoreResponse> {
        tracing::warn!("this function is not implemented!");
        todo!()
    }
}
