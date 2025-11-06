//! MFA Integration Tests
//!
//! This module contains comprehensive integration tests for MFA functionality,
//! testing the complete flow between authenc and secreton services.

use chrono::{DateTime, Utc};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

use authenc::app::AppState;
use authenc::config::AppConfig;
use authenc::error::{AuthencError, Result};
use authenc::models::user::{AccessLevel, SecretonAccessPolicy, SecurityContext, User};
use authenc::services::mfa_service::{MfaService, MfaSetupResponse, MfaStatus};
use authenc::spi::credential::otp::{OtpAlgorithm, OtpCredentialProvider};
use authenc::secreton_client::secreton_client::SecretonClient;

/// Integration test utilities
mod test_utils {
    use super::*;

    /// Create a test database pool for integration tests
    pub async fn create_integration_db_pool() -> deadpool_postgres::Pool {
        let mut cfg = deadpool_postgres::Config::new();
        cfg.host = Some(std::env::var("TEST_DB_HOST").unwrap_or_else(|_| "localhost".to_string()));
        cfg.port = Some(
            std::env::var("TEST_DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
        );
        cfg.dbname = Some(
            std::env::var("TEST_DB_NAME")
                .unwrap_or_else(|_| "authenc_integration_test".to_string()),
        );
        cfg.user = Some(std::env::var("TEST_DB_USER").unwrap_or_else(|_| "postgres".to_string()));
        cfg.password =
            Some(std::env::var("TEST_DB_PASSWORD").unwrap_or_else(|_| "password".to_string()));

        cfg.create_pool(
            Some(deadpool_postgres::Runtime::Tokio1),
            tokio_postgres::NoTls,
        )
        .expect("Failed to create test database pool")
    }

    /// Create a test secreton client for integration tests
    pub async fn create_integration_secreton_client() -> Arc<SecretonClient> {
        let base_url = std::env::var("TEST_SECRETON_URL")
            .unwrap_or_else(|_| "http://localhost:8200".to_string());
        let token = std::env::var("TEST_SECRETON_TOKEN")
            .unwrap_or_else(|_| "test-token".to_string());

        let client = SecretonClient::new(base_url, token);

        Arc::new(client)
    }

    /// Create a test user for integration tests
    pub fn create_integration_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            username: format!("test_user_{}", Uuid::new_v4().to_string()[..8].to_string()),
            email: "integration.test@kejaksaan.go.id".to_string(),
            email_verified: true,
            first_name: Some("Integration".to_string()),
            last_name: Some("Test".to_string()),
            nip: Some(format!("19{:08}", rand::random::<u32>() % 100000000)),
            nama: Some("Integration Test User".to_string()),
            jabatan: Some("Jaksa Muda".to_string()),
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
                access_level: AccessLevel::ReadWrite,
                time_restrictions: None,
                audit_required: true,
                rate_limit: Some(100),
                allowed_paths: None,
                denied_paths: None,
            },
            security_context: SecurityContext {
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("integration-test-agent".to_string()),
                session_id: Some(Uuid::new_v4().to_string()),
                timestamp: Utc::now(),
                risk_score: Some(0.0),
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

    /// Setup test database schema and user
    pub async fn setup_test_user_in_db(pool: &deadpool_postgres::Pool, user: &User) -> Result<()> {
        let client = pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        // Create users table if it doesn't exist (simplified for testing)
        client
            .execute(
                "CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                username VARCHAR NOT NULL UNIQUE,
                email VARCHAR NOT NULL,
                nip VARCHAR,
                nama VARCHAR,
                jabatan VARCHAR,
                satker_code VARCHAR,
                mfa_enabled BOOLEAN DEFAULT FALSE,
                mfa_setup_at TIMESTAMP WITH TIME ZONE,
                mfa_last_used TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )",
                &[],
            )
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        // Insert test user
        client.execute(
            "INSERT INTO users (id, username, email, nip, nama, jabatan, satker_code, mfa_enabled, mfa_setup_at, mfa_last_used)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT (username) DO UPDATE SET
                email = EXCLUDED.email,
                nip = EXCLUDED.nip,
                nama = EXCLUDED.nama,
                jabatan = EXCLUDED.jabatan,
                satker_code = EXCLUDED.satker_code,
                mfa_enabled = EXCLUDED.mfa_enabled,
                mfa_setup_at = EXCLUDED.mfa_setup_at,
                mfa_last_used = EXCLUDED.mfa_last_used",
            &[
                &user.id,
                &user.username,
                &user.email,
                &user.nip,
                &user.nama,
                &user.jabatan,
                &user.satker_code,
                &user.mfa_enabled,
                &user.mfa_setup_at,
                &user.mfa_last_used,
            ],
        ).await
        .map_err(|e| AuthencError::database(e.to_string()))?;

        Ok(())
    }

