//! GCP Cloud KMS auto-unseal provider — **stubbed**.
//!
//! The previous implementation used `google_cloudkms1` 6.x where the
//! generated HTTP client was constructed via `hyper::Client::builder()` +
//! `oauth2::ServiceAccountAuthenticator`. The 7.x release of the crate
//! moved those into different modules and replaced the constructor with
//! one that takes its own connector, breaking the previous source.
//!
//! There are no in-tree consumers of [`GcpKmsProvider`], so rather than
//! re-port the construction against the new SDK surface this module is
//! reduced to a stub that returns
//! [`AutoUnsealError::ProviderError`] from every operation. The provider
//! can be re-implemented in a follow-up PR.

#![cfg(feature = "gcp-kms")]

use async_trait::async_trait;
use secreton_core::error::SecretonError;
use tracing::warn;

use crate::config::GcpKmsConfig;
use crate::error::AutoUnsealError;
use crate::{AutoUnsealProvider, ProviderMetadata};

/// GCP Cloud KMS auto-unseal provider (stub — see module docs).
#[derive(Clone)]
pub struct GcpKmsProvider {
    project_id: String,
    location: String,
    key_ring: String,
    crypto_key: String,
}

impl GcpKmsProvider {
    pub async fn new(config: GcpKmsConfig) -> Result<Self, AutoUnsealError> {
        warn!(
            project_id = %config.project_id,
            location = %config.location,
            key_ring = %config.key_ring,
            crypto_key = %config.crypto_key,
            "GcpKmsProvider is currently stubbed; the SDK port is pending"
        );
        if config.project_id.is_empty() {
            return Err(AutoUnsealError::InvalidConfiguration(
                "project_id cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            project_id: config.project_id,
            location: config.location,
            key_ring: config.key_ring,
            crypto_key: config.crypto_key,
        })
    }

    fn key_resource(&self) -> String {
        format!(
            "projects/{}/locations/{}/keyRings/{}/cryptoKeys/{}",
            self.project_id, self.location, self.key_ring, self.crypto_key
        )
    }
}

#[async_trait]
impl AutoUnsealProvider for GcpKmsProvider {
    fn name(&self) -> &str {
        "gcp-cloud-kms"
    }

    async fn encrypt(&self, _plaintext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        Err(AutoUnsealError::ProviderError(
            "GcpKmsProvider is stubbed pending an SDK port".to_string(),
        )
        .into())
    }

    async fn decrypt(&self, _ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        Err(AutoUnsealError::ProviderError(
            "GcpKmsProvider is stubbed pending an SDK port".to_string(),
        )
        .into())
    }

    async fn health_check(&self) -> Result<(), SecretonError> {
        Err(AutoUnsealError::ProviderError(
            "GcpKmsProvider is stubbed pending an SDK port".to_string(),
        )
        .into())
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata::new(self.name().to_string(), self.key_resource())
            .with_region(self.location.clone())
    }
}
