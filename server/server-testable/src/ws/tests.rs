#![allow(dead_code)]

use axum::Router;
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::{Message, WebSocket};
use axum::response::Response;
use axum::routing::get;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;

pub async fn listener() -> (TcpListener, SocketAddr) {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
        .await
        .unwrap();

    let addr = listener.local_addr().unwrap();

    (listener, addr)
}

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
                .send(Message::Text(format!("{msg}").into()))
                .await
                .is_err()
            {
                break;
            }
        }
    }
}
