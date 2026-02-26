//! AWS KMS auto-unseal provider
//!
//! This provider uses AWS Key Management Service (KMS) for auto-unsealing.
//! It supports:
//! - IAM role authentication (recommended for EC2/ECS/EKS)
//! - Access key authentication (for development/testing)
//! - Custom endpoints (for testing or AWS-compatible services)
//! - Multiple key identifier formats (key ID, ARN, alias)
//!
//! # Authentication Methods
//!
//! ## IAM Role (Recommended for Production)
//! ```toml
//! [auto_unseal]
//! provider = "aws-kms"
//! key_id = "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012"
//! region = "us-east-1"
//! ```
//!
//! ## Access Key (Development/Testing)
//! ```toml
//! [auto_unseal]
//! provider = "aws-kms"
//! key_id = "alias/secreton-auto-unseal"
//! region = "us-east-1"
//! access_key_id = "AKIAIOSFODNN7EXAMPLE"
//! secret_access_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
//! ```

#[cfg(feature = "aws-kms")]
use async_trait::async_trait;
#[cfg(feature = "aws-kms")]
use aws_config::BehaviorVersion;
#[cfg(feature = "aws-kms")]
use aws_sdk_kms::{
    Client as KmsClient,
    config::{Credentials, Region},
    primitives::Blob,
};
#[cfg(feature = "aws-kms")]
use secreton_core::error::SecretonError;
#[cfg(feature = "aws-kms")]
use tracing::{debug, error, info, warn};

#[cfg(feature = "aws-kms")]
use crate::config::AwsKmsConfig;
#[cfg(feature = "aws-kms")]
use crate::error::AutoUnsealError;
#[cfg(feature = "aws-kms")]
use crate::{AutoUnsealProvider, ProviderMetadata};

/// AWS KMS auto-unseal provider
///
/// Uses AWS Key Management Service to encrypt/decrypt the master key.
/// Supports IAM role authentication (recommended) and access key authentication.
#[cfg(feature = "aws-kms")]
#[derive(Clone, Debug)]
pub struct AwsKmsProvider {
    client: KmsClient,
    key_id: String,
    region: String,
    endpoint: Option<String>,
}

#[cfg(feature = "aws-kms")]
impl AwsKmsProvider {
    /// Create a new AWS KMS provider
    ///
    /// # Arguments
    ///
    /// * `config` - AWS KMS configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration is invalid
    /// - AWS credentials cannot be loaded
    /// - The KMS client cannot be created
    ///
    /// # Example
    ///
    /// ```no_run
    /// use secreton_auto_unseal::aws_kms::AwsKmsProvider;
    /// use secreton_auto_unseal::config::AwsKmsConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = AwsKmsConfig {
    ///     key_id: "alias/secreton-auto-unseal".to_string(),
    ///     region: "us-east-1".to_string(),
    ///     endpoint: None,
    ///     access_key_id: None,
    ///     secret_access_key: None,
    ///     session_token: None,
    /// };
    ///
    /// let provider = AwsKmsProvider::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: AwsKmsConfig) -> Result<Self, AutoUnsealError> {
        info!(
            key_id = %config.key_id,
            region = %config.region,
            "Initializing AWS KMS auto-unseal provider"
        );

        // Validate configuration
        if config.key_id.is_empty() {
            return Err(AutoUnsealError::InvalidConfiguration(
                "key_id cannot be empty".to_string(),
            ));
        }

        if config.region.is_empty() {
            return Err(AutoUnsealError::InvalidConfiguration(
                "region cannot be empty".to_string(),
            ));
        }

