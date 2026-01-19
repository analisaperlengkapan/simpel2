//! Agent configuration

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `AgentConfig`.
pub struct AgentConfig {
    /// Vault server URL
    pub server_url: String,

    /// Fallback server URLs
    #[serde(default)]
    pub server_urls: Vec<String>,

    /// Authentication method
    pub auth_method: String,

    /// Authentication configuration
    pub auth_config: HashMap<String, String>,

    /// Templates to render
    #[serde(default)]
    pub templates: Vec<TemplateConfig>,

    /// Token renewal interval in seconds
    #[serde(default = "default_renewal_interval")]
    pub token_renewal_interval_secs: u64,

    /// Template rendering interval in seconds
    #[serde(default = "default_template_interval")]
    pub template_interval_secs: u64,

    /// Token sink configuration
    pub sink: Option<SinkConfig>,

    /// Health check port
    pub health_port: Option<u16>,

    /// Child process to run
    pub run: Option<Vec<String>>,

    /// Restart child on failure
    #[serde(default)]
    pub restart_child: bool,

    /// Restart delay in seconds
    #[serde(default = "default_restart_delay")]
    pub restart_delay_secs: u64,
}

impl AgentConfig {
    /// Load from file
    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;

        if path.ends_with(".yaml") || path.ends_with(".yml") {
            Ok(serde_yaml::from_str(&content)?)
        } else if path.ends_with(".toml") {
            Ok(toml::from_str(&content)?)
        } else {
            Err(anyhow::anyhow!("Unsupported config format"))
        }
    }

    /// Get token renewal interval
    pub fn token_renewal_interval(&self) -> Duration {
        Duration::from_secs(self.token_renewal_interval_secs)
    }

    /// Get template rendering interval
    pub fn template_interval(&self) -> Duration {
        Duration::from_secs(self.template_interval_secs)
    }

    /// Get restart delay
    pub fn restart_delay(&self) -> Duration {
        Duration::from_secs(self.restart_delay_secs)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `TemplateConfig`.
pub struct TemplateConfig {
    /// Secret path in vault
    pub source: String,

    /// Destination file path
    pub dest: String,

    /// File permissions (octal)
    pub permissions: Option<String>,

    /// Template content (optional, for inline templates)
    pub template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `SinkConfig`.
pub struct SinkConfig {
    /// Sink types: file, env, child
    pub types: Vec<String>,

    /// File path for file sink
    pub file_path: Option<String>,

    /// File permissions
    pub file_permissions: Option<String>,

    /// Environment variable name for env sink
    pub env_var: Option<String>,
}

fn default_renewal_interval() -> u64 {
    300 // 5 minutes
}

fn default_template_interval() -> u64 {
    60 // 1 minute
}

fn default_restart_delay() -> u64 {
    5
}
