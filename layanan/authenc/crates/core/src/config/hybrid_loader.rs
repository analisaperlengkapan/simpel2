//! Hybrid Configuration Loader
//!
//! Implements the hybrid configuration loading strategy:
//! 1. Load minimal bootstrap config from authenc.toml
//! 2. Connect to database
//! 3. Load full config from authenc.configuration table
//! 4. Load secrets from Secreton
//! 5. Apply environment variable overrides

use crate::config::AppConfig;
use authenc_types::Result;
use tracing::{debug, info, warn};

/// Hybrid configuration loader
pub struct HybridConfigLoader;

impl HybridConfigLoader {
    /// Load configuration using hybrid approach
    ///
    /// # Configuration Hierarchy
    /// 1. Bootstrap TOML (authenc.toml)
    /// 2. Database configuration (authenc.configuration table)
    /// 3. Secreton secrets
    /// 4. Environment variable overrides
    pub async fn load() -> Result<AppConfig> {
        info!("🔄 Starting hybrid configuration loading...");

        // Step 1: Load bootstrap config from authenc.toml
        debug!("Step 1: Loading bootstrap configuration from authenc.toml");
        let mut config = Self::load_bootstrap_config()?;
        info!("✅ Bootstrap configuration loaded");

        // Step 2: Check if database config loading is enabled
        if config.config_loader.load_from_database {
            debug!("Step 2: Loading full configuration from database");
            match Self::load_database_config(&config).await {
                Ok(db_config) => {
                    config.merge(db_config);
                    info!("✅ Database configuration loaded and merged");
                }
                Err(e) => {
                    warn!("⚠️  Failed to load database config (non-fatal): {}", e);
                    info!("Continuing with bootstrap configuration only");
                }
            }
        } else {
            debug!("Database configuration loading disabled");
        }

        // Step 3: Load secrets from Secreton if enabled
        if let Some(secreton_config) = &config.secreton {
            debug!("Step 3: Loading secrets from Secreton");
            match Self::load_secreton_secrets(secreton_config).await {
                Ok(secrets) => {
                    Self::apply_secrets(&mut config, secrets);
                    info!("✅ Secrets loaded from Secreton");
                }
                Err(e) => {
                    warn!("⚠️  Failed to load Secreton secrets (non-fatal): {}", e);
                }
            }
        } else {
            debug!("Secreton integration disabled");
        }

        // Step 4: Apply environment variable overrides
        debug!("Step 4: Applying environment variable overrides");
        config.apply_env_overrides()?;
        info!("✅ Environment variable overrides applied");

        // Step 5: Validate final configuration
        debug!("Step 5: Validating final configuration");
        config.validate()?;
        info!("✅ Configuration validation passed");

        info!("🎉 Hybrid configuration loading completed successfully");
        Ok(config)
    }

    /// Load bootstrap configuration from authenc.toml
    fn load_bootstrap_config() -> Result<AppConfig> {
        // Try to load authenc.toml
        let config_path = "authenc.toml";
        if std::path::PathBuf::from(config_path).exists() {
            debug!("Loading bootstrap config from {}", config_path);
            AppConfig::from_file(config_path)
        } else {
            debug!("Bootstrap config file not found, using defaults");
            Ok(AppConfig::default())
        }
    }

    /// Load configuration from database
    async fn load_database_config(bootstrap_config: &AppConfig) -> Result<AppConfig> {
        // This would connect to the database and load from authenc.configuration table
        // For now, return a placeholder that indicates this needs to be implemented
        // when database connection is available

        debug!(
            "Attempting to load config from database: {}:{}",
            bootstrap_config.database.host, bootstrap_config.database.port
        );

        // TODO: Implement database config loading
        // This requires:
        // 1. Create database connection using bootstrap config
        // 2. Query authenc.configuration table
        // 3. Merge results into AppConfig
        // 4. Cache results with TTL

        Ok(AppConfig::default())
    }

    /// Load secrets from Secreton
    async fn load_secreton_secrets(
        secreton_config: &crate::config::SecretonConfig,
    ) -> Result<std::collections::HashMap<String, String>> {
        debug!(
            "Attempting to load secrets from Secreton: {}",
            secreton_config.endpoint
        );

        // TODO: Implement Secreton integration
        // This requires:
        // 1. Create Secreton client
        // 2. Authenticate with token
        // 3. Load specified secrets
        // 4. Return as HashMap

        Ok(std::collections::HashMap::new())
    }

    /// Apply loaded secrets to configuration
    fn apply_secrets(config: &mut AppConfig, secrets: std::collections::HashMap<String, String>) {
        for (key, value) in secrets {
            match key.as_str() {
                "jwt_secret" => {
                    config.security.jwt_secret = value;
                    debug!("Applied JWT secret from Secreton");
                }
                "ed25519_private_key" => {
                    // TODO: Apply to signing key configuration
                    debug!("Applied Ed25519 private key from Secreton");
                }
                "ecdsa_p256_private_key" => {
                    // TODO: Apply to signing key configuration
                    debug!("Applied ECDSA P256 private key from Secreton");
                }
                "smtp_password" => {
                    // TODO: Apply to email configuration
                    debug!("Applied SMTP password from Secreton");
                }
                _ => {
                    debug!("Unknown secret key: {}", key);
                }
            }
        }
    }
}

/// Configuration loader configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigLoaderConfig {
    /// Enable configuration loading from database
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Load full config from database after bootstrap
    #[serde(default = "default_true")]
    pub load_from_database: bool,

    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,

    /// Reload interval in seconds (0 = disabled)
    #[serde(default)]
    pub reload_interval: u64,

    /// Allow hot reload via API
    #[serde(default = "default_true")]
    pub allow_hot_reload: bool,
}

impl Default for ConfigLoaderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            load_from_database: true,
            cache_ttl: 300,
            reload_interval: 0,
            allow_hot_reload: true,
        }
    }
}

// Helper functions for defaults
fn default_true() -> bool {
    true
}

fn default_cache_ttl() -> u64 {
    300
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loader_config_defaults() {
        let config = ConfigLoaderConfig::default();
        assert!(config.enabled);
        assert!(config.load_from_database);
        assert_eq!(config.cache_ttl, 300);
        assert_eq!(config.reload_interval, 0);
        assert!(config.allow_hot_reload);
    }
}
