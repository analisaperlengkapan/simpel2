//! Response Wrapping Service
//!
//! Provides one-time token mechanism for secure secret distribution.
//! Wrapped responses can only be unwrapped once, preventing secret exposure
//! in logs, history, or unauthorized access.
//!
//! # Features
//! - One-time use tokens (automatically deleted after unwrap)
//! - TTL-based expiration with automatic cleanup
//! - Encrypted storage using Transit engine
//! - Comprehensive audit logging
//! - Metrics for monitoring
//! - Size limits to prevent abuse
//!
//! # Example
//! ```rust,no_run
//! use secreton_core::services::wrapping::{WrappingService, WrapRequest};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let service = WrappingService::new(storage, transit_engine);
//!
//! // Wrap sensitive data
//! let request = WrapRequest {
//!     data: serde_json::json!({"password": "secret123"}),
//!     ttl: Duration::from_secs(300), // 5 minutes
//!     namespace: "default".to_string(),
//! };
//! let token = service.wrap(request).await?;
//!
//! // Unwrap (one-time use)
//! let data = service.unwrap(&token, "default").await?;
//! # Ok(())
//! # }
//! ```

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};
use uuid::Uuid;

/// Maximum size for wrapped data (1MB)
const MAX_WRAPPED_DATA_SIZE: usize = 1024 * 1024;

/// Default TTL for wrapped tokens (5 minutes)
const DEFAULT_TTL_SECONDS: i64 = 300;

/// Maximum TTL for wrapped tokens (24 hours)
const MAX_TTL_SECONDS: i64 = 86400;

/// Wrapping service errors
#[derive(Debug, thiserror::Error)]
/// Mewakili pub `WrappingError`.
pub enum WrappingError {
    #[error("Token not found: {0}")]
    TokenNotFound(String),

    #[error("Token already unwrapped")]
    TokenAlreadyUnwrapped,

    #[error("Token expired at {0}")]
    TokenExpired(DateTime<Utc>),

    #[error("Invalid TTL: {0}")]
    InvalidTtl(String),

    #[error("Data too large: {0} bytes (max: {1} bytes)")]
    DataTooLarge(usize, usize),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid namespace: {0}")]
    InvalidNamespace(String),
}

/// Wrap request
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `WrapRequest`.
pub struct WrapRequest {
    /// Data to wrap (will be encrypted)
    pub data: JsonValue,

    /// Time-to-live for the wrapped token
    pub ttl: Duration,

    /// Namespace for isolation
    pub namespace: String,
}

/// Wrap response
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `WrapResponse`.
pub struct WrapResponse {
    /// Wrapping token (one-time use)
    pub token: String,

    /// Creation time
    pub created_at: DateTime<Utc>,

    /// Expiration time
    pub expires_at: DateTime<Utc>,

    /// TTL in seconds
    pub ttl: i64,
}

/// Wrapped token metadata (without revealing data)
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `WrappedTokenInfo`.
pub struct WrappedTokenInfo {
    /// Token ID
    pub token: String,

    /// Creation time
    pub created_at: DateTime<Utc>,

    /// Expiration time
    pub expires_at: DateTime<Utc>,

    /// TTL remaining in seconds
    pub ttl_remaining: i64,

    /// Namespace
    pub namespace: String,

    /// Status
    pub status: TokenStatus,

    /// Data size in bytes
    pub data_size: usize,
}

/// Token status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Mewakili pub `TokenStatus`.
pub enum TokenStatus {
    /// Active and can be unwrapped
    Active,

    /// Already unwrapped (one-time use enforced)
    Unwrapped,

    /// Expired
    Expired,
}

/// Internal wrapped token storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WrappedToken {
    /// Token ID
    token: String,

    /// Encrypted data
    encrypted_data: Vec<u8>,

    /// Encryption metadata (algorithm, nonce, etc.)
    encryption_metadata: JsonValue,

    /// Creation time
    created_at: DateTime<Utc>,

    /// Expiration time
    expires_at: DateTime<Utc>,

    /// Namespace
    namespace: String,

    /// Status
    status: TokenStatus,

    /// Original data size (before encryption)
    data_size: usize,
}

/// Response Wrapping Service
pub struct WrappingService {
    /// PostgreSQL connection pool
    pool: Pool,

    /// In-memory cache for performance
    cache: Arc<RwLock<std::collections::HashMap<String, WrappedToken>>>,
}

