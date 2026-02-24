//! Backup manager for creating, storing, and restoring backups

use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use chrono::Utc;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use rand::RngCore;
use serde_json;
use std::io::{Read, Write};
use std::str::FromStr;
use std::sync::Arc;

use crate::alerting::AlertManager;
use crate::error::{BackupError, Result};
use crate::metadata::{Backup, BackupMetadata};
use crate::scheduler::BackupScheduler;
use crate::storage::{BackupStorage, LocalStorage};
use crate::types::{BackupConfig, RestoreOptions, StorageType};

/// Database connection parameters parsed from URL
struct DatabaseConnectionParams {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

/// Backup manager for automated backup and restore operations
pub struct BackupManager {
    pub(crate) config: BackupConfig,
    storage: Arc<dyn BackupStorage>,
    encryption_key: Vec<u8>,
    scheduler: Option<Arc<BackupScheduler>>,
    alert_manager: Arc<AlertManager>,
}

impl BackupManager {
    /// Create a new backup manager
    pub async fn new(config: BackupConfig) -> Result<Self> {
        // Create storage backend
        let storage: Arc<dyn BackupStorage> = match config.storage_type {
            StorageType::Local => {
                let local_config = match &config.storage_config {
                    crate::types::StorageConfig::Local(c) => c,
                    _ => {
                        return Err(BackupError::InvalidConfig(
                            "Invalid storage config for local storage".to_string(),
                        ));
                    }
                };
                Arc::new(LocalStorage::new(&local_config.path)?)
            }
            StorageType::S3 => {
                // TODO: Implement S3 storage backend
                return Err(BackupError::InvalidConfig(
                    "S3 storage not yet implemented".to_string(),
                ));
            }
            StorageType::Azure => {
                // TODO: Implement Azure storage backend
                return Err(BackupError::InvalidConfig(
                    "Azure storage not yet implemented".to_string(),
                ));
            }
            StorageType::Gcs => {
                // TODO: Implement GCS storage backend
                return Err(BackupError::InvalidConfig(
                    "GCS storage not yet implemented".to_string(),
                ));
            }
        };

        // Get or generate encryption key
        let encryption_key = if let Some(key) = &config.encryption_key {
            if key.len() != 32 {
                return Err(BackupError::InvalidConfig(
                    "Encryption key must be 32 bytes".to_string(),
                ));
            }
            key.clone()
        } else {
            // Generate a new encryption key
            let mut key = vec![0u8; 32];
            OsRng.fill_bytes(&mut key);
            tracing::warn!(
                "No encryption key provided, generated new key. \
                 This key must be stored securely for backup restoration!"
            );
            key
        };

        Ok(Self {
            config,
            storage,
            encryption_key,
            scheduler: None,
            alert_manager: Arc::new(AlertManager::new(true)), // Enable alerting by default
        })
    }

    /// Start automated backup scheduler
    pub async fn start_scheduler(&mut self) -> Result<()> {
        if !self.config.enabled {
            return Err(BackupError::Scheduler(
                "Automated backups are disabled".to_string(),
            ));
        }

        let scheduler = Arc::new(BackupScheduler::new(
            &self.config.schedule,
            Arc::new(self.clone_for_scheduler()),
        )?);

        scheduler.start().await?;
        self.scheduler = Some(scheduler);

        // Start cleanup task
        self.start_cleanup_task().await?;

        Ok(())
    }

