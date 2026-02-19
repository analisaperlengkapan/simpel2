//! Transit auto-unseal provider
//!
//! This provider uses another Secreton instance's Transit engine for auto-unsealing.
//! This is useful for:
//! - Development and testing
//! - Air-gapped deployments where cloud KMS is not available
//! - Hierarchical unsealing (primary Secreton unseals secondary instances)

#[cfg(feature = "transit")]
use async_trait::async_trait;
#[cfg(feature = "transit")]
use secreton_core::error::SecretonError;
#[cfg(feature = "transit")]
use tracing::{debug, error, info};

#[cfg(feature = "transit")]
use crate::{AutoUnsealProvider, ProviderMetadata};
#[cfg(feature = "transit")]
use crate::config::TransitConfig;
#[cfg(feature = "transit")]
use crate::error::AutoUnsealError;

// gRPC imports
#[cfg(feature = "transit")]
use tonic::transport::{Channel, ClientTlsConfig, Certificate, Identity};
#[cfg(feature = "transit")]
use tonic::metadata::MetadataValue;

// Base64 encoding
#[cfg(feature = "transit")]
use base64::Engine;

// Generated proto types - import directly from secreton-grpc
#[cfg(feature = "transit")]
use secreton_grpc::generated::secreton::v1::{
    secreton_service_client::SecretonServiceClient,
    EncryptRequest,
    DecryptRequest,
};

/// Transit auto-unseal provider
///
/// Uses another Secreton instance's Transit engine to encrypt/decrypt the master key.
#[cfg(feature = "transit")]
#[derive(Clone)]
pub struct TransitProvider {
    endpoint: String,
    key_name: String,
    token: String,
    timeout_secs: u64,
    tls_cert_path: Option<String>,
    tls_key_path: Option<String>,
    tls_ca_path: Option<String>,
}

// Type alias for the gRPC client with interceptor
#[cfg(feature = "transit")]
type TransitClient = SecretonServiceClient<tonic::transport::Channel>;

#[cfg(feature = "transit")]
impl TransitProvider {
    /// Create a new Transit provider
    ///
    /// # Arguments
    ///
    /// * `config` - Transit configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or connection fails
    pub async fn new(config: TransitConfig) -> Result<Self, AutoUnsealError> {
        info!(
            endpoint = %config.endpoint,
            key_name = %config.key_name,
            "Initializing Transit auto-unseal provider"
        );

        // Validate configuration
        if config.endpoint.is_empty() {
            return Err(AutoUnsealError::InvalidConfiguration(
                "endpoint cannot be empty".to_string(),
            ));
        }

        if config.key_name.is_empty() {
            return Err(AutoUnsealError::InvalidConfiguration(
                "key_name cannot be empty".to_string(),
            ));
        }

        if config.token.is_empty() {
            return Err(AutoUnsealError::InvalidConfiguration(
                "token cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            endpoint: config.endpoint,
            key_name: config.key_name,
            token: config.token,
            timeout_secs: config.timeout_secs,
            tls_cert_path: config.tls_cert_path,
            tls_key_path: config.tls_key_path,
            tls_ca_path: config.tls_ca_path,
        })
    }

    /// Create a new Transit provider with explicit parameters
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Secreton gRPC endpoint
    /// * `key_name` - Transit key name
    /// * `token` - Authentication token
    pub async fn from_params(
        endpoint: String,
        key_name: String,
        token: String,
    ) -> Result<Self, AutoUnsealError> {
        let config = TransitConfig {
            endpoint,
            key_name,
            token,
            tls_cert_path: None,
            tls_key_path: None,
            tls_ca_path: None,
            timeout_secs: 30,
        };

        Self::new(config).await
    }

    /// Create a gRPC client with mTLS configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be created
    async fn create_client(&self) -> Result<TransitClient, AutoUnsealError> {
        debug!(endpoint = %self.endpoint, "Creating gRPC client");

        // Parse endpoint
        let endpoint = Channel::from_shared(self.endpoint.clone())
            .map_err(|e| {
                AutoUnsealError::InvalidConfiguration(format!("Invalid endpoint: {}", e))
            })?;

        // Configure TLS if certificates are provided
        let channel = if let Some(ca_path) = &self.tls_ca_path {
            debug!(ca_path = %ca_path, "Configuring mTLS");

            // Load CA certificate
            let ca_cert = tokio::fs::read(ca_path).await.map_err(|e| {
                AutoUnsealError::InvalidConfiguration(format!(
                    "Failed to read CA certificate from {}: {}",
                    ca_path, e
                ))
            })?;

            let ca = Certificate::from_pem(ca_cert);

            let mut tls_config = ClientTlsConfig::new().ca_certificate(ca);

            // Load client certificate and key for mTLS
            if let (Some(cert_path), Some(key_path)) = (&self.tls_cert_path, &self.tls_key_path) {
                debug!(
                    cert_path = %cert_path,
                    key_path = %key_path,
                    "Loading client certificate for mTLS"
                );

                let cert = tokio::fs::read(cert_path).await.map_err(|e| {
                    AutoUnsealError::InvalidConfiguration(format!(
                        "Failed to read client certificate from {}: {}",
                        cert_path, e
                    ))
                })?;

                let key = tokio::fs::read(key_path).await.map_err(|e| {
                    AutoUnsealError::InvalidConfiguration(format!(
                        "Failed to read client key from {}: {}",
                        key_path, e
                    ))
                })?;

                let identity = Identity::from_pem(cert, key);
                tls_config = tls_config.identity(identity);
            }

            endpoint
                .tls_config(tls_config)
                .map_err(|e| {
                    AutoUnsealError::InvalidConfiguration(format!("TLS config failed: {}", e))
                })?
                .connect()
                .await
                .map_err(|e| {
                    AutoUnsealError::NetworkError(format!("Connection failed: {}", e))
                })?
        } else {
            // No TLS configuration, connect directly
            debug!("Connecting without TLS");
            endpoint.connect().await.map_err(|e| {
                AutoUnsealError::NetworkError(format!("Connection failed: {}", e))
            })?
        };

        // Create client (we'll add auth headers per-request)
        let client = SecretonServiceClient::new(channel);

        debug!("gRPC client created successfully");
        Ok(client)
    }

    /// Add authentication header to a request
    fn add_auth_header<T>(&self, mut request: tonic::Request<T>) -> tonic::Request<T> {
        if let Ok(token_value) = MetadataValue::try_from(format!("Bearer {}", self.token)) {
            request.metadata_mut().insert("authorization", token_value);
        }
        request
    }
}

#[cfg(feature = "transit")]
#[async_trait]
impl AutoUnsealProvider for TransitProvider {
    fn name(&self) -> &str {
        "transit"
    }

    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        debug!(
            key_name = %self.key_name,
            plaintext_len = plaintext.len(),
            "Encrypting master key with Transit provider"
        );

