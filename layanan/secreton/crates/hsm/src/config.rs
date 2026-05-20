//! HSM configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// HSM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmConfig {
    /// Enable HSM integration
    pub enabled: bool,

    /// HSM provider type (pkcs11, aws-kms, azure-keyengine, gcp-kms)
    pub provider: HsmProvider,

    /// PKCS#11 library path (for PKCS#11 provider)
    pub pkcs11_library_path: Option<PathBuf>,

    /// HSM slot ID (for PKCS#11 provider)
    pub slot_id: Option<u64>,

    /// HSM token label (for PKCS#11 provider)
    pub token_label: Option<String>,

    /// HSM PIN/password
    pub pin: Option<String>,

    /// Key label prefix for generated keys
    pub key_label_prefix: String,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Operation timeout in seconds
    pub operation_timeout: u64,

    /// Enable health checks
    pub health_check_enabled: bool,

    /// Health check interval in seconds
    pub health_check_interval: u64,

    /// Maximum retry attempts for failed operations
    pub max_retries: u32,

    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

/// HSM provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HsmProvider {
    /// PKCS#11 compatible HSM
    Pkcs11,
    /// AWS KMS
    AwsKms,
    /// Azure Key Engine
    AzureKeyEngine,
    /// Google Cloud KMS
    GcpKms,
}

impl Default for HsmConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: HsmProvider::Pkcs11,
            pkcs11_library_path: None,
            slot_id: Some(0),
            token_label: None,
            pin: None,
            key_label_prefix: "secreton-".to_string(),
            connection_timeout: 30,
            operation_timeout: 60,
            health_check_enabled: true,
            health_check_interval: 60,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

impl HsmConfig {
    /// Validate HSM configuration
    pub fn validate(&self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        match self.provider {
            HsmProvider::Pkcs11 => {
                if self.pkcs11_library_path.is_none() {
                    return Err("PKCS#11 library path is required".to_string());
                }
                if self.slot_id.is_none() {
                    return Err("HSM slot ID is required".to_string());
                }
            }
            HsmProvider::AwsKms => {
                // AWS KMS validation would go here
            }
            HsmProvider::AzureKeyEngine => {
                // Azure Key Engine validation would go here
            }
            HsmProvider::GcpKms => {
                // GCP KMS validation would go here
            }
        }

        if self.connection_timeout == 0 {
            return Err("Connection timeout must be greater than 0".to_string());
        }

        if self.operation_timeout == 0 {
            return Err("Operation timeout must be greater than 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HsmConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.provider, HsmProvider::Pkcs11);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_pkcs11_validation() {
        let mut config = HsmConfig {
            enabled: true,
            provider: HsmProvider::Pkcs11,
            ..Default::default()
        };

        // Missing library path
        assert!(config.validate().is_err());

        // Add library path
        config.pkcs11_library_path = Some(PathBuf::from("/usr/lib/libpkcs11.so"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_timeout_validation() {
        let config = HsmConfig {
            enabled: true,
            pkcs11_library_path: Some(PathBuf::from("/usr/lib/libpkcs11.so")),
            connection_timeout: 0,
            ..Default::default()
        };

        assert!(config.validate().is_err());
    }
}
