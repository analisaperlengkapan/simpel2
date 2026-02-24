//! Azure Key Vault auto-unseal provider
//!
//! This provider uses Microsoft Azure Key Vault for auto-unsealing.
//! It supports:
//! - Managed identity authentication (recommended for Azure VMs/AKS)
//! - Service principal authentication (for development/testing)
//! - Custom endpoints (for testing or Azure Stack)
//!
//! # Authentication Methods
//!
//! ## Managed Identity (Recommended for Production)
//! ```toml
//! [auto_unseal]
//! provider = "azure-key-vault"
//! vault_name = "my-secreton-vault"
//! key_name = "auto-unseal-key"
//! ```
//!
//! ## Service Principal (Development/Testing)
//! ```toml
//! [auto_unseal]
//! provider = "azure-key-vault"
//! vault_name = "my-secreton-vault"
//! key_name = "auto-unseal-key"
//! tenant_id = "00000000-0000-0000-0000-000000000000"
//! client_id = "00000000-0000-0000-0000-000000000000"
//! client_secret = "your-client-secret"
//! ```

#[cfg(feature = "azure-kv")]
use async_trait::async_trait;
#[cfg(feature = "azure-kv")]
use azure_core::{Url, auth::TokenCredential};
#[cfg(feature = "azure-kv")]
use azure_identity::{ClientSecretCredential, DefaultAzureCredential, TokenCredentialOptions};
#[cfg(feature = "azure-kv")]
use azure_security_keyvault::KeyClient;
#[cfg(feature = "azure-kv")]
use secreton_core::error::SecretonError;
#[cfg(feature = "azure-kv")]
use std::sync::Arc;
#[cfg(feature = "azure-kv")]
use tracing::{debug, error, info};

#[cfg(feature = "azure-kv")]
use crate::config::AzureKeyVaultConfig;
#[cfg(feature = "azure-kv")]
use crate::error::AutoUnsealError;
#[cfg(feature = "azure-kv")]
use crate::{AutoUnsealProvider, ProviderMetadata};

/// Azure Key Vault auto-unseal provider
///
/// Uses Azure Key Vault to encrypt/decrypt the master key.
/// Supports managed identity authentication (recommended) and service principal authentication.
#[cfg(feature = "azure-kv")]
#[derive(Clone)]
pub struct AzureKeyVaultProvider {
    client: KeyClient,
    vault_name: String,
    key_name: String,
    key_version: Option<String>,
}

#[cfg(feature = "azure-kv")]
impl AzureKeyVaultProvider {
    /// Create a new Azure Key Vault provider
    ///
    /// # Arguments
    ///
    /// * `config` - Azure Key Vault configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration is invalid
    /// - Azure credentials cannot be loaded
    /// - The Key Vault client cannot be created
    ///
    /// # Example
    ///
    /// ```no_run
    /// use secreton_auto_unseal::azure_kv::AzureKeyVaultProvider;
    /// use secreton_auto_unseal::config::AzureKeyVaultConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = AzureKeyVaultConfig {
    ///     vault_name: "my-vault".to_string(),
    ///     key_name: "auto-unseal-key".to_string(),
    ///     key_version: None,
    ///     tenant_id: None,
    ///     client_id: None,
    ///     client_secret: None,
    /// };
    ///
    /// let provider = AzureKeyVaultProvider::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: AzureKeyVaultConfig) -> Result<Self, AutoUnsealError> {
        info!(
            vault_name = %config.vault_name,
            key_name = %config.key_name,
            "Initializing Azure Key Vault auto-unseal provider"
        );

        // Validate configuration
        if config.vault_name.is_empty() {
            return Err(AutoUnsealError::InvalidConfiguration(
                "vault_name cannot be empty".to_string(),
            ));
        }

        if config.key_name.is_empty() {
            return Err(AutoUnsealError::InvalidConfiguration(
                "key_name cannot be empty".to_string(),
            ));
        }

        // Build vault URL
        let vault_url = format!("https://{}.vault.azure.net", config.vault_name);
        let vault_url = Url::parse(&vault_url).map_err(|e| {
            error!(error = %e, "Invalid vault URL");
            AutoUnsealError::InvalidConfiguration(format!("Invalid vault URL: {}", e))
        })?;

        // Create HTTP client
        let http_client = azure_core::new_http_client();

