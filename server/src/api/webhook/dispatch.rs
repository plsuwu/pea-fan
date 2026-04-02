use futures::stream::{FuturesUnordered, StreamExt};
use tracing::instrument;

use crate::api::webhook::{StreamGenericRequestType, WebhookError};
use crate::db::prelude::ChannelId;
use crate::util::helix::Helix;

type Result<T> = core::result::Result<T, WebhookError>;

const HELIX_URL: &str = "https://api.twitch.tv/helix";

#[instrument(skip(ids))]
pub async fn reset_hooks(ids: &[String]) -> Result<()> {
    let active_hooks = Helix::get_active_subscriptions().await?;
    tracing::debug!(?active_hooks, "active_hooks");

    if !active_hooks.is_empty() {
        tracing::debug!("active_hooks populated - deleting...");
        Helix::delete_subscriptions(&active_hooks).await?;
    }

    let mut futs: FuturesUnordered<_> = ids
        .iter()
        .map(|id| {
            Helix::create_subscription(ChannelId(id.clone()), StreamGenericRequestType::Online)
        })
        .collect();

    futs.extend(ids.iter().map(|id| {
        Helix::create_subscription(ChannelId(id.clone()), StreamGenericRequestType::Offline)
    }));

    while let Some(result) = futs.next().await {
        match result {
            Ok(res) => tracing::info!(?res, "HOOK SUBSCRIPTION OK"),
            Err(e) => tracing::error!(error = ?e, "HOOK SUBSCRIPTION FAIL"),
        }
    }

    Ok(())
}
