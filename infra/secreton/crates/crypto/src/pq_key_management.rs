//! Post-Quantum Key Management for SIMKARI
//!
//! This module provides comprehensive post-quantum key management including:
//! - ML-KEM key encapsulation for long-term secrets
//! - ML-DSA signatures for authentication tokens
//! - Hybrid key exchange (X25519 + ML-KEM)
//! - Post-quantum archive security
//! - Key lifecycle management and rotation

use crate::{
    error::{CryptoError, CryptoResult},
    pqc::{
        mldsa::{MLDsaKeypair, MLDsaVariant},
        mlkem::{MLKemKeypair, MLKemVariant},
    },
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Post-quantum key management system
pub struct PostQuantumKeyManager {
    /// ML-KEM key pairs for key encapsulation
    mlkem_keys: Arc<RwLock<HashMap<String, MLKemKeyEntry>>>,
    /// ML-DSA key pairs for digital signatures
    mldsa_keys: Arc<RwLock<HashMap<String, MLDsaKeyEntry>>>,
    /// Hybrid key pairs (X25519 + ML-KEM)
    hybrid_keys: Arc<RwLock<HashMap<String, HybridKeyEntry>>>,
    /// Key rotation policies
    rotation_policies: Arc<RwLock<HashMap<String, KeyRotationPolicy>>>,
    /// Archive encryption keys for long-term storage
    archive_keys: Arc<RwLock<HashMap<String, ArchiveKeyEntry>>>,
    /// Performance metrics
    metrics: Arc<RwLock<KeyManagementMetrics>>,
}

/// ML-KEM key entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLKemKeyEntry {
    /// Key pair
    pub keypair: MLKemKeypair,
    /// Key identifier
    pub key_id: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Expiration timestamp
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Usage purpose
    pub purpose: KeyPurpose,
    /// Usage count
    pub usage_count: u64,
    /// Last used timestamp
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    /// Key status
    pub status: KeyStatus,
    /// Associated metadata
    pub metadata: HashMap<String, String>,
}

/// ML-DSA key entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLDsaKeyEntry {
    /// Key pair
    pub keypair: MLDsaKeypair,
    /// Key identifier
    pub key_id: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Expiration timestamp
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Usage purpose
    pub purpose: KeyPurpose,
    /// Usage count
    pub usage_count: u64,
    /// Last used timestamp
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    /// Key status
    pub status: KeyStatus,
    /// Associated metadata
    pub metadata: HashMap<String, String>,
}

/// Hybrid key entry (X25519 + ML-KEM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridKeyEntry {
    /// Classical X25519 key pair
    pub x25519_keypair: X25519KeyPair,
    /// ML-KEM key pair
    pub mlkem_keypair: MLKemKeypair,
    /// Key identifier
    pub key_id: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Expiration timestamp
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Usage purpose
    pub purpose: KeyPurpose,
    /// Usage count
    pub usage_count: u64,
    /// Last used timestamp
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    /// Key status
    pub status: KeyStatus,
    /// Associated metadata
    pub metadata: HashMap<String, String>,
}

/// Archive key entry for long-term storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveKeyEntry {
    /// ML-KEM key pair for key encapsulation
    pub mlkem_keypair: MLKemKeypair,
    /// ML-DSA key pair for integrity signatures
    pub mldsa_keypair: MLDsaKeypair,
    /// Key identifier
    pub key_id: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Archive retention period
    pub retention_period: chrono::Duration,
    /// Archive purpose
    pub purpose: ArchivePurpose,
    /// Usage count
    pub usage_count: u64,
    /// Last used timestamp
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    /// Key status
    pub status: KeyStatus,
    /// Associated metadata
    pub metadata: HashMap<String, String>,
}

/// X25519 key pair for classical ECDH
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct X25519KeyPair {
    /// Public key (32 bytes)
    pub public_key: [u8; 32],
    /// Private key (32 bytes)
    pub private_key: [u8; 32],
}

