use serde::{Deserialize, Serialize};

use crate::db::repositories::leaderboard::ScorePagination;

pub mod channel;
pub mod chatter;
pub mod leaderboard;

#[inline]
const fn default_offset() -> i64 {
    0
}

#[inline]
const fn default_limit() -> i64 {
    50
}

#[inline]
fn default_score_pagination() -> ScorePagination {
    ScorePagination::new(50, 0)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub score_limit: i64,
    #[serde(default = "default_offset")]
    pub score_page: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub page: i64,
    pub total_items: i64,
    pub total_pages: i64,
    #[serde(default = "default_limit")]
    pub page_size: i64,
    
    // #[serde(default = "default_offset")]
    // pub chatter_offset: i64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total_items: i64, page_size: i64, page: i64) -> Self {
        let total_pages = (total_items as f64 / page_size as f64).ceil() as i64;
        Self {
            items,
            page,
            page_size,
            total_items,
            total_pages,
        }
    }
}

pub mod prelude {}
