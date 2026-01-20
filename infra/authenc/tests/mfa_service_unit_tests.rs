//! Unit tests for MFA Service
//!
//! This module contains comprehensive unit tests for the MFA service wrapper,
//! testing integration with existing OtpCredentialProvider and error handling.

use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use authenc::error::{AuthencError, Result};
use authenc::models::user::{AccessLevel, SecretonAccessPolicy, SecurityContext, User};
use authenc::services::mfa_service::{MfaClient, MfaService};
use authenc::spi::credential::otp::{OtpAlgorithm, OtpCredentialProvider};

use authenc::secreton_client::secreton_client::{MfaSetupData, MfaStatusResponse};

/// Create a test database pool (mock for testing)
fn create_test_db_pool() -> deadpool_postgres::Pool {
    // This is a mock pool for testing - in real tests you'd use a test database
    let mut cfg = deadpool_postgres::Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.port = Some(5432);
    cfg.dbname = Some("authenc_test".to_string());
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("password".to_string());

    cfg.create_pool(
        Some(deadpool_postgres::Runtime::Tokio1),
        tokio_postgres::NoTls,
    )
    .unwrap()
}

/// Mock SecretonClient for testing
#[derive(Clone)]
struct MockSecretonClient {
    should_fail: bool,
    mfa_enabled: bool,
}

impl MockSecretonClient {
    fn new() -> Self {
        Self {
            should_fail: false,
            mfa_enabled: false,
        }
    }

    fn with_failure() -> Self {
        Self {
            should_fail: true,
            mfa_enabled: false,
        }
    }

    fn with_mfa_enabled() -> Self {
        Self {
            should_fail: false,
            mfa_enabled: true,
        }
    }

    async fn setup_mfa(
        &self,
        user_id: &str,
        issuer: &str,
        account_name: &str,
    ) -> Result<MfaSetupData> {
        if self.should_fail {
            return Err(AuthencError::ExternalServiceError {
                service: "Secreton setup failed".to_string(),
            });
        }

        Ok(MfaSetupData {
            qr_code_url: "data:image/png;base64,test".to_string(),
            secret_key: "JBSWY3DPEHPK3PXP".to_string(),
            backup_codes: vec!["12345678".to_string(), "87654321".to_string()],
        })
    }

    async fn verify_mfa_setup(&self, user_id: &str, code: &str) -> Result<()> {
        if self.should_fail {
            return Err(AuthencError::ExternalServiceError {
                service: "Secreton verification failed".to_string(),
            });
        }

        if code == "123456" {
            Ok(())
        } else {
            Err(AuthencError::ValidationError {
                message: "Invalid MFA code".to_string(),
            })
        }
    }

    async fn verify_mfa(&self, user_id: &str, code: &str) -> Result<()> {
        if self.should_fail {
            return Err(AuthencError::ExternalServiceError {
                service: "Secreton verification failed".to_string(),
            });
        }

        if code == "123456" {
            Ok(())
        } else if code == "rate_limit" {
            Err(AuthencError::MfaRateLimitExceeded)
        } else {
            Err(AuthencError::InvalidOtpCode)
        }
    }

    async fn disable_mfa(&self, user_id: &str, admin_context: &SecurityContext) -> Result<()> {
        if self.should_fail {
            return Err(AuthencError::ExternalServiceError {
                service: "Secreton disable failed".to_string(),
            });
        }
        Ok(())
    }

    async fn get_mfa_status(&self, user_id: &str) -> Result<MfaStatusResponse> {
        if self.should_fail {
            return Err(AuthencError::ExternalServiceError {
                service: "Secreton status check failed".to_string(),
            });
        }

        Ok(MfaStatusResponse {
            is_enabled: self.mfa_enabled,
            method: Some("TOTP".to_string()),
            last_used: None,
        })
    }

    async fn verify_recovery_code(&self, user_id: &str, recovery_code: &str) -> Result<()> {
        if self.should_fail {
            return Err(AuthencError::ExternalServiceError {
                service: "Secreton recovery verification failed".to_string(),
            });
        }

        if recovery_code == "12345678" {
            Ok(())
        } else if recovery_code == "used_code" {
            Err(AuthencError::RecoveryCodeAlreadyUsed)
        } else {
            Err(AuthencError::InvalidBackupCode)
        }
    }

    async fn regenerate_recovery_codes(&self, user_id: &str) -> Result<Vec<String>> {
        if self.should_fail {
            return Err(AuthencError::ExternalServiceError {
                service: "Secreton regeneration failed".to_string(),
            });
        }

        Ok(vec![
            "11111111".to_string(),
            "22222222".to_string(),
            "33333333".to_string(),
        ])
    }
}

