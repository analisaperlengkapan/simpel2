//! Seal/Unseal Enhancement
//!
//! Enhanced seal/unseal operations with progress tracking, rekey support,
//! and seal migration capabilities. Integrated with Shamir Secret Sharing
//! for secure master key distribution.

use chrono::{DateTime, Utc};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use zeroize::{Zeroize, ZeroizeOnDrop};

// Import Shamir Secret Sharing from crypto crate
use lib_crypto::shamir::{
    Commitment, ShamirConfig, Share, generate_shares_with_commitments, reconstruct_secret_verified,
    validate_shares,
};

// Import crypto primitives for master key encryption
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use sha2::{Digest, Sha256};
use tracing::instrument;

/// Seal errors
#[derive(Debug, thiserror::Error)]
pub enum SealError {
    #[error("Already sealed")]
    AlreadySealed,

    #[error("Already unsealed")]
    AlreadyUnsealed,

    #[error("Invalid unseal key")]
    InvalidUnsealKey,

    #[error("Threshold not met: {0}/{1}")]
    ThresholdNotMet(usize, usize),

    #[error("Rekey in progress")]
    RekeyInProgress,

    #[error("No rekey in progress")]
    NoRekeyInProgress,

    #[error("Vault not initialized")]
    NotInitialized,

    #[error("Invalid configuration")]
    InvalidConfiguration,

    #[error("Shamir secret sharing error")]
    ShamirError,

    #[error("Share verification failed")]
    ShareVerificationFailed,

    #[error("Secret reconstruction failed")]
    ReconstructionFailed,

    #[error("Vault is sealed")]
    VaultSealed,

    #[error("Master key not available")]
    MasterKeyNotAvailable,

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
}

/// Seal status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SealState {
    /// Sealed
    Sealed,

    /// Unsealing in progress
    Unsealing,

    /// Unsealed
    Unsealed,
}

/// Seal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealConfig {
    /// Seal type ("shamir" or "auto")
    pub seal_type: String,

    /// Secret shares (N)
    pub secret_shares: usize,

    /// Secret threshold (T)
    pub secret_threshold: usize,

    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Vault state stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultState {
    /// Encrypted master key
    pub encrypted_master_key: Vec<u8>,

    /// Seal configuration
    pub seal_config: SealConfig,

    /// Shamir commitments for verification
    pub shamir_commitments: Vec<u8>, // Serialized Commitment

    /// Encryption metadata
    pub encryption_metadata: EncryptionMetadata,

    /// Version for key rotation
    pub version: u32,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Updated at
    pub updated_at: DateTime<Utc>,
}

/// Encryption metadata for master key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Algorithm used (aes-256-gcm)
    pub algorithm: String,

    /// Nonce/IV used for encryption
    pub nonce: Vec<u8>,

    /// Salt used for key derivation
    pub salt: Vec<u8>,

    /// Key derivation function (argon2id)
    pub kdf: String,

    /// KDF parameters
    pub kdf_params: KdfParams,
}

/// Key derivation function parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    /// Memory cost in KiB
    pub memory_cost: u32,

    /// Time cost (iterations)
    pub time_cost: u32,

    /// Parallelism factor
    pub parallelism: u32,
}

impl Default for SealConfig {
    fn default() -> Self {
        Self {
            seal_type: "shamir".to_string(),
            secret_shares: 5,
            secret_threshold: 3,
            created_at: Utc::now(),
        }
    }
}

/// Seal status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealStatus {
    /// Current state
    pub state: SealState,

    /// Seal type
    pub seal_type: String,

    /// Is initialized
    pub initialized: bool,

    /// Total shares required
    pub total_shares: usize,

    /// Threshold
    pub threshold: usize,

    /// Current progress (shares provided)
    pub progress: usize,

    /// Nonce (for unseal session)
    pub nonce: Option<String>,

    /// Version
    pub version: String,
}

impl SealStatus {
    fn new(config: &SealConfig, state: SealState, progress: usize) -> Self {
        Self {
            state,
            seal_type: config.seal_type.clone(),
            initialized: true,
            total_shares: config.secret_shares,
            threshold: config.secret_threshold,
            progress,
            nonce: None,
            version: "1.0.0".to_string(),
        }
    }
}

/// Rekey operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekeyOperation {
    /// New shares
    pub new_shares: usize,

    /// New threshold
    pub new_threshold: usize,

    /// Progress (master key shares provided)
    pub progress: usize,

    /// Required shares to authorize rekey
    pub required: usize,

    /// Started at
    pub started_at: DateTime<Utc>,

    /// Nonce
    pub nonce: String,
}

/// Master key wrapper with zeroization
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
struct MasterKey {
    #[zeroize(skip)]
    key: Option<Vec<u8>>,
}

impl MasterKey {
    fn new(key: Vec<u8>) -> Self {
        Self { key: Some(key) }
    }

