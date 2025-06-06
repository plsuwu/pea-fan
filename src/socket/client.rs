use chrono::prelude::*;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, RwLock};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Result;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use super::settings::ConnectionSettings;

pub type Writer = Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>;
pub type Reader = Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>;

pub struct Client {
    pub writer: Writer,
    pub reader: Reader,
}

impl Client {
    /// Creates the RW streams for the given URL
    ///
    /// I am unsure if this actually CREATES a connection? But the `open` method actually sends
    /// the required auth messages.
    pub async fn new(conn: &RwLock<ConnectionSettings>) -> Result<Self> {
        let url = &conn.read().unwrap().url;

        let (stream, _) = connect_async(url).await?;

        let (w, r) = stream.split();

        let writer = Arc::new(Mutex::new(w));
        let reader = Arc::new(Mutex::new(r));

        Ok(Self { reader, writer })
    }
    
    /// Sends the Twitch IRC authentication commands to the broadcaster chat
    pub async fn open(&self, conn: &RwLock<ConnectionSettings>) -> Result<()> {
        for cmd in &conn.read().unwrap().ws_auth_commands {
            self.write(cmd).await?;
        }

        Ok(())
    }
    
    /// Loops over the input reader for this client, checking for incoming IRC messages
    pub async fn loop_read(&self) {
        let reader_clone = self.reader.clone();
        loop {
            if let Some(incoming) = Self::read(&reader_clone).await {
                let raw = incoming.to_string();
                println!("{:?}", raw);
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
