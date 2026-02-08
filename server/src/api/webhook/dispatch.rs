#![allow(dead_code)]

use tracing::{self, instrument};

use crate::api::webhook::{StreamGenericRequestType, WebhookError};
use crate::util::helix::Helix;

type Result<T> = core::result::Result<T, WebhookError>;

#[instrument]
pub async fn reset_hooks(ids: &[String]) -> Result<()> {
    let active_hooks = Helix::get_active_subscriptions().await?;
    tracing::debug!(?active_hooks, "ACTIVE_HOOKS");

    if !active_hooks.is_empty() {
        Helix::delete_subscriptions(&active_hooks).await?;
    }

    for id in ids {
        Helix::create_subscription(id.clone().into(), StreamGenericRequestType::Online).await?;
        Helix::create_subscription(id.clone().into(), StreamGenericRequestType::Offline).await?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::net::SocketAddr;

    use futures::future::join_all;
    use tokio::sync::oneshot::Sender;

    use crate::{api::server::start_server, irc::client::start_irc_handler};

    use super::*;

    #[tokio::test]
    async fn test_hooks() {
        let provider = crate::util::tracing::build_subscriber().await.unwrap();

        let (tx_server, rx) = tokio::sync::mpsc::unbounded_channel::<SocketAddr>();
        let (tx_from_api, rx_from_api) =
            tokio::sync::mpsc::unbounded_channel::<(String, Sender<Vec<String>>)>();

        let channels = ["plss", "gibbbons", "chikogaki"]
            .into_iter()
            .map(|ch| ch.to_string())
            .collect();
        let mut handles = start_server(tx_server, tx_from_api, rx).await.unwrap();
        handles.extend(start_irc_handler(channels, rx_from_api).await.unwrap());

        let ids: [String; 1] = [String::from("103033809")];

        reset_hooks(&ids).await.unwrap();
        let hooks = Helix::get_active_subscriptions().await.unwrap();

        tracing::debug!(hooks = ?hooks);
        Helix::delete_subscriptions(&hooks).await.unwrap();

        _ = join_all(handles).await;
        crate::util::tracing::destroy_tracer(provider);


    }
}
