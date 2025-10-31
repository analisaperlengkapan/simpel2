//! Cryptographic error types

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Cryptographic error types
#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CryptoError {
    // Key management errors
    #[error("Key already exists: {0}")]
    KeyAlreadyExists(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Key version not found: {0}")]
    KeyVersionNotFound(u32),

    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[error("Key rotation failed: {0}")]
    KeyRotationFailed(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    // Encryption/Decryption errors
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid ciphertext: {0}")]
    InvalidCiphertext(String),

    // Signing/Verification errors
    #[error("Signing failed: {0}")]
    SigningFailed(String),

    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    // Input validation errors
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("Invalid nonce/IV length")]
    InvalidNonceLength,

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Invalid algorithm: {0}")]
    InvalidAlgorithm(String),

    // Policy and permission errors
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    // System errors
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Random generation failed")]
    RandomGenerationFailed,

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Invalid usage: {0}")]
    InvalidUsage(String),

    #[error("Hash failed: {0}")]
    HashFailed(String),
}

/// Result type for cryptographic operations
pub type CryptoResult<T> = Result<T, CryptoError>;

impl From<std::io::Error> for CryptoError {
    fn from(err: std::io::Error) -> Self {
        CryptoError::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for CryptoError {
    fn from(err: serde_json::Error) -> Self {
        CryptoError::SerializationError(err.to_string())
    }
}

impl From<bincode::error::EncodeError> for CryptoError {
    fn from(err: bincode::error::EncodeError) -> Self {
        CryptoError::SerializationError(err.to_string())
    }
}

impl From<bincode::error::DecodeError> for CryptoError {
    fn from(err: bincode::error::DecodeError) -> Self {
        CryptoError::SerializationError(err.to_string())
    }
}
