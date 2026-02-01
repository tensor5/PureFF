use anyhow::{Context as AnyhowContext, Result};
use octofer::Context;
use octofer::github::pulls::PullRequest;
use octofer::github::webhook_events::WebhookEventPayload;
use octofer::github::webhook_events::payload::IssueCommentWebhookEventAction;
use octofer::github::{CommentId, IssueState, Repository, UserId};
use octofer::octocrab::Octocrab;
use std::sync::Arc;
use tracing::{info, warn};

use crate::AppData;
use crate::messages::{CHECKED, MERGED, NOT_MERGEABLE, UNCHECKED};
use crate::octocrab_ext::{FastForwardStatus, OctocrabExt};

/// Handles issue comment webhook events
pub async fn issue_comment_handler(context: Context, extra: Arc<AppData>) -> Result<()> {
    info!("Received issue comment event");

    let event = match context.event() {
        Some(event) => event,
        None => {
            warn!("No event found in context");
            return Ok(());
        }
    };

    let payload = match &event.specific {
        WebhookEventPayload::IssueComment(payload) => payload,
        _ => return Ok(()),
    };

    if !matches!(payload.action, IssueCommentWebhookEventAction::Edited) {
        return Ok(());
    }

    // Only process comments from the bot on open PRs
    if !is_bot_comment_on_open_pr(
        payload.comment.user.id,
        payload.issue.pull_request.as_ref(),
        &payload.issue.state,
        extra.bot_user_id,
    ) {
        return Ok(());
    }

    let repository = event
        .repository
        .as_ref()
        .context("Repository information missing from event")?;

    let body = payload.comment.body.as_deref().unwrap_or_default();
    let comment_id = payload.comment.id;
    let pr_number = payload.issue.number;
    let pr_url = payload.issue.html_url.as_str();

    info!("Processing bot comment: {}", body);

    let repo_name = repository.name.as_str();
    let owner = repository
        .owner
        .as_ref()
        .context("Owner information missing from repository")?
        .login
        .as_str();

    let client = match context.installation_client().await? {
        Some(client) => client,
        None => {
            info!("No installation client available for {owner}/{repo_name}");
            return Ok(());
        }
    };

    let pr = client
        .pulls(owner, repo_name)
        .get(pr_number)
        .await
        .context("Failed to fetch pull request")?;

    let is_mergeable = client
        .fast_forward_mergeable(owner, repo_name, &pr)
        .await
        .context("Failed to check if PR is mergeable")?;

    match is_mergeable {
        FastForwardStatus::NotMergeable => match body {
            NOT_MERGEABLE => {}
            _ => {
                info!("PR #{} is not mergeable, updating comment", pr_number);
                update_comment(&client, repository, comment_id, NOT_MERGEABLE).await?;
            }
        },

        FastForwardStatus::Mergeable => {
            match body {
                CHECKED => {
                    handle_merge_request(&client, repository, &pr, comment_id, pr_number, pr_url)
                        .await?;
                }
                UNCHECKED => {
                    // Comment is already in the correct state
                }
                _ => {
                    // Restore the original message if it has been modified
                    info!("Restoring comment {} to unchecked state", comment_id.0);
                    update_comment(&client, repository, comment_id, UNCHECKED).await?;
                }
            }
        }

        FastForwardStatus::Merged => {}
    }

    Ok(())
}

/// Checks if the comment is from the bot and the issue is an open PR
fn is_bot_comment_on_open_pr(
    comment_user_id: UserId,
    pull_request: Option<&octofer::github::issues::PullRequestLink>,
    issue_state: &IssueState,
    bot_user_id: UserId,
) -> bool {
    comment_user_id == bot_user_id && pull_request.is_some() && *issue_state == IssueState::Open
}

/// Handles the merge request when the bot comment is checked
async fn handle_merge_request(
    client: &Octocrab,
    repository: &Repository,
    pr: &PullRequest,
    comment_id: CommentId,
    pr_number: u64,
    pr_url: &str,
) -> Result<()> {
    info!("Fast-forward merging PR #{} ({})", pr_number, pr_url);

    client
        .fast_forward_merge(repository, pr)
        .await
        .context("Failed to fast-forward merge PR")?;

    info!("Successfully merged PR #{}", pr_number);

    update_comment(client, repository, comment_id, MERGED).await?;

    Ok(())
}

/// Updates a comment with the given body
async fn update_comment(
    client: &Octocrab,
    repository: &Repository,
    comment_id: CommentId,
    body: &str,
) -> Result<()> {
    client
        .issues_by_id(repository.id)
        .update_comment(comment_id, body)
        .await
        .context("Failed to update comment")?;

    Ok(())
}
