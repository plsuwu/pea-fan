use tracing::{self, instrument};

use crate::api::webhook::{StreamGenericRequestType, WebhookError};
use crate::util::helix::Helix;

type Result<T> = core::result::Result<T, WebhookError>;

#[instrument]
pub async fn reset_hooks(ids: &[String]) -> Result<()> {
    let active_hooks = Helix::get_active_subscriptions().await?;
    tracing::debug!(?active_hooks, "ACTIVE_HOOKS");

    if active_hooks.len() != 0 {
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

    use super::*;

    #[tokio::test]
    async fn test_hooks() {
        let provider = crate::util::tracing::build_subscriber().await.unwrap();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<SocketAddr>();
        let server_handles = crate::api::server::start_server(tx, rx).await.unwrap();

        let ids: [String; 1] = [String::from("103033809")];

        reset_hooks(&ids).await.unwrap();
        let hooks = Helix::get_active_subscriptions().await.unwrap();

        tracing::debug!(hooks = ?hooks);
        Helix::delete_subscriptions(&hooks).await.unwrap();

        _ = join_all(server_handles).await;
        crate::util::tracing::destroy_tracer(provider);
    }
}
