use core::fmt;
use std::sync::LazyLock;

use futures::{StreamExt, stream};
use http::header::{AUTHORIZATION, InvalidHeaderValue};
use http::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::OnceCell;
use tracing::{Instrument, debug, error, info, instrument, trace, warn};

use crate::util::env::{EnvErr, Var};
use crate::var;

// TODO: these two consts are for redis-related processing and should be instantiated in the
// correct module :))
pub const NOT_PRESENT_IN_CACHE: &str = "[NOT_PRESENT_IN_CACHE]";
pub const NOT_VALID_HELIX_USER: &str = "[NOT_VALID_HELIX_USER]";

pub struct Helix;
impl Helix {
    #[instrument(skip(users), fields(user_count = users.len()))]
    /// Fetch a list of users' Twitch information via their IDs.
    ///
    /// # Reliability
    ///
    /// This is the preferred method for fetching a user's information, as an account's ID cannot
    /// be changed.
    pub async fn fetch_users_by_id(users: &mut Vec<String>) -> HelixResult<Vec<HelixUser>> {
        let mut retrieved = Vec::new();
        let uri_params = build_query_params(HelixParamType::Id, users);

        for param in uri_params {
            let uri = format!("{}{}", String::from(HelixUri::Users), param);
            Self::fetch_users::<HelixDataResponse<HelixUser>>(uri)
                .await?
                .data
                .into_iter()
                .for_each(|user| retrieved.push(user));
        }

        tracing::debug!(fetched_count = retrieved.len(), "fetched primary user data");

        Self::fetch_auxilliary_data(&mut retrieved).await
    }

    #[instrument(skip(users), fields(user_count = users.len()))]
    /// Fetch a list of users' Twitch information via their logins.
    ///
    /// # Reliability
    ///
    /// Fetching via a login is far more fragile due to the impermanence of Twitch usernames. As such,
    /// `Helix::fetch_users_by_id` is preferred over this function.
    pub async fn fetch_users_by_login(mut users: Vec<String>) -> HelixResult<Vec<HelixUser>> {
        let mut retrieved = Vec::new();
        let uri_params = build_query_params(HelixParamType::Login, &mut users);

        for (i, param) in uri_params.iter().enumerate() {
            let uri_users = format!("{}{}", String::from(HelixUri::Users), param);
            let user_queries = match Self::fetch_users::<HelixDataResponse<HelixUser>>(uri_users)
                .await
            {
                Ok(d) => d,
                // if a username is found to be invalid, we find which request actually went
                // wrong and attempt to refetch users in that chunk one-by-one, skipping over
                // the erroneous users
                Err(HelixErr::InvalidUsername) => {
                    let users_chunk = users
                        .get(i * 100..)
                        .and_then(|rest| rest.get(..100))
                        .unwrap_or_default();

                    tracing::error!(
                        iterator_idx = i,
                        from_idx = i * 100,
                        to_idx = users_chunk.len() + (i * 100),
                        chunk_length = users_chunk.len(),
                        "found bad chunk"
                    );
                    tracing::trace!(users_in_chunk = ?users);
                    Self::try_refetch(users_chunk.to_owned(), HelixParamType::Login).await?
                }

                Err(e) => {
                    tracing::error!(error = ?e, "unhandled error response during helix request");
                    continue;
                }
            };

            retrieved.extend(user_queries.data);
        }

        tracing::debug!(
            retrieved_len = retrieved.len(),
            "user fetch by login complete"
        );

        Self::fetch_auxilliary_data(&mut retrieved).await
    }

    #[instrument]
    /// Sends off a request to a given URI using a `reqwest` client
    async fn send(uri: String) -> HelixResult<reqwest::Response> {
        let client = reqwest::Client::new();
        let headers = auth_headers().await?.bearer.clone();

        client
            .get(uri)
            .headers(headers)
            .send()
            .await
            .map_err(|e| HelixErr::ReqwestError(e))
    }

