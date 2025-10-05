use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use super::ForgeType;
use crate::services::{git::GitServiceError, git_cli::GitCliError};

#[derive(Debug, Error, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(use_ts_enum)]
pub enum ForgeError {
    #[ts(skip)]
    #[error("Repository error: {0}")]
    Repository(String),
    
    #[ts(skip)]
    #[error("Pull request error: {0}")]
    PullRequest(String),
    
    #[ts(skip)]
    #[error("Branch error: {0}")]
    Branch(String),
    
    #[error("Authentication token is invalid or expired")]
    TokenInvalid,
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    
    #[error("Repository not found or no access")]
    RepoNotFoundOrNoAccess,
    
    #[error("No authentication token configured")]
    NoToken,
    
    #[ts(skip)]
    #[error("No configuration found for forge type: {0}")]
    NoConfigForForge(ForgeType),
    
    #[ts(skip)]
    #[error("HTTP client error: {0}")]
    HttpClient(String),
    
    #[ts(skip)]
    #[serde(skip)]
    #[error(transparent)]
    GitService(GitServiceError),
}

impl ForgeError {
    pub fn is_api_data(&self) -> bool {
        matches!(
            self,
            ForgeError::TokenInvalid
                | ForgeError::InsufficientPermissions
                | ForgeError::RepoNotFoundOrNoAccess
                | ForgeError::NoToken
        )
    }

    pub fn should_retry(&self) -> bool {
        !self.is_api_data()
    }
}

// Convert from octocrab errors (for GitHub adapter)
impl From<octocrab::Error> for ForgeError {
    fn from(err: octocrab::Error) -> Self {
        match &err {
            octocrab::Error::GitHub { source, .. } => {
                let status = source.status_code.as_u16();
                let msg = source.message.to_ascii_lowercase();
                if status == 401 || msg.contains("bad credentials") || msg.contains("token expired")
                {
                    ForgeError::TokenInvalid
                } else if status == 403 {
                    ForgeError::InsufficientPermissions
                } else {
                    ForgeError::HttpClient(err.to_string())
                }
            }
            _ => ForgeError::HttpClient(err.to_string()),
        }
    }
}

// Convert from reqwest errors (for Forgejo/HTTP-based adapters)
impl From<reqwest::Error> for ForgeError {
    fn from(err: reqwest::Error) -> Self {
        if let Some(status) = err.status() {
            match status.as_u16() {
                401 => ForgeError::TokenInvalid,
                403 => ForgeError::InsufficientPermissions,
                404 => ForgeError::RepoNotFoundOrNoAccess,
                _ => ForgeError::HttpClient(err.to_string()),
            }
        } else {
            ForgeError::HttpClient(err.to_string())
        }
    }
}

// Convert from GitHubServiceError
impl From<crate::services::github_service::GitHubServiceError> for ForgeError {
    fn from(error: crate::services::github_service::GitHubServiceError) -> Self {
        use crate::services::github_service::GitHubServiceError;
        match error {
            GitHubServiceError::TokenInvalid => ForgeError::TokenInvalid,
            GitHubServiceError::InsufficientPermissions => ForgeError::InsufficientPermissions,
            GitHubServiceError::RepoNotFoundOrNoAccess => ForgeError::RepoNotFoundOrNoAccess,
            GitHubServiceError::Client(e) => ForgeError::HttpClient(e.to_string()),
            GitHubServiceError::Repository(msg) => ForgeError::Repository(msg),
            GitHubServiceError::PullRequest(msg) => ForgeError::PullRequest(msg),
            GitHubServiceError::Branch(msg) => ForgeError::Branch(msg),
            GitHubServiceError::GitService(e) => ForgeError::GitService(e),
        }
    }
}

impl From<GitServiceError> for ForgeError {
    fn from(error: GitServiceError) -> Self {
        match error {
            GitServiceError::GitCLI(GitCliError::AuthFailed(_)) => Self::TokenInvalid,
            GitServiceError::GitCLI(GitCliError::CommandFailed(msg)) => {
                let lower = msg.to_ascii_lowercase();
                if lower.contains("the requested url returned error: 403") {
                    Self::InsufficientPermissions
                } else if lower.contains("the requested url returned error: 404") {
                    Self::RepoNotFoundOrNoAccess
                } else {
                    Self::GitService(GitServiceError::GitCLI(GitCliError::CommandFailed(msg)))
                }
            }
            other => Self::GitService(other),
        }
    }
}
