use std::collections::HashMap;

use redis::AsyncCommands;
use sqlx::{Pool, Postgres, Transaction};
use tracing::instrument;

use crate::db::prelude::{Channel, ChannelId, ChatterId};
use crate::db::redis::migrator::{LeaderboardMap, LeaderboardRow, transform, util};
use crate::db::redis::redis_pool::{self, KeyType, RedisResult};
use crate::redis_key;

#[derive(Debug)]
pub struct RedisHandler<'a, R: AsyncCommands + Sync>(pub &'a mut R);

impl<'a, R> RedisHandler<'a, R>
where
    R: AsyncCommands + Sync,
{
    pub async fn fetch_keys(&mut self, key_type: KeyType) -> RedisResult<Vec<String>> {
        let wildcard_key = match key_type {
            KeyType::Chatter => redis_key!(user, total),
            KeyType::Channel => redis_key!(channel, total),
        };

        Ok(self.0.keys(wildcard_key).await?)
    }

    #[instrument(skip(self, key, key_type))]
    pub async fn fetch_leaderboard(
        &mut self,
        key: &str,
        key_type: KeyType,
    ) -> RedisResult<HashMap<String, i64>> {
        let redis_key = match key_type {
            redis_pool::KeyType::Chatter => redis_key!(user, leaderboard, key),
            redis_pool::KeyType::Channel => redis_key!(channel, leaderboard, key),
        };

        let vec_raw: Vec<(String, i64)> = self.0.zrange_withscores(redis_key, 0, -1).await?;

        let map_raw: HashMap<String, i64> =
            vec_raw.into_iter().map(|(key, val)| (key, val)).collect();

        tracing::debug!(user_name = ?key, leaderboard = ?map_raw, "mapped raw leaderboard");

        Ok(map_raw)
    }

    #[instrument(skip(self))]
    pub async fn fetch_alias_leaderboards(
        &mut self,
        keylist: &[String],
    ) -> RedisResult<HashMap<String, i64>> {
        let mut pipeline = redis::pipe();
        let mut output = HashMap::new();

        tracing::debug!(
            keys_count = keylist.len(),
            "building redis pipeline for alias fetch"
        );

        keylist.into_iter().for_each(|alias| {
            let key = redis_key!(user, leaderboard, alias);
            tracing::debug!(key, "built alias leaderboard key");

            pipeline.zrange_withscores(key, 0, -1);
        });

        let results: Vec<Vec<(String, i64)>> = match pipeline.query_async(self.0).await {
            Ok(val) => val,
            Err(e) => {
                tracing::error!(error = ?e, "failed to fetch aliases from cache");
                return Err(redis_pool::RedisErr::RedisClientError(e));
            }
        };

        tracing::debug!(?results, "fetched aliased leaderboards");

        for leaderboard in results {
            leaderboard.iter().for_each(|(channel_raw, score)| {
                tracing::debug!(channel_raw, score, "parse raw channel name");

                let channel_trimmed = transform::trim_octo(&channel_raw);
                let channel_id = util::resolve_channel_id(&channel_trimmed);

                output
                    .entry(channel_id)
                    .and_modify(|curr: &mut i64| *curr += *score)
                    .or_insert(*score);
            });
        }

        tracing::debug!(?output, "returning mapped leaderboard");

        Ok(output)
    }
}

#[derive(Debug)]
pub struct PgHandler<'a>(pub &'a Pool<Postgres>);

impl<'a> PgHandler<'a> {
    #[instrument(skip(self, leaderboards, offset_days))]
    pub async fn migrate(
        &self,
        leaderboards: LeaderboardMap,
        offset_days: i64,
    ) -> Result<(), sqlx::Error> {
        let timestamp = util::create_timestamp(offset_days);
        let mut tx = self.0.begin().await?;

        Triggers(&mut tx).disable().await?;

        for (chatter_id, leaderboard) in leaderboards {
            tracing::debug!(chatter_id, ?leaderboard, "handling leaderboard");

            let chatter_id = ChatterId(chatter_id);

            for (channel_id, score) in leaderboard {
                Self::record_score_event(
                    &mut tx,
                    &chatter_id,
                    &ChannelId(channel_id),
                    score,
                    timestamp,
                )
                .await?;
            }
        }

        Triggers(&mut tx).enable().await?;
        Self::recalculate_aggregates(&mut tx).await?;

        tx.commit().await?;

        Ok(())
    }

