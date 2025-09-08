use core::fmt;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{RwLock, mpsc, oneshot};
use tokio::time::{interval, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::parsing::parser::IrcMessage;

#[async_trait]
pub trait MessageHandler: Send + Sync + fmt::Debug {
    async fn handle_message(&self, channel: &str, message: &IrcMessage) -> Option<String>;
    async fn on_join(&self, channel: &str) {}
    async fn on_part(&self, channel: &str) {}
}

#[async_trait]
pub trait EventHandler: Send + Sync + fmt::Debug {
    async fn on_connect(&self, connection_id: usize) {}
    async fn on_disconnect(&self, connection_id: usize) {}
    async fn on_error(&self, connection_id: usize, error: &str) {}
}
