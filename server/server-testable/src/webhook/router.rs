use axum::{Router, body::Body, routing::post};
use http::{HeaderMap, StatusCode};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::sync::oneshot;
use tower_http::cors::{self, Any};

use crate::webhook::middleware::verify::{self, VerifiedBody};

pub type TxSender = (SocketAddr, String);
pub async fn route(tx: oneshot::Sender<TxSender>) {
    let app = Router::new()
        .route("/webhook-global", post(webhook_handler))
        .route_layer(axum::middleware::from_fn(
            crate::webhook::middleware::verify::sender_ident,
        ));

    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3000);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();

    _ = tx.send((bind_addr, get_debug()));
    axum::serve(listener, app).await.unwrap();
}

async fn webhook_handler(headers: HeaderMap, body: VerifiedBody) -> Result<Body, StatusCode> {
    todo!()
}

fn get_debug() -> String {
    verify::SESSION_KEY.get_hex_key()
}
