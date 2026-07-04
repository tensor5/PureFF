use octofer::Context;
use octofer::github::webhook_events::WebhookEventPayload;
use std::sync::Arc;
use tracing::{info, warn};

use crate::AppData;

pub async fn installation_handler(context: Context, _: Arc<AppData>) -> anyhow::Result<()> {
    info!("Received installation event");

    let event = match context.event() {
        Some(event) => event,
        None => {
            warn!("No event found in context");
            return Ok(());
        }
    };

    let payload = match &event.specific {
        WebhookEventPayload::Installation(payload) => payload,
        _ => return Ok(()), // Not an installation event
    };

    let repositories = match &payload.repositories {
        Some(repos) => repos,
        None => {
            info!("No repositories in installation payload");
            return Ok(());
        }
    };

    for repo in repositories {
        info!("Installed in: {}", repo.full_name);
    }

    Ok(())
}
