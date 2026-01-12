use sqlx::{Pool, Postgres, Result as SqlxResult};
use tracing::instrument;

use crate::db::models::PaginatedResponse;
use crate::db::models::channel::{
    ChannelId, ChannelLeaderboardEntry, ChannelLeaderboardRow, ChannelScoreSummary,
};
use crate::db::models::chatter::{
    ChatterId, ChatterLeaderboardEntry, ChatterLeaderboardRow, ChatterScoreSummary,
};
use crate::db::prelude::{
    Channel, ChannelRepository, Chatter, ChatterRepository, Repository, Score, ScoreSummary,
};

pub struct LeaderboardRepository {
    pool: &'static Pool<Postgres>,
}

impl LeaderboardRepository {
    pub fn new(pool: &'static Pool<Postgres>) -> Self {
        Self { pool }
    }

    #[instrument(skip(self, channel, chatter, value), fields(channel = channel.id.0, chatter = chatter.id.0))]
    pub async fn increment_by(
        &self,
        channel: &Chatter,
        chatter: &Chatter,
        value: i64,
    ) -> SqlxResult<Option<ScoreSummary>> {
        let score = sqlx::query_as!(
            ScoreSummary,
            r#"
            INSERT INTO score (
                channel_id,
                chatter_id,
                score,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, NOW(), NOW())
            ON CONFLICT (channel_id, chatter_id)
            DO UPDATE SET
                score = score.score + $3,
                updated_at = NOW()
            RETURNING 
                channel_id,
                chatter_id,
                score
            "#,
            channel.id.0,
            chatter.id.0,
            value
        )
        .fetch_optional(self.pool)
        .await;

        match score {
            Ok(Some(v)) => {
                let chatter_repo = ChatterRepository::new(self.pool);
                let channel_repo = ChannelRepository::new(self.pool);

                chatter_repo.increment_score(chatter).await?;
                channel_repo
                    .increment_score(&Channel::from(channel.clone()))
                    .await?;

                Ok(Some(v))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::error!(error = ?e, "score increment failure");
                Err(e)
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn increment_score(
        &self,
        channel: &Chatter,
        chatter: &Chatter,
    ) -> SqlxResult<Option<ScoreSummary>> {
        self.increment_by(channel, chatter, 1).await
    }

    #[instrument(skip(self))]
    pub async fn get_chatter_rank(&self, chatter_id: &ChatterId) -> SqlxResult<Option<i64>> {
        sqlx::query_scalar!("SELECT get_chatter_rank($1)", chatter_id.0)
            .fetch_one(self.pool)
            .await
    }

    #[instrument(skip(self))]
    pub async fn get_channel_rank(&self, channel_id: &ChannelId) -> SqlxResult<Option<i64>> {
        sqlx::query_scalar!("SELECT get_channel_rank($1)", &channel_id.0)
            .fetch_one(self.pool)
            .await
    }

    #[instrument(skip(self))]
    pub async fn get_relational_score(
        &self,
        channel_id: &ChannelId,
        chatter_id: &ChatterId,
    ) -> SqlxResult<Option<Score>> {
        sqlx::query_as!(
            Score,
            r#"
            SELECT * FROM score
            WHERE chatter_id = $1 
            AND channel_id = $2
            "#,
            &channel_id.0,
            &chatter_id.0
        )
        .fetch_optional(self.pool)
        .await
    }

    #[instrument(skip(self))]
    pub async fn get_single_channel_leaderboard(
        &self,
        id: ChannelId,
    ) -> SqlxResult<Option<ChannelLeaderboardEntry>> {
        let channel = sqlx::query_as!(
            ChannelLeaderboardRow,
            r#"
            SELECT 
                id AS "id!",
                name AS "name!",
                login AS "login!",
                color AS "color!",
                image AS "image!",
                total_chatter AS "total_chatter!",
                total_channel AS "total_channel!",
                ranking AS "ranking!",
                created_at AS "created_at!",
                updated_at AS "updated_at!"
            FROM channel_leaderboard
            WHERE id = $1
            "#,
            &id.to_string()
        )
        .fetch_optional(self.pool)
        .await?;

        match channel {
            Some(ch) => {
                let scores = self.get_chatter_scores_batch(&[id]).await?;
                let chatter_scores: Vec<ChatterScoreSummary> = scores
                    .iter()
                    .filter(|s| s.channel_id == ch.id)
                    .cloned()
                    .map(|s| s.into())
                    .collect();

                Ok(Some(ch.into_leaderboard_entry(chatter_scores)))
            }

            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    pub async fn get_single_chatter_leaderboard(
        &self,
        id: ChatterId,
    ) -> SqlxResult<Option<ChatterLeaderboardEntry>> {
        let chatter = sqlx::query_as!(
            ChatterLeaderboardRow,
            r#"
            SELECT 
                id as "id!",
                name as "name!",
                login as "login!",
                color as "color!",
                image as "image!",
                total as "total!",
                private as "private!",
                ranking as "ranking!",
                created_at as "created_at!",
                updated_at as "updated_at!"
            FROM chatter_leaderboard
            WHERE id = $1
            "#,
            &id.to_string()
        )
        .fetch_optional(self.pool)
        .await?;

        match chatter {
            Some(ch) => {
                let scores = self.get_channel_scores_batch(&[ch.id.clone().0]).await?;
                let channel_scores: Vec<ChannelScoreSummary> = scores
                    .iter()
                    .filter(|s| s.chatter_id == ch.id)
                    .cloned()
                    .map(|s| s.into())
                    .collect();

                Ok(Some(ch.into_leaderboard_entry(channel_scores)))
            }

            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    pub async fn get_chatter_leaderboard(
        &self,
        limit: i64,
        offset: i64,
    ) -> SqlxResult<PaginatedResponse<ChatterLeaderboardEntry>> {
        let total_items: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM chatter")
            .fetch_one(self.pool)
            .await?
            .unwrap_or_default();

        let chatters = sqlx::query_as!(
            ChatterLeaderboardRow,
            r#"
            SELECT 
                id  as "id!",
                name as "name!",
                login as "login!",
                color as "color!",
                image as "image!",
                total as "total!",
                private as "private!",
                ranking as "ranking!",
                created_at as "created_at!",
                updated_at as "updated_at!"
            FROM chatter_leaderboard
            ORDER BY ranking ASC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(self.pool)
        .await?;

        let ids: &[String] = &chatters.iter().map(|c| c.id.clone().0).collect::<Vec<_>>();
        let scores = if !ids.is_empty() {
            self.get_channel_scores_batch(&ids).await?
        } else {
            Vec::new()
        };

        let mut entries = Vec::new();
        for chatter in chatters {
            let channel_scores: Vec<ChannelScoreSummary> = scores
                .iter()
                .filter(|s| s.chatter_id == chatter.id)
                .cloned()
                .map(|s| s.into())
                .collect();

            // channel_scores.sort_by(|a, b| b.score.cmp(&a.score));
            entries.push(chatter.into_leaderboard_entry(channel_scores));
        }

        Ok(PaginatedResponse::new(
            entries,
            total_items,
            limit,
            offset / limit + 1,
        ))
    }

    #[instrument(skip(self))]
    pub async fn get_channel_leaderboard(
        &self,
        limit: i64,
        offset: i64,
    ) -> SqlxResult<PaginatedResponse<ChannelLeaderboardEntry>> {
        let total_items: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM channel")
            .fetch_one(self.pool)
            .await?;

        let channels = sqlx::query_as!(
            ChannelLeaderboardRow,
            r#"
            SELECT 
                id AS "id!",
                name AS "name!",
                login AS "login!",
                color AS "color!",
                image AS "image!",
                total_chatter AS "total_chatter!",
                total_channel AS "total_channel!",
                ranking AS "ranking!",
                created_at AS "created_at!",
                updated_at AS "updated_at!"
            FROM channel_leaderboard
            ORDER BY ranking ASC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(self.pool)
        .await?;

        let ids: Vec<ChannelId> = channels.iter().map(|ch| ch.id.clone().into()).collect();
        let scores = if !ids.is_empty() {
            self.get_chatter_scores_batch(&ids).await?
        } else {
            Vec::new()
        };

        let mut entries = Vec::new();
        for channel in channels {
            let scores: Vec<ChatterScoreSummary> = scores
                .iter()
                .filter(|s| s.channel_id == channel.id)
                .cloned()
                .map(|s| s.into())
                .collect();

            entries.push(channel.into_leaderboard_entry(scores));
        }

        Ok(PaginatedResponse::new(
            entries,
            total_items,
            limit,
            offset / limit + 1,
        ))
    }

    #[instrument(skip(self, ids))]
    async fn get_chatter_scores_batch(
        &self,
        ids: &[ChannelId],
    ) -> SqlxResult<Vec<ChatterScoreSummary>> {
        let ids: &[String] = &ids.iter().map(|id| id.0.clone().into()).collect::<Vec<_>>();

        #[derive(sqlx::FromRow)]
        struct TempRow {
            channel_id: String,
            chatter_id: String,
            chatter_name: String,
            chatter_login: String,
            chatter_color: String,
            chatter_image: String,
            score: i64,
            ranking: i64,
        }

        let rows = sqlx::query_as!(
            TempRow,
            r#"
            SELECT 
                rs.chatter_id as "chatter_id!",
                rs.channel_id as "channel_id!",
                c.name as "chatter_name!",
                c.login as "chatter_login!",
                c.color as "chatter_color!",
                c.image as "chatter_image!",
                rs.score as "score!",
                rs.ranking as "ranking!"
            FROM ranked_scores_view_per_channel rs
            JOIN chatter c ON rs.chatter_id = c.id
            WHERE rs.channel_id = ANY($1)
            ORDER BY rs.channel_id, rs.score DESC
            "#,
            &ids,
        )
        .fetch_all(self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ChatterScoreSummary {
                chatter_id: r.chatter_id.into(),
                channel_id: r.channel_id.into(),
                chatter_login: r.chatter_login,
                chatter_name: r.chatter_name,
                chatter_color: r.chatter_color,
                chatter_image: r.chatter_image,
                score: r.score,
                ranking: r.ranking,
            })
            .collect())
    }

    #[instrument(skip(self))]
    async fn get_channel_scores_batch(
        &self,
        ids: &[String],
    ) -> SqlxResult<Vec<ChannelScoreSummary>> {
        #[derive(sqlx::FromRow)]
        struct TempRow {
            channel_id: String,
            chatter_id: String,
            channel_name: String,
            channel_login: String,
            channel_color: String,
            channel_image: String,
            score: i64,
            ranking: i64,
        }

        let rows = sqlx::query_as!(
            TempRow,
            r#"
            SELECT 
                rs.channel_id as "channel_id!",
                rs.chatter_id as "chatter_id!",
                ch_chatter.name as "channel_name!",
                ch_chatter.login as "channel_login!",
                ch_chatter.color as "channel_color!",
                ch_chatter.image as "channel_image!",
                rs.score as "score!",
                rs.ranking as "ranking!"
            FROM ranked_scores_view_per_channel rs
            JOIN channel ch ON rs.channel_id = ch.id
            JOIN chatter ch_chatter ON ch.id = ch_chatter.id
            WHERE rs.chatter_id = ANY($1)
            ORDER BY rs.chatter_id, rs.score DESC
            "#,
            &ids,
        )
        .fetch_all(self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ChannelScoreSummary {
                chatter_id: r.chatter_id.into(),
                channel_id: r.channel_id.into(),
                channel_login: r.channel_login,
                channel_name: r.channel_name,
                channel_color: r.channel_color,
                channel_image: r.channel_image,
                score: r.score,
                ranking: r.ranking,
            })
            .collect())
    }
}
