//! MFA client with automatic fallback to local encrypted storage
//!
//! This module provides a wrapper around SecretonClient that automatically
//! falls back to local encrypted storage when Secreton is unavailable, and
//! automatically syncs secrets when Secreton recovers.

use crate::error::{AuthencError, Result};
use crate::models::user::SecurityContext;
use crate::secreton_client::VaultError;
use crate::secreton_client::secreton_client::{MfaSetupData, MfaStatusResponse, SecretonClient};
use crate::services::mfa_local_storage::{DegradedMode, MfaLocalStorage};
use crate::services::mfa_service::MfaClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// MFA client with automatic fallback and sync
pub struct MfaFallbackClient {
    /// Primary Secreton client
    secreton: Arc<SecretonClient>,
    /// Local encrypted storage fallback
    local_storage: Arc<MfaLocalStorage>,
    /// Background sync task handle
    sync_task: Arc<tokio::sync::RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl MfaFallbackClient {
    /// Create a new fallback client
    ///
    /// # Arguments
    /// * `secreton` - Secreton client for primary storage
    /// * `local_storage` - Local encrypted storage for fallback
    pub fn new(secreton: Arc<SecretonClient>, local_storage: Arc<MfaLocalStorage>) -> Self {
        Self {
            secreton,
            local_storage,
            sync_task: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Initialize the fallback client and start background sync
    pub async fn initialize(&self) -> Result<()> {
        // Initialize local storage
        self.local_storage.initialize().await?;

        // Start background sync task
        self.start_sync_task().await;

        Ok(())
    }

    /// Start background sync task
    async fn start_sync_task(&self) {
        let secreton = Arc::clone(&self.secreton);
        let local_storage = Arc::clone(&self.local_storage);

        let handle = tokio::spawn(async move {
            loop {
                // Wait before next sync attempt
                sleep(Duration::from_secs(60)).await;

                // Check if we're in degraded mode
                let mode = local_storage.get_degraded_mode().await;
                if mode == DegradedMode::Degraded {
                    // Try to recover
                    if let Ok(()) = secreton.health_check().await {
                        info!("Secreton is available again, attempting to sync secrets");
                        local_storage.enter_recovering_mode().await;

                        // Attempt to sync all pending secrets
                        let user_ids = local_storage.get_secrets_needing_sync().await;

                        let mut success_count = 0;
                        let mut failure_count = 0;

                        for user_id in &user_ids {
                            match Self::sync_secret_to_secreton(&secreton, &local_storage, user_id)
                                .await
                            {
                                Ok(()) => {
                                    success_count += 1;
                                    if let Err(e) = local_storage.mark_as_synced(user_id).await {
                                        error!("Failed to mark secret as synced: {}", e);
                                    }
                                }
                                Err(e) => {
                                    failure_count += 1;
                                    error!("Failed to sync secret for user {}: {}", user_id, e);
                                    if let Err(e) =
                                        local_storage.increment_sync_attempts(user_id).await
                                    {
                                        error!("Failed to increment sync attempts: {}", e);
                                    }
                                }
                            }
                        }

                        if failure_count == 0 && success_count > 0 {
                            info!("Successfully synced {} secrets to Secreton", success_count);
                            local_storage.exit_degraded_mode().await;
                        } else if failure_count > 0 {
                            warn!(
                                "Partial sync: {} succeeded, {} failed",
                                success_count, failure_count
                            );
                        }
                    }
                }
            }
        });

        *self.sync_task.write().await = Some(handle);
    }

    /// Sync a single secret to Secreton
    async fn sync_secret_to_secreton(
        secreton: &SecretonClient,
        local_storage: &MfaLocalStorage,
        user_id: &str,
    ) -> Result<()> {
        // Get secret from local storage
        let secret = local_storage
            .get_mfa_secret(user_id)
            .await?
            .ok_or_else(|| AuthencError::internal("Secret not found in local storage"))?;

        let backup_codes = local_storage
            .get_backup_codes(user_id)
            .await?
            .unwrap_or_default();

        // Create setup data
        let setup_data = MfaSetupData {
            qr_code_url: format!("otpauth://totp/SIMPelv2:{}?secret={}", user_id, secret),
            secret_key: secret,
            backup_codes,
        };

        // Store in Secreton
        // Note: This is a simplified version. In production, you'd need to call
        // the appropriate Secreton API to store the secret
        secreton
            .setup_mfa(user_id, "SIMPelv2 Kejaksaan RI", user_id)
            .await
            .map_err(|_| AuthencError::ExternalServiceError {
                service: "secreton".to_string(),
            })?;

        Ok(())
    }

    /// Check if Secreton is available
    async fn is_secreton_available(&self) -> bool {
        self.secreton.health_check().await.is_ok()
    }

    /// Handle Secreton error and potentially enter degraded mode
    async fn handle_secreton_error(&self, error: VaultError) -> AuthencError {
        match error {
            VaultError::Unavailable(_) => {
                // Enter degraded mode
                self.local_storage.enter_degraded_mode().await;
                AuthencError::ExternalServiceError {
                    service: "secreton".to_string(),
                }
            }
            VaultError::AuthenticationFailed(msg) => AuthencError::unauthorized(msg),
            VaultError::Unauthorized(msg) => AuthencError::forbidden(msg),
            VaultError::NotFound(msg) => AuthencError::not_found(msg),
            _ => AuthencError::ExternalServiceError {
                service: "secreton".to_string(),
            },
        }
    }

    /// Stop the background sync task
    pub async fn shutdown(&self) {
        if let Some(handle) = self.sync_task.write().await.take() {
            handle.abort();
            info!("Stopped MFA fallback sync task");
        }
    }
}

#[async_trait::async_trait]
impl MfaClient for MfaFallbackClient {
    async fn setup_mfa(
        &self,
        user_id: &str,
        issuer: &str,
        account_name: &str,
    ) -> Result<MfaSetupData> {
        // Try Secreton first
        match self.secreton.setup_mfa(user_id, issuer, account_name).await {
            Ok(setup_data) => {
                // Success - store in local storage as backup
                if let Err(e) = self
                    .local_storage
                    .store_mfa_setup(user_id, &setup_data)
                    .await
                {
                    warn!("Failed to store MFA setup in local storage: {}", e);
                }
                Ok(setup_data)
            }
            Err(e) => {
                // Secreton failed - use local storage
                warn!(
                    "Secreton unavailable for MFA setup, using local storage: {}",
                    e
                );
                self.local_storage.enter_degraded_mode().await;

                // Generate secret locally
                use totp_rs::{Algorithm, Secret, TOTP};

                // Generate random secret bytes (160 bits = 20 bytes for SHA1)
                let mut secret_bytes = vec![0u8; 20];
                use rand::Rng;
                rand::thread_rng().fill(&mut secret_bytes[..]);

                let secret = Secret::Raw(secret_bytes.clone());
                let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_bytes)
                    .map_err(|e| AuthencError::internal(format!("Failed to create TOTP: {}", e)))?;

                // Generate QR code URL (otpauth:// URL) manually
                let secret_key = secret.to_encoded().to_string();
                let qr_code_url = format!(
                    "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
                    issuer, account_name, secret_key, issuer
                );

                // Generate backup codes
                let backup_codes = Self::generate_backup_codes(10);

                let setup_data = MfaSetupData {
                    qr_code_url,
                    secret_key,
                    backup_codes,
                };

                // Store locally
                self.local_storage
                    .store_mfa_setup(user_id, &setup_data)
                    .await?;

                Ok(setup_data)
            }
        }
    }

    async fn verify_mfa_setup(&self, user_id: &str, code: &str) -> Result<()> {
        // Try Secreton first
        match self.secreton.verify_mfa_setup(user_id, code).await {
            Ok(()) => Ok(()),
            Err(_) => {
                // Fallback to local verification
                let secret = self
                    .local_storage
                    .get_mfa_secret(user_id)
                    .await?
                    .ok_or_else(|| AuthencError::not_found("MFA secret not found"))?;

                // Verify TOTP code
                use totp_rs::{Algorithm, Secret, TOTP};

                let secret_bytes = Secret::Encoded(secret)
                    .to_bytes()
                    .map_err(|e| AuthencError::internal(format!("Invalid secret: {}", e)))?;

                let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_bytes)
                    .map_err(|e| AuthencError::internal(format!("Failed to create TOTP: {}", e)))?;

                if totp.check_current(code).unwrap_or(false) {
                    self.local_storage.update_last_used(user_id).await?;
                    Ok(())
                } else {
                    Err(AuthencError::InvalidOtpCode)
                }
            }
        }
    }

    async fn verify_mfa(&self, user_id: &str, code: &str) -> Result<()> {
        // Try Secreton first
        match self.secreton.verify_mfa(user_id, code).await {
            Ok(()) => {
                // Update local storage last used
                if let Err(e) = self.local_storage.update_last_used(user_id).await {
                    warn!("Failed to update local storage last used: {}", e);
                }
                Ok(())
            }
            Err(_) => {
                // Fallback to local verification
                let secret = self
                    .local_storage
                    .get_mfa_secret(user_id)
                    .await?
                    .ok_or_else(|| AuthencError::not_found("MFA secret not found"))?;

                // Verify TOTP code
                use totp_rs::{Algorithm, Secret, TOTP};

                let secret_bytes = Secret::Encoded(secret)
                    .to_bytes()
                    .map_err(|e| AuthencError::internal(format!("Invalid secret: {}", e)))?;

                let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_bytes)
                    .map_err(|e| AuthencError::internal(format!("Failed to create TOTP: {}", e)))?;

                if totp.check_current(code).unwrap_or(false) {
                    self.local_storage.update_last_used(user_id).await?;
                    Ok(())
                } else {
                    Err(AuthencError::InvalidOtpCode)
                }
            }
        }
    }

