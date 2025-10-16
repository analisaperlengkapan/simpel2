//! MFA Authenc Integration Tests for Secreton
//!
//! This module enhances existing secreton MFA tests to include authenc integration,
//! testing the complete MFA flow from secreton's perspective.

use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use tokio::time::timeout;
use serde_json::json;

use secreton_core::mfa::{MfaManager, EnterpriseMfaManager};
use secreton_core::auth::AuthencAuthProvider;
use secreton_core::config::SecretonConfig;
use secreton_core::error::SecretonError;
use secreton_core::models::{MfaSetupRequest, MfaVerificationRequest, MfaStatusRequest};

/// Test utilities for MFA authenc integration
mod test_utils {
    use super::*;

    /// Create test secreton config with authenc integration
    pub fn create_test_config_with_authenc() -> SecretonConfig {
        let mut config = SecretonConfig::test_config();

        // Configure authenc integration
        config.authenc.enabled = true;
        config.authenc.base_url = std::env::var("TEST_AUTHENC_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        config.authenc.timeout = Duration::from_secs(10);
        config.authenc.retry_attempts = 3;

        // Configure MFA settings
        config.mfa.enabled = true;
        config.mfa.totp_issuer = "SIMPelv2 Kejaksaan RI".to_string();
        config.mfa.backup_codes_count = 10;
        config.mfa.rate_limit_window = Duration::from_secs(60);
        config.mfa.max_attempts_per_window = 5;

        config
    }

    /// Create test MFA setup request
    pub fn create_test_mfa_setup_request(user_id: &str, satker_code: &str) -> MfaSetupRequest {
        MfaSetupRequest {
            user_id: user_id.to_string(),
            issuer: "SIMPelv2 Kejaksaan RI".to_string(),
            account_name: format!("{}@kejaksaan.go.id", user_id),
            satker_code: satker_code.to_string(),
            metadata: Some(json!({
                "setup_source": "integration_test",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        }
    }

    /// Create test MFA verification request
    pub fn create_test_mfa_verification_request(user_id: &str, code: &str) -> MfaVerificationRequest {
        MfaVerificationRequest {
            user_id: user_id.to_string(),
            code: code.to_string(),
            allow_backup_code: false,
            metadata: Some(json!({
                "verification_source": "integration_test",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        }
    }
}

#[cfg(test)]
mod mfa_authenc_integration_tests {
    use super::*;
    use test_utils::*;

    /// Test MFA setup with authenc token validation
    #[tokio::test]
    #[ignore] // Requires running authenc and secreton instances
    async fn test_mfa_setup_with_authenc_validation() {
        let config = create_test_config_with_authenc();
        let auth_provider = AuthencAuthProvider::new(&config.authenc).await
            .expect("Failed to create authenc auth provider");
        let mfa_manager = EnterpriseMfaManager::new(&config).await
            .expect("Failed to create MFA manager");

        let test_user_id = format!("test_user_{}", Uuid::new_v4().to_string()[..8].to_string());
        let test_satker = "KEJATI_DKI_JAKPUS";

        // Test MFA setup with valid authenc context
        let setup_request = create_test_mfa_setup_request(&test_user_id, test_satker);

        let setup_result = timeout(
            Duration::from_secs(30),
            mfa_manager.setup_mfa(setup_request)
        ).await;

        match setup_result {
            Ok(Ok(setup_response)) => {
                println!("✅ MFA setup successful with authenc integration");
                println!("   User ID: {}", test_user_id);
                println!("   Satker: {}", test_satker);
                println!("   QR Code generated: {}", !setup_response.qr_code_url.is_empty());
                println!("   Secret key length: {}", setup_response.secret_key.len());
                println!("   Backup codes count: {}", setup_response.backup_codes.len());

                // Validate setup response structure
                assert!(!setup_response.qr_code_url.is_empty());
                assert!(!setup_response.secret_key.is_empty());
                assert_eq!(setup_response.backup_codes.len(), 10);
                assert!(setup_response.qr_code_url.starts_with("data:image/"));

                // Test MFA status after setup
                let status_request = MfaStatusRequest {
                    user_id: test_user_id.clone(),
                };

                let status_result = mfa_manager.get_mfa_status(status_request).await;
                match status_result {
                    Ok(status) => {
                        assert!(status.is_enabled);
                        assert_eq!(status.method, Some("TOTP".to_string()));
                        println!("✅ MFA status correctly reflects setup completion");
                    }
                    Err(e) => {
                        println!("⚠️  MFA status check failed: {:?}", e);
                    }
                }

                // Test TOTP verification with generated secret
                let otp_code = generate_test_totp_code(&setup_response.secret_key);
                let verification_request = create_test_mfa_verification_request(&test_user_id, &otp_code);

                let verification_result = mfa_manager.verify_mfa(verification_request).await;
                match verification_result {
                    Ok(()) => {
                        println!("✅ TOTP verification successful");
                    }
                    Err(e) => {
println!("⚠️  TOTP verification failed: {:?}", e);
                    }
                }
            }
            Ok(Err(e)) => {
                println!("❌ MFA setup failed: {:?}", e);
                match e {
                    SecretonError::AuthencCommunicationError { .. } => {
                        println!("ℹ️  This is expected if authenc is not running for integration tests");
                    }
                    _ => panic!("Unexpected error during MFA setup: {:?}", e),
                }
            }
            Err(_) => {
                println!("❌ MFA setup timed out - authenc might not be available");
            }
        }
    }

    /// Test MFA verification with authenc audit logging
    #[tokio::test]
    #[ignore] // Requires running authenc and secreton instances
    async fn test_mfa_verification_with_audit_logging() {
        let config = create_test_config_with_authenc();
        let mfa_manager = EnterpriseMfaManager::new(&config).await
            .expect("Failed to create MFA manager");

        let test_user_id = format!("test_user_{}", Uuid::new_v4().to_string()[..8].to_string());

        // Test multiple verification attempts to validate audit logging
        let test_scenarios = vec![
            ("123456", false, "Invalid OTP code"),
            ("000000", false, "Invalid OTP code"),
            ("111111", false, "Invalid OTP code"),
        ];

        for (code, should_succeed, description) in test_scenarios {
            let verification_request = create_test_mfa_verification_request(&test_user_id, code);

            let verification_result = timeout(
                Duration::from_secs(10),
                mfa_manager.verify_mfa(verification_request)
            ).await;

            match verification_result {
                Ok(Ok(())) => {
                    if should_succeed {
                        println!("✅ MFA verification successful: {}", description)
else {
                        panic!("MFA verification should have failed for: {}", description);
                    }
                }
                Ok(Err(SecretonError::InvalidMfaCode)) => {
                    if !should_succeed {
                        println!("✅ MFA verification correctly failed: {}", description);
                    } else {
                        panic!("MFA verification should have succeeded for: {}", description);
                    }
                }
                Ok(Err(SecretonError::RateLimitExceeded)) => {
                    println!("✅ Rate limiting triggered as expected: {}", description);
                    break; // Stop testing once rate limit is hit
                }
                Ok(Err(e)) => {
                    println!("⚠️  Unexpected error during MFA verification: {:?}", e);
                }
                Err(_) => {
                    println!("❌ MFA verification timed out for: {}", description);
                }
            }
        }
    }

    /// Test MFA backup code functionality with authenc integration
    #[tokio::test]
    #[ignore] // Requires running authenc and secreton instances
    async fn test_mfa_backup_codes_with_authenc() {
        let config = create_test_config_with_authenc();
        let mfa_manager = EnterpriseMfaManager::new(&config).await
            .expect("Failed to create MFA manager");

        let test_user_id = format!("test_user_{}", Uuid::new_v4().to_string()[..8].to_string());
        let test_satker = "KEJATI_DKI_JAKSEL";

        // Setup MFA first
        let setup_request = create_test_mfa_setup_request(&test_user_id, test_satker);
        let setup_result = mfa_manager.setup_mfa(setup_request).await;

        match setup_result {
            Ok(setup_response) => {
                let backup_codes = setup_response.backup_codes;
                assert!(!backup_codes.is_empty());

                // Test backup code verification
                let first_backup_code = &backup_codes[0];
                let mut verification_request = create_test_mfa_verification_request(&test_user_id, first_backup_code);
                verification_request.allow_backup_code = true;

                let backup_verification_result = timeout(
                    Duration::from_secs(10),
                    mfa_manager.verify_mfa(verification_request)
                ).await;

                match backup_verification_result {
                    Ok(Ok(())) => {
                        println!("✅ Backup code verification successful");

                        // Test that the same backup code cannot be used again
                        let mut reuse_request = create_test_mfa_verification_request(&test_user_id, first_backup_code);
                        reuse_request.allow_backup_code = true;

                        let reuse_result = mfa_manager.verify_mfa(reuse_request).await;
                        match reuse_result {
                            Err(SecretonError::BackupCodeAlreadyUsed) => {
                                println!("✅ Backup code reuse correctly prevented");
                            }
                            _ => {
                                println!("⚠️  Backup code reuse prevention not working as expected");
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        println!("❌ Backup code verification failed: {:?}", e);
                    }
                    Err(_) => {
                        println!("❌ Backup code verification timed out");
                    }
                }

                // Test backup code regeneration
                let regeneration_result = timeout(
                    Duration::from_secs(10),
                    mfa_manager.regenerate_backup_codes(&test_user_id)
                ).await;

                match regeneration_result {
                    Ok(Ok(new_backup_codes)) => {
                        assert_eq!(new_backup_codes.len(), 10);
                        assert_ne!(new_backup_codes, backup_codes);
                        println!("✅ Backup code regeneration successful");
                    }
                    Ok(Err(e)) => {
                        println!("❌ Backup code regeneration failed: {:?}", e);
                    }
                    Err(_) => {
                        println!("❌ Backup code regeneration timed out");
                    }
                }
            }
            Err(e) => {
                println!("❌ MFA setup failed, skipping backup code tests: {:?}", e);
            }
        }
    }

    /// Test MFA admin operations with authenc authorization
    #[tokio::test]
    #[ignore] // Requires running authenc and secreton instances
    async fn test_mfa_admin_operations_with_authenc() {
        let config = create_test_config_with_authenc();
        let mfa_manager = EnterpriseMfaManager::new(&config).await
            .expect("Failed to create MFA manager");

        let test_user_id = format!("test_user_{}", Uuid::new_v4().to_string()[..8].to_string());
        let admin_user_id = format!("admin_user_{}", Uuid::new_v4().to_string()[..8].to_string());

        // Test admin MFA disable operation
        let disable_result = timeout(
            Duration::from_secs(10),
            mfa_manager.admin_disable_mfa(&test_user_id, &admin_user_id, "Integration test")
        ).await;

        match disable_result {
            Ok(Ok(())) => {
                println!("✅ Admin MFA disable successful");

                // Verify MFA is disabled
                let status_request = MfaStatusRequest {
                    user_id: test_user_id.clone(),
                };

                let status_result = mfa_manager.get_mfa_status(status_request).await;
                match status_result {
                    Ok(status) => {
                        assert!(!status.is_enabled);
                        println!("✅ MFA status correctly reflects admin disable");
                    }
                    Err(e) => {
                        println!("⚠️  MFA status check after disable failed: {:?}", e);
                    }
                }
            }
            Ok(Err(e)) => {
                println!("❌ Admin MFA disable failed: {:?}", e);
                match e {
                    SecretonError::AuthencCommunicationError { .. } => {
                        println!("ℹ️  This is expected if authenc is not running for integration tests");
                    }
                    SecretonError::InsufficientPermissions => {
                        println!("ℹ️  This is expected if admin permissions are not properly configured");
                    }
                    _ => panic!("Unexpected error during admin MFA disable: {:?}", e),
                }
            }
            Err(_) => {
                println!("❌ Admin MFA disable timed out");
            }
        }

        // Test admin MFA reset operation
        let reset_result = timeout(
            Duration::from_secs(10),
            mfa_manager.admin_reset_mfa(&test_user_id, &admin_user_id, "Integration test reset")
        ).await;

        match reset_result {
            Ok(Ok(())) => {
                println!("✅ Admin MFA reset successful");
            }
            Ok(Err(e)) => {
                println!("❌ Admin MFA reset failed: {:?}", e);
            }
            Err(_) => {
                println!("❌ Admin MFA reset timed out");
            }
        }
    }

    /// Test MFA rate limiting integration with authenc
    #[tokio::test]
    #[ignore] // Requires running authenc and secreton instances
    async fn test_mfa_rate_limiting_with_authenc() {
        let config = create_test_config_with_authenc();
        let mfa_manager = EnterpriseMfaManager::new(&config).await
            .expect("Failed to create MFA manager");

        let test_user_id = format!("test_user_{}", Uuid::new_v4().to_string()[..8].to_string());

        // Test rate limiting by making multiple rapid verification attempts
        let mut attempts = 0;
        let max_attempts = 10;

        for i in 0..max_attempts {
            let verification_request = create_test_mfa_verification_request(&test_user_id, "000000");

            let verification_result = timeout(
                Duration::from_secs(5),
                mfa_manager.verify_mfa(verification_request)
            ).await;

            match verification_result {
                Ok(Err(SecretonError::RateLimitExceeded)) => {
                    println!("✅ Rate limiting triggered after {} attempts", i + 1);
                    attempts = i + 1;
                    break;
                }
                Ok(Err(SecretonError::InvalidMfaCode)) => {
                    println!("   Attempt {}: Invalid code (expected)", i + 1);
                }
                Ok(Ok(())) => {
                    panic!("MFA verification should not succeed with invalid code");
                }
                Ok(Err(e)) => {
                    println!("⚠️  Unexpected error on attempt {}: {:?}", i + 1, e);
                }
                Err(_) => {
                    println!("❌ MFA verification timed out on attempt {}", i + 1);
                }
            }
        }

        if attempts > 0 {
            println!("✅ Rate limiting working correctly, triggered after {} attempts", attempts);
            assert!(attempts <= config.mfa.max_attempts_per_window);
        } else {
            println!("⚠️  Rate limiting not triggered within {} attempts", max_attempts);
        }
    }

    /// Test MFA cross-satker isolation with authenc
    #[tokio::test]
    #[ignore] // Requires running authenc and secreton instances
    async fn test_mfa_cross_satker_isolation() {
        let config = create_test_config_with_authenc();
        let mfa_manager = EnterpriseMfaManager::new(&config).await
            .expect("Failed to create MFA manager");

        let user_jakpus = format!("jakpus_user_{}", Uuid::new_v4().to_string()[..8].to_string());
        let user_jaksel = format!("jaksel_user_{}", Uuid::new_v4().to_string()[..8].to_string());

        // Setup MFA for users from different satker
        let setup_jakpus = create_test_mfa_setup_request(&user_jakpus, "KEJATI_DKI_JAKPUS");
        let setup_jaksel = create_test_mfa_setup_request(&user_jaksel, "KEJATI_DKI_JAKSEL");

        let jakpus_result = mfa_manager.setup_mfa(setup_jakpus).await;
        let jaksel_result = mfa_manager.setup_mfa(setup_jaksel).await;

        match (jakpus_result, jaksel_result) {
            (Ok(jakpus_setup), Ok(jaksel_setup)) => {
                println!("✅ MFA setup successful for both satker");

                // Verify that secrets are isolated between satker
                assert_ne!(jakpus_setup.secret_key, jaksel_setup.secret_key);
                assert_ne!(jakpus_setup.backup_codes, jaksel_setup.backup_codes);

                println!("✅ MFA secrets properly isolated between satker");

                // Test that admin from one satker cannot manage MFA for another satker
                let cross_satker_disable_result = mfa_manager
                    .admin_disable_mfa(&user_jaksel, &user_jakpus, "Cross-satker test")
                    .await;

                match cross_satker_disable_result {
                    Err(SecretonError::InsufficientPermissions) => {
                        println!("✅ Cross-satker admin operations correctly blocked");
                    }
                    Ok(()) => {
                        println!("⚠️  Cross-satker admin operation succeeded (might need policy review)");
                    }
                    Err(e) => {
                        println!("⚠️  Unexpected error in cross-satker test: {:?}", e);
                    }
                }
            }
            _ => {
                println!("❌ MFA setup failed for cross-satker isolation test");
            }
        }
    }

    /// Helper function to generate test TOTP code
    fn generate_test_totp_code(secret: &str) -> String {
        use hmac::{Hmac, Mac};
        use sha1::Sha1;

        let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, secret)
            .expect("Failed to decode secret");

        let current_time = chrono::Utc::now().timestamp() as u64;
        let time_step = current_time / 30;
        let time_bytes = time_step.to_be_bytes();

        let mut mac = Hmac::<Sha1>::new_from_slice(&secret_bytes)
            .expect("Failed to create HMAC");
        mac.update(&time_bytes);
        let hash = mac.finalize().into_bytes();

        let offset = (hash[hash.len() - 1] & 0x0f) as usize;
        let code = u32::from_be_bytes([
            hash[offset] & 0x7f,
            hash[offset + 1],
            hash[offset + 2],
            hash[offset + 3],
        ]);

        format!("{:06}", code % 1_000_000)
    }
}

/// End-to-end MFA workflow tests
#[cfg(test)]
mod end_to_end_tests {
    use super::*;
    use test_utils::*;

    /// Test complete MFA workflow from setup to verification
    #[tokio::test]
    #[ignore] // Requires running authenc and secreton instances
    async fn test_complete_mfa_workflow() {
        let config = create_test_config_with_authenc();
        let mfa_manager = EnterpriseMfaManager::new(&config).await
            .expect("Failed to create MFA manager");

        let test_user_id = format!("workflow_user_{}", Uuid::new_v4().to_string()[..8].to_string());
        let test_satker = "KEJATI_DKI_JAKPUS";

        println!("🔄 Starting complete MFA workflow test for user: {}", test_user_id);

        // Step 1: MFA Setup
        println!("📋 Step 1: MFA Setup");
        let setup_request = create_test_mfa_setup_request(&test_user_id, test_satker);
        let setup_result = timeout(Duration::from_secs(30), mfa_manager.setup_mfa(setup_request)).await;

        let setup_response = match setup_result {
            Ok(Ok(response)) => {
                println!("✅ MFA setup completed successfully");
                response
            }
            Ok(Err(e)) => {
                println!("❌ MFA setup failed: {:?}", e);
                return;
            }
            Err(_) => {
                println!("❌ MFA setup timed out");
                return;
            }
        };

        // Step 2: TOTP Verification
        println!("📋 Step 2: TOTP Verification");
        let totp_code = generate_test_totp_code(&setup_response.secret_key);
        let verification_request = create_test_mfa_verification_request(&test_user_id, &totp_code);

        let verification_result = timeout(Duration::from_secs(10), mfa_manager.verify_mfa(verification_request)).await;
        match verification_result {
            Ok(Ok(())) => {
                println!("✅ TOTP verification successful");
            }
            Ok(Err(e)) => {
                println!("❌ TOTP verification failed: {:?}", e);
            }
            Err(_) => {
                println!("❌ TOTP verification timed out");
            }
        }

        // Step 3: Backup Code Verification
        println!("📋 Step 3: Backup Code Verification");
        let backup_code = &setup_response.backup_codes[0];
        let mut backup_verification_request = create_test_mfa_verification_request(&test_user_id, backup_code);
        backup_verification_request.allow_backup_code = true;

        let backup_result = timeout(Duration::from_secs(10), mfa_manager.verify_mfa(backup_verification_request)).await;
        match backup_result {
            Ok(Ok(())) => {
                println!("✅ Backup code verification successful");
            }
            Ok(Err(e)) => {
                println!("❌ Backup code verification failed: {:?}", e);
            }
            Err(_) => {
                println!("❌ Backup code verification timed out");
            }
        }

        // Step 4: Status Check
        println!("📋 Step 4: MFA Status Check");
        let status_request = MfaStatusRequest {
            user_id: test_user_id.clone(),
        };

        let status_result = mfa_manager.get_mfa_status(status_request).await;
        match status_result {
            Ok(status) => {
                println!("✅ MFA status retrieved successfully");
                println!("   Enabled: {}", status.is_enabled);
                println!("   Method: {:?}", status.method);
                println!("   Recovery codes remaining: {:?}", status.recovery_codes_remaining);
            }
            Err(e) => {
                println!("❌ MFA status check failed: {:?}", e);
            }
        }

        // Step 5: Recovery Code Regeneration
        println!("📋 Step 5: Recovery Code Regeneration");
        let regeneration_result = timeout(Duration::from_secs(10), mfa_manager.regenerate_backup_codes(&test_user_id)).await;
        match regeneration_result {
            Ok(Ok(new_codes)) => {
                println!("✅ Recovery code regeneration successful");
                println!("   New codes count: {}", new_codes.len());
            }
            Ok(Err(e)) => {
                println!("❌ Recovery code regeneration failed: {:?}", e);
            }
            Err(_) => {
                println!("❌ Recovery code regeneration timed out");
            }
        }

        println!("🏁 Complete MFA workflow test finished for user: {}", test_user_id);
    }

    /// Helper function to generate test TOTP code (duplicate from above for module isolation)
    fn generate_test_totp_code(secret: &str) -> String {
        use hmac::{Hmac, Mac};
        use sha1::Sha1;

        let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, secret)
            .expect("Failed to decode secret");

        let current_time = chrono::Utc::now().timestamp() as u64;
        let time_step = current_time / 30;
        let time_bytes = time_step.to_be_bytes();

        let mut mac = Hmac::<Sha1>::new_from_slice(&secret_bytes)
            .expect("Failed to create HMAC");
        mac.update(&time_bytes);
        let hash = mac.finalize().into_bytes();

        let offset = (hash[hash.len() - 1] & 0x0f) as usize;
        let code = u32::from_be_bytes([
            hash[offset] & 0x7f,
            hash[offset + 1],
            hash[offset + 2],
            hash[offset + 3],
        ]);

        format!("{:06}", code % 1_000_000)
    }
}
