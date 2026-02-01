use octofer::github::Repository;
use octofer::github::commits::GithubCommitStatus;
use octofer::github::pulls::PullRequest;
use octofer::octocrab::Octocrab;
use serde_json::json;
use tracing::info;

#[derive(Debug, PartialEq, Eq)]
pub enum FastForwardStatus {
    Mergeable,
    Merged,
    NotMergeable,
}

pub trait OctocrabExt {
    async fn fast_forward_merge(
        &self,
        repository: &Repository,
        pr: &PullRequest,
    ) -> anyhow::Result<()>;

    async fn fast_forward_mergeable(
        &self,
        owner: impl Into<String>,
        repo: impl Into<String>,
        pr: &PullRequest,
    ) -> anyhow::Result<FastForwardStatus>;
}

impl OctocrabExt for Octocrab {
    async fn fast_forward_merge(
        &self,
        repository: &Repository,
        pr: &PullRequest,
    ) -> anyhow::Result<()> {
        let repo = &repository.name;
        let owner = &repository.owner.as_ref().unwrap().login;
        let branch = pr.base.ref_field.as_str();
        let endpoint = format!("/repos/{owner}/{repo}/git/refs/heads/{branch}");
        let sha = pr.head.sha.as_str();
        let _: serde_json::Value = self
            .patch(
                endpoint,
                Some(&json!({
                    "sha": sha,
                    "force": false
                })),
            )
            .await?;
        Ok(())
    }

    async fn fast_forward_mergeable(
        &self,
        owner: impl Into<String>,
        repo: impl Into<String>,
        pr: &PullRequest,
    ) -> anyhow::Result<FastForwardStatus> {
        let base = &pr.base.ref_field;
        let head = &pr.head.ref_field;
        info!("Checking if {head} is fast-forward mergeable into {base}");
        let res = self.commits(owner, repo).compare(base, head).send().await?;
        match res.status {
            GithubCommitStatus::Ahead => Ok(FastForwardStatus::Mergeable),
            GithubCommitStatus::Behind => Ok(FastForwardStatus::Merged),
            GithubCommitStatus::Identical => Ok(FastForwardStatus::Merged),
            _ => Ok(FastForwardStatus::NotMergeable),
        }
    }
}
