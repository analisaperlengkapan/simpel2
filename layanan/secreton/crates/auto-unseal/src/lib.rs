//! Auto-unseal providers for Secreton
//!
//! This crate provides automatic unsealing capabilities using external key management services.
//! Supported providers:
//! - Transit: Use another Secreton instance's Transit engine
//! - AWS KMS: Amazon Web Services Key Management Service
//! - GCP KMS: Google Cloud Platform Key Management Service
//! - Azure Key Vault: Microsoft Azure Key Vault
//!
//! # Example
//!
//! ```no_run
//! use secreton_auto_unseal::{AutoUnsealProvider, ProviderMetadata};
//!
//! async fn unseal_example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a provider (e.g., Transit)
//!     #[cfg(feature = "transit")]
//!     {
//!         use secreton_auto_unseal::transit::TransitProvider;
//!         use secreton_auto_unseal::config::TransitConfig;
//!
//!         let config = TransitConfig {
//!             endpoint: "https://secreton.internal:50052".to_string(),
//!             key_name: "auto-unseal-key".to_string(),
//!             token: "s.token123".to_string(),
//!             tls_cert_path: None,
//!             tls_key_path: None,
//!             tls_ca_path: None,
//!             timeout_secs: 30,
//!         };
//!
//!         let provider = TransitProvider::new(config).await?;
//!
//!         // Encrypt master key
//!         let master_key = b"secret-master-key";
//!         let encrypted = provider.encrypt(master_key).await?;
//!
//!         // Decrypt master key
//!         let decrypted = provider.decrypt(&encrypted).await?;
//!         assert_eq!(master_key, decrypted.as_slice());
//!     }
//!
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use secreton_core::error::SecretonError;
use serde::{Deserialize, Serialize};

// Re-export provider modules
#[cfg(feature = "transit")]
pub mod transit;

#[cfg(feature = "aws-kms")]
pub mod aws_kms;

#[cfg(feature = "gcp-kms")]
pub mod gcp_kms;

#[cfg(feature = "azure-kv")]
pub mod azure_kv;

pub mod config;
pub mod error;
pub mod fallback;

pub use config::{AutoUnsealConfig, FallbackConfig};
pub use error::AutoUnsealError;
pub use fallback::{AutoUnsealManager, UnsealResult};

/// Auto-unseal provider trait
///
/// Implementations of this trait provide encryption and decryption of the master key
/// using external key management services. This enables automatic unsealing of Secreton
/// on startup without manual intervention.
#[async_trait]
pub trait AutoUnsealProvider: Send + Sync {
    /// Provider name (e.g., "aws-kms", "gcp-kms", "azure-kv", "transit")
    fn name(&self) -> &str;

    /// Encrypt the master key with the provider's key
    ///
    /// # Arguments
    ///
    /// * `plaintext` - The master key to encrypt
    ///
    /// # Returns
    ///
    /// The encrypted master key as bytes
    ///
    /// # Errors
    ///
    /// Returns an error if encryption fails due to:
    /// - Network connectivity issues
    /// - Invalid credentials
    /// - Key not found
    /// - Provider-specific errors
    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretonError>;

    /// Decrypt the master key with the provider's key
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - The encrypted master key
    ///
    /// # Returns
    ///
    /// The decrypted master key as bytes
    ///
    /// # Errors
    ///
    /// Returns an error if decryption fails due to:
    /// - Network connectivity issues
    /// - Invalid credentials
    /// - Key not found
    /// - Invalid ciphertext
    /// - Provider-specific errors
    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError>;

    /// Verify provider connectivity and permissions
    ///
    /// This method should verify that:
    /// - The provider is reachable
    /// - Credentials are valid
    /// - The key exists and is accessible
    /// - Required permissions are granted
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails
    async fn health_check(&self) -> Result<(), SecretonError>;

    /// Get provider-specific metadata
    ///
    /// Returns metadata about the provider configuration, useful for
    /// logging, monitoring, and debugging.
    fn metadata(&self) -> ProviderMetadata;
}

/// Provider metadata
///
/// Contains information about the auto-unseal provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Provider type (e.g., "aws-kms", "gcp-kms", "azure-kv", "transit")
    pub provider_type: String,

    /// Key identifier (ARN, resource ID, key name, etc.)
    pub key_id: String,

    /// Region (for cloud providers)
    pub region: Option<String>,

    /// Endpoint URL (for custom endpoints)
    pub endpoint: Option<String>,
}

impl ProviderMetadata {
    /// Create new provider metadata
    pub fn new(provider_type: String, key_id: String) -> Self {
        Self {
            provider_type,
            key_id,
            region: None,
            endpoint: None,
        }
    }

    /// Set region
    pub fn with_region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }

    /// Set endpoint
    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata_construction() {
        let metadata = ProviderMetadata::new("aws-kms".to_string(), "key-123".to_string())
            .with_region("us-east-1".to_string())
            .with_endpoint("https://kms.us-east-1.amazonaws.com".to_string());

        assert_eq!(metadata.provider_type, "aws-kms");
        assert_eq!(metadata.key_id, "key-123");
        assert_eq!(metadata.region, Some("us-east-1".to_string()));
        assert_eq!(
            metadata.endpoint,
            Some("https://kms.us-east-1.amazonaws.com".to_string())
        );
    }

    #[test]
    fn test_provider_metadata_minimal() {
        let metadata = ProviderMetadata::new("transit".to_string(), "auto-unseal-key".to_string());

        assert_eq!(metadata.provider_type, "transit");
        assert_eq!(metadata.key_id, "auto-unseal-key");
        assert_eq!(metadata.region, None);
        assert_eq!(metadata.endpoint, None);
    }
}
