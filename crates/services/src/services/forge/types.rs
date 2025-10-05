use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Supported git forge platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(use_ts_enum)]
pub enum ForgeType {
    GitHub,
    Forgejo,
    Tangled,
}

impl std::fmt::Display for ForgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForgeType::GitHub => write!(f, "GitHub"),
            ForgeType::Forgejo => write!(f, "Forgejo"),
            ForgeType::Tangled => write!(f, "Tangled"),
        }
    }
}

/// Repository information extracted from a remote URL
#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub forge_type: ForgeType,
    pub owner: String,
    pub repo_name: String,
    /// Base URL for self-hosted instances (e.g., "https://git.example.com")
    /// None for default public instances
    pub base_url: Option<String>,
}

impl RepoInfo {
    pub fn new(
        forge_type: ForgeType,
        owner: String,
        repo_name: String,
        base_url: Option<String>,
    ) -> Self {
        Self {
            forge_type,
            owner,
            repo_name,
            base_url,
        }
    }

    /// Get the base URL for API calls
    pub fn api_base_url(&self) -> String {
        match &self.base_url {
            Some(url) => url.clone(),
            None => match self.forge_type {
                ForgeType::GitHub => "https://api.github.com".to_string(),
                ForgeType::Forgejo => "https://codeberg.org".to_string(),
                ForgeType::Tangled => "https://tangled.org".to_string(),
            },
        }
    }
}

/// Configuration for a git forge account
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ForgeConfig {
    pub forge_type: ForgeType,
    pub pat: Option<String>,
    pub oauth_token: Option<String>,
    pub username: Option<String>,
    pub primary_email: Option<String>,
    pub default_pr_base: Option<String>,
    /// Base URL for self-hosted instances
    pub base_url: Option<String>,
}

impl ForgeConfig {
    pub fn token(&self) -> Option<String> {
        self.pat
            .as_deref()
            .or(self.oauth_token.as_deref())
            .map(|s| s.to_string())
    }
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            forge_type: ForgeType::GitHub,
            pat: None,
            oauth_token: None,
            username: None,
            primary_email: None,
            default_pr_base: Some("main".to_string()),
            base_url: None,
        }
    }
}

/// Request to create a pull request
#[derive(Debug, Clone)]
pub struct CreatePrRequest {
    pub title: String,
    pub body: Option<String>,
    pub head_branch: String,
    pub base_branch: String,
}

/// Repository information returned from forge APIs
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct RepositoryInfo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub owner: String,
    pub description: Option<String>,
    pub clone_url: String,
    pub ssh_url: String,
    pub default_branch: String,
    pub private: bool,
}
