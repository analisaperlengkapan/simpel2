//! Local encrypted storage fallback for MFA secrets
//!
//! This module provides a fallback mechanism for storing MFA secrets locally
//! when Secreton is unavailable. It uses AES-256-GCM encryption to protect
//! secrets at rest and implements automatic synchronization when Secreton recovers.

use crate::crypto::aes_gcm::{AesGcmService, EncryptedData};
use crate::error::{AuthencError, Result};
use crate::secreton_client::secreton_client::MfaSetupData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// MFA secret stored locally with encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMfaSecret {
    /// User identifier
    pub user_id: String,
    /// Encrypted TOTP secret
    pub encrypted_secret: EncryptedData,
    /// Encrypted backup codes
    pub encrypted_backup_codes: Vec<EncryptedData>,
    /// When the secret was created
    pub created_at: DateTime<Utc>,
    /// When the secret was last used
    pub last_used: Option<DateTime<Utc>>,
    /// Whether this secret needs to be synced to Secreton
    pub needs_sync: bool,
    /// Number of sync attempts
    pub sync_attempts: u32,
}

/// Local MFA storage metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorageMetadata {
    /// Total secrets stored
    pub total_secrets: usize,
    /// Secrets pending sync
    pub pending_sync: usize,
    /// Last sync attempt
    pub last_sync_attempt: Option<DateTime<Utc>>,
    /// Last successful sync
    pub last_successful_sync: Option<DateTime<Utc>>,
    /// Storage version
    pub version: u32,
}

/// Degraded mode status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradedMode {
    /// Normal operation - Secreton available
    Normal,
    /// Degraded - using local storage
    Degraded,
    /// Recovering - attempting to sync
    Recovering,
}

/// Local encrypted storage for MFA secrets
pub struct MfaLocalStorage {
    /// AES-GCM encryption service
    encryption: Arc<AesGcmService>,
    /// In-memory cache of secrets
    secrets: Arc<RwLock<HashMap<String, LocalMfaSecret>>>,
    /// Storage file path
    storage_path: PathBuf,
    /// Current degraded mode status
    degraded_mode: Arc<RwLock<DegradedMode>>,
    /// Metrics for monitoring
    metrics: Arc<RwLock<LocalStorageMetrics>>,
}

/// Metrics for local storage usage
#[derive(Debug, Clone, Default)]
pub struct LocalStorageMetrics {
    /// Number of secrets stored locally
    pub secrets_stored: u64,
    /// Number of secrets retrieved from local storage
    pub secrets_retrieved: u64,
    /// Number of successful syncs to Secreton
    pub successful_syncs: u64,
    /// Number of failed syncs to Secreton
    pub failed_syncs: u64,
    /// Time spent in degraded mode (seconds)
    pub degraded_mode_duration: u64,
    /// Last degraded mode entry time
    pub last_degraded_entry: Option<DateTime<Utc>>,
}

impl MfaLocalStorage {
    /// Create a new local storage instance
    ///
    /// # Arguments
    /// * `storage_path` - Path to the encrypted storage file
    /// * `encryption_key` - 32-byte encryption key for AES-256-GCM
    pub fn new(storage_path: PathBuf, encryption_key: &[u8; 32]) -> Result<Self> {
        let encryption = Arc::new(AesGcmService::with_key(encryption_key)?);

        Ok(Self {
            encryption,
            secrets: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
            degraded_mode: Arc::new(RwLock::new(DegradedMode::Normal)),
            metrics: Arc::new(RwLock::new(LocalStorageMetrics::default())),
        })
    }

    /// Initialize storage by loading existing secrets from disk
    pub async fn initialize(&self) -> Result<()> {
        if self.storage_path.exists() {
            match self.load_from_disk().await {
                Ok(count) => {
                    info!(
                        "Loaded {} MFA secrets from local storage at {:?}",
                        count, self.storage_path
                    );
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to load local MFA storage: {}", e);
                    warn!("Starting with empty local storage");
                    Ok(())
                }
            }
        } else {
            info!("No existing local storage found, starting fresh");
            Ok(())
        }
    }