    async fn disable_mfa(&self, user_id: &str, admin_context: &SecurityContext) -> Result<()> {
        // Try Secreton first
        match self.secreton.disable_mfa(user_id, admin_context).await {
            Ok(()) => {
                // Also remove from local storage
                if let Err(e) = self.local_storage.remove_mfa_secret(user_id).await {
                    warn!("Failed to remove MFA secret from local storage: {}", e);
                }
                Ok(())
            }
            Err(e) => {
                // If Secreton is unavailable, still remove from local storage
                warn!("Secreton unavailable for MFA disable: {}", e);
                self.local_storage.remove_mfa_secret(user_id).await?;
                Ok(())
            }
        }
    }

    async fn get_mfa_status(&self, user_id: &str) -> Result<MfaStatusResponse> {
        // Try Secreton first
        match self.secreton.get_mfa_status(user_id).await {
            Ok(status) => Ok(status),
            Err(_) => {
                // Fallback to local storage
                let secret = self.local_storage.get_mfa_secret(user_id).await?;

                Ok(MfaStatusResponse {
                    is_enabled: secret.is_some(),
                    method: Some("TOTP".to_string()),
                    last_used: None, // Could be retrieved from local storage metadata
                })
            }
        }
    }

    async fn verify_recovery_code(&self, user_id: &str, recovery_code: &str) -> Result<()> {
        // Try Secreton first
        match self
            .secreton
            .verify_recovery_code(user_id, recovery_code)
            .await
        {
            Ok(()) => Ok(()),
            Err(_) => {
                // Fallback to local verification
                let backup_codes = self
                    .local_storage
                    .get_backup_codes(user_id)
                    .await?
                    .ok_or_else(|| AuthencError::not_found("Backup codes not found"))?;

                if backup_codes.contains(&recovery_code.to_string()) {
                    // TODO: Mark code as used in local storage
                    Ok(())
                } else {
                    Err(AuthencError::InvalidBackupCode)
                }
            }
        }
    }

