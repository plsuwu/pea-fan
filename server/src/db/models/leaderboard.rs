use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

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
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TimeWindow {
    Yesterday,
    PrevWeek,
    PrevMonth,
    PrevYear,
    Last7Days,
    Last30Days,
}

impl TimeWindow {
    pub fn as_date_trunc(&self) -> &'static str {
        match self {
            TimeWindow::Yesterday => {
                r#"date_trunc('day', CURRENT_TIMESTAMP - interval '1 day')
                AND earned_at < date_trunc('day', CURRENT_TIMESTAMP)"#
            }
            TimeWindow::PrevWeek => {
                r#"date_trunc('week', CURRENT_TIMESTAMP - interval '1 week') 
                AND earned_at < date_trunc('week', CURRENT_TIMESTAMP)"#
            }
            TimeWindow::PrevMonth => {
                r#"date_trunc('month', CURRENT_TIMESTAMP - interval '1 month')
                AND earned_at < date_trunc('month', CURRENT_TIMESTAMP)"#
            }
            TimeWindow::PrevYear => {
                r#"date_trunc('year', CURRENT_TIMESTAMP - interval '1 year') 
                AND earned_at < date_trunc('year', CURRENT_TIMESTAMP)"#
            }
            TimeWindow::Last7Days => "CURRENT_TIMESTAMP - interval '7 days'",
            TimeWindow::Last30Days => "CURRENT_TIMESTAMP - interval '30 days'",
        }
    }

    /// `user_type` should be either "chatter" or "channel", I just can't be bothered fixing its
    /// type safety at present.
    ///
    /// # Todo
    ///
    /// TODO fix `user_type` typing smile
    pub fn into_query(&self, user_type: &str) -> String {
        format!(
            r#"
            SELECT COALESCE(COUNT(*), 0) FROM score_event
            WHERE {}_id = $1
            AND earned_at >= {}
            "#,
            user_type,
            self.as_date_trunc()
        )
    }
}