        // Create gRPC client
        let mut client = self.create_client().await.map_err(|e| {
            error!(error = %e, "Failed to create gRPC client");
            AutoUnsealError::EncryptionFailed(format!("Failed to create gRPC client: {}", e))
        })?;

        // Prepare encrypt request
        let request = self.add_auth_header(tonic::Request::new(EncryptRequest {
            key_name: self.key_name.clone(),
            plaintext: plaintext.to_vec(),
            context: None,
        }));

        // Call Transit encrypt
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            client.encrypt(request),
        )
        .await
        .map_err(|_| {
            error!("Transit encrypt operation timed out");
            AutoUnsealError::Timeout(format!(
                "Encrypt operation timed out after {} seconds",
                self.timeout_secs
            ))
        })?
        .map_err(|e| {
            error!(error = %e, "Transit encrypt failed");
            AutoUnsealError::EncryptionFailed(format!("gRPC call failed: {}", e))
        })?;

        let encrypt_response = response.into_inner();

        // Decode ciphertext from base64 (Transit returns base64-encoded ciphertext)
        let ciphertext = base64::engine::general_purpose::STANDARD
            .decode(&encrypt_response.ciphertext)
            .map_err(|e| {
                error!(error = %e, "Failed to decode ciphertext from base64");
                AutoUnsealError::EncryptionFailed(format!("Base64 decode failed: {}", e))
            })?;

        info!(
            key_name = %self.key_name,
            key_version = encrypt_response.key_version,
            ciphertext_len = ciphertext.len(),
            "Successfully encrypted master key with Transit provider"
        );

        Ok(ciphertext)
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        debug!(
            key_name = %self.key_name,
            ciphertext_len = ciphertext.len(),
            "Decrypting master key with Transit provider"
        );

        // Create gRPC client
        let mut client = self.create_client().await.map_err(|e| {
            error!(error = %e, "Failed to create gRPC client");
            AutoUnsealError::DecryptionFailed(format!("Failed to create gRPC client: {}", e))
        })?;

        // Encode ciphertext to base64 (Transit expects base64-encoded ciphertext)
        let ciphertext_b64 = base64::engine::general_purpose::STANDARD.encode(ciphertext);

