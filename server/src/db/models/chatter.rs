use core::fmt;

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    db::{
        models::channel::{ChannelId, ChannelScoreSummary},
        repositories::leaderboard::ScorePagination,
    },
    util::helix::HelixUser,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct ChatterId(pub String);

/// Base chatter table model
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Chatter {
    pub id: ChatterId,
    pub login: String,
    pub name: String,
    pub color: String,
    pub image: String,
    pub total: i64,
    pub private: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatterLeaderboardEntry {
    pub id: ChatterId,
    pub login: String,
    pub name: String,
    pub color: String,
    pub image: String,
    pub total: i64,
    pub ranking: i64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub channel_scores: Vec<super::channel::ChannelScoreSummary>,
    pub total_scores: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatterScoreSummary {
    pub channel_id: ChannelId,
    pub chatter_id: ChatterId,
    pub chatter_login: String,
    pub chatter_name: String,
    pub chatter_color: String,
    pub chatter_image: String,
    pub score: i64,
    pub ranking: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChatterLeaderboardRow {
    pub id: ChatterId,
    pub login: String,
    pub name: String,
    pub color: String,
    pub image: String,
    pub total: i64,
    pub private: bool,
    pub ranking: i64,
    pub total_scores: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl ChatterLeaderboardRow {
    pub fn into_leaderboard_entry(
        self,
        channel_scores: Vec<ChannelScoreSummary>,
    ) -> ChatterLeaderboardEntry {
        ChatterLeaderboardEntry {
            id: self.id,
            login: self.login,
            name: self.name,
            color: self.color,
            image: self.image,
            total: self.total,
            ranking: self.ranking,
            total_scores: self.total_scores,
            channel_scores,
        }
    }
}

impl From<String> for ChatterId {
    fn from(value: String) -> Self {
        ChatterId(value)
    }
}

impl From<&str> for ChatterId {
    fn from(value: &str) -> Self {
        ChatterId(value.to_string())
    }
}

impl From<super::channel::ChannelId> for ChatterId {
    fn from(value: super::channel::ChannelId) -> Self {
        ChatterId(value.0)
    }
}

impl From<HelixUser> for Chatter {
    fn from(value: HelixUser) -> Self {
        Self {
            id: value.id.into(),
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

impl fmt::Display for ChatterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
