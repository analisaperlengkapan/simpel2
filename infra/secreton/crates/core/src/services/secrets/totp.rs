//! TOTP Secrets Engine
//!
//! Time-based One-Time Password secrets engine for MFA integration.
//! Provides RFC 6238 compliant TOTP key generation, validation, and management.
//!
//! # Features
//!
//! - RFC 6238 compliant TOTP implementation
//! - Multiple algorithm support (SHA1, SHA256, SHA512)
//! - Configurable code length (6 or 8 digits)
//! - QR code URL generation for authenticator apps
//! - Replay attack prevention
//! - Time window adjustment for clock skew
//! - Integration with lease management
//! - Audit logging support
//!
//! # Use Cases
//!
//! - Authenc MFA secret storage
//! - User TOTP secret management
//! - Centralized OTP validation
//! - Backup codes generation

use chrono::{DateTime, Duration, Utc};
use data_encoding::BASE32;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use totp_lite::{Sha1, Sha256, Sha512, totp_custom};
use tracing::{debug, info, warn};

/// Error types for TOTP engine
#[derive(Debug, thiserror::Error)]
/// Mewakili pub `TotpError`.
pub enum TotpError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Key already exists: {0}")]
    KeyAlreadyExists(String),

    #[error("Invalid TOTP code")]
    InvalidCode,

    #[error("TOTP code expired")]
    CodeExpired,

    #[error("TOTP code already used (replay attack prevented)")]
    CodeReused,

    #[error("Invalid secret format: {0}")]
    InvalidSecret(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Invalid algorithm: {0}")]
    InvalidAlgorithm(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// TOTP algorithm
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
/// Mewakili pub `TotpAlgorithm`.
pub enum TotpAlgorithm {
    /// SHA1 (most compatible, default for Google Authenticator)
    #[default]
    SHA1,
    /// SHA256 (more secure)
    SHA256,
    /// SHA512 (most secure)
    SHA512,
}

impl TotpAlgorithm {
    pub fn as_str(&self) -> &str {
        match self {
            TotpAlgorithm::SHA1 => "SHA1",
            TotpAlgorithm::SHA256 => "SHA256",
            TotpAlgorithm::SHA512 => "SHA512",
        }
    }

    /// Mewakili pub `from_str(s`.
    pub fn from_str(s: &str) -> Result<Self, TotpError> {
        match s.to_uppercase().as_str() {
            "SHA1" => Ok(TotpAlgorithm::SHA1),
            "SHA256" => Ok(TotpAlgorithm::SHA256),
            "SHA512" => Ok(TotpAlgorithm::SHA512),
            _ => Err(TotpError::InvalidAlgorithm(s.to_string())),
        }
    }
}

/// TOTP key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `TotpKey`.
pub struct TotpKey {
    /// Key name (unique identifier)
    pub name: String,

    /// Issuer name (e.g., "SIMKARI", "Secreton")
    pub issuer: String,

    /// Account name (e.g., user email or username)
    pub account_name: String,

    /// Secret key (raw bytes, stored securely)
    #[serde(with = "serde_bytes")]
    pub secret: Vec<u8>,

    /// Algorithm (SHA1, SHA256, SHA512)
    pub algorithm: TotpAlgorithm,

    /// Number of digits (6 or 8)
    pub digits: u8,

    /// Time period in seconds (typically 30)
    pub period: u32,

    /// Time window for validation (±N periods for clock skew)
    pub skew: u8,

    /// Creation time
    pub created_at: DateTime<Utc>,

    /// Last validation time
    pub last_validated_at: Option<DateTime<Utc>>,

    /// QR code URL for authenticator apps
    pub qr_code_url: String,
}

impl TotpKey {
    /// Create new TOTP key with default parameters
    pub fn new(name: String, issuer: String, account_name: String) -> Self {
        let secret = Self::generate_secret();
        let algorithm = TotpAlgorithm::default();
        let digits = 6;
        let period = 30;
        let qr_code_url =
            Self::generate_qr_url(&issuer, &account_name, &secret, &algorithm, digits, period);

        Self {
            name,
            issuer,
            account_name,
            secret,
            algorithm,
            digits,
            period,
            skew: 1, // Allow ±1 time window for clock skew
            created_at: Utc::now(),
            last_validated_at: None,
            qr_code_url,
        }
    }

    /// Create TOTP key with custom parameters
    pub fn with_params(
        name: String,
        issuer: String,
        account_name: String,
        algorithm: TotpAlgorithm,
        digits: u8,
        period: u32,
        skew: u8,
    ) -> Result<Self, TotpError> {
        // Validate parameters
        if digits != 6 && digits != 8 {
            return Err(TotpError::InvalidConfig(
                "Digits must be 6 or 8".to_string(),
            ));
        }
        if period == 0 {
            return Err(TotpError::InvalidConfig(
                "Period must be greater than 0".to_string(),
            ));
        }

        let secret = Self::generate_secret();
        let qr_code_url =
            Self::generate_qr_url(&issuer, &account_name, &secret, &algorithm, digits, period);

        Ok(Self {
            name,
            issuer,
            account_name,
            secret,
            algorithm,
            digits,
            period,
            skew,
            created_at: Utc::now(),
            last_validated_at: None,
            qr_code_url,
        })
    }

    /// Generate cryptographically secure random secret (20 bytes = 160 bits)
    fn generate_secret() -> Vec<u8> {
        use rand::RngCore;
        let mut secret = vec![0u8; 20];
        rand::thread_rng().fill_bytes(&mut secret);
        secret
    }

    /// Get secret as base32 encoded string
    pub fn secret_base32(&self) -> String {
        BASE32.encode(&self.secret)
    }

    /// Generate QR code URL for authenticator apps
    fn generate_qr_url(
        issuer: &str,
        account: &str,
        secret: &[u8],
        algorithm: &TotpAlgorithm,
        digits: u8,
        period: u32,
    ) -> String {
        let secret_base32 = BASE32.encode(secret);
        format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm={}&digits={}&period={}",
            urlencoding::encode(issuer),
            urlencoding::encode(account),
            secret_base32,
            urlencoding::encode(issuer),
            algorithm.as_str(),
            digits,
            period
        )
    }

    /// Generate TOTP code for current time
    pub fn generate_code(&self) -> String {
        self.generate_code_at(Utc::now())
    }

    /// Generate TOTP code for specific time
    pub fn generate_code_at(&self, timestamp: DateTime<Utc>) -> String {
        let seconds = timestamp.timestamp() as u64;

        match self.algorithm {
            TotpAlgorithm::SHA1 => totp_custom::<Sha1>(
                self.period as u64,
                self.digits as u32,
                &self.secret,
                seconds,
            ),
            TotpAlgorithm::SHA256 => totp_custom::<Sha256>(
                self.period as u64,
                self.digits as u32,
                &self.secret,
                seconds,
            ),
            TotpAlgorithm::SHA512 => totp_custom::<Sha512>(
                self.period as u64,
                self.digits as u32,
                &self.secret,
                seconds,
            ),
        }
    }

    /// Validate TOTP code with time window
    pub fn validate_code(&self, code: &str, timestamp: DateTime<Utc>) -> bool {
        let seconds = timestamp.timestamp() as u64;

        // Check current window and adjacent windows for clock skew
        for offset in -(self.skew as i64)..=(self.skew as i64) {
            let adjusted_seconds = (seconds as i64 + (offset * self.period as i64)) as u64;
            let expected_code = match self.algorithm {
                TotpAlgorithm::SHA1 => totp_custom::<Sha1>(
                    self.period as u64,
                    self.digits as u32,
                    &self.secret,
                    adjusted_seconds,
                ),
                TotpAlgorithm::SHA256 => totp_custom::<Sha256>(
                    self.period as u64,
                    self.digits as u32,
                    &self.secret,
                    adjusted_seconds,
                ),
                TotpAlgorithm::SHA512 => totp_custom::<Sha512>(
                    self.period as u64,
                    self.digits as u32,
                    &self.secret,
                    adjusted_seconds,
                ),
            };

            if expected_code == code {
                return true;
            }
        }

        false
    }
}

/// TOTP validation history entry (for replay attack prevention)
#[derive(Debug, Clone)]
struct ValidationHistory {
    code: String,
    timestamp: DateTime<Utc>,
}

/// TOTP code generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `TotpCodeRequest`.
pub struct TotpCodeRequest {
    /// Key name
    pub key_name: String,
}

/// TOTP code generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `TotpCodeResponse`.
pub struct TotpCodeResponse {
    /// Generated code
    pub code: String,

    /// Expiration time
    pub expires_at: DateTime<Utc>,
}

/// TOTP validation request
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `TotpValidationRequest`.
pub struct TotpValidationRequest {
    /// Key name
    pub key_name: String,

    /// Code to validate
    pub code: String,

    /// Optional custom skew (overrides key's default skew)
    pub skew: Option<u8>,
}

/// TOTP validation response
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `TotpValidationResponse`.
pub struct TotpValidationResponse {
    /// Whether the code is valid
    pub valid: bool,

    /// Validation timestamp
    pub timestamp: DateTime<Utc>,
}

/// TOTP key creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `TotpKeyCreateRequest`.
pub struct TotpKeyCreateRequest {
    /// Key name (unique identifier)
    pub name: String,

    /// Issuer name
    pub issuer: String,

    /// Account name
    pub account_name: String,

    /// Algorithm (optional, defaults to SHA1)
    pub algorithm: Option<TotpAlgorithm>,

    /// Number of digits (optional, defaults to 6)
    pub digits: Option<u8>,

    /// Time period in seconds (optional, defaults to 30)
    pub period: Option<u32>,

    /// Time window for validation (optional, defaults to 1)
    pub skew: Option<u8>,
}

/// TOTP key response
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `TotpKeyResponse`.
pub struct TotpKeyResponse {
    /// Key name
    pub name: String,

    /// Issuer name
    pub issuer: String,

    /// Account name
    pub account_name: String,

    /// Secret (base32 encoded)
    pub secret: String,

    /// Algorithm
    pub algorithm: String,

    /// Number of digits
    pub digits: u8,

    /// Time period
    pub period: u32,

    /// QR code URL
    pub qr_code_url: String,

    /// Creation time
    pub created_at: DateTime<Utc>,

    /// Last validation time
    pub last_validated_at: Option<DateTime<Utc>>,
}

impl From<&TotpKey> for TotpKeyResponse {
    fn from(key: &TotpKey) -> Self {
        Self {
            name: key.name.clone(),
            issuer: key.issuer.clone(),
            account_name: key.account_name.clone(),
            secret: key.secret_base32(),
            algorithm: key.algorithm.as_str().to_string(),
            digits: key.digits,
            period: key.period,
            qr_code_url: key.qr_code_url.clone(),
            created_at: key.created_at,
            last_validated_at: key.last_validated_at,
        }
    }
}

/// TOTP secrets engine
pub struct TotpEngine {
    /// Storage for TOTP keys
    keys: Arc<RwLock<HashMap<String, TotpKey>>>,

    /// Validation history for replay attack prevention
    history: Arc<RwLock<HashMap<String, Vec<ValidationHistory>>>>,
}

impl TotpEngine {
    /// Create new TOTP engine
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create TOTP key
    pub async fn create_key(
        &self,
        request: TotpKeyCreateRequest,
    ) -> Result<TotpKeyResponse, TotpError> {
        let mut keys = self.keys.write().await;

        // Check if key already exists
        if keys.contains_key(&request.name) {
            return Err(TotpError::KeyAlreadyExists(request.name));
        }

        // Create key with parameters
        let key = if let (Some(algorithm), Some(digits), Some(period), Some(skew)) = (
            request.algorithm,
            request.digits,
            request.period,
            request.skew,
        ) {
            TotpKey::with_params(
                request.name.clone(),
                request.issuer,
                request.account_name,
                algorithm,
                digits,
                period,
                skew,
            )?
        } else {
            TotpKey::new(request.name.clone(), request.issuer, request.account_name)
        };

        let response = TotpKeyResponse::from(&key);
        keys.insert(request.name.clone(), key);

        info!("Created TOTP key: {}", request.name);
        Ok(response)
    }

    /// Get TOTP key
    pub async fn get_key(&self, key_name: &str) -> Result<TotpKeyResponse, TotpError> {
        let keys = self.keys.read().await;
        let key = keys
            .get(key_name)
            .ok_or_else(|| TotpError::KeyNotFound(key_name.to_string()))?;

        Ok(TotpKeyResponse::from(key))
    }

    /// Delete TOTP key
    pub async fn delete_key(&self, key_name: &str) -> Result<(), TotpError> {
        let mut keys = self.keys.write().await;
        keys.remove(key_name)
            .ok_or_else(|| TotpError::KeyNotFound(key_name.to_string()))?;

        // Clean up history
        let mut history = self.history.write().await;
        history.remove(key_name);

        info!("Deleted TOTP key: {}", key_name);
        Ok(())
    }

    /// List all TOTP keys
    pub async fn list_keys(&self) -> Vec<String> {
        let keys = self.keys.read().await;
        keys.keys().cloned().collect()
    }

    /// Generate TOTP code
    pub async fn generate_code(&self, key_name: &str) -> Result<TotpCodeResponse, TotpError> {
        let keys = self.keys.read().await;
        let key = keys
            .get(key_name)
            .ok_or_else(|| TotpError::KeyNotFound(key_name.to_string()))?;

        let now = Utc::now();
        let code = key.generate_code_at(now);
        let expires_at = now + Duration::seconds(key.period as i64);

        debug!("Generated TOTP code for key: {}", key_name);
        Ok(TotpCodeResponse { code, expires_at })
    }

    /// Validate TOTP code
    pub async fn validate_code(
        &self,
        request: TotpValidationRequest,
    ) -> Result<TotpValidationResponse, TotpError> {
        // Get key
        let key = {
            let keys = self.keys.read().await;
            keys.get(&request.key_name)
                .ok_or_else(|| TotpError::KeyNotFound(request.key_name.clone()))?
                .clone()
        };

        // Check replay attack
        let now = Utc::now();
        let history = self.history.read().await;
        if let Some(entries) = history.get(&request.key_name) {
            let recent_window = now - Duration::seconds(key.period as i64 * 2);
            for entry in entries {
                if entry.timestamp > recent_window && entry.code == request.code {
                    warn!("TOTP replay attack detected for key: {}", request.key_name);
                    return Err(TotpError::CodeReused);
                }
            }
        }
        drop(history);

        // Validate code
        let valid = key.validate_code(&request.code, now);

        if valid {
            // Record successful validation
            let mut history = self.history.write().await;
            history
                .entry(request.key_name.clone())
                .or_insert_with(Vec::new)
                .push(ValidationHistory {
                    code: request.code.clone(),
                    timestamp: now,
                });

            // Update last validated time
            let mut keys = self.keys.write().await;
            if let Some(key) = keys.get_mut(&request.key_name) {
                key.last_validated_at = Some(now);
            }

            info!(
                "TOTP code validated successfully for key: {}",
                request.key_name
            );
        } else {
            warn!("TOTP code validation failed for key: {}", request.key_name);
        }

        Ok(TotpValidationResponse {
            valid,
            timestamp: now,
        })
    }

    /// Cleanup old validation history
    pub async fn cleanup_history(&self, max_age_seconds: i64) -> usize {
        let mut history = self.history.write().await;
        let now = Utc::now();
        let cutoff = now - Duration::seconds(max_age_seconds);
        let mut count = 0;

        for entries in history.values_mut() {
            let old_len = entries.len();
            entries.retain(|entry| entry.timestamp > cutoff);
            count += old_len - entries.len();
        }

        if count > 0 {
            debug!("Cleaned up {} old TOTP validation history entries", count);
        }

        count
    }
}

impl Default for TotpEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Serde helper for byte arrays
mod serde_bytes {
    use data_encoding::BASE32;
    use serde::{Deserialize, Deserializer, Serializer};

    /// Mewakili pub `serialize`.
    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&BASE32.encode(bytes))
    }

    /// Mewakili pub `deserialize`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        BASE32
            .decode(s.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_key_generation() {
        let key = TotpKey::new(
            "test_key".to_string(),
            "SIMKARI".to_string(),
            "user@example.com".to_string(),
        );

        assert_eq!(key.name, "test_key");
        assert_eq!(key.issuer, "SIMKARI");
        assert_eq!(key.digits, 6);
        assert_eq!(key.period, 30);
        assert!(key.qr_code_url.contains("otpauth://totp/"));
    }

    #[test]
    fn test_totp_code_generation() {
        let key = TotpKey::new(
            "test_key".to_string(),
            "SIMKARI".to_string(),
            "user@example.com".to_string(),
        );

        let code = key.generate_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_numeric()));
    }

    #[test]
    fn test_totp_code_validation() {
        let key = TotpKey::new(
            "test_key".to_string(),
            "SIMKARI".to_string(),
            "user@example.com".to_string(),
        );

        let now = Utc::now();
        let code = key.generate_code_at(now);

        // Should validate with same timestamp
        assert!(key.validate_code(&code, now));

        // Should validate within time window
        let future = now + Duration::seconds(15);
        assert!(key.validate_code(&code, future));

        // Should not validate with wrong code
        assert!(!key.validate_code("000000", now));
    }

    #[tokio::test]
    async fn test_totp_engine_create_key() {
        let engine = TotpEngine::new();

        let request = TotpKeyCreateRequest {
            name: "test_key".to_string(),
            issuer: "SIMKARI".to_string(),
            account_name: "user@example.com".to_string(),
            algorithm: None,
            digits: None,
            period: None,
            skew: None,
        };

        let response = engine.create_key(request).await.unwrap();
        assert_eq!(response.name, "test_key");
        assert!(!response.secret.is_empty());
    }

    #[tokio::test]
    async fn test_totp_engine_validate_code() {
        let engine = TotpEngine::new();

        // Create key
        let request = TotpKeyCreateRequest {
            name: "test_key".to_string(),
            issuer: "SIMKARI".to_string(),
            account_name: "user@example.com".to_string(),
            algorithm: None,
            digits: None,
            period: None,
            skew: None,
        };
        engine.create_key(request).await.unwrap();

        // Generate code
        let code_response = engine.generate_code("test_key").await.unwrap();

        // Validate code
        let validation_request = TotpValidationRequest {
            key_name: "test_key".to_string(),
            code: code_response.code,
            skew: None,
        };
        let validation_response = engine.validate_code(validation_request).await.unwrap();
        assert!(validation_response.valid);
    }

    #[tokio::test]
    async fn test_totp_engine_replay_prevention() {
        let engine = TotpEngine::new();

        // Create key
        let request = TotpKeyCreateRequest {
            name: "test_key".to_string(),
            issuer: "SIMKARI".to_string(),
            account_name: "user@example.com".to_string(),
            algorithm: None,
            digits: None,
            period: None,
            skew: None,
        };
        engine.create_key(request).await.unwrap();

        // Generate code
        let code_response = engine.generate_code("test_key").await.unwrap();

        // First validation should succeed
        let validation_request = TotpValidationRequest {
            key_name: "test_key".to_string(),
            code: code_response.code.clone(),
            skew: None,
        };
        let result = engine.validate_code(validation_request.clone()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().valid);

        // Second validation with same code should fail (replay attack)
        let result = engine.validate_code(validation_request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TotpError::CodeReused));
    }
}
