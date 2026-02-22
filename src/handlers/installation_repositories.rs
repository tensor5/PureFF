use octofer::Context;
use octofer::github::webhook_events::WebhookEventPayload;
use std::sync::Arc;
use tracing::{info, warn};

use crate::AppData;

pub async fn installation_repositories_handler(
    context: Context,
    _: Arc<AppData>,
) -> anyhow::Result<()> {
    info!("Received installation_repositories event");

    let event = match context.event() {
        Some(event) => event,
        None => {
            warn!("No event found in context");
            return Ok(());
        }
    };

    let payload = match &event.specific {
        WebhookEventPayload::InstallationRepositories(payload) => payload,
        _ => return Ok(()), // Not an installation event
    };

    for repo in &payload.repositories_added {
        info!("Added to {}", repo.full_name);
    }

    for repo in &payload.repositories_removed {
        info!("Removed from {}", repo.full_name);
    }

    Ok(())
}