        // Build AWS config
        let mut aws_config_builder = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()));

        // Use explicit credentials if provided, otherwise use default credential chain
        if let (Some(access_key_id), Some(secret_access_key)) =
            (&config.access_key_id, &config.secret_access_key)
        {
            debug!("Using explicit AWS credentials");

            let credentials = if let Some(session_token) = &config.session_token {
                Credentials::new(
                    access_key_id,
                    secret_access_key,
                    Some(session_token.clone()),
                    None,
                    "secreton-auto-unseal",
                )
            } else {
                Credentials::new(
                    access_key_id,
                    secret_access_key,
                    None,
                    None,
                    "secreton-auto-unseal",
                )
            };

            aws_config_builder = aws_config_builder.credentials_provider(credentials);
        } else {
            debug!("Using default AWS credential chain (IAM role, environment, etc.)");
        }

        let aws_config = aws_config_builder.load().await;

        // Create KMS client
        let mut kms_config_builder = aws_sdk_kms::config::Builder::from(&aws_config);

        // Set custom endpoint if provided
        if let Some(endpoint) = &config.endpoint {
            debug!(endpoint = %endpoint, "Using custom KMS endpoint");
            kms_config_builder = kms_config_builder.endpoint_url(endpoint);
        }

        let kms_config = kms_config_builder.build();
        let client = KmsClient::from_conf(kms_config);

        info!(
            key_id = %config.key_id,
            region = %config.region,
            "AWS KMS auto-unseal provider initialized successfully"
        );

        Ok(Self {
            client,
            key_id: config.key_id,
            region: config.region,
            endpoint: config.endpoint,
        })
    }

    /// Create a new AWS KMS provider with explicit parameters
    ///
    /// This is a convenience method for creating a provider without a config struct.
    ///
    /// # Arguments
    ///
    /// * `key_id` - KMS key ID, ARN, or alias
    /// * `region` - AWS region
    ///
    /// # Example
    ///
    /// ```no_run
    /// use secreton_auto_unseal::aws_kms::AwsKmsProvider;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = AwsKmsProvider::from_params(
    ///     "alias/secreton-auto-unseal".to_string(),
    ///     "us-east-1".to_string(),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_params(key_id: String, region: String) -> Result<Self, AutoUnsealError> {
        let config = AwsKmsConfig {
            key_id,
            region,
            endpoint: None,
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
        };

        Self::new(config).await
    }

    /// Validate the key ID format
    ///
    /// AWS KMS supports multiple key identifier formats:
    /// - Key ID: "1234abcd-12ab-34cd-56ef-1234567890ab"
    /// - Key ARN: "arn:aws:kms:us-east-1:123456789012:key/1234abcd-12ab-34cd-56ef-1234567890ab"
    /// - Alias name: "alias/my-key"
    /// - Alias ARN: "arn:aws:kms:us-east-1:123456789012:alias/my-key"
    fn validate_key_id(&self) -> Result<(), AutoUnsealError> {
        let key_id = &self.key_id;

        // Check if it's an ARN
        if key_id.starts_with("arn:aws:kms:") {
            if !key_id.contains(":key/") && !key_id.contains(":alias/") {
                return Err(AutoUnsealError::InvalidConfiguration(format!(
                    "Invalid KMS key ARN format: {}",
                    key_id
                )));
            }
            return Ok(());
        }

        // Check if it's an alias
        if key_id.starts_with("alias/") {
            if key_id.len() <= 6 {
                return Err(AutoUnsealError::InvalidConfiguration(
                    "Alias name cannot be empty".to_string(),
                ));
            }
            return Ok(());
        }

        // Check if it's a key ID (UUID format)
        if key_id.len() == 36 && key_id.chars().filter(|c| *c == '-').count() == 4 {
            return Ok(());
        }

        warn!(
            key_id = %key_id,
            "Key ID format not recognized, but will attempt to use it anyway"
        );

        Ok(())
    }
}

#[cfg(feature = "aws-kms")]
#[async_trait]
impl AutoUnsealProvider for AwsKmsProvider {
    fn name(&self) -> &str {
        "aws-kms"
    }

    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        debug!(
            key_id = %self.key_id,
            plaintext_len = plaintext.len(),
            "Encrypting master key with AWS KMS"
        );

        // Validate key ID format
        self.validate_key_id().map_err(|e| {
            error!(error = %e, "Invalid key ID format");
            e
        })?;

