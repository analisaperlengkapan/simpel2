//! Backup codes generation and validation
//!
//! Provides one-time use backup codes for MFA recovery.

use async_trait::async_trait;
use authenc_types::{AuthencError, Result, UserId};
use chrono::{DateTime, Utc};
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Backup code configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodesConfig {
    /// Number of backup codes to generate (default: 10)
    pub count: usize,
    /// Length of each backup code (default: 8)
    pub length: usize,
}

impl Default for BackupCodesConfig {
    fn default() -> Self {
        Self {
            count: 10,
            length: 8,
        }
    }
}

/// Backup code with usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCode {
    /// The backup code value
    pub code: String,
    /// Whether the code has been used
    pub used: bool,
    /// When the code was used (if used)
    pub used_at: Option<DateTime<Utc>>,
    /// When the code was created
    pub created_at: DateTime<Utc>,
}

impl BackupCode {
    /// Create a new unused backup code
    pub fn new(code: String) -> Self {
        Self {
            code,
            used: false,
            used_at: None,
            created_at: Utc::now(),
        }
    }

    /// Mark the code as used
    pub fn mark_used(&mut self) {
        self.used = true;
        self.used_at = Some(Utc::now());
    }
}

/// Backup codes set for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodesSet {
    /// User ID
    pub user_id: UserId,
    /// List of backup codes
    pub codes: Vec<BackupCode>,
    /// When the codes were generated
    pub generated_at: DateTime<Utc>,
}

/// Trait for backup codes storage
#[async_trait]
pub trait BackupCodesStore: Send + Sync {
    /// Store backup codes for a user
    async fn store_backup_codes(&self, user_id: UserId, codes: &[BackupCode]) -> Result<()>;

    /// Retrieve backup codes for a user
    async fn get_backup_codes(&self, user_id: UserId) -> Result<Option<Vec<BackupCode>>>;

    /// Update a specific backup code (mark as used)
    async fn update_backup_code(&self, user_id: UserId, code: &str, used: bool) -> Result<()>;

    /// Delete all backup codes for a user
    async fn delete_backup_codes(&self, user_id: UserId) -> Result<()>;
}

/// Backup codes service
pub struct BackupCodesService<S: BackupCodesStore> {
    config: BackupCodesConfig,
    store: Arc<S>,
}

impl<S: BackupCodesStore> BackupCodesService<S> {
    /// Create a new backup codes service
    pub fn new(config: BackupCodesConfig, store: Arc<S>) -> Self {
        Self { config, store }
    }

    /// Generate a single backup code
    fn generate_code(&self) -> String {
        let mut rng = rand::thread_rng();

        // Generate alphanumeric code
        let code: String = (0..self.config.length)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect();

        // Format with dashes for readability (e.g., ABCD-EFGH)
        if self.config.length >= 8 {
            format!(
                "{}-{}",
                &code[..4].to_uppercase(),
                &code[4..].to_uppercase()
            )
        } else {
            code.to_uppercase()
        }
    }

    /// Generate backup codes for a user
    pub async fn generate_backup_codes(&self, user_id: UserId) -> Result<Vec<String>> {
        info!(
            "Generating {} backup codes for user: {}",
            self.config.count, user_id
        );

        // Generate codes
        let codes: Vec<BackupCode> = (0..self.config.count)
            .map(|_| BackupCode::new(self.generate_code()))
            .collect();

        // Store codes
        self.store.store_backup_codes(user_id, &codes).await?;

        // Return code values (without metadata)
        let code_values: Vec<String> = codes.iter().map(|c| c.code.clone()).collect();

        info!("Backup codes generated successfully for user: {}", user_id);

        Ok(code_values)
    }

    /// Verify a backup code
    pub async fn verify_backup_code(&self, user_id: UserId, code: &str) -> Result<bool> {
        debug!("Verifying backup code for user: {}", user_id);

        // Retrieve backup codes
        let codes = self
            .store
            .get_backup_codes(user_id)
            .await?
            .ok_or_else(|| AuthencError::not_found("Backup codes not found"))?;

        // Normalize input code (remove spaces AND dashes, uppercase)
        let normalized_code = code.replace(' ', "").replace('-', "").to_uppercase();

        // Find matching code (also normalize stored code)
        let matching_code = codes
            .iter()
            .find(|c| c.code.replace('-', "").replace(' ', "").to_uppercase() == normalized_code);

        match matching_code {
            Some(backup_code) if !backup_code.used => {
                // Mark code as used
                self.store
                    .update_backup_code(user_id, &backup_code.code, true)
                    .await?;

                info!("Backup code verified successfully for user: {}", user_id);
                Ok(true)
            }
            Some(_) => {
                warn!("Backup code already used for user: {}", user_id);
                Ok(false)
            }
            None => {
                warn!("Invalid backup code for user: {}", user_id);
                Ok(false)
            }
        }
    }

    /// Regenerate backup codes (invalidate old ones)
    pub async fn regenerate_backup_codes(&self, user_id: UserId) -> Result<Vec<String>> {
        info!("Regenerating backup codes for user: {}", user_id);

        // Delete old codes
        self.store.delete_backup_codes(user_id).await?;

        // Generate new codes
        self.generate_backup_codes(user_id).await
    }

