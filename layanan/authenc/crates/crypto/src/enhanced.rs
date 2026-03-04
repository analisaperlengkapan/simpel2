//! Enhanced Cryptographic Engine for SIMKARI Operations
//!
//! This module provides enhanced cryptographic operations specifically designed
//! for the SIMKARI super app and Indonesian government operations, including:
//! - JWT signing for pegawai (employee) authentication
//! - Batch token validation capabilities
//! - Session data encryption for SIMKARI users
//! - Audit signature generation for compliance
//! - Performance monitoring for cryptographic operations

use crate::aes_gcm::{AesGcmService, EncryptedData};
use authenc_types::{AuthencError, Result};
use base64ct::{Base64UrlUnpadded, Encoding};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

// Global Ed25519 keypair (lazy static)
use once_cell::sync::Lazy;

static ED25519_KEYPAIR: Lazy<SigningKey> = Lazy::new(|| {
    // In production, load from Secreton
    // For now, generate a key (this should be loaded from secure storage)
    let mut csprng = rand::rngs::OsRng;
    SigningKey::generate(&mut csprng)
});

/// Standard JWT claims for user authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Username
    pub username: String,
    /// Email
    pub email: String,
    /// Realm ID
    pub realm_id: String,
    /// User roles
    pub roles: Vec<String>,
    /// Expiration time (Unix timestamp)
    pub exp: usize,
    /// Issued at (Unix timestamp)
    pub iat: usize,
    /// Issuer
    pub iss: String,
}

/// Enhanced cryptographic engine for SIMKARI operations
pub struct EnhancedCryptoEngine {
    /// AES-GCM service for session data encryption
    aes_service: AesGcmService,
    /// Performance metrics cache
    metrics: Arc<RwLock<CryptoMetrics>>,
    /// Token validation cache for batch operations
    validation_cache: Arc<RwLock<HashMap<String, CachedValidation>>>,
    /// Post-quantum crypto mode
    pq_mode: PostQuantumMode,
}

/// Post-quantum cryptography mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostQuantumMode {
    /// Classical cryptography only (Ed25519, AES-256-GCM)
    Classical,
    /// Hybrid mode (Classical + Post-Quantum)
    Hybrid,
    /// Post-quantum only (ML-DSA, ML-KEM)
    PostQuantum,
}

/// Performance metrics for cryptographic operations
#[derive(Debug, Default, Clone)]
pub struct CryptoMetrics {
    /// JWT signing operations count and timing
    pub jwt_signing: OperationMetrics,
    /// JWT verification operations count and timing
    pub jwt_verification: OperationMetrics,
    /// Batch validation operations count and timing
    pub batch_validation: OperationMetrics,
    /// Session encryption operations count and timing
    pub session_encryption: OperationMetrics,
    /// Audit signature operations count and timing
    pub audit_signature: OperationMetrics,
    /// Post-quantum operations count and timing
    pub post_quantum: OperationMetrics,
}

/// Metrics for a specific operation type
#[derive(Debug, Default, Clone)]
pub struct OperationMetrics {
    /// Total number of operations
    pub count: u64,
    /// Total time spent in operations (milliseconds)
    pub total_time_ms: u64,
    /// Average time per operation (milliseconds)
    pub avg_time_ms: f64,
    /// Minimum operation time (milliseconds)
    pub min_time_ms: u64,
    /// Maximum operation time (milliseconds)
    pub max_time_ms: u64,
    /// Number of failed operations
    pub failures: u64,
}

/// Cached validation result
#[derive(Debug, Clone)]
struct CachedValidation {
    /// Whether the token is valid
    valid: bool,
    /// Expiration time of the cache entry
    expires_at: DateTime<Utc>,
    /// User claims from the token
    claims: Option<UserClaims>,
}

/// Enhanced JWT claims for SIMKARI pegawai
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PegawaiClaims {
    /// Standard JWT claims
    #[serde(flatten)]
    pub standard: UserClaims,
    /// NIP (Nomor Induk Pegawai)
    pub nip: String,
    /// Satuan Kerja (Work Unit) code
    pub satker_code: String,
    /// Jabatan (Position)
    pub jabatan: String,
    /// Eselon level
    pub eselon: Option<String>,
    /// Wilayah (Region) code for kejaksaan tinggi
    pub wilayah_code: Option<String>,
    /// Admin level code for hierarchical operations (e.g. "pusat", "satker")
    pub admin_level: Option<String>,
    /// Secreton access permissions
    pub secreton_permissions: SecretonPermissions,
}

