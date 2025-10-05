use regex::Regex;

use super::{
    adapters::{forgejo::ForgejoAdapter, github::GitHubAdapter},
    error::ForgeError,
    types::{ForgeConfig, ForgeType, RepoInfo},
    ForgeService,
};

pub struct ForgeFactory;

impl ForgeFactory {
    /// Create a forge service from configuration
    pub fn create(config: &ForgeConfig) -> Result<Box<dyn ForgeService>, ForgeError> {
        let token = config.token().ok_or(ForgeError::NoToken)?;

        match config.forge_type {
            ForgeType::GitHub => Ok(Box::new(GitHubAdapter::new(&token)?)),
            ForgeType::Forgejo => {
                let base_url = config
                    .base_url
                    .as_deref()
                    .unwrap_or("https://codeberg.org");
                Ok(Box::new(ForgejoAdapter::new(&token, base_url)?))
            }
            ForgeType::Tangled => {
                // TODO: Implement Tangled adapter when needed
                Err(ForgeError::HttpClient(
                    "Tangled support not yet implemented".to_string(),
                ))
            }
        }
    }

    /// Create forge service and parse repo info from a remote URL
    /// 
    /// This will detect the forge type from the URL and find matching config
    pub fn from_remote_url(
        url: &str,
        configs: &[ForgeConfig],
    ) -> Result<(Box<dyn ForgeService>, RepoInfo), ForgeError> {
        let (forge_type, base_url) = Self::detect_forge_type(url)?;

        // Find matching config for this forge type and base URL
        let config = configs
            .iter()
            .find(|c| {
                c.forge_type == forge_type
                    && match (&c.base_url, &base_url) {
                        (Some(a), Some(b)) => a == b,
                        (None, None) => true,
                        _ => false,
                    }
            })
            .ok_or(ForgeError::NoConfigForForge(forge_type))?;

        let service = Self::create(config)?;
        let repo_info = service.parse_remote_url(url)?;

        Ok((service, repo_info))
    }

    /// Detect forge type and base URL from a remote URL
    fn detect_forge_type(url: &str) -> Result<(ForgeType, Option<String>), ForgeError> {
        // GitHub patterns
        if url.contains("github.com") {
            return Ok((ForgeType::GitHub, None));
        }

        // Codeberg (public Forgejo instance)
        if url.contains("codeberg.org") {
            return Ok((ForgeType::Forgejo, None));
        }

        // Tangled
        if url.contains("tangled.org") {
            return Ok((ForgeType::Tangled, None));
        }

        // Try to detect self-hosted Forgejo/Gitea
        // These typically have patterns like: https://git.example.com/owner/repo
        // or git@git.example.com:owner/repo.git
        let self_hosted_re = Regex::new(
            r"(?:https?://|git@)([^/:]+)[:/]",
        )
        .map_err(|e| ForgeError::Repository(format!("Invalid regex: {}", e)))?;

        if let Some(caps) = self_hosted_re.captures(url) {
            let host = caps.get(1).unwrap().as_str();
            // Common patterns for self-hosted Forgejo/Gitea
            if host.starts_with("git.") 
                || host.contains("gitea") 
                || host.contains("forgejo")
                || url.contains("/api/v1/") // Gitea/Forgejo API pattern
            {
                let base_url = format!("https://{}", host);
                return Ok((ForgeType::Forgejo, Some(base_url)));
            }
        }

        Err(ForgeError::Repository(format!(
            "Could not detect forge type from URL: {}",
            url
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_github() {
        let (forge_type, base_url) =
            ForgeFactory::detect_forge_type("https://github.com/owner/repo.git").unwrap();
        assert_eq!(forge_type, ForgeType::GitHub);
        assert_eq!(base_url, None);

        let (forge_type, base_url) =
            ForgeFactory::detect_forge_type("git@github.com:owner/repo.git").unwrap();
        assert_eq!(forge_type, ForgeType::GitHub);
        assert_eq!(base_url, None);
    }

    #[test]
    fn test_detect_codeberg() {
        let (forge_type, base_url) =
            ForgeFactory::detect_forge_type("https://codeberg.org/owner/repo.git").unwrap();
        assert_eq!(forge_type, ForgeType::Forgejo);
        assert_eq!(base_url, None);
    }

    #[test]
    fn test_detect_self_hosted_forgejo() {
        let (forge_type, base_url) =
            ForgeFactory::detect_forge_type("https://git.example.com/owner/repo.git").unwrap();
        assert_eq!(forge_type, ForgeType::Forgejo);
        assert_eq!(base_url, Some("https://git.example.com".to_string()));
    }

    #[test]
    fn test_detect_tangled() {
        let (forge_type, base_url) =
            ForgeFactory::detect_forge_type("https://tangled.org/@owner/repo").unwrap();
        assert_eq!(forge_type, ForgeType::Tangled);
        assert_eq!(base_url, None);
    }
}
