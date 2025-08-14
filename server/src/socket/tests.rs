#![allow(dead_code)]

use crate::ws::client::{CacheCounter, WsClientError, WsClientResult};
use async_trait::async_trait;
use axum::Router;
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::{Message, WebSocket};
use axum::response::Response;
use axum::routing::get;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;

/// Constructs a websocket server listener and binds it to `0.0.0.0`, returning the `TcpListener` and
/// `SocketAddr` the caller.
pub async fn listener() -> (TcpListener, SocketAddr) {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
        .await
        .unwrap();

    let addr = listener.local_addr().unwrap();

    (listener, addr)
}

/// Endpoint(s) to test websocket client reads/writes.
///
/// _Should_ implement an endpoint to check that sent data references the intended data but I don't really want to
/// do this right now.
pub fn router() -> Router {
    Router::new().route("/test-client-send", get(send_handler))
}

async fn send_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handler_recv_socket)
}

async fn handler_recv_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(msg) = msg {
            if socket
                .send(Message::Text(format!("{}", msg).into()))
                .await
                .is_err()
            {
                break;
            }
        }
    }
}

#[derive(Debug)]
pub struct MockRedisLayer {
    client: redis::Client,
}

impl MockRedisLayer {
    pub async fn new(url: &str) -> WsClientResult<Self> {
        let client = redis::Client::open(url).map_err(|e| WsClientError::Redis(e))?;

        Ok(Self { client })
    }
}

#[async_trait]
impl CacheCounter for MockRedisLayer {
    async fn increment_counter(&self, _channel: &str, _user: &str) -> WsClientResult<()> {
        // this is fine for now as we don't need this test layer to actually do anything
        Ok(())
    }
}
