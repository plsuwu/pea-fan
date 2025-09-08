use std::collections::HashMap;

use chrono::Local;
use thiserror::Error;
use tracing::{debug, info, instrument};

use crate::database;
use crate::database::schema::{Channel, Chatter};
use crate::util::helix::{Helix, HelixError};

const CHANNELS_LIST: &str =
    "https://raw.githubusercontent.com/plsuwu/pea-fan/refs/heads/static/channels";

#[derive(Error, Debug)]
pub enum ChannelUtilError {
    #[error("helix fetch error: {0}")]
    HelixFetchError(#[from] HelixError),

    #[error("postgres error: {0}")]
    PostgresError(#[from] database::postgres::PostgresError),

    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

#[instrument]
pub async fn get_tracked_channels() -> Result<HashMap<String, String>, ChannelUtilError> {
    let channel_list = reqwest::get(CHANNELS_LIST)
        .await?
        .text()
        .await?
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();

    info!(
        "Current channel list ({} items): {:#?}",
        channel_list.len(),
        channel_list
    );

    let result = insert_new(&channel_list).await?;

    Ok(result)
}

#[instrument(skip(channel_logins))]
pub async fn insert_new(
    channel_logins: &Vec<String>,
) -> Result<HashMap<String, String>, ChannelUtilError> {
    let existing = Channel::get_existing_ids_by_login(&channel_logins).await?;

    let mut needs_fetch = channel_logins
        .iter()
        .filter(|login| !existing.iter().any(|chan| &&chan.login == login))
        .map(|login| login.to_string())
        .collect::<Vec<String>>();

    debug!("needs: {:?}", needs_fetch);

    if needs_fetch.len() > 0 {
        let broadcasters = Helix::fetch_user_by_login(&mut needs_fetch).await?;
        let channels: Vec<Channel> = broadcasters
            .iter()
            .map(|br| {
                let localtime_now = Local::now();
                Channel {
                    id: br.id.clone(),
                    total: 0,
                    created_at: localtime_now.naive_local(),
                    updated_at: localtime_now.naive_local(),
                }
            })
            .collect();

        debug!("inserting: {:?}", channels);

        Channel::bulk_upsert(&channels).await?;
        Chatter::bulk_upsert(&broadcasters).await?;
    }

    let mut result = HashMap::new();
    existing.iter().for_each(|chan| {
        result.insert(chan.login.clone(), chan.id.clone()).unwrap();
    });

    Ok(result)
}
