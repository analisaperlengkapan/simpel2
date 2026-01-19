//! Secreton client abstraction for secret management in Authenc
//! Internal secret management module - uses Secreton service only

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Represents a secret value fetched from a vault
#[derive(Debug, Clone)]
pub struct Secret {
    /// The actual secret value
    pub value: String,
    /// Optional metadata associated with the secret
    pub metadata: Option<HashMap<String, String>>,
    /// Version of the secret (for rotation tracking)
    pub version: Option<u32>,
    /// Creation timestamp
    pub created_at: Option<DateTime<Utc>>,
    /// Expiration timestamp (for automatic rotation)
    pub expires_at: Option<DateTime<Utc>>,
}

/// Result of secret rotation operation
#[derive(Debug, Clone)]
pub struct RotationResult {
    /// New secret value
    pub new_secret: Secret,
    /// Old secret value (for graceful transition)
    pub old_secret: Option<Secret>,
    /// Rotation timestamp
    pub rotated_at: DateTime<Utc>,
}

/// HSM key metadata
#[derive(Debug, Clone)]
pub struct HsmKeyMetadata {
    /// Key ID in HSM
    pub key_id: String,
    /// Key algorithm (RSA, ECDSA, AES, etc.)
    pub algorithm: String,
    /// Key size in bits
    pub key_size: u32,
    /// Whether key is exportable
    pub exportable: bool,
    /// Key usage (sign, encrypt, wrap, etc.)
    pub usage: Vec<String>,
}

/// SecretonClient trait for secret backends
#[async_trait]
pub trait SecretonClientTrait: Send + Sync {
    /// Fetch a secret by key (optionally scoped by realm)
    async fn get_secret(&self, key: &str, realm: Option<&str>) -> Option<Secret>;

    /// Store or update a secret
    async fn put_secret(
        &self,
        key: &str,
        value: &str,
        realm: Option<&str>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), SecretonError>;

    /// Delete a secret
    async fn delete_secret(&self, key: &str, realm: Option<&str>) -> Result<(), SecretonError>;

    /// List all secret keys (with optional realm filter)
    async fn list_secrets(&self, realm: Option<&str>) -> Result<Vec<String>, SecretonError>;

    /// Rotate a secret (generate new value, keep old for transition)
    async fn rotate_secret(
        &self,
        key: &str,
        realm: Option<&str>,
        generator: Box<dyn Fn() -> String + Send>,
    ) -> Result<RotationResult, SecretonError>;

    /// Get secret version history
    async fn get_secret_versions(
        &self,
        key: &str,
        realm: Option<&str>,
    ) -> Result<Vec<Secret>, SecretonError>;

    /// Check if secreton is healthy and accessible
    async fn health_check(&self) -> Result<bool, SecretonError>;
}

/// Extended secreton trait for HSM integration
#[async_trait]
pub trait HsmSecretonClient: SecretonClientTrait {
    /// Generate a key in HSM
    async fn generate_hsm_key(
        &self,
        key_id: &str,
        algorithm: &str,
        key_size: u32,
        usage: Vec<String>,
    ) -> Result<HsmKeyMetadata, SecretonError>;

    /// Sign data using HSM key
    async fn hsm_sign(
        &self,
        key_id: &str,
        data: &[u8],
        algorithm: &str,
    ) -> Result<Vec<u8>, SecretonError>;

    /// Encrypt data using HSM key
    async fn hsm_encrypt(&self, key_id: &str, plaintext: &[u8]) -> Result<Vec<u8>, SecretonError>;

    /// Decrypt data using HSM key
    async fn hsm_decrypt(&self, key_id: &str, ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError>;

    /// List all HSM keys
    async fn list_hsm_keys(&self) -> Result<Vec<HsmKeyMetadata>, SecretonError>;

    /// Delete HSM key
    async fn delete_hsm_key(&self, key_id: &str) -> Result<(), SecretonError>;
}

/// Secreton error types
#[derive(Debug, Clone)]
pub enum SecretonError {
    /// Secret not found
    NotFound(String),
    /// Secreton backend unreachable
    Unavailable(String),
    /// Authentication failed
    AuthenticationFailed(String),
    /// Authorization failed
    Unauthorized(String),
    /// Invalid secret format
    InvalidFormat(String),
    /// HSM operation failed
    HsmError(String),
    /// Generic error
    Other(String),
}

impl std::fmt::Display for SecretonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretonError::NotFound(msg) => write!(f, "Secret not found: {}", msg),
            SecretonError::Unavailable(msg) => write!(f, "Secreton unavailable: {}", msg),
            SecretonError::AuthenticationFailed(msg) => {
                write!(f, "Secreton authentication failed: {}", msg)
            }
            SecretonError::Unauthorized(msg) => write!(f, "Secreton unauthorized: {}", msg),
            SecretonError::InvalidFormat(msg) => write!(f, "Invalid secret format: {}", msg),
            SecretonError::HsmError(msg) => write!(f, "HSM error: {}", msg),
            SecretonError::Other(msg) => write!(f, "Secreton error: {}", msg),
        }
    }
}

impl std::error::Error for SecretonError {}

// Type aliases for backward compatibility during migration
/// Alias tipe `Vault`.
pub type Vault = dyn SecretonClientTrait;
/// Alias tipe `VaultError`.
pub type VaultError = SecretonError;

/// Modul `grpc_client`.
// Secreton client (custom Rust-based secret manager) - primary integration
/// Modul `grpc_client`.
pub mod grpc_client;
pub mod secreton_client;

pub use grpc_client::GrpcSecretonClient;

// ...existing code for provider modules will be implemented separately...
