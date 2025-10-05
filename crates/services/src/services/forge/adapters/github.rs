use async_trait::async_trait;
use regex::Regex;

use crate::services::{
    forge::{
        error::ForgeError, CreatePrRequest, ForgeService, ForgeType, RepoInfo,
    },
    github_service::GitHubService as LegacyGitHubService,
};

#[cfg(feature = "cloud")]
use crate::services::forge::RepositoryInfo;
use db::models::merge::PullRequestInfo;

/// GitHub adapter implementing the ForgeService trait
pub struct GitHubAdapter {
    service: LegacyGitHubService,
}

impl GitHubAdapter {
    pub fn new(token: &str) -> Result<Self, ForgeError> {
        let service = LegacyGitHubService::new(token)
            .map_err(|e| ForgeError::HttpClient(e.to_string()))?;
        Ok(Self { service })
    }
}

#[async_trait]
impl ForgeService for GitHubAdapter {
    async fn check_token(&self) -> Result<(), ForgeError> {
        self.service
            .check_token()
            .await
            .map_err(|e| ForgeError::from(e))
    }

    async fn create_pr(
        &self,
        repo_info: &RepoInfo,
        request: &CreatePrRequest,
    ) -> Result<PullRequestInfo, ForgeError> {
        // Convert RepoInfo to GitHubRepoInfo
        let gh_repo_info = crate::services::github_service::GitHubRepoInfo {
            owner: repo_info.owner.clone(),
            repo_name: repo_info.repo_name.clone(),
        };

        // Convert CreatePrRequest to GitHub's format
        let gh_request = crate::services::github_service::CreatePrRequest {
            title: request.title.clone(),
            body: request.body.clone(),
            head_branch: request.head_branch.clone(),
            base_branch: request.base_branch.clone(),
        };

        self.service
            .create_pr(&gh_repo_info, &gh_request)
            .await
            .map_err(|e| ForgeError::from(e))
    }

    async fn get_pr_status(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<PullRequestInfo, ForgeError> {
        let gh_repo_info = crate::services::github_service::GitHubRepoInfo {
            owner: repo_info.owner.clone(),
            repo_name: repo_info.repo_name.clone(),
        };

        self.service
            .update_pr_status(&gh_repo_info, pr_number)
            .await
            .map_err(|e| ForgeError::from(e))
    }

    #[cfg(feature = "cloud")]
    async fn list_repositories(&self, page: u8) -> Result<Vec<RepositoryInfo>, ForgeError> {
        self.service
            .list_repositories(page)
            .await
            .map_err(|e| ForgeError::from(e))
    }

    fn parse_remote_url(&self, url: &str) -> Result<RepoInfo, ForgeError> {
        // Supports SSH, HTTPS and PR GitHub URLs
        let re = Regex::new(r"github\.com[:/](?P<owner>[^/]+)/(?P<repo>[^/]+?)(?:\.git)?(?:/|$)")
            .map_err(|e| ForgeError::Repository(format!("Failed to compile regex: {e}")))?;

        let caps = re.captures(url).ok_or_else(|| {
            ForgeError::Repository(format!("Invalid GitHub URL format: {url}"))
        })?;

        Ok(RepoInfo::new(
            ForgeType::GitHub,
            caps.name("owner").unwrap().as_str().to_string(),
            caps.name("repo").unwrap().as_str().to_string(),
            None, // GitHub always uses default base URL
        ))
    }

    fn forge_type(&self) -> ForgeType {
        ForgeType::GitHub
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_https_url() {
        let adapter = GitHubAdapter::new("fake_token").unwrap();
        let info = adapter
            .parse_remote_url("https://github.com/owner/repo.git")
            .unwrap();
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo_name, "repo");
        assert_eq!(info.forge_type, ForgeType::GitHub);
    }

    #[test]
    fn test_parse_ssh_url() {
        let adapter = GitHubAdapter::new("fake_token").unwrap();
        let info = adapter
            .parse_remote_url("git@github.com:owner/repo.git")
            .unwrap();
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo_name, "repo");
    }
}