    #[instrument(skip(uri))]
    /// Performs a GET request to a given URI and parses the response according to the specified
    /// `T` output type
    async fn fetch_users<T>(uri: String) -> HelixResult<T>
    where
        T: DeserializeOwned + fmt::Debug,
    {
        let res = Self::send(uri).await?;

        // if the request was unsuccessful, check to see whether the response
        // contained extra details about the error and return the corresponding
        // detail available
        if res.status() != 200 {
            let status_code = res.status();
            tracing::error!(code = %status_code, "non-200/OK response");
            if let Ok(reason) = res.json::<Value>().await {
                tracing::error!(body = ?reason, "error message in response");
                let reason_clone = reason["message"].clone();

                // check if the error reason was due to an invalid username in the query, which we
                // handle specifically
                let reason_str = reason_clone.as_str().ok_or(HelixErr::FetchErrWithBody {
                    body: reason.clone(),
                })?;

                // perhaps also a specific handler for `401: Unauthorized`-type errors as this is
                // due to something like expired app/user tokens

                return Err(match reason_str.starts_with("Invalid username") {
                    true => HelixErr::InvalidUsername,
                    false => HelixErr::FetchErrWithBody { body: reason },
                });
            } else {
                // if no extra detail available with error, just return with status code as an
                // error
                return Err(HelixErr::FetchErr(status_code.to_string()));
            }
        }

        // log rate limit
        let rl_remaining = res.headers().get("ratelimit-remaining");
        let rl_total = res.headers().get("ratelimit-limit");

        if let Some(remaining) = rl_remaining
            && let Some(total) = rl_total
        {
            tracing::info!(ratelimit_available = ?remaining, ratelimit_total = ?total, "rate-limit bucket");
            // ... implement some kind of backoff if we start to saturate this limit
        }

        let res_body = res.json::<T>().await.map_err(|e| HelixErr::ReqwestError(e));
        res_body
    }

    #[instrument(skip(users, param_type))]
    pub async fn try_refetch(
        users: Vec<String>,
        param_type: HelixParamType,
    ) -> HelixResult<HelixDataResponse<HelixUser>> {
        let requests = {
            let _span = tracing::debug_span!("build_requests").entered();
            users.into_iter().map(|user| {
                //
                // perform a copy of the param type here prior to move later
                let param_type = param_type;
                async move {
                    let params = build_query_params(param_type, &mut vec![user.to_string()]);
                    let uri = format!("{}{}", String::from(HelixUri::Users), params[0]);
                    match Self::fetch_users::<HelixDataResponse<HelixUser>>(uri).await {
                        Ok(r) => {
                            // NOTE:
                            //   helix would return a 200 status response during testing where it would
                            //   provide a valid-LOOKING `data` array, but the array was (unintuitively)
                            //   completely empty.
                            //
                            //   this appears to be due to querying for a user's `display_name` rather
                            //   than their `login`, but this is still probably a good sanity check to
                            //   have.
                            if r.data.len() > 0 {
                                return Ok((r.data, user));
                            }

                            tracing::warn!(
                                user,
                                data_length = r.data.len(),
                                "got 200/OK response containing empty data field"
                            );
                            Err((HelixErr::EmptyDataField, user))
                        }
                        Err(e) => Err((e, user)),
                    }
                }
            })
        };

        tracing::debug!(requests_count = requests.len(), "built requests vec");
        let mut refetched = Vec::new();

        // spawn threads to process user entries concurrently
        let results: Vec<_> = stream::iter(requests)
            .buffer_unordered(NUM_WORKER_THREADS)
            .collect()
            .instrument(tracing::debug_span!("resolve_and_unwrap_futures"))
            .await;

        tracing::debug!(results_count = results.len(), "checking request results");
        for result in results {
            match result {
                Ok((res, _)) => refetched.push(res[0].clone()),
                Err((e, user)) => {
                    tracing::error!(user, error = ?e, "invalid user, manual fix required");
                }
            }
        }

        tracing::debug!(total_refetched = refetched.len(), "refetch complete");
        Ok(HelixDataResponse { data: refetched })
    }

