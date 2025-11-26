//! Automatic key rotation service for enhanced security
//!
//! This module provides automatic key rotation functionality that integrates
//! with Secreton for secure key management. Keys are rotated on a configurable
//! schedule (default: every 30 days) to minimize the impact of key compromise.

use crate::error::{AuthencError, Result};
use crate::models::user::SecurityContext;
use crate::secreton_client::secreton_client::SecretonClient;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Key rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationConfig {
    /// Whether automatic key rotation is enabled
    pub enabled: bool,
    /// Rotation interval in days (default: 30)
    pub rotation_interval_days: u32,
    /// Grace period for old keys in days (default: 7)
    pub grace_period_days: u32,
    /// Whether to send notifications on rotation
    pub enable_notifications: bool,
}

impl Default for KeyRotationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rotation_interval_days: 30,
            grace_period_days: 7,
            enable_notifications: true,
        }
    }
}

/// Key rotation event for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationEvent {
    /// Event ID
    pub id: Uuid,
    /// Timestamp of the rotation
    pub timestamp: DateTime<Utc>,
    /// Key identifier that was rotated
    pub key_id: String,
    /// Key type (signing, encryption, etc.)
    pub key_type: KeyType,
    /// Previous key version
    pub old_version: u32,
    /// New key version
    pub new_version: u32,
    /// Rotation status
    pub status: RotationStatus,
    /// Optional error message if rotation failed
    pub error_message: Option<String>,
    /// User or system that initiated the rotation
    pub initiated_by: String,
}

/// Type of cryptographic key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyType {
    /// JWT signing key (Ed25519)
    JwtSigning,
    /// Session encryption key (AES-GCM)
    SessionEncryption,
    /// MFA secret encryption key
    MfaEncryption,
    /// Database encryption key
    DatabaseEncryption,
    /// Generic encryption key
    Generic,
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::JwtSigning => write!(f, "jwt_signing"),
            KeyType::SessionEncryption => write!(f, "session_encryption"),
            KeyType::MfaEncryption => write!(f, "mfa_encryption"),
            KeyType::DatabaseEncryption => write!(f, "database_encryption"),
            KeyType::Generic => write!(f, "generic"),
        }
    }
}

/// Status of a key rotation operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RotationStatus {
    /// Rotation completed successfully
    Success,
    /// Rotation failed
    Failed,
    /// Rotation in progress
    InProgress,
    /// Rotation scheduled
    Scheduled,
}

/// Key metadata for tracking rotation schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Key identifier
    pub key_id: String,
    /// Key type
    pub key_type: KeyType,
    /// Current version
    pub version: u32,
    /// Last rotation timestamp
    pub last_rotated: DateTime<Utc>,
    /// Next scheduled rotation
    pub next_rotation: DateTime<Utc>,
    /// Whether the key is active
    pub is_active: bool,
}

