use async_trait::async_trait;
use backon::{ExponentialBuilder, Retryable};
use chrono::{DateTime, Utc};
use db::models::merge::{MergeStatus, PullRequestInfo};
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tracing::info;

use crate::services::forge::{
    error::ForgeError, CreatePrRequest, ForgeService, ForgeType, RepoInfo,
};

#[cfg(feature = "cloud")]
use crate::services::forge::RepositoryInfo;

/// Forgejo/Gitea adapter implementing the ForgeService trait
/// 
/// Uses the Gitea/Forgejo REST API v1 which is compatible between both platforms
pub struct ForgejoAdapter {
    client: Client,
    base_url: String,
    token: String,
}

impl ForgejoAdapter {
    pub fn new(token: &str, base_url: &str) -> Result<Self, ForgeError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ForgeError::HttpClient(e.to_string()))?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
        })
    }

    /// Make an authenticated request to the Forgejo API
    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<T, ForgeError> {
        let url = format!("{}{}", self.base_url, path);
        
        let mut request = self
            .client
            .request(method, &url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/json");

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(match status {
                401 => ForgeError::TokenInvalid,
                403 => ForgeError::InsufficientPermissions,
                404 => ForgeError::RepoNotFoundOrNoAccess,
                _ => ForgeError::HttpClient(format!("HTTP {}: {}", status, text)),
            });
        }

        response.json().await.map_err(|e| {
            ForgeError::HttpClient(format!("Failed to parse response: {}", e))
        })
    }
}

#[async_trait]
impl ForgeService for ForgejoAdapter {
    async fn check_token(&self) -> Result<(), ForgeError> {
        // Use GET /api/v1/user to validate token
        let _user: ForgejoUser = self
            .request(reqwest::Method::GET, "/api/v1/user", None)
            .await?;
        Ok(())
    }

    async fn create_pr(
        &self,
        repo_info: &RepoInfo,
        request: &CreatePrRequest,
    ) -> Result<PullRequestInfo, ForgeError> {
        (|| async { self.create_pr_internal(repo_info, request).await })
            .retry(
                &ExponentialBuilder::default()
                    .with_min_delay(Duration::from_secs(1))
                    .with_max_delay(Duration::from_secs(30))
                    .with_max_times(3)
                    .with_jitter(),
            )
            .when(|e| e.should_retry())
            .notify(|err: &ForgeError, dur: Duration| {
                tracing::warn!(
                    "Forgejo API call failed, retrying after {:.2}s: {}",
                    dur.as_secs_f64(),
                    err
                );
            })
            .await
    }