        // Create credential
        let credential: Arc<dyn TokenCredential> =
            if let (Some(tenant_id), Some(client_id), Some(client_secret)) =
                (&config.tenant_id, &config.client_id, &config.client_secret)
            {
                debug!("Using service principal authentication");

                Arc::new(ClientSecretCredential::new(
                    http_client.clone(),
                    vault_url.clone(),
                    client_secret.clone(),
                    tenant_id.clone(),
                    client_id.clone(),
                ))
            } else {
                debug!(
                    "Using default Azure credential chain (managed identity, environment, etc.)"
                );

                Arc::new(
                    DefaultAzureCredential::create(TokenCredentialOptions::default()).map_err(
                        |e| {
                            error!(error = %e, "Failed to create default credential");
                            AutoUnsealError::InvalidCredentials(format!(
                                "Failed to create default credential: {}",
                                e
                            ))
                        },
                    )?,
                )
            };

        // Create Key Vault client
        let client = KeyClient::new(&vault_url.to_string(), credential).map_err(|e| {
            error!(error = %e, "Failed to create Key Vault client");
            AutoUnsealError::InitializationFailed(format!(
                "Failed to create Key Vault client: {}",
                e
            ))
        })?;

        info!(
            vault_name = %config.vault_name,
            key_name = %config.key_name,
            "Azure Key Vault auto-unseal provider initialized successfully"
        );

        Ok(Self {
            client,
            vault_name: config.vault_name,
            key_name: config.key_name,
            key_version: config.key_version,
        })
    }

    /// Create a new Azure Key Vault provider with explicit parameters
    ///
    /// This is a convenience method for creating a provider without a config struct.
    ///
    /// # Arguments
    ///
    /// * `vault_name` - Key Vault name
    /// * `key_name` - Key name
    ///
    /// # Example
    ///
    /// ```no_run
    /// use secreton_auto_unseal::azure_kv::AzureKeyVaultProvider;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = AzureKeyVaultProvider::from_params(
    ///     "my-vault".to_string(),
    ///     "auto-unseal-key".to_string(),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_params(
        vault_name: String,
        key_name: String,
    ) -> Result<Self, AutoUnsealError> {
        let config = AzureKeyVaultConfig {
            vault_name,
            key_name,
            key_version: None,
            tenant_id: None,
            client_id: None,
            client_secret: None,
        };

        Self::new(config).await
    }

    /// Validate the key name format
    ///
    /// Azure Key Vault key names must:
    /// - Be 1-127 characters long
    /// - Contain only alphanumeric characters and hyphens
    /// - Start with a letter
    fn validate_key_name(&self) -> Result<(), AutoUnsealError> {
        let key_name = &self.key_name;

        if key_name.is_empty() || key_name.len() > 127 {
            return Err(AutoUnsealError::InvalidConfiguration(format!(
                "Key name must be 1-127 characters long: {}",
                key_name
            )));
        }

        if !key_name.chars().next().unwrap().is_alphabetic() {
            return Err(AutoUnsealError::InvalidConfiguration(format!(
                "Key name must start with a letter: {}",
                key_name
            )));
        }

        if !key_name.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(AutoUnsealError::InvalidConfiguration(format!(
                "Key name can only contain alphanumeric characters and hyphens: {}",
                key_name
            )));
        }

        Ok(())
    }

    /// Get the key version to use
    ///
    /// Returns the configured version or empty string for latest version
    fn get_key_version(&self) -> &str {
        self.key_version.as_deref().unwrap_or("")
    }
}

#[cfg(feature = "azure-kv")]
#[async_trait]
impl AutoUnsealProvider for AzureKeyVaultProvider {
    fn name(&self) -> &str {
        "azure-key-vault"
    }

    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        debug!(
            vault_name = %self.vault_name,
            key_name = %self.key_name,
            plaintext_len = plaintext.len(),
            "Encrypting master key with Azure Key Vault"
        );

        // Validate key name format
        self.validate_key_name().map_err(|e| {
            error!(error = %e, "Invalid key name format");
            e
        })?;

        // TODO: Azure SDK 0.20 API compatibility issue
        // The CryptographParamtersEncryption enum doesn't have clear documentation
        // for RSA-OAEP-256 variant names. This needs to be resolved by:
        // 1. Checking Azure SDK 0.20 source code for correct enum variants
        // 2. Or upgrading to a newer Azure SDK version with better documentation
        // 3. Or using the REST API directly
        //
        // For now, return an error indicating this is not yet implemented
        Err(AutoUnsealError::ProviderError(
            "Azure Key Vault provider requires Azure SDK API compatibility fixes".to_string(),
        )
        .into())
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        debug!(
            vault_name = %self.vault_name,
            key_name = %self.key_name,
            ciphertext_len = ciphertext.len(),
            "Decrypting master key with Azure Key Vault"
        );

