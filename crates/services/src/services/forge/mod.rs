pub mod adapters;
pub mod error;
pub mod factory;
pub mod types;

use async_trait::async_trait;
use db::models::merge::PullRequestInfo;

pub use error::ForgeError;
pub use factory::ForgeFactory;
pub use types::{CreatePrRequest, ForgeConfig, ForgeType, RepoInfo, RepositoryInfo};

/// Core trait for git forge operations
/// 
/// This trait defines the interface for interacting with different git forges
/// (GitHub, Forgejo, Tangled, etc.) in a uniform way.
#[async_trait]
pub trait ForgeService: Send + Sync {
    /// Verify that the authentication token is valid
    async fn check_token(&self) -> Result<(), ForgeError>;

    /// Create a pull request / merge request
    async fn create_pr(
        &self,
        repo_info: &RepoInfo,
        request: &CreatePrRequest,
    ) -> Result<PullRequestInfo, ForgeError>;

    /// Get the current status of a pull request
    async fn get_pr_status(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<PullRequestInfo, ForgeError>;

    /// List repositories accessible to the authenticated user
    /// 
    /// Note: This is primarily used for cloud features
    #[cfg(feature = "cloud")]
    async fn list_repositories(&self, page: u8) -> Result<Vec<RepositoryInfo>, ForgeError>;

    /// Parse a git remote URL to extract repository information
    fn parse_remote_url(&self, url: &str) -> Result<RepoInfo, ForgeError>;

    /// Get the type of forge this service represents
    fn forge_type(&self) -> ForgeType;
}
