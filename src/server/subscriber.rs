#![allow(non_snake_case, dead_code, unused_variables)]

use super::types::ChannelChatMessagePayload;
use anyhow::anyhow;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const CALLBACK_ROUTE: &'static str = "https://api.piss.fan/webhook-global";
const API_GQL_URL: &'static str = "https://gql.twitch.tv/gql";
const API_HELIX_URL: &'static str = "https://api.twitch.tv/helix";

pub async fn subscribe(
    broadcaster_login: &str,
    user_login: &str,
) -> anyhow::Result<ChannelChatMessagePayload> {
    let broadcaster_id: String = get_user_id(broadcaster_login).await?;
    let user_id: String = get_user_id(user_login).await?;

    let subs_uri = format!("{}/eventsub/subscriptions", API_HELIX_URL);

    todo!();
}

pub async fn verify_signature() {
    todo!();
}

pub async fn get_user_id(login: &str) -> anyhow::Result<String> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "client-id",
        "kimne78kx3ncx6brgo4mv6wki5h1ko".try_into().unwrap(),
    );

    let query = ChatChannelData::new(login);
    let client = reqwest::Client::new();

    let req = client.post(API_GQL_URL).json(&query).headers(headers);
    let res = req.send().await?;
    if res.status() != 200 {
        return Err(anyhow!(format!(
            "Status of request was not 200/OK: {:#?}",
            res
        )));
    }

    let body: Value = serde_json::from_str(&res.text().await?)?;
    if let Some(broadcaster_id) = &body["data"]["channel"]["id"].as_str() {
        return Ok((*broadcaster_id).to_owned());
    } else {
        return Err(anyhow!(format!(
            "Unable to read the broadcaster_id as a string: {:#?}",
            body
        )));
    }
}

/// Serde-derivable struct representing the GQL query JSON body
#[derive(Deserialize, Serialize)]
pub struct ChatChannelData {
    pub operationName: String,
    pub variables: Variables,
    pub extensions: Extensions,
}

impl ChatChannelData {
    /// Retrieves a user id for the GQL API
    ///
    /// I'm too lazy at this point in time to set up a whole OAuth flow and I will resist doing
    /// so until I can't avoid it any longer.
    ///
    /// # [dev.twitch.tv TOS]
    ///
    /// Technically this is not TOS as we don't reverse engineering anything!
    /// Also this query doesn't require authorization so like surely this is free game please
    /// don't ban me PLEASE pleaseplaap0olesepalspalep im begging IM
    ///
    /// *gasps for air*
    ///
    /// KNEELING im clasping my hands together im looking up at you and there are tears in
    /// my eyes im just a guy IM JUSTa  a guy *moans* *sobs* *pukes*
    ///
    /// [dev.twitch.tv TOS]: https://legal.twitch.com/legal/developer-agreement/
    pub fn new(login: &str) -> Self {
        let variables = Variables {
            channelLogin: login.to_string(),
        };

        let extensions = Extensions {
            persistedQuery: PersistedQuery {
                version: 1,
                sha256Hash: "fa66abee26833eb414516b617bc3b051664e57ecc816704fce0b91344cae6ecd"
                    .to_string(),
            },
        };

        Self {
            operationName: "Chat_ChannelData".to_string(),
            variables,
            extensions,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Extensions {
    persistedQuery: PersistedQuery,
}

#[derive(Deserialize, Serialize)]
pub struct PersistedQuery {
    version: usize,
    sha256Hash: String,
}

#[derive(Deserialize, Serialize)]
pub struct Variables {
    channelLogin: String,
}
