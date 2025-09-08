use std::collections::HashMap;

use async_trait::async_trait;
use thiserror::Error;
use tracing::{debug, error};

use crate::parsing::commands::{IrcCommand as ParsedCommand, UserInfo};
use crate::socket::client::IrcResult;
use crate::socket::core::IrcEvent;

pub type HandlerResult<T> = core::result::Result<T, HandlerError>;

#[derive(Error, Debug)]
pub enum HandlerError {
    // #[error("e: {0}")]
}

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &IrcEvent) -> HandlerResult<()>;
}

pub struct EventRouter {
    handlers: HashMap<String, Vec<Box<dyn EventHandler>>>,
}

impl EventRouter {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register<H>(&mut self, pattern: &str, handler: H)
    where
        H: EventHandler + 'static,
    {
        self.handlers
            .entry(pattern.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }

    pub async fn route(&self, event: &IrcEvent) {
        for (pattern, handlers) in &self.handlers {
            if self.matches_pattern(event, pattern) {
                for handler in handlers {
                    if let Err(e) = handler.handle(event).await {
                        error!("handler error (on pattern '{}'): {}", pattern, e);
                    }
                }
            }
        }
    }

    fn matches_pattern(&self, event: &IrcEvent, pattern: &str) -> bool {
        match pattern {
            "logger" => true,
            "counter" => event.is_privmsg(),
            // channel if channel.starts_with('#') => {
            // true
            //     event.channel_name().map(|c| c == channel).unwrap_or(false)
            // }
            _ => false,
        }
    }
}

pub struct IrcCounter {
    pub channel: String,
    pub trigger: String,
    pub proc_chat: bool,
}

impl IrcCounter {
    pub fn new(channel: &str, trigger: &str, proc_chat: bool) -> Self {
        Self {
            channel: channel.to_string(),
            trigger: trigger.to_string(),
            proc_chat,
        }
    }

    pub fn check_message(
        &self,
        channel: &str,
        message: &str,
        user_info: &Option<UserInfo>,
    ) -> HandlerResult<()> {
        let tokenized: Vec<&str> = message.split_whitespace().collect();

        if tokenized.len() == 2
            // TODO: make these constants or struct fields PLEASE pelaeplaepleapls
            && tokenized[0] == "@plss"
            && tokenized[1] == "!gpc"
            && self.proc_chat
        {
            debug!(
                "PROC ON {} -> ...pull db stuff for user: {:#?}",
                channel, user_info
            );
        } else if message.contains(&self.trigger) {
            debug!(
                "TRIGGERED ON {} -> ...push db stuff for user: {:#?}",
                channel, user_info
            );
        }

        // let res =
        // IrcCommand::PrivMsg {
        //     channel: self.channel,
        //     message: ,
        //     user_info: (),
        // };

        Ok(())
    }
}

#[async_trait]
impl EventHandler for IrcCounter {
    async fn handle(&self, event: &IrcEvent) -> HandlerResult<()> {
        if let IrcEvent::PrivMsgRx {
            channel,
            user_id,
            message,
            user_info,
        } = event
        {
            debug!("{}:[RX]: {:#?}", channel, event);
            self.check_message(&channel, &message, &user_info)?;
        }

        Ok(())
    }
}

pub struct IrcLogger {
    channel: String,
}

impl IrcLogger {
    pub fn new(channel: &str) -> Self {
        Self {
            channel: channel.to_string(),
        }
    }
}

#[async_trait]
impl EventHandler for IrcLogger {
    async fn handle(&self, event: &IrcEvent) -> HandlerResult<()> {
        if let IrcEvent::RawMsg(ast) = event {
            debug!("rx: {:?}", ast);
        }

        Ok(())
    }
}
