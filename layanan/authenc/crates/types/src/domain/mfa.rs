//! MFA domain types for Authenc identity provider.
//!
//! These types define the data structures used for MFA operations,
//! including setup, verification, and status tracking.

use serde::{Deserialize, Serialize};

/// MFA setup data returned when initializing TOTP for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetupData {
    /// QR code URL for scanning with authenticator apps
    pub qr_code_url: String,
    /// Secret key for manual entry in authenticator apps
    pub secret_key: String,
    /// Backup codes for emergency access
    pub backup_codes: Vec<String>,
}

/// MFA status response describing current MFA state for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaStatusResponse {
    /// Whether MFA is enabled for the user
    pub is_enabled: bool,
    /// MFA method type (e.g., "TOTP")
    pub method: Option<String>,
    /// Last time MFA was used (ISO 8601 format)
    pub last_used: Option<String>,
}

/// Errors specific to vault/secrets operations for MFA.
#[derive(Debug, Clone, thiserror::Error)]
pub enum VaultError {
    /// Secret not found
    #[error("Secret not found: {0}")]
    NotFound(String),

    /// Connection error to secrets backend
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Encryption/decryption error
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    /// Internal vault error
    #[error("Internal error: {0}")]
    Internal(String),
}
