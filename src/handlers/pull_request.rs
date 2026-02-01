use anyhow::Context as AnyhowContext;
use octofer::Context;
use octofer::github::webhook_events::WebhookEventPayload;
use octofer::github::webhook_events::payload::PullRequestWebhookEventAction;
use std::sync::Arc;
use tracing::{info, warn};

use crate::AppData;
use crate::messages::{NOT_MERGEABLE, UNCHECKED};
use crate::octocrab_ext::{FastForwardStatus, OctocrabExt};

pub async fn pull_request_handler(context: Context, _: Arc<AppData>) -> anyhow::Result<()> {
    info!("Received pull request event");

    let event = match context.event() {
        Some(event) => event,
        None => {
            warn!("No event found in context");
            return Ok(());
        }
    };

    let payload = match &event.specific {
        WebhookEventPayload::PullRequest(payload) => payload,
        _ => {
            // Not a PR event, nothing to do
            return Ok(());
        }
    };

    let repo = event
        .repository
        .as_ref()
        .context("Repository information missing from event")?;

    let name = repo.name.as_str();
    let owner = repo
        .owner
        .as_ref()
        .context("Owner information missing from repository")?
        .login
        .as_str();

    info!(
        "PR title: {}",
        payload
            .pull_request
            .title
            .as_deref()
            .unwrap_or("(no title)")
    );

    if !matches!(payload.action, PullRequestWebhookEventAction::Opened) {
        // Only handle opened PRs
        return Ok(());
    }

    let pr = &payload.pull_request;
    let pr_number = pr.number;
    let repo_id = event
        .repository
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Repository information missing from event"))?
        .id;

    let client = match context.installation_client().await? {
        Some(client) => client,
        None => {
            warn!("No installation client available");
            return Ok(());
        }
    };

    let comment_body = match client.fast_forward_mergeable(owner, name, pr).await? {
        FastForwardStatus::Mergeable => UNCHECKED,

        FastForwardStatus::Merged => UNCHECKED,

        FastForwardStatus::NotMergeable => NOT_MERGEABLE,
    };

    client
        .issues_by_id(repo_id)
        .create_comment(pr_number, comment_body)
        .await?;

    Ok(())
}
