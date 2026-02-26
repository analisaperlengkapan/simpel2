//! GCP KMS auto-unseal provider
//!
//! This provider uses Google Cloud Platform Key Management Service (KMS) for auto-unsealing.

#[cfg(feature = "gcp-kms")]
use async_trait::async_trait;
#[cfg(feature = "gcp-kms")]
use google_cloudkms1::{
    CloudKMS,
    api::{DecryptRequest, EncryptRequest},
    hyper::{self, client::HttpConnector},
    hyper_rustls::{self, HttpsConnector},
    oauth2::{self, ServiceAccountAuthenticator},
};
#[cfg(feature = "gcp-kms")]
use secreton_core::error::SecretonError;
#[cfg(feature = "gcp-kms")]
use tracing::{debug, error, info, warn};

#[cfg(feature = "gcp-kms")]
use base64;

#[cfg(feature = "gcp-kms")]
use crate::config::GcpKmsConfig;
#[cfg(feature = "gcp-kms")]
use crate::error::AutoUnsealError;
#[cfg(feature = "gcp-kms")]
use crate::{AutoUnsealProvider, ProviderMetadata};

/// GCP KMS auto-unseal provider
#[cfg(feature = "gcp-kms")]
#[derive(Clone)]
pub struct GcpKmsProvider {
    hub: CloudKMS<HttpsConnector<HttpConnector>>,
    key_name: String,
    #[allow(dead_code)]
    project_id: String,
    location: String,
    #[allow(dead_code)]
    key_ring: String,
    #[allow(dead_code)]
    crypto_key: String,
}

#[cfg(feature = "gcp-kms")]
impl GcpKmsProvider {
    /// Create a new GCP KMS provider
    pub async fn new(config: GcpKmsConfig) -> Result<Self, AutoUnsealError> {
        info!(
            key_name = %config.key_name,
            project_id = %config.project_id,
            location = %config.location,
            "Initializing GCP KMS auto-unseal provider"
        );

        // Validate configuration
        if config.key_name.is_empty() {
            return Err(AutoUnsealError::InvalidConfiguration(
                "key_name cannot be empty".to_string(),
            ));
        }

        if config.project_id.is_empty() {
            return Err(AutoUnsealError::InvalidConfiguration(
                "project_id cannot be empty".to_string(),
            ));
        }

        // Create authenticator
        let auth = if let Some(credentials_file) = &config.credentials_file {
            debug!(credentials_file = %credentials_file, "Using service account credentials");

            let service_account_key = oauth2::read_service_account_key(credentials_file)
                .await
                .map_err(|e| {
                    error!(error = %e, "Failed to read service account key");
                    AutoUnsealError::InvalidCredentials(format!(
                        "Failed to read service account key: {}",
                        e
                    ))
                })?;

            ServiceAccountAuthenticator::builder(service_account_key)
                .build()
                .await
                .map_err(|e| {
                    error!(error = %e, "Failed to create authenticator");
                    AutoUnsealError::InvalidCredentials(format!(
                        "Failed to create authenticator: {}",
                        e
                    ))
                })?
        } else {
            debug!("Using application default credentials");

            let creds_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").unwrap_or_else(|_| {
                "~/.config/gcloud/application_default_credentials.json".to_string()
            });

            let service_account_key =
                oauth2::read_application_secret(creds_path)
                    .await
                    .map_err(|e| {
                        error!(error = %e, "Failed to read application default credentials");
                        AutoUnsealError::InvalidCredentials(format!(
                            "Failed to read credentials: {}",
                            e
                        ))
                    })?;

            ServiceAccountAuthenticator::builder(service_account_key)
                .build()
                .await
                .map_err(|e| {
                    error!(error = %e, "Failed to create authenticator");
                    AutoUnsealError::InvalidCredentials(format!(
                        "Failed to create authenticator: {}",
                        e
                    ))
                })?
        };

        // Create HTTPS connector
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .map_err(|e| {
                error!(error = %e, "Failed to create HTTPS connector");
                AutoUnsealError::InitializationFailed(format!(
                    "Failed to create HTTPS connector: {}",
                    e
                ))
            })?
            .https_or_http()
            .enable_http1()
            .build();

        let hub = CloudKMS::new(hyper::Client::builder().build(https), auth);

        info!("GCP KMS auto-unseal provider initialized successfully");

        Ok(Self {
            hub,
            key_name: config.key_name,
            project_id: config.project_id,
            location: config.location,
            key_ring: config.key_ring,
            crypto_key: config.crypto_key,
        })
    }

    /// Create provider with explicit parameters
    pub async fn from_params(
        project_id: String,
        location: String,
        key_ring: String,
        crypto_key: String,
    ) -> Result<Self, AutoUnsealError> {
        let key_name = format!(
            "projects/{}/locations/{}/keyRings/{}/cryptoKeys/{}",
            project_id, location, key_ring, crypto_key
        );

        let config = GcpKmsConfig {
            key_name,
            project_id,
            location,
            key_ring,
            crypto_key,
            credentials_file: None,
        };

        Self::new(config).await
    }

    fn validate_key_name(&self) -> Result<(), AutoUnsealError> {
        let key_name = &self.key_name;

        if !key_name.starts_with("projects/") {
            return Err(AutoUnsealError::InvalidConfiguration(format!(
                "Invalid key name: {}",
                key_name
            )));
        }

        let parts: Vec<&str> = key_name.split('/').collect();
        if parts.len() != 8 {
            return Err(AutoUnsealError::InvalidConfiguration(format!(
                "Invalid key name format: {}",
                key_name
            )));
        }

        Ok(())
    }
}

