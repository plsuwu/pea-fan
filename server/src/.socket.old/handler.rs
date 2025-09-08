use core::fmt;

use async_trait::async_trait;
use tracing::{info, instrument};

use crate::{
    parsing::parser::{self, IrcMessage, IrcParser, Parser},
    socket::core::{EventHandler, MessageHandler},
};

#[derive(Debug)]
pub struct EchoMessageHandler;

#[derive(Debug)]
pub struct EchoEventHandler;

impl EchoEventHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventHandler for EchoEventHandler {
    async fn on_connect(&self, connection_id: usize) {
        info!("connected: {}", connection_id);
    }
    async fn on_disconnect(&self, connection_id: usize) {
        info!("disconnected: {}", connection_id);
    }
    async fn on_error(&self, connection_id: usize, error: &str) {
        info!(
            "handler error: connection {}, error: {}",
            connection_id, error
        );
    }
}

#[async_trait]
impl MessageHandler for EchoMessageHandler {
    async fn handle_message(&self, channel: &str, message: &IrcMessage) -> Option<String> {
        let parser = IrcParser::new();
        if let Ok(text) = parser.extract_chat_data(message) {
            info!("msg: {:#?}", text);

            return Some(text.message.to_string());
        }

        None
    }

    async fn on_join(&self, channel: &str) {
        info!("issued JOIN: #{}", channel);
    }
    async fn on_part(&self, channel: &str) {
        info!("issued PART: #{}", channel);
    }
}

impl EchoMessageHandler {
    pub fn new() -> Self {
        Self
    }
}