    /// Store MFA setup data locally
    pub async fn store_mfa_setup(&self, user_id: &str, setup_data: &MfaSetupData) -> Result<()> {
        // Encrypt the secret
        let encrypted_secret = self.encryption.encrypt(setup_data.secret_key.as_bytes())?;

        // Encrypt backup codes
        let mut encrypted_backup_codes = Vec::new();
        for code in &setup_data.backup_codes {
            let encrypted_code = self.encryption.encrypt(code.as_bytes())?;
            encrypted_backup_codes.push(encrypted_code);
        }

        let local_secret = LocalMfaSecret {
            user_id: user_id.to_string(),
            encrypted_secret,
            encrypted_backup_codes,
            created_at: Utc::now(),
            last_used: None,
            needs_sync: true,
            sync_attempts: 0,
        };

        // Store in memory
        self.secrets
            .write()
            .await
            .insert(user_id.to_string(), local_secret);

        // Persist to disk
        self.save_to_disk().await?;

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.secrets_stored += 1;

        info!("Stored MFA secret locally for user: {}", user_id);

        Ok(())
    }

    /// Retrieve MFA secret from local storage
    pub async fn get_mfa_secret(&self, user_id: &str) -> Result<Option<String>> {
        let secrets = self.secrets.read().await;

        if let Some(local_secret) = secrets.get(user_id) {
            // Decrypt the secret
            let decrypted = self.encryption.decrypt(&local_secret.encrypted_secret)?;
            let secret = String::from_utf8(decrypted)
                .map_err(|_| AuthencError::internal("Invalid UTF-8 in decrypted MFA secret"))?;

            // Update metrics
            let mut metrics = self.metrics.write().await;
            metrics.secrets_retrieved += 1;

            Ok(Some(secret))
        } else {
            Ok(None)
        }
    }

    /// Retrieve backup codes from local storage
    pub async fn get_backup_codes(&self, user_id: &str) -> Result<Option<Vec<String>>> {
        let secrets = self.secrets.read().await;

        if let Some(local_secret) = secrets.get(user_id) {
            let mut backup_codes = Vec::new();

            for encrypted_code in &local_secret.encrypted_backup_codes {
                let decrypted = self.encryption.decrypt(encrypted_code)?;
                let code = String::from_utf8(decrypted).map_err(|_| {
                    AuthencError::internal("Invalid UTF-8 in decrypted backup code")
                })?;
                backup_codes.push(code);
            }

            Ok(Some(backup_codes))
        } else {
            Ok(None)
        }
    }

    /// Update last used timestamp for a secret
    pub async fn update_last_used(&self, user_id: &str) -> Result<()> {
        let mut secrets = self.secrets.write().await;

        if let Some(secret) = secrets.get_mut(user_id) {
            secret.last_used = Some(Utc::now());
            drop(secrets);
            self.save_to_disk().await?;
        }

        Ok(())
    }

    /// Remove MFA secret from local storage
    pub async fn remove_mfa_secret(&self, user_id: &str) -> Result<()> {
        self.secrets.write().await.remove(user_id);
        self.save_to_disk().await?;

        info!(
            "Removed MFA secret from local storage for user: {}",
            user_id
        );

        Ok(())
    }

    /// Mark secret as needing sync
    pub async fn mark_for_sync(&self, user_id: &str) -> Result<()> {
        let mut secrets = self.secrets.write().await;

        if let Some(secret) = secrets.get_mut(user_id) {
            secret.needs_sync = true;
            secret.sync_attempts = 0;
        }

        Ok(())
    }

    /// Get list of secrets that need syncing
    pub async fn get_secrets_needing_sync(&self) -> Vec<String> {
        let secrets = self.secrets.read().await;

        secrets
            .values()
            .filter(|s| s.needs_sync && s.sync_attempts < 10)
            .map(|s| s.user_id.clone())
            .collect()
    }

    /// Mark secret as synced
    pub async fn mark_as_synced(&self, user_id: &str) -> Result<()> {
        let mut secrets = self.secrets.write().await;

        if let Some(secret) = secrets.get_mut(user_id) {
            secret.needs_sync = false;
            secret.sync_attempts = 0;
            drop(secrets);
            self.save_to_disk().await?;

            info!("Marked MFA secret as synced for user: {}", user_id);
        }

        Ok(())
    }