    async fn get_pr_status(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<PullRequestInfo, ForgeError> {
        (|| async {
            let path = format!(
                "/api/v1/repos/{}/{}/pulls/{}",
                repo_info.owner, repo_info.repo_name, pr_number
            );

            let pr: ForgejoPullRequest = self.request(reqwest::Method::GET, &path, None).await?;

            Ok(Self::map_pull_request(pr))
        })
        .retry(
            &ExponentialBuilder::default()
                .with_min_delay(Duration::from_secs(1))
                .with_max_delay(Duration::from_secs(30))
                .with_max_times(3)
                .with_jitter(),
        )
        .when(|err: &ForgeError| err.should_retry())
        .notify(|err: &ForgeError, dur: Duration| {
            tracing::warn!(
                "Forgejo API call failed, retrying after {:.2}s: {}",
                dur.as_secs_f64(),
                err
            );
        })
        .await
    }

    #[cfg(feature = "cloud")]
    async fn list_repositories(&self, page: u8) -> Result<Vec<RepositoryInfo>, ForgeError> {
        let path = format!("/api/v1/user/repos?page={}&limit=50", page);
        let repos: Vec<ForgejoRepository> =
            self.request(reqwest::Method::GET, &path, None).await?;

        let repositories: Vec<RepositoryInfo> = repos
            .into_iter()
            .map(|repo| RepositoryInfo {
                id: repo.id,
                name: repo.name,
                full_name: repo.full_name,
                owner: repo.owner.map(|o| o.login).unwrap_or_default(),
                description: repo.description,
                clone_url: repo.clone_url,
                ssh_url: repo.ssh_url,
                default_branch: repo.default_branch.unwrap_or_else(|| "main".to_string()),
                private: repo.private.unwrap_or(false),
            })
            .collect();

        info!(
            "Retrieved {} repositories from Forgejo (page {})",
            repositories.len(),
            page
        );

        Ok(repositories)
    }

    fn parse_remote_url(&self, url: &str) -> Result<RepoInfo, ForgeError> {
        // Parse URLs like:
        // - https://codeberg.org/owner/repo.git
        // - git@codeberg.org:owner/repo.git
        // - https://git.example.com/owner/repo.git
        
        let re = Regex::new(r"(?:https?://|git@)([^/:]+)[:/](?P<owner>[^/]+)/(?P<repo>[^/]+?)(?:\.git)?(?:/|$)")
            .map_err(|e| ForgeError::Repository(format!("Failed to compile regex: {e}")))?;

        let caps = re.captures(url).ok_or_else(|| {
            ForgeError::Repository(format!("Invalid Forgejo URL format: {url}"))
        })?;

        let host = caps.get(1).unwrap().as_str();
        
        // Determine if this is a self-hosted instance or Codeberg
        let base_url = if host == "codeberg.org" {
            None
        } else {
            Some(format!("https://{}", host))
        };

        Ok(RepoInfo::new(
            ForgeType::Forgejo,
            caps.name("owner").unwrap().as_str().to_string(),
            caps.name("repo").unwrap().as_str().to_string(),
            base_url,
        ))
    }

    fn forge_type(&self) -> ForgeType {
        ForgeType::Forgejo
    }
}

impl ForgejoAdapter {
    async fn create_pr_internal(
        &self,
        repo_info: &RepoInfo,
        request: &CreatePrRequest,
    ) -> Result<PullRequestInfo, ForgeError> {
        let path = format!(
            "/api/v1/repos/{}/{}/pulls",
            repo_info.owner, repo_info.repo_name
        );

        let body = serde_json::json!({
            "title": request.title,
            "body": request.body.as_deref().unwrap_or(""),
            "head": request.head_branch,
            "base": request.base_branch,
        });

        let pr: ForgejoPullRequest = self.request(reqwest::Method::POST, &path, Some(body)).await
            .map_err(|e| match e {
                ForgeError::HttpClient(msg) => ForgeError::PullRequest(format!(
                    "Failed to create PR for '{} -> {}': {}",
                    request.head_branch, request.base_branch, msg
                )),
                other => other,
            })?;

        info!(
            "Created Forgejo PR #{} for branch {} in {}/{}",
            pr.number, request.head_branch, repo_info.owner, repo_info.repo_name
        );

        Ok(Self::map_pull_request(pr))
    }

    fn map_pull_request(pr: ForgejoPullRequest) -> PullRequestInfo {
        let state = match pr.state.as_deref() {
            Some("open") => MergeStatus::Open,
            Some("closed") => {
                if pr.merged {
                    MergeStatus::Merged
                } else {
                    MergeStatus::Closed
                }
            }
            _ => MergeStatus::Unknown,
        };

        PullRequestInfo {
            number: pr.number,
            url: pr.html_url.unwrap_or_default(),
            status: state,
            merged_at: pr.merged_at,
            merge_commit_sha: pr.merge_commit_sha,
        }
    }
}

// Forgejo API response types
#[derive(Debug, Deserialize)]
struct ForgejoUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct ForgejoPullRequest {
    number: i64,
    state: Option<String>,
    html_url: Option<String>,
    merged: bool,
    merged_at: Option<DateTime<Utc>>,
    merge_commit_sha: Option<String>,
}

#[cfg(feature = "cloud")]
#[derive(Debug, Deserialize)]
struct ForgejoRepository {
    id: i64,
    name: String,
    full_name: String,
    owner: Option<ForgejoUser>,
    description: Option<String>,
    clone_url: String,
    ssh_url: String,
    default_branch: Option<String>,
    private: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_codeberg_https() {
        let adapter = ForgejoAdapter::new("fake_token", "https://codeberg.org").unwrap();
        let info = adapter
            .parse_remote_url("https://codeberg.org/owner/repo.git")
            .unwrap();
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo_name, "repo");
        assert_eq!(info.forge_type, ForgeType::Forgejo);
        assert_eq!(info.base_url, None);
    }

    #[test]
    fn test_parse_codeberg_ssh() {
        let adapter = ForgejoAdapter::new("fake_token", "https://codeberg.org").unwrap();
        let info = adapter
            .parse_remote_url("git@codeberg.org:owner/repo.git")
            .unwrap();
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo_name, "repo");
    }

    #[test]
    fn test_parse_self_hosted() {
        let adapter = ForgejoAdapter::new("fake_token", "https://git.example.com").unwrap();
        let info = adapter
            .parse_remote_url("https://git.example.com/owner/repo.git")
            .unwrap();
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo_name, "repo");
        assert_eq!(info.base_url, Some("https://git.example.com".to_string()));
    }
}
