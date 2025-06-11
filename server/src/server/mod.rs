pub mod midware;
pub mod subscriber;
pub mod types;

use crate::CHANNELS;
use crate::args::parse_cli_args;
use crate::db::redis::redis_pool;
use crate::server::midware::verify;
use crate::server::midware::verify::VerifiedBody;
use crate::socket::client::Client;
use crate::socket::settings::ConnectionSettings;
use axum::body::Body;
use axum::extract::Query;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware;
use axum::routing::{get, post};
use axum::{Json, Router};
use http::{Method, Request, Response, header};
use ring::digest;
use ring::hmac::{self, Key};
use ring::rand;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use subscriber::{get_user_data, stream_online};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};
use types::{
    ChallengeRequest, StreamCommonEvent, StreamCommonSubscription, StreamOfflinePayload,
    StreamOnlinePayload,
};

/**
 *   N.B:
 *
 *      Some of this could (and probably should) be pulled out for organization purposes
 * */

#[derive(Debug)]
pub struct IrcConnection {
    handle: JoinHandle<()>,
    cancellation_token: CancellationToken,
}

#[derive(Debug)]
pub struct IrcHandles {
    connections: HashMap<String, IrcConnection>,
}

impl IrcHandles {
    pub fn new() -> IrcHandles {
        IrcHandles {
            connections: HashMap::new(),
        }
    }

    pub fn get_connection_state(&self) -> Vec<(String, bool)> {
        self.connections
            .iter()
            .map(|(chan, conn)| (chan.clone(), !conn.handle.is_finished()))
            .collect()
    }

    pub fn get_connection_summary(&self) -> (Vec<String>, Vec<String>) {
        let mut active = Vec::new();
        let mut inactive = Vec::new();

        for (channel, conn) in &self.connections {
            if conn.handle.is_finished() {
                inactive.push(channel.clone());
            } else {
                active.push(channel.clone());
            }
        }

        (active, inactive)
    }

    pub fn is_active(&self, channel: &str) -> bool {
        if let Some(conn) = self.connections.get(channel) {
            !conn.handle.is_finished()
        } else {
            false
        }
    }

    pub fn cleanup_complete(&mut self) {
        self.connections
            .retain(|_chan, conn| !conn.handle.is_finished());
    }
}

static IRC_HANDLES: LazyLock<Arc<Mutex<IrcHandles>>> =
    LazyLock::new(|| Arc::new(Mutex::new(IrcHandles::new())));

const TWITCH_MESSAGE_TYPE_HEADER: &'static str = "Twitch-Eventsub-Message-Type";
const SERVER_PORT: u16 = 3000;

// We could probably wrap this with a sync primitive such that we can just call e.g `KEY_DIGEST.get_hex()`
// or `KEY_DIGEST.get_key()` from whatever thread in order to return a copy of/reference to the data
// we want
pub static KEY_DIGEST: LazyLock<RwLock<Secret>> = LazyLock::new(|| RwLock::new(Secret::new()));

/// Struct for HMAC key storage and generation methods.
///
/// Key is stored in-memory for the duration of the server's uptime; restarting the server should
/// reset this key (this mechanism is, at present, by design).
///
/// # Security
///
/// In a vacuum, `ring::rand::SystemRandom` is cryptographically secure (as far as I am aware
/// smile).
#[derive(Debug)]
pub struct Secret {
    pub key: Key,
    _digest: [u8; digest::SHA256_OUTPUT_LEN],
    pub _hex: String,
}

impl Secret {
    pub fn new() -> Self {
        let rng = rand::SystemRandom::new();
        let _digest: [u8; digest::SHA256_OUTPUT_LEN] = rand::generate(&rng).unwrap().expose();
        let _hex = hex::encode(_digest);

        let key = Key::new(hmac::HMAC_SHA256, &_hex.as_bytes());

        Self { _digest, _hex, key }
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self._hex)
    }
}

pub enum WebhookMessageType {
    Verify,
    Notify,
    Revoke,
}

