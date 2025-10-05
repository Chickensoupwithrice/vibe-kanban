use anyhow::Error;
use executors::{executors::BaseCodingAgent, profile::ExecutorProfileId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::services::{
    config::versions::v7,
    forge::types::{ForgeConfig, ForgeType},
};

pub use v7::{EditorConfig, EditorType, NotificationConfig, SoundFile, ThemeMode, UiLanguage};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Config {
    pub config_version: String,
    pub theme: ThemeMode,
    pub executor_profile: ExecutorProfileId,
    pub disclaimer_acknowledged: bool,
    pub onboarding_acknowledged: bool,
    pub github_login_acknowledged: bool,
    pub telemetry_acknowledged: bool,
    pub notifications: NotificationConfig,
    pub editor: EditorConfig,
    /// Multiple forge configurations (GitHub, Forgejo, etc.)
    pub forges: Vec<ForgeConfig>,
    pub analytics_enabled: Option<bool>,
    pub workspace_dir: Option<String>,
    pub last_app_version: Option<String>,
    pub show_release_notes: bool,
    #[serde(default)]
    pub language: UiLanguage,
}

impl Config {
    pub fn from_previous_version(raw_config: &str) -> Result<Self, Error> {
        let old_config = match serde_json::from_str::<v7::Config>(raw_config) {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::error!("❌ Failed to parse config: {}", e);
                tracing::error!("   at line {}, column {}", e.line(), e.column());
                return Err(e.into());
            }
        };

        // Migrate single GitHub config to multi-forge config
        let forges = vec![ForgeConfig {
            forge_type: ForgeType::GitHub,
            pat: old_config.github.pat.clone(),
            oauth_token: old_config.github.oauth_token.clone(),
            username: old_config.github.username.clone(),
            primary_email: old_config.github.primary_email.clone(),
            default_pr_base: old_config.github.default_pr_base.clone(),
            base_url: None, // GitHub always uses default URL
        }];

        tracing::info!("Migrated GitHub config to multi-forge configuration");

        Ok(Self {
            config_version: "v8".to_string(),
            theme: old_config.theme,
            executor_profile: old_config.executor_profile,
            disclaimer_acknowledged: old_config.disclaimer_acknowledged,
            onboarding_acknowledged: old_config.onboarding_acknowledged,
            github_login_acknowledged: old_config.github_login_acknowledged,
            telemetry_acknowledged: old_config.telemetry_acknowledged,
            notifications: old_config.notifications,
            editor: old_config.editor,
            forges,
            analytics_enabled: old_config.analytics_enabled,
            workspace_dir: old_config.workspace_dir,
            last_app_version: old_config.last_app_version,
            show_release_notes: old_config.show_release_notes,
            language: old_config.language,
        })
    }

    /// Get GitHub config for backward compatibility
    /// Returns the first GitHub forge config found
    pub fn github_config(&self) -> Option<&ForgeConfig> {
        self.forges
            .iter()
            .find(|f| f.forge_type == ForgeType::GitHub)
    }

    /// Get all forge configs of a specific type
    pub fn forge_configs(&self, forge_type: ForgeType) -> Vec<&ForgeConfig> {
        self.forges
            .iter()
            .filter(|f| f.forge_type == forge_type)
            .collect()
    }

    /// Get mutable GitHub config for backward compatibility
    pub fn github_config_mut(&mut self) -> Option<&mut ForgeConfig> {
        self.forges
            .iter_mut()
            .find(|f| f.forge_type == ForgeType::GitHub)
    }
}

impl From<String> for Config {
    fn from(raw_config: String) -> Self {
        if let Ok(config) = serde_json::from_str::<Config>(&raw_config)
            && config.config_version == "v8"
        {
            return config;
        }

        match Self::from_previous_version(&raw_config) {
            Ok(config) => {
                tracing::info!("Config upgraded to v8");
                config
            }
            Err(e) => {
                tracing::warn!("Config migration failed: {}, using default", e);
                Self::default()
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_version: "v8".to_string(),
            theme: ThemeMode::System,
            executor_profile: ExecutorProfileId::new(BaseCodingAgent::ClaudeCode),
            disclaimer_acknowledged: false,
            onboarding_acknowledged: false,
            github_login_acknowledged: false,
            telemetry_acknowledged: false,
            notifications: NotificationConfig::default(),
            editor: EditorConfig::default(),
            forges: vec![ForgeConfig::default()], // Default GitHub config
            analytics_enabled: None,
            workspace_dir: None,
            last_app_version: None,
            show_release_notes: false,
            language: UiLanguage::default(),
        }
    }
}
