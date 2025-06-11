#![allow(non_snake_case, dead_code, unused_variables)]

use super::types::{StreamGenericRequest, StreamGenericRequestType, SubscriptionGenericResponse};
use crate::server::KEY_DIGEST;
use crate::socket::client::get_current_time;
use anyhow::anyhow;
use reqwest::Client;
use reqwest::header::{AUTHORIZATION, HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "production")]
const CALLBACK_ROUTE: &'static str = "https://api.piss.fan/webhook-global";

#[cfg(not(feature = "production"))]
const CALLBACK_ROUTE: &'static str = "https://pls.ngrok.io/webhook-global";

const API_GQL_URL: &'static str = "https://gql.twitch.tv/gql";
const API_HELIX_URL: &'static str = "https://api.twitch.tv/helix";
const BROWSER_CLIENT_ID: &'static str = "kimne78kx3ncx6brgo4mv6wki5h1ko";
const TESTING_CLIENT_ID: &'static str = "7jz14ixoeglm6aq8eott8196p4g5ox";

/// Subcribes to the required stream webhook events
///
/// Makes a subscribe request to the twitch API for both `stream.online` and `stream.offline` events
/// for a given broadcaster `broadcaster_login`.
pub async fn sub_stream_event_multi(broadcaster_login: &str, token: &str) -> anyhow::Result<()> {
    // Current server session's secret key instance
    //
    // This should be constant for the lifetime of the server listener and changes
    // on application restart
    let key = (&*KEY_DIGEST).read().unwrap()._hex.clone();
    let broadcaster_user_id: String = get_user_id(broadcaster_login).await?;

    // `stream.online` subscription
    subscribe_stream_event(
        &broadcaster_user_id,
        token,
        StreamGenericRequestType::Online,
        &key,
    )
    .await?;

    // `stream.offline` subscription
    subscribe_stream_event(
        &broadcaster_user_id,
        token,
        StreamGenericRequestType::Offline,
        &key,
    )
    .await?;

    Ok(())
}

/// Subscribes to a single (supported) stream event instance
///
/// # Stream events
///
/// The `StreamGenericRequestType` enum describes the webhook `type` field to request
/// notifications for.
///
/// This will one of:
/// - `StreamGenericRequestType::Online` (`stream.online`),
/// - `StreamGenericRequestType::Offline` (`stream.offline`),
pub async fn subscribe_stream_event(
    broadcaster_user_id: &str,
    token: &str,
    notify_type: StreamGenericRequestType,
    key: &str,
) -> anyhow::Result<SubscriptionGenericResponse> {
    let client = reqwest::Client::new();
    let subs_uri = format!("{}/eventsub/subscriptions", API_HELIX_URL);
    let headers = build_headers(token)?;

    let request_body =
        StreamGenericRequest::new(&broadcaster_user_id, &CALLBACK_ROUTE, key, notify_type);

    // this was split into two because its easier to debug but realistically we could combine this
    // into a single let binding
    let req = client.post(subs_uri).json(&request_body).headers(headers);
    let res = req.send().await?;

    if res.status() != 200 && res.status() != 202 {
        match res.status() {
            // StatusCode::CONFLICT => {
            //     // revoke and retry ??
            // },
            _ => {
                let err: Value = serde_json::from_str(&res.text().await?)?;
                Err(anyhow!(format!(
                    "Status of request (`stream.online/.offline`) not 200 | OK: {:#?}",
                    err
                )))
            }
        }
    } else {
        // return the successfully retrieved information
        let unserialized_body: Value = serde_json::from_str(&res.text().await?)?;

        let status = &unserialized_body["data"][0]["status"].as_str().unwrap();
        let subscription_type = &unserialized_body["data"][0]["type"].as_str().unwrap();
        let broadcaster_id = &unserialized_body["data"][0]["condition"]["broadcaster_user_id"]
            .as_str()
            .unwrap();

        println!(
            "[{}] recv new: STATUS '{}' -> {} (for uid '{}')",
            get_current_time(),
            status,
            subscription_type,
            broadcaster_id
        );

        Ok(serde_json::from_value(unserialized_body)?)
    }
}

pub async fn delete_subscription_multi(subscription_id: &str, token: &str) -> anyhow::Result<()> {
    let headers = build_headers(token)?;
    let client = Client::new();
    let subs_uri = format!(
        "{}/eventsub/subscriptions?id={}",
        API_HELIX_URL, subscription_id
    );

    let res = client.delete(subs_uri).headers(headers).send().await;
    match res {
        Ok(r) => println!("[+] subscription '{}' deleted ok", subscription_id),
        Err(e) => eprintln!("[x] error during subscription deletion: {:?}", e),
    }

    Ok(())
}

pub async fn get_active_hooks(token: &str) -> Option<Vec<Value>> {
    let client = reqwest::Client::new();
    let subs_uri = format!("{}/eventsub/subscriptions?status=enabled", API_HELIX_URL);

    let headers = build_headers(token).unwrap();

    let req = client.get(subs_uri).headers(headers);
    let res = req.send().await.unwrap();

    let mut deserialized: Value = serde_json::from_str(&res.text().await.unwrap()).unwrap();
    if let Some(active_count) = deserialized["total"].take().as_u64() {
        let maybe_data: Result<Vec<Value>, serde_json::Error> =
            serde_json::from_value(deserialized["data"].clone());
        if let Ok(data_array) = maybe_data {
            return Some(data_array);
        }
    }

    None
}