    /// Get remaining backup codes count
    pub async fn get_remaining_codes_count(&self, user_id: UserId) -> Result<usize> {
        let codes = self.store.get_backup_codes(user_id).await?;

        match codes {
            Some(codes) => Ok(codes.iter().filter(|c| !c.used).count()),
            None => Ok(0),
        }
    }

    /// Check if user has backup codes
    pub async fn has_backup_codes(&self, user_id: UserId) -> Result<bool> {
        let codes = self.store.get_backup_codes(user_id).await?;
        Ok(codes.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    /// Mock backup codes store for testing
    struct MockBackupCodesStore {
        codes: Arc<RwLock<HashMap<UserId, Vec<BackupCode>>>>,
    }

    impl MockBackupCodesStore {
        fn new() -> Self {
            Self {
                codes: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl BackupCodesStore for MockBackupCodesStore {
        async fn store_backup_codes(&self, user_id: UserId, codes: &[BackupCode]) -> Result<()> {
            self.codes.write().await.insert(user_id, codes.to_vec());
            Ok(())
        }

        async fn get_backup_codes(&self, user_id: UserId) -> Result<Option<Vec<BackupCode>>> {
            Ok(self.codes.read().await.get(&user_id).cloned())
        }

        async fn update_backup_code(&self, user_id: UserId, code: &str, used: bool) -> Result<()> {
            if let Some(codes) = self.codes.write().await.get_mut(&user_id) {
                if let Some(backup_code) = codes.iter_mut().find(|c| c.code == code) {
                    backup_code.used = used;
                    if used {
                        backup_code.used_at = Some(Utc::now());
                    }
                }
            }
            Ok(())
        }

        async fn delete_backup_codes(&self, user_id: UserId) -> Result<()> {
            self.codes.write().await.remove(&user_id);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_generate_backup_codes() {
        let store = Arc::new(MockBackupCodesStore::new());
        let service = BackupCodesService::new(BackupCodesConfig::default(), store.clone());

        let user_id = UserId::new();
        let codes = service.generate_backup_codes(user_id).await.unwrap();

        // Verify count
        assert_eq!(codes.len(), 10);

        // Verify format (XXXX-XXXX)
        for code in &codes {
            assert_eq!(code.len(), 9); // 4 + 1 (dash) + 4
            assert!(code.contains('-'));
        }

        // Verify codes are stored
        let stored_codes = store.get_backup_codes(user_id).await.unwrap().unwrap();
        assert_eq!(stored_codes.len(), 10);
    }

    #[tokio::test]
    async fn test_verify_backup_code() {
        let store = Arc::new(MockBackupCodesStore::new());
        let service = BackupCodesService::new(BackupCodesConfig::default(), store.clone());

        let user_id = UserId::new();
        let codes = service.generate_backup_codes(user_id).await.unwrap();

        // Verify valid code
        let is_valid = service
            .verify_backup_code(user_id, &codes[0])
            .await
            .unwrap();
        assert!(is_valid);

        // Verify code cannot be reused
        let is_valid = service
            .verify_backup_code(user_id, &codes[0])
            .await
            .unwrap();
        assert!(!is_valid);

        // Verify invalid code
        let is_valid = service
            .verify_backup_code(user_id, "INVALID-CODE")
            .await
            .unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_regenerate_backup_codes() {
        let store = Arc::new(MockBackupCodesStore::new());
        let service = BackupCodesService::new(BackupCodesConfig::default(), store.clone());

        let user_id = UserId::new();
        let old_codes = service.generate_backup_codes(user_id).await.unwrap();

        // Regenerate codes
        let new_codes = service.regenerate_backup_codes(user_id).await.unwrap();

        // Verify new codes are different
        assert_ne!(old_codes, new_codes);

        // Verify old codes are invalid
        let is_valid = service
            .verify_backup_code(user_id, &old_codes[0])
            .await
            .unwrap();
        assert!(!is_valid);

        // Verify new codes are valid
        let is_valid = service
            .verify_backup_code(user_id, &new_codes[0])
            .await
            .unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_remaining_codes_count() {
        let store = Arc::new(MockBackupCodesStore::new());
        let service = BackupCodesService::new(BackupCodesConfig::default(), store.clone());

        let user_id = UserId::new();
        let codes = service.generate_backup_codes(user_id).await.unwrap();

        // Initially all codes are unused
        let count = service.get_remaining_codes_count(user_id).await.unwrap();
        assert_eq!(count, 10);

        // Use one code
        service
            .verify_backup_code(user_id, &codes[0])
            .await
            .unwrap();

        // Verify count decreased
        let count = service.get_remaining_codes_count(user_id).await.unwrap();
        assert_eq!(count, 9);
    }

    #[tokio::test]
    async fn test_code_normalization() {
        let store = Arc::new(MockBackupCodesStore::new());
        let service = BackupCodesService::new(BackupCodesConfig::default(), store.clone());

        let user_id = UserId::new();
        let codes = service.generate_backup_codes(user_id).await.unwrap();

        // Test with spaces
        let code_with_spaces = codes[0].replace('-', " ");
        let is_valid = service
            .verify_backup_code(user_id, &code_with_spaces)
            .await
            .unwrap();
        assert!(is_valid);

        // Test with lowercase
        let code_lowercase = codes[1].to_lowercase();
        let is_valid = service
            .verify_backup_code(user_id, &code_lowercase)
            .await
            .unwrap();
        assert!(is_valid);
    }
}