    /// Increment sync attempt counter
    pub async fn increment_sync_attempts(&self, user_id: &str) -> Result<()> {
        let mut secrets = self.secrets.write().await;

        if let Some(secret) = secrets.get_mut(user_id) {
            secret.sync_attempts += 1;
        }

        Ok(())
    }

    /// Enter degraded mode
    pub async fn enter_degraded_mode(&self) {
        let mut mode = self.degraded_mode.write().await;
        if *mode == DegradedMode::Normal {
            *mode = DegradedMode::Degraded;

            let mut metrics = self.metrics.write().await;
            metrics.last_degraded_entry = Some(Utc::now());

            warn!("Entered degraded mode - using local encrypted storage for MFA secrets");
        }
    }

    /// Enter recovering mode
    pub async fn enter_recovering_mode(&self) {
        *self.degraded_mode.write().await = DegradedMode::Recovering;
        info!("Entering recovering mode - attempting to sync MFA secrets to Secreton");
    }

    /// Exit degraded mode
    pub async fn exit_degraded_mode(&self) {
        let mut mode = self.degraded_mode.write().await;
        let previous_mode = *mode;
        *mode = DegradedMode::Normal;

        if previous_mode != DegradedMode::Normal {
            // Update metrics
            let mut metrics = self.metrics.write().await;
            if let Some(entry_time) = metrics.last_degraded_entry {
                let duration = (Utc::now() - entry_time).num_seconds() as u64;
                metrics.degraded_mode_duration += duration;
            }

            info!("Exited degraded mode - Secreton is now available");
        }
    }

    /// Get current degraded mode status
    pub async fn get_degraded_mode(&self) -> DegradedMode {
        *self.degraded_mode.read().await
    }

    /// Check if in degraded mode
    pub async fn is_degraded(&self) -> bool {
        *self.degraded_mode.read().await != DegradedMode::Normal
    }

    /// Get storage metadata
    pub async fn get_metadata(&self) -> LocalStorageMetadata {
        let secrets = self.secrets.read().await;
        let pending_sync = secrets.values().filter(|s| s.needs_sync).count();

        LocalStorageMetadata {
            total_secrets: secrets.len(),
            pending_sync,
            last_sync_attempt: None, // Will be updated by sync process
            last_successful_sync: None,
            version: 1,
        }
    }

    /// Get metrics for monitoring
    pub async fn get_metrics(&self) -> LocalStorageMetrics {
        self.metrics.read().await.clone()
    }

    /// Save secrets to disk
    async fn save_to_disk(&self) -> Result<()> {
        let secrets = self.secrets.read().await;

        // Serialize secrets
        let secrets_vec: Vec<LocalMfaSecret> = secrets.values().cloned().collect();
        let json_data = serde_json::to_vec(&secrets_vec)
            .map_err(|e| AuthencError::internal(format!("Failed to serialize secrets: {}", e)))?;

        // Encrypt the entire storage file
        let encrypted_storage = self.encryption.encrypt(&json_data)?;
        let encrypted_json = serde_json::to_vec(&encrypted_storage).map_err(|e| {
            AuthencError::internal(format!("Failed to serialize encrypted storage: {}", e))
        })?;

        // Write to disk
        fs::write(&self.storage_path, encrypted_json)
            .await
            .map_err(|e| AuthencError::internal(format!("Failed to write storage file: {}", e)))?;

        Ok(())
    }

    /// Load secrets from disk
    async fn load_from_disk(&self) -> Result<usize> {
        // Read from disk
        let encrypted_json = fs::read(&self.storage_path)
            .await
            .map_err(|e| AuthencError::internal(format!("Failed to read storage file: {}", e)))?;

        // Deserialize encrypted storage
        let encrypted_storage: EncryptedData =
            serde_json::from_slice(&encrypted_json).map_err(|e| {
                AuthencError::internal(format!("Failed to deserialize encrypted storage: {}", e))
            })?;

        // Decrypt the storage
        let json_data = self.encryption.decrypt(&encrypted_storage)?;

        // Deserialize secrets
        let secrets_vec: Vec<LocalMfaSecret> = serde_json::from_slice(&json_data)
            .map_err(|e| AuthencError::internal(format!("Failed to deserialize secrets: {}", e)))?;

        // Load into memory
        let mut secrets = self.secrets.write().await;
        secrets.clear();

        for secret in secrets_vec {
            secrets.insert(secret.user_id.clone(), secret);
        }

        let count = secrets.len();
        Ok(count)
    }