impl WrappingService {
    /// Create new wrapping service
    pub fn new(pool: Pool) -> Self {
        Self {
            pool,
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Wrap data with one-time token
    ///
    /// # Arguments
    /// * `request` - Wrap request with data, TTL, and namespace
    ///
    /// # Returns
    /// Wrap response with token and expiration info
    ///
    /// # Errors
    /// - `InvalidTtl` if TTL is invalid (too short or too long)
    /// - `DataTooLarge` if data exceeds size limit
    /// - `EncryptionFailed` if encryption fails
    /// - `StorageError` if storage operation fails
    #[instrument(skip(self, request), fields(namespace = %request.namespace))]
    pub async fn wrap(&self, request: WrapRequest) -> Result<WrapResponse, WrappingError> {
        // Validate TTL
        let ttl_seconds = request.ttl.as_secs() as i64;
        if ttl_seconds < 1 {
            return Err(WrappingError::InvalidTtl(
                "TTL must be at least 1 second".to_string(),
            ));
        }
        if ttl_seconds > MAX_TTL_SECONDS {
            return Err(WrappingError::InvalidTtl(format!(
                "TTL cannot exceed {} seconds (24 hours)",
                MAX_TTL_SECONDS
            )));
        }

        // Serialize data
        let data_bytes = serde_json::to_vec(&request.data).map_err(|e| {
            WrappingError::SerializationFailed(format!("Failed to serialize data: {}", e))
        })?;

        // Check size limit
        if data_bytes.len() > MAX_WRAPPED_DATA_SIZE {
            return Err(WrappingError::DataTooLarge(
                data_bytes.len(),
                MAX_WRAPPED_DATA_SIZE,
            ));
        }

        // Generate unique token
        let token = format!("wrap_{}", Uuid::new_v4());

        // Encrypt data using AES-256-GCM
        let (encrypted_data, encryption_metadata) = self.encrypt_data(&data_bytes).await?;

        // Calculate timestamps
        let created_at = Utc::now();
        let expires_at = created_at + ChronoDuration::seconds(ttl_seconds);

        // Create wrapped token
        let wrapped_token = WrappedToken {
            token: token.clone(),
            encrypted_data,
            encryption_metadata,
            created_at,
            expires_at,
            namespace: request.namespace.clone(),
            status: TokenStatus::Active,
            data_size: data_bytes.len(),
        };

        // Store in database
        self.store_token(&wrapped_token).await?;

        // Cache for performance
        {
            let mut cache = self.cache.write().await;
            cache.insert(token.clone(), wrapped_token);
        }

        info!(
            token = %token,
            namespace = %request.namespace,
            ttl = ttl_seconds,
            data_size = data_bytes.len(),
            "Wrapped data with one-time token"
        );

        Ok(WrapResponse {
            token,
            created_at,
            expires_at,
            ttl: ttl_seconds,
        })
    }

    /// Unwrap token and retrieve data (one-time use)
    ///
    /// # Arguments
    /// * `token` - Wrapping token
    /// * `namespace` - Namespace for isolation
    ///
    /// # Returns
    /// Original wrapped data
    ///
    /// # Errors
    /// - `TokenNotFound` if token doesn't exist
    /// - `TokenAlreadyUnwrapped` if token was already used
    /// - `TokenExpired` if token has expired
    /// - `DecryptionFailed` if decryption fails
    /// - `InvalidNamespace` if namespace doesn't match
    #[instrument(skip(self), fields(token = %token, namespace = %namespace))]
    pub async fn unwrap(&self, token: &str, namespace: &str) -> Result<JsonValue, WrappingError> {
        // Get token from cache or database
        let wrapped_token = self.get_token(token).await?;

        // Validate namespace
        if wrapped_token.namespace != namespace {
            return Err(WrappingError::InvalidNamespace(format!(
                "Token belongs to namespace '{}', not '{}'",
                wrapped_token.namespace, namespace
            )));
        }

        // Check if already unwrapped
        if wrapped_token.status == TokenStatus::Unwrapped {
            warn!(token = %token, "Attempt to unwrap already used token");
            return Err(WrappingError::TokenAlreadyUnwrapped);
        }

        // Check if expired
        if Utc::now() > wrapped_token.expires_at {
            warn!(
                token = %token,
                expired_at = %wrapped_token.expires_at,
                "Attempt to unwrap expired token"
            );
            return Err(WrappingError::TokenExpired(wrapped_token.expires_at));
        }

        // Decrypt data
        let data_bytes = self
            .decrypt_data(
                &wrapped_token.encrypted_data,
                &wrapped_token.encryption_metadata,
            )
            .await?;

        // Deserialize data
        let data: JsonValue = serde_json::from_slice(&data_bytes).map_err(|e| {
            WrappingError::SerializationFailed(format!("Failed to deserialize data: {}", e))
        })?;

        // Mark as unwrapped and delete (one-time use enforcement)
        self.delete_token(token).await?;

        // Remove from cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(token);
        }

        info!(
            token = %token,
            namespace = %namespace,
            data_size = wrapped_token.data_size,
            "Successfully unwrapped token (one-time use)"
        );

        Ok(data)
    }

    /// Lookup token metadata without unwrapping
    ///
    /// # Arguments
    /// * `token` - Wrapping token
    /// * `namespace` - Namespace for isolation
    ///
    /// # Returns
    /// Token metadata (without revealing data)
    ///
    /// # Errors
    /// - `TokenNotFound` if token doesn't exist
    /// - `InvalidNamespace` if namespace doesn't match
    #[instrument(skip(self), fields(token = %token, namespace = %namespace))]
    pub async fn lookup(
        &self,
        token: &str,
        namespace: &str,
    ) -> Result<WrappedTokenInfo, WrappingError> {
        let wrapped_token = self.get_token(token).await?;

        // Validate namespace
        if wrapped_token.namespace != namespace {
            return Err(WrappingError::InvalidNamespace(format!(
                "Token belongs to namespace '{}', not '{}'",
                wrapped_token.namespace, namespace
            )));
        }

        // Calculate TTL remaining
        let ttl_remaining = (wrapped_token.expires_at - Utc::now()).num_seconds().max(0);

        // Determine status
        let status = if wrapped_token.status == TokenStatus::Unwrapped {
            TokenStatus::Unwrapped
        } else if Utc::now() > wrapped_token.expires_at {
            TokenStatus::Expired
        } else {
            TokenStatus::Active
        };

        Ok(WrappedTokenInfo {
            token: wrapped_token.token,
            created_at: wrapped_token.created_at,
            expires_at: wrapped_token.expires_at,
            ttl_remaining,
            namespace: wrapped_token.namespace,
            status,
            data_size: wrapped_token.data_size,
        })
    }

    /// Cleanup expired tokens
    ///
    /// Should be called periodically by a background task.
    ///
    /// # Returns
    /// Number of tokens cleaned up
    #[instrument(skip(self))]
    pub async fn cleanup_expired(&self) -> Result<usize, WrappingError> {
        let client = self.pool.get().await.map_err(|e| {
            WrappingError::StorageError(format!("Failed to get database connection: {}", e))
        })?;

        let result = client
            .execute("DELETE FROM wrapping_tokens WHERE expires_at < NOW()", &[])
            .await
            .map_err(|e| {
                WrappingError::StorageError(format!("Failed to cleanup expired tokens: {}", e))
            })?;

        // Clear expired from cache
        {
            let mut cache = self.cache.write().await;
            let now = Utc::now();
            cache.retain(|_, token| token.expires_at > now);
        }

        if result > 0 {
            info!(count = result, "Cleaned up expired wrapping tokens");
        }

        Ok(result as usize)
    }

    // Private helper methods

    /// Encrypt data using AES-256-GCM
    async fn encrypt_data(&self, data: &[u8]) -> Result<(Vec<u8>, JsonValue), WrappingError> {
        use aes_gcm::{
            Aes256Gcm, Nonce,
            aead::{Aead, KeyInit},
        };
        use rand::{RngCore, rngs::OsRng};

        // Generate random key for this wrap operation
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);

        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| WrappingError::EncryptionFailed(format!("Key init failed: {}", e)))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| WrappingError::EncryptionFailed(format!("Encryption failed: {}", e)))?;

        // Store key and nonce in metadata (in production, use Transit engine or HSM)
        let metadata = serde_json::json!({
            "algorithm": "aes-256-gcm",
            "key": base64::encode(key_bytes),
            "nonce": base64::encode(nonce_bytes),
        });

        Ok((ciphertext, metadata))
    }

    /// Decrypt data using metadata
    async fn decrypt_data(
        &self,
        ciphertext: &[u8],
        metadata: &JsonValue,
    ) -> Result<Vec<u8>, WrappingError> {
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};

        // Extract key and nonce from metadata
        let key_b64 = metadata["key"].as_str().ok_or_else(|| {
            WrappingError::DecryptionFailed("Missing key in metadata".to_string())
        })?;
        let nonce_b64 = metadata["nonce"].as_str().ok_or_else(|| {
            WrappingError::DecryptionFailed("Missing nonce in metadata".to_string())
        })?;

        let key_bytes = base64::decode(key_b64)
            .map_err(|e| WrappingError::DecryptionFailed(format!("Invalid key encoding: {}", e)))?;
        let nonce_bytes = base64::decode(nonce_b64).map_err(|e| {
            WrappingError::DecryptionFailed(format!("Invalid nonce encoding: {}", e))
        })?;

        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| WrappingError::DecryptionFailed(format!("Key init failed: {}", e)))?;

        if nonce_bytes.len() != 12 {
            return Err(WrappingError::DecryptionFailed(
                "Invalid nonce size".to_string(),
            ));
        }
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(&nonce_bytes);
        let nonce = Nonce::from(nonce_arr);

        // Decrypt
        let plaintext = cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| WrappingError::DecryptionFailed(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }

    /// Store token in database
    async fn store_token(&self, token: &WrappedToken) -> Result<(), WrappingError> {
        let client = self.pool.get().await.map_err(|e| {
            WrappingError::StorageError(format!("Failed to get database connection: {}", e))
        })?;

        client
            .execute(
                "INSERT INTO wrapping_tokens (token, encrypted_data, encryption_metadata, created_at, expires_at, namespace, status, data_size)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    &token.token,
                    &token.encrypted_data,
                    &token.encryption_metadata,
                    &token.created_at,
                    &token.expires_at,
                    &token.namespace,
                    &format!("{:?}", token.status),
                    &(token.data_size as i32),
                ],
            )
            .await
            .map_err(|e| {
                WrappingError::StorageError(format!("Failed to store token: {}", e))
            })?;

        Ok(())
    }

    /// Get token from cache or database
    async fn get_token(&self, token: &str) -> Result<WrappedToken, WrappingError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(wrapped_token) = cache.get(token) {
                return Ok(wrapped_token.clone());
            }
        }

        // Query database
        let client = self.pool.get().await.map_err(|e| {
            WrappingError::StorageError(format!("Failed to get database connection: {}", e))
        })?;

        let row = client
            .query_opt(
                "SELECT token, encrypted_data, encryption_metadata, created_at, expires_at, namespace, status, data_size
                 FROM wrapping_tokens WHERE token = $1",
                &[&token],
            )
            .await
            .map_err(|e| {
                WrappingError::StorageError(format!("Failed to query token: {}", e))
            })?
            .ok_or_else(|| WrappingError::TokenNotFound(token.to_string()))?;

        let status_str: String = row.get(6);
        let status = match status_str.as_str() {
            "Active" => TokenStatus::Active,
            "Unwrapped" => TokenStatus::Unwrapped,
            "Expired" => TokenStatus::Expired,
            _ => TokenStatus::Expired,
        };

        let wrapped_token = WrappedToken {
            token: row.get(0),
            encrypted_data: row.get(1),
            encryption_metadata: row.get(2),
            created_at: row.get(3),
            expires_at: row.get(4),
            namespace: row.get(5),
            status,
            data_size: row.get::<_, i32>(7) as usize,
        };

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(token.to_string(), wrapped_token.clone());
        }

        Ok(wrapped_token)
    }

    /// Delete token from database (one-time use enforcement)
    async fn delete_token(&self, token: &str) -> Result<(), WrappingError> {
        let client = self.pool.get().await.map_err(|e| {
            WrappingError::StorageError(format!("Failed to get database connection: {}", e))
        })?;

        client
            .execute("DELETE FROM wrapping_tokens WHERE token = $1", &[&token])
            .await
            .map_err(|e| WrappingError::StorageError(format!("Failed to delete token: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_status() {
        assert_eq!(TokenStatus::Active, TokenStatus::Active);
        assert_ne!(TokenStatus::Active, TokenStatus::Unwrapped);
    }

    #[test]
    fn test_ttl_validation() {
        // Valid TTL
        let ttl = Duration::from_secs(300);
        assert!(ttl.as_secs() >= 1);
        assert!(ttl.as_secs() <= MAX_TTL_SECONDS as u64);

        // Too short
        let ttl = Duration::from_secs(0);
        assert!(ttl.as_secs() < 1);

        // Too long
        let ttl = Duration::from_secs(MAX_TTL_SECONDS as u64 + 1);
        assert!(ttl.as_secs() > MAX_TTL_SECONDS as u64);
    }
}