/// Secreton access permissions for role-based access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretonPermissions {
    /// Secrets that can be read
    pub read_secrets: Vec<String>,
    /// Secrets that can be written
    pub write_secrets: Vec<String>,
    /// Whether admin operations are allowed
    pub admin_operations: bool,
    /// Whether audit access is granted
    pub audit_access: bool,
    /// Satker-specific permissions
    pub satker_permissions: HashMap<String, Vec<String>>,
}

/// Encrypted session data for SIMKARI users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSessionData {
    /// Encrypted session payload
    pub encrypted_data: EncryptedData,
    /// Session metadata
    pub metadata: SessionMetadata,
    /// Encryption algorithm used
    pub algorithm: String,
    /// Key ID used for encryption
    pub key_id: String,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Session ID
    pub session_id: String,
    /// User ID
    pub user_id: Uuid,
    /// Client IP address
    pub client_ip: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Session creation time
    pub created_at: DateTime<Utc>,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
    /// Session expiration time
    pub expires_at: DateTime<Utc>,
}

/// Audit signature for compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSignature {
    /// Digital signature
    pub signature: String,
    /// Signature algorithm
    pub algorithm: String,
    /// Signer information
    pub signer: SignerInfo,
    /// Timestamp of signing
    pub signed_at: DateTime<Utc>,
    /// Post-quantum signature (if hybrid mode)
    pub pq_signature: Option<String>,
}

/// Signer information for audit trails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerInfo {
    /// Signer user ID
    pub user_id: Uuid,
    /// Signer NIP
    pub nip: String,
    /// Signer name
    pub name: String,
    /// Signer role
    pub role: String,
    /// Satker code
    pub satker_code: String,
    /// Admin level of the signer
    pub admin_level: Option<String>,
}

/// Batch validation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchValidationRequest {
    /// Tokens to validate
    pub tokens: Vec<String>,
    /// Whether to use cache
    pub use_cache: bool,
    /// Maximum age of cached results (seconds)
    pub max_cache_age: Option<u64>,
}

/// Batch validation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchValidationResponse {
    /// Validation results
    pub results: Vec<TokenValidationResult>,
    /// Total processing time (milliseconds)
    pub processing_time_ms: u64,
}

/// Token validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidationResult {
    /// Token index in the batch
    pub index: usize,
    /// Whether the token is valid
    pub valid: bool,
    /// User claims if valid
    pub claims: Option<UserClaims>,
    /// Error message if invalid
    pub error: Option<String>,
}

/// Batch operation performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchMetrics {
    /// Total tokens processed
    pub total_tokens: usize,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
    /// Total processing time (milliseconds)
    pub total_time_ms: u64,
    /// Average time per token (milliseconds)
    pub avg_time_per_token_ms: f64,
}

impl Default for EnhancedCryptoEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedCryptoEngine {
    /// Create a new enhanced crypto engine
    pub fn new() -> Self {
        Self {
            aes_service: AesGcmService::new(),
            metrics: Arc::new(RwLock::new(CryptoMetrics::default())),
            validation_cache: Arc::new(RwLock::new(HashMap::new())),
            pq_mode: PostQuantumMode::Classical,
        }
    }

    /// Create enhanced crypto engine with specific post-quantum mode
    pub fn with_pq_mode(pq_mode: PostQuantumMode) -> Self {
        Self {
            aes_service: AesGcmService::new(),
            metrics: Arc::new(RwLock::new(CryptoMetrics::default())),
            validation_cache: Arc::new(RwLock::new(HashMap::new())),
            pq_mode,
        }
    }

