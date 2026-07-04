use octofer::Context;
use std::sync::Arc;
use tracing::{info, warn};

use crate::AppData;

pub async fn github_app_authorization_handler(
    context: Context,
    _: Arc<AppData>,
) -> anyhow::Result<()> {
    info!("Received github_app_authorization event");

    let event = match context.event() {
        Some(event) => event,
        None => {
            warn!("No event found in context");
            return Ok(());
        }
    };

    let repository = match &event.repository {
        Some(repository) => repository,
        None => {
            warn!("No repository found in event");
            return Ok(());
        }
    };

    let full_name = match &repository.full_name {
        Some(full_name) => full_name,
        None => {
            warn!("No full name found in repository");
            return Ok(());
        }
    };

    info!("Authorization revoked from {full_name}");

    Ok(())
}
