//! HSM error types

use thiserror::Error;

/// Result type for HSM operations
pub type HsmResult<T> = Result<T, HsmError>;

/// HSM operation errors
#[derive(Debug, Error, Clone)]
pub enum HsmError {
    /// HSM not initialized
    #[error("HSM not initialized")]
    NotInitialized,

    /// HSM connection failed
    #[error("HSM connection failed: {0}")]
    ConnectionFailed(String),

    /// HSM authentication failed
    #[error("HSM authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Key not found in HSM
    #[error("Key not found in HSM: {0}")]
    KeyNotFound(String),

    /// Key already exists in HSM
    #[error("Key already exists in HSM: {0}")]
    KeyAlreadyExists(String),

    /// Invalid key algorithm
    #[error("Invalid key algorithm: {0}")]
    InvalidAlgorithm(String),

    /// Invalid key size
    #[error("Invalid key size: {0}")]
    InvalidKeySize(u32),

    /// HSM operation failed
    #[error("HSM operation failed: {0}")]
    OperationFailed(String),

    /// PKCS#11 error
    #[error("PKCS#11 error: {0}")]
    Pkcs11Error(String),

    /// Encryption failed
    #[error("HSM encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption failed
    #[error("HSM decryption failed: {0}")]
    DecryptionFailed(String),

    /// Signing failed
    #[error("HSM signing failed: {0}")]
    SigningFailed(String),

    /// Verification failed
    #[error("HSM verification failed: {0}")]
    VerificationFailed(String),

    /// Invalid data format
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),

    /// HSM unavailable
    #[error("HSM unavailable: {0}")]
    Unavailable(String),

    /// Configuration error
    #[error("HSM configuration error: {0}")]
    ConfigError(String),

    /// Generic error
    #[error("HSM error: {0}")]
    Other(String),
}

// NOTE: CoreError conversion removed - HSM is now a separate crate.
// If core needs to convert HsmError, implement From<HsmError> in core.
