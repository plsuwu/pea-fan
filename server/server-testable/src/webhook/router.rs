use axum::Router;
use std::net::SocketAddr;
use tokio::sync::oneshot;

pub type TxSender = (SocketAddr, Option<String>);
pub async fn route(tx: oneshot::Sender<TxSender>) {
    let app = Router::new()
        .route("/webhook-global", post(webhook_handler))
        .route_layer(super::middleware::verify::sender_ident);
}
