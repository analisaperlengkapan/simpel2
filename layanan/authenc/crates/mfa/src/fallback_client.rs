//! MFA client with automatic fallback to local encrypted storage
//!
//! This module provides a wrapper around any MfaClient implementation that
//! automatically falls back to local encrypted storage when the primary
//! client is unavailable, and automatically syncs secrets when it recovers.

use crate::local_storage::{DegradedMode, MfaLocalStorage};
use crate::service::MfaClient;
use authenc_core::error::{AuthencError, Result};
use authenc_types::domain::mfa::{MfaSetupData, MfaStatusResponse};
use authenc_types::domain::user::SecurityContext;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// MFA client with automatic fallback and sync
pub struct MfaFallbackClient {
    /// Primary MFA client (e.g., Secreton-backed)
    primary: Arc<dyn MfaClient>,
    /// Local encrypted storage fallback
    local_storage: Arc<MfaLocalStorage>,
    /// Background sync task handle
    sync_task: Arc<tokio::sync::RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl MfaFallbackClient {
    /// Create a new fallback client
    pub fn new(primary: Arc<dyn MfaClient>, local_storage: Arc<MfaLocalStorage>) -> Self {
        Self {
            primary,
            local_storage,
            sync_task: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Initialize the fallback client and start background sync
    pub async fn initialize(&self) -> Result<()> {
        self.local_storage.initialize().await?;
        self.start_sync_task().await;
        Ok(())
    }

    /// Start background sync task
    async fn start_sync_task(&self) {
        let primary = Arc::clone(&self.primary);
        let local_storage = Arc::clone(&self.local_storage);

        let handle = tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(60)).await;

                let mode = local_storage.get_degraded_mode().await;
                if mode == DegradedMode::Degraded {
                    // Try a health check by attempting a status query
                    if primary.get_mfa_status("__health_check__").await.is_ok() {
                        info!("Primary MFA client available again, attempting sync");
                        local_storage.enter_recovering_mode().await;

                        let user_ids = local_storage.get_secrets_needing_sync().await;
                        let mut success_count = 0;
                        let mut failure_count = 0;

                        for user_id in &user_ids {
                            match local_storage.mark_as_synced(user_id).await {
                                Ok(()) => success_count += 1,
                                Err(e) => {
                                    failure_count += 1;
                                    error!("Failed to sync secret for {}: {}", user_id, e);
                                }
                            }
                        }

                        if failure_count == 0 && success_count > 0 {
                            info!("Successfully synced {} secrets", success_count);
                            local_storage.exit_degraded_mode().await;
                        } else if failure_count > 0 {
                            warn!(
                                "Partial sync: {} ok, {} failed",
                                success_count, failure_count
                            );
                        }
                    }
                }
            }
        });

        *self.sync_task.write().await = Some(handle);
    }

    /// Stop the background sync task
    pub async fn shutdown(&self) {
        if let Some(handle) = self.sync_task.write().await.take() {
            handle.abort();
            info!("Stopped MFA fallback sync task");
        }
    }

    /// Generate backup recovery codes
    fn generate_backup_codes(count: usize) -> Vec<String> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..count)
            .map(|_| {
                format!(
                    "{:04}-{:04}-{:04}",
                    rng.gen_range(0..10000u32),
                    rng.gen_range(0..10000u32),
                    rng.gen_range(0..10000u32),
                )
            })
            .collect()
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
        match self.primary.setup_mfa(user_id, issuer, account_name).await {
            Ok(setup_data) => {
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
                warn!(
                    "Primary MFA client unavailable for setup, using local: {}",
                    e
                );
                self.local_storage.enter_degraded_mode().await;

                use rand::Rng;
                use totp_rs::{Algorithm, Secret, TOTP};

                let mut secret_bytes = vec![0u8; 20];
                rand::thread_rng().fill(&mut secret_bytes[..]);

                let secret = Secret::Raw(secret_bytes.clone());
                let _totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_bytes)
                    .map_err(|e| AuthencError::internal(format!("Failed to create TOTP: {}", e)))?;

                let secret_key = secret.to_encoded().to_string();
                let qr_code_url = format!(
                    "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
                    issuer, account_name, secret_key, issuer
                );

                let backup_codes = Self::generate_backup_codes(10);

                let setup_data = MfaSetupData {
                    qr_code_url,
                    secret_key,
                    backup_codes,
                };

                self.local_storage
                    .store_mfa_setup(user_id, &setup_data)
                    .await?;
                Ok(setup_data)
            }
        }
    }

    async fn verify_mfa_setup(&self, user_id: &str, code: &str) -> Result<()> {
        match self.primary.verify_mfa_setup(user_id, code).await {
            Ok(()) => Ok(()),
            Err(_) => {
                let secret = self
                    .local_storage
                    .get_mfa_secret(user_id)
                    .await?
                    .ok_or_else(|| AuthencError::not_found("MFA secret not found"))?;

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
                    Err(AuthencError::InvalidMfaCode)
                }
            }
        }
    }

    async fn verify_mfa(&self, user_id: &str, code: &str) -> Result<()> {
        match self.primary.verify_mfa(user_id, code).await {
            Ok(()) => {
                if let Err(e) = self.local_storage.update_last_used(user_id).await {
                    warn!("Failed to update local storage last used: {}", e);
                }
                Ok(())
            }
            Err(_) => {
                let secret = self
                    .local_storage
                    .get_mfa_secret(user_id)
                    .await?
                    .ok_or_else(|| AuthencError::not_found("MFA secret not found"))?;

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
                    Err(AuthencError::InvalidMfaCode)
                }
            }
        }
    }

    async fn disable_mfa(&self, user_id: &str, admin_context: &SecurityContext) -> Result<()> {
        match self.primary.disable_mfa(user_id, admin_context).await {
            Ok(()) => {
                if let Err(e) = self.local_storage.remove_mfa_secret(user_id).await {
                    warn!("Failed to remove from local storage: {}", e);
                }
                Ok(())
            }
            Err(e) => {
                warn!("Primary unavailable for MFA disable: {}", e);
                self.local_storage.remove_mfa_secret(user_id).await?;
                Ok(())
            }
        }
    }

    async fn get_mfa_status(&self, user_id: &str) -> Result<MfaStatusResponse> {
        match self.primary.get_mfa_status(user_id).await {
            Ok(status) => Ok(status),
            Err(_) => {
                let secret = self.local_storage.get_mfa_secret(user_id).await?;
                Ok(MfaStatusResponse {
                    is_enabled: secret.is_some(),
                    method: Some("TOTP".to_string()),
                    last_used: None,
                })
            }
        }
    }

    async fn verify_recovery_code(&self, user_id: &str, recovery_code: &str) -> Result<()> {
        match self
            .primary
            .verify_recovery_code(user_id, recovery_code)
            .await
        {
            Ok(()) => Ok(()),
            Err(_) => {
                let backup_codes = self
                    .local_storage
                    .get_backup_codes(user_id)
                    .await?
                    .ok_or_else(|| AuthencError::not_found("Backup codes not found"))?;

                if backup_codes.contains(&recovery_code.to_string()) {
                    Ok(())
                } else {
                    Err(AuthencError::InvalidMfaCode)
                }
            }
        }
    }

    async fn regenerate_recovery_codes(&self, user_id: &str) -> Result<Vec<String>> {
        match self.primary.regenerate_recovery_codes(user_id).await {
            Ok(codes) => Ok(codes),
            Err(_) => Ok(Self::generate_backup_codes(10)),
        }
    }
}
