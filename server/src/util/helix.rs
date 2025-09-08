use core::fmt;
use futures::{StreamExt, stream};
use http::header::{AUTHORIZATION, InvalidHeaderValue};
use http::{HeaderMap, HeaderValue};
use reqwest::Response;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::cmp::max_by;
use std::sync::{LazyLock, PoisonError, RwLockReadGuard};
use thiserror::Error;
use tokio::sync::OnceCell;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::database::redis::NOT_VALID_HELIX_USER;
use crate::util::secrets::{ENV_SECRETS, Env};

static HEADERS: LazyLock<OnceCell<AuthHeaders>> = LazyLock::new(OnceCell::new);
pub async fn auth_headers() -> HelixResult<&'static AuthHeaders> {
    HEADERS
        .get_or_try_init(|| async { AuthHeaders::new().await })
        .await
}

pub const HELIX_URI_BASE: &str = "https://api.twitch.tv/helix";
pub const HELIX_URN_USERS: &str = "users";
pub const HELIX_URN_STREAMS: &str = "streams";
pub const HELIX_URN_COLORS: &str = "chat/color";

pub type HelixResult<T> = core::result::Result<T, HelixError>;

#[derive(Debug, Error)]
pub enum HelixError {
    #[error("error during helix fetch")]
    FetchError,

    #[error("error during helix fetch: {:#?}", body)]
    FetchErrorBody { body: Value },

    #[error("error during helix fetch: invalid username in query")]
    FetchInvalidUsername,

    #[error("response contains missing or empty data field")]
    EmptyDataField,

