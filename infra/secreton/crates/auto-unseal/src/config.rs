//! Configuration structures for auto-unseal

use serde::{Deserialize, Serialize};

/// Auto-unseal configuration
///
/// This configuration is stored in the bootstrap config (secreton.toml)
/// and specifies which auto-unseal provider to use and its parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "kebab-case")]
pub enum AutoUnsealConfig {
    /// Transit auto-unseal using another Secreton instance
    #[cfg(feature = "transit")]
    Transit(TransitConfig),

    /// AWS KMS auto-unseal
    #[cfg(feature = "aws-kms")]
    AwsKms(AwsKmsConfig),

    /// GCP KMS auto-unseal
    #[cfg(feature = "gcp-kms")]
    GcpKms(GcpKmsConfig),

    /// Azure Key Vault auto-unseal
    #[cfg(feature = "azure-kv")]
    AzureKeyVault(AzureKeyVaultConfig),
}

/// Transit provider configuration
#[cfg(feature = "transit")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitConfig {
    /// Secreton gRPC endpoint (e.g., "https://secreton.internal:50052")
    pub endpoint: String,

    /// Transit key name
    pub key_name: String,

    /// Authentication token
    ///
    /// This should be a long-lived token with permission to use the transit key.
    /// Consider using a token with a renewable lease.
    pub token: String,

    /// TLS certificate path (optional, for mTLS)
    pub tls_cert_path: Option<String>,

    /// TLS key path (optional, for mTLS)
    pub tls_key_path: Option<String>,

    /// TLS CA certificate path (optional, for custom CA)
    pub tls_ca_path: Option<String>,

    /// Connection timeout in seconds (default: 30)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

/// AWS KMS provider configuration
#[cfg(feature = "aws-kms")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsKmsConfig {
    /// KMS key ID or ARN
    ///
    /// Examples:
    /// - Key ID: "1234abcd-12ab-34cd-56ef-1234567890ab"
    /// - Key ARN: "arn:aws:kms:us-east-1:123456789012:key/1234abcd-12ab-34cd-56ef-1234567890ab"
    /// - Alias: "alias/my-key"
    pub key_id: String,

    /// AWS region (e.g., "us-east-1")
    pub region: String,

    /// Custom endpoint (optional, for testing or custom deployments)
    pub endpoint: Option<String>,

    /// Access key ID (optional, uses IAM role if not provided)
    pub access_key_id: Option<String>,

    /// Secret access key (optional, uses IAM role if not provided)
    pub secret_access_key: Option<String>,

    /// Session token (optional, for temporary credentials)
    pub session_token: Option<String>,
}

/// GCP KMS provider configuration
#[cfg(feature = "gcp-kms")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpKmsConfig {
    /// KMS key resource name
    ///
    /// Format: "projects/{project}/locations/{location}/keyRings/{keyRing}/cryptoKeys/{cryptoKey}"
    pub key_name: String,

    /// GCP project ID
    pub project_id: String,

    /// Location (e.g., "us-east1", "global")
    pub location: String,

    /// Key ring name
    pub key_ring: String,

    /// Crypto key name
    pub crypto_key: String,

    /// Service account key file path (optional, uses default credentials if not provided)
    pub credentials_file: Option<String>,
}

/// Azure Key Vault provider configuration
#[cfg(feature = "azure-kv")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureKeyVaultConfig {
    /// Key Vault name
    pub vault_name: String,

    /// Key name
    pub key_name: String,

    /// Key version (optional, uses latest if not provided)
    pub key_version: Option<String>,

    /// Tenant ID (optional, uses default if not provided)
    pub tenant_id: Option<String>,

    /// Client ID (optional, uses managed identity if not provided)
    pub client_id: Option<String>,

    /// Client secret (optional, uses managed identity if not provided)
    pub client_secret: Option<String>,
}

/// Fallback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// Enable fallback to manual unseal if auto-unseal fails
    #[serde(default = "default_true")]
    pub fallback_to_manual: bool,

    /// Maximum retry attempts before falling back
    #[serde(default = "default_retry_attempts")]
    pub max_retries: u32,

    /// Initial retry delay in seconds (exponential backoff)
    #[serde(default = "default_initial_retry_delay")]
    pub initial_retry_delay_secs: u64,

    /// Maximum retry delay in seconds (cap for exponential backoff)
    #[serde(default = "default_max_retry_delay")]
    pub max_retry_delay_secs: u64,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            fallback_to_manual: true,
            max_retries: 5,
            initial_retry_delay_secs: 1,
            max_retry_delay_secs: 16,
        }
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_true() -> bool {
    true
}

fn default_retry_attempts() -> u32 {
    5
}

fn default_initial_retry_delay() -> u64 {
    1
}

fn default_max_retry_delay() -> u64 {
    16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "transit")]
    fn test_transit_config_deserialization() {
        let toml = r#"
            provider = "transit"
            endpoint = "https://secreton.internal:50052"
            key_name = "auto-unseal-key"
            token = "s.token123"
            timeout_secs = 60
        "#;

        let config: AutoUnsealConfig = toml::from_str(toml).unwrap();

        match config {
            AutoUnsealConfig::Transit(transit) => {
                assert_eq!(transit.endpoint, "https://secreton.internal:50052");
                assert_eq!(transit.key_name, "auto-unseal-key");
                assert_eq!(transit.token, "s.token123");
                assert_eq!(transit.timeout_secs, 60);
            }
            #[allow(unreachable_patterns)]
            _ => panic!("Expected Transit config"),
        }
    }

    #[test]
    #[cfg(feature = "aws-kms")]
    fn test_aws_kms_config_deserialization() {
        let toml = r#"
            provider = "aws-kms"
            key_id = "alias/my-key"
            region = "us-east-1"
        "#;

        let config: AutoUnsealConfig = toml::from_str(toml).unwrap();

        match config {
            AutoUnsealConfig::AwsKms(aws) => {
                assert_eq!(aws.key_id, "alias/my-key");
                assert_eq!(aws.region, "us-east-1");
                assert_eq!(aws.endpoint, None);
            }
            #[allow(unreachable_patterns)]
            _ => panic!("Expected AWS KMS config"),
        }
    }

    #[test]
    fn test_fallback_config_default() {
        let config = FallbackConfig::default();

        assert!(config.fallback_to_manual);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_retry_delay_secs, 1);
        assert_eq!(config.max_retry_delay_secs, 16);
    }
}