/// Key usage purpose
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KeyPurpose {
    /// Authentication token signing
    AuthenticationSigning,
    /// Long-term secret encryption
    SecretEncryption,
    /// Key exchange for session establishment
    KeyExchange,
    /// Archive encryption for compliance
    ArchiveEncryption,
    /// Audit log signing
    AuditSigning,
    /// Inter-service communication
    ServiceCommunication,
    /// User data encryption
    UserDataEncryption,
    /// Configuration encryption
    ConfigurationEncryption,
}

/// Archive purpose for long-term storage
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ArchivePurpose {
    /// Legal compliance archive
    LegalCompliance,
    /// Audit trail archive
    AuditTrail,
    /// Backup archive
    Backup,
    /// Historical data archive
    HistoricalData,
    /// Forensic evidence archive
    ForensicEvidence,
}

/// Key status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyStatus {
    /// Active and ready for use
    Active,
    /// Scheduled for rotation
    PendingRotation,
    /// Deprecated but still valid
    Deprecated,
    /// Revoked and should not be used
    Revoked,
    /// Expired
    Expired,
}

/// Key rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationPolicy {
    /// Policy identifier
    pub policy_id: String,
    /// Rotation interval
    pub rotation_interval: chrono::Duration,
    /// Maximum usage count before rotation
    pub max_usage_count: Option<u64>,
    /// Grace period after expiration
    pub grace_period: chrono::Duration,
    /// Automatic rotation enabled
    pub auto_rotation: bool,
    /// Notification settings
    pub notifications: RotationNotifications,
}

/// Rotation notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationNotifications {
    /// Notify before rotation
    pub notify_before_days: u32,
    /// Notify after rotation
    pub notify_after_rotation: bool,
    /// Notification endpoints
    pub endpoints: Vec<String>,
}

/// Key management performance metrics
#[derive(Debug, Default, Clone)]
pub struct KeyManagementMetrics {
    /// Key generation metrics
    pub key_generation: OperationMetrics,
    /// Key usage metrics
    pub key_usage: OperationMetrics,
    /// Key rotation metrics
    pub key_rotation: OperationMetrics,
    /// Archive operations
    pub archive_operations: OperationMetrics,
    /// Hybrid operations
    pub hybrid_operations: OperationMetrics,
    /// Current key counts by type
    pub key_counts: KeyCounts,
}

/// Operation metrics
#[derive(Debug, Default, Clone)]
pub struct OperationMetrics {
    /// Total operations
    pub count: u64,
    /// Total time (milliseconds)
    pub total_time_ms: u64,
    /// Average time (milliseconds)
    pub avg_time_ms: f64,
    /// Success count
    pub success_count: u64,
    /// Failure count
    pub failure_count: u64,
}

/// Key counts by type
#[derive(Debug, Default, Clone)]
pub struct KeyCounts {
    /// ML-KEM keys
    pub mlkem_keys: u64,
    /// ML-DSA keys
    pub mldsa_keys: u64,
    /// Hybrid keys
    pub hybrid_keys: u64,
    /// Archive keys
    pub archive_keys: u64,
    /// Active keys
    pub active_keys: u64,
    /// Expired keys
    pub expired_keys: u64,
}

/// Hybrid key exchange result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridKeyExchangeResult {
    /// Classical X25519 shared secret
    pub x25519_shared_secret: [u8; 32],
    /// ML-KEM shared secret
    pub mlkem_shared_secret: Vec<u8>,
    /// Combined shared secret (HKDF of both)
    pub combined_shared_secret: [u8; 32],
    /// ML-KEM ciphertext
    pub mlkem_ciphertext: Vec<u8>,
    /// Performance metrics
    pub performance: HybridKeyExchangePerformance,
}

/// Performance metrics for hybrid key exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridKeyExchangePerformance {
    /// Total operation time
    pub total_time_ms: u64,
    /// X25519 operation time
    pub x25519_time_ms: u64,
    /// ML-KEM operation time
    pub mlkem_time_ms: u64,
    /// HKDF operation time
    pub hkdf_time_ms: u64,
}

