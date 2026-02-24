//! Error types for auto-unseal operations

use secreton_core::error::SecretonError;
use thiserror::Error;

/// Auto-unseal specific errors
#[derive(Error, Debug)]
pub enum AutoUnsealError {
    /// Provider not configured
    #[error("Auto-unseal provider not configured")]
    NotConfigured,

    /// Provider initialization failed
    #[error("Failed to initialize auto-unseal provider: {0}")]
    InitializationFailed(String),

    /// Encryption operation failed
    #[error("Auto-unseal encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption operation failed
    #[error("Auto-unseal decryption failed: {0}")]
    DecryptionFailed(String),

    /// Health check failed
    #[error("Auto-unseal health check failed: {0}")]
    HealthCheckFailed(String),

    /// Invalid configuration
    #[error("Invalid auto-unseal configuration: {0}")]
    InvalidConfiguration(String),

    /// Provider not found
    #[error("Auto-unseal provider '{0}' not found")]
    ProviderNotFound(String),

    /// Key not found
    #[error("Auto-unseal key not found: {0}")]
    KeyNotFound(String),

    /// Invalid credentials
    #[error("Invalid auto-unseal credentials: {0}")]
    InvalidCredentials(String),

    /// Network error
    #[error("Auto-unseal network error: {0}")]
    NetworkError(String),

    /// Timeout
    #[error("Auto-unseal operation timed out: {0}")]
    Timeout(String),

    /// Permission denied
    #[error("Auto-unseal permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid ciphertext
    #[error("Invalid ciphertext: {0}")]
    InvalidCiphertext(String),

    /// Provider-specific error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// All retry attempts exhausted
    #[error("Auto-unseal failed after {0} retry attempts")]
    RetriesExhausted(u32),

    /// Fallback to manual unseal
    #[error("Auto-unseal failed, falling back to manual unseal")]
    FallbackToManual,
}

impl From<AutoUnsealError> for secreton_core::error::SecretonError {
    fn from(err: AutoUnsealError) -> Self {
        use secreton_core::error::CoreError;

        match err {
            AutoUnsealError::NotConfigured => {
                SecretonError::Core(CoreError::configuration("Auto-unseal not configured"))
            }
            AutoUnsealError::InitializationFailed(msg) => SecretonError::Core(
                CoreError::configuration(format!("Auto-unseal initialization failed: {}", msg)),
            ),
            AutoUnsealError::EncryptionFailed(_msg) => SecretonError::EncryptionFailed,
            AutoUnsealError::DecryptionFailed(msg) => SecretonError::Core(CoreError::internal(
                format!("Auto-unseal decryption failed: {}", msg),
            )),
            AutoUnsealError::HealthCheckFailed(msg) => SecretonError::Core(
                CoreError::service_unavailable(format!("Auto-unseal health check failed: {}", msg)),
            ),
            AutoUnsealError::InvalidConfiguration(msg) => SecretonError::Core(
                CoreError::configuration(format!("Invalid auto-unseal configuration: {}", msg)),
            ),
            AutoUnsealError::ProviderNotFound(provider) => SecretonError::Core(
                CoreError::not_found(format!("Auto-unseal provider: {}", provider)),
            ),
            AutoUnsealError::KeyNotFound(_key) => SecretonError::KeyNotFound,
            AutoUnsealError::InvalidCredentials(msg) => SecretonError::Core(
                CoreError::authentication(format!("Invalid auto-unseal credentials: {}", msg)),
            ),
            AutoUnsealError::NetworkError(msg) => SecretonError::Core(CoreError::network(format!(
                "Auto-unseal network error: {}",
                msg
            ))),
            AutoUnsealError::Timeout(msg) => {
                SecretonError::Core(CoreError::timeout(format!("Auto-unseal: {}", msg)))
            }
            AutoUnsealError::PermissionDenied(msg) => SecretonError::Core(
                CoreError::authorization(format!("Auto-unseal permission denied: {}", msg)),
            ),
            AutoUnsealError::InvalidCiphertext(msg) => SecretonError::Core(CoreError::validation(
                format!("Invalid ciphertext: {}", msg),
            )),
            AutoUnsealError::ProviderError(msg) => {
                SecretonError::Core(CoreError::internal(format!("Provider error: {}", msg)))
            }
            AutoUnsealError::RetriesExhausted(attempts) => {
                SecretonError::Core(CoreError::internal(format!(
                    "Auto-unseal failed after {} retry attempts",
                    attempts
                )))
            }
            AutoUnsealError::FallbackToManual => SecretonError::Core(CoreError::internal(
                "Auto-unseal failed, falling back to manual unseal".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secreton_core::error::SecretonError;

    #[test]
    fn test_error_conversion() {
        let auto_unseal_error = AutoUnsealError::NotConfigured;
        let secreton_error: SecretonError = auto_unseal_error.into();

        match secreton_error {
            SecretonError::Core(_) => {
                // Expected
            }
            _ => panic!("Expected Core error"),
        }
    }

    #[test]
    fn test_error_messages() {
        let error = AutoUnsealError::EncryptionFailed("test error".to_string());
        assert_eq!(
            error.to_string(),
            "Auto-unseal encryption failed: test error"
        );

        let error = AutoUnsealError::KeyNotFound("key-123".to_string());
        assert_eq!(error.to_string(), "Auto-unseal key not found: key-123");
    }
}