/// Automatic key rotation service
pub struct KeyRotationService {
    /// Secreton client for key operations
    secreton_client: Arc<SecretonClient>,
    /// Database for audit logging
    database: Arc<crate::database::Database>,
    /// Configuration
    config: KeyRotationConfig,
    /// Key metadata cache
    key_metadata: Arc<RwLock<Vec<KeyMetadata>>>,
    /// Rotation task handle
    task_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl KeyRotationService {
    /// Create a new key rotation service
    pub fn new(
        secreton_client: Arc<SecretonClient>,
        database: Arc<crate::database::Database>,
        config: KeyRotationConfig,
    ) -> Self {
        Self {
            secreton_client,
            database,
            config,
            key_metadata: Arc::new(RwLock::new(Vec::new())),
            task_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the automatic key rotation scheduler
    pub fn start(&self) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("Key rotation is disabled in configuration");
            return Ok(());
        }

        let service = self.clone();
        let handle = tokio::spawn(async move {
            service.run_scheduler().await;
        });

        let mut task_handle = self.task_handle.blocking_write();
        *task_handle = Some(handle);

        tracing::info!(
            "Key rotation scheduler started (interval: {} days)",
            self.config.rotation_interval_days
        );

        Ok(())
    }

    /// Stop the key rotation scheduler
    pub async fn stop(&self) {
        let mut task_handle = self.task_handle.write().await;
        if let Some(handle) = task_handle.take() {
            handle.abort();
            tracing::info!("Key rotation scheduler stopped");
        }
    }

    /// Run the scheduler loop
    async fn run_scheduler(&self) {
        let check_interval = tokio::time::Duration::from_secs(24 * 60 * 60); // Check daily

        loop {
            tokio::time::sleep(check_interval).await;

            if let Err(e) = self.check_and_rotate_keys().await {
                tracing::error!("Key rotation check failed: {}", e);
            }
        }
    }

    /// Check all keys and rotate if necessary
    async fn check_and_rotate_keys(&self) -> Result<()> {
        tracing::info!("Checking keys for rotation");

        let metadata = self.key_metadata.read().await;
        let now = Utc::now();

        // Collect keys that need rotation
        let keys_to_rotate: Vec<(String, KeyType)> = metadata
            .iter()
            .filter(|key| key.is_active && now >= key.next_rotation)
            .map(|key| (key.key_id.clone(), key.key_type.clone()))
            .collect();

        drop(metadata); // Release read lock before rotation

        // Rotate each key
        for (key_id, key_type) in keys_to_rotate {
            tracing::info!("Key {} (type: {}) is due for rotation", key_id, key_type);

            if let Err(e) = self.rotate_key(&key_id, &key_type).await {
                tracing::error!("Failed to rotate key {}: {}", key_id, e);
            }
        }

        Ok(())
    }

    /// Rotate a specific key
    pub async fn rotate_key(&self, key_id: &str, key_type: &KeyType) -> Result<()> {
        tracing::info!("Starting rotation for key: {} (type: {})", key_id, key_type);

        let event_id = Uuid::new_v4();
        let start_time = Utc::now();

        // Get current key metadata
        let current_version = self.get_key_version(key_id).await?;

        // Create rotation event (in progress)
        let mut event = KeyRotationEvent {
            id: event_id,
            timestamp: start_time,
            key_id: key_id.to_string(),
            key_type: key_type.clone(),
            old_version: current_version,
            new_version: current_version + 1,
            status: RotationStatus::InProgress,
            error_message: None,
            initiated_by: "system".to_string(),
        };

        // Log rotation start
        self.log_rotation_event(&event).await?;

        // Perform rotation via Secreton
        match self.perform_secreton_rotation(key_id, key_type).await {
            Ok(new_version) => {
                event.status = RotationStatus::Success;
                event.new_version = new_version;

                // Update key metadata
                self.update_key_metadata(key_id, new_version).await?;

                // Log successful rotation
                self.log_rotation_event(&event).await?;

                // Send notification if enabled
                if self.config.enable_notifications {
                    self.send_rotation_notification(&event).await?;
                }

                tracing::info!(
                    "Successfully rotated key {} from version {} to {}",
                    key_id,
                    current_version,
                    new_version
                );

                Ok(())
            }
            Err(e) => {
                event.status = RotationStatus::Failed;
                event.error_message = Some(e.to_string());

                // Log failed rotation
                self.log_rotation_event(&event).await?;

                tracing::error!("Key rotation failed for {}: {}", key_id, e);

                Err(e)
            }
        }
    }

    /// Perform key rotation via Secreton API
    async fn perform_secreton_rotation(&self, key_id: &str, key_type: &KeyType) -> Result<u32> {
        // Create security context for the rotation request
        let context = SecurityContext {
            session_id: Some("system-rotation".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Authenc-KeyRotation/1.0".to_string()),
            timestamp: chrono::Utc::now(),
            risk_score: Some(0.0),
            metadata: Some(serde_json::json!({
                "operation": "key_rotation",
                "key_type": key_type.to_string(),
            })),
        };

        // Call Secreton's RotateKey API
        match key_type {
            KeyType::JwtSigning => {
                // Rotate signing key
                let new_key = self
                    .secreton_client
                    .rotate_signing_key(key_id, &context)
                    .await
                    .map_err(|e| AuthencError::ExternalServiceError {
                        service: format!("Secreton rotation failed: {}", e),
                    })?;

                Ok(self.extract_version_from_key_id(&new_key.key_id))
            }
            KeyType::SessionEncryption | KeyType::MfaEncryption | KeyType::DatabaseEncryption => {
                // Rotate encryption key
                let new_key = self
                    .secreton_client
                    .rotate_encryption_key(key_id, &context)
                    .await
                    .map_err(|e| AuthencError::ExternalServiceError {
                        service: format!("Secreton rotation failed: {}", e),
                    })?;

                Ok(self.extract_version_from_key_id(&new_key.key_id))
            }
            KeyType::Generic => {
                // Generic key rotation
                Err(AuthencError::ValidationError {
                    message: "Generic key rotation not implemented".to_string(),
                })
            }
        }
    }

    /// Extract version number from key ID (assumes format: key_name_v{version})
    fn extract_version_from_key_id(&self, key_id: &str) -> u32 {
        key_id
            .rsplit('_')
            .next()
            .and_then(|v| v.strip_prefix('v'))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1)
    }

    /// Get current version of a key
    pub async fn get_key_version(&self, key_id: &str) -> Result<u32> {
        let metadata = self.key_metadata.read().await;
        metadata
            .iter()
            .find(|k| k.key_id == key_id)
            .map(|k| k.version)
            .ok_or_else(|| AuthencError::ResourceNotFound {
                resource: format!("Key: {}", key_id),
            })
    }

    /// Update key metadata after rotation
    async fn update_key_metadata(&self, key_id: &str, new_version: u32) -> Result<()> {
        let mut metadata = self.key_metadata.write().await;

        if let Some(key) = metadata.iter_mut().find(|k| k.key_id == key_id) {
            key.version = new_version;
            key.last_rotated = Utc::now();
            key.next_rotation =
                Utc::now() + Duration::days(self.config.rotation_interval_days as i64);
        }

        Ok(())
    }

    /// Log rotation event to database
    async fn log_rotation_event(&self, event: &KeyRotationEvent) -> Result<()> {
        let query = r#"
            INSERT INTO key_rotation_audit (
                id, timestamp, key_id, key_type, old_version, new_version,
                status, error_message, initiated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#;

        self.database
            .execute(
                query,
                &[
                    &event.id,
                    &event.timestamp,
                    &event.key_id,
                    &event.key_type.to_string(),
                    &(event.old_version as i32),
                    &(event.new_version as i32),
                    &format!("{:?}", event.status),
                    &event.error_message,
                    &event.initiated_by,
                ],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to log rotation event: {}", e)))?;

        Ok(())
    }

    /// Send notification about key rotation completion
    async fn send_rotation_notification(&self, event: &KeyRotationEvent) -> Result<()> {
        // TODO: Integrate with notification service (email, Slack, etc.)
        tracing::info!(
            "Key rotation notification: {} rotated from v{} to v{} - Status: {:?}",
            event.key_id,
            event.old_version,
            event.new_version,
            event.status
        );

        Ok(())
    }

    /// Register a key for automatic rotation
    pub async fn register_key(
        &self,
        key_id: String,
        key_type: KeyType,
        current_version: u32,
    ) -> Result<()> {
        let now = Utc::now();
        let metadata = KeyMetadata {
            key_id: key_id.clone(),
            key_type,
            version: current_version,
            last_rotated: now,
            next_rotation: now + Duration::days(self.config.rotation_interval_days as i64),
            is_active: true,
        };

        let mut keys = self.key_metadata.write().await;
        keys.push(metadata);

        tracing::info!("Registered key {} for automatic rotation", key_id);

        Ok(())
    }

    /// Unregister a key from automatic rotation
    pub async fn unregister_key(&self, key_id: &str) -> Result<()> {
        let mut keys = self.key_metadata.write().await;
        keys.retain(|k| k.key_id != key_id);

        tracing::info!("Unregistered key {} from automatic rotation", key_id);

        Ok(())
    }

    /// Get rotation history for a key
    pub async fn get_rotation_history(
        &self,
        key_id: &str,
        limit: i64,
    ) -> Result<Vec<KeyRotationEvent>> {
        let query = r#"
            SELECT id, timestamp, key_id, key_type, old_version, new_version,
                   status, error_message, initiated_by
            FROM key_rotation_audit
            WHERE key_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
        "#;

        let rows: Vec<tokio_postgres::Row> = self
            .database
            .query(query, &[&key_id, &limit])
            .await
            .map_err(|e| {
            AuthencError::database(format!("Failed to fetch rotation history: {}", e))
        })?;

        let mut events = Vec::new();
        for row in rows {
            let status_str: String = row.get(6);
            let status = match status_str.as_str() {
                "Success" => RotationStatus::Success,
                "Failed" => RotationStatus::Failed,
                "InProgress" => RotationStatus::InProgress,
                "Scheduled" => RotationStatus::Scheduled,
                _ => RotationStatus::Failed,
            };

            let key_type_str: String = row.get(3);
            let key_type = match key_type_str.as_str() {
                "jwt_signing" => KeyType::JwtSigning,
                "session_encryption" => KeyType::SessionEncryption,
                "mfa_encryption" => KeyType::MfaEncryption,
                "database_encryption" => KeyType::DatabaseEncryption,
                _ => KeyType::Generic,
            };

            events.push(KeyRotationEvent {
                id: row.get(0),
                timestamp: row.get(1),
                key_id: row.get(2),
                key_type,
                old_version: row.get::<_, i32>(4) as u32,
                new_version: row.get::<_, i32>(5) as u32,
                status,
                error_message: row.get(7),
                initiated_by: row.get(8),
            });
        }

        Ok(events)
    }
}

impl Clone for KeyRotationService {
    fn clone(&self) -> Self {
        Self {
            secreton_client: self.secreton_client.clone(),
            database: self.database.clone(),
            config: self.config.clone(),
            key_metadata: self.key_metadata.clone(),
            task_handle: Arc::new(RwLock::new(None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_rotation_config_default() {
        let config = KeyRotationConfig::default();
        assert_eq!(config.rotation_interval_days, 30);
        assert_eq!(config.grace_period_days, 7);
        assert!(config.enabled);
        assert!(config.enable_notifications);
    }

    #[test]
    fn test_key_type_display() {
        assert_eq!(KeyType::JwtSigning.to_string(), "jwt_signing");
        assert_eq!(KeyType::SessionEncryption.to_string(), "session_encryption");
        assert_eq!(KeyType::MfaEncryption.to_string(), "mfa_encryption");
    }

    #[tokio::test]
    async fn test_extract_version_from_key_id() {
        let service = KeyRotationService {
            secreton_client: Arc::new(SecretonClient::new(
                "http://localhost:8200".to_string(),
                "token".to_string(),
            )),
            // Use mock database to avoid external dependency in unit test
            database: Arc::new(crate::database::Database::mock().await),
            config: KeyRotationConfig::default(),
            key_metadata: Arc::new(RwLock::new(Vec::new())),
            task_handle: Arc::new(RwLock::new(None)),
        };

        assert_eq!(service.extract_version_from_key_id("jwt_signing_v1"), 1);
        assert_eq!(service.extract_version_from_key_id("jwt_signing_v42"), 42);
        assert_eq!(service.extract_version_from_key_id("invalid_key"), 1);
    }
}