#[cfg(feature = "gcp-kms")]
#[async_trait]
impl AutoUnsealProvider for GcpKmsProvider {
    fn name(&self) -> &str {
        "gcp-kms"
    }

    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        debug!(plaintext_len = plaintext.len(), "Encrypting with GCP KMS");

        self.validate_key_name()?;

        let request = EncryptRequest {
            plaintext: Some(base64::encode(plaintext)),
            additional_authenticated_data: None,
            plaintext_crc32c: None,
            additional_authenticated_data_crc32c: None,
        };

        let response = self
            .hub
            .projects()
            .locations_key_rings_crypto_keys_encrypt(request, &self.key_name)
            .doit()
            .await
            .map_err(|e| {
                error!(error = %e, "GCP KMS encrypt failed");
                let error_msg = e.to_string();
                if error_msg.contains("NOT_FOUND") {
                    AutoUnsealError::KeyNotFound(self.key_name.clone())
                } else if error_msg.contains("PERMISSION_DENIED") {
                    AutoUnsealError::PermissionDenied(format!(
                        "No permission to encrypt with key: {}",
                        self.key_name
                    ))
                } else {
                    AutoUnsealError::EncryptionFailed(format!("GCP KMS error: {}", error_msg))
                }
            })?;

        let (_, encrypt_response) = response;
        let ciphertext_base64 = encrypt_response.ciphertext.ok_or_else(|| {
            error!("No ciphertext returned");
            AutoUnsealError::EncryptionFailed("No ciphertext returned".to_string())
        })?;

        let ciphertext = base64::decode(&ciphertext_base64).map_err(|e| {
            error!(error = %e, "Failed to decode ciphertext");
            AutoUnsealError::EncryptionFailed(format!("Decode error: {}", e))
        })?;

        info!(ciphertext_len = ciphertext.len(), "Successfully encrypted");

        Ok(ciphertext)
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        debug!(ciphertext_len = ciphertext.len(), "Decrypting with GCP KMS");

        let request = DecryptRequest {
            ciphertext: Some(base64::encode(ciphertext)),
            additional_authenticated_data: None,
            ciphertext_crc32c: None,
            additional_authenticated_data_crc32c: None,
        };

        let response = self
            .hub
            .projects()
            .locations_key_rings_crypto_keys_decrypt(request, &self.key_name)
            .doit()
            .await
            .map_err(|e| {
                error!(error = %e, "GCP KMS decrypt failed");
                let error_msg = e.to_string();
                if error_msg.contains("NOT_FOUND") {
                    AutoUnsealError::KeyNotFound(self.key_name.clone())
                } else if error_msg.contains("PERMISSION_DENIED") {
                    AutoUnsealError::PermissionDenied(format!(
                        "No permission to decrypt with key: {}",
                        self.key_name
                    ))
                } else if error_msg.contains("INVALID_ARGUMENT") {
                    AutoUnsealError::InvalidCiphertext("Invalid ciphertext".to_string())
                } else {
                    AutoUnsealError::DecryptionFailed(format!("GCP KMS error: {}", error_msg))
                }
            })?;

        let (_, decrypt_response) = response;
        let plaintext_base64 = decrypt_response.plaintext.ok_or_else(|| {
            error!("No plaintext returned");
            AutoUnsealError::DecryptionFailed("No plaintext returned".to_string())
        })?;

        let plaintext = base64::decode(&plaintext_base64).map_err(|e| {
            error!(error = %e, "Failed to decode plaintext");
            AutoUnsealError::DecryptionFailed(format!("Decode error: {}", e))
        })?;

        info!(plaintext_len = plaintext.len(), "Successfully decrypted");

        Ok(plaintext)
    }

    async fn health_check(&self) -> Result<(), SecretonError> {
        debug!("Performing GCP KMS health check");

        self.validate_key_name()?;

        let response = self
            .hub
            .projects()
            .locations_key_rings_crypto_keys_get(&self.key_name)
            .doit()
            .await
            .map_err(|e| {
                error!(error = %e, "Health check failed");
                let error_msg = e.to_string();
                if error_msg.contains("NOT_FOUND") {
                    AutoUnsealError::KeyNotFound(self.key_name.clone())
                } else if error_msg.contains("PERMISSION_DENIED") {
                    AutoUnsealError::PermissionDenied(format!(
                        "No permission to access key: {}",
                        self.key_name
                    ))
                } else {
                    AutoUnsealError::HealthCheckFailed(format!("GCP KMS error: {}", error_msg))
                }
            })?;

        let (_, crypto_key) = response;

        let state = crypto_key
            .primary
            .as_ref()
            .and_then(|v| v.state.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("UNKNOWN");

        if state != "ENABLED" {
            error!(state = %state, "Key not enabled");
            return Err(AutoUnsealError::ProviderError(format!(
                "Key is in state {}, expected ENABLED",
                state
            ))
            .into());
        }

        info!("GCP KMS health check passed");

        Ok(())
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata::new("gcp-kms".to_string(), self.key_name.clone())
            .with_region(self.location.clone())
    }
}