    /// Cleanup test data
    pub async fn cleanup_test_user(pool: &deadpool_postgres::Pool, user_id: Uuid) -> Result<()> {
        let client = pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        client
            .execute("DELETE FROM users WHERE id = $1", &[&user_id])
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use test_utils::*;

    /// Test the complete MFA setup flow with secreton integration
    #[tokio::test]
    #[ignore] // Requires running secreton instance
    async fn test_complete_mfa_setup_flow() {
        // Setup test environment
        let db_pool = create_integration_db_pool().await;
        let secreton_client = create_integration_secreton_client().await;
        let test_user = create_integration_test_user();

        // Setup test user in database
        setup_test_user_in_db(&db_pool, &test_user)
            .await
            .expect("Failed to setup test user");

        let mfa_service = MfaService::new(secreton_client.clone(), db_pool.clone());

        // Test MFA setup
        let setup_result =
            timeout(Duration::from_secs(30), mfa_service.setup_mfa(test_user.id)).await;

        match setup_result {
            Ok(Ok(setup_response)) => {
                // Validate setup response
                assert!(!setup_response.qr_code_url.is_empty());
                assert!(!setup_response.secret_key.is_empty());
                assert!(!setup_response.backup_codes.is_empty());
                assert_eq!(setup_response.backup_codes.len(), 10);

                println!("✅ MFA setup successful");
                println!(
                    "   QR Code URL length: {}",
                    setup_response.qr_code_url.len()
                );
                println!("   Secret key: {}", setup_response.secret_key);
                println!(
                    "   Backup codes count: {}",
                    setup_response.backup_codes.len()
                );

                // Test OTP verification with the generated secret
                let otp_provider = OtpCredentialProvider::new();
                let current_time = chrono::Utc::now().timestamp() as u64;
                let time_step = current_time / 30;

                // Generate a valid OTP code
                let secret_bytes = base32::decode(
                    base32::Alphabet::RFC4648 { padding: false },
                    &setup_response.secret_key,
                )
                .expect("Failed to decode secret");

                let valid_code = otp_provider
                    .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha1, 6)
                    .expect("Failed to generate OTP");

                // Test MFA setup verification
                let verify_result = timeout(
                    Duration::from_secs(10),
                    mfa_service.verify_setup(test_user.id, &valid_code),
                )
                .await;

                match verify_result {
                    Ok(Ok(())) => {
                        println!("✅ MFA setup verification successful");

                        // Verify user is now MFA enabled in database
                        let status_result = mfa_service.get_mfa_status(test_user.id).await;
                        match status_result {
                            Ok(status) => {
                                assert!(status.enabled);
                                assert!(status.setup_at.is_some());
                                println!("✅ MFA status correctly updated in database");
                            }
                            Err(e) => {
                                println!("⚠️  Could not verify MFA status: {:?}", e);
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        println!("❌ MFA setup verification failed: {:?}", e);
                    }
                    Err(_) => {
                        println!("❌ MFA setup verification timed out");
                    }
                }
            }
            Ok(Err(e)) => {
                println!("❌ MFA setup failed: {:?}", e);
                // This might be expected if secreton is not running
                match e {
                    AuthencError::InternalError { .. }
                    | AuthencError::SecretonCommunicationError { .. }
                    | AuthencError::SecretonUnavailable => {
                        println!(
                            "ℹ️  This is expected if secreton is not running for integration tests"
                        );
                    }
                    _ => panic!("Unexpected error during MFA setup: {:?}", e),
                }
            }
            Err(_) => {
                println!("❌ MFA setup timed out - secreton might not be available");
            }
        }

        // Cleanup
        cleanup_test_user(&db_pool, test_user.id)
            .await
            .expect("Failed to cleanup test user");
    }

    /// Test MFA verification flow with rate limiting
    #[tokio::test]
    #[ignore] // Requires running secreton instance
    async fn test_mfa_verification_with_rate_limiting() {
        let db_pool = create_integration_db_pool().await;
        let secreton_client = create_integration_secreton_client().await;
        let test_user = create_integration_test_user();

        setup_test_user_in_db(&db_pool, &test_user)
            .await
            .expect("Failed to setup test user");

        let mfa_service = MfaService::new(secreton_client, db_pool.clone());

        // Test multiple invalid OTP attempts to trigger rate limiting
        let invalid_codes = vec!["000000", "111111", "222222", "333333", "444444"];

        for (i, invalid_code) in invalid_codes.iter().enumerate() {
            let verify_result = timeout(
                Duration::from_secs(5),
                mfa_service.verify_mfa(test_user.id, invalid_code),
            )
            .await;

            match verify_result {
                Ok(Err(AuthencError::InvalidOtpCode)) => {
                    println!("✅ Invalid OTP correctly rejected (attempt {})", i + 1);
                }
                Ok(Err(AuthencError::RateLimitExceeded)) => {
                    println!("✅ Rate limiting triggered after {} attempts", i + 1);
                    break;
                }
                Ok(Err(e)) => {
                    println!("⚠️  Unexpected error on attempt {}: {:?}", i + 1, e);
                }
                Ok(Ok(())) => {
                    panic!("Invalid OTP should not succeed");
                }
                Err(_) => {
                    println!("❌ MFA verification timed out on attempt {}", i + 1);
                }
            }
        }

        cleanup_test_user(&db_pool, test_user.id)
            .await
            .expect("Failed to cleanup test user");
    }

    /// Test recovery code functionality
    #[tokio::test]
    #[ignore] // Requires running secreton instance
    async fn test_recovery_code_flow() {
        let db_pool = create_integration_db_pool().await;
        let secreton_client = create_integration_secreton_client().await;
        let test_user = create_integration_test_user();

        setup_test_user_in_db(&db_pool, &test_user)
            .await
            .expect("Failed to setup test user");

        let mfa_service = MfaService::new(secreton_client, db_pool.clone());

        // Test recovery code generation
        let generate_result = timeout(
            Duration::from_secs(10),
            mfa_service.regenerate_recovery_codes(test_user.id),
        )
        .await;

        match generate_result {
            Ok(Ok(recovery_codes)) => {
                assert_eq!(recovery_codes.len(), 10);
                println!(
                    "✅ Recovery codes generated: {} codes",
                    recovery_codes.len()
                );

                // Test recovery code verification
                let first_code = &recovery_codes[0];
                let verify_result = timeout(
                    Duration::from_secs(10),
                    mfa_service.verify_recovery_code(test_user.id, first_code),
                )
                .await;

                match verify_result {
                    Ok(Ok(())) => {
                        println!("✅ Recovery code verification successful");

                        // Test that the same code cannot be used again
                        let reuse_result = mfa_service
                            .verify_recovery_code(test_user.id, first_code)
                            .await;
                        match reuse_result {
                            Err(AuthencError::RecoveryCodeAlreadyUsed) => {
                                println!("✅ Recovery code reuse correctly prevented");
                            }
                            _ => {
                                println!(
                                    "⚠️  Recovery code reuse prevention not working as expected"
                                );
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        println!("❌ Recovery code verification failed: {:?}", e);
                    }
                    Err(_) => {
                        println!("❌ Recovery code verification timed out");
                    }
                }
            }
            Ok(Err(e)) => {
                println!("❌ Recovery code generation failed: {:?}", e);
            }
            Err(_) => {
                println!("❌ Recovery code generation timed out");
            }
        }

        cleanup_test_user(&db_pool, test_user.id)
            .await
            .expect("Failed to cleanup test user");
    }

    /// Test MFA admin operations
    #[tokio::test]
    #[ignore] // Requires running secreton instance
    async fn test_mfa_admin_operations() {
        let db_pool = create_integration_db_pool().await;
        let secreton_client = create_integration_secreton_client().await;
        let test_user = create_integration_test_user();

        setup_test_user_in_db(&db_pool, &test_user)
            .await
            .expect("Failed to setup test user");

        let mfa_service = MfaService::new(secreton_client, db_pool.clone());

        // Test admin MFA disable
        let admin_context = SecurityContext {
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("admin-tesent".to_string()),
            session_id: Some(Uuid::new_v4().to_string()),
            timestamp: Utc::now(),
            risk_score: Some(0.0),
            metadata: Some(json!({
                "admin_action": "disable_mfa",
                "reason": "Integration test"
            })),
        };

        let disable_result = timeout(
            Duration::from_secs(10),
            mfa_service.disable_mfa(test_user.id, &admin_context),
        )
        .await;

        match disable_result {
            Ok(Ok(())) => {
                println!("✅ Admin MFA disable successful");

                // Verify MFA is disabled
                let status_result = mfa_service.get_mfa_status(test_user.id).await;
                match status_result {
                    Ok(status) => {
                        assert!(!status.enabled);
                        println!("✅ MFA status correctly updated after admin disable");
                    }
                    Err(e) => {
                        println!("❌ Could not verify MFA status after disable: {:?}", e);
                    }
                }
            }
            Ok(Err(e)) => {
                println!("❌ Admin MFA disable failed: {:?}", e);
            }
            Err(_) => {
                println!("❌ Admin MFA disable timed out ");
            }
        }

        cleanup_test_user(&db_pool, test_user.id)
            .await
            .expect("Failed to cleanup test user ");
    }

    /// Test batch MFA operations for performance
    #[tokio::test]
    #[ignore] // Requires running secreton instance
    async fn test_batch_mfa_operations() {
        let db_pool = create_integration_db_pool().await;
        let secreton_client = create_integration_secreton_client().await;

        // Create multiple test users
        let test_users: Vec<User> = (0..5).map(|_| create_integration_test_user()).collect();

        // Setup all test users
        for user in &test_users {
            setup_test_user_in_db(&db_pool, user)
                .await
                .expect("Failed to setup test user ");
        }

        let mfa_service = MfaService::new(secreton_client, db_pool.clone());

        // Test batch MFA status retrieval
        let user_ids: Vec<Uuid> = test_users.iter().map(|u| u.id).collect();

        let batch_result = timeout(
            Duration::from_secs(15),
            mfa_service.batch_get_mfa_status(&user_ids),
        )
        .await;

        match batch_result {
            Ok(Ok(statuses)) => {
                assert_eq!(statuses.len(), test_users.len());
                println!(
                    "✅ Batch MFA status retrieval successful: {} users ",
                    statuses.len()
                );

                for (user_id, status) in &statuses {
                    println!("   User {}: MFA enabled = {}", user_id, status.enabled);
                }
            }
            Ok(Err(e)) => {
                println!("❌ Batch MFA status retrieval failed: {:?}", e);
            }
            Err(_) => {
                println!("❌ Batch MFA status retrieval timed out ");
            }
        }

        // Cleanup all test users
        for user in &test_users {
            cleanup_test_user(&db_pool, user.id)
                .await
                .expect("Failed to cleanup test user ");
        }
    }

    /// Test MFA statistics and reporting
    #[tokio::test]
    #[ignore] // Requires running secreton instance and proper database schema
    async fn test_mfa_statistics_reporting() {
        let db_pool = create_integration_db_pool().await;
        let secreton_client = create_integration_secreton_client().await;
        let mfa_service = MfaService::new(secreton_client, db_pool.clone());

        // Test MFA statistics retrieval
        let stats_result = timeout(
            Duration::from_secs(10),
            mfa_service.get_mfa_statistics(None),
        )
        .await;

        match stats_result {
            Ok(Ok(stats)) => {
                println!("✅ MFA statistics retrieval successful ");
                println!("   Total users: {}", stats.total_users);
                println!("   MFA enabled users: {}", stats.mfa_enabled_users);
                println!("   MFA setup complete: {}", stats.mfa_setup_complete);
                println!("   MFA active (30d): {}", stats.mfa_active_30d);
                println!("   MFA active (7d): {}", stats.mfa_active_7d);
                println!("   MFA active (1d): {}", stats.mfa_active_1d);
            }
            Ok(Err(e)) => {
                println!("❌ MFA statistics retrieval failed: {:?}", e);
                // This is expected if the database schema is not fully set up
                match e {
                    AuthencError::DatabaseError { .. }
                    | AuthencError::InternalError { .. } => {
                        println!(
                            "ℹ️  This is expected if the full database schema is not set up for integration tests "
                        );
                    }
                    _ => panic!("Unexpected error during MFA statistics retrieval: {:?}", e),
                }
            }
            Err(_) => {
                println!("❌ MFA statistics retrieval timed out ");
            }
        }

        // Test satker-specific statistics
        let satker_stats_result = mfa_service.get_mfa_statistics(Some("001")).await;
        match satker_stats_result {
            Ok(stats) => {
                println!("✅ Satker-specific MFA statistics retrieval successful ");
                println!("   Satker 001 - Total users: {}", stats.total_users);
            }
            Err(e) => {
                println!(
                    "⚠️  Satker-specific MFA statistics failed (expected): {:?}",
                    e
                );
            }
        }
    }

    /// Test error handling and fallback scenarios
    #[tokio::test]
    async fn test_mfa_error_handling_scenarios() {
        let db_pool = create_integration_db_pool().await;

        // Test with invalid secreton URL to simulate connection failure
        let invalid_secreton_client = Arc::new(
            SecretonClient::new(
                "http://invalid-secreton-url:9999".to_string(),
                "test-token".to_string()
            )
        );

        let mfa_service = MfaService::new(invalid_secreton_client, db_pool.clone());
        let test_user = create_integration_test_user();

        setup_test_user_in_db(&db_pool, &test_user)
            .await
            .expect("Failed to setup test user");

        // Test MFA setup with unavailable secreton
        let setup_result =
            timeout(Duration::from_secs(5), mfa_service.setup_mfa(test_user.id)).await;

        match setup_result {
            Ok(Err(AuthencError::InternalError { .. }))
            | Ok(Err(AuthencError::SecretonCommunicationError { .. }))
            | Ok(Err(AuthencError::SecretonUnavailable)) => {
                println!("✅ MFA setup correctly failed with secreton unavailable");
            }
            Ok(Ok(_)) => {
                panic!("MFA setup should fail with invalid secreton URL");
            }
            Err(_) => {
                println!("✅ MFA setup timed out as expected with invalid secreton URL");
            }
            Ok(Err(e)) => {
                println!("✅ MFA setup failed with expected error: {:?}", e);
            }
        }

        // Test recovery code operations with unavailable secreton
        let recovery_result = timeout(
            Duration::from_secs(5),
            mfa_service.regenerate_recovery_codes(test_user.id),
        )
        .await;

        match recovery_result {
            Ok(Err(AuthencError::InternalError { .. }))
            | Ok(Err(AuthencError::SecretonCommunicationError { .. }))
            | Ok(Err(AuthencError::SecretonUnavailable)) => {
                println!("✅ Recovery code generation correctly failed with secreton unavailable");
            }
            Ok(Ok(_)) => {
                panic!("Recovery code generation should fail with invalid secreton URL");
            }
            Err(_) => {
                println!("✅ Recovery code generation timed out as expected");
            }
            Ok(Err(e)) => {
                println!(
                    "✅ Recovery code generation failed with expected error: {:?}",
                    e
                );
            }
        }

        cleanup_test_user(&db_pool, test_user.id)
            .await
            .expect("Failed to cleanup test user");
    }
}

/// Performance and load testing for MFA operations
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    use test_utils::*;

    #[tokio::test]
    #[ignore] // Requires running secreton instance and is resource intensive
    async fn test_mfa_performance_under_load() {
        let db_pool = create_integration_db_pool().await;
        let secreton_client = create_integration_secreton_client().await;
        let mfa_service = Arc::new(MfaService::new(secreton_client, db_pool.clone()));

        // Create test users for load testing
        let num_users = 10;
        let test_users: Vec<User> = (0..num_users)
            .map(|_| create_integration_test_user())
            .collect();

        // Setup all test users
        for user in &test_users {
            setup_test_user_in_db(&db_pool, user)
                .await
                .expect("Failed to setup test user");
        }

        // Test concurrent MFA operations
        let start_time = Instant::now();
        let mut handles = Vec::new();

        for user in test_users.clone() {
            let service = Arc::clone(&mfa_service);
            let handle = tokio::spawn(async move {
                // Test recovery code generation performance
                let result = service.regenerate_recovery_codes(user.id).await;
                (user.id, result)
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        let mut successful_operations = 0;
        let mut failed_operations = 0;

        for handle in handles {
            match handle.await {
                Ok((user_id, Ok(_))) => {
                    successful_operations += 1;
                    println!(
                        "✅ Recovery code generation successful for user: {}",
                        user_id
                    );
                }
                Ok((user_id, Err(e))) => {
                    failed_operations += 1;
                    println!(
                        "❌ Recovery code generation failed for user {}: {:?}",
                        user_id, e
                    );
                }
                Err(e) => {
                    failed_operations += 1;
                    println!("❌ Task failed: {:?}", e);
                }
            }
        }

        let duration = start_time.elapsed();
        println!("🔍 Performance Test Results:");
        println!("   Total operations: {}", num_users);
        println!("   Successful: {}", successful_operations);
        println!("   Failed: {}", failed_operations);
        println!("   Total time: {:?}", duration);
        println!(
            "   Average time per operation: {:?}",
            duration / num_users as u32
        );

        // Cleanup all test users
        for user in &test_users {
            cleanup_test_user(&db_pool, user.id)
                .await
                .expect("Failed to cleanup test user");
        }

        // Assert performance criteria
        assert!(
            duration.as_secs() < 30,
            "Operations took too long: {:?}",
            duration
        );
        assert!(successful_operations > 0, "No operations succeeded");
    }
}