        // Call KMS Encrypt API
        let response = self
            .client
            .encrypt()
            .key_id(&self.key_id)
            .plaintext(Blob::new(plaintext))
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "AWS KMS encrypt failed");

                // Map AWS SDK errors to AutoUnsealError
                let error_msg = e.to_string();
                if error_msg.contains("NotFoundException") || error_msg.contains("not found") {
                    AutoUnsealError::KeyNotFound(self.key_id.clone())
                } else if error_msg.contains("AccessDeniedException")
                    || error_msg.contains("UnauthorizedException")
                {
                    AutoUnsealError::PermissionDenied(format!(
                        "No permission to encrypt with key: {}",
                        self.key_id
                    ))
                } else if error_msg.contains("InvalidKeyUsageException") {
                    AutoUnsealError::ProviderError(format!(
                        "Key {} is not enabled for encryption",
                        self.key_id
                    ))
                } else if error_msg.contains("DisabledException") {
                    AutoUnsealError::ProviderError(format!("Key {} is disabled", self.key_id))
                } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
                    AutoUnsealError::Timeout("AWS KMS encrypt operation timed out".to_string())
                } else {
                    AutoUnsealError::EncryptionFailed(format!("AWS KMS error: {}", error_msg))
                }
            })?;

        // Extract ciphertext blob
        let ciphertext = response
            .ciphertext_blob()
            .ok_or_else(|| {
                error!("AWS KMS returned no ciphertext");
                AutoUnsealError::EncryptionFailed("No ciphertext returned from AWS KMS".to_string())
            })?
            .as_ref()
            .to_vec();

        info!(
            key_id = %self.key_id,
            ciphertext_len = ciphertext.len(),
            "Successfully encrypted master key with AWS KMS"
        );

        Ok(ciphertext)
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        debug!(
            key_id = %self.key_id,
            ciphertext_len = ciphertext.len(),
            "Decrypting master key with AWS KMS"
        );

        // Call KMS Decrypt API
        // Note: key_id is optional for decrypt, as it's embedded in the ciphertext
        // But we provide it for better error messages and validation
        let response = self
            .client
            .decrypt()
            .key_id(&self.key_id)
            .ciphertext_blob(Blob::new(ciphertext))
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "AWS KMS decrypt failed");

                // Map AWS SDK errors to AutoUnsealError
                let error_msg = e.to_string();
                if error_msg.contains("NotFoundException") || error_msg.contains("not found") {
                    AutoUnsealError::KeyNotFound(self.key_id.clone())
                } else if error_msg.contains("AccessDeniedException")
                    || error_msg.contains("UnauthorizedException")
                {
                    AutoUnsealError::PermissionDenied(format!(
                        "No permission to decrypt with key: {}",
                        self.key_id
                    ))
                } else if error_msg.contains("InvalidCiphertextException") {
                    AutoUnsealError::InvalidCiphertext(
                        "Ciphertext is invalid or was not encrypted with this key".to_string(),
                    )
                } else if error_msg.contains("DisabledException") {
                    AutoUnsealError::ProviderError(format!("Key {} is disabled", self.key_id))
                } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
                    AutoUnsealError::Timeout("AWS KMS decrypt operation timed out".to_string())
                } else {
                    AutoUnsealError::DecryptionFailed(format!("AWS KMS error: {}", error_msg))
                }
            })?;

        // Extract plaintext
        let plaintext = response
            .plaintext()
            .ok_or_else(|| {
                error!("AWS KMS returned no plaintext");
                AutoUnsealError::DecryptionFailed("No plaintext returned from AWS KMS".to_string())
            })?
            .as_ref()
            .to_vec();

        // Verify the key ID matches (if returned)
        if let Some(returned_key_id) = response.key_id() {
            if !returned_key_id.contains(&self.key_id) && !self.key_id.contains(returned_key_id) {
                warn!(
                    expected_key_id = %self.key_id,
                    returned_key_id = %returned_key_id,
                    "Key ID mismatch in decrypt response"
                );
            }
        }

        info!(
            key_id = %self.key_id,
            plaintext_len = plaintext.len(),
            "Successfully decrypted master key with AWS KMS"
        );

        Ok(plaintext)
    }

    async fn health_check(&self) -> Result<(), SecretonError> {
        debug!(
            key_id = %self.key_id,
            region = %self.region,
            "Performing AWS KMS health check"
        );

        // Validate key ID format first
        self.validate_key_id().map_err(|e| {
            error!(error = %e, "Invalid key ID format in health check");
            e
        })?;

        // Use DescribeKey to verify the key exists and is accessible
        let response = self
            .client
            .describe_key()
            .key_id(&self.key_id)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "AWS KMS health check failed");

                let error_msg = e.to_string();
                if error_msg.contains("NotFoundException") || error_msg.contains("not found") {
                    AutoUnsealError::KeyNotFound(self.key_id.clone())
                } else if error_msg.contains("AccessDeniedException")
                    || error_msg.contains("UnauthorizedException")
                {
                    AutoUnsealError::PermissionDenied(format!(
                        "No permission to describe key: {}",
                        self.key_id
                    ))
                } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
                    AutoUnsealError::Timeout("AWS KMS health check timed out".to_string())
                } else {
                    AutoUnsealError::HealthCheckFailed(format!("AWS KMS error: {}", error_msg))
                }
            })?;

        // Verify key metadata
        let key_metadata = response.key_metadata().ok_or_else(|| {
            error!("AWS KMS returned no key metadata");
            AutoUnsealError::HealthCheckFailed("No key metadata returned".to_string())
        })?;

        // Check if key is enabled
        if !key_metadata.enabled() {
            error!(key_id = %self.key_id, "KMS key is disabled");
            return Err(AutoUnsealError::ProviderError(format!(
                "KMS key {} is disabled",
                self.key_id
            ))
            .into());
        }

        // Check key state
        let key_state = key_metadata.key_state();
        if !matches!(
            key_state,
            Some(aws_sdk_kms::types::KeyState::Enabled)
                | Some(aws_sdk_kms::types::KeyState::Updating)
        ) {
            error!(
                key_id = %self.key_id,
                key_state = ?key_state,
                "KMS key is not in a usable state"
            );
            return Err(AutoUnsealError::ProviderError(format!(
                "KMS key {} is in state {:?}, expected Enabled",
                self.key_id, key_state
            ))
            .into());
        }

        // Verify key usage allows encryption/decryption
        let key_usage = key_metadata.key_usage();
        if !matches!(
            key_usage,
            Some(aws_sdk_kms::types::KeyUsageType::EncryptDecrypt)
        ) {
            error!(
                key_id = %self.key_id,
                key_usage = ?key_usage,
                "KMS key does not support encryption/decryption"
            );
            return Err(AutoUnsealError::ProviderError(format!(
                "KMS key {} has usage {:?}, expected ENCRYPT_DECRYPT",
                self.key_id, key_usage
            ))
            .into());
        }

        info!(
            key_id = %self.key_id,
            key_arn = ?key_metadata.arn(),
            key_state = ?key_state,
            "AWS KMS health check passed"
        );

        Ok(())
    }

    fn metadata(&self) -> ProviderMetadata {
        let mut metadata = ProviderMetadata::new("aws-kms".to_string(), self.key_id.clone())
            .with_region(self.region.clone());

        if let Some(endpoint) = &self.endpoint {
            metadata = metadata.with_endpoint(endpoint.clone());
        }

        metadata
    }
}

