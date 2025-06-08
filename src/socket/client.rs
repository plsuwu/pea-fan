use chrono::prelude::*;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, RwLock};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Result;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tokio_util::sync::CancellationToken;

use crate::parser::parser;
use crate::socket::settings::ConnectionSettings;

pub type Writer = Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>;
pub type Reader = Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>;

#[derive(Debug, Clone)]
pub struct Client {
    pub writer: Writer,
    pub reader: Reader,
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

        Ok(Self { reader, writer })
    }

    /// Sends the Twitch IRC authentication commands to the broadcaster chat
    pub async fn open(&self, conn: &Arc<ConnectionSettings>) -> Result<()> {
        for cmd in &conn.ws_authentication {
            self.write(cmd).await?;
        }

        Ok(())
    }

    /// Loops over the input reader for this client, checking for incoming IRC messages
    pub async fn loop_read(&self, cancel_token: CancellationToken) {
        let reader_clone = self.reader.clone();

        loop {
            tokio::select! {
                incoming_res = Self::read(&reader_clone) => {
                    if let Some(incoming) = incoming_res {
                        let raw_data = incoming.to_string();
                        let parser = parser::IrcParser::new();
                        match parser.parse_message(&raw_data) {
                            Ok(parsed) => {
                                println!("[+] parsed incoming notification from irc ws:");
                                println!("[+] {:#?}", parsed);
                            }

                            Err(e) => {
                                println!("[x] failed to parse irc notification: {:?}", e);

                                // could break here depending on error??
                                continue;
                            }
                        }

                    } else {
                        println!("[x] irc conn appears closed.");
                        break;
                    }
                }

                _ = cancel_token.cancelled() => {
                    println!("[+] irc read loop cancelled gracefully.");
                    break;
                }
            }
        }
    }

    pub async fn write(&self, data: &str) -> Result<()> {
        let msg = Message::text(data.to_string());
        Self::print(&msg);

        Ok(self.writer.lock().await.send(msg).await?)
    }

    pub async fn read(reader: &Reader) -> Option<Message> {
        let mut lock = reader.lock().await;

        if let Some(incoming) = lock.next().await {
            return Some(incoming.unwrap_or(Message::text("INCOMING_FAULT")));
        }

        None
    }

    fn print(data: &Message) {
        let curr = get_current_time();
        println!("[{}] SENT: {:?}", curr, data);
    }
}

/// Returns the current date and time as a String
///
/// Format: `YYYY-MM-DD[HH:MM:SS]`
pub fn get_current_time() -> String {
    let curr = Local::now();
    curr.format("%Y-%m-%d[%H:%M:%S]").to_string()
}