    #[instrument(skip(users), fields(users_count = users.len()))]
    async fn fetch_auxilliary_data(users: &mut Vec<HelixUser>) -> HelixResult<Vec<HelixUser>> {
        let mut colors = Self::fetch_colors(&users).await?;

        // ... space for future aux data fetching function calls :3

        {
            let _span = tracing::debug_span!("sort_vecs").entered();
            users.sort();
            colors.sort();
        }

        tracing::debug!("sorted auxilliary data vectors");
        {
            let _span = tracing::debug_span!("merge_vecs").entered();
            users.iter_mut().enumerate().for_each(|(idx, user)| {
                if !colors[idx].color.is_empty() {
                    user.color = colors[idx].color.to_string();
                }
            });
        }

        tracing::debug!(merged_count = users.len(), "merged auxilliary data vectors");
        Ok(users.to_owned())
    }

    #[instrument(skip(users), fields(users_count = users.len()))]
    pub async fn fetch_colors(users: &Vec<HelixUser>) -> HelixResult<Vec<HelixColor>> {
        let mut retrieved = Vec::new();
        let ids = users.iter().map(|user| user.id.clone()).collect::<Vec<_>>();

        let params = build_query_params(HelixParamType::UserId, &ids);

        for param in params {
            let uri = format!("{}{}", String::from(HelixUri::Colors), param);
            let queries = Self::fetch_users::<HelixDataResponse<HelixColor>>(uri).await?;
            retrieved.extend(queries.data.into_iter());
        }

        tracing::debug!(color_count = retrieved.len(), "fetched colors for users");
        Ok(retrieved)
    }
}

pub const HELIX_URI_BASE: &str = "https://api.twitch.tv/helix";
pub const HELIX_URN_USERS: &str = "users";
pub const HELIX_URN_STREAMS: &str = "streams";
pub const HELIX_URN_COLORS: &str = "chat/color";
const NUM_WORKER_THREADS: usize = 25;

#[derive(Debug)]
pub enum HelixUri {
    Users,
    Streams,
    Colors,
}

#[derive(Debug, Clone, Copy)]
pub enum HelixParamType {
    UserLogin,
    UserId,
    Login,
    Id,
}

impl From<HelixUri> for String {
    fn from(value: HelixUri) -> Self {
        format!(
            "{}/{}",
            HELIX_URI_BASE,
            match value {
                HelixUri::Users => HELIX_URN_USERS,
                HelixUri::Streams => HELIX_URN_STREAMS,
                HelixUri::Colors => HELIX_URN_COLORS,
            }
        )
    }
}

impl From<HelixParamType> for String {
    fn from(value: HelixParamType) -> Self {
        match value {
            HelixParamType::UserLogin => "user_login=".to_string(),
            HelixParamType::UserId => "user_id=".to_string(),
            HelixParamType::Login => "login=".to_string(),
            HelixParamType::Id => "id=".to_string(),
        }
    }
}

#[instrument(skip(items), fields(item_count = items.len()))]
pub fn build_query_params(param_type: HelixParamType, items: &[String]) -> Vec<String> {
    let queries: Vec<_> = items
        .chunks(100)
        .map(|chunk| {
            let mut query = format!(
                "?{}{}",
                String::from(param_type.clone()),
                chunk[0].to_lowercase()
            );
            for val in &chunk[1..] {
                query.push_str(&format!(
                    "&{}{}",
                    String::from(param_type.clone()),
                    val.to_string().to_lowercase()
                ));
            }

            query
        })
        .collect();

    queries
}