impl WebhookMessageType {
    pub fn parse_to_str(&self) -> &'static str {
        match self {
            Self::Verify => "webhook_callback_verification",
            Self::Notify => "notification",
            Self::Revoke => "revocation",
        }
    }

    pub fn parse_from_str(rx: &str) -> Result<Self, String> {
        match rx {
            "webhook_callback_verification" => Ok(Self::Verify),
            "notification" => Ok(Self::Notify),
            "revocation" => Ok(Self::Revoke),
            _ => Err(format!("Received an invalid type header: {:?}", &rx)),
        }
    }
}

impl TryFrom<&str> for WebhookMessageType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        WebhookMessageType::parse_from_str(&value.to_string())
    }
}

impl Into<&str> for WebhookMessageType {
    fn into(self) -> &'static str {
        self.parse_to_str()
    }
}

/// Server listener
pub async fn serve(tx: oneshot::Sender<(SocketAddr, Option<String>)>) {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route("/webhook-global", post(webhook_handler))
        .route_layer(middleware::from_fn(verify::verify_sender_ident))
        // .route("/", get(root))
        .route(
            "/",
            get(|| async { "root endpoint has no content yet, leave me be or i will scream" }),
        )
        .route("/active-sockets", get(activity))
        .route("/ceilings/channel", get(get_channel))
        .route("/ceilings/user", get(get_user))
        .route("/checkhealth", get(|| async { "SERVER_OK" }))
        .layer(cors);

    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), SERVER_PORT);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();

    _ = tx.send((bind_addr, get_debug()));
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct RootSitemap {
    endpoints: Vec<String>,
}

// pub async fn root() -> Json<RootSitemap> {
// }

#[derive(Serialize, Deserialize)]
pub struct ActivitySummary {
    active_count: usize,
    active_broadcasters: Vec<String>,
}

// #[allow(unused_variables)]
pub async fn activity() -> Json<ActivitySummary> {
    let handles_guard = IRC_HANDLES.lock().unwrap();
    let (active, _) = handles_guard.get_connection_summary();

    println!("active: {:#?}", active);

    let summary = ActivitySummary {
        active_count: active.len(),
        active_broadcasters: active,
    };

    Json(summary)
}

#[derive(Serialize, Deserialize)]
pub struct GetChannelQueryParams {
    name: String,
}