// :((
// pub async fn subscribe_chat_messages(
//     broadcaster_login: &str,
//     user_login: &str,
//     token: &str,
// ) -> anyhow::Result<SubscriptionGenericResponse> {
//     let key_lock = (&*KEY_DIGEST).read().unwrap()._hex.clone();
//
//     let broadcaster_id: String = get_user_id(broadcaster_login).await?;
//     let user_id: String = get_user_id(user_login).await?;
//     let request_chat =
//         ChannelChatMessageRequest::new(&broadcaster_id, &user_id, CALLBACK_ROUTE, &key_lock);
//
//     println!("req_body: {:#?}", serde_json::to_string(&request_chat));
//
//     let headers = build_headers(token)?;
//
//     let subs_uri = format!("{}/eventsub/subscriptions", API_HELIX_URL);
//     let client = reqwest::Client::new();
//     let req = client.post(subs_uri).json(&request_chat).headers(headers);
//     println!("req: {:#?}", req);
//
//     let res = req.send().await?;
//     if res.status() != 200 {
//         let err: Value = serde_json::from_str(&res.text().await?)?;
//         return Err(anyhow!(format!(
//             "Status of request (subscription) was not 200/OK: {:#?}",
//             err
//         )));
//     }
//
//     let pre_conv: Value = serde_json::from_str(&res.text().await?)?;
//     println!("{:#?}", pre_conv);
//
//     let body: SubscriptionGenericResponse = serde_json::from_value(pre_conv)?;
//
//     Ok(body)
// }

// async fn get_app_token() -> anyhow::Result<String> {
//
// }

// pub async fn verify_signature() {
//     todo!();
// }

// let broadcaster_login = get_user_data(token, broadcaster_id).await?.login;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamsQueryData {
    pub id: String,
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub game_id: String,
    pub game_name: String,
    pub r#type: String,
    pub title: String,
    pub viewer_count: usize,
    pub started_at: String,
    pub language: String,
    pub thumbnail_url: String,

    /// Deprecated field, will always be empty
    pub tag_ids: Vec<String>,
    pub tags: Vec<String>,
    pub is_mature: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaginationData {
    pub cursor: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamsQueryResponse {
    pub data: Vec<StreamsQueryData>,
    pub pagination: PaginationData,
}

pub async fn stream_online(token: &str, broadcaster_id: &str) -> anyhow::Result<bool> {
    let client = reqwest::Client::new();
    let headers = build_headers(token)?;
    let uri = format!("{}/streams?user_id={}", API_HELIX_URL, broadcaster_id);

    let res = client.get(uri).headers(headers).send().await?;
    if res.status() != 200 {
        return Err(anyhow!(format!(
            "Status of request was not 200/OK: {:#?}",
            res
        )));
    }

    let maybe_data: serde_json::Result<StreamsQueryResponse> =
        serde_json::from_str(&(res.text().await?));

    match maybe_data {
        Ok(data) => {
            if data.data.len() > 0 && data.data[0].r#type == "live" {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(e) => {
            eprintln!(
                "[x] failed to parse streams query response for '{}': {:?}",
                broadcaster_id, e
            );
            Err(anyhow!(e))
        }
    }
}

fn build_headers(token: &str) -> anyhow::Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert("client-id", TESTING_CLIENT_ID.try_into().unwrap());
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", token).try_into().unwrap(),
    );

    Ok(headers)
}

pub async fn get_user_id(login: &str) -> anyhow::Result<String> {
    let mut headers = HeaderMap::new();
    headers.insert("client-id", BROWSER_CLIENT_ID.try_into().unwrap());

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

pub async fn get_user_data(token: &str, user_id: &str) -> anyhow::Result<UsersQueryData> {
    let headers = build_headers(token)?;
    let uri = format!("{}/users?id={}", API_HELIX_URL, user_id);

    let client = reqwest::Client::new();
    let res = client.get(uri).headers(headers).send().await?;

    if res.status() != 200 {
        return Err(anyhow!(format!(
            "Status of request was not 200/OK: {:#?}",
            res
        )));
    }

    let body: HelixUsersQuery = serde_json::from_str(&res.text().await?)?;
    Ok(body.data[0].clone())
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HelixUsersQuery {
    pub data: Vec<UsersQueryData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UsersQueryData {
    pub id: String,
    pub login: String,
    pub display_name: String,
    pub r#type: String,
    pub broadcaster_type: String,
    pub description: String,
    pub profile_image_url: String,
    pub offline_image_url: String,
    pub view_count: usize,
    pub created_at: String,
}

/// Serde-derivable struct representing the GQL query JSON body
#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Extensions {
    persistedQuery: PersistedQuery,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PersistedQuery {
    version: usize,
    sha256Hash: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Variables {
    channelLogin: String,
}