#[derive(Debug, Clone, Deserialize)]
pub struct HelixDataResponse<T> {
    data: Vec<T>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct HelixUser {
    pub id: String,
    pub login: String,

    #[serde(rename = "display_name")]
    pub name: String,
    #[serde(rename = "profile_image_url")]
    pub image: String,
    #[serde(default = "get_default_color")]
    pub color: String,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub private: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HelixStream {
    pub title: String,
    pub tags: Vec<String>,
    #[serde(rename = "user_id")]
    pub id: String,
    #[serde(rename = "user_login")]
    pub login: String,
    #[serde(rename = "thumbnail_url")]
    pub thumnail: String,
    #[serde(rename = "game_name")]
    pub game: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HelixColor {
    color: String,

    #[serde(rename = "user_id")]
    id: String,
}

pub trait CommonUser {
    fn id(&self) -> &str;
}

#[macro_export]
macro_rules! impl_common_user {
    ($struct:ty, $id:ident) => {
        impl CommonUser for $struct {
            fn id(&self) -> &str {
                &self.$id
            }
        }

        impl PartialEq for $struct {
            fn eq(&self, other: &Self) -> bool {
                self.$id == other.$id
            }
        }

        impl Eq for $struct {}

        impl PartialOrd for $struct {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.$id.cmp(&other.$id))
            }
        }

        impl Ord for $struct {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.$id
                    .len()
                    .cmp(&other.$id.len())
                    .then(self.$id.cmp(&other.$id))
            }
        }
    };
}

impl_common_user!(HelixColor, id);
impl_common_user!(HelixUser, id);

#[inline]
fn get_default_color() -> String {
    String::from("#000000")
}

pub struct AuthHeaders {
    bearer: HeaderMap,
    oauth: HeaderMap,
}

impl AuthHeaders {
    #[instrument]
    pub async fn new() -> HelixResult<Self> {
        let client_id = HeaderValue::from_str(var!(Var::ClientId).await?)?;
        let browser_id = HeaderValue::from_str(var!(Var::BrowserId).await?)?;
        let oauth_value = HeaderValue::from_str(&format!("OAuth {}", var!(Var::UserToken).await?))?;
        let bearer_value =
            HeaderValue::from_str(&format!("Bearer {}", var!(Var::AppToken).await?))?;

        let mut bearer = HeaderMap::new();
        bearer.insert(AUTHORIZATION, bearer_value);
        bearer.insert("Client-Id", client_id);

        let mut oauth = HeaderMap::new();
        oauth.insert(AUTHORIZATION, oauth_value);
        oauth.insert("Client-Id", browser_id);

        tracing::debug!("built AUTHORIZATION headers for OAuth + Bearer tokens");

        Ok(Self { bearer, oauth })
    }
}

static HEADERS: LazyLock<OnceCell<AuthHeaders>> = LazyLock::new(OnceCell::new);
pub async fn auth_headers() -> HelixResult<&'static AuthHeaders> {
    HEADERS.get_or_try_init(|| AuthHeaders::new()).await
}

pub type HelixResult<T> = core::result::Result<T, HelixErr>;

#[derive(Debug, Error)]
pub enum HelixErr {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("while parsing environment vars: {0}")]
    EnvError(#[from] EnvErr),

    #[error("while creating a HeaderValue ({0})")]
    HeaderError(#[from] InvalidHeaderValue),

    #[error("tried to make a request containing an invalid username")]
    InvalidUsername,

    #[error("error during helix fetch: {0}")]
    FetchErr(String),

    #[error("error (with detail) during helix fetch: {:#?}", body)]
    FetchErrWithBody { body: Value },

    #[error("empty data field")]
    EmptyDataField,
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_fetch_user_by_id() {
        let provider = crate::util::tracing::build_subscriber().await.unwrap();

        {
            let _span = tracing::info_span!("test_span").entered();

            let mut user_ids = vec![
                "188503312", /* milia */
                "478187203", /* myramors */
            ]
            .into_iter()
            .map(|item| item.to_string())
            .collect();

            let user_details = Helix::fetch_users_by_id(&mut user_ids).await.unwrap();
            assert_eq!(user_details.len(), user_ids.len());
        }

        crate::util::tracing::destroy_tracer(provider);
    }
}
