use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::{Database, Pool, Postgres};
use tracing::{Instrument, debug, info, instrument, trace};

use super::pg::{DisplayableChannel, DisplayableChatter};
use crate::db::pg::{PgErr, PgResult};
use crate::util::helix::HelixUser;

#[async_trait]
pub trait DbUser<T, R>
where
    T: Database,
    R: Sized,
{
    async fn inc_total(conn: &'static Pool<T>, chatter: &mut Chatter) -> PgResult<()>;
    async fn inc_total_by(conn: &'static Pool<T>, chatter: &mut Chatter, amt: i64) -> PgResult<()>;

    async fn upsert_multi(conn: &'static Pool<T>, vals: &Vec<R>) -> PgResult<()>;
    async fn upsert(conn: &'static Pool<T>, val: &R) -> PgResult<()>;

    async fn get_all(conn: &'static Pool<T>) -> PgResult<Vec<R>>;
    async fn get_by_id(conn: &'static Pool<T>, id: &str) -> PgResult<R>;
    async fn get_by_ids(conn: &'static Pool<Postgres>, ids: &Vec<String>) -> PgResult<Vec<R>>;
    async fn get_by_login(conn: &'static Pool<T>, login: &str) -> PgResult<R>;
}

#[async_trait]
impl DbUser<Postgres, Self> for Chatter {
    #[instrument(skip(conn, val), fields(target_id = val.id))]
    async fn inc_total(conn: &'static Pool<Postgres>, val: &mut Self) -> PgResult<()> {
        val.total += 1;
        Self::upsert(conn, val).await?;

        Ok(())
    }

    #[instrument(skip(conn, val), fields(target_id = val.id))]
    async fn inc_total_by(conn: &'static Pool<Postgres>, val: &mut Self, amt: i64) -> PgResult<()> {
        val.total += amt;
        Self::upsert(conn, val).await?;

        Ok(())
    }

    #[instrument(skip(conn, vals), fields(count = vals.len()))]
    async fn upsert_multi(conn: &'static Pool<Postgres>, vals: &Vec<Self>) -> PgResult<()> {
        let mut tx = conn.begin().await?;
        for v in vals {
            sqlx::query_as!(
                Chatter,
                r#"
                    INSERT INTO chatter (
                        id, 
                        login, 
                        name, 
                        color, 
                        image, 
                        total, 
                        private
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    ON CONFLICT (id)
                    DO UPDATE SET
                        login = $2,
                        name = $3,
                        color = $4,
                        image = $5,
                        total = $6,
                        private = $7,
                        updated_at = NOW()
                    "#,
                v.id,
                v.login,
                v.name,
                v.color,
                v.image,
                v.total as _,
                v.private,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    #[instrument(skip(conn, val), fields(target_id = val.id))]
    async fn upsert(conn: &'static Pool<Postgres>, val: &Self) -> PgResult<()> {
        sqlx::query_as!(
            Chatter,
            r#"
            INSERT INTO chatter (
                id, 
                login, 
                name, 
                color, 
                image, 
                total, 
                private
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id)
            DO UPDATE SET
                login = $2,
                name = $3,
                color = $4,
                image = $5,
                total = $6,
                private = $7,
                updated_at = NOW()
            "#,
            val.id,
            val.login,
            val.name,
            val.color,
            val.image,
            val.total as _,
            val.private,
        )
        .execute(conn)
        .await?;

        Ok(())
    }

    #[instrument(skip(conn))]
    async fn get_all(conn: &'static Pool<Postgres>) -> PgResult<Vec<Self>> {
        sqlx::query_as!(
            Chatter,
            r#"
            SELECT * FROM chatter
            "#
        )
        .fetch_all(conn)
        .await
        .map_err(|err| PgErr::SqlxError(err))
    }

    #[instrument(skip(conn, ids), fields(count = ids.len()))]
    async fn get_by_ids(conn: &'static Pool<Postgres>, ids: &Vec<String>) -> PgResult<Vec<Self>> {
        let mut tx = conn.begin().await?;
        let mut fetched = Vec::new();

        for id in ids {
            let row = sqlx::query_as!(
                Chatter,
                r#"
                SELECT * FROM chatter
                WHERE id = $1
                "#,
                id.into()
            )
            .fetch_optional(&mut *tx)
            .await?;

            fetched.push(row);
        }

        tx.commit().await?;
        let filtered = fetched
            .into_iter()
            .filter(|f| f.is_some())
            .map(|f| f.unwrap())
            .collect::<Vec<_>>();

        Ok(filtered)
    }

    #[instrument(skip(conn, id), fields(target_id = id))]
    async fn get_by_id(conn: &'static Pool<Postgres>, id: &str) -> PgResult<Self> {
        sqlx::query_as!(
            Chatter,
            r#"
            SELECT * FROM chatter
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(conn)
        .await
        .map_err(|err| PgErr::SqlxError(err))
    }

    #[instrument(skip(conn, login), fields(target_login = login))]
    async fn get_by_login(conn: &'static Pool<Postgres>, login: &str) -> PgResult<Self> {
        let login = login.to_lowercase();
        sqlx::query_as!(
            Chatter,
            r#"
            SELECT * FROM chatter
            WHERE login = $1
            "#,
            login
        )
        .fetch_one(conn)
        .await
        .map_err(|err| PgErr::SqlxError(err))
    }
}

#[async_trait]
impl DbUser<Postgres, Self> for Channel {
    #[instrument(skip(conn, broadcaster))]
    async fn inc_total(conn: &'static Pool<Postgres>, broadcaster: &mut Chatter) -> PgResult<()> {
        let mut channel: Channel = Channel::from(broadcaster.clone());
        channel.total += 1;
        let mut tx = conn.begin().await?;

        sqlx::query!(
            r#"
            INSERT INTO chatter (
                id, 
                login,
                name,
                color,
                image,
                total,
                private
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id)
            DO NOTHING
            "#,
            broadcaster.id,
            broadcaster.login,
            broadcaster.name,
            broadcaster.color,
            broadcaster.image,
            broadcaster.total as _,
            broadcaster.private
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO channel (id, total)
            VALUES ($1, $2)
            ON CONFLICT (id) 
            DO UPDATE SET
                total = $2,
                updated_at = NOW()
            "#,
            channel.id,
            broadcaster.total as _,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    #[instrument(skip(conn, broadcaster, amt))]
    async fn inc_total_by(
        conn: &'static Pool<Postgres>,
        broadcaster: &mut Chatter,
        amt: i64,
    ) -> PgResult<()> {
        let mut channel: Channel = Channel::from(broadcaster);
        channel.total += amt;

        Self::upsert(conn, &channel).await?;
        Ok(())
    }

    #[instrument(skip(conn, vals))]
    async fn upsert_multi(conn: &'static Pool<Postgres>, vals: &Vec<Self>) -> PgResult<()> {
        let mut tx = conn.begin().await?;
        for v in vals {
            sqlx::query_as!(
                Channel,
                r#"
                INSERT INTO channel (id, total)
                VALUES ($1, $2)
                ON CONFLICT (id)
                DO UPDATE SET
                    total = $2,
                    updated_at = NOW()
                "#,
                v.id,
                v.total as _
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    #[instrument(skip(conn, val))]
    async fn upsert(conn: &'static Pool<Postgres>, val: &Self) -> PgResult<()> {
        sqlx::query_as!(
            Channel,
            r#"
            INSERT INTO channel (id, total)
            VALUES ($1, $2)
            ON CONFLICT (id)
            DO UPDATE SET
                total = $2,
                updated_at = NOW()
            "#,
            val.id,
            val.total as _
        )
        .execute(conn)
        .await?;

        Ok(())
    }

    #[instrument(skip(conn))]
    async fn get_all(conn: &'static Pool<Postgres>) -> PgResult<Vec<Self>> {
        sqlx::query_as!(
            Channel,
            r#"
            SELECT * FROM channel
            "#
        )
        .fetch_all(conn)
        .await
        .map_err(|err| PgErr::SqlxError(err))
    }

    #[instrument(skip(conn, id))]
    async fn get_by_id(conn: &'static Pool<Postgres>, id: &str) -> PgResult<Self> {
        Ok(sqlx::query_as!(
            Channel,
            r#"
            SELECT * FROM channel
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(conn)
        .await?)
    }

    #[instrument(skip(conn, ids))]
    async fn get_by_ids(conn: &'static Pool<Postgres>, ids: &Vec<String>) -> PgResult<Vec<Self>> {
        let mut tx = conn.begin().await?;
        let mut fetched = Vec::new();

        for id in ids {
            let row = sqlx::query_as!(
                Channel,
                r#"
                SELECT * FROM channel
                WHERE id = $1
                "#,
                id
            )
            .fetch_optional(&mut *tx)
            .await?;

            fetched.push(row);
        }

        tx.commit().await?;
        let filtered = fetched
            .into_iter()
            .filter(|f| f.is_some())
            .map(|f| f.unwrap())
            .collect::<Vec<_>>();

        Ok(filtered)
    }

    #[instrument(skip(conn, login))]
    async fn get_by_login(conn: &'static Pool<Postgres>, login: &str) -> PgResult<Self> {
        let chatter = Chatter::get_by_login(conn, login).await?;
        Ok(Self::get_by_id(conn, &chatter.id).await?)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Chatter {
    pub id: String,
    pub login: String,
    pub name: String,
    pub color: String,
    pub image: String,
    pub total: i64,
    pub private: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Chatter {
    #[instrument(skip(conn))]
    pub async fn query_by_id(
        conn: &'static Pool<Postgres>,
        id: &str,
    ) -> PgResult<DisplayableChatter> {
        let row = sqlx::query_as::<_, DisplayableChatter>(
            r#"
            SELECT 
                c.id,
                c.name,
                c.login,
                c.color,
                c.image,
                c.total::text as total,
                COALESCE(
                    json_agg(
                        json_build_object(
                            'channel_id', s.channel_id,
                            'chatter_id', s.chatter_id,
                            'score', s.score,
                            'rank', s.score_rank,
                            'channel', json_build_object(
                                'id', ch.id,
                                'name', channel_chatter.name,
                                'login', channel_chatter.login,
                                'color', channel_chatter.color,
                                'image', channel_chatter.image,
                                'total_as_chatter', channel_chatter.total,
                                'total_as_broadcaster', ch.total
                            )
                        ) ORDER BY s.score_rank
                    ) FILTER (WHERE s.channel_id IS NOT NULL),
                    '[]'::json
                ) as channels
            FROM chatter c
            LEFT JOIN (
                SELECT 
                    channel_id,
                    chatter_id,
                    score,
                    created_at,
                    ROW_NUMBER() OVER (
                        PARTITION BY channel_id 
                        ORDER BY score DESC, created_at ASC
                    ) AS score_rank
                FROM score
            ) s ON c.id = s.chatter_id
            LEFT JOIN channel ch ON s.channel_id = ch.id
            LEFT JOIN chatter channel_chatter ON ch.id = channel_chatter.id
            WHERE c.id = $1
            GROUP BY c.id, c.name, c.login, c.color, c.image, c.total, c.created_at
            ORDER BY c.total DESC, c.created_at ASC
            "#,
        )
        .bind(id)
        .fetch_one(conn)
        .await?;

        Ok(row)
    }

    #[instrument(skip(conn))]
    pub async fn get_range(
        conn: &'static Pool<Postgres>,
        max: i64,
        offset: i64,
    ) -> PgResult<(Vec<DisplayableChatter>, i64)> {
        let rows = sqlx::query_as::<_, DisplayableChatter>(
            r#"
            SELECT 
                c.id,
                c.name,
                c.login,
                c.color,
                c.image,
                c.total::text as total,
                COALESCE(
                    json_agg(
                        json_build_object(
                            'channel_id', s.channel_id,
                            'chatter_id', s.chatter_id,
                            'score', s.score,
                            'rank', s.score_rank,
                            'channel', json_build_object(
                                'id', ch.id,
                                'name', channel_chatter.name,
                                'login', channel_chatter.login,
                                'color', channel_chatter.color,
                                'image', channel_chatter.image,
                                'total_as_chatter', channel_chatter.total,
                                'total_as_broadcaster', ch.total
                            )
                        ) ORDER BY s.score_rank
                    ) FILTER (WHERE s.channel_id IS NOT NULL),
                    '[]'::json
                ) as channels
            FROM chatter c
            LEFT JOIN (
                SELECT 
                    channel_id,
                    chatter_id,
                    score,
                    created_at,
                    ROW_NUMBER() OVER (
                        PARTITION BY channel_id 
                        ORDER BY score DESC, created_at ASC
                    ) AS score_rank
                FROM score
            ) s ON c.id = s.chatter_id
            LEFT JOIN channel ch ON s.channel_id = ch.id
            LEFT JOIN chatter channel_chatter ON ch.id = channel_chatter.id
            GROUP BY c.id, c.name, c.login, c.color, c.image, c.total, c.created_at
            ORDER BY c.total DESC, c.created_at ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(max)
        .bind(offset)
        .fetch_all(conn)
        .await?;

        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS count FROM chatter
            "#
        )
        .fetch_one(conn)
        .await?;

        Ok((rows, count.unwrap_or_default()))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub id: String,
    pub total: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Channel {
    #[instrument(skip(conn))]
    pub async fn query_from_id(
        conn: &'static Pool<Postgres>,
        id: String,
    ) -> PgResult<DisplayableChannel> {
        let mut tx = conn.begin().await?;

        let query = sqlx::query_as!(
            DisplayableChannel,
            r#"
            SELECT 
                c.id, 
                c.name,
                c.login,
                c.color,
                c.image,
                b.total as total_as_broadcaster,
                c.total as total_as_chatter,
                COALESCE(
                    json_agg(
                        json_build_object(
                            'id', chatter.id,
                            'name', chatter.name,
                            'login', chatter.login,
                            'color', chatter.color,
                            'image', chatter.image,
                            'total', chatter.total::text,
                            'channels', NULL
                        ) ORDER BY s.score_rank
                    ) FILTER (WHERE s.chatter_id IS NOT NULL),
                    '[]'::json
                ) AS "chatters: sqlx::types::Json<Vec<DisplayableChatter>>"
            FROM channel b
            JOIN chatter c ON b.id = c.id
            LEFT JOIN (
                SELECT 
                    channel_id,
                    chatter_id,
                    score,
                    created_at, 
                    ROW_NUMBER() OVER (
                        PARTITION BY channel_id
                        ORDER BY score DESC, created_at ASC
                    ) AS score_rank
                FROM score
            ) s ON b.id = s.channel_id
            LEFT JOIN chatter chatter ON s.chatter_id = chatter.id
            WHERE b.id = $1
            GROUP BY c.id, c.name, c.login, c.color, c.image, c.total, b.total, b.created_at
            ORDER BY b.total DESC, b.created_at ASC
            "#,
            id,
        )
        .fetch_one(&mut *tx)
        .await?;

        Ok(query)
    }

    #[instrument(skip(conn))]
    pub async fn get_range(
        conn: &'static Pool<Postgres>,
        max: i64,
        offset: i64,
    ) -> PgResult<Vec<DisplayableChannel>> {
        let query = sqlx::query_as!(
            DisplayableChannel,
            r#"
            SELECT 
                ch.id,
                c.name,
                c.login,
                c.color,
                c.image,
                c.total as total_as_chatter,
                ch.total as total_as_broadcaster,
                COALESCE(
                    json_agg(
                        json_build_object(
                            'id', chatter.id,
                            'name', chatter.name,
                            'login', chatter.login,
                            'color', chatter.color,
                            'image', chatter.image,
                            'total', chatter.total::text,
                            'channels', (
                                SELECT COALESCE(json_agg(
                                    json_build_object(
                                        'channel_id', s2.channel_id,
                                        'chatter_id', s2.chatter_id,
                                        'score', s2.score,
                                        'rank', s2.score_rank,
                                        'channel', json_build_object(
                                            'id', ch2.id,
                                            'name', c2.name,
                                            'login', c2.login,
                                            'color', c2.color,
                                            'image', c2.image,
                                            'total_as_chatter', chatter.total,
                                            'total_as_broadcaster', ch2.total
                                        )
                                    ) ORDER BY s2.score_rank
                                ), '[]'::json)
                                FROM (
                                    SELECT 
                                        channel_id,
                                        chatter_id,
                                        score,
                                        ROW_NUMBER() OVER (
                                            PARTITION BY channel_id 
                                            ORDER BY score DESC, created_at ASC
                                        ) AS score_rank
                                    FROM score
                                    WHERE chatter_id = chatter.id
                                ) s2
                                LEFT JOIN channel ch2 ON s2.channel_id = ch2.id
                                LEFT JOIN chatter c2 ON ch2.id = c2.id
                            )
                        ) ORDER BY s.score_rank
                    ) FILTER (WHERE s.chatter_id IS NOT NULL),
                    '[]'::json
                ) as "chatters: sqlx::types::Json<Vec<DisplayableChatter>>"
            FROM channel ch
            LEFT JOIN chatter c ON ch.id = c.id
            LEFT JOIN (
                SELECT 
                    channel_id,
                    chatter_id,
                    score,
                    created_at,
                    ROW_NUMBER() OVER (
                        PARTITION BY channel_id 
                        ORDER BY score DESC, created_at ASC
                    ) AS score_rank
                FROM score
            ) s ON ch.id = s.channel_id
            LEFT JOIN chatter chatter ON s.chatter_id = chatter.id
            GROUP BY ch.id, c.name, c.login, c.color, c.image, c.total, ch.total, ch.created_at
            ORDER BY ch.total DESC, ch.created_at ASC
            LIMIT $1 OFFSET $2
            "#,
            max,
            offset,
        )
        .fetch_all(conn)
        .await?;

        Ok(query)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Broadcaster {
    pub user: Chatter,
    pub channel: Channel,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Score {
    pub channel_id: String,
    pub chatter_id: String,
    pub score: i64,
    pub rank: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Score {
    #[instrument(skip(conn, scores))]
    pub async fn update_multi(
        conn: &'static Pool<Postgres>,
        scores: HashMap<String, HashMap<String, i32>>,
    ) -> PgResult<()> {
        let mut tx = conn.begin().await?;
        for (chatter_id, val) in scores.iter() {
            for (channel_id, score) in val.iter() {
                sqlx::query!(
                    r#"
                    INSERT INTO score (chatter_id, channel_id, score)
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

    #[instrument(skip(conn, chatter_id, channel_id, score))]
    pub async fn update(
        conn: &'static Pool<Postgres>,
        chatter_id: &str,
        channel_id: &str,
        score: i64,
    ) -> PgResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO score (chatter_id, channel_id, score)
            VALUES ($1, $2, $3)
            ON CONFLICT (chatter_id, channel_id)
            DO UPDATE SET
                score = $3,
                updated_at = NOW()
            "#,
            chatter_id,
            channel_id,
            score as _,
        )
        .execute(conn)
        .await?;
        Ok(())
    }

    #[instrument(skip(conn, chatter_id, channel_id))]
    pub async fn inc_score(
        conn: &'static Pool<Postgres>,
        chatter_id: &str,
        channel_id: &str,
    ) -> PgResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO score (chatter_id, channel_id, score)
            VALUES ($1, $2, 1)
            ON CONFLICT (chatter_id, channel_id)
            DO UPDATE SET
                score = score.score + 1,
                updated_at = NOW()
            "#,
            chatter_id,
            channel_id,
        )
        .execute(conn)
        .await?;

        Ok(())
    }

    #[instrument(skip(conn, channel_id, chatter_id))]
    pub async fn get_chatter_score_on_channel(
        conn: &'static Pool<Postgres>,
        chatter_id: &str,
        channel_id: &str,
    ) -> PgResult<Self> {
        Ok(sqlx::query_as!(
            Score,
            r#"
            WITH leaderboard AS (
                SELECT 
                    *,
                    ROW_NUMBER() OVER (
                        ORDER BY score DESC, 
                        created_at ASC
                    ) AS "rank!"
                FROM score
                WHERE channel_id = $1
            )
            SELECT * FROM leaderboard
            WHERE chatter_id = $2
            "#,
            channel_id,
            chatter_id,
        )
        .fetch_one(conn)
        .await?)
    }

    #[instrument(skip(conn, channel_id))]
    pub async fn get_all_ranks_on_channel(
        conn: &'static Pool<Postgres>,
        channel_id: &str,
    ) -> PgResult<Vec<Self>> {
        Ok(sqlx::query_as!(
            Score,
            r#"
            WITH leaderboard AS (
                SELECT 
                    *,
                    ROW_NUMBER() OVER (
                        ORDER BY score DESC, 
                        created_at ASC
                    ) AS "rank!"
                FROM score
                WHERE channel_id = $1
            )
            SELECT * FROM leaderboard
            "#,
            channel_id,
        )
        .fetch_all(conn)
        .await?)
    }

    #[instrument(skip(conn, chatter_id))]
    pub async fn get_all_ranks_on_chatter(
        conn: &'static Pool<Postgres>,
        chatter_id: &str,
    ) -> PgResult<Vec<Self>> {
        Ok(sqlx::query_as!(
            Score,
            r#"
            WITH leaderboard AS (
                SELECT 
                    *,
                    ROW_NUMBER() OVER (
                        ORDER BY score DESC,
                        created_at ASC
                    ) AS "rank!"
                FROM score
                WHERE chatter_id = $1
            )
            SELECT * FROM leaderboard
            "#,
            chatter_id,
        )
        .fetch_all(conn)
        .await?)
    }
}

impl From<HelixUser> for Chatter {
    fn from(value: HelixUser) -> Self {
        Self {
            id: value.id,
            login: value.login,
            name: value.name,
            color: value.color,
            image: value.image,
            total: value.total,
            private: value.private,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl From<HelixUser> for Channel {
    fn from(value: HelixUser) -> Self {
        Self {
            id: value.id,
            total: value.total,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl From<&mut Chatter> for Channel {
    fn from(value: &mut Chatter) -> Self {
        Self {
            id: value.id.clone(),
            total: value.total,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl From<Chatter> for Channel {
    fn from(value: Chatter) -> Self {
        Self {
            id: value.id,
            total: value.total,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl Chatter {
    pub fn new(
        id: &str,
        login: &str,
        name: &str,
        color: &str,
        image: &str,
        total: i64,
        private: bool,
    ) -> Self {
        Self {
            total,
            private,
            id: id.to_string(),
            login: login.to_string(),
            name: name.to_string(),
            color: color.to_string(),
            image: image.to_string(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl Channel {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            total: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::pg::{self, db_pool};

    #[tokio::test]
    async fn test_update() {
        crate::util::tracing::build_subscriber().await.unwrap();

        let conn = pg::db_pool().await.unwrap();
        let mut broadcaster = Chatter::new(
            "000000001",
            "richard2",
            "Richard2",
            "#FFFFFE",
            "https://static-cdn.jtvnw.net/jtv_user_pictures/5a03526c-841f-47d8-a07b-9c0d0553f2df-profile_image-300x300.png",
            10,
            false,
        );

        let mut chatter = Chatter::new(
            "000000000",
            "richard",
            "richarD",
            "#FFFFFF",
            "https://static-cdn.jtvnw.net/jtv_user_pictures/5a03526c-841f-47d8-a07b-9c0d0553f2df-profile_image-300x300.png",
            999,
            false,
        );

        let mut chatter_2 = chatter.clone();
        chatter_2.id = "000000002".to_string();
        chatter_2.login = "not_richard".to_string();
        chatter_2.name = "not_richard".to_string();
        chatter_2.total = 2;

        let channel = Channel::new(&broadcaster.id);

        Chatter::inc_total(conn, &mut chatter).await.unwrap();
        Chatter::inc_total_by(conn, &mut chatter_2, 3)
            .await
            .unwrap();
        Channel::inc_total(conn, &mut broadcaster).await.unwrap();
        let score_res = Score::inc_score(conn, &chatter.id, &channel.id)
            .await
            .unwrap();

        trace!("{:#?}", score_res);

        let retrieved_chatter = Chatter::get_by_id(conn, &chatter.id).await.unwrap();
        let retrieved_channel = Channel::get_by_id(conn, &broadcaster.id).await.unwrap();

        let retrieved_score_channel =
            Score::get_chatter_score_on_channel(conn, &chatter.id, &channel.id)
                .await
                .unwrap();

        let retrieved_lb_channel = Score::get_all_ranks_on_channel(conn, &channel.id)
            .await
            .unwrap();

        let retrieved_lb_chatter = Score::get_all_ranks_on_chatter(conn, &chatter.id)
            .await
            .unwrap();

        println!("{:#?}", retrieved_chatter);
        println!("{:#?}", retrieved_channel);
        println!("{:#?}", retrieved_lb_channel);
        println!("{:#?}", retrieved_score_channel);
        println!("{:#?}", retrieved_lb_chatter);
    }

    #[tokio::test]
    async fn test_coalesce_query() {
        crate::util::tracing::build_subscriber().await.unwrap();

        let offset = 0;
        let max = 2;

        let conn = db_pool().await.unwrap();
        let res = Chatter::get_range(conn, max, offset).await.unwrap();

        assert_eq!(res.0.len(), 2);
    }
}