    fn get(&self) -> Option<&[u8]> {
        self.key.as_deref()
    }

    fn clear(&mut self) {
        if let Some(ref mut k) = self.key {
            k.zeroize();
        }
        self.key = None;
    }
}

/// Seal/Unseal service with Shamir Secret Sharing integration
pub struct SealService {
    config: Arc<RwLock<SealConfig>>,
    state: Arc<RwLock<SealState>>,
    master_key: Arc<RwLock<MasterKey>>, // Master key (only in memory when unsealed)
    unseal_shares: Arc<RwLock<Vec<Share>>>, // Collected unseal shares
    commitment: Arc<RwLock<Option<Commitment>>>, // Feldman VSS commitment for verification
    rekey_operation: Arc<RwLock<Option<RekeyOperation>>>,
    storage_backend: Option<Arc<dyn VaultStateStorage>>, // Optional storage for persistence
}

/// Storage backend trait for vault state persistence
/// This is a minimal trait to avoid circular dependencies
#[async_trait::async_trait]
pub trait VaultStateStorage: Send + Sync {
    /// Store vault state
    async fn store_vault_state(&self, state: &VaultState) -> Result<(), String>;

    /// Load vault state
    async fn load_vault_state(&self) -> Result<Option<VaultState>, String>;
}

// Implement a simple in-memory storage for testing
pub struct InMemoryVaultStateStorage {
    state: Arc<RwLock<Option<VaultState>>>,
}

impl Default for InMemoryVaultStateStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryVaultStateStorage {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl VaultStateStorage for InMemoryVaultStateStorage {
    async fn store_vault_state(&self, vault_state: &VaultState) -> Result<(), String> {
        let mut state = self.state.write().await;
        *state = Some(vault_state.clone());
        Ok(())
    }

