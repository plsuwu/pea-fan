#![allow(non_snake_case, dead_code)]

use super::types::{ChannelChatMessagePayload, ChannelChatMessageRequest};
use anyhow::anyhow;
use reqwest::header::HeaderMap;
use ring::{
    hmac::{self, Key},
    rand::SystemRandom,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::sync::{LazyLock, RwLock};

pub static KEY: LazyLock<RwLock<Secret>> = LazyLock::new(|| RwLock::new(Secret::new()));

#[derive(Debug)]
pub struct Secret {
    key: Key,
}

impl Secret {
    pub fn new() -> Self {
        let rng = SystemRandom::new();
        let key = Key::generate(hmac::HMAC_SHA256, &rng).unwrap();

        Self { key }
    }
}

const MOCK_API_CLIENT_ID: &'static str = "ae8f82186f9295cc0123057fd282f6";
const MOCK_API_SECRET: &'static str = "0d69582c2728f646e54b1eb112efcf";
const MOCK_API_TOKEN: &'static str = "b0f4d55ef52450a";

const CALLBACK_ROUTE: &'static str = "https://piss.fan/api/webhook-callback";
const API_GQL_URL: &'static str = "https://gql.twitch.tv/gql";
const API_HELIX_URL: &'static str = "https://api.twitch.tv/helix";

const HMAC_PREFIX: &'static str = "sha256=";
const TWITCH_MESSAGE_ID: &'static str = "Twitch-Eventsub-Message-Id";
const TWITCH_MESSAGE_TIMESTAMP: &'static str = "Twitch-Eventsub-Message-Timestamp";
const TWITCH_MESSAGE_SIGNATURE: &'static str = "Twitch-Eventsub-Message-Signature";
const MESSAGE_TYPE: &'static str = "Twitch-Eventsub-Message-Type";

pub async fn subscribe(
    broadcaster_login: &str,
    user_login: &str,
) -> anyhow::Result<ChannelChatMessagePayload> {
    // retrieves user id for broadcaster and user from GQL API
    //
    // technically this is not TOS as we don't reverse engineering anything!
    // also this query doesn't require authorization so like please don't ban me PLEASE
    // pleaseplaap[lesepalspalep im begging IM 
    // *gasps for air*
    // KNEELING im clasping my hands together im looking up at you and there are tears in
    // my eyes im just a guy IM JUSTa  a guy *moans* *sobs* *pukes*
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

#[derive(Deserialize, Serialize)]
pub struct ChatChannelData {
    pub operationName: String,
    pub variables: Variables,
    pub extensions: Extensions,
}

impl ChatChannelData {
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