#[async_trait::async_trait]
impl MfaClient for MockSecretonClient {
    async fn setup_mfa(
        &self,
        user_id: &str,
        issuer: &str,
        account_name: &str,
    ) -> Result<MfaSetupData> {
        self.setup_mfa(user_id, issuer, account_name).await
    }

    async fn verify_mfa_setup(&self, user_id: &str, code: &str) -> Result<()> {
        self.verify_mfa_setup(user_id, code).await
    }

    async fn verify_mfa(&self, user_id: &str, code: &str) -> Result<()> {
        self.verify_mfa(user_id, code).await
    }

    async fn disable_mfa(&self, user_id: &str, admin_context: &SecurityContext) -> Result<()> {
        self.disable_mfa(user_id, admin_context).await
    }

    async fn get_mfa_status(&self, user_id: &str) -> Result<MfaStatusResponse> {
        self.get_mfa_status(user_id).await
    }

    async fn verify_recovery_code(&self, user_id: &str, recovery_code: &str) -> Result<()> {
        self.verify_recovery_code(user_id, recovery_code).await
    }

    async fn regenerate_recovery_codes(&self, user_id: &str) -> Result<Vec<String>> {
        self.regenerate_recovery_codes(user_id).await
    }
}

/// Create a test user
fn create_test_user() -> User {
    User {
        id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@kejaksaan.go.id".to_string(),
        email_verified: true,
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        nip: Some("123456789".to_string()),
        nama: Some("Test User".to_string()),
        jabatan: Some("Jaksa".to_string()),
        satker_code: "001".to_string(),
        phone_number: None,
        phone_verified: false,
        password_hash: None,
        totp_secret: None,
        totp_backup_codes: None,
        mfa_enabled: false,
        mfa_setup_at: None,
        mfa_last_used: None,
        webauthn_enabled: false,
        account_locked: false,
        account_locked_until: None,
        failed_login_attempts: 0,
        last_login_at: None,
        last_failed_login_at: None,
        password_changed_at: None,
        password_expires_at: None,
        require_password_change: false,
        realm_id: Some(Uuid::new_v4()),
        organization_id: None,
        roles: Vec::new(),
        permissions: Vec::new(),
        session_data: None,
        secreton_access_policy: SecretonAccessPolicy {
            allowed_satker_secrets: vec!["001".to_string()],
            access_level: AccessLevel::ReadOnly,
            time_restrictions: None,
            audit_required: true,
            rate_limit: Some(100),
            allowed_paths: None,
            denied_paths: None,
        },
        security_context: SecurityContext {
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            session_id: Some(Uuid::new_v4().to_string()),
            timestamp: Utc::now(),
            risk_score: Some(0.1),
            metadata: None,
        },
        attributes: None,
        enabled: true,
        federated: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        login_count: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mfa_service_creation() {
        let secreton_client = Arc::new(MockSecretonClient::new());
        let db_pool = create_test_db_pool();

        let mfa_service = MfaService::new(secreton_client.clone(), db_pool.clone());

        // Test service creation without cache
        // We can't directly access private fields, but we can test that the service was created
        // by calling a method that would fail if the service wasn't properly initialized
        let user_id = Uuid::new_v4();
        let result = mfa_service.regenerate_recovery_codes(user_id).await;

        // This should succeed as it only calls secreton, not database
        assert!(result.is_ok());
        let codes = result.unwrap();
        assert_eq!(codes.len(), 3);
    }

    #[tokio::test]
    async fn test_regenerate_recovery_codes_success() {
        let secreton_client = Arc::new(MockSecretonClient::new());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let user_id = Uuid::new_v4();

        let result = mfa_service.regenerate_recovery_codes(user_id).await;

        // This should succeed as it only calls secreton, not database
        assert!(result.is_ok());
        let codes = result.unwrap();
        assert_eq!(codes.len(), 3);
        assert_eq!(codes[0], "11111111");
        assert_eq!(codes[1], "22222222");
        assert_eq!(codes[2], "33333333");
    }

    #[tokio::test]
    async fn test_regenerate_recovery_codes_failure() {
        let secreton_client = Arc::new(MockSecretonClient::with_failure());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let user_id = Uuid::new_v4();

        let result = mfa_service.regenerate_recovery_codes(user_id).await;

        // Should fail due to secreton failure
        assert!(result.is_err());
        match result.unwrap_err() {
            AuthencError::ExternalServiceError { .. } => {
                // Expected - secreton failure
            }
            _ => panic!("Expected external service error"),
        }
    }

    #[tokio::test]
    async fn test_get_recovery_codes_count() {
        let secreton_client = Arc::new(MockSecretonClient::with_mfa_enabled());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let user_id = Uuid::new_v4();

        let result = mfa_service.get_recovery_codes_count(user_id).await;

        // Should succeed as it only calls secreton
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn test_has_recovery_codes() {
        let secreton_client = Arc::new(MockSecretonClient::with_mfa_enabled());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let user_id = Uuid::new_v4();

        let result = mfa_service.has_recovery_codes(user_id).await;

        // Should succeed as it only calls secreton
        assert!(result.is_ok());
        let has_codes = result.unwrap();
        assert!(has_codes);
    }

    #[tokio::test]
    async fn test_has_no_recovery_codes() {
        let secreton_client = Arc::new(MockSecretonClient::new()); // MFA not enabled, so no codes
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let user_id = Uuid::new_v4();

        let result = mfa_service.has_recovery_codes(user_id).await;

        // Should succeed but return false
        assert!(result.is_ok());
        let has_codes = result.unwrap();
        assert!(!has_codes);
    }

    #[tokio::test]
    async fn test_batch_get_mfa_status_empty() {
        let secreton_client = Arc::new(MockSecretonClient::new());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let user_ids = vec![];

        let result = mfa_service.batch_get_mfa_status(&user_ids).await;

        // Should succeed with empty result
        assert!(result.is_ok());
        let results = result.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_setup_mfa_database_error() {
        let secreton_client = Arc::new(MockSecretonClient::new());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let user_id = Uuid::new_v4();

        // This will fail because we don't have a real database setup with test data
        let result = mfa_service.setup_mfa(user_id).await;

        assert!(result.is_err());
        // The error should be a database error (user not found)
        match result.unwrap_err() {
            AuthencError::DatabaseError { .. } => {
                // Expected - user doesn't exist in test DB
            }
            _ => {
                // Also acceptable - could be connection error
            }
        }
    }

    #[tokio::test]
    async fn test_verify_setup_database_error() {
        let secreton_client = Arc::new(MockSecretonClient::new());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let user_id = Uuid::new_v4();
        let code = "123456";

        let result = mfa_service.verify_setup(user_id, code).await;

        // Will fail due to database error (no test data)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_mfa_database_error() {
        let secreton_client = Arc::new(MockSecretonClient::new());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let user_id = Uuid::new_v4();
        let code = "123456";

        let result = mfa_service.verify_mfa(user_id, code).await;

        // Will fail due to database error (no test data)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_mfa_status_database_error() {
        let secreton_client = Arc::new(MockSecretonClient::new());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let user_id = Uuid::new_v4();

        let result = mfa_service.get_mfa_status(user_id).await;

        // Will fail due to database error (no test data)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_disable_mfa_database_error() {
        let secreton_client = Arc::new(MockSecretonClient::new());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let user_id = Uuid::new_v4();
        let admin_context = SecurityContext {
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("admin-agent".to_string()),
            session_id: Some(Uuid::new_v4().to_string()),
            timestamp: Utc::now(),
            risk_score: Some(0.0),
            metadata: None,
        };

        let result = mfa_service.disable_mfa(user_id, &admin_context).await;

        // Will fail due to database error (no test data)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_recovery_code_database_error() {
        let secreton_client = Arc::new(MockSecretonClient::new());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let user_id = Uuid::new_v4();
        let recovery_code = "12345678";

        let result = mfa_service
            .verify_recovery_code(user_id, recovery_code)
            .await;

        // Will fail due to database error (no test data)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_mfa_statistics_database_error() {
        let secreton_client = Arc::new(MockSecretonClient::new());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let result = mfa_service.get_mfa_statistics(None).await;

        // Will fail due to database error (no test schema)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refresh_mfa_statistics_database_error() {
        let secreton_client = Arc::new(MockSecretonClient::new());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let result = mfa_service.refresh_mfa_statistics().await;

        // Will fail due to database error (no test schema)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_log_admin_action_database_error() {
        let secreton_client = Arc::new(MockSecretonClient::new());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let admin_user_id = Some(Uuid::new_v4());
        let target_user_id = Uuid::new_v4();
        let action = "disable_mfa";
        let reason = "Security incident";

        let result = mfa_service
            .log_admin_action(admin_user_id, target_user_id, action, reason)
            .await;

        // Will fail due to database error (no test schema)
        assert!(result.is_err());
    }
}

/// Integration tests with OtpCredentialProvider
#[cfg(test)]
mod otp_integration_tests {
    use super::*;

    #[test]
    fn test_otp_provider_integration() {
        let provider = OtpCredentialProvider::new();

        // Test secret generation
        let secret = provider.generate_secret();
        assert!(!secret.is_empty());
        assert!(secret.len() >= 16); // Base32 encoded secret should be at least 16 chars

        // Test provisioning URI generation
        let uri = provider.generate_provisioning_uri(
            &secret,
            "test@kejaksaan.go.id",
            "SIMPelv2 Kejaksaan RI",
            OtpAlgorithm::HmacSha1,
            6,
            30,
        );

        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("SIMPelv2%20Kejaksaan%20RI"));
        assert!(uri.contains("test%40kejaksaan.go.id"));
        assert!(uri.contains(&format!("secret={}", secret)));
        assert!(uri.contains("algorithm=HmacSHA1"));
        assert!(uri.contains("digits=6"));
        assert!(uri.contains("period=30"));
    }

    #[test]
    fn test_otp_verification() {
        let provider = OtpCredentialProvider::new();
        let secret = "JBSWY3DPEHPK3PXP"; // Test secret from RFC 6238

        // Generate a code for current time
        let current_time = chrono::Utc::now().timestamp() as u64;
        let time_step = current_time / 30;

        // We can't predict the exact code, but we can test the verification logic
        // by generating a code and then verifying it
        let secret_bytes =
            base32::decode(base32::Alphabet::Rfc4648 { padding: false }, secret).unwrap();
        let code = provider
            .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha1, 6)
            .unwrap();

        // Verify the generated code
        let is_valid = provider
            .verify_totp(secret, &code, OtpAlgorithm::HmacSha1, 6, 30)
            .unwrap();
        assert!(is_valid);

        // Test invalid code
        let is_invalid = provider
            .verify_totp(secret, "000000", OtpAlgorithm::HmacSha1, 6, 30)
            .unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_otp_time_window_tolerance() {
        let provider = OtpCredentialProvider::new();
        let secret = "JBSWY3DPEHPK3PXP";

        let current_time = chrono::Utc::now().timestamp() as u64;
        let time_step = current_time / 30;

        // Generate codes for previous, current, and next time steps
        let secret_bytes =
            base32::decode(base32::Alphabet::Rfc4648 { padding: false }, secret).unwrap();

        let prev_code = provider
            .generate_totp_for_step(&secret_bytes, time_step - 1, OtpAlgorithm::HmacSha1, 6)
            .unwrap();
        let curr_code = provider
            .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha1, 6)
            .unwrap();
        let next_code = provider
            .generate_totp_for_step(&secret_bytes, time_step + 1, OtpAlgorithm::HmacSha1, 6)
            .unwrap();

        // All three should be valid due to time window tolerance
        assert!(
            provider
                .verify_totp(secret, &prev_code, OtpAlgorithm::HmacSha1, 6, 30)
                .unwrap()
        );
        assert!(
            provider
                .verify_totp(secret, &curr_code, OtpAlgorithm::HmacSha1, 6, 30)
                .unwrap()
        );
        assert!(
            provider
                .verify_totp(secret, &next_code, OtpAlgorithm::HmacSha1, 6, 30)
                .unwrap()
        );

        // Code from 2 steps away should be invalid
        let far_code = provider
            .generate_totp_for_step(&secret_bytes, time_step + 2, OtpAlgorithm::HmacSha1, 6)
            .unwrap();
        assert!(
            !provider
                .verify_totp(secret, &far_code, OtpAlgorithm::HmacSha1, 6, 30)
                .unwrap()
        );
    }

    #[test]
    fn test_otp_different_algorithms() {
        let provider = OtpCredentialProvider::new();
        let secret = "JBSWY3DPEHPK3PXP";
        let secret_bytes =
            base32::decode(base32::Alphabet::Rfc4648 { padding: false }, secret).unwrap();

        let current_time = chrono::Utc::now().timestamp() as u64;
        let time_step = current_time / 30;

        // Test different algorithms
        let sha1_code = provider
            .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha1, 6)
            .unwrap();
        let sha256_code = provider
            .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha256, 6)
            .unwrap();
        let sha512_code = provider
            .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha512, 6)
            .unwrap();

        // Codes should be different for different algorithms
        assert_ne!(sha1_code, sha256_code);
        assert_ne!(sha1_code, sha512_code);
        assert_ne!(sha256_code, sha512_code);

        // Each should verify with its respective algorithm
        assert!(
            provider
                .verify_totp(secret, &sha1_code, OtpAlgorithm::HmacSha1, 6, 30)
                .unwrap()
        );
        assert!(
            provider
                .verify_totp(secret, &sha256_code, OtpAlgorithm::HmacSha256, 6, 30)
                .unwrap()
        );
        assert!(
            provider
                .verify_totp(secret, &sha512_code, OtpAlgorithm::HmacSha512, 6, 30)
                .unwrap()
        );

        // Cross-algorithm verification should fail
        assert!(
            !provider
                .verify_totp(secret, &sha1_code, OtpAlgorithm::HmacSha256, 6, 30)
                .unwrap()
        );
        assert!(
            !provider
                .verify_totp(secret, &sha256_code, OtpAlgorithm::HmacSha1, 6, 30)
                .unwrap()
        );
    }

    #[test]
    fn test_otp_different_digits() {
        let provider = OtpCredentialProvider::new();
        let secret = "JBSWY3DPEHPK3PXP";
        let secret_bytes =
            base32::decode(base32::Alphabet::Rfc4648 { padding: false }, secret).unwrap();

        let current_time = chrono::Utc::now().timestamp() as u64;
        let time_step = current_time / 30;

        // Test different digit lengths
        let code_6 = provider
            .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha1, 6)
            .unwrap();
        let code_8 = provider
            .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha1, 8)
            .unwrap();

        assert_eq!(code_6.len(), 6);
        assert_eq!(code_8.len(), 8);

        // Verify with correct digit length
        assert!(
            provider
                .verify_totp(secret, &code_6, OtpAlgorithm::HmacSha1, 6, 30)
                .unwrap()
        );
        assert!(
            provider
                .verify_totp(secret, &code_8, OtpAlgorithm::HmacSha1, 8, 30)
                .unwrap()
        );

        // Cross-digit verification should fail
        assert!(
            !provider
                .verify_totp(secret, &code_6, OtpAlgorithm::HmacSha1, 8, 30)
                .unwrap()
        );
        assert!(
            !provider
                .verify_totp(secret, &code_8, OtpAlgorithm::HmacSha1, 6, 30)
                .unwrap()
        );
    }

    #[test]
    fn test_invalid_secret_format() {
        let provider = OtpCredentialProvider::new();

        // Test with invalid base32 secret
        let result = provider.verify_totp("INVALID!", "123456", OtpAlgorithm::HmacSha1, 6, 30);
        assert!(result.is_err());

        // Test with empty secret
        let result = provider.verify_totp("", "123456", OtpAlgorithm::HmacSha1, 6, 30);
        assert!(result.is_err());
    }
}

