use std::collections::HashMap;

use chrono::{Days, Utc};
use sqlx::{Pool, Postgres};
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};

use crate::db::models::{Chatter, DbUser};
use crate::db::pg::{PgErr, db_pool};
use crate::util::helix::{Helix, HelixErr};

#[instrument(skip(from_url))]
pub async fn update_channels(from_url: Option<&str>) -> ChannelResult<HashMap<String, Chatter>> {
    let channel_list = reqwest::get(from_url.unwrap_or(CHANNELS_LIST_URL))
        .await?
        .text()
        .await?;

    tracing::debug!(list_url = from_url, "fetching channel list");
    let ids: Vec<String> = channel_list
        .lines()
        .filter_map(|line| line.split(':').nth(1).map(|s| s.to_owned()))
        .collect();

    update_stored_channels(&ids).await
}

#[instrument(skip(chatter), fields(id = chatter.id))]
pub fn update_threshold_elapsed(chatter: &Chatter) -> bool {
    let current_ts = Utc::now().naive_utc();
    let threshold = Days::new(1);

    if let Some(update_ts) = chatter.updated_at.checked_add_days(threshold)
        && update_ts < current_ts
    {
        tracing::debug!(
            last_updated = %chatter.updated_at,
            utc_now = %current_ts,
            chatter_login = chatter.login,
            "data refresh threshold exceeded for chatter"
        );

        return true;
    }

    false
}

#[instrument(skip(ids), fields(count = ids.len()))]
pub async fn update_stored_channels(ids: &Vec<String>) -> ChannelResult<HashMap<String, Chatter>> {
    tracing::debug!("performing stored channel checks");
    let mut requires_update: Vec<String> = Vec::new();

    let conn = db_pool().await?;
    let mut existing = Chatter::get_by_ids(conn, &ids).await?;

    for id in ids {
        if !existing.iter().any(|e_br| &e_br.id == id) {
            tracing::warn!(channel = id, "uncached channel");
            requires_update.push(id.clone());
        } else if let Some(e_br) = existing.iter().find(|e_br| &e_br.id == id)
            && update_threshold_elapsed(e_br)
        {
            requires_update.push(id.clone());
        }
    }

    let channel_list = if requires_update.len() != 0 {
        tracing::info!(count = requires_update.len(), "refreshing channel data");
        get_and_refresh_chatter_data(conn, &mut existing, &mut requires_update).await?
    } else {
        existing
    };

    let mut channel_map = HashMap::new();
    channel_list.into_iter().for_each(|channel| {
        channel_map.insert(channel.login.clone(), channel);
    });

    Ok(channel_map)
}

#[instrument(skip(conn, existing, requires_update), fields(update_required_count = requires_update.len(), total_existing_count = existing.len()))]
/// Updates existing database entries with refreshed data
///
/// This function performs the retrieval of chatter data from Helix, calls the database upsert, and
/// then returns the full list of chatters to its caller
pub async fn get_and_refresh_chatter_data(
    conn: &'static Pool<Postgres>,
    existing: &mut Vec<Chatter>,
    requires_update: &mut Vec<String>,
) -> ChannelResult<Vec<Chatter>> {
    let fetched = Helix::fetch_users_by_id(requires_update)
        .await?
        .iter_mut()
        .map(|f_br| {
            if let Some(e_br) = existing.iter().find(|e| e.id == f_br.id) {
                f_br.total = e_br.total;
            }

            Chatter::from(f_br.to_owned())
        })
        .collect::<Vec<Chatter>>();

    let pre_retain = existing.len();
    existing.retain(|e_br| !fetched.iter().any(|f_br| f_br.id == e_br.id));
    existing.extend_from_slice(&fetched);

    Chatter::upsert_multi(conn, &existing).await?;

    tracing::debug!(
        refreshed_count = fetched.len(),
        old_total_count = pre_retain,
        new_total_count = existing.len(),
        "chatter data refresh complete",
    );

    Ok(existing.to_owned())
}

pub type ChannelResult<T> = core::result::Result<T, ChannelError>;

#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Helix(#[from] HelixErr),

    #[error(transparent)]
    Pg(#[from] PgErr),
}

const CHANNELS_LIST_URL: &str =
    "https://raw.githubusercontent.com/plsuwu/pea-fan/refs/heads/static/channels";

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_channel_fetch() {
        let provider = crate::util::tracing::build_subscriber().await.unwrap();
        let test_chans =
            Some("https://storage.googleapis.com/scope-shaky-majority/test-channels-20252325.txt");

        assert!(update_channels(test_chans).await.is_ok());

        crate::util::tracing::destroy_tracer(provider);
    }
}