    /// Clear all local storage (for testing or emergency)
    pub async fn clear_all(&self) -> Result<()> {
        self.secrets.write().await.clear();

        if self.storage_path.exists() {
            fs::remove_file(&self.storage_path).await.map_err(|e| {
                AuthencError::internal(format!("Failed to remove storage file: {}", e))
            })?;
        }

        warn!("Cleared all local MFA storage");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_local_storage_store_and_retrieve() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("mfa_storage.enc");
        let key = AesGcmService::generate_key();

        let storage = MfaLocalStorage::new(storage_path, &key).unwrap();
        storage.initialize().await.unwrap();

        let setup_data = MfaSetupData {
            qr_code_url: "otpauth://totp/test".to_string(),
            secret_key: "JBSWY3DPEHPK3PXP".to_string(),
            backup_codes: vec!["123456".to_string(), "789012".to_string()],
        };

        // Store
        storage
            .store_mfa_setup("user123", &setup_data)
            .await
            .unwrap();

        // Retrieve
        let secret = storage.get_mfa_secret("user123").await.unwrap();
        assert_eq!(secret, Some("JBSWY3DPEHPK3PXP".to_string()));

        let codes = storage.get_backup_codes("user123").await.unwrap();
        assert_eq!(
            codes,
            Some(vec!["123456".to_string(), "789012".to_string()])
        );
    }

    #[tokio::test]
    async fn test_degraded_mode() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("mfa_storage.enc");
        let key = AesGcmService::generate_key();

        let storage = MfaLocalStorage::new(storage_path, &key).unwrap();

        assert_eq!(storage.get_degraded_mode().await, DegradedMode::Normal);
        assert!(!storage.is_degraded().await);

        storage.enter_degraded_mode().await;
        assert_eq!(storage.get_degraded_mode().await, DegradedMode::Degraded);
        assert!(storage.is_degraded().await);

        storage.exit_degraded_mode().await;
        assert_eq!(storage.get_degraded_mode().await, DegradedMode::Normal);
        assert!(!storage.is_degraded().await);
    }

    #[tokio::test]
    async fn test_persistence() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("mfa_storage.enc");
        let key = AesGcmService::generate_key();

        // Create and store
        {
            let storage = MfaLocalStorage::new(storage_path.clone(), &key).unwrap();
            storage.initialize().await.unwrap();

            let setup_data = MfaSetupData {
                qr_code_url: "otpauth://totp/test".to_string(),
                secret_key: "JBSWY3DPEHPK3PXP".to_string(),
                backup_codes: vec!["123456".to_string()],
            };

            storage
                .store_mfa_setup("user123", &setup_data)
                .await
                .unwrap();
        }

        // Load from disk
        {
            let storage = MfaLocalStorage::new(storage_path, &key).unwrap();
            storage.initialize().await.unwrap();

            let secret = storage.get_mfa_secret("user123").await.unwrap();
            assert_eq!(secret, Some("JBSWY3DPEHPK3PXP".to_string()));
        }
    }

    #[tokio::test]
    async fn test_sync_tracking() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("mfa_storage.enc");
        let key = AesGcmService::generate_key();

        let storage = MfaLocalStorage::new(storage_path, &key).unwrap();
        storage.initialize().await.unwrap();

        let setup_data = MfaSetupData {
            qr_code_url: "otpauth://totp/test".to_string(),
            secret_key: "JBSWY3DPEHPK3PXP".to_string(),
            backup_codes: vec!["123456".to_string()],
        };

        storage
            .store_mfa_setup("user123", &setup_data)
            .await
            .unwrap();

        // Should need sync
        let needs_sync = storage.get_secrets_needing_sync().await;
        assert_eq!(needs_sync, vec!["user123"]);

        // Mark as synced
        storage.mark_as_synced("user123").await.unwrap();

        let needs_sync = storage.get_secrets_needing_sync().await;
        assert!(needs_sync.is_empty());
    }
}