    #[error("dotenvy error: {0}")]
    EnvError(#[from] dotenvy::Error),

    #[error("rwlock error (auth headers): {0}")]
    RwLockAuthHeadersError(#[from] PoisonError<RwLockReadGuard<'static, AuthHeaders>>),

    #[error("PoisonError while acquiring read lock on env: {0}")]
    RwLockError(#[from] PoisonError<RwLockReadGuard<'static, Env>>),

    #[error("reqwest error during fetch: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Invalid HeaverValue: {0}")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[error("unable to convert json to struct: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

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

pub struct Helix;

impl Helix {
    async fn make_request(uri: String) -> HelixResult<Response> {
        let client = reqwest::Client::new();
        let headers = auth_headers().await?.bearer.clone();

        debug!("using headers: {:?}", headers);

        client
            .get(uri)
            .headers(headers)
            .send()
            .await
            .map_err(|e| HelixError::ReqwestError(e))
    }

    async fn fetch_user_generic<T>(uri: String) -> HelixResult<T>
    where
        T: DeserializeOwned + fmt::Debug,
    {
        let response = Self::make_request(uri).await?;
        debug!("raw response: {:?}", response);

        if response.status() != 200 {
            error!("helix response status was not 200/OK: {:#?} ", response);
            if let Ok(reason) = response.json::<Value>().await {
                error!("recv error body: {:#?}", reason);

                let reason_clone = reason["message"].clone();
                let reason_str = reason_clone.as_str().ok_or(HelixError::FetchErrorBody {
                    body: reason.clone(),
                })?;

                if reason_str.starts_with("Invalid username") {
                    Err(HelixError::FetchInvalidUsername)
                } else {
                    Err(HelixError::FetchErrorBody { body: reason })
                }
            } else {
                Err(HelixError::FetchError)
            }
        } else {
            let rate_limit_remaining = response.headers().get("ratelimit-remaining");
            let rate_limit_total = response.headers().get("ratelimit-limit");
            if rate_limit_total.is_some() && rate_limit_remaining.is_some() {
                debug!(
                    "rate limit: {:?} of {:?}",
                    rate_limit_remaining.unwrap(),
                    rate_limit_total.unwrap()
                );
            }

            let response_body = response
                .json::<T>()
                .await
                .map_err(|e| HelixError::ReqwestError(e));

            debug!("{:?}", response_body);
            response_body
        }
    }

    #[instrument(skip(users, param_type))]
    pub async fn try_refetch(
        users: Vec<String>,
        param_type: HelixParamType,
    ) -> HelixResult<HelixResponse<HelixUser>> {
        let requests = users.into_iter().map(|user| {
            // copy value prior to move
            let param_type = param_type;
            async move {
                let params = build_query_params(param_type, &mut vec![user.to_string()]);
                let uri = format!("{}{}", String::from(HelixUri::Users), params[0]);

                match Self::fetch_user_generic::<HelixResponse<HelixUser>>(uri).await {
                    Ok(r) => {
                        // helix would return a 200 status response during testing where
                        // it would provide a valid-looking `data` array, but the array was
                        // completely empty and i have no idea why
                        if r.data.len() > 0 {
                            Ok((r.data, user))
                        } else {
                            Err((HelixError::EmptyDataField, user))
                        }
                    }
                    Err(e) => Err((e, user)),
                }
            }
        });

        // spawn threads to concurrently process users
        // TODO: un-magic number the worker thread count
        let results: Vec<_> = stream::iter(requests).buffer_unordered(50).collect().await;

        let mut refetched = Vec::new();
        for res in results {
            match res {
                Ok((res, _)) => refetched.push(res[0].clone()),
                Err((e, user)) => {
                    error!(NOT_VALID_HELIX_USER);
                    error!("manual intervention required for user '{}': {:?}", user, e)
                }
            }
        }

        Ok(HelixResponse { data: refetched })
    }

    #[allow(dead_code)]
    #[instrument(skip(users))]
    pub async fn fetch_user_by_id(users: &mut Vec<String>) -> HelixResult<Vec<InternalUser>> {
        debug!("input keyspace: {} items", users.len());

        let mut retrieved = Vec::new();
        let uri_params = build_query_params(HelixParamType::Id, users);

        for param in uri_params {
            let uri = format!("{}{}", String::from(HelixUri::Users), param);
            let queries = Self::fetch_user_generic::<HelixResponse<HelixUser>>(uri).await?;

            let users = queries
                .data
                .into_iter()
                .map(|user| InternalUser::from(user))
                .collect::<Vec<_>>();

            users.into_iter().for_each(|user| retrieved.push(user));
        }

        debug!("output keyspace: {} items", retrieved.len());
        Ok(retrieved)
    }

    #[instrument(skip(users))]
    pub async fn fetch_user_by_login(users: &mut Vec<String>) -> HelixResult<Vec<InternalUser>> {
        debug!("input keyspace: {} items", users.len());

        let mut retrieved = Vec::new();
        let uri_params = build_query_params(HelixParamType::Login, users);

        for (i, param) in uri_params.iter().enumerate() {
            let uri_users = format!("{}{}", String::from(HelixUri::Users), param);
            let user_queries =
                match Self::fetch_user_generic::<HelixResponse<HelixUser>>(uri_users).await {
                    Ok(d) => d,

                    // refetches the bad chunk user-by-user
                    //
                    // probably a better way of doing this heuristically (e.g discard user 
                    // if their name contains non-ascii characters, but for now we are just
                    // going to brute-force it
                    Err(HelixError::FetchInvalidUsername) => {
                        let chunk_start = i * 100;
                        let chunk_end = std::cmp::min(chunk_start + 100, users.len());
                        let users_chunk = users[chunk_start..chunk_end].to_vec();

                        error!("REFETCH REQUIRED FOR THESE USERS:");
                        error!("{:#?}", users_chunk);
                        error!("({} users total)", users_chunk.len());
                        error!("(at position: {} -> {}..{})", i, chunk_start, chunk_end);

                        Self::try_refetch(users_chunk.to_owned(), HelixParamType::Login).await?
                    }

                    // these errors are probably pretty tricky to recover from in the application's
                    // current state, so i'm skipping over them for now; this _probably_ will occur due
                    // to e.g invalid/expired token used in headers or something, however.
                    Err(e) => {
                        error!("helix api responded with an error: {:?}", e);
                        continue;
                    }
                };

            retrieved.extend(user_queries.data.into_iter().map(InternalUser::from));
        }

        trace!("{:?}", retrieved);
        info!(
            "output keyspace: {} of {} items",
            retrieved.len(),
            users.len()
        );

        let mut colors = Self::fetch_chat_colors(&retrieved).await?;

        // Sort user + color data by the user's ID and merge color into user
        //  - See `Ord` impl macro
        retrieved.sort();
        colors.sort();
        retrieved.iter_mut().enumerate().for_each(|(idx, user)| {
            if colors[idx].color.is_empty() {
                user.color = "#000000".to_string();
            } else {
                user.color = colors[idx].color.to_string();
            }
        });

        Ok(retrieved)
    }

    #[instrument(skip(broadcaster_ids))]
    pub async fn fetch_live_state(
        broadcaster_ids: &mut Vec<String>,
    ) -> HelixResult<Vec<InternalStream>> {
        let mut retrieved = Vec::new();
        let uri_params = build_query_params(HelixParamType::UserId, broadcaster_ids);

        for param in uri_params {
            let uri_streams = format!("{}{}", String::from(HelixUri::Streams), param);
            let queries =
                Self::fetch_user_generic::<HelixResponse<HelixStream>>(uri_streams).await?;

            retrieved.extend(queries.data.into_iter().map(InternalStream::from));
        }

        Ok(retrieved)
    }

    #[instrument(skip(users))]
    pub async fn fetch_chat_colors(users: &Vec<InternalUser>) -> HelixResult<Vec<HelixColor>> {
        let mut ids = users.iter().map(|u| u.id.as_str()).collect::<Vec<_>>();
        let params = build_query_params(HelixParamType::UserId, &mut ids);
        let mut retrieved = Vec::new();

        for param in params {
            let uri = format!("{}{}", String::from(HelixUri::Colors), param);
            let queries = Self::fetch_user_generic::<HelixResponse<HelixColor>>(uri).await?;
            retrieved.extend(queries.data.into_iter());
        }

        Ok(retrieved)
    }
}

#[instrument(skip(items))]
pub fn build_query_params(
    param_type: HelixParamType,
    items: &mut Vec<impl ToString>,
) -> Vec<String> {
    let queries: Vec<_> = items
        .chunks(100)
        .map(|chunk| {
            let mut query = format!(
                "?{}{}",
                String::from(param_type.clone()),
                chunk[0].to_string().to_lowercase()
            );
            for val in chunk[1..].into_iter() {
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

pub struct AuthHeaders {
    bearer: HeaderMap,

    #[allow(dead_code)]
    oauth: HeaderMap,
}

impl AuthHeaders {
    pub async fn new() -> HelixResult<Self> {
        let app_token = HeaderValue::from_str(&format!("Bearer {}", ENV_SECRETS.get().app_token))?;
        let user_token = HeaderValue::from_str(&format!("OAuth {}", ENV_SECRETS.get().user_token))?;

        let client_id = HeaderValue::from_str(&ENV_SECRETS.get().client_id)?;
        let global_client_id = HeaderValue::from_str(&ENV_SECRETS.get().global_client_id)?;

        let mut bearer = HeaderMap::new();
        let mut oauth = HeaderMap::new();

        bearer.insert(AUTHORIZATION, app_token);
        bearer.insert("client-id", client_id);

        oauth.insert(AUTHORIZATION, user_token);
        oauth.insert("client-id", global_client_id);

        Ok(Self { bearer, oauth })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelixResponse<T> {
    data: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelixStream {
    pub id: String,
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub game_id: String,
    pub game_name: String,
    pub r#type: String,
    pub title: String,
    pub tags: Vec<String>,
    pub viewer_count: i32,
    pub started_at: String,
    pub language: String,
    pub thumbnail_url: String,
    pub tag_ids: Vec<String>,
    pub is_mature: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalStream {
    pub id: String,
    pub login: String,
    pub thumbnail: String,
    pub game: String,
    pub title: String,
}

impl From<HelixStream> for InternalStream {
    fn from(value: HelixStream) -> Self {
        Self {
            id: value.user_id,
            login: value.user_login,
            thumbnail: value.thumbnail_url,
            game: value.game_name,
            title: value.title,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelixUser {
    pub broadcaster_type: String,
    pub created_at: String,
    pub description: String,
    pub display_name: String,
    pub id: String,
    pub login: String,
    pub offline_image_url: String,
    pub profile_image_url: String,
    pub r#type: String,
    pub view_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelixColor {
    pub user_id: String,
    pub user_name: String,
    pub user_login: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalUser {
    pub id: String,
    pub name: String,
    pub login: String,
    pub image: String,
    pub color: String,
    pub total: i32,
    pub redact: bool,
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

impl_common_user!(HelixColor, user_id);
impl_common_user!(HelixStream, id);
impl_common_user!(InternalStream, id);
impl_common_user!(InternalUser, id);

impl InternalUser {
    pub async fn new_from_logins(logins: &mut Vec<String>) -> HelixResult<Vec<Self>> {
        Helix::fetch_user_by_login(logins).await
    }
}

impl From<HelixUser> for InternalUser {
    fn from(value: HelixUser) -> Self {
        Self {
            id: value.id,
            login: value.login,
            name: value.display_name,
            image: value.profile_image_url,
            total: 0,
            color: String::from("#000000"),
            redact: false,
        }
    }
}
