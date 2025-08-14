use crate::args::get_cli_args;
use crate::constants::STREAM_ONLINE;
use crate::server::types::{StreamCommonEvent, StreamCommonSubscription};
use crate::socket::{client::Client, settings::ConnectionSettings};
use axum::body::Body;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub static IRC_HANDLES: LazyLock<Arc<Mutex<IrcHandles>>> =
    LazyLock::new(|| Arc::new(Mutex::new(IrcHandles::new())));

pub enum WebhookMessageType {
    Verify,
    Notify,
    Revoke,
}

impl WebhookMessageType {
    pub fn parse_from_str(rx: &str) -> Result<Self, String> {
        match rx {
            "webhook_callback_verification" => Ok(Self::Verify),
            "notification" => Ok(Self::Notify),
            "revocation" => Ok(Self::Revoke),
            _ => Err(format!("Received an invalid type header: {:?}", &rx)),
        }
    }
}

impl From<&str> for WebhookMessageType {
    fn from(value: &str) -> Self {
        WebhookMessageType::parse_from_str(value).unwrap()
    }
}

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

    #[allow(dead_code)]
    /// Retrieves the state of a single websocket connection
    pub fn get_connection_state(&self) -> Vec<(String, bool)> {
        self.connections
            .iter()
            .map(|(chan, conn)| (chan.clone(), !conn.handle.is_finished()))
            .collect()
    }

    /// Retrieves a summary of all connections
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

    /// Retrieves the activity state of a specified connection
    pub fn is_active(&self, channel: &str) -> bool {
        if let Some(conn) = self.connections.get(channel) {
            !conn.handle.is_finished()
        } else {
            false
        }
    }

    /// Disposes of finalized connections
    pub fn cleanup_complete(&mut self) {
        self.connections
            .retain(|_chan, conn| !conn.handle.is_finished());
    }
}

/// Safe deserialization of a subscription notification
///
/// # Trait Generic
///
/// Instead of using an `Option` wrapper and requiring checks for `Some` when trying to use certain
/// fields of stream.[event]-related JSON structures, we use `StreamCommonEvent` and 
/// `StreamCommonSubscription` trait implementations to facilitate access to required fields on
/// nested `event` and `subscription` parent fields via named methods, performing a single check
/// for `Some` at the function call site instead.
///
/// For example, `StreamOnlineEvent` and `StreamOfflineEvent` both implement the
/// `StreamCommonEvent` trait, so their fields are accessed via a (simplified) method call:
///
/// ```
/// #[derive(Deserialize, Clone)]
/// struct EventOne {
///     broadcaster_uid: String,
///     some_other_field: String,
/// }
/// 
/// #[derive(Deserialize, Clone)]
/// struct EventTwo {
///     broadcaster_id: String,
///     // end of struct fields
/// }
///
/// // Both structs implement the `broadcaster_*` fields that `stream_event_notify` requires, so we
/// // can implement trait methods from `StreamEventCommon` so this function can use the data.
/// impl StreamCommonEvent for EventOne {
///     fn broadcaster_id(&self) -> &str {
///         self.broadcaster_uid
///     }
/// }
///
/// impl StreamCommonEvent for EventTwo {
///     fn broadcaster_id(&self) -> &str {
///         self.broadcaster_id
///     }
/// }
/// 
/// # fn handler(body: Value) {
/// let dispatch_handler = match body["subscription"]["type"] {
///     Some("stream.event_one") => stream_event_notify::<StreamCommonEvent>(body),
///     _ => /* ... */
/// }
/// # }
/// ```
/// > Note: 
/// > The above trait implementation is intended as an example for how this function is
/// > intended to be used. 
/// >
/// > In practice, `StreamCommonEvent` requires more function implementations
/// > than shown here.
pub fn stream_event_notify<T>(body: serde_json::Value) -> Result<Body, serde_json::Error>
where
    T: StreamCommonEvent + StreamCommonSubscription + serde::de::DeserializeOwned + Clone + 'static,
{
    let payload: T = serde_json::from_value(body)?;
    let channel = if payload.broadcaster_login() == "testBroadcaster" {
        "plss".to_string()
    } else {
        payload.broadcaster_login().to_string()
    };

    println!("[+] recv '{}' event for '{}'.", payload.r#type(), channel);

    let channel_clone = channel.clone();
    if payload.r#type() == STREAM_ONLINE {
        tokio::task::spawn(async move {
            if let Err(e) = open_websocket(&channel_clone).await {
                eprintln!(
                    "[x] failed to open websocket for '{}': '{:?}'",
                    channel_clone, e
                )
            }
        });
    } else {
        println!("[+] recv '{}' event for '{}'.", payload.r#type(), channel);
        tokio::task::spawn(async move {
            if let Err(e) = close_websocket(&channel_clone).await {
                eprintln!(
                    "[x] failed to open websocket for '{}': '{:?}'",
                    channel_clone, e
                )
            }
        });
    }

    Ok(channel.into())
}

/// Opens a websocket connection to the specified twitch channel
pub async fn open_websocket(channel: &str) -> anyhow::Result<()> {
    let mut irc_handles_guard = IRC_HANDLES.lock().unwrap();
    irc_handles_guard.cleanup_complete();

    if irc_handles_guard.is_active(channel) {
        println!("[x] socket handle ('{}') is already open.", channel);
        return Ok(());
    }

    let args = get_cli_args();
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

    let now_active = irc_handles_guard.is_active(channel);
    println!("[+] was socket started Ok? {}", now_active);

    println!("[+] opened websocket connection '{}'", channel);

    Ok(())
}

/// Closes a websocket connection
///
/// # Implementation Note
///
/// The compiler does not let us do this by simply acquiring the IRC_HANDLES lock, cancelling the
/// cancellation token, and calling `drop(irc_handles_guard)`; it doesn't appear to recognise
/// that `drop` is discarding the reference by itself (which I *think* means we're not holding the
/// lock across an await?):
///
/// ```ignore
/// # let channel = "";
/// let mut irc_handles_guard = IRC_HANDLES.lock().unwrap();
///
/// // `remove` method pops the channel from map and binds to `conn`; lock on the `IRC_HANDLES`
/// // mutex should no longer be required as the key/value is no longer associated with the map
/// if let Some(conn) = irc_handles_guard.connections.remove(channel) {
///
///     conn.cancellation_token.cancel();
///     drop(irc_handles_guard); // Mutex lock is dropped here
///
///     match timeout(
///         Duration::from_secs(5),
///         conn.handle
///     ).await 
/// //:  ^^^^^^ future is not `Send` as this value is used across an await
///     # {
///     #     # _ => (),
///     # }
/// # }
/// ```
///
/// We get around this by acquiring the lock on IRC_HANDLES in a separate lexical scope,
/// cancelling the cancellation token, and finally binding the handle to a variable outside
/// the scope.
///
/// [This issue] seems to indicate the compiler could be made to recognize the drop's move
/// semantics with `-Zdrop-tracking` but I prefer the manual scoping solution over requiring
/// compilation flags.
///
/// [This issue]: https://github.com/rust-lang/rust/issues/87309
pub async fn close_websocket(channel: &str) -> anyhow::Result<bool> {
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
