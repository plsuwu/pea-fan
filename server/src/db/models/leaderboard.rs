use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Base score table model
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct Score {
    pub channel_id: super::channel::ChannelId,
    pub chatter_id: super::chatter::ChatterId,
    pub score: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScoreSummary {
    pub channel_id: super::channel::ChannelId,
    pub chatter_id: super::chatter::ChatterId,
    pub score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScoreEvent {
    pub id: String,
    pub chatter_id: super::chatter::ChatterId,
    pub channel_id: super::channel::ChannelId,
    pub points: i64,
    pub earned_at: NaiveDateTime,
}
