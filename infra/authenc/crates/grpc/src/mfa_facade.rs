//! MFA Service Facade
//!
//! Provides a unified interface for MFA operations, integrating TOTP and backup codes services.

use crate::service::{MfaServiceFacade, MfaSetupResponse};
use authenc_mfa::{
    BackupCodesConfig, BackupCodesService, BackupCodesStore, TotpConfig, TotpService, TotpStore,
};
use authenc_types::{Result, UserId};
use std::sync::Arc;
use tracing::{debug, info};

/// Concrete implementation of MFA service facade
///
/// This implementation integrates authenc-mfa services (TOTP and backup codes)
/// and provides a unified interface for the gRPC service.
pub struct MfaServiceFacadeImpl<T: TotpStore, B: BackupCodesStore> {
    totp_service: Arc<TotpService<T>>,
    backup_codes_service: Arc<BackupCodesService<B>>,
}

impl<T: TotpStore, B: BackupCodesStore> MfaServiceFacadeImpl<T, B> {
    /// Create a new MFA service facade
    pub fn new(
        totp_service: Arc<TotpService<T>>,
        backup_codes_service: Arc<BackupCodesService<B>>,
    ) -> Self {
        Self {
            totp_service,
            backup_codes_service,
        }
    }

    /// Create with default configurations
    pub fn with_defaults(totp_store: Arc<T>, backup_codes_store: Arc<B>) -> Self {
        let totp_service = Arc::new(TotpService::new(TotpConfig::default(), totp_store));
        let backup_codes_service = Arc::new(BackupCodesService::new(
            BackupCodesConfig::default(),
            backup_codes_store,
        ));

        Self::new(totp_service, backup_codes_service)
    }
}

#[async_trait::async_trait]
impl<T: TotpStore, B: BackupCodesStore> MfaServiceFacade for MfaServiceFacadeImpl<T, B> {
    async fn setup_totp(&self, user_id: UserId, username: &str) -> Result<MfaSetupResponse> {
        info!("Setting up MFA for user: {}", user_id);

        // Setup TOTP
        let totp_setup = self.totp_service.setup_totp(user_id, username).await?;

        // Generate backup codes
        let backup_codes = self
            .backup_codes_service
            .generate_backup_codes(user_id)
            .await?;

        debug!(
            "MFA setup completed for user: {} (TOTP + {} backup codes)",
            user_id,
            backup_codes.len()
        );

        Ok(MfaSetupResponse {
            secret: totp_setup.secret,
            qr_code: totp_setup.qr_code_svg,
            backup_codes,
        })
    }

    async fn verify_totp(&self, user_id: UserId, code: &str) -> Result<bool> {
        debug!("Verifying TOTP code for user: {}", user_id);

        // Try TOTP verification first (but don't fail if TOTP is not set up)
        match self.totp_service.verify_totp(user_id, code).await {
            Ok(true) => {
                debug!("TOTP verification successful");
                return Ok(true);
            }
            Ok(false) => {
                // TOTP code is invalid, try backup code
                debug!("TOTP code invalid, trying backup code for user: {}", user_id);
            }
            Err(e) => {
                // TOTP verification failed (e.g., secret not found), try backup code
                debug!("TOTP verification error ({}), trying backup code for user: {}", e, user_id);
            }
        }

        // Try backup code
        match self.backup_codes_service.verify_backup_code(user_id, code).await {
            Ok(valid) => {
                debug!("Backup code verification result: {}", valid);
                Ok(valid)
            }
            Err(e) => {
                debug!("Backup code verification error: {}", e);
                Err(e)
            }
        }
    }