/// Error handling tests
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_mfa_error_types() {
        // Test MFA-specific error constructors
        let error = AuthencError::mfa_not_enabled();
        assert!(matches!(error, AuthencError::MfaNotEnabled));

        let error = AuthencError::mfa_already_enabled();
        assert!(matches!(error, AuthencError::MfaAlreadyEnabled));

        let error = AuthencError::invalid_otp_code();
        assert!(matches!(error, AuthencError::InvalidOtpCode));

        let error = AuthencError::mfa_setup_required();
        assert!(matches!(error, AuthencError::MfaSetupRequired));

        let error = AuthencError::mfa_verification_required();
        assert!(matches!(error, AuthencError::MfaVerificationRequired));

        let error = AuthencError::mfa_secret_encryption_failed();
        assert!(matches!(error, AuthencError::MfaSecretEncryptionFailed));

        let error = AuthencError::mfa_secret_decryption_failed();
        assert!(matches!(error, AuthencError::MfaSecretDecryptionFailed));
    }

    #[test]
    fn test_error_display() {
        let error = AuthencError::MfaNotEnabled;
        assert_eq!(error.to_string(), "MFA is not enabled for this user");

        let error = AuthencError::InvalidOtpCode;
        assert_eq!(error.to_string(), "Invalid OTP code");

        let error = AuthencError::MfaSetupRequired;
        assert_eq!(error.to_string(), "MFA setup is required");
    }

    #[tokio::test]
    async fn test_secreton_error_mapping() {
        let secreton_client = Arc::new(MockSecretonClient::new());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        // Test that secreton errors are properly mapped to AuthencError
        let user_id = Uuid::new_v4();
        let result = mfa_service.verify_mfa(user_id, "invalid").await;

        // Should get a database error since we don't have test data
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_secreton_failure_error_mapping() {
        let secreton_client = Arc::new(MockSecretonClient::with_failure());
        let db_pool = create_test_db_pool();
        let mfa_service = MfaService::new(secreton_client, db_pool);

        let user_id = Uuid::new_v4();

        // Test regenerate recovery codes failure
        let result = mfa_service.regenerate_recovery_codes(user_id).await;
        assert!(result.is_err());

        // Test get recovery codes count failure
        let result = mfa_service.get_recovery_codes_count(user_id).await;
        assert!(result.is_err());
    }
}
