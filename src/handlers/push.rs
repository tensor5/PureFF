use anyhow::Context as AnyhowContext;
use octofer::github::webhook_events::WebhookEventPayload;
use octofer::{Context, octocrab::params::State};
use std::sync::Arc;
use tracing::{info, warn};

use crate::AppData;
use crate::messages::{NOT_MERGEABLE, UNCHECKED};
use crate::octocrab_ext::{FastForwardStatus, OctocrabExt};

pub async fn push_handler(context: Context, extra: Arc<AppData>) -> anyhow::Result<()> {
    info!("Received push event");

    let event = match context.event() {
        Some(event) => event,
        None => {
            warn!("No event found in context");
            return Ok(());
        }
    };

    let payload = match &event.specific {
        WebhookEventPayload::Push(payload) => payload,
        _ => return Ok(()), // Not a push event
    };

    let push_ref = payload.r#ref.as_str();

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

    info!("Push to: {owner}/{name}:{push_ref}");

    let client = match context.installation_client().await? {
        Some(client) => client,
        None => {
            info!(
                "No installation client available for push to: {} by {}",
                name, owner
            );
            return Ok(());
        }
    };

    // Fetch all open PRs matching the pushed ref (as base or head)
    let pulls_matching_base_ref = client
        .pulls(owner, name)
        .list()
        .state(State::Open)
        .base(push_ref)
        .per_page(100)
        .send()
        .await?;

    let pulls_matching_head_ref = client
        .pulls(owner, name)
        .list()
        .state(State::Open)
        .head(format!("{owner}:{push_ref}"))
        .per_page(100)
        .send()
        .await?;

    // Process all matching PRs
    for pr in pulls_matching_base_ref
        .into_iter()
        .chain(pulls_matching_head_ref)
    {
        info!("Processing pull request: {}", pr.number);

        let is_mergeable = client.fast_forward_mergeable(owner, name, &pr).await?;
        let issue_handler = client.issues_by_id(repo.id);

        let comments = issue_handler
            .list_comments(pr.number)
            .per_page(100)
            .send()
            .await?;

        // Find and update bot comments if needed
        for comment in comments {
            if comment.user.id != extra.bot_user_id {
                continue;
            }

            let comment_body = match &comment.body {
                Some(body) => body.as_str(),
                None => continue,
            };

            let needs_update = (is_mergeable == FastForwardStatus::NotMergeable
                && comment_body != NOT_MERGEABLE)
                || (is_mergeable == FastForwardStatus::Mergeable && comment_body != UNCHECKED);

            if needs_update {
                let new_body = if is_mergeable == FastForwardStatus::NotMergeable {
                    NOT_MERGEABLE
                } else {
                    UNCHECKED
                };
                issue_handler.update_comment(comment.id, new_body).await?;
                info!(
                    "Updated comment {} on PR {} with: {}",
                    comment.id, pr.number, new_body
                );
            }
        }
    }

    Ok(())
}