#[cfg(all(test, feature = "aws-kms"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_aws_kms_provider_creation() {
        let config = AwsKmsConfig {
            key_id: "alias/secreton-auto-unseal".to_string(),
            region: "us-east-1".to_string(),
            endpoint: None,
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
        };

        let provider = AwsKmsProvider::new(config).await.unwrap();

        assert_eq!(provider.name(), "aws-kms");
        assert_eq!(provider.key_id, "alias/secreton-auto-unseal");
        assert_eq!(provider.region, "us-east-1");
    }

    #[tokio::test]
    async fn test_aws_kms_provider_invalid_config() {
        let config = AwsKmsConfig {
            key_id: "".to_string(),
            region: "us-east-1".to_string(),
            endpoint: None,
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
        };

        let result = AwsKmsProvider::new(config).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AutoUnsealError::InvalidConfiguration(_)
        ));
    }

    #[tokio::test]
    async fn test_aws_kms_provider_from_params() {
        let provider =
            AwsKmsProvider::from_params("alias/test-key".to_string(), "us-west-2".to_string())
                .await
                .unwrap();

        assert_eq!(provider.key_id, "alias/test-key");
        assert_eq!(provider.region, "us-west-2");
    }

    #[tokio::test]
    async fn test_aws_kms_provider_metadata() {
        let provider =
            AwsKmsProvider::from_params("alias/test-key".to_string(), "eu-west-1".to_string())
                .await
                .unwrap();

        let metadata = provider.metadata();
        assert_eq!(metadata.provider_type, "aws-kms");
        assert_eq!(metadata.key_id, "alias/test-key");
        assert_eq!(metadata.region, Some("eu-west-1".to_string()));
    }

    #[test]
    fn test_key_id_validation() {
        // Create a minimal AWS config for testing
        let aws_config = aws_sdk_kms::Config::builder()
            .behavior_version(aws_config::BehaviorVersion::latest())
            .build();

        // Test with alias
        let provider = AwsKmsProvider {
            client: KmsClient::from_conf(aws_config.clone()),
            key_id: "alias/my-key".to_string(),
            region: "us-east-1".to_string(),
            endpoint: None,
        };
        assert!(provider.validate_key_id().is_ok());

        // Test with key ID (UUID)
        let provider = AwsKmsProvider {
            client: KmsClient::from_conf(aws_config.clone()),
            key_id: "12345678-1234-1234-1234-123456789012".to_string(),
            region: "us-east-1".to_string(),
            endpoint: None,
        };
        assert!(provider.validate_key_id().is_ok());

        // Test with key ARN
        let provider = AwsKmsProvider {
            client: KmsClient::from_conf(aws_config.clone()),
            key_id: "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012"
                .to_string(),
            region: "us-east-1".to_string(),
            endpoint: None,
        };
        assert!(provider.validate_key_id().is_ok());

        // Test with alias ARN
        let provider = AwsKmsProvider {
            client: KmsClient::from_conf(aws_config.clone()),
            key_id: "arn:aws:kms:us-east-1:123456789012:alias/my-key".to_string(),
            region: "us-east-1".to_string(),
            endpoint: None,
        };
        assert!(provider.validate_key_id().is_ok());

        // Test with invalid alias
        let provider = AwsKmsProvider {
            client: KmsClient::from_conf(aws_config),
            key_id: "alias/".to_string(),
            region: "us-east-1".to_string(),
            endpoint: None,
        };
        assert!(provider.validate_key_id().is_err());
    }
}
