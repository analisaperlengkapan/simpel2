//! Azure Key Vault auto-unseal provider — **stubbed**.
//!
//! The previous implementation targeted `azure_identity` 0.20.x +
//! `azure_security_keyvault` 0.20.x. Their successors at 0.33.x moved
//! `DefaultAzureCredential`, `TokenCredentialOptions`, `Url`,
//! `auth::TokenCredential`, and the `new_http_client` constructor —
//! the previous source no longer compiles against published SDKs.
//!
//! There are no in-tree consumers of [`AzureKeyVaultProvider`], so rather
//! than gut the public surface or pin to a removed SDK version we expose
//! the same struct + trait impl shape but every method returns
//! [`AutoUnsealError::ProviderError`]. A follow-up PR can re-implement the
//! provider against the new SDK (azure-sdk-rs 0.x has stabilised the
//! Url/credential helpers under fresh paths).

#![cfg(feature = "azure-kv")]

use async_trait::async_trait;
use secreton_core::error::SecretonError;
use tracing::warn;

use crate::config::AzureKeyVaultConfig;
use crate::error::AutoUnsealError;
use crate::{AutoUnsealProvider, ProviderMetadata};

/// Azure Key Vault auto-unseal provider (stub — see module docs).
#[derive(Clone)]
pub struct AzureKeyVaultProvider {
    vault_name: String,
    key_name: String,
    key_version: Option<String>,
}

impl AzureKeyVaultProvider {
    pub async fn new(config: AzureKeyVaultConfig) -> Result<Self, AutoUnsealError> {
        warn!(
            vault_name = %config.vault_name,
            key_name = %config.key_name,
            "AzureKeyVaultProvider is currently stubbed; the SDK port is pending"
        );
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
        Ok(Self {
            vault_name: config.vault_name,
            key_name: config.key_name,
            key_version: config.key_version,
        })
    }

    pub async fn from_params(
        vault_name: String,
        key_name: String,
    ) -> Result<Self, AutoUnsealError> {
        Self::new(AzureKeyVaultConfig {
            vault_name,
            key_name,
            key_version: None,
            tenant_id: None,
            client_id: None,
            client_secret: None,
        })
        .await
    }
}

#[async_trait]
impl AutoUnsealProvider for AzureKeyVaultProvider {
    fn name(&self) -> &str {
        "azure-key-vault"
    }

    async fn encrypt(&self, _plaintext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        Err(AutoUnsealError::ProviderError(
            "AzureKeyVaultProvider is stubbed pending an SDK port".to_string(),
        )
        .into())
    }

    async fn decrypt(&self, _ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        Err(AutoUnsealError::ProviderError(
            "AzureKeyVaultProvider is stubbed pending an SDK port".to_string(),
        )
        .into())
    }

    async fn health_check(&self) -> Result<(), SecretonError> {
        Err(AutoUnsealError::ProviderError(
            "AzureKeyVaultProvider is stubbed pending an SDK port".to_string(),
        )
        .into())
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata::new(
            self.name().to_string(),
            format!("{}/{}", self.vault_name, self.key_name),
        )
    }
}