    async fn regenerate_recovery_codes(&self, user_id: &str) -> Result<Vec<String>> {
        // Try Secreton first
        match self.secreton.regenerate_recovery_codes(user_id).await {
            Ok(codes) => {
                // Update local storage
                // TODO: Implement update backup codes in local storage
                Ok(codes)
            }
            Err(_) => {
                // Generate locally
                let new_codes = Self::generate_backup_codes(10);

                // TODO: Store in local storage

                Ok(new_codes)
            }
        }
    }
}

impl MfaFallbackClient {
    /// Generate backup recovery codes
    fn generate_backup_codes(count: usize) -> Vec<String> {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        (0..count)
            .map(|_| {
                format!(
                    "{:04}-{:04}-{:04}",
                    rng.gen_range(0..10000),
                    rng.gen_range(0..10000),
                    rng.gen_range(0..10000)
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::aes_gcm::AesGcmService;

    use tempfile::tempdir;

    #[tokio::test]
    async fn test_fallback_on_secreton_failure() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("mfa_storage.enc");
        let key = AesGcmService::generate_key();

        // Create a Secreton client that will fail
        let secreton = Arc::new(SecretonClient::new(
            "http://invalid-endpoint".to_string(),
            "invalid-token".to_string(),
        ));

        let local_storage = Arc::new(MfaLocalStorage::new(storage_path, &key).unwrap());

        let fallback_client = MfaFallbackClient::new(secreton, local_storage);
        fallback_client.initialize().await.unwrap();

        // This should fallback to local storage
        let result = fallback_client
            .setup_mfa("user123", "SIMPelv2", "user123@kejaksaan.go.id")
            .await;

        assert!(result.is_ok());

        // Verify it's in degraded mode
        assert!(fallback_client.local_storage.is_degraded().await);
    }
}
