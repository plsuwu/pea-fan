use tracing::instrument;

use crate::api::webhook::{StreamGenericRequestType, WebhookError};
use crate::util::helix::Helix;

type Result<T> = core::result::Result<T, WebhookError>;

const HELIX_URL: &str = "https://api.twitch.tv/helix";

#[instrument]
pub async fn reset_hooks(ids: &[String]) -> Result<()> {
    let active_hooks = Helix::get_active_subscriptions().await?;
    tracing::debug!(?active_hooks, "active_hooks");

    if !active_hooks.is_empty() {
        tracing::debug!("active_hooks populated - deleting...");
        Helix::delete_subscriptions(&active_hooks).await?;
    }

    for id in ids {
        Helix::create_subscription(id.clone().into(), StreamGenericRequestType::Online).await?;
        Helix::create_subscription(id.clone().into(), StreamGenericRequestType::Offline).await?;
    }

    Ok(())
}