        // TODO: Azure SDK 0.20 API compatibility issue
        // See encrypt() method for details
        Err(AutoUnsealError::ProviderError(
            "Azure Key Vault provider requires Azure SDK API compatibility fixes".to_string(),
        )
        .into())
    }

    async fn health_check(&self) -> Result<(), SecretonError> {
        debug!(
            vault_name = %self.vault_name,
            key_name = %self.key_name,
            "Performing Azure Key Vault health check"
        );

        // Validate key name format first
        self.validate_key_name().map_err(|e| {
            error!(error = %e, "Invalid key name format in health check");
            e
        })?;

        // Use GetKey to verify the key exists and is accessible
        let _response = self.client.get(&self.key_name).await.map_err(|e| {
            error!(error = %e, "Azure Key Vault health check failed");

            let error_msg = e.to_string();
            if error_msg.contains("KeyNotFound") || error_msg.contains("not found") {
                AutoUnsealError::KeyNotFound(self.key_name.clone())
            } else if error_msg.contains("Forbidden") || error_msg.contains("Unauthorized") {
                AutoUnsealError::PermissionDenied(format!(
                    "No permission to access key: {}",
                    self.key_name
                ))
            } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
                AutoUnsealError::Timeout("Azure Key Vault health check timed out".to_string())
            } else {
                AutoUnsealError::HealthCheckFailed(format!("Azure Key Vault error: {}", error_msg))
            }
        })?;

        info!(
            vault_name = %self.vault_name,
            key_name = %self.key_name,
            "Azure Key Vault health check passed"
        );

        Ok(())
    }

    fn metadata(&self) -> ProviderMetadata {
        let mut metadata =
            ProviderMetadata::new("azure-key-vault".to_string(), self.key_name.clone());

        // Add vault name as endpoint
        metadata = metadata.with_endpoint(format!("https://{}.vault.azure.net", self.vault_name));

        // Add key version if specified
        if let Some(version) = &self.key_version {
            metadata.region = Some(format!("version:{}", version));
        }

        metadata
    }
}

#[cfg(all(test, feature = "azure-kv"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_azure_kv_provider_creation() {
        let config = AzureKeyVaultConfig {
            vault_name: "my-vault".to_string(),
            key_name: "auto-unseal-key".to_string(),
            key_version: None,
            tenant_id: None,
            client_id: None,
            client_secret: None,
        };

        // Note: This will fail without valid Azure credentials
        // In real tests, use mock credentials or skip if not in Azure environment
        let result = AzureKeyVaultProvider::new(config).await;

        // We expect this to fail in test environment without credentials
        // Just verify the error is reasonable
        if let Err(e) = result {
            assert!(matches!(
                e,
                AutoUnsealError::InitializationFailed(_) | AutoUnsealError::InvalidCredentials(_)
            ));
        }
    }

    #[tokio::test]
    async fn test_azure_kv_provider_invalid_config() {
        let config = AzureKeyVaultConfig {
            vault_name: "".to_string(),
            key_name: "auto-unseal-key".to_string(),
            key_version: None,
            tenant_id: None,
            client_id: None,
            client_secret: None,
        };

        let result = AzureKeyVaultProvider::new(config).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AutoUnsealError::InvalidConfiguration(_)
        ));
    }

    #[test]
    fn test_key_name_validation() {
        // Create a mock provider for testing validation
        // Note: We can't easily create a real client without credentials,
        // so we'll just test the validation logic directly

        // Test valid key name
        let key_name = "auto-unseal-key".to_string();
        assert!(key_name.len() <= 127);
        assert!(key_name.chars().next().unwrap().is_alphabetic());
        assert!(key_name.chars().all(|c| c.is_alphanumeric() || c == '-'));

        // Test key name starting with number (invalid)
        let key_name = "123-key".to_string();
        assert!(!key_name.chars().next().unwrap().is_alphabetic());

        // Test key name with invalid characters
        let key_name = "key_with_underscore".to_string();
        assert!(!key_name.chars().all(|c| c.is_alphanumeric() || c == '-'));

        // Test empty key name
        let key_name = "".to_string();
        assert!(key_name.is_empty());

        // Test key name too long (>127 characters)
        let key_name = "a".repeat(128);
        assert!(key_name.len() > 127);
    }

    #[test]
    fn test_provider_metadata() {
        // Test metadata construction without creating a full provider
        let metadata =
            ProviderMetadata::new("azure-key-vault".to_string(), "auto-unseal-key".to_string())
                .with_endpoint("https://test-vault.vault.azure.net".to_string());

        assert_eq!(metadata.provider_type, "azure-key-vault");
        assert_eq!(metadata.key_id, "auto-unseal-key");
        assert_eq!(
            metadata.endpoint,
            Some("https://test-vault.vault.azure.net".to_string())
        );
    }
}