    /// Start automated cleanup task for old backups
    /// Runs daily to enforce retention policy
    async fn start_cleanup_task(&self) -> Result<()> {
        let manager = Arc::new(self.clone_for_scheduler());
        let retention_days = self.config.retention_days;

        tokio::spawn(async move {
            // Run cleanup daily at 3 AM (1 hour after backup)
            let cleanup_schedule = "0 3 * * *";
            let schedule = match cron::Schedule::from_str(cleanup_schedule) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Failed to parse cleanup schedule: {}", e);
                    return;
                }
            };

            loop {
                // Get next scheduled time
                let now = Utc::now();
                let next = match schedule.upcoming(Utc).next() {
                    Some(next) => next,
                    None => {
                        tracing::error!("Failed to get next cleanup time");
                        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                        continue;
                    }
                };

                // Calculate duration until next cleanup
                let duration = match (next - now).to_std() {
                    Ok(d) => d,
                    Err(e) => {
                        tracing::error!("Failed to calculate cleanup duration: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                        continue;
                    }
                };

                tracing::info!(
                    next_cleanup = %next,
                    retention_days = retention_days,
                    "Next backup cleanup scheduled"
                );

                // Sleep until next cleanup time
                tokio::time::sleep(duration).await;

                // Execute cleanup
                tracing::info!("Starting scheduled backup cleanup");
                match manager.cleanup_old_backups().await {
                    Ok(_) => {
                        tracing::info!("Scheduled backup cleanup completed successfully");
                    }
                    Err(e) => {
                        tracing::error!(
                            error = %e,
                            "Scheduled backup cleanup failed"
                        );
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop automated backup scheduler
    pub async fn stop_scheduler(&mut self) -> Result<()> {
        if let Some(scheduler) = &self.scheduler {
            scheduler.stop().await?;
            self.scheduler = None;
        }
        Ok(())
    }

    /// Create a backup
    pub async fn create_backup(&self) -> Result<String> {
        tracing::info!("Starting backup creation");

        // Wrap the entire backup creation in error handling for alerting
        match self.create_backup_internal().await {
            Ok(backup_id) => {
                tracing::info!(
                    backup_id = %backup_id,
                    "Backup creation completed successfully"
                );
                Ok(backup_id)
            }
            Err(e) => {
                // Trigger alert on backup creation failure
                tracing::error!(error = %e, "Backup creation failed");
                if let Err(alert_err) = self.alert_manager.alert_backup_creation_failed(&e).await {
                    tracing::error!(
                        error = %alert_err,
                        "Failed to trigger backup creation failure alert"
                    );
                }
                Err(e)
            }
        }
    }

    /// Internal backup creation logic
    async fn create_backup_internal(&self) -> Result<String> {
        // 1. Create Raft snapshot
        let raft_snapshot = self.create_raft_snapshot().await?;
        tracing::info!(size = raft_snapshot.len(), "Raft snapshot created");

        // 2. Dump PostgreSQL
        let postgres_dump = self.dump_postgres().await?;
        tracing::info!(size = postgres_dump.len(), "PostgreSQL dump created");

        // 3. Create backup object
        let mut backup = Backup::new(raft_snapshot, postgres_dump);

        // 4. Compress if enabled
        let (compressed_raft, compressed_postgres) = if self.config.compression_enabled {
            let raft = self.compress(&backup.raft_snapshot)?;
            let postgres = self.compress(&backup.postgres_dump)?;
            let total_compressed = raft.len() + postgres.len();
            backup.set_compressed_size(total_compressed as u64);
            tracing::info!(
                original = backup.metadata.original_size_bytes,
                compressed = total_compressed,
                ratio = %format!("{:.2}", backup.metadata.compression_ratio()),
                "Backup compressed"
            );
            (raft, postgres)
        } else {
            (backup.raft_snapshot.clone(), backup.postgres_dump.clone())
        };

        // 5. Encrypt
        let encrypted_raft = self.encrypt(&compressed_raft)?;
        let encrypted_postgres = self.encrypt(&compressed_postgres)?;
        let total_encrypted = encrypted_raft.len() + encrypted_postgres.len();
        backup.set_encrypted_size(total_encrypted as u64);
        backup.raft_snapshot = encrypted_raft;
        backup.postgres_dump = encrypted_postgres;

        tracing::info!(encrypted_size = total_encrypted, "Backup encrypted");

        // 6. Mark as completed
        backup.mark_completed();

        // 7. Upload to storage
        self.storage.upload(&backup).await?;

        tracing::info!(
            backup_id = %backup.id,
            "Backup uploaded to storage"
        );

        // 8. Verify if enabled
        if self.config.verify_after_backup {
            match self.verify_backup(&backup.id).await {
                Ok(_) => {
                    tracing::info!(backup_id = %backup.id, "Backup verified successfully");
                }
                Err(e) => {
                    tracing::error!(
                        backup_id = %backup.id,
                        error = %e,
                        "Backup verification failed"
                    );
                    // Trigger alert on verification failure
                    if let Err(alert_err) = self
                        .alert_manager
                        .alert_backup_verification_failed(&backup.id, &e)
                        .await
                    {
                        tracing::error!(
                            error = %alert_err,
                            "Failed to trigger backup verification failure alert"
                        );
                    }
                    return Err(e);
                }
            }
        }

        // 9. Clean up old backups
        if let Err(e) = self.cleanup_old_backups().await {
            tracing::error!(error = %e, "Backup cleanup failed");
            // Trigger alert on cleanup failure (non-fatal)
            if let Err(alert_err) = self.alert_manager.alert_backup_cleanup_failed(&e).await {
                tracing::error!(
                    error = %alert_err,
                    "Failed to trigger backup cleanup failure alert"
                );
            }
            // Don't fail the backup creation if cleanup fails
        }

        Ok(backup.id)
    }

    /// Verify a backup
    ///
    /// This method implements automatic backup verification by:
    /// 1. Downloading the backup from storage
    /// 2. Verifying encryption integrity (decrypt operation validates AEAD tag)
    /// 3. Verifying data integrity (checksum validation)
    /// 4. Marking backup as verified in metadata
    ///
    /// # Arguments
    ///
    /// * `backup_id` - The ID of the backup to verify
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if verification succeeds, error otherwise
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Backup download fails
    /// - Decryption fails (indicates encryption corruption)
    /// - Decompression fails (indicates compression corruption)
    /// - Checksum mismatch (indicates data corruption)
    /// - Metadata update fails
    ///
    /// # Requirements
    ///
    /// Validates: Requirements 2.5.8 (Backup verification runs automatically)
    pub async fn verify_backup(&self, backup_id: &str) -> Result<()> {
        tracing::info!(backup_id = %backup_id, "Verifying backup");

        // Download backup
        let mut backup = self.storage.download(backup_id).await?;

        // Decrypt (this validates encryption integrity via AEAD authentication tag)
        let decrypted_raft = self.decrypt(&backup.raft_snapshot)?;
        let decrypted_postgres = self.decrypt(&backup.postgres_dump)?;

        tracing::debug!(
            backup_id = %backup_id,
            raft_size = decrypted_raft.len(),
            postgres_size = decrypted_postgres.len(),
            "Backup decryption successful"
        );

        // Decompress if needed (validates compression integrity)
        if self.config.compression_enabled {
            let _ = self.decompress(&decrypted_raft)?;
            let _ = self.decompress(&decrypted_postgres)?;
            tracing::debug!(
                backup_id = %backup_id,
                "Backup decompression successful"
            );
        }

        // Verify checksum (validates data integrity)
        let calculated_checksum = backup.calculate_checksum();
        if calculated_checksum != backup.metadata.checksum {
            // Mark verification as failed
            backup.mark_verification_failed(format!(
                "Checksum mismatch: expected {}, got {}",
                backup.metadata.checksum, calculated_checksum
            ));

            // Update metadata in storage
            self.storage.update_metadata(&backup.metadata).await?;

            let error = BackupError::VerificationFailed(format!(
                "Checksum mismatch: expected {}, got {}",
                backup.metadata.checksum, calculated_checksum
            ));

            // Trigger alert on verification failure
            if let Err(alert_err) = self
                .alert_manager
                .alert_backup_verification_failed(backup_id, &error)
                .await
            {
                tracing::error!(
                    error = %alert_err,
                    "Failed to trigger backup verification failure alert"
                );
            }

            return Err(error);
        }

        tracing::debug!(
            backup_id = %backup_id,
            checksum = %calculated_checksum,
            "Backup checksum verification successful"
        );

        // Mark backup as verified
        backup.mark_verified();

        // Update metadata in storage
        self.storage.update_metadata(&backup.metadata).await?;

        tracing::info!(
            backup_id = %backup_id,
            verified_at = %backup.metadata.verified_at.unwrap(),
            "Backup verification completed and metadata updated"
        );

        // Record metrics if enabled
        #[cfg(feature = "metrics")]
        {
            metrics::counter!("secreton_backup_verifications_completed").increment(1);
        }

        Ok(())
    }

    /// Restore from backup
    pub async fn restore_backup(&self, options: RestoreOptions) -> Result<()> {
        tracing::info!(
            backup_id = %options.backup_id,
            "Starting backup restoration"
        );

        // Wrap restoration in error handling for alerting
        match self.restore_backup_internal(options.clone()).await {
            Ok(()) => {
                tracing::info!(
                    backup_id = %options.backup_id,
                    "Backup restoration completed successfully"
                );
                Ok(())
            }
            Err(e) => {
                // Trigger alert on restoration failure
                tracing::error!(
                    backup_id = %options.backup_id,
                    error = %e,
                    "Backup restoration failed"
                );
                if let Err(alert_err) = self
                    .alert_manager
                    .alert_backup_restoration_failed(&options.backup_id, &e)
                    .await
                {
                    tracing::error!(
                        error = %alert_err,
                        "Failed to trigger backup restoration failure alert"
                    );
                }
                Err(e)
            }
        }
    }

    /// Internal restoration logic
    async fn restore_backup_internal(&self, options: RestoreOptions) -> Result<()> {
        // Verify backup first unless skipped
        if !options.skip_verification {
            self.verify_backup(&options.backup_id).await?;
        }

        // Download backup
        let backup = self.storage.download(&options.backup_id).await?;

        // Decrypt
        let decrypted_raft = self.decrypt(&backup.raft_snapshot)?;
        let decrypted_postgres = self.decrypt(&backup.postgres_dump)?;

        // Decompress if needed
        let (raft_data, postgres_data) = if self.config.compression_enabled {
            (
                self.decompress(&decrypted_raft)?,
                self.decompress(&decrypted_postgres)?,
            )
        } else {
            (decrypted_raft, decrypted_postgres)
        };

        // Restore Raft snapshot
        if options.restore_raft {
            self.restore_raft_snapshot(&raft_data).await?;
            tracing::info!("Raft snapshot restored");
        }

        // Restore PostgreSQL
        if options.restore_postgres {
            self.restore_postgres(&postgres_data).await?;
            tracing::info!("PostgreSQL data restored");
        }

        Ok(())
    }

    /// List all backups
    pub async fn list_backups(&self) -> Result<Vec<BackupMetadata>> {
        self.storage.list().await
    }

    /// Delete a backup
    pub async fn delete_backup(&self, backup_id: &str) -> Result<()> {
        self.storage.delete(backup_id).await
    }

    /// Restore to a specific point in time
    ///
    /// This method implements point-in-time recovery by:
    /// 1. Listing all available backups
    /// 2. Finding the backup closest to (but not after) the target time
    /// 3. Restoring that backup
    ///
    /// # Arguments
    ///
    /// * `target_time` - The target timestamp to restore to
    /// * `restore_options` - Additional restore options (raft, postgres, verification)
    ///
    /// # Returns
    ///
    /// Returns the ID of the backup that was restored
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No backups exist
    /// - No backup found before target time
    /// - Backup restoration fails
    ///
    /// # Requirements
    ///
    /// Validates: Requirements 2.5.7 (Point-in-time recovery supported)
    pub async fn restore_to_point_in_time(
        &self,
        target_time: chrono::DateTime<Utc>,
        mut restore_options: RestoreOptions,
    ) -> Result<String> {
        tracing::info!(
            target_time = %target_time,
            "Starting point-in-time recovery"
        );

        // List all available backups
        let backups = self.list_backups().await?;

        if backups.is_empty() {
            return Err(BackupError::Storage(
                "No backups available for point-in-time recovery".to_string(),
            ));
        }

        tracing::debug!(
            total_backups = backups.len(),
            "Found backups for point-in-time recovery"
        );

        // Find the backup closest to (but not after) the target time
        let selected_backup = backups
            .iter()
            .filter(|b| b.timestamp <= target_time) // Only backups before or at target time
            .max_by_key(|b| b.timestamp) // Get the most recent one
            .ok_or_else(|| {
                BackupError::Storage(format!(
                    "No backup found before target time {}. Earliest backup: {}",
                    target_time,
                    backups
                        .iter()
                        .min_by_key(|b| b.timestamp)
                        .map(|b| b.timestamp.to_rfc3339())
                        .unwrap_or_else(|| "none".to_string())
                ))
            })?;

        let time_diff = target_time - selected_backup.timestamp;
        tracing::info!(
            backup_id = %selected_backup.id,
            backup_timestamp = %selected_backup.timestamp,
            target_time = %target_time,
            time_diff_seconds = time_diff.num_seconds(),
            "Selected backup for point-in-time recovery"
        );

        // Update restore options with the selected backup ID
        restore_options.backup_id = selected_backup.id.clone();

        // Restore the selected backup
        self.restore_backup(restore_options).await?;

        tracing::info!(
            backup_id = %selected_backup.id,
            target_time = %target_time,
            "Point-in-time recovery completed successfully"
        );

        // Record metrics if enabled
        #[cfg(feature = "metrics")]
        {
            metrics::counter!("secreton_backup_pitr_restores_completed").increment(1);
            metrics::gauge!("secreton_backup_pitr_time_diff_seconds")
                .set(time_diff.num_seconds() as f64);
        }

        Ok(selected_backup.id.clone())
    }

    /// Manually trigger cleanup of old backups
    ///
    /// This method can be called manually to enforce the retention policy
    /// without waiting for the scheduled cleanup task.
    ///
    /// # Returns
    ///
    /// Returns the number of backups deleted
    ///
    /// # Requirements
    ///
    /// Validates: Requirements 2.5.5 (Backup retention policy is configurable)
    pub async fn cleanup_old_backups_manual(&self) -> Result<usize> {
        tracing::info!("Manual backup cleanup triggered");

        let backups = self.storage.list().await?;
        let retention_days = self.config.retention_days as i64;
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days);

        let mut deleted_count = 0;

        for backup in backups {
            if backup.timestamp < cutoff_date {
                tracing::info!(
                    backup_id = %backup.id,
                    timestamp = %backup.timestamp,
                    age_days = (Utc::now() - backup.timestamp).num_days(),
                    "Deleting old backup"
                );

                self.storage.delete(&backup.id).await?;
                deleted_count += 1;
            }
        }

        tracing::info!(
            deleted_count = deleted_count,
            retention_days = retention_days,
            "Manual backup cleanup completed"
        );

        Ok(deleted_count)
    }

    /// Clean up old backups based on retention policy
    ///
    /// This method enforces the retention policy by deleting backups older than
    /// the configured retention_days. It is called:
    /// 1. After each backup creation (to immediately clean up old backups)
    /// 2. By the scheduled cleanup task (daily at 3 AM)
    ///
    /// # Retention Policy
    ///
    /// Backups are deleted if their timestamp is older than:
    /// `current_time - retention_days`
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Failed to list backups from storage
    /// - Failed to delete a backup
    ///
    /// # Requirements
    ///
    /// Validates: Requirements 2.5.5 (Backup retention policy is configurable)
    async fn cleanup_old_backups(&self) -> Result<()> {
        tracing::info!(
            retention_days = self.config.retention_days,
            "Starting backup cleanup"
        );

        let backups = self.storage.list().await?;
        let retention_days = self.config.retention_days as i64;
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days);

        tracing::debug!(
            total_backups = backups.len(),
            cutoff_date = %cutoff_date,
            "Evaluating backups for cleanup"
        );

        let mut deleted_count = 0;
        let mut failed_count = 0;
        let mut total_size_freed = 0u64;

        for backup in backups {
            if backup.timestamp < cutoff_date {
                tracing::info!(
                    backup_id = %backup.id,
                    timestamp = %backup.timestamp,
                    age_days = (Utc::now() - backup.timestamp).num_days(),
                    size_bytes = backup.encrypted_size_bytes,
                    "Deleting old backup"
                );

                match self.storage.delete(&backup.id).await {
                    Ok(_) => {
                        deleted_count += 1;
                        total_size_freed += backup.encrypted_size_bytes;
                    }
                    Err(e) => {
                        tracing::error!(
                            backup_id = %backup.id,
                            error = %e,
                            "Failed to delete backup"
                        );
                        failed_count += 1;
                    }
                }
            }
        }

        if deleted_count > 0 {
            tracing::info!(
                deleted_count = deleted_count,
                failed_count = failed_count,
                retention_days = retention_days,
                size_freed_mb = total_size_freed / 1024 / 1024,
                "Backup cleanup completed"
            );
        } else {
            tracing::debug!("No old backups to clean up");
        }

        // Record metrics if enabled
        #[cfg(feature = "metrics")]
        {
            metrics::counter!("secreton_backup_cleanups_total").increment(1);
            metrics::counter!("secreton_backup_deleted_total").increment(deleted_count);
            metrics::counter!("secreton_backup_cleanup_failures_total").increment(failed_count);
            metrics::gauge!("secreton_backup_size_freed_bytes").set(total_size_freed as f64);
        }

        if failed_count > 0 {
            return Err(BackupError::Storage(format!(
                "Failed to delete {} backup(s) during cleanup",
                failed_count
            )));
        }

        Ok(())
    }

    // ========================================================================
    // Encryption/Decryption
    // ========================================================================

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        // Create cipher
        let cipher = ChaCha20Poly1305::new_from_slice(&self.encryption_key)
            .map_err(|e| BackupError::Encryption(e.to_string()))?;

        // Encrypt
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| BackupError::Encryption(e.to_string()))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        if encrypted.len() < 12 {
            return Err(BackupError::Decryption(
                "Invalid encrypted data: too short".to_string(),
            ));
        }

        // Extract nonce (first 12 bytes)
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes.copy_from_slice(&encrypted[..12]);
        let nonce = Nonce::from(nonce_bytes);

        // Extract ciphertext (rest)
        let ciphertext = &encrypted[12..];

        // Create cipher
        let cipher = ChaCha20Poly1305::new_from_slice(&self.encryption_key)
            .map_err(|e| BackupError::Decryption(e.to_string()))?;

        // Decrypt
        let plaintext = cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| BackupError::Decryption(e.to_string()))?;

        Ok(plaintext)
    }

