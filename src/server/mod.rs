pub mod midware;
pub mod subscriber;
pub mod types;

use crate::args::parse_cli_args;
use crate::server::midware::verify;
use crate::server::midware::verify::VerifiedBody;
use crate::socket::client::Client;
use crate::socket::settings::ConnectionSettings;
use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware;
use axum::routing::{get, post};
use ring::digest;
use ring::hmac::{self, Key};
use ring::rand;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use types::{
    StreamCommonEvent, StreamCommonSubscription, StreamOfflinePayload, StreamOnlinePayload,
};

// static IRC_HANDLES: LazyLock<IrcHandlesMutex> = LazyLock::new(|| IrcHandlesMutex::new());

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
    let app = Router::new()
        .route("/", get(root))
        .route("/webhook-global", post(webhook_handler))
        .route_layer(middleware::from_fn(verify::verify_sender_ident))
        .route("/active-sockets", post(activity))
        .route("/health", get(|| async { "OK" }));

    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), SERVER_PORT);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();

    _ = tx.send((bind_addr, get_debug()));
    axum::serve(listener, app).await.unwrap();
}

pub async fn root() -> &'static str {
    "://"
}

#[allow(unused_variables)]
pub async fn activity(headers: HeaderMap) -> &'static str {
    "unimplemented :("
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
        WebhookMessageType::Verify => Ok(get_challenge_res(notification)
            .unwrap_or("".to_string())
            .into()),

        WebhookMessageType::Notify => Ok(read_notification(notification).unwrap().into()),

        WebhookMessageType::Revoke => {
            println!("");
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

fn stream_event_notify<T>(body: serde_json::Value) -> Result<String, serde_json::Error>
where
    T: StreamCommonEvent + StreamCommonSubscription + serde::de::DeserializeOwned,
{
    let payload: T = serde_json::from_value(body)?;
    let channel = payload.broadcaster_login();

    println!("[+] recv '{}' event for '{}'.", payload.r#type(), channel);

    if payload.r#type() == "stream.online" {
        _ = open_websocket(channel);
    } else {
        _ = close_websocket(channel);
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

pub async fn close_websocket(channel: &str) -> anyhow::Result<()> {
    Ok(())
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

pub async fn run_websocket_conn(
    conn_settings: Arc<ConnectionSettings>,
    cancel_token: CancellationToken,
) -> anyhow::Result<()> {
    let socket = Client::new(&conn_settings).await?;
    socket.open(&conn_settings).await?;

    socket.loop_read(cancel_token).await;

    Ok(())
}
//
//     // let args_clone = args.clone();
//     // let conn_settings = Arc::new(ConnectionSettings::new(
//     //     &args_clone.user_token,
//     //     &args_clone.login,
//     //     br,
//     // ));
//     //
//     // let irc_handle = tokio::task::spawn(async move {
//     //     let socket = Client::new(&conn_settings).await.unwrap();
//     //     socket.open(&conn_settings).await.unwrap();
//     //
//     //     socket.loop_read().await;
//     // });
//     //
//     // irc_handles.push(irc_handle);
// }

/// Returns the challenge string from the remote verification request
pub fn get_challenge_res(body: Value) -> Option<String> {
    let challenge_str = body["challenge"].as_str();
    if let Some(string) = challenge_str {
        Some(string.to_owned())
    } else {
        None
    }
}

//
//
//
//

// fn parse_message<T>(body: serde_json::Value) -> Result<String, serde_json::Error>
// where
//     T: ChatMessageCommon + serde::de::DeserializeOwned,
// {
//     let payload: T = serde_json::from_value(body)?;
//     let message = &payload.message().text;
//     if message.contains(REQUIRED_STRING) {
//         let _broadcaster = payload.broadcaster_user_login();
//         let chatter = payload.user_login();
//
//         println!("user:    \t'{}'", chatter);
//         println!("message: \t'{}'", message);
//
//         // do redis stuff
//         // ...
//     }
//
//     Ok("".to_string())
// }
