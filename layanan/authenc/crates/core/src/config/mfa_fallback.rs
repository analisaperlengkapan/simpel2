//! Configuration for MFA fallback and local storage

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for MFA local storage fallback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaFallbackConfig {
    /// Enable local storage fallback
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Path to encrypted storage file
    #[serde(default = "default_storage_path")]
    pub storage_path: PathBuf,

    /// Encryption key for local storage (base64 encoded)
    /// If not provided, will be generated and stored in environment
    pub encryption_key: Option<String>,

    /// Sync interval in seconds
    #[serde(default = "default_sync_interval")]
    pub sync_interval_seconds: u64,

    /// Maximum sync attempts before giving up
    #[serde(default = "default_max_sync_attempts")]
    pub max_sync_attempts: u32,

    /// Enable automatic sync when Secreton recovers
    #[serde(default = "default_auto_sync")]
    pub auto_sync_enabled: bool,

    /// Log degraded mode warnings
    #[serde(default = "default_log_warnings")]
    pub log_degraded_warnings: bool,
}

fn default_enabled() -> bool {
    true
}

fn default_storage_path() -> PathBuf {
    PathBuf::from("/var/lib/authenc/mfa_storage.enc")
}

fn default_sync_interval() -> u64 {
    60 // 1 minute
}

fn default_max_sync_attempts() -> u32 {
    10
}

fn default_auto_sync() -> bool {
    true
}

fn default_log_warnings() -> bool {
    true
}

impl Default for MfaFallbackConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            storage_path: default_storage_path(),
            encryption_key: None,
            sync_interval_seconds: default_sync_interval(),
            max_sync_attempts: default_max_sync_attempts(),
            auto_sync_enabled: default_auto_sync(),
            log_degraded_warnings: default_log_warnings(),
        }
    }
}

impl MfaFallbackConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(enabled) = std::env::var("MFA_FALLBACK_ENABLED") {
            config.enabled = enabled.parse().unwrap_or(true);
        }

        if let Ok(path) = std::env::var("MFA_FALLBACK_STORAGE_PATH") {
            config.storage_path = PathBuf::from(path);
        }

        if let Ok(key) = std::env::var("MFA_FALLBACK_ENCRYPTION_KEY") {
            config.encryption_key = Some(key);
        }

        if let Ok(interval) = std::env::var("MFA_FALLBACK_SYNC_INTERVAL") {
            config.sync_interval_seconds = interval.parse().unwrap_or(60);
        }

        if let Ok(attempts) = std::env::var("MFA_FALLBACK_MAX_SYNC_ATTEMPTS") {
            config.max_sync_attempts = attempts.parse().unwrap_or(10);
        }

        if let Ok(auto_sync) = std::env::var("MFA_FALLBACK_AUTO_SYNC") {
            config.auto_sync_enabled = auto_sync.parse().unwrap_or(true);
        }

        config
    }

    /// Get or generate encryption key
    pub fn get_encryption_key(&self) -> Result<[u8; 32], String> {
        if let Some(key_b64) = &self.encryption_key {
            // Decode from base64
            use base64::{Engine, engine::general_purpose::STANDARD};
            let key_bytes = STANDARD
                .decode(key_b64)
                .map_err(|e| format!("Invalid base64 encryption key: {}", e))?;

            if key_bytes.len() != 32 {
                return Err(format!(
                    "Encryption key must be 32 bytes, got {}",
                    key_bytes.len()
                ));
            }

            let mut key = [0u8; 32];
            key.copy_from_slice(&key_bytes);
            Ok(key)
        } else {
            // Generate a new key
            use authenc_crypto::aes_gcm::AesGcmService;
            Ok(AesGcmService::generate_key())
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.enabled {
            // Check if storage directory exists or can be created
            if let Some(parent) = self.storage_path.parent()
                && !parent.exists()
            {
                return Err(format!("Storage directory does not exist: {:?}", parent));
            }

            // Validate sync interval
            if self.sync_interval_seconds < 10 {
                return Err("Sync interval must be at least 10 seconds".to_string());
            }

            // Validate max sync attempts
            if self.max_sync_attempts == 0 {
                return Err("Max sync attempts must be greater than 0".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MfaFallbackConfig::default();
        assert!(config.enabled);
        assert_eq!(config.sync_interval_seconds, 60);
        assert_eq!(config.max_sync_attempts, 10);
        assert!(config.auto_sync_enabled);
    }

    #[test]
    fn test_get_encryption_key() {
        let config = MfaFallbackConfig::default();
        let key = config.get_encryption_key().unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_validate_config() {
        let mut config = MfaFallbackConfig::default();
        config.storage_path = PathBuf::from("/tmp/test_mfa_storage.enc");

        // Should pass validation
        assert!(config.validate().is_ok());

        // Invalid sync interval
        config.sync_interval_seconds = 5;
        assert!(config.validate().is_err());

        // Invalid max sync attempts
        config.sync_interval_seconds = 60;
        config.max_sync_attempts = 0;
        assert!(config.validate().is_err());
    }
}