    #[instrument(skip(self, chatter_id, leaderboard, offset_days))]
    pub async fn migrate_alias(
        &self,
        chatter_id: &ChatterId,
        leaderboard: LeaderboardRow,
        offset_days: i64,
    ) -> Result<(), sqlx::Error> {
        let timestamp = util::create_timestamp(offset_days);

        let mut tx = self.0.begin().await?;
        Triggers(&mut tx).disable().await?;

        for (channel_id, score) in leaderboard {
            Self::record_score_event(
                &mut tx,
                chatter_id,
                &ChannelId(channel_id),
                score,
                timestamp,
            )
            .await?;
        }

        Triggers(&mut tx).enable().await?;
        Self::recalculate_aggregates(&mut tx).await?;

        tx.commit().await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn insert_reply_config(&self, channels: &[Channel]) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO reply (id) 
            SELECT channel.id FROM channel
            ON CONFLICT (id) 
            DO NOTHING;
            "#,
        )
        .execute(self.0)
        .await?;

        Ok(())
    }

    #[instrument(skip(self, chatter_id))]
    pub async fn clear_scores_for_chatter<'id>(
        &self,
        chatter_id: &'id ChatterId,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.0.begin().await?;

        sqlx::query!(
            r#"
            DELETE FROM score_event 
            WHERE chatter_id = $1
            "#,
            chatter_id.0,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            DELETE FROM score
            WHERE chatter_id = $1
            "#,
            chatter_id.0,
        )
        .execute(&mut *tx)
        .await?;

        Self::recalculate_aggregates(&mut tx).await?;

        tx.commit().await?;

        Ok(())
    }

    #[instrument(skip(tx, chatter_id, channel_id, count, base_timestamp))]
    async fn record_score_event<'id>(
        tx: &mut Transaction<'_, Postgres>,
        chatter_id: &'id ChatterId,
        channel_id: &'id ChannelId,
        count: i64,
        base_timestamp: chrono::NaiveDateTime,
    ) -> Result<(), sqlx::Error> {
        if count <= 0 {
            tracing::warn!("invalid count - nothing to insert");
            return Ok(());
        }

        sqlx::query!(
            r#"
            INSERT INTO score_event (chatter_id, channel_id, earned_at)
            SELECT 
                $1::varchar(16),
                $2::varchar(16),
                $3::timestamp + make_interval(secs => generate_series(1, $4))
            "#,
            chatter_id.0,
            channel_id.0,
            base_timestamp,
            count as i32,
        )
        .execute(tx.as_mut())
        .await?;

        Ok(())
    }

    #[instrument(skip(tx))]
    async fn recalculate_aggregates(tx: &mut Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE chatter 
            SET 
                total = (
                    SELECT COALESCE(COUNT(*), 0) 
                    FROM score_event
                    WHERE chatter_id = chatter.id
                ), 
                updated_at = NOW()
            "#
        )
        .execute(tx.as_mut())
        .await?;

        sqlx::query!(
            r#"
            UPDATE channel 
            SET 
                channel_total = (
                    SELECT COALESCE(COUNT(*), 0) 
                    FROM score_event
                    WHERE channel_id = channel.id
                ),
                updated_at = NOW()
            "#
        )
        .execute(tx.as_mut())
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO score (
                chatter_id,
                channel_id,
                score,
                updated_at
            )
            SELECT 
                chatter_id,
                channel_id, 
                COUNT(*), 
                NOW()
            FROM score_event 
            GROUP BY chatter_id, channel_id
            ON CONFLICT (chatter_id, channel_id)
            DO UPDATE SET 
                score = EXCLUDED.score, 
                updated_at = EXCLUDED.updated_at
            "#
        )
        .execute(tx.as_mut())
        .await?;

        Ok(())
    }
}

pub struct Triggers<'a, 'b>(&'a mut Transaction<'b, Postgres>);

impl<'a, 'b> Triggers<'a, 'b> {
    #[instrument(skip(self))]
    async fn enable(&mut self) -> Result<(), sqlx::Error> {
        tracing::warn!("enable score_event-score incrementer triggers");
        sqlx::query!("ALTER TABLE score_event ENABLE TRIGGER score_event_increment_trigger")
            .execute(self.0.as_mut())
            .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn disable(&mut self) -> Result<(), sqlx::Error> {
        tracing::warn!("disable score_event-score incrementer triggers");
        sqlx::query!("ALTER TABLE score_event DISABLE TRIGGER score_event_increment_trigger")
            .execute(self.0.as_mut())
            .await?;

        Ok(())
    }
}
