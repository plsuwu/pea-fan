use super::super::constants::{IRC_COMMAND_CHAT, IRC_COMMAND_JOIN, IRC_COMMAND_PING};
use super::super::constants::{KEEPALIVE_RESPONSE, NEEDLE};
use crate::db::redis::redis_pool;
use crate::parser::parser::{self, IrcMessage, IrcParser};
use crate::socket::settings::ConnectionSettings;
use chrono::prelude::*;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Result;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tokio_util::sync::CancellationToken;

pub type Writer = Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>;
pub type Reader = Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>;

#[derive(Debug)]
pub struct Client {
    pub writer: Writer,
    pub reader: Reader,
    channel: String,
}

impl Client {
    /// Creates the RW streams for the given URL
    ///
    /// I am unsure if this actually CREATES a connection? But the `open` method actually sends
    /// the required auth messages.
    pub async fn new(conn: &Arc<ConnectionSettings>) -> Result<Self> {
        let url = &conn.url;

        let (stream, _) = connect_async(*url).await?;
        let (w, r) = stream.split();

        let writer = Arc::new(Mutex::new(w));
        let reader = Arc::new(Mutex::new(r));

        Ok(Self {
            reader,
            writer,
            channel: conn.channel.clone(),
        })
    }

    /// Sends the Twitch IRC authentication commands to the broadcaster chat
    pub async fn open(&self, conn: &Arc<ConnectionSettings>) -> Result<()> {
        for cmd in &conn.ws_authentication {
            self.write(cmd).await?;
        }

        Ok(())
    }

    /// Loops over the input reader for this client, checking for incoming IRC messages
    pub async fn loop_read(&self, cancel_token: CancellationToken) -> anyhow::Result<()> {
        let reader_clone = self.reader.clone();

        loop {
            tokio::select! {
                incoming_res = Self::read(&reader_clone) => {
                    if let Some(incoming) = incoming_res {
                        let raw_data = incoming.to_string();

                        // we could construct a parser to a specification based on the broadcaster
                        // to search for unique words/strings/??
                        let parser = parser::IrcParser::new();

                        // check parsed success/failure and handle accordingly
                        match parser.parse_socket_data(&raw_data) {
                            Ok(parsed) => {
                                self.action_irc(&parsed, &parser).await?;
                            }
                            Err(e) => {
                                println!("[x] failed to parse irc notification: {:?}", e);
                                continue;
                            }
                        }

                    } else {
                        // attempted to perform some action on a connection that was not open
                        println!("[x] irc conn appears closed.");
                        break;
                    }
                }

                // received a cancel signal in the token
                _ = cancel_token.cancelled() => {
                    println!("[{}] irc read loop cancellation initiated - sending `PART` command.", get_current_time());
                    match self.write(&format!("PART #{}", self.channel)).await {
                        Err(e) => eprintln!("[{}] failed to send `PART` command to '{}': {:?}", get_current_time(), self.channel, e),
                            _ => (),
                    }

                    break;
                }
            }
        }

        Ok(())
    }

    /// Determine the type of message sent and process it accordingly
    pub async fn action_irc<'a>(
        &self,
        data: &IrcMessage<'a>,
        parser: &IrcParser,
    ) -> anyhow::Result<()> {
        match data.command {
            IRC_COMMAND_PING => {
                println!("[{}] rx KEEPALIVE", get_current_time());
                match Self::write(self, KEEPALIVE_RESPONSE).await {
                    Err(e) => println!("[{}] tx KEEPALIVE err: {:?}", get_current_time(), e),
                    _ => (),
                }
            }

            IRC_COMMAND_JOIN => println!(
                "[{}] JOIN: '{}'",
                get_current_time(),
                parser
                    .get_channel(&data)
                    .unwrap_or("INVALID_CHANNEL_RESULT"),
            ),

            IRC_COMMAND_CHAT => {
                if let Ok(chat) = parser.get_chat(&data) {
                    let message = chat.message;
                    if message.to_lowercase().contains(NEEDLE) {
                        let chatter = chat.display_name;
                        let channel = chat.channel;
                        println!(
                            "[{}] in [{}] PRIVMSG::{}: '{}'",
                            get_current_time(),
                            channel,
                            chatter,
                            message,
                        );

                        let pool = redis_pool().await?;
                        pool.increment(channel, chatter).await?;
                    }
                } else {
                    eprintln!("[x] err parsing chat msg: {:?}", data);
                }
            }

            // we DEFINITELY want to handle more cases here.
            // _ => println!("[-] rx unhandled ws message {:?}", data),
            _ => (),
        }

        Ok(())
    }

    /// Send a message outbound to the socket
    pub async fn write(&self, data: &str) -> Result<()> {
        let msg = Message::text(data.to_string());
        Ok(self.writer.lock().await.send(msg).await?)
    }

    /// Read an incoming message from the socket
    pub async fn read(reader: &Reader) -> Option<Message> {
        let mut lock = reader.lock().await;

        if let Some(incoming) = lock.next().await {
            return Some(incoming.unwrap_or(Message::text("INCOMING_FAULT")));
        }

        None
    }

    /// Debug print helper function - used to log a `Message` struct
    #[allow(dead_code, unused_variables)]
    #[cfg(not(feature = "production"))]
    fn print(data: &Message) {
        let curr = get_current_time();
        let text = data.to_text().unwrap_or("FAILED_TEXT_CONV");
        if !text.contains("PASS oauth:") {
            println!("[{}] SENT: {:?}", curr, text);
        }
    }
}

/// Returns the current date and time as a String
///
/// Format: `YYYY-MM-DD[HH:MM:SS]`
pub fn get_current_time() -> String {
    let curr = Local::now();
    curr.format("%Y-%m-%d@%H:%M:%S").to_string()
}