    /// Sign JWT for pegawai authentication with enhanced claims
    pub async fn sign_jwt_for_pegawai(&self, claims: &PegawaiClaims) -> Result<String> {
        let start = Instant::now();

        // Create JWT header
        let header = serde_json::json!({
            "alg": "EdDSA",
            "typ": "JWT",
            "kid": "authenc-ed25519-key"
        });

        // Serialize header and claims
        let header_json = serde_json::to_string(&header)
            .map_err(|e| AuthencError::validation(format!("Failed to serialize header: {}", e)))?;

        let claims_json = serde_json::to_value(claims)
            .and_then(|v| serde_json::to_string(&v))
            .map_err(|e| {
                AuthencError::validation(format!("Failed to serialize pegawai claims: {}", e))
            })?;

        // Base64URL encode header and payload
        let header_b64 = Base64UrlUnpadded::encode_string(header_json.as_bytes());
        let payload_b64 = Base64UrlUnpadded::encode_string(claims_json.as_bytes());

        // Create signing input
        let signing_input = format!("{}.{}", header_b64, payload_b64);

        // Sign with Ed25519
        let signature: Signature = ED25519_KEYPAIR.sign(signing_input.as_bytes());
        let signature_b64 = Base64UrlUnpadded::encode_string(signature.to_bytes().as_ref());

        let result = Ok(format!("{}.{}", signing_input, signature_b64));

        // Update metrics
        let duration = start.elapsed();
        self.update_metrics("jwt_signing", duration, result.is_ok())
            .await;

        // Log for audit trail
        info!(
            nip = claims.nip,
            satker_code = claims.satker_code,
            duration_ms = duration.as_millis(),
            "JWT signed for pegawai"
        );

        result
    }

    /// Batch token validation for improved performance
    pub async fn validate_tokens_batch(
        &self,
        request: BatchValidationRequest,
    ) -> Result<BatchValidationResponse> {
        let start = Instant::now();
        let mut results = Vec::with_capacity(request.tokens.len());

        // Process tokens
        for (index, token) in request.tokens.iter().enumerate() {
            let validation_result = if request.use_cache {
                // Check cache first
                if let Some(cached) = self.get_cached_validation(token).await {
                    TokenValidationResult {
                        index,
                        valid: cached.valid,
                        error: None,
                        claims: cached.claims,
                    }
                } else {
                    let result = self.validate_single_token(token, index).await?;
                    self.cache_validation_result(token, &result).await;
                    result
                }
            } else {
                self.validate_single_token(token, index).await?
            };

            results.push(validation_result);
        }

        let duration = start.elapsed();
        self.update_metrics("batch_validation", duration, true)
            .await;

        Ok(BatchValidationResponse {
            results,
            processing_time_ms: duration.as_millis() as u64,
        })
    }

    /// Encrypt session data for SIMKARI users
    pub async fn encrypt_session_data(
        &self,
        session_data: &serde_json::Value,
        metadata: SessionMetadata,
    ) -> Result<EncryptedSessionData> {
        let start = Instant::now();

        // Serialize session data
        let data_bytes = serde_json::to_vec(session_data).map_err(|e| {
            AuthencError::validation(format!("Failed to serialize session data: {}", e))
        })?;

        // Encrypt with AES-GCM
        let encrypted_data = self.aes_service.encrypt(&data_bytes)?;

        let result = Ok(EncryptedSessionData {
            encrypted_data,
            metadata,
            algorithm: "AES-256-GCM".to_string(),
            key_id: "session-key-v1".to_string(),
        });

        let duration = start.elapsed();
        self.update_metrics("session_encryption", duration, result.is_ok())
            .await;

        result
    }

    /// Decrypt session data for SIMKARI users
    pub async fn decrypt_session_data(
        &self,
        encrypted_session: &EncryptedSessionData,
    ) -> Result<serde_json::Value> {
        let start = Instant::now();

        // Decrypt with AES-GCM
        let decrypted_bytes = self
            .aes_service
            .decrypt(&encrypted_session.encrypted_data)?;

        // Deserialize session data
        let result = serde_json::from_slice(&decrypted_bytes).map_err(|e| {
            AuthencError::validation(format!("Failed to deserialize session data: {}", e))
        });

        let duration = start.elapsed();
        self.update_metrics("session_encryption", duration, result.is_ok())
            .await;

        result
    }

    /// Generate audit signature for compliance
    pub async fn generate_audit_signature(
        &self,
        data: &[u8],
        signer: SignerInfo,
    ) -> Result<AuditSignature> {
        let start = Instant::now();

        let result = self.generate_classical_audit_signature(data, signer).await;

        let duration = start.elapsed();
        self.update_metrics("audit_signature", duration, result.is_ok())
            .await;

        result
    }