/// Archive encryption result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEncryptionResult {
    /// Encrypted data
    pub encrypted_data: Vec<u8>,
    /// ML-KEM encapsulated key
    pub encapsulated_key: Vec<u8>,
    /// ML-DSA integrity signature
    pub integrity_signature: Vec<u8>,
    /// Archive metadata
    pub metadata: ArchiveMetadata,
}

/// Archive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMetadata {
    /// Archive identifier
    pub archive_id: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Retention until timestamp
    pub retention_until: chrono::DateTime<chrono::Utc>,
    /// Archive purpose
    pub purpose: ArchivePurpose,
    /// Data classification
    pub classification: String,
    /// Checksum for integrity
    pub checksum: String,
}

impl Default for PostQuantumKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PostQuantumKeyManager {
    /// Create a new post-quantum key manager
    pub fn new() -> Self {
        Self {
            mlkem_keys: Arc::new(RwLock::new(HashMap::new())),
            mldsa_keys: Arc::new(RwLock::new(HashMap::new())),
            hybrid_keys: Arc::new(RwLock::new(HashMap::new())),
            rotation_policies: Arc::new(RwLock::new(HashMap::new())),
            archive_keys: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(KeyManagementMetrics::default())),
        }
    }

    /// Generate ML-KEM key pair for long-term secrets
    pub async fn generate_mlkem_key(
        &self,
        key_id: String,
        variant: MLKemVariant,
        purpose: KeyPurpose,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        metadata: HashMap<String, String>,
    ) -> CryptoResult<String> {
        let start = Instant::now();

        // Generate the key pair
        let keypair = MLKemKeypair::generate(variant)?;

        let entry = MLKemKeyEntry {
            keypair,
            key_id: key_id.clone(),
            created_at: chrono::Utc::now(),
            expires_at,
            purpose,
            usage_count: 0,
            last_used: None,
            status: KeyStatus::Active,
            metadata,
        };

        // Store the key
        {
            let mut keys = self.mlkem_keys.write().await;
            keys.insert(key_id.clone(), entry);
        }

        // Update metrics
        let duration = start.elapsed();
        self.update_generation_metrics(duration, true).await;

        info!(
            key_id = %key_id,
            variant = %variant,
            purpose = ?purpose,
            duration_ms = duration.as_millis(),
            "ML-KEM key pair generated"
        );

        Ok(key_id)
    }

    /// Generate ML-DSA key pair for authentication tokens
    pub async fn generate_mldsa_key(
        &self,
        key_id: String,
        variant: MLDsaVariant,
        purpose: KeyPurpose,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        metadata: HashMap<String, String>,
    ) -> CryptoResult<String> {
        let start = Instant::now();

        // Generate the key pair
        let keypair = MLDsaKeypair::generate(variant)?;

        let entry = MLDsaKeyEntry {
            keypair,
            key_id: key_id.clone(),
            created_at: chrono::Utc::now(),
            expires_at,
            purpose,
            usage_count: 0,
            last_used: None,
            status: KeyStatus::Active,
            metadata,
        };

        // Store the key
        {
            let mut keys = self.mldsa_keys.write().await;
            keys.insert(key_id.clone(), entry);
        }

        // Update metrics
        let duration = start.elapsed();
        self.update_generation_metrics(duration, true).await;

        info!(
            key_id = %key_id,
            variant = %variant,
            purpose = ?purpose,
            duration_ms = duration.as_millis(),
            "ML-DSA key pair generated"
        );

        Ok(key_id)
    }

    /// Generate hybrid key pair (X25519 + ML-KEM)
    pub async fn generate_hybrid_key(
        &self,
        key_id: String,
        mlkem_variant: MLKemVariant,
        purpose: KeyPurpose,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        metadata: HashMap<String, String>,
    ) -> CryptoResult<String> {
        let start = Instant::now();

        // Generate X25519 key pair
        let x25519_keypair = self.generate_x25519_keypair()?;

        // Generate ML-KEM key pair
        let mlkem_keypair = MLKemKeypair::generate(mlkem_variant)?;

        let entry = HybridKeyEntry {
            x25519_keypair,
            mlkem_keypair,
            key_id: key_id.clone(),
            created_at: chrono::Utc::now(),
            expires_at,
            purpose,
            usage_count: 0,
            last_used: None,
            status: KeyStatus::Active,
            metadata,
        };

        // Store the key
        {
            let mut keys = self.hybrid_keys.write().await;
            keys.insert(key_id.clone(), entry);
        }

        // Update metrics
        let duration = start.elapsed();
        self.update_generation_metrics(duration, true).await;

        info!(
            key_id = %key_id,
            mlkem_variant = %mlkem_variant,
            purpose = ?purpose,
            duration_ms = duration.as_millis(),
            "Hybrid key pair generated"
        );

        Ok(key_id)
    }

    /// Generate archive key pair for long-term storage
    pub async fn generate_archive_key(
        &self,
        key_id: String,
        mlkem_variant: MLKemVariant,
        mldsa_variant: MLDsaVariant,
        purpose: ArchivePurpose,
        retention_period: chrono::Duration,
        metadata: HashMap<String, String>,
    ) -> CryptoResult<String> {
        let start = Instant::now();

        // Generate ML-KEM key pair for encryption
        let mlkem_keypair = MLKemKeypair::generate(mlkem_variant)?;

        // Generate ML-DSA key pair for integrity signatures
        let mldsa_keypair = MLDsaKeypair::generate(mldsa_variant)?;

        let entry = ArchiveKeyEntry {
            mlkem_keypair,
            mldsa_keypair,
            key_id: key_id.clone(),
            created_at: chrono::Utc::now(),
            retention_period,
            purpose,
            usage_count: 0,
            last_used: None,
            status: KeyStatus::Active,
            metadata,
        };

        // Store the key
        {
            let mut keys = self.archive_keys.write().await;
            keys.insert(key_id.clone(), entry);
        }

        // Update metrics
        let duration = start.elapsed();
        self.update_generation_metrics(duration, true).await;

        info!(
            key_id = %key_id,
            mlkem_variant = %mlkem_variant,
            mldsa_variant = %mldsa_variant,
            purpose = ?purpose,
            retention_days = retention_period.num_days(),
            duration_ms = duration.as_millis(),
            "Archive key pair generated"
        );

        Ok(key_id)
    }

    /// Perform hybrid key exchange (X25519 + ML-KEM)
    pub async fn hybrid_key_exchange(
        &self,
        key_id: &str,
        peer_x25519_public: &[u8; 32],
        peer_mlkem_public: &[u8],
    ) -> CryptoResult<HybridKeyExchangeResult> {
        let start = Instant::now();

        // Get the hybrid key
        let hybrid_key = {
            let mut keys = self.hybrid_keys.write().await;
            let entry = keys.get_mut(key_id)
                .ok_or_else(|| CryptoError::InvalidKey(format!("Hybrid key not found: {}", key_id)))?;

            if entry.status != KeyStatus::Active {
                return Err(CryptoError::InvalidKey(format!("Key is not active: {}", key_id)));
            }

            // Update usage
            entry.usage_count += 1;
            entry.last_used = Some(chrono::Utc::now());

            entry.clone()
        };

        // Perform X25519 key exchange
        let x25519_start = Instant::now();
        let x25519_shared_secret = self.x25519_key_exchange(
            &hybrid_key.x25519_keypair.private_key,
            peer_x25519_public,
        )?;
        let x25519_time = x25519_start.elapsed();

        // Perform ML-KEM encapsulation
        let mlkem_start = Instant::now();
        let mlkem_keypair_for_encap = MLKemKeypair {
            public_key: peer_mlkem_public.to_vec(),
            private_key: Vec::new(), // Not needed for encapsulation
            variant: hybrid_key.mlkem_keypair.variant,
        };
        let (mlkem_shared_secret, mlkem_ciphertext) = mlkem_keypair_for_encap.encapsulate()?;
        let mlkem_time = mlkem_start.elapsed();

        // Combine secrets using HKDF
        let hkdf_start = Instant::now();
        let combined_shared_secret = self.combine_shared_secrets(
            &x25519_shared_secret,
            &mlkem_shared_secret,
        )?;
        let hkdf_time = hkdf_start.elapsed();

        let total_time = start.elapsed();

        // Update metrics
        self.update_hybrid_metrics(total_time, true).await;

        debug!(
            key_id = %key_id,
            x25519_time_ms = x25519_time.as_millis(),
            mlkem_time_ms = mlkem_time.as_millis(),
            hkdf_time_ms = hkdf_time.as_millis(),
            total_time_ms = total_time.as_millis(),
            "Hybrid key exchange completed"
        );

        Ok(HybridKeyExchangeResult {
            x25519_shared_secret,
            mlkem_shared_secret,
            combined_shared_secret,
            mlkem_ciphertext,
            performance: HybridKeyExchangePerformance {
                total_time_ms: total_time.as_millis() as u64,
                x25519_time_ms: x25519_time.as_millis() as u64,
                mlkem_time_ms: mlkem_time.as_millis() as u64,
                hkdf_time_ms: hkdf_time.as_millis() as u64,
            },
        })
    }

    /// Encrypt data for archive storage
    pub async fn encrypt_for_archive(
        &self,
        key_id: &str,
        data: &[u8],
        archive_id: String,
        classification: String,
    ) -> CryptoResult<ArchiveEncryptionResult> {
        let start = Instant::now();

        // Get the archive key
        let archive_key = {
            let mut keys = self.archive_keys.write().await;
            let entry = keys.get_mut(key_id)
                .ok_or_else(|| CryptoError::InvalidKey(format!("Archive key not found: {}", key_id)))?;

            if entry.status != KeyStatus::Active {
                return Err(CryptoError::InvalidKey(format!("Archive key is not active: {}", key_id)));
            }

            // Update usage
            entry.usage_count += 1;
            entry.last_used = Some(chrono::Utc::now());

            entry.clone()
        };

        // Generate a random symmetric key for data encryption
        let symmetric_key = crate::generate_key(crate::AlgorithmId::Aes256Gcm)?;

        // Encrypt the data with the symmetric key
        let crypto_engine = crate::encryption::CryptoEngine::new();
        let encrypted_data_result = crypto_engine.encrypt(
            crate::AlgorithmId::Aes256Gcm,
            data,
            &symmetric_key,
        )?;

        // Encapsulate the symmetric key using ML-KEM
        let (_encapsulated_symmetric_key, encapsulated_key) = archive_key.mlkem_keypair.encapsulate()?;

        // Create integrity signature using ML-DSA
        let integrity_data = [data, &encapsulated_key].concat();
        let integrity_signature = archive_key.mldsa_keypair.sign(&integrity_data)?;

        // Calculate checksum
        let checksum = self.calculate_checksum(data)?;

        // Create metadata
        let metadata = ArchiveMetadata {
            archive_id,
            created_at: chrono::Utc::now(),
            retention_until: chrono::Utc::now() + archive_key.retention_period,
            purpose: archive_key.purpose,
            classification,
            checksum,
        };

        let duration = start.elapsed();
        self.update_archive_metrics(duration, true).await;

        info!(
            key_id = %key_id,
            archive_id = %metadata.archive_id,
            data_size = data.len(),
            duration_ms = duration.as_millis(),
            "Data encrypted for archive"
        );

        Ok(ArchiveEncryptionResult {
            encrypted_data: encrypted_data_result.ciphertext,
            encapsulated_key,
            integrity_signature,
            metadata,
        })
    }

    /// Sign authentication token using ML-DSA
    pub async fn sign_authentication_token(
        &self,
        key_id: &str,
        token_data: &[u8],
    ) -> CryptoResult<Vec<u8>> {
        let start = Instant::now();

        // Get the ML-DSA key
        let mldsa_key = {
            let mut keys = self.mldsa_keys.write().await;
            let entry = keys.get_mut(key_id)
                .ok_or_else(|| CryptoError::InvalidKey(format!("ML-DSA key not found: {}", key_id)))?;

            if entry.status != KeyStatus::Active {
                return Err(CryptoError::InvalidKey(format!("Key is not active: {}", key_id)));
            }

            // Check if key is for authentication
            if !matches!(entry.purpose, KeyPurpose::AuthenticationSigning) {
                return Err(CryptoError::InvalidKey("Key is not for authentication signing".to_string()));
            }

            // Update usage
            entry.usage_count += 1;
            entry.last_used = Some(chrono::Utc::now());

            entry.clone()
        };

        // Sign the token
        let signature = mldsa_key.keypair.sign(token_data)?;

        let duration = start.elapsed();
        self.update_usage_metrics(duration, true).await;

        debug!(
            key_id = %key_id,
            token_size = token_data.len(),
            signature_size = signature.len(),
            duration_ms = duration.as_millis(),
            "Authentication token signed"
        );

        Ok(signature)
    }

    /// Get key management metrics
    pub async fn get_metrics(&self) -> KeyManagementMetrics {
        let mut metrics = (*self.metrics.read().await).clone();

        // Update key counts
        let mlkem_count = self.mlkem_keys.read().await.len() as u64;
        let mldsa_count = self.mldsa_keys.read().await.len() as u64;
        let hybrid_count = self.hybrid_keys.read().await.len() as u64;
        let archive_count = self.archive_keys.read().await.len() as u64;

        metrics.key_counts = KeyCounts {
            mlkem_keys: mlkem_count,
            mldsa_keys: mldsa_count,
            hybrid_keys: hybrid_count,
            archive_keys: archive_count,
            active_keys: self.count_active_keys().await,
            expired_keys: self.count_expired_keys().await,
        };

        metrics
    }

    /// Set key rotation policy
    pub async fn set_rotation_policy(
        &self,
        policy_id: String,
        policy: KeyRotationPolicy,
    ) -> CryptoResult<()> {
        let mut policies = self.rotation_policies.write().await;
        policies.insert(policy_id, policy);
        Ok(())
    }

    /// Check and perform key rotations
    pub async fn check_key_rotations(&self) -> CryptoResult<Vec<String>> {
        let mut rotated_keys = Vec::new();

        // Check ML-KEM keys
        rotated_keys.extend(self.check_mlkem_rotations().await?);

        // Check ML-DSA keys
        rotated_keys.extend(self.check_mldsa_rotations().await?);

        // Check hybrid keys
        rotated_keys.extend(self.check_hybrid_rotations().await?);

        // Check archive keys
        rotated_keys.extend(self.check_archive_rotations().await?);

        if !rotated_keys.is_empty() {
            info!(
                rotated_count = rotated_keys.len(),
                "Key rotation check completed"
            );
        }

        Ok(rotated_keys)
    }

    // Private helper methods

    fn generate_x25519_keypair(&self) -> CryptoResult<X25519KeyPair> {
        use rand::RngCore;
        let mut rng = rand::rngs::OsRng;

        let mut private_key = [0u8; 32];
        rng.fill_bytes(&mut private_key);

        // Clamp the private key (X25519 requirement)
        private_key[0] &= 248;
        private_key[31] &= 127;
        private_key[31] |= 64;

        // Generate public key using curve25519-dalek
        use curve25519_dalek::{scalar::Scalar, constants::X25519_BASEPOINT};
        let scalar = Scalar::from_bytes_mod_order(private_key);
        let public_key_point = scalar * X25519_BASEPOINT;
        let public_key = public_key_point.to_bytes();

        Ok(X25519KeyPair {
            public_key,
            private_key,
        })
    }

    fn x25519_key_exchange(
        &self,
        private_key: &[u8; 32],
        peer_public_key: &[u8; 32],
    ) -> CryptoResult<[u8; 32]> {
        use curve25519_dalek::scalar::Scalar;

        let scalar = Scalar::from_bytes_mod_order(*private_key);
        let peer_point = curve25519_dalek::montgomery::MontgomeryPoint(*peer_public_key);
        let shared_secret = scalar * peer_point;

        Ok(shared_secret.to_bytes())
    }

    fn combine_shared_secrets(
        &self,
        x25519_secret: &[u8; 32],
        mlkem_secret: &[u8],
    ) -> CryptoResult<[u8; 32]> {
        use hkdf::Hkdf;
        use sha2::Sha256;

        // Combine the secrets using HKDF
        let combined_input = [x25519_secret.as_slice(), mlkem_secret].concat();
        let hk = Hkdf::<Sha256>::new(None, &combined_input);

        let mut output = [0u8; 32];
        hk.expand(b"SIMKARI-hybrid-key-exchange", &mut output)
            .map_err(|_| CryptoError::Internal("HKDF expansion failed".to_string()))?;

        Ok(output)
    }

    fn calculate_checksum(&self, data: &[u8]) -> CryptoResult<String> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        Ok(format!("{:x}", hasher.finalize()))
    }

    async fn count_active_keys(&self) -> u64 {
        let mut count = 0;

        // Count active ML-KEM keys
        let mlkem_keys = self.mlkem_keys.read().await;
        count += mlkem_keys.values()
            .filter(|k| k.status == KeyStatus::Active)
            .count() as u64;

        // Count active ML-DSA keys
        let mldsa_keys = self.mldsa_keys.read().await;
        count += mldsa_keys.values()
            .filter(|k| k.status == KeyStatus::Active)
            .count() as u64;

        // Count active hybrid keys
        let hybrid_keys = self.hybrid_keys.read().await;
        count += hybrid_keys.values()
            .filter(|k| k.status == KeyStatus::Active)
            .count() as u64;

        // Count active archive keys
        let archive_keys = self.archive_keys.read().await;
        count += archive_keys.values()
            .filter(|k| k.status == KeyStatus::Active)
            .count() as u64;

        count
    }

    async fn count_expired_keys(&self) -> u64 {
        let now = chrono::Utc::now();
        let mut count = 0;

        // Count expired ML-KEM keys
        let mlkem_keys = self.mlkem_keys.read().await;
        count += mlkem_keys.values()
            .filter(|k| k.expires_at.is_some_and(|exp| exp < now))
            .count() as u64;

        // Count expired ML-DSA keys
        let mldsa_keys = self.mldsa_keys.read().await;
        count += mldsa_keys.values()
            .filter(|k| k.expires_at.is_some_and(|exp| exp < now))
            .count() as u64;

        // Count expired hybrid keys
        let hybrid_keys = self.hybrid_keys.read().await;
        count += hybrid_keys.values()
            .filter(|k| k.expires_at.is_some_and(|exp| exp < now))
            .count() as u64;

        count
    }

    async fn check_mlkem_rotations(&self) -> CryptoResult<Vec<String>> {
        // Placeholder for ML-KEM key rotation logic
        Ok(Vec::new())
    }

    async fn check_mldsa_rotations(&self) -> CryptoResult<Vec<String>> {
        // Placeholder for ML-DSA key rotation logic
        Ok(Vec::new())
    }

    async fn check_hybrid_rotations(&self) -> CryptoResult<Vec<String>> {
        // Placeholder for hybrid key rotation logic
        Ok(Vec::new())
    }

    async fn check_archive_rotations(&self) -> CryptoResult<Vec<String>> {
        // Placeholder for archive key rotation logic
        Ok(Vec::new())
    }



    async fn update_generation_metrics(&self, duration: std::time::Duration, success: bool) {
        let mut metrics = self.metrics.write().await;
        let duration_ms = duration.as_millis() as u64;

        metrics.key_generation.count += 1;
        metrics.key_generation.total_time_ms += duration_ms;
        metrics.key_generation.avg_time_ms =
            metrics.key_generation.total_time_ms as f64 / metrics.key_generation.count as f64;

        if success {
            metrics.key_generation.success_count += 1;
        } else {
            metrics.key_generation.failure_count += 1;
        }
    }

    async fn update_usage_metrics(&self, duration: std::time::Duration, success: bool) {
        let mut metrics = self.metrics.write().await;
        let duration_ms = duration.as_millis() as u64;

        metrics.key_usage.count += 1;
        metrics.key_usage.total_time_ms += duration_ms;
        metrics.key_usage.avg_time_ms =
            metrics.key_usage.total_time_ms as f64 / metrics.key_usage.count as f64;

        if success {
            metrics.key_usage.success_count += 1;
        } else {
            metrics.key_usage.failure_count += 1;
        }
    }

    async fn update_hybrid_metrics(&self, duration: std::time::Duration, success: bool) {
        let mut metrics = self.metrics.write().await;
        let duration_ms = duration.as_millis() as u64;

        metrics.hybrid_operations.count += 1;
        metrics.hybrid_operations.total_time_ms += duration_ms;
        metrics.hybrid_operations.avg_time_ms =
            metrics.hybrid_operations.total_time_ms as f64 / metrics.hybrid_operations.count as f64;

        if success {
            metrics.hybrid_operations.success_count += 1;
        } else {
            metrics.hybrid_operations.failure_count += 1;
        }
    }

    async fn update_archive_metrics(&self, duration: std::time::Duration, success: bool) {
        let mut metrics = self.metrics.write().await;
        let duration_ms = duration.as_millis() as u64;

        metrics.archive_operations.count += 1;
        metrics.archive_operations.total_time_ms += duration_ms;
        metrics.archive_operations.avg_time_ms =
            metrics.archive_operations.total_time_ms as f64 / metrics.archive_operations.count as f64;

        if success {
            metrics.archive_operations.success_count += 1;
        } else {
            metrics.archive_operations.failure_count += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_key_manager_creation() {
        let manager = PostQuantumKeyManager::new();
        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.key_counts.mlkem_keys, 0);
        assert_eq!(metrics.key_counts.mldsa_keys, 0);
    }

    #[tokio::test]
    async fn test_mlkem_key_generation() {
        let manager = PostQuantumKeyManager::new();

        let key_id = manager.generate_mlkem_key(
            "test-mlkem-key".to_string(),
            MLKemVariant::MLKem768,
            KeyPurpose::SecretEncryption,
            Some(chrono::Utc::now() + chrono::Duration::days(30)),
            HashMap::new(),
        ).await;

        assert!(key_id.is_ok());

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.key_counts.mlkem_keys, 1);
    }

    #[tokio::test]
    async fn test_mldsa_key_generation() {
        let manager = PostQuantumKeyManager::new();

        let key_id = manager.generate_mldsa_key(
            "test-mldsa-key".to_string(),
            MLDsaVariant::MLDsa65,
            KeyPurpose::AuthenticationSigning,
            Some(chrono::Utc::now() + chrono::Duration::days(30)),
            HashMap::new(),
        ).await;

        assert!(key_id.is_ok());

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.key_counts.mldsa_keys, 1);
    }

    #[tokio::test]
    async fn test_hybrid_key_generation() {
        let manager = PostQuantumKeyManager::new();

        let key_id = manager.generate_hybrid_key(
            "test-hybrid-key".to_string(),
            MLKemVariant::MLKem768,
            KeyPurpose::KeyExchange,
            Some(chrono::Utc::now() + chrono::Duration::days(30)),
            HashMap::new(),
        ).await;

        assert!(key_id.is_ok());

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.key_counts.hybrid_keys, 1);
    }

    #[tokio::test]
    async fn test_archive_key_generation() {
        let manager = PostQuantumKeyManager::new();

        let key_id = manager.generate_archive_key(
            "test-archive-key".to_string(),
            MLKemVariant::MLKem1024,
            MLDsaVariant::MLDsa87,
            ArchivePurpose::LegalCompliance,
            chrono::Duration::days(2555), // 7 years
            HashMap::new(),
        ).await;

        assert!(key_id.is_ok());

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.key_counts.archive_keys, 1);
    }

    #[tokio::test]
    async fn test_authentication_token_signing() {
        let manager = PostQuantumKeyManager::new();

        // Generate ML-DSA key for authentication
        let key_id = manager.generate_mldsa_key(
            "auth-key".to_string(),
            MLDsaVariant::MLDsa65,
            KeyPurpose::AuthenticationSigning,
            None,
            HashMap::new(),
        ).await.unwrap();

        // Sign a token
        let token_data = b"authentication-token-data";
        let signature = manager.sign_authentication_token(&key_id, token_data).await;

        assert!(signature.is_ok());
        let sig = signature.unwrap();
        assert!(!sig.is_empty());
    }
}