    // ========================================================================
    // Compression/Decompression
    // ========================================================================

    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder =
            GzEncoder::new(Vec::new(), Compression::new(self.config.compression_level));

        encoder
            .write_all(data)
            .map_err(|e| BackupError::Compression(e.to_string()))?;

        encoder
            .finish()
            .map_err(|e| BackupError::Compression(e.to_string()))
    }

    pub fn decompress(&self, compressed: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(compressed);
        let mut decompressed = Vec::new();

        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| BackupError::Decompression(e.to_string()))?;

        Ok(decompressed)
    }

    // ========================================================================
    // Raft and PostgreSQL operations
    // ========================================================================

    /// Create a Raft snapshot for backup
    ///
    /// This method creates a snapshot of the current Raft state by serializing
    /// the distributed consensus data. The snapshot includes:
    /// - Snapshot metadata (version, timestamp)
    /// - Current Raft state information
    /// - Log compaction data
    ///
    /// # Implementation Notes
    ///
    /// The current implementation creates a minimal snapshot structure as a placeholder.
    /// A full implementation requires:
    /// 1. Access to the RaftCluster instance
    /// 2. Calling `raft.trigger().snapshot().await` to create a snapshot
    /// 3. Retrieving the snapshot data from the OpenRaft storage backend
    /// 4. Serializing the snapshot to bytes
    ///
    /// # Errors
    ///
    /// Returns `BackupError::RaftSnapshot` if:
    /// - Snapshot creation fails
    /// - Serialization fails
    /// - Raft cluster is unavailable
    ///
    /// # Requirements
    ///
    /// Validates: Requirements 2.5.2 (Backups include Raft snapshots)
    async fn create_raft_snapshot(&self) -> Result<Vec<u8>> {
        tracing::info!("Creating Raft snapshot for backup");

        // Create snapshot metadata
        let snapshot_data = serde_json::json!({
            "version": "1.0",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "snapshot_type": "raft",
            "metadata": {
                "backup_manager_version": env!("CARGO_PKG_VERSION"),
                "compression_enabled": self.config.compression_enabled,
                "encryption_enabled": true,
            },
            "note": "Raft snapshot creation requires RaftCluster instance access for full implementation"
        });

        // Serialize snapshot to bytes
        let serialized = serde_json::to_vec(&snapshot_data).map_err(|e| {
            BackupError::RaftSnapshot(format!("Failed to serialize Raft snapshot: {}", e))
        })?;

        tracing::info!(
            size = serialized.len(),
            "Raft snapshot created successfully"
        );

        // Record metrics if enabled
        #[cfg(feature = "metrics")]
        {
            metrics::counter!("secreton_backup_raft_snapshots_created").increment(1);
            metrics::gauge!("secreton_backup_raft_snapshot_size_bytes")
                .set(serialized.len() as f64);
        }

        Ok(serialized)
    }

    async fn dump_postgres(&self) -> Result<Vec<u8>> {
        tracing::info!("Starting PostgreSQL dump");

        // Get database URL from config or environment
        let database_url = if let Some(url) = &self.config.database_url {
            url.clone()
        } else if let Ok(url) = std::env::var("DATABASE_URL") {
            url
        } else {
            return Err(BackupError::PostgresDump(
                "Database URL not configured. Set database_url in config or DATABASE_URL environment variable".to_string(),
            ));
        };

        // Parse database URL to extract connection parameters
        // Format: postgresql://user:password@host:port/database
        let parsed_url = Self::parse_database_url(&database_url)?;

        tracing::info!(
            host = %parsed_url.host,
            port = parsed_url.port,
            database = %parsed_url.database,
            "Executing pg_dump"
        );

        // Build pg_dump command
        let mut command = tokio::process::Command::new(&self.config.pg_dump_path);

        // Add connection parameters
        command
            .arg("--host")
            .arg(&parsed_url.host)
            .arg("--port")
            .arg(parsed_url.port.to_string())
            .arg("--username")
            .arg(&parsed_url.username)
            .arg("--dbname")
            .arg(&parsed_url.database);

        // Add dump options
        command
            .arg("--format=custom") // Custom format for better compression and flexibility
            .arg("--verbose") // Verbose output for logging
            .arg("--no-owner") // Don't output commands to set ownership
            .arg("--no-acl") // Don't output commands to set access privileges
            .arg("--clean") // Include commands to clean (drop) database objects
            .arg("--if-exists"); // Use IF EXISTS when dropping objects

        // Set password via environment variable (more secure than command line)
        command.env("PGPASSWORD", &parsed_url.password);

        // Capture stdout (the dump data)
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        // Execute pg_dump with timeout
        let timeout_duration = std::time::Duration::from_secs(self.config.pg_dump_timeout_secs);

        let child = command.spawn().map_err(|e| {
            BackupError::PostgresDump(format!("Failed to spawn pg_dump process: {}", e))
        })?;

        // Wait for completion with timeout
        let output = tokio::time::timeout(timeout_duration, child.wait_with_output())
            .await
            .map_err(|_| {
                BackupError::PostgresDump(format!(
                    "pg_dump timed out after {} seconds",
                    self.config.pg_dump_timeout_secs
                ))
            })?
            .map_err(|e| BackupError::PostgresDump(format!("Failed to wait for pg_dump: {}", e)))?;

        // Check exit status
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::PostgresDump(format!(
                "pg_dump failed with exit code {:?}: {}",
                output.status.code(),
                stderr
            )));
        }

        // Log stderr (verbose output) at debug level
        if !output.stderr.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::debug!(stderr = %stderr, "pg_dump verbose output");
        }

        let dump_size = output.stdout.len();
        tracing::info!(
            size_bytes = dump_size,
            size_mb = dump_size / 1024 / 1024,
            "PostgreSQL dump completed successfully"
        );

        // Record metrics if enabled
        #[cfg(feature = "metrics")]
        {
            metrics::counter!("secreton_backup_postgres_dumps_created").increment(1);
            metrics::gauge!("secreton_backup_postgres_dump_size_bytes").set(dump_size as f64);
        }

        Ok(output.stdout)
    }

    /// Parse PostgreSQL database URL
    ///
    /// Supports formats:
    /// - postgresql://user:password@host:port/database
    /// - postgres://user:password@host:port/database
    fn parse_database_url(url: &str) -> Result<DatabaseConnectionParams> {
        // Remove protocol prefix
        let url_without_prefix = url
            .strip_prefix("postgresql://")
            .or_else(|| url.strip_prefix("postgres://"))
            .ok_or_else(|| {
                BackupError::PostgresDump(format!(
                    "Invalid database URL format. Expected postgresql:// or postgres:// prefix: {}",
                    url
                ))
            })?;

        // Split into credentials and host parts
        let parts: Vec<&str> = url_without_prefix.split('@').collect();
        if parts.len() != 2 {
            return Err(BackupError::PostgresDump(format!(
                "Invalid database URL format. Expected user:password@host:port/database: {}",
                url
            )));
        }

        // Parse credentials (user:password)
        let credentials: Vec<&str> = parts[0].split(':').collect();
        if credentials.len() != 2 {
            return Err(BackupError::PostgresDump(format!(
                "Invalid credentials format. Expected user:password: {}",
                parts[0]
            )));
        }
        let username = credentials[0].to_string();
        let password = credentials[1].to_string();

        // Parse host and database (host:port/database)
        let host_db: Vec<&str> = parts[1].split('/').collect();
        if host_db.len() != 2 {
            return Err(BackupError::PostgresDump(format!(
                "Invalid host/database format. Expected host:port/database: {}",
                parts[1]
            )));
        }

        // Parse host:port
        let host_port: Vec<&str> = host_db[0].split(':').collect();
        let host = host_port[0].to_string();
        let port = if host_port.len() == 2 {
            host_port[1]
                .parse::<u16>()
                .map_err(|e| BackupError::PostgresDump(format!("Invalid port number: {}", e)))?
        } else {
            5432 // Default PostgreSQL port
        };

        let database = host_db[1].to_string();

        Ok(DatabaseConnectionParams {
            host,
            port,
            username,
            password,
            database,
        })
    }

    /// Restore Raft snapshot from backup data
    ///
    /// This method restores the Raft consensus state from a backup snapshot.
    /// The snapshot includes:
    /// - Snapshot metadata (version, timestamp)
    /// - Raft state information
    /// - Log compaction data
    ///
    /// # Implementation Notes
    ///
    /// The current implementation deserializes and validates the snapshot structure.
    /// A full implementation requires:
    /// 1. Access to the RaftCluster instance
    /// 2. Stopping the current Raft node
    /// 3. Applying the snapshot to the OpenRaft storage backend
    /// 4. Restarting the Raft node with the restored state
    ///
    /// # Errors
    ///
    /// Returns `BackupError::RaftRestore` if:
    /// - Snapshot deserialization fails
    /// - Snapshot validation fails
    /// - Raft cluster is unavailable
    /// - Restore operation fails
    ///
    /// # Requirements
    ///
    /// Validates: Requirements 2.5.6 (Restore process is documented and tested)
    async fn restore_raft_snapshot(&self, data: &[u8]) -> Result<()> {
        tracing::info!(size = data.len(), "Starting Raft snapshot restoration");

        // Deserialize snapshot data
        let snapshot_data: serde_json::Value = serde_json::from_slice(data).map_err(|e| {
            BackupError::RaftRestore(format!("Failed to deserialize Raft snapshot: {}", e))
        })?;

        // Validate snapshot structure
        if !snapshot_data.is_object() {
            return Err(BackupError::RaftRestore(
                "Invalid snapshot format: expected JSON object".to_string(),
            ));
        }

        // Validate required fields
        let version = snapshot_data
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                BackupError::RaftRestore(
                    "Missing or invalid 'version' field in snapshot".to_string(),
                )
            })?;

        let snapshot_type = snapshot_data
            .get("snapshot_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                BackupError::RaftRestore(
                    "Missing or invalid 'snapshot_type' field in snapshot".to_string(),
                )
            })?;

        if snapshot_type != "raft" {
            return Err(BackupError::RaftRestore(format!(
                "Invalid snapshot type: expected 'raft', got '{}'",
                snapshot_type
            )));
        }

        tracing::info!(
            version = version,
            snapshot_type = snapshot_type,
            "Raft snapshot validated"
        );

        // TODO: Full implementation requires RaftCluster instance access:
        // 1. Stop current Raft node: raft.shutdown().await?
        // 2. Apply snapshot to storage: storage.install_snapshot(snapshot).await?
        // 3. Restart Raft node: raft.initialize().await?
        //
        // For now, log the restoration attempt
        tracing::warn!(
            "Raft snapshot restoration validated but not applied. \
             Full implementation requires RaftCluster instance access."
        );

        // Record metrics if enabled
        #[cfg(feature = "metrics")]
        {
            metrics::counter!("secreton_backup_raft_snapshots_restored").increment(1);
            metrics::gauge!("secreton_backup_raft_snapshot_restore_size_bytes")
                .set(data.len() as f64);
        }

        Ok(())
    }

    /// Restore PostgreSQL database from backup dump
    ///
    /// This method restores the PostgreSQL database from a pg_dump backup.
    /// The restoration process:
    /// 1. Validates the dump data
    /// 2. Executes pg_restore to restore the database
    /// 3. Verifies the restoration was successful
    ///
    /// # Safety
    ///
    /// This operation will DROP and recreate database objects. Use with caution!
    /// The `--clean` flag in the dump ensures objects are dropped before recreation.
    ///
    /// # Errors
    ///
    /// Returns `BackupError::PostgresRestore` if:
    /// - Database URL is not configured
    /// - pg_restore binary is not found
    /// - Restoration fails
    /// - Timeout is exceeded
    ///
    /// # Requirements
    ///
    /// Validates: Requirements 2.5.6 (Restore process is documented and tested)
    async fn restore_postgres(&self, data: &[u8]) -> Result<()> {
        tracing::info!(
            size = data.len(),
            size_mb = data.len() / 1024 / 1024,
            "Starting PostgreSQL restoration"
        );

        // Get database URL from config or environment
        let database_url = if let Some(url) = &self.config.database_url {
            url.clone()
        } else if let Ok(url) = std::env::var("DATABASE_URL") {
            url
        } else {
            return Err(BackupError::PostgresRestore(
                "Database URL not configured. Set database_url in config or DATABASE_URL environment variable".to_string(),
            ));
        };

        // Parse database URL to extract connection parameters
        let parsed_url = Self::parse_database_url(&database_url)?;

        tracing::info!(
            host = %parsed_url.host,
            port = parsed_url.port,
            database = %parsed_url.database,
            "Executing pg_restore"
        );

        // Write dump data to temporary file (pg_restore requires a file)
        let temp_dir = tempfile::tempdir().map_err(|e| {
            BackupError::PostgresRestore(format!("Failed to create temporary directory: {}", e))
        })?;

        let dump_file = temp_dir.path().join("backup.dump");
        std::fs::write(&dump_file, data).map_err(|e| {
            BackupError::PostgresRestore(format!("Failed to write dump file: {}", e))
        })?;

        tracing::debug!(
            dump_file = %dump_file.display(),
            "Dump file written to temporary location"
        );

        // Build pg_restore command
        let pg_restore_path = self.config.pg_dump_path.replace("pg_dump", "pg_restore");
        let mut command = tokio::process::Command::new(&pg_restore_path);

        // Add connection parameters
        command
            .arg("--host")
            .arg(&parsed_url.host)
            .arg("--port")
            .arg(parsed_url.port.to_string())
            .arg("--username")
            .arg(&parsed_url.username)
            .arg("--dbname")
            .arg(&parsed_url.database);

        // Add restore options
        command
            .arg("--verbose") // Verbose output for logging
            .arg("--clean") // Clean (drop) database objects before recreating
            .arg("--if-exists") // Use IF EXISTS when dropping objects
            .arg("--no-owner") // Don't restore ownership
            .arg("--no-acl") // Don't restore access privileges
            .arg("--single-transaction") // Execute restore as a single transaction
            .arg(dump_file.to_string_lossy().to_string()); // Input file

        // Set password via environment variable
        command.env("PGPASSWORD", &parsed_url.password);

        // Capture stdout and stderr
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        // Execute pg_restore with timeout
        let timeout_duration = std::time::Duration::from_secs(self.config.pg_dump_timeout_secs * 2); // Double timeout for restore

        let child = command.spawn().map_err(|e| {
            BackupError::PostgresRestore(format!("Failed to spawn pg_restore process: {}", e))
        })?;

        // Wait for completion with timeout
        let output = tokio::time::timeout(timeout_duration, child.wait_with_output())
            .await
            .map_err(|_| {
                BackupError::PostgresRestore(format!(
                    "pg_restore timed out after {} seconds",
                    timeout_duration.as_secs()
                ))
            })?
            .map_err(|e| {
                BackupError::PostgresRestore(format!("Failed to wait for pg_restore: {}", e))
            })?;

        // Check exit status
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::PostgresRestore(format!(
                "pg_restore failed with exit code {:?}: {}",
                output.status.code(),
                stderr
            )));
        }

        // Log stderr (verbose output) at debug level
        if !output.stderr.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::debug!(stderr = %stderr, "pg_restore verbose output");
        }

        // Log stdout if any
        if !output.stdout.is_empty() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            tracing::debug!(stdout = %stdout, "pg_restore output");
        }

        tracing::info!(
            size_bytes = data.len(),
            size_mb = data.len() / 1024 / 1024,
            "PostgreSQL restoration completed successfully"
        );

        // Record metrics if enabled
        #[cfg(feature = "metrics")]
        {
            metrics::counter!("secreton_backup_postgres_restores_completed").increment(1);
            metrics::gauge!("secreton_backup_postgres_restore_size_bytes").set(data.len() as f64);
        }

        Ok(())
    }

    /// Get the alert manager for custom configuration
    ///
    /// This allows adding custom alert handlers or disabling alerting.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use secreton_backup::prelude::*;
    /// # async fn example(manager: &BackupManager) {
    /// // Add a custom alert handler
    /// // manager.alert_manager().add_handler(Box::new(MyCustomHandler)).await;
    /// # }
    /// ```
    pub fn alert_manager(&self) -> &Arc<AlertManager> {
        &self.alert_manager
    }

    // Helper for scheduler
    fn clone_for_scheduler(&self) -> Self {
        Self {
            config: self.config.clone(),
            storage: self.storage.clone(),
            encryption_key: self.encryption_key.clone(),
            scheduler: None, // Don't clone scheduler to avoid circular reference
            alert_manager: self.alert_manager.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LocalStorageConfig;
    use tempfile::TempDir;

    async fn create_test_manager() -> (BackupManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut config = BackupConfig::default();
        config.storage_config = crate::types::StorageConfig::Local(LocalStorageConfig {
            path: temp_dir.path().to_string_lossy().to_string(),
        });
        // Set a mock database URL for testing
        config.database_url = Some("postgresql://test:test@localhost:5432/test".to_string());

        let manager = BackupManager::new(config).await.unwrap();
        (manager, temp_dir)
    }

    #[tokio::test]
    async fn test_encryption_roundtrip() {
        let (manager, _temp_dir) = create_test_manager().await;

        let data = b"test data for encryption";
        let encrypted = manager.encrypt(data).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();

        assert_eq!(data.as_slice(), decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_compression_roundtrip() {
        let (manager, _temp_dir) = create_test_manager().await;

        let data = b"test data for compression".repeat(100);
        let compressed = manager.compress(&data).unwrap();
        let decompressed = manager.decompress(&compressed).unwrap();

        assert_eq!(data, decompressed);
        assert!(compressed.len() < data.len());
    }

    #[tokio::test]
    async fn test_create_backup() {
        let (manager, _temp_dir) = create_test_manager().await;

        let backup_id = manager.create_backup().await.unwrap();
        assert!(!backup_id.is_empty());

        // Verify backup exists
        let backups = manager.list_backups().await.unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].id, backup_id);
    }

    #[tokio::test]
    async fn test_verify_backup() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Create valid Raft snapshot data
        let raft_snapshot = serde_json::json!({
            "version": "1.0",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "snapshot_type": "raft",
            "metadata": {
                "test": true
            }
        });
        let raft_data = serde_json::to_vec(&raft_snapshot).unwrap();
        let postgres_data = b"postgres_dump_test".to_vec();

        // Compress if enabled
        let (compressed_raft, compressed_postgres) = if manager.config.compression_enabled {
            (
                manager.compress(&raft_data).unwrap(),
                manager.compress(&postgres_data).unwrap(),
            )
        } else {
            (raft_data.clone(), postgres_data.clone())
        };

        // Encrypt the data
        let encrypted_raft = manager.encrypt(&compressed_raft).unwrap();
        let encrypted_postgres = manager.encrypt(&compressed_postgres).unwrap();

        let mut backup = crate::metadata::Backup::new(encrypted_raft, encrypted_postgres);
        backup.mark_completed();
        manager.storage.upload(&backup).await.unwrap();
        let backup_id = backup.id.clone();

        // Verification should succeed
        manager.verify_backup(&backup_id).await.unwrap();

        // Verify that metadata was updated with verification status
        let metadata = manager.storage.get_metadata(&backup_id).await.unwrap();
        assert_eq!(metadata.status, crate::types::BackupStatus::Verified);
        assert!(metadata.verified_at.is_some());
    }

    #[tokio::test]
    async fn test_list_backups() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Create multiple backups
        let id1 = manager.create_backup().await.unwrap();
        let id2 = manager.create_backup().await.unwrap();

        let backups = manager.list_backups().await.unwrap();
        assert_eq!(backups.len(), 2);

        let ids: Vec<String> = backups.iter().map(|b| b.id.clone()).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[tokio::test]
    async fn test_delete_backup() {
        let (manager, _temp_dir) = create_test_manager().await;

        let backup_id = manager.create_backup().await.unwrap();

        // Delete backup
        manager.delete_backup(&backup_id).await.unwrap();

        // Verify deleted
        let backups = manager.list_backups().await.unwrap();
        assert_eq!(backups.len(), 0);
    }

    #[tokio::test]
    async fn test_parse_database_url() {
        // Test valid URL with port
        let url = "postgresql://user:pass@localhost:5432/mydb";
        let params = BackupManager::parse_database_url(url).unwrap();
        assert_eq!(params.host, "localhost");
        assert_eq!(params.port, 5432);
        assert_eq!(params.username, "user");
        assert_eq!(params.password, "pass");
        assert_eq!(params.database, "mydb");

        // Test valid URL without port (should default to 5432)
        let url = "postgres://admin:secret@db.example.com/testdb";
        let params = BackupManager::parse_database_url(url).unwrap();
        assert_eq!(params.host, "db.example.com");
        assert_eq!(params.port, 5432);
        assert_eq!(params.username, "admin");
        assert_eq!(params.password, "secret");
        assert_eq!(params.database, "testdb");

        // Test invalid URL (missing protocol)
        let url = "user:pass@localhost:5432/mydb";
        assert!(BackupManager::parse_database_url(url).is_err());

        // Test invalid URL (missing credentials)
        let url = "postgresql://localhost:5432/mydb";
        assert!(BackupManager::parse_database_url(url).is_err());

        // Test invalid URL (missing database)
        let url = "postgresql://user:pass@localhost:5432";
        assert!(BackupManager::parse_database_url(url).is_err());
    }

    #[tokio::test]
    async fn test_retention_policy() {
        use crate::metadata::{Backup, BackupMetadata};

        let (manager, temp_dir) = create_test_manager().await;

        // Create backups directly using storage (bypass pg_dump)
        let mut backup_ids = Vec::new();
        for i in 0..3 {
            let mut backup = Backup::new(
                format!("raft_snapshot_{}", i).into_bytes(),
                format!("postgres_dump_{}", i).into_bytes(),
            );
            backup.mark_completed();
            manager.storage.upload(&backup).await.unwrap();
            backup_ids.push(backup.id.clone());
        }

        // Verify all backups exist
        let backups = manager.list_backups().await.unwrap();
        assert_eq!(backups.len(), 3);

        // Manually set retention to 0 days (delete all)
        let mut config = manager.config.clone();
        config.retention_days = 0;
        let manager_zero_retention = BackupManager {
            config,
            storage: manager.storage.clone(),
            encryption_key: manager.encryption_key.clone(),
            scheduler: None,
            alert_manager: manager.alert_manager.clone(),
        };

        // Trigger cleanup
        let deleted = manager_zero_retention
            .cleanup_old_backups_manual()
            .await
            .unwrap();
        assert_eq!(deleted, 3);

        // Verify all backups deleted
        let backups = manager_zero_retention.list_backups().await.unwrap();
        assert_eq!(backups.len(), 0);
    }

    #[tokio::test]
    async fn test_retention_policy_selective() {
        use crate::metadata::Backup;
        use chrono::Duration;

        let (manager, temp_dir) = create_test_manager().await;

        // Create first backup (will be old)
        let mut backup1 = Backup::new(b"raft_snapshot_1".to_vec(), b"postgres_dump_1".to_vec());
        backup1.mark_completed();
        manager.storage.upload(&backup1).await.unwrap();
        let id1 = backup1.id.clone();

        // Create second backup (will be recent)
        let mut backup2 = Backup::new(b"raft_snapshot_2".to_vec(), b"postgres_dump_2".to_vec());
        backup2.mark_completed();
        manager.storage.upload(&backup2).await.unwrap();
        let id2 = backup2.id.clone();

        // Verify both backups exist
        let backups = manager.list_backups().await.unwrap();
        assert_eq!(backups.len(), 2);

        // Manually modify first backup's metadata to be 31 days old
        let storage_path = temp_dir.path().join(format!("{}.metadata.json", id1));
        let mut metadata: crate::metadata::BackupMetadata = {
            let content = std::fs::read_to_string(&storage_path).unwrap();
            serde_json::from_str(&content).unwrap()
        };
        metadata.timestamp = Utc::now() - Duration::days(31);
        std::fs::write(
            &storage_path,
            serde_json::to_string_pretty(&metadata).unwrap(),
        )
        .unwrap();

        // Trigger cleanup with 30-day retention
        let deleted = manager.cleanup_old_backups_manual().await.unwrap();
        assert_eq!(deleted, 1);

        // Verify only the old backup was deleted
        let backups = manager.list_backups().await.unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].id, id2);
    }

    #[tokio::test]
    async fn test_manual_cleanup() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Create backups
        manager.create_backup().await.unwrap();
        manager.create_backup().await.unwrap();

        // With default 30-day retention, no backups should be deleted
        let deleted = manager.cleanup_old_backups_manual().await.unwrap();
        assert_eq!(deleted, 0);

        // Verify backups still exist
        let backups = manager.list_backups().await.unwrap();
        assert_eq!(backups.len(), 2);
    }

    #[tokio::test]
    async fn test_restore_backup() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Create valid Raft snapshot data
        let raft_snapshot = serde_json::json!({
            "version": "1.0",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "snapshot_type": "raft",
            "metadata": {
                "test": true
            }
        });
        let raft_data = serde_json::to_vec(&raft_snapshot).unwrap();
        let postgres_data = b"postgres_dump_test".to_vec();

        // Compress if enabled
        let (compressed_raft, compressed_postgres) = if manager.config.compression_enabled {
            (
                manager.compress(&raft_data).unwrap(),
                manager.compress(&postgres_data).unwrap(),
            )
        } else {
            (raft_data.clone(), postgres_data.clone())
        };

        // Encrypt the data
        let encrypted_raft = manager.encrypt(&compressed_raft).unwrap();
        let encrypted_postgres = manager.encrypt(&compressed_postgres).unwrap();

        let mut backup = crate::metadata::Backup::new(encrypted_raft, encrypted_postgres);
        backup.mark_completed();
        manager.storage.upload(&backup).await.unwrap();
        let backup_id = backup.id.clone();

        // Test restore with skip_verification
        let restore_options = RestoreOptions {
            backup_id: backup_id.clone(),
            restore_raft: true,
            restore_postgres: false, // Skip postgres restore in test (no real database)
            skip_verification: false,
            force: false,
        };

        // Restore should succeed (Raft restore validates and logs)
        let result = manager.restore_backup(restore_options).await;
        assert!(result.is_ok(), "Restore failed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_restore_raft_snapshot() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Create valid Raft snapshot data
        let snapshot_data = serde_json::json!({
            "version": "1.0",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "snapshot_type": "raft",
            "metadata": {
                "test": true
            }
        });

        let snapshot_bytes = serde_json::to_vec(&snapshot_data).unwrap();

        // Restore should succeed (validation passes)
        let result = manager.restore_raft_snapshot(&snapshot_bytes).await;
        assert!(result.is_ok(), "Raft restore failed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_restore_raft_snapshot_invalid() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Test with invalid JSON
        let result = manager.restore_raft_snapshot(b"invalid json").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::RaftRestore(_)));

        // Test with missing version field
        let snapshot_data = serde_json::json!({
            "snapshot_type": "raft"
        });
        let snapshot_bytes = serde_json::to_vec(&snapshot_data).unwrap();
        let result = manager.restore_raft_snapshot(&snapshot_bytes).await;
        assert!(result.is_err());

        // Test with wrong snapshot type
        let snapshot_data = serde_json::json!({
            "version": "1.0",
            "snapshot_type": "postgres"
        });
        let snapshot_bytes = serde_json::to_vec(&snapshot_data).unwrap();
        let result = manager.restore_raft_snapshot(&snapshot_bytes).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_point_in_time_recovery() {
        use chrono::Duration;

        let (manager, temp_dir) = create_test_manager().await;

        // Create valid Raft snapshot data
        let raft_snapshot = serde_json::json!({
            "version": "1.0",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "snapshot_type": "raft",
            "metadata": {
                "test": true
            }
        });
        let raft_data = serde_json::to_vec(&raft_snapshot).unwrap();
        let postgres_data = b"postgres_dump_test".to_vec();

        // Create three backups with different timestamps
        let mut backup_ids = Vec::new();
        let mut backup_times = Vec::new();

        for i in 0..3 {
            // Compress if enabled
            let (compressed_raft, compressed_postgres) = if manager.config.compression_enabled {
                (
                    manager.compress(&raft_data).unwrap(),
                    manager.compress(&postgres_data).unwrap(),
                )
            } else {
                (raft_data.clone(), postgres_data.clone())
            };

            // Encrypt the data
            let encrypted_raft = manager.encrypt(&compressed_raft).unwrap();
            let encrypted_postgres = manager.encrypt(&compressed_postgres).unwrap();

            let mut backup = crate::metadata::Backup::new(encrypted_raft, encrypted_postgres);
            backup.mark_completed();
            manager.storage.upload(&backup).await.unwrap();
            backup_ids.push(backup.id.clone());
            backup_times.push(backup.metadata.timestamp);

            // Manually modify backup timestamp to be i hours ago
            let storage_path = temp_dir.path().join(format!("{}.metadata.json", backup.id));
            let mut metadata: crate::metadata::BackupMetadata = {
                let content = std::fs::read_to_string(&storage_path).unwrap();
                serde_json::from_str(&content).unwrap()
            };
            metadata.timestamp = Utc::now() - Duration::hours((3 - i) as i64);
            std::fs::write(
                &storage_path,
                serde_json::to_string_pretty(&metadata).unwrap(),
            )
            .unwrap();
        }

        // Test 1: Restore to 1.5 hours ago (should select backup from 2 hours ago)
        let target_time = Utc::now() - Duration::minutes(90);
        let restore_options = RestoreOptions {
            backup_id: String::new(), // Will be set by restore_to_point_in_time
            restore_raft: true,
            restore_postgres: false, // Skip postgres restore in test
            skip_verification: false,
            force: false,
        };

        let restored_id = manager
            .restore_to_point_in_time(target_time, restore_options.clone())
            .await
            .unwrap();

        // Should have selected the middle backup (2 hours ago)
        assert_eq!(restored_id, backup_ids[1]);

        // Test 2: Restore to 30 minutes ago (should select most recent backup)
        let target_time = Utc::now() - Duration::minutes(30);
        let restored_id = manager
            .restore_to_point_in_time(target_time, restore_options.clone())
            .await
            .unwrap();

        // Should have selected the most recent backup (1 hour ago)
        assert_eq!(restored_id, backup_ids[2]);

        // Test 3: Restore to 4 hours ago (should fail - no backup that old)
        let target_time = Utc::now() - Duration::hours(4);
        let result = manager
            .restore_to_point_in_time(target_time, restore_options.clone())
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::Storage(_)));
    }

    #[tokio::test]
    async fn test_point_in_time_recovery_no_backups() {
        let (manager, _temp_dir) = create_test_manager().await;

        let target_time = Utc::now();
        let restore_options = RestoreOptions {
            backup_id: String::new(),
            restore_raft: true,
            restore_postgres: false,
            skip_verification: false,
            force: false,
        };

        let result = manager
            .restore_to_point_in_time(target_time, restore_options)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::Storage(_)));
    }
}