    /// Verify audit signature for compliance
    pub async fn verify_audit_signature(
        &self,
        data: &[u8],
        signature: &AuditSignature,
    ) -> Result<bool> {
        let start = Instant::now();

        let result = match signature.algorithm.as_str() {
            "Ed25519" => self.verify_classical_audit_signature(data, signature).await,
            _ => {
                warn!(
                    algorithm = signature.algorithm,
                    "Unknown signature algorithm"
                );
                Ok(false)
            }
        };

        let duration = start.elapsed();
        self.update_metrics("audit_signature", duration, result.is_ok())
            .await;

        result
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> CryptoMetrics {
        (*self.metrics.read().await).clone()
    }

    /// Clear performance metrics
    pub async fn clear_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = CryptoMetrics::default();
    }

    /// Clear validation cache
    pub async fn clear_validation_cache(&self) {
        let mut cache = self.validation_cache.write().await;
        cache.clear();
    }

    /// Set post-quantum mode
    pub fn set_pq_mode(&mut self, mode: PostQuantumMode) {
        self.pq_mode = mode;
        info!(mode = ?mode, "Post-quantum mode updated");
    }

    // Private helper methods

    async fn validate_single_token(
        &self,
        token: &str,
        index: usize,
    ) -> Result<TokenValidationResult> {
        // Validate JWT token
        let validation = Validation::new(Algorithm::EdDSA);
        let verifying_key = ED25519_KEYPAIR.verifying_key();
        let decoding_key = DecodingKey::from_ed_der(verifying_key.as_bytes());

        match decode::<PegawaiClaims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                let user_claims = token_data.claims.standard.clone();
                Ok(TokenValidationResult {
                    index,
                    valid: true,
                    error: None,
                    claims: Some(user_claims),
                })
            }
            Err(e) => Ok(TokenValidationResult {
                index,
                valid: false,
                error: Some(e.to_string()),
                claims: None,
            }),
        }
    }

    async fn get_cached_validation(&self, token: &str) -> Option<CachedValidation> {
        let cache = self.validation_cache.read().await;
        if let Some(cached) = cache.get(token) {
            if cached.expires_at > Utc::now() {
                return Some(cached.clone());
            }
        }
        None
    }

    async fn cache_validation_result(&self, token: &str, result: &TokenValidationResult) {
        let mut cache = self.validation_cache.write().await;
        let cached = CachedValidation {
            valid: result.valid,
            expires_at: Utc::now() + chrono::Duration::minutes(5),
            claims: result.claims.clone(),
        };
        cache.insert(token.to_string(), cached);
    }

    async fn generate_classical_audit_signature(
        &self,
        data: &[u8],
        signer: SignerInfo,
    ) -> Result<AuditSignature> {
        let signature: Signature = ED25519_KEYPAIR.sign(data);
        let signature_b64 = Base64UrlUnpadded::encode_string(&signature.to_bytes());

        Ok(AuditSignature {
            signature: signature_b64,
            algorithm: "Ed25519".to_string(),
            signer,
            signed_at: Utc::now(),
            pq_signature: None,
        })
    }

    async fn verify_classical_audit_signature(
        &self,
        data: &[u8],
        signature: &AuditSignature,
    ) -> Result<bool> {
        let sig_bytes = Base64UrlUnpadded::decode_vec(&signature.signature)
            .map_err(|_| AuthencError::validation("Invalid signature encoding"))?;

        let sig_array: [u8; 64] = sig_bytes
            .try_into()
            .map_err(|_| AuthencError::validation("Invalid signature length"))?;
        let sig = Signature::from_bytes(&sig_array);

        let verifying_key = ED25519_KEYPAIR.verifying_key();
        match verifying_key.verify(data, &sig) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn update_metrics(&self, operation: &str, duration: Duration, success: bool) {
        let mut metrics = self.metrics.write().await;
        let duration_us = duration.as_micros() as f64;
        let duration_ms_f64 = duration_us / 1000.0;
        let duration_ms = duration.as_millis() as u64;

        let op_metrics = match operation {
            "jwt_signing" => &mut metrics.jwt_signing,
            "jwt_verification" => &mut metrics.jwt_verification,
            "batch_validation" => &mut metrics.batch_validation,
            "session_encryption" => &mut metrics.session_encryption,
            "audit_signature" => &mut metrics.audit_signature,
            "post_quantum" => &mut metrics.post_quantum,
            _ => return,
        };

        op_metrics.count += 1;
        let old_avg = op_metrics.avg_time_ms;
        op_metrics.avg_time_ms = old_avg + (duration_ms_f64 - old_avg) / op_metrics.count as f64;
        op_metrics.total_time_ms += duration_ms;

        if op_metrics.count == 1 {
            op_metrics.min_time_ms = duration_ms;
            op_metrics.max_time_ms = duration_ms;
        } else {
            op_metrics.min_time_ms = op_metrics.min_time_ms.min(duration_ms);
            op_metrics.max_time_ms = op_metrics.max_time_ms.max(duration_ms);
        }

        if !success {
            op_metrics.failures += 1;
        }
    }
}

