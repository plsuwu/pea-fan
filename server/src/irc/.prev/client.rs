use std::collections::HashSet;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use thiserror::Error;
use tokio::sync::mpsc::UnboundedReceiver;
use twitch_irc::login::{
    CredentialsPair, RefreshingLoginCredentials, StaticLoginCredentials, TokenStorage,
    UserAccessToken,
};
use twitch_irc::message::ServerMessage;
use twitch_irc::{ClientConfig, SecureWSTransport};
use twitch_irc::{TwitchIRCClient, validate};

use crate::util::env::Var;
use crate::util::{self, env};
use crate::var;

pub type IrcResult<T> = core::result::Result<T, IrcClientError>;

#[derive(Debug, Error)]
pub enum IrcClientError {
    #[error(transparent)]
    ChannelError(#[from] util::channel::ChannelError),

    #[error(transparent)]
    EnvError(#[from] env::EnvErr),

    #[error(transparent)]
    ValidateError(#[from] validate::Error),
}

#[derive(Debug, Clone)]
pub struct IrcChannel {
    channel_name: String,
    keywords: Vec<String>,
}

impl IrcChannel {
    pub fn new(channel: &str, keywords: &[String]) -> Self {
        let channel_name = String::from(channel);
        let keywords = keywords.to_vec();

        Self {
            channel_name,
            keywords,
        }
    }

    pub fn has_keyword(&self, needle: &str) -> bool {
        self.keywords.contains(&needle.to_string())
    }

    pub fn get_keywords(&self) -> Vec<String> {
        self.keywords.clone()
    }
}

pub struct IrcClient {
    client: TwitchIRCClient<SecureWSTransport, StaticLoginCredentials>,
    channels: Vec<IrcChannel>,
}

impl IrcClient {
    pub async fn new(channels: Vec<String>) -> IrcResult<(Self, UnboundedReceiver<ServerMessage>)> {
        let irc_channels = channels
            .iter()
            .map(|channel| {
                let keywords = ["piss".to_string()];
                IrcChannel::new(&channel, &keywords)
            })
            .collect();

        let login = var!(Var::UserLogin).await?.to_string();
        let token = var!(Var::UserToken).await?.to_string();

        let mut config = ClientConfig::default();
        config.login_credentials.credentials = CredentialsPair {
            login,
            token: Some(token),
        };

        config.new_connection_every = Duration::from_secs(2);

        let (transport, client) = TwitchIRCClient::new(config);
        client.set_wanted_channels(HashSet::from_iter(channels))?;

        Ok((
            Self {
                client,
                channels: irc_channels,
            },
            transport,
        ))
    }
}

#[cfg(test)]
mod test {
    use futures::future::join_all;
    use twitch_irc::message::{IRCMessage, IRCTags};

    use crate::util::{
        channel::update_channels,
        tracing::{build_subscriber, destroy_tracer},
    };

    use super::*;

    #[tokio::test]
    async fn test_irc_handler_many() {
        let provider = build_subscriber().await.unwrap();

        let channel_map = update_channels(None).await.unwrap();
        let test_channels = channel_map.into_keys().collect();

        tracing::debug!(test_channels = ?test_channels);

        let (client, mut rx) = IrcClient::new(test_channels).await.unwrap();

        let handle = tokio::spawn(async move {
            loop {
                if let Some(message) = rx.recv().await {
                    tracing::info!(message = ?message);
                }
            }
        });

        let timer_handle = tokio::spawn(async move {
            loop {
                let channels_clone = client.channels.clone();

                let timeout = tokio::time::Duration::from_secs(10);
                tokio::time::sleep(timeout).await;

                let mut channel_states = Vec::new();
                for ch in channels_clone {
                    let name = ch.channel_name;
                    let state = client.client.get_channel_status(name.clone()).await;

                    channel_states.push((name, state));
                }

                channel_states.iter().for_each(|ch| {
                    if !ch.1.1 {
                        tracing::info!(channel = ch.0, "trying join");
                        client.client.join(ch.0.clone()).unwrap();
                    }
                });
            }
        });

        _ = join_all([timer_handle, handle]).await;
        destroy_tracer(provider);
    }
}
