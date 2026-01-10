use core::fmt;

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    db::models::chatter::{ChatterId, ChatterScoreSummary},
    util::helix::HelixUser,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct ChannelId(pub String);

/// Base channel table model
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Channel {
    pub id: ChannelId,
    pub channel_total: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelLeaderboardEntry {
    pub id: ChannelId,
    pub name: String,
    pub login: String,
    pub color: String,
    pub image: String,
    pub total_chatter: i64,
    pub total_channel: i64,
    pub ranking: i64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub chatter_scores: Vec<super::chatter::ChatterScoreSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelScoreSummary {
    pub chatter_id: ChatterId,
    pub channel_id: ChannelId,
    pub channel_name: String,
    pub channel_login: String,
    pub channel_color: String,
    pub channel_image: String,
    pub score: i64,
    pub ranking: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelLeaderboardRow {
    pub id: ChannelId,
    pub name: String,
    pub login: String,
    pub color: String,
    pub image: String,
    pub total_chatter: i64,
    pub total_channel: i64,
    pub ranking: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl ChannelLeaderboardRow {
    pub fn into_leaderboard_entry(
        self,
        chatter_scores: Vec<ChatterScoreSummary>,
    ) -> ChannelLeaderboardEntry {
        ChannelLeaderboardEntry {
            id: self.id.into(),
            login: self.login,
            name: self.name,
            color: self.color,
            image: self.image,
            ranking: self.ranking,
            total_channel: self.total_channel,
            total_chatter: self.total_chatter,
            chatter_scores,
        }
    }
}

impl From<String> for ChannelId {
    fn from(value: String) -> Self {
        ChannelId(value)
    }
}

impl From<&str> for ChannelId {
    fn from(value: &str) -> Self {
        ChannelId(value.to_string())
    }
}

impl From<super::chatter::ChatterId> for ChannelId {
    fn from(value: super::chatter::ChatterId) -> Self {
        ChannelId(value.0)
    }
}

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<HelixUser> for Channel {
    fn from(value: HelixUser) -> Self {
        Self {
            id: value.id.into(),
            channel_total: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl From<super::chatter::Chatter> for Channel {
    fn from(value: super::chatter::Chatter) -> Self {
        Self {
            id: value.id.into(),
            channel_total: 0,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

// impl From<Vec<super::chatter::Chatter>> for Channel {
//     fn from(values: Vec<super::chatter::Chatter>) -> Vec<Self> {
//         values.into_iter().map(Self::from).collect()
//     }
// }