#[derive(Deserialize)]
pub struct GetUserQueryParams {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedisQueryResponse {
    pub err: bool,
    pub err_msg: String,
    pub total: String,
    pub leaderboard: Vec<(String, isize)>,
}

pub async fn get_channel(Query(query): Query<GetChannelQueryParams>) -> Json<RedisQueryResponse> {
    if !CHANNELS.contains(&query.name.as_str()) {
        Json(RedisQueryResponse {
            err: true,
            err_msg: "NOT_TRACKED".to_string(),
            total: "0".to_string(),
            leaderboard: Vec::new(),
        })
    } else {
        let redis = redis_pool().await.unwrap();
        let res = redis.get_channel_data(&query.name).await;
        match res {
            Ok(r) => Json(r),
            Err(e) => {
                println!("[x] got error from redis: {:?}", e);

                // needs proper handling (e.g if a tracked chanel has no data)
                // but asdljk;ffasjdkl;jlfk;dsjl;kf for now
                Json(RedisQueryResponse {
                    err: true,
                    err_msg: format!("REDIS_ERROR({})", e),
                    total: "0".to_string(),
                    leaderboard: Vec::new(),
                })
            }
        }
    }
}

pub async fn get_user(Query(query): Query<GetUserQueryParams>) -> Json<RedisQueryResponse> {
    let redis = redis_pool().await.unwrap();
    match redis.get_user_data(&query.name).await {
        Err(_) => Json(RedisQueryResponse {
            err: true,
            err_msg: "NOT_TRACKED".to_string(),
            total: "0".to_string(),
            leaderboard: Vec::new(),
        }),
        Ok(val) => Json(val),
    }
}

/// Handles webhook callbacks on the `<ROOT_URL>:<PORT>/webhook-global` endpoint
pub async fn webhook_handler(headers: HeaderMap, body: VerifiedBody) -> Result<Body, StatusCode> {
    let notification: serde_json::Value = body.as_json().map_err(|_| StatusCode::BAD_REQUEST)?;
    let msg_type = headers
        .get(TWITCH_MESSAGE_TYPE_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // println!("type: {}", msg_type);
    // println!("content: {:#?}", notification);

    let webhook_msg_type = TryInto::<WebhookMessageType>::try_into(msg_type)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match webhook_msg_type {
        WebhookMessageType::Verify => {
            //
            // notification is a challenge request, verify to confirm the webhook
            //
            // (big ugly routine - maybe pull this out but i want to finish the functionality
            // for now!!)
            //
            let deserialization_result: serde_json::Result<ChallengeRequest> =
                serde_json::from_value(notification);
            if let Ok(challenge_req) = deserialization_result {
                let broadcaster_id = challenge_req
                    .subscription
                    .condition
                    .broadcaster_user_id
                    .clone();
                let broadcaster_data = get_user_data(&parse_cli_args().app_token, &broadcaster_id)
                    .await
                    .unwrap();
                if challenge_req.subscription.r#type == "stream.offline" {
                    // if we're receiving this websocket event, we can probe to see if the
                    // broadcaster is live and open a socket if required.
                    let is_streaming =
                        stream_online(&parse_cli_args().app_token, &broadcaster_id).await;

                    match is_streaming {
                        Ok(true) => {
                            tokio::task::spawn(async move {
                                if let Err(e) = open_websocket(&broadcaster_data.login).await {
                                    eprintln!(
                                        "[x] failed to open websocket for '{}': '{:?}'",
                                        &broadcaster_id, e
                                    )
                                }
                            });
                        }
                        Err(e) => {
                            println!(
                                "[x] failed to retrieve broadcaster '{}' online state: {:?}",
                                broadcaster_data.login, e
                            );
                        }
                        _ => (),
                    }
                }

                println!(
                    "[+] responding to challenge for '{}-{}' with '{}'",
                    broadcaster_data.id, challenge_req.subscription.r#type, challenge_req.challenge
                );

                Ok(challenge_req.challenge.into())
            } else {
                Ok("".to_string().into())
            }
        }

        WebhookMessageType::Notify => Ok(read_notification(notification).unwrap().into()),
        WebhookMessageType::Revoke => {
            // idk what to do with this yet but we can cross this bridge when we come to it i
            // think
            println!("[x] received a 'revoke' type token: {:#?}", notification);
            Ok("".into())
        }
    }
}

pub fn read_notification(body: Value) -> Result<String, serde_json::Error> {
    match &body["subscription"]["type"].as_str() {
        Some("stream.online") => stream_event_notify::<StreamOnlinePayload>(body),
        Some("stream.offline") => stream_event_notify::<StreamOfflinePayload>(body),

        // shouldn't hit this arm as we're only going to be notified for
        // events on topics we're subscribed to
        _ => Ok("".to_string()),
    }
}

/// Safe deserialization of a subscription notification
///
/// `StreamCommonEvent` and `StreamCommonSubscription` trait implementations facilitate access
/// to required fields on nested in `event` and `subscription` parent fields via methods.
fn stream_event_notify<T>(body: serde_json::Value) -> Result<String, serde_json::Error>
where
    T: StreamCommonEvent + StreamCommonSubscription + serde::de::DeserializeOwned + Clone + 'static,
{
    let payload: T = serde_json::from_value(body)?;
    let channel = if payload.broadcaster_login() == "testBroadcaster" {
        "sleepiebug".to_string()
    } else {
        payload.broadcaster_login().to_string()
    };

    println!("[+] recv '{}' event for '{}'.", payload.r#type(), channel);

    let channel_clone = channel.clone();
    if payload.r#type() == "stream.online" {
        tokio::task::spawn(async move {
            if let Err(e) = open_websocket(&channel_clone).await {
                eprintln!(
                    "[x] failed to open websocket for '{}': '{:?}'",
                    channel_clone, e
                )
            }
        });
    } else {
        tokio::task::spawn(async move {
            if let Err(e) = close_websocket(&channel_clone).await {
                eprintln!(
                    "[x] failed to open websocket for '{}': '{:?}'",
                    channel_clone, e
                )
            }
        });
    }

    Ok(channel.to_string())
}

/// Log server port and secret key string to stdout
///
/// Intended for debugging; this would be automatically sent on subscription to a topic in
/// production.
fn get_debug() -> Option<String> {
    let digest_lock = &*KEY_DIGEST;
    if let Ok(digest) = digest_lock.read() {
        Some(digest._hex.clone())
    } else {
        None
    }
}

pub async fn open_websocket(channel: &str) -> anyhow::Result<()> {
    let mut irc_handles_guard = IRC_HANDLES.lock().unwrap();
    irc_handles_guard.cleanup_complete();

    if irc_handles_guard.is_active(channel) {
        println!("[x] socket handle ('{}') is already open.", channel);
        return Ok(());
    }

    let args = parse_cli_args();
    let conn_settings = Arc::new(ConnectionSettings::new(
        &args.user_token,
        &args.login,
        channel,
    ));

    let cancellation_token = CancellationToken::new();
    let cancel_token_clone_runner = cancellation_token.clone();
    let cancel_token_clone_reader = cancellation_token.clone();

    let channel_name = channel.to_string();
    let irc_handle = tokio::task::spawn(async move {
        tokio::select! {
            result = run_websocket_conn(conn_settings, cancel_token_clone_runner.clone()) => {
                match result {
                    Ok(()) => println!("[+] websocket '{}' completed normally", channel_name),
                    Err(e) => println!("[x] websocket '{}' failed: {}", channel_name, e),
                }
            }

            _ = cancel_token_clone_reader.cancelled() => {
                println!("[+] websocket '{}' cancelled gracefully.", channel_name);
            }
        }
    });

    let connection = IrcConnection {
        handle: irc_handle,
        cancellation_token,
    };

    irc_handles_guard
        .connections
        .insert(channel.to_string(), connection);
    println!("[+] opened websocket connection '{}'", channel);

    Ok(())
}

pub async fn close_websocket(channel: &str) -> anyhow::Result<bool> {
    // The compiler does not let us do this by simply acquiring the IRC_HANDLES lock, cancelling the
    // cancellation token, and calling `drop(irc_handles_guard)`; it doesn't appear to recognise
    // that `drop` is discarding the reference by itself.
    //
    // We get around this by acquiring the lock on IRC_HANDLES in a separate lexical scope,
    // cancelling the cancellation token, and finally binding the handle to a variable outside
    // the scope.
    //
    // [This issue] seems to indicate the compiler could be made to recognize the drop's move
    // semantics with `-Zdrop-tracking` but the language server errors are also kind of annoying
    //
    // [This issue]: https://github.com/rust-lang/rust/issues/87309
    let mut connection_handle = None;
    {
        let mut irc_handles_guard = IRC_HANDLES.lock().unwrap();
        let conn = irc_handles_guard.connections.remove(channel);
        if let Some(c) = conn {
            c.cancellation_token.cancel();
            connection_handle = Some(c.handle);
        }
    }
    if let Some(handle) = connection_handle {
        let timeout = tokio::time::timeout(std::time::Duration::from_secs(5), handle);
        match timeout.await {
            Ok(Ok(())) => {
                // graceful closure without error
                println!("[+] websocket task '{}' closure ok", channel);
                Ok(true)
            }
            Ok(Err(e)) => {
                // error in nested handler (still consider this 'successfully' closed)
                println!("[x] websocket task '{}' panicked: {:?}", channel, e);
                Ok(true)
            }
            Err(_) => {
                // timeout (force-closed, so still technically successful)
                println!("[x] timeout during websocket task '{}' closure", channel);
                Ok(true)
            }
        }
    } else {
        println!("[x] no active websocket task for '{}'", channel);
        Ok(false)
    }
}

pub async fn run_websocket_conn(
    conn_settings: Arc<ConnectionSettings>,
    cancel_token: CancellationToken,
) -> anyhow::Result<()> {
    let socket = Client::new(&conn_settings).await?;

    socket.open(&conn_settings).await?;
    socket.loop_read(cancel_token).await?;

    Ok(())
}