    async fn load_vault_state(&self) -> Result<Option<VaultState>, String> {
        let state = self.state.read().await;
        Ok(state.clone())
    }
}

impl SealService {
    /// Create new seal service
    pub fn new(config: SealConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(SealState::Sealed)),
            master_key: Arc::new(RwLock::new(MasterKey { key: None })),
            unseal_shares: Arc::new(RwLock::new(Vec::new())),
            commitment: Arc::new(RwLock::new(None)),
            rekey_operation: Arc::new(RwLock::new(None)),
            storage_backend: None,
        }
    }

    /// Create new seal service with storage backend
    pub fn with_storage(config: SealConfig, storage: Arc<dyn VaultStateStorage>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(SealState::Sealed)),
            master_key: Arc::new(RwLock::new(MasterKey { key: None })),
            unseal_shares: Arc::new(RwLock::new(Vec::new())),
            commitment: Arc::new(RwLock::new(None)),
            rekey_operation: Arc::new(RwLock::new(None)),
            storage_backend: Some(storage),
        }
    }

    /// Derive encryption key from seal key (reconstructed from shares)
    /// Uses Argon2id for key derivation
    fn derive_encryption_key(seal_key: &[u8], salt: &[u8]) -> Result<Vec<u8>, SealError> {
        // Use Argon2id with secure parameters
        let argon2 = Argon2::default();

        // Create a password hash with the seal key
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| SealError::KeyDerivationFailed(e.to_string()))?;

        let password_hash = argon2
            .hash_password(seal_key, &salt_string)
            .map_err(|e| SealError::KeyDerivationFailed(e.to_string()))?;

        // Extract the hash bytes (32 bytes for AES-256)
        let hash_bytes = password_hash
            .hash
            .ok_or_else(|| SealError::KeyDerivationFailed("No hash generated".to_string()))?;

        Ok(hash_bytes.as_bytes()[..32].to_vec())
    }

    /// Encrypt master key with seal key
    fn encrypt_master_key(
        master_key: &[u8],
        seal_key: &[u8],
    ) -> Result<(Vec<u8>, EncryptionMetadata), SealError> {
        // Generate random salt for key derivation
        let mut salt = vec![0u8; 16];
        (&mut OsRng).fill_bytes(&mut salt);

        // Derive encryption key from seal key
        let encryption_key = Self::derive_encryption_key(seal_key, &salt)?;

        // Generate random nonce for AES-GCM
        let mut nonce_bytes = [0u8; 12];
        (&mut OsRng).fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        // Create AES-256-GCM cipher
        let cipher = Aes256Gcm::new_from_slice(&encryption_key)
            .map_err(|e| SealError::EncryptionFailed(e.to_string()))?;

        // Encrypt master key
        let ciphertext = cipher
            .encrypt(&nonce, master_key)
            .map_err(|e| SealError::EncryptionFailed(e.to_string()))?;

        // Create encryption metadata
        let metadata = EncryptionMetadata {
            algorithm: "aes-256-gcm".to_string(),
            nonce: nonce_bytes.to_vec(),
            salt,
            kdf: "argon2id".to_string(),
            kdf_params: KdfParams {
                memory_cost: 19456, // 19 MiB
                time_cost: 2,
                parallelism: 1,
            },
        };

        Ok((ciphertext, metadata))
    }

    /// Decrypt master key with seal key
    fn decrypt_master_key(
        encrypted_master_key: &[u8],
        seal_key: &[u8],
        metadata: &EncryptionMetadata,
    ) -> Result<Vec<u8>, SealError> {
        // Derive encryption key from seal key
        let encryption_key = Self::derive_encryption_key(seal_key, &metadata.salt)?;

        // Create AES-256-GCM cipher
        let cipher = Aes256Gcm::new_from_slice(&encryption_key)
            .map_err(|e| SealError::DecryptionFailed(e.to_string()))?;

        // Decrypt master key
        if metadata.nonce.len() != 12 {
            return Err(SealError::DecryptionFailed(
                "Invalid nonce size".to_string(),
            ));
        }
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(&metadata.nonce);
        let nonce = Nonce::from(nonce_arr);
        let plaintext = cipher
            .decrypt(&nonce, encrypted_master_key)
            .map_err(|e| SealError::DecryptionFailed(e.to_string()))?;

        Ok(plaintext)
    }

    /// Store encrypted master key to storage backend
    async fn store_encrypted_master_key(
        &self,
        encrypted_master_key: Vec<u8>,
        metadata: EncryptionMetadata,
        commitment: &Commitment,
    ) -> Result<(), SealError> {
        let storage = self
            .storage_backend
            .as_ref()
            .ok_or_else(|| SealError::StorageError("No storage backend configured".to_string()))?;

        let config = self.config.read().await;

        // Serialize commitment to JSON
        let commitment_bytes = serde_json::to_vec(commitment).map_err(|e| {
            SealError::StorageError(format!("Failed to serialize commitment: {}", e))
        })?;

        let vault_state = VaultState {
            encrypted_master_key,
            seal_config: config.clone(),
            shamir_commitments: commitment_bytes,
            encryption_metadata: metadata,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        storage
            .store_vault_state(&vault_state)
            .await
            .map_err(SealError::StorageError)?;

        Ok(())
    }

    /// Load encrypted master key from storage backend
    async fn load_encrypted_master_key(&self) -> Result<Option<VaultState>, SealError> {
        let storage = self
            .storage_backend
            .as_ref()
            .ok_or_else(|| SealError::StorageError("No storage backend configured".to_string()))?;

        storage
            .load_vault_state()
            .await
            .map_err(SealError::StorageError)
    }

    /// Initialize vault with new master key and generate Shamir shares
    /// Returns the shares that must be distributed to operators
    /// This should only be called once during initial setup
    ///
    /// CRITICAL SECURITY: After initialization, the vault remains SEALED.
    /// Operators must manually unseal with threshold shares.
    /// This follows HashiCorp Vault security best practices.
    #[instrument(skip(self), fields(operation = "initialize_vault"))]
    pub async fn initialize(&self) -> Result<Vec<Share>, SealError> {
        let state = self.state.read().await;
        if *state != SealState::Sealed {
            return Err(SealError::AlreadyUnsealed);
        }
        drop(state);

        let config = self.config.read().await;
        let threshold = config.secret_threshold;
        let num_shares = config.secret_shares;
        drop(config);

        // Generate 32-byte master key using cryptographically secure RNG
        let mut master_key_bytes = vec![0u8; 32];
        (&mut OsRng).fill_bytes(&mut master_key_bytes);

        // Create Shamir configuration
        let shamir_config = ShamirConfig::new(threshold, num_shares)
            .map_err(|_| SealError::InvalidConfiguration)?;

        // Split master key using Shamir Secret Sharing with Feldman VSS
        let (mut shares, commitment) =
            generate_shares_with_commitments(&master_key_bytes, &shamir_config)
                .map_err(|_| SealError::ShamirError)?;

        // Validate shares
        validate_shares(&mut shares).map_err(|_| SealError::ShamirError)?;

        // Store commitment for later verification
        let mut commitment_lock = self.commitment.write().await;
        *commitment_lock = Some(commitment.clone());
        drop(commitment_lock);

        // If storage backend is configured, encrypt and store master key
        if self.storage_backend.is_some() {
            // Derive seal key from first threshold shares (for initial encryption)
            // In production, this would use a separate seal key or HSM
            // For now, we'll use a hash of the master key itself as the seal key
            let mut hasher = Sha256::new();
            hasher.update(&master_key_bytes);
            hasher.update(b"seal-key-derivation");
            let seal_key = hasher.finalize().to_vec();

            // Encrypt master key
            let (encrypted_master_key, metadata) =
                Self::encrypt_master_key(&master_key_bytes, &seal_key)?;

            // Store encrypted master key
            self.store_encrypted_master_key(encrypted_master_key, metadata, &commitment)
                .await?;
        }

        // CRITICAL SECURITY FIX: DO NOT store master key in memory after init
        // DO NOT unseal the vault automatically
        // Vault remains in Sealed state
        // Operators must manually unseahold shares

        // Zeroize master key bytes immediately
        let mut master_key_bytes_mut = master_key_bytes;
        master_key_bytes_mut.zeroize();

        // Vault remains sealed - state is already Sealed, no change needed
        tracing::info!(
            "Vault initialized successfully. Vault remains SEALED. Operators must unseal with {} of {} shares.",
            threshold,
            num_shares
        );

        Ok(shares)
    }

    /// Get seal status
    pub async fn status(&self) -> SealStatus {
        let config = self.config.read().await;
        let state = self.state.read().await;
        let shares = self.unseal_shares.read().await;

        SealStatus::new(&config, state.clone(), shares.len())
    }

    /// Seal the vault - clears master key from memory
    #[instrument(skip(self), fields(operation = "seal_vault"))]
    pub async fn seal(&self) -> Result<(), SealError> {
        let mut state = self.state.write().await;

        if *state == SealState::Sealed {
            return Err(SealError::AlreadySealed);
        }

        *state = SealState::Sealed;

        // Zeroize and clear master key from memory
        let mut master_key = self.master_key.write().await;
        master_key.clear();
        drop(master_key);

        // Clear unseal shares
        let mut shares = self.unseal_shares.write().await;
        shares.clear();

        Ok(())
    }

    /// Provide unseal share - uses Shamir Secret Sharing reconstruction
    /// The share parameter should be a serialized Share from initialization
    #[instrument(skip(self, share_bytes), fields(
        share_length = share_bytes.len(),
        operation = "unseal_with_share"
    ))]
    pub async fn unseal_with_share(&self, share_bytes: &[u8]) -> Result<SealStatus, SealError> {
        let state = self.state.read().await;

        if *state == SealState::Unsealed {
            return Err(SealError::AlreadyUnsealed);
        }
        drop(state);

        // Deserialize and validate share
        let share = Share::from_bytes(share_bytes).map_err(|_| SealError::InvalidUnsealKey)?;

        // Get commitment for verification
        let commitment_lock = self.commitment.read().await;
        let commitment = commitment_lock.as_ref().ok_or(SealError::NotInitialized)?;

        // Verify share against commitment using Feldman VSS
        lib_crypto::shamir::verify_share_with_commitment(&share, commitment)
            .map_err(|_| SealError::ShareVerificationFailed)?;

        drop(commitment_lock);

        let config = self.config.read().await;
        let threshold = config.secret_threshold;
        drop(config);

        // Add share to collection
        let mut shares = self.unseal_shares.write().await;

        // Avoid duplicate shares (check by x coordinate)
        if !shares.iter().any(|s| s.x() == share.x()) {
            shares.push(share);
        }

        // Check if threshold met
        if shares.len() >= threshold {
            // Get commitment again for reconstruction
            let commitment_lock = self.commitment.read().await;
            let commitment = commitment_lock.as_ref().ok_or(SealError::NotInitialized)?;

            // Reconstruct seal key using Shamir with verification
            // Note: In a real implementation, the seal key would be different from master key
            // For now, we reconstruct and use it to decrypt the stored master key
            let reconstructed_seal_key = reconstruct_secret_verified(&shares, commitment)
                .map_err(|_| SealError::ReconstructionFailed)?;

            drop(commitment_lock);

            // Try to load encrypted master key from storage
            let master_key_bytes =
                if let Some(vault_state) = self.load_encrypted_master_key().await? {
                    // Decrypt master key using reconstructed seal key
                    // Use hash of reconstructed key as seal key (same as in initialize)
                    let mut hasher = Sha256::new();
                    hasher.update(&reconstructed_seal_key);
                    hasher.update(b"seal-key-derivation");
                    let seal_key = hasher.finalize().to_vec();

                    Self::decrypt_master_key(
                        &vault_state.encrypted_master_key,
                        &seal_key,
                        &vault_state.encryption_metadata,
                    )?
                } else {
                    // No stored master key, use reconstructed key directly
                    // This happens when storage backend is not configured
                    reconstructed_seal_key
                };

            // Store master key in memory
            let mut master_key = self.master_key.write().await;
            *master_key = MasterKey::new(master_key_bytes);
            drop(master_key);

            // Unseal!
            let mut state = self.state.write().await;
            *state = SealState::Unsealed;

            // Clear shares after successful unseal
            shares.clear();
        } else {
            let mut state = self.state.write().await;
            *state = SealState::Unsealing;
        }

        drop(shares);

        Ok(self.status().await)
    }

    /// Legacy unseal method for backward compatibility
    /// Converts string key to share bytes (assumes base64 encoding)
    pub async fn unseal(&self, key: String) -> Result<SealStatus, SealError> {
        // Decode base64 key to bytes
        use base64::{Engine as _, engine::general_purpose};
        let share_bytes = general_purpose::STANDARD
            .decode(&key)
            .map_err(|_| SealError::InvalidUnsealKey)?;
        self.unseal_with_share(&share_bytes).await
    }

    /// Reset unseal progress
    pub async fn reset_unseal(&self) -> Result<(), SealError> {
        let mut shares = self.unseal_shares.write().await;
        shares.clear();

        let mut state = self.state.write().await;
        if *state == SealState::Unsealing {
            *state = SealState::Sealed;
        }

        Ok(())
    }

    /// Get master key (only available when unsealed)
    pub async fn get_master_key(&self) -> Result<Vec<u8>, SealError> {
        let state = self.state.read().await;
        if *state != SealState::Unsealed {
            return Err(SealError::VaultSealed);
        }
        drop(state);

        let master_key = self.master_key.read().await;
        master_key
            .get()
            .map(|k| k.to_vec())
            .ok_or(SealError::MasterKeyNotAvailable)
    }

    /// Start rekey operation
    #[instrument(skip(self), fields(
        new_shares = %new_shares,
        new_threshold = %new_threshold,
        operation = "start_rekey"
    ))]
    pub async fn start_rekey(
        &self,
        new_shares: usize,
        new_threshold: usize,
    ) -> Result<RekeyOperation, SealError> {
        let rekey = self.rekey_operation.read().await;
        if rekey.is_some() {
            return Err(SealError::RekeyInProgress);
        }
        drop(rekey);

        let config = self.config.read().await;
        let required = config.secret_threshold;
        drop(config);

        let operation = RekeyOperation {
            new_shares,
            new_threshold,
            progress: 0,
            required,
            started_at: Utc::now(),
            nonce: uuid::Uuid::new_v4().to_string(),
        };

        let mut rekey = self.rekey_operation.write().await;
        *rekey = Some(operation.clone());

        Ok(operation)
    }

    /// Provide rekey share
    pub async fn rekey_update(&self, key: String) -> Result<RekeyOperation, SealError> {
        let mut rekey_opt = self.rekey_operation.write().await;
        let rekey = rekey_opt.as_mut().ok_or(SealError::NoRekeyInProgress)?;

        // Validate key
        if key.is_empty() {
            return Err(SealError::InvalidUnsealKey);
        }

        // Increment progress
        rekey.progress += 1;

        // Check if rekey complete
        if rekey.progress >= rekey.required {
            // Apply new configuration
            let new_shares = rekey.new_shares;
            let new_threshold = rekey.new_threshold;

            drop(rekey_opt);

            let mut config = self.config.write().await;
            config.secret_shares = new_shares;
            config.secret_threshold = new_threshold;

            // Clear rekey operation
            let mut rekey_opt = self.rekey_operation.write().await;
            let completed = rekey_opt.take().unwrap();

            Ok(completed)
        } else {
            Ok(rekey.clone())
        }
    }

    /// Cancel rekey operation
    pub async fn cancel_rekey(&self) -> Result<(), SealError> {
        let mut rekey = self.rekey_operation.write().await;

        if rekey.is_none() {
            return Err(SealError::NoRekeyInProgress);
        }

        *rekey = None;
        Ok(())
    }

    /// Get rekey progress
    pub async fn rekey_progress(&self) -> Option<RekeyOperation> {
        let rekey = self.rekey_operation.read().await;
        rekey.clone()
    }

    /// Check if sealed
    pub async fn is_sealed(&self) -> bool {
        let state = self.state.read().await;
        *state == SealState::Sealed || *state == SealState::Unsealing
    }

    /// Check if unsealed
    pub async fn is_unsealed(&self) -> bool {
        let state = self.state.read().await;
        *state == SealState::Unsealed
    }

    /// Rotate master key - generates new master key and re-encrypts with new shares
    /// Returns new shares that must be distributed to operators
    /// Vault must be unsealed to perform rotation
    #[instrument(skip(self), fields(operation = "rotate_master_key"))]
    pub async fn rotate_master_key(&self) -> Result<Vec<Share>, SealError> {
        // Check if unsealed
        let state = self.state.read().await;
        if *state != SealState::Unsealed {
            return Err(SealError::VaultSealed);
        }
        drop(state);

        let config = self.config.read().await;
        let threshold = config.secret_threshold;
        let num_shares = config.secret_shares;
        drop(config);

        // Generate new 32-byte master key
        let mut new_master_key_bytes = vec![0u8; 32];
        (&mut OsRng).fill_bytes(&mut new_master_key_bytes);

        // Create Shamir configuration
        let shamir_config = ShamirConfig::new(threshold, num_shares)
            .map_err(|_| SealError::InvalidConfiguration)?;

        // Split new master key using Shamir Secret Sharing with Feldman VSS
        let (mut shares, commitment) =
            generate_shares_with_commitments(&new_master_key_bytes, &shamir_config)
                .map_err(|_| SealError::ShamirError)?;

        // Validate shares
        validate_shares(&mut shares).map_err(|_| SealError::ShamirError)?;

        // Update commitment
        let mut commitment_lock = self.commitment.write().await;
        *commitment_lock = Some(commitment.clone());
        drop(commitment_lock);

        // If storage backend is configured, encrypt and store new master key
        if self.storage_backend.is_some() {
            // Derive seal key from new master key
            let mut hasher = Sha256::new();
            hasher.update(&new_master_key_bytes);
            hasher.update(b"seal-key-derivation");
            let seal_key = hasher.finalize().to_vec();

            // Encrypt new master key
            let (encrypted_master_key, metadata) =
                Self::encrypt_master_key(&new_master_key_bytes, &seal_key)?;

            // Store encrypted master key (this will update the version)
            self.store_encrypted_master_key(encrypted_master_key, metadata, &commitment)
                .await?;
        }

        // Update master key in memory
        let mut master_key_lock = self.master_key.write().await;
        *master_key_lock = MasterKey::new(new_master_key_bytes);
        drop(master_key_lock);

        Ok(shares)
    }

    /// Load vault state from storage on startup
    /// This allows the vault to resume from a previous state
    pub async fn load_from_storage(&self) -> Result<bool, SealError> {
        if let Some(vault_state) = self.load_encrypted_master_key().await? {
            // Update configuration
            let mut config = self.config.write().await;
            *config = vault_state.seal_config;
            drop(config);

            // Deserialize and store commitment
            let commitment: Commitment = serde_json::from_slice(&vault_state.shamir_commitments)
                .map_err(|e| {
                    SealError::StorageError(format!("Failed to deserialize commitment: {}", e))
                })?;

            let mut commitment_lock = self.commitment.write().await;
            *commitment_lock = Some(commitment);
            drop(commitment_lock);

            // Vault remains sealed, waiting for unseal shares
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize_and_unseal_flow() {
        let config = SealConfig {
            seal_type: "shamir".to_string(),
            secret_shares: 5,
            secret_threshold: 3,
            created_at: Utc::now(),
        };

        // Use storage backend for proper initialization
        let storage = Arc::new(InMemoryVaultStateStorage::new());
        let service = SealService::with_storage(config, storage);

        // Initialize vault - generates master key and shares
        let shares = service.initialize().await.unwrap();
        assert_eq!(shares.len(), 5);

        // CRITICAL SECURITY FIX: After initialization, vault remains SEALED
        // This follows HashiCorp Vault best practices
        assert!(
            service.is_sealed().await,
            "Vault should remain sealed after initialization"
        );

        // Unseal with threshold shares (3 of 5)
        for share in shares.iter().take(3) {
            let share_bytes = share.to_bytes().unwrap();
            service.unseal_with_share(&share_bytes).await.unwrap();
        }

        // Should be unsealed now
        assert!(service.is_unsealed().await);

        // Master key should be available
        let master_key = service.get_master_key().await.unwrap();
        assert_eq!(master_key.len(), 32);
    }

    #[tokio::test]
    async fn test_initialize_with_storage() {
        let config = SealConfig {
            seal_type: "shamir".to_string(),
            secret_shares: 5,
            secret_threshold: 3,
            created_at: Utc::now(),
        };

        let storage = Arc::new(InMemoryVaultStateStorage::new());
        let service = SealService::with_storage(config, storage.clone());

        // Initialize vault - generates master key and shares
        let shares = service.initialize().await.unwrap();
        assert_eq!(shares.len(), 5);

        // Verify vault state was stored
        let vault_state = storage.load_vault_state().await.unwrap();
        assert!(vault_state.is_some());
        let state = vault_state.unwrap();
        assert_eq!(state.seal_config.secret_shares, 5);
        assert_eq!(state.seal_config.secret_threshold, 3);
        assert!(!state.encrypted_master_key.is_empty());

        // CRITICAL SECURITY FIX: After initialization, vault remains SEALED
        assert!(
            service.is_sealed().await,
            "Vault should remain sealed after initialization"
        );

        // Unseal to get master key
        for share in shares.iter().take(3) {
            let share_bytes = share.to_bytes().unwrap();
            service.unseal_with_share(&share_bytes).await.unwrap();
        }
        assert!(service.is_unsealed().await);

        // Get original master key
        let original_master_key = service.get_master_key().await.unwrap();

        // Seal the vault again
        service.seal().await.unwrap();
        assert!(service.is_sealed().await);

        // Unseal with threshold shares (3 of 5)
        for share in shares.iter().take(3) {
            let share_bytes = share.to_bytes().unwrap();
            service.unseal_with_share(&share_bytes).await.unwrap();
        }

        // Should be unsealed now
        assert!(service.is_unsealed().await);

        // Master key should be the same as original
        let recovered_master_key = service.get_master_key().await.unwrap();
        assert_eq!(original_master_key, recovered_master_key);
    }

    #[tokio::test]
    async fn test_master_key_rotation() {
        let config = SealConfig {
            seal_type: "shamir".to_string(),
            secret_shares: 5,
            secret_threshold: 3,
            created_at: Utc::now(),
        };

        let storage = Arc::new(InMemoryVaultStateStorage::new());
        let service = SealService::with_storage(config, storage.clone());

        // Initialize vault
        let initial_shares = service.initialize().await.unwrap();

        // CRITICAL SECURITY FIX: Vault remains sealed after init, must unseal first
        assert!(service.is_sealed().await);
        for share in initial_shares.iter().take(3) {
            let share_bytes = share.to_bytes().unwrap();
            service.unseal_with_share(&share_bytes).await.unwrap();
        }
        assert!(service.is_unsealed().await);

        let original_master_key = service.get_master_key().await.unwrap();

        // Rotate master key
        let new_shares = service.rotate_master_key().await.unwrap();
        assert_eq!(new_shares.len(), 5);

        // Master key should be different
        let rotated_master_key = service.get_master_key().await.unwrap();
        assert_ne!(original_master_key, rotated_master_key);

        // Seal and unseal with new shares
        service.seal().await.unwrap();

        for share in new_shares.iter().take(3) {
            let share_bytes = share.to_bytes().unwrap();
            service.unseal_with_share(&share_bytes).await.unwrap();
        }

        assert!(service.is_unsealed().await);
        let recovered_key = service.get_master_key().await.unwrap();
        assert_eq!(rotated_master_key, recovered_key);
    }

    #[tokio::test]
    async fn test_load_from_storage() {
        let config = SealConfig {
            seal_type: "shamir".to_string(),
            secret_shares: 5,
            secret_threshold: 3,
            created_at: Utc::now(),
        };

        let storage = Arc::new(InMemoryVaultStateStorage::new());

        // Initialize first service
        let service1 = SealService::with_storage(config.clone(), storage.clone());
        let shares = service1.initialize().await.unwrap();

        // CRITICAL SECURITY FIX: Vault remains sealed after init, must unseal to get master key
        assert!(service1.is_sealed().await);
        for share in shares.iter().take(3) {
            let share_bytes = share.to_bytes().unwrap();
            service1.unseal_with_share(&share_bytes).await.unwrap();
        }
        assert!(service1.is_unsealed().await);

        let original_master_key = service1.get_master_key().await.unwrap();
        service1.seal().await.unwrap();

        // Create second service and load from storage
        let service2 = SealService::with_storage(config, storage);
        let loaded = service2.load_from_storage().await.unwrap();
        assert!(loaded);

        // Should be sealed
        assert!(service2.is_sealed().await);

        // Unseal with shares
        for share in shares.iter().take(3) {
            let share_bytes = share.to_bytes().unwrap();
            service2.unseal_with_share(&share_bytes).await.unwrap();
        }

        // Should recover same master key
        let recovered_master_key = service2.get_master_key().await.unwrap();
        assert_eq!(original_master_key, recovered_master_key);
    }

    #[tokio::test]
    async fn test_encryption_decryption() {
        let master_key = vec![1u8; 32];
        let seal_key = vec![2u8; 32];

        // Encrypt
        let (ciphertext, metadata) =
            SealService::encrypt_master_key(&master_key, &seal_key).unwrap();
        assert!(!ciphertext.is_empty());
        assert_eq!(metadata.algorithm, "aes-256-gcm");
        assert_eq!(metadata.kdf, "argon2id");

        // Decrypt
        let decrypted = SealService::decrypt_master_key(&ciphertext, &seal_key, &metadata).unwrap();
        assert_eq!(master_key, decrypted);
    }

    #[tokio::test]
    async fn test_decryption_with_wrong_key_fails() {
        let master_key = vec![1u8; 32];
        let seal_key = vec![2u8; 32];
        let wrong_key = vec![3u8; 32];

        // Encrypt with correct key
        let (ciphertext, metadata) =
            SealService::encrypt_master_key(&master_key, &seal_key).unwrap();

        // Try to decrypt with wrong key
        let result = SealService::decrypt_master_key(&ciphertext, &wrong_key, &metadata);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_seal_clears_master_key() {
        // Use storage backend for proper initialization
        let storage = Arc::new(InMemoryVaultStateStorage::new());
        let service = SealService::with_storage(SealConfig::default(), storage);

        // Initialize
        let shares = service.initialize().await.unwrap();

        // CRITICAL SECURITY FIX: Vault remains sealed after init
        assert!(service.is_sealed().await);

        // Master key should NOT be available when sealed
        assert!(service.get_master_key().await.is_err());

        // Unseal first
        for share in shares.iter().take(3) {
            let share_bytes = share.to_bytes().unwrap();
            service.unseal_with_share(&share_bytes).await.unwrap();
        }
        assert!(service.is_unsealed().await);

        // Master key should be available after unseal
        assert!(service.get_master_key().await.is_ok());

        // Seal
        service.seal().await.unwrap();
        assert!(service.is_sealed().await);

        // Master key should not be available
        assert!(service.get_master_key().await.is_err());
    }

    #[tokio::test]
    async fn test_share_verification() {
        let storage = Arc::new(InMemoryVaultStateStorage::new());
        let service = SealService::with_storage(SealConfig::default(), storage);

        // Initialize
        let shares = service.initialize().await.unwrap();

        // Vault is already sealed after initialization (CRITICAL SECURITY FIX)
        assert!(service.is_sealed().await);

        // Try to unseal with invalid share
        let invalid_share = vec![0u8; 100];
        let result = service.unseal_with_share(&invalid_share).await;
        assert!(result.is_err());

        // Valid shares should work
        for share in shares.iter().take(3) {
            let share_bytes = share.to_bytes().unwrap();
            service.unseal_with_share(&share_bytes).await.unwrap();
        }

        assert!(service.is_unsealed().await);
    }

    #[tokio::test]
    async fn test_insufficient_shares() {
        let storage = Arc::new(InMemoryVaultStateStorage::new());
        let service = SealService::with_storage(SealConfig::default(), storage);

        // Initialize
        let shares = service.initialize().await.unwrap();

        // Vault is already sealed after initialization (CRITICAL SECURITY FIX)
        assert!(service.is_sealed().await);

        // Provide only 2 shares (need 3)
        for share in shares.iter().take(2) {
            let share_bytes = share.to_bytes().unwrap();
            service.unseal_with_share(&share_bytes).await.unwrap();
        }

        // Should still be sealed
        assert!(service.is_sealed().await);

        let status = service.status().await;
        assert_eq!(status.progress, 2);
        assert_eq!(status.state, SealState::Unsealing);
    }

    #[tokio::test]
    async fn test_reset_unseal() {
        let storage = Arc::new(InMemoryVaultStateStorage::new());
        let service = SealService::with_storage(SealConfig::default(), storage);

        // Initialize
        let shares = service.initialize().await.unwrap();

        // Vault is already sealed after initialization (CRITICAL SECURITY FIX)
        assert!(service.is_sealed().await);

        // Provide 2 shares
        for share in shares.iter().take(2) {
            let share_bytes = share.to_bytes().unwrap();
            service.unseal_with_share(&share_bytes).await.unwrap();
        }

        let status = service.status().await;
        assert_eq!(status.progress, 2);

        // Reset
        service.reset_unseal().await.unwrap();

        let status = service.status().await;
        assert_eq!(status.progress, 0);
        assert_eq!(status.state, SealState::Sealed);
    }

    #[tokio::test]
    async fn test_duplicate_shares_ignored() {
        let storage = Arc::new(InMemoryVaultStateStorage::new());
        let service = SealService::with_storage(SealConfig::default(), storage);

        // Initialize
        let shares = service.initialize().await.unwrap();

        // Vault is already sealed after initialization (CRITICAL SECURITY FIX)
        assert!(service.is_sealed().await);

        // Provide same share twice
        let share_bytes = shares[0].to_bytes().unwrap();
        service.unseal_with_share(&share_bytes).await.unwrap();
        service.unseal_with_share(&share_bytes).await.unwrap();

        let status = service.status().await;
        assert_eq!(status.progress, 1); // Should only count once
    }

    #[tokio::test]
    async fn test_any_threshold_combination_works() {
        let storage = Arc::new(InMemoryVaultStateStorage::new());
        let service = SealService::with_storage(SealConfig::default(), storage);

        // Initialize
        let shares = service.initialize().await.unwrap();

        // Vault is already sealed after initialization (CRITICAL SECURITY FIX)
        assert!(service.is_sealed().await);

        // Test different combinations of 3 shares
        let combinations = vec![vec![0, 1, 2], vec![0, 2, 4], vec![1, 3, 4]];

        for combo in combinations {
            // Seal if not already sealed (first iteration is already sealed)
            if service.is_unsealed().await {
                service.seal().await.unwrap();
            }

            // Unseal with this combination
            for &idx in &combo {
                let share_bytes = shares[idx].to_bytes().unwrap();
                service.unseal_with_share(&share_bytes).await.unwrap();
            }

            // Should be unsealed
            assert!(service.is_unsealed().await);
        }
    }
}