        // Prepare decrypt request
        let request = self.add_auth_header(tonic::Request::new(DecryptRequest {
            key_name: self.key_name.clone(),
            ciphertext: ciphertext_b64,
            context: None,
        }));

        // Call Transit decrypt
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            client.decrypt(request),
        )
        .await
        .map_err(|_| {
            error!("Transit decrypt operation timed out");
            AutoUnsealError::Timeout(format!(
                "Decrypt operation timed out after {} seconds",
                self.timeout_secs
            ))
        })?
        .map_err(|e| {
            error!(error = %e, "Transit decrypt failed");
            AutoUnsealError::DecryptionFailed(format!("gRPC call failed: {}", e))
        })?;

        let decrypt_response = response.into_inner();
        let plaintext = decrypt_response.plaintext;

        info!(
            key_name = %self.key_name,
            plaintext_len = plaintext.len(),
            "Successfully decrypted master key with Transit provider"
        );

        Ok(plaintext)
    }

    async fn health_check(&self) -> Result<(), SecretonError> {
        debug!(
            endpoint = %self.endpoint,
            key_name = %self.key_name,
            "Performing Transit provider health check"
        );

        // Create gRPC client
        let mut client = self.create_client().await.map_err(|e| {
            error!(error = %e, "Failed to create gRPC client for health check");
            AutoUnsealError::HealthCheckFailed(format!("Failed to create gRPC client: {}", e))
        })?;

        // Try to encrypt a test payload to verify the key exists and is accessible
        let test_payload = b"health-check-test";
        let request = self.add_auth_header(tonic::Request::new(EncryptRequest {
            key_name: self.key_name.clone(),
            plaintext: test_payload.to_vec(),
            context: None,
        }));

        // Call Transit encrypt with timeout
        tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            client.encrypt(request),
        )
        .await
        .map_err(|_| {
            error!("Transit health check timed out");
            AutoUnsealError::Timeout(format!(
                "Health check timed out after {} seconds",
                self.timeout_secs
            ))
        })?
        .map_err(|e| {
            error!(error = %e, "Transit health check failed");
            match e.code() {
                tonic::Code::NotFound => AutoUnsealError::KeyNotFound(self.key_name.clone()),
                tonic::Code::Unauthenticated => {
                    AutoUnsealError::InvalidCredentials("Invalid token".to_string())
                }
                tonic::Code::PermissionDenied => {
                    AutoUnsealError::PermissionDenied(format!("No access to key: {}", self.key_name))
                }
                tonic::Code::Unavailable => {
                    AutoUnsealError::NetworkError(format!("Secreton unavailable at {}", self.endpoint))
                }
                _ => AutoUnsealError::HealthCheckFailed(format!("gRPC error: {}", e)),
            }
        })?;

        info!(
            endpoint = %self.endpoint,
            key_name = %self.key_name,
            "Transit provider health check passed"
        );

        Ok(())
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata::new("transit".to_string(), self.key_name.clone())
            .with_endpoint(self.endpoint.clone())
    }
}

#[cfg(all(test, feature = "transit"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transit_provider_creation() {
        let config = TransitConfig {
            endpoint: "https://secreton.internal:50052".to_string(),
            key_name: "auto-unseal-key".to_string(),
            token: "s.token123".to_string(),
            tls_cert_path: None,
            tls_key_path: None,
            tls_ca_path: None,
            timeout_secs: 30,
        };

        let provider = TransitProvider::new(config).await.unwrap();

        assert_eq!(provider.name(), "transit");
        assert_eq!(provider.key_name, "auto-unseal-key");
        assert_eq!(provider.endpoint, "https://secreton.internal:50052");
    }

    #[tokio::test]
    async fn test_transit_provider_invalid_config() {
        let config = TransitConfig {
            endpoint: "".to_string(),
            key_name: "auto-unseal-key".to_string(),
            token: "s.token123".to_string(),
            tls_cert_path: None,
            tls_key_path: None,
            tls_ca_path: None,
            timeout_secs: 30,
        };

        let result = TransitProvider::new(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transit_provider_metadata() {
        let provider = TransitProvider::from_params(
            "https://secreton.internal:50052".to_string(),
            "auto-unseal-key".to_string(),
            "s.token123".to_string(),
        )
        .await
        .unwrap();

        let metadata = provider.metadata();
        assert_eq!(metadata.provider_type, "transit");
        assert_eq!(metadata.key_id, "auto-unseal-key");
        assert_eq!(
            metadata.endpoint,
            Some("https://secreton.internal:50052".to_string())
        );
    }
}