    async fn disable_totp(&self, user_id: UserId) -> Result<()> {
        info!("Disabling MFA for user: {}", user_id);

        // Disable TOTP
        self.totp_service.disable_totp(user_id).await?;

        // Delete backup codes
        self.backup_codes_service
            .regenerate_backup_codes(user_id)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authenc_types::UserId;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    // Mock stores for testing
    struct MockTotpStore {
        secrets: Arc<RwLock<HashMap<UserId, String>>>,
    }

    impl MockTotpStore {
        fn new() -> Self {
            Self {
                secrets: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl TotpStore for MockTotpStore {
        async fn store_totp_secret(&self, user_id: UserId, secret: &str) -> Result<()> {
            self.secrets
                .write()
                .await
                .insert(user_id, secret.to_string());
            Ok(())
        }

        async fn get_totp_secret(&self, user_id: UserId) -> Result<Option<String>> {
            Ok(self.secrets.read().await.get(&user_id).cloned())
        }

        async fn delete_totp_secret(&self, user_id: UserId) -> Result<()> {
            self.secrets.write().await.remove(&user_id);
            Ok(())
        }
    }

    struct MockBackupCodesStore {
        codes: Arc<RwLock<HashMap<UserId, Vec<authenc_mfa::BackupCode>>>>,
    }

    impl MockBackupCodesStore {
        fn new() -> Self {
            Self {
                codes: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl BackupCodesStore for MockBackupCodesStore {
        async fn store_backup_codes(
            &self,
            user_id: UserId,
            codes: &[authenc_mfa::BackupCode],
        ) -> Result<()> {
            self.codes
                .write()
                .await
                .insert(user_id, codes.to_vec());
            Ok(())
        }

        async fn get_backup_codes(
            &self,
            user_id: UserId,
        ) -> Result<Option<Vec<authenc_mfa::BackupCode>>> {
            Ok(self.codes.read().await.get(&user_id).cloned())
        }

        async fn update_backup_code(
            &self,
            user_id: UserId,
            code: &str,
            used: bool,
        ) -> Result<()> {
            if let Some(codes) = self.codes.write().await.get_mut(&user_id) {
                if let Some(backup_code) = codes.iter_mut().find(|c| c.code == code) {
                    backup_code.used = used;
                    if used {
                        backup_code.used_at = Some(chrono::Utc::now());
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
    async fn test_mfa_setup() {
        let totp_store = Arc::new(MockTotpStore::new());
        let backup_store = Arc::new(MockBackupCodesStore::new());
        let facade = MfaServiceFacadeImpl::with_defaults(totp_store, backup_store);

        let user_id = UserId::new();
        let response = facade
            .setup_totp(user_id, "test@example.com")
            .await
            .unwrap();

        // Verify response
        assert!(!response.secret.is_empty());
        assert!(!response.qr_code.is_empty());
        assert_eq!(response.backup_codes.len(), 10);
    }

    #[tokio::test]
    async fn test_totp_verification() {
        let totp_store = Arc::new(MockTotpStore::new());
        let backup_store = Arc::new(MockBackupCodesStore::new());
        let facade = MfaServiceFacadeImpl::with_defaults(totp_store, backup_store);

        let user_id = UserId::new();
        let response = facade
            .setup_totp(user_id, "test@example.com")
            .await
            .unwrap();

        // Generate current TOTP code
        use totp_rs::{Algorithm, Secret, TOTP};
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(response.secret).to_bytes().unwrap(),
        )
        .unwrap();

        let code = totp.generate_current().unwrap();

        // Verify TOTP code
        let is_valid = facade.verify_totp(user_id, &code).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_backup_code_verification() {
        // Arrange: Setup stores and services
        let totp_store = Arc::new(MockTotpStore::new());
        let backup_store = Arc::new(MockBackupCodesStore::new());
        let facade = MfaServiceFacadeImpl::with_defaults(totp_store, backup_store);

        let user_id = UserId::new();

        // Act: Setup MFA (generates TOTP + backup codes)
        let response = facade
            .setup_totp(user_id, "test@example.com")
            .await
            .expect("MFA setup should succeed");

        // Assert: Verify response structure
        assert!(!response.secret.is_empty(), "TOTP secret should be generated");
        assert!(!response.qr_code.is_empty(), "QR code should be generated");
        assert_eq!(response.backup_codes.len(), 10, "Should generate 10 backup codes");

        // Act: Verify first backup code
        let backup_code = &response.backup_codes[0];
        let is_valid = facade
            .verify_totp(user_id, backup_code)
            .await
            .expect("Backup code verification should not error");

        // Assert: First use should be valid
        assert!(is_valid, "First backup code should be valid");

        // Act: Try to reuse the same backup code
        let is_valid_reuse = facade
            .verify_totp(user_id, backup_code)
            .await
            .expect("Backup code verification should not error");

        // Assert: Reuse should fail
        assert!(!is_valid_reuse, "Backup code should not be reusable");

        // Act: Verify another backup code
        let second_code = &response.backup_codes[1];
        let is_valid_second = facade
            .verify_totp(user_id, second_code)
            .await
            .expect("Second backup code verification should not error");

        // Assert: Second code should be valid
        assert!(is_valid_second, "Second backup code should be valid");

        // Act: Test with invalid code
        let is_valid_invalid = facade
            .verify_totp(user_id, "INVALID-CODE")
            .await
            .expect("Invalid code verification should not error");

        // Assert: Invalid code should fail
        assert!(!is_valid_invalid, "Invalid backup code should fail");
    }

    #[tokio::test]
    async fn test_backup_code_normalization() {
        // Test that backup codes work with different formats
        let totp_store = Arc::new(MockTotpStore::new());
        let backup_store = Arc::new(MockBackupCodesStore::new());
        let facade = MfaServiceFacadeImpl::with_defaults(totp_store, backup_store);

        let user_id = UserId::new();
        let response = facade
            .setup_totp(user_id, "test@example.com")
            .await
            .unwrap();

        let backup_code = &response.backup_codes[0];

        // Test with original format (XXXX-XXXX)
        let is_valid = facade.verify_totp(user_id, backup_code).await.unwrap();
        assert!(is_valid, "Original format should work");

        // Test with lowercase
        let user_id2 = UserId::new();
        let response2 = facade.setup_totp(user_id2, "test2@example.com").await.unwrap();
        let code_lowercase = response2.backup_codes[0].to_lowercase();
        let is_valid = facade.verify_totp(user_id2, &code_lowercase).await.unwrap();
        assert!(is_valid, "Lowercase format should work");

        // Test without dash
        let user_id3 = UserId::new();
        let response3 = facade.setup_totp(user_id3, "test3@example.com").await.unwrap();
        let code_no_dash = response3.backup_codes[0].replace('-', "");
        let is_valid = facade.verify_totp(user_id3, &code_no_dash).await.unwrap();
        assert!(is_valid, "Format without dash should work");

        // Test with spaces instead of dash
        let user_id4 = UserId::new();
        let response4 = facade.setup_totp(user_id4, "test4@example.com").await.unwrap();
        let code_with_space = response4.backup_codes[0].replace('-', " ");
        let is_valid = facade.verify_totp(user_id4, &code_with_space).await.unwrap();
        assert!(is_valid, "Format with space should work");

        // Test with mixed case and no dash
        let user_id5 = UserId::new();
        let response5 = facade.setup_totp(user_id5, "test5@example.com").await.unwrap();
        let code_mixed = response5.backup_codes[0].replace('-', "").to_lowercase();
        let is_valid = facade.verify_totp(user_id5, &code_mixed).await.unwrap();
        assert!(is_valid, "Mixed case without dash should work");
    }

    #[tokio::test]
    async fn test_disable_mfa() {
        let totp_store = Arc::new(MockTotpStore::new());
        let backup_store = Arc::new(MockBackupCodesStore::new());
        let facade = MfaServiceFacadeImpl::with_defaults(totp_store.clone(), backup_store.clone());

        let user_id = UserId::new();
        facade
            .setup_totp(user_id, "test@example.com")
            .await
            .unwrap();

        // Disable MFA
        facade.disable_totp(user_id).await.unwrap();

        // Verify TOTP secret is deleted
        let secret = totp_store.get_totp_secret(user_id).await.unwrap();
        assert!(secret.is_none());
    }
}