impl OperationMetrics {
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            ((self.count - self.failures) as f64 / self.count as f64) * 100.0
        }
    }

    /// Get operations per second based on total time
    pub fn ops_per_second(&self) -> f64 {
        if self.total_time_ms == 0 {
            0.0
        } else {
            (self.count as f64 * 1000.0) / self.total_time_ms as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enhanced_crypto_engine_creation() {
        let engine = EnhancedCryptoEngine::new();
        assert_eq!(engine.pq_mode, PostQuantumMode::Classical);
    }

    #[tokio::test]
    async fn test_pegawai_jwt_signing() {
        let engine = EnhancedCryptoEngine::new();

        let claims = PegawaiClaims {
            standard: UserClaims {
                sub: "user123".to_string(),
                username: "john.doe".to_string(),
                email: "john.doe@kejaksaan.go.id".to_string(),
                realm_id: "kejaksaan".to_string(),
                roles: vec!["pegawai".to_string()],
                exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                iat: Utc::now().timestamp() as usize,
                iss: "authenc".to_string(),
            },
            nip: "198501012010011001".to_string(),
            satker_code: "A.01.01".to_string(),
            jabatan: "Jaksa Muda".to_string(),
            eselon: Some("IV/a".to_string()),
            wilayah_code: Some("DKI".to_string()),
            admin_level: None,
            secreton_permissions: SecretonPermissions {
                read_secrets: vec!["config/*".to_string()],
                write_secrets: vec![],
                admin_operations: false,
                audit_access: false,
                satker_permissions: HashMap::new(),
            },
        };

        let result = engine.sign_jwt_for_pegawai(&claims).await;
        assert!(result.is_ok());

        let jwt = result.unwrap();
        assert!(!jwt.is_empty());
        assert!(jwt.contains('.'));
    }

    #[tokio::test]
    async fn test_session_data_encryption() {
        let engine = EnhancedCryptoEngine::new();

        let session_data = serde_json::json!({
            "user_id": "123",
            "preferences": {
                "theme": "dark",
                "language": "id"
            }
        });

        let metadata = SessionMetadata {
            session_id: "sess_123".to_string(),
            user_id: Uuid::new_v4(),
            client_ip: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
        };

        let encrypted = engine
            .encrypt_session_data(&session_data, metadata.clone())
            .await;
        assert!(encrypted.is_ok());

        let encrypted_session = encrypted.unwrap();
        assert_eq!(encrypted_session.algorithm, "AES-256-GCM");

        let decrypted = engine.decrypt_session_data(&encrypted_session).await;
        assert!(decrypted.is_ok());
        assert_eq!(decrypted.unwrap(), session_data);
    }

    #[tokio::test]
    async fn test_audit_signature_generation() {
        let engine = EnhancedCryptoEngine::new();

        let data = b"Important audit data";
        let signer = SignerInfo {
            user_id: Uuid::new_v4(),
            nip: "198501012010011001".to_string(),
            name: "John Doe".to_string(),
            role: "admin".to_string(),
            satker_code: "A.01.01".to_string(),
            admin_level: Some("satker".to_string()),
        };

        let signature = engine.generate_audit_signature(data, signer.clone()).await;
        assert!(signature.is_ok());

        let audit_sig = signature.unwrap();
        assert_eq!(audit_sig.algorithm, "Ed25519");
        assert_eq!(audit_sig.signer.nip, signer.nip);
        assert!(!audit_sig.signature.is_empty());

        let verification = engine.verify_audit_signature(data, &audit_sig).await;
        assert!(verification.is_ok());
        assert!(verification.unwrap());
    }
}
