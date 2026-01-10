use core::fmt;

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

// use crate::{db::models_new::{channel, chatter}, util::helix::HelixUser};


// --
// Relational models
// --

/// Score with computed rank to be used in queries with window functions
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RankedScore {
    pub channel_id: ChannelId,
    pub chatter_id: ChatterId,
    pub score: i64,
    pub ranking: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// --
// API response models
// ---

/// API response model for chatter with list of chatter-channel score relations
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatterResponse {
    pub id: String,
    pub name: String,
    pub login: String,
    pub color: String,
    pub image: String,
    pub total: i64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scores: Vec<ScoreResponse>,
}

/// API response model for channel with list of channel-chatter score relations
#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelResponse {
    pub id: String,
    pub name: String,
    pub login: String,
    pub color: String,
    pub image: String,
    pub total_chatter: i64,
    pub total_channel: i64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scores: Vec<ScoreResponse>,
}

/// API response model for the channel-chatter-score relation
#[derive(Debug, Serialize, Deserialize)]
pub struct ScoreResponse {
    pub channel: ChannelSummary,
    pub chatter: ChatterSummary,
    pub score: i64,
    pub ranking: i64,
}

/// API response model for a single-level channel item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSummary {
    pub id: String,
    pub name: String,
    pub login: String,
    pub color: String,
    pub image: String,
    pub total_chatter: i64,
    pub total_channel: i64,
}

/// API response model for a single-level chatter item
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ChatterSummary {
    pub id: String,
    pub name: String,
    pub login: String,
    pub color: String,
    pub image: String,
    pub total: i64,
}

// --
// trait impls
// --

impl From<Chatter> for ChatterSummary {
    fn from(value: Chatter) -> Self {
        Self {
            id: value.id.0,
            name: value.name,
            login: value.login,
            color: value.color,
            image: value.image,
            total: value.total,
        }
    }
}

impl From<ChatterInfo> for ChatterSummary {
    fn from(value: ChatterInfo) -> Self {
        Self {
            id: value.id.0,
            name: value.name,
            login: value.login,
            color: value.color,
            image: value.image,
            total: value.total,
        }
    }
}

impl From<Chatter> for ChatterResponse {
    fn from(value: Chatter) -> Self {
        Self {
            id: value.id.0,
            name: value.name,
            login: value.login,
            color: value.color,
            image: value.image,
            total: value.total,
            scores: Vec::new(),
        }
    }
}


impl From<ChannelInfo> for ChannelSummary {
    fn from(value: ChannelInfo) -> Self {
        Self {
            id: value.id.0,
            name: value.name,
            login: value.login,
            color: value.color,
            image: value.image,
            total_chatter: value.total_chatter,
            total_channel: value.total_channel,
        }
    }
}

impl From<HelixUser> for chatter::Chatter {
    fn from(value: HelixUser) -> Self {
        Self {
            id: chatter::ChatterId(value.id),
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

impl From<HelixUser> for channel::Channel {
    fn from(value: HelixUser) -> Self {
        Self {
            id: channel::ChannelId(value.id),
            channel_total: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}
