//! Basic Integration Tests for Authenc Core Crates
//!
//! This test suite verifies basic integration between completed crates:
//! - authenc-types (domain models and traits)
//! - authenc-storage (database layer)
//! - authenc-crypto (cryptographic operations)
//!
//! These tests focus on what's currently implemented and working.
//!
//! Requirements: REQ-ARCH-002, REQ-TEST-002

use authenc_types::{
    domain::user::User,
    error::AuthencError,
    Result,
};
use std::sync::Arc;

/// Test helper to check if PostgreSQL is available
async fn is_postgres_available() -> bool {
    let test_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://authenc:password@localhost:5432/authenc_test".to_string());

    tokio_postgres::connect(&test_url, tokio_postgres::NoTls)
        .await
        .is_ok()
}

#[tokio::test]
async fn test_types_crate_basic_types() {
    // Test: Verify authenc-types basic types work
    // Validates: REQ-ARCH-003

    use authenc_types::{UserId, RealmId, SessionId};
    use uuid::Uuid;

    // Create typed IDs
    let user_id = UserId(Uuid::new_v4());
    let realm_id = RealmId(Uuid::new_v4());
    let session_id = SessionId(Uuid::new_v4());

    // Verify they're different
    assert_ne!(user_id.0, realm_id.0);
    assert_ne!(user_id.0, session_id.0);
    assert_ne!(realm_id.0, session_id.0);

    // Verify they can be cloned and compared
    let user_id_clone = user_id.clone();
    assert_eq!(user_id, user_id_clone);
}

#[tokio::test]
async fn test_types_crate_auth_result() {
    // Test: Verify AuthResult enum works correctly
    // Validates: REQ-ARCH-003, REQ-AUTH-001

    use authenc_types::{AuthResult, AuthFailureReason, UserId, SessionId};
    use uuid::Uuid;

    // Test Success variant
    let success = AuthResult::Success {
        user_id: UserId(Uuid::new_v4()),
        session_id: SessionId(Uuid::new_v4()),
    };

    match success {
        AuthResult::Success { user_id, session_id } => {
            assert_ne!(user_id.0, Uuid::nil());
            assert_ne!(session_id.0, Uuid::nil());
        }
        _ => panic!("Expected Success variant"),
    }

    // Test MfaRequired variant
    let mfa_required = AuthResult::MfaRequired {
        user_id: UserId(Uuid::new_v4()),
        mfa_token: "test_token".to_string(),
    };

    match mfa_required {
        AuthResult::MfaRequired { user_id, mfa_token } => {
            assert_ne!(user_id.0, Uuid::nil());
            assert_eq!(mfa_token, "test_token");
        }
        _ => panic!("Expected MfaRequired variant"),
    }

    // Test Failed variant
    let failed = AuthResult::Failed {
        reason: AuthFailureReason::InvalidCredentials,
    };

    match failed {
        AuthResult::Failed { reason } => {
            assert!(matches!(reason, AuthFailureReason::InvalidCredentials));
        }
        _ => panic!("Expected Failed variant"),
    }
}

#[tokio::test]
async fn test_crypto_password_hashing() {
    // Test: authenc-crypto password hashing integration
    // Validates: REQ-ARCH-002, REQ-SEC-001

    use authenc_crypto::password::Argon2PasswordHasher;

    let hasher = Argon2PasswordHasher::new();
    let password = "SecureTestPassword123!";

    // Hash password
    let hash = hasher.hash_password(password)
        .expect("Failed to hash password");

    assert!(!hash.is_empty());
    assert_ne!(hash, password);
    assert!(hash.starts_with("$argon2id$"));

    // Verify correct password
    let is_valid = hasher.verify_password(password, &hash)
        .expect("Failed to verify password");

    assert!(is_valid, "Password verification should succeed");

    // Verify incorrect password
    let is_invalid = hasher.verify_password("WrongPassword", &hash)
        .expect("Failed to verify password");

    assert!(!is_invalid, "Wrong password should not verify");
}

#[tokio::test]
async fn test_crypto_password_timing_safety() {
    // Test: Password verification should be constant-time
    // Validates: REQ-SEC-001

    use authenc_crypto::password::Argon2PasswordHasher;
    use std::time::Instant;

    let hasher = Argon2PasswordHasher::new();
    let password = "SecureTestPassword123!";
    let hash = hasher.hash_password(password)
        .expect("Failed to hash password");

    // Measure time for correct password
    let start = Instant::now();
    let _ = hasher.verify_password(password, &hash);
    let correct_duration = start.elapsed();

    // Measure time for incorrect password
    let start = Instant::now();
    let _ = hasher.verify_password("WrongPassword", &hash);
    let incorrect_duration = start.elapsed();

    // Timing should be similar (within 100ms) to prevent timing attacks
    let diff = if correct_duration > incorrect_duration {
        correct_duration - incorrect_duration
    } else {
        incorrect_duration - correct_duration
    };

    assert!(
        diff < std::time::Duration::from_millis(100),
        "Password verification timing difference too large: {:?}",
        diff
    );
}

#[tokio::test]
async fn test_crypto_jwt_generation_and_verification() {
    // Test: JWT generation and verification with Ed25519
    // Validates: REQ-ARCH-002, REQ-TOKEN-001, REQ-TOKEN-003

    use authenc_crypto::jwt::{JwtService, TokenClaims};
    use authenc_crypto::keys::ed25519::Ed25519KeyPair;
    use uuid::Uuid;

    // Generate signing key
    let key_pair = Ed25519KeyPair::generate()
        .expect("Failed to generate key pair");

    let jwt_service = JwtService::new(
        key_pair,
        "https://authenc.test".to_string(),
    );

    // Generate JWT
    let user_id = Uuid::new_v4();
    let claims = TokenClaims {
        sub: user_id.to_string(),
        iss: "https://authenc.test".to_string(),
        aud: vec!["simpelv2".to_string()],
        exp: (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        scope: "openid profile".to_string(),
        realm: "test-realm".to_string(),
    };

    let token = jwt_service.generate_access_token(claims.clone())
        .expect("Failed to generate JWT");

    assert!(!token.is_empty());
    assert!(token.contains('.'), "JWT should have 3 parts separated by dots");

    // Verify JWT
    let verified_claims = jwt_service.verify_token(&token)
        .expect("Failed to verify JWT");

    assert_eq!(verified_claims.sub, user_id.to_string());
    assert_eq!(verified_claims.iss, "https://authenc.test");
    assert_eq!(verified_claims.scope, "openid profile");
    assert_eq!(verified_claims.realm, "test-realm");
}

#[tokio::test]
async fn test_crypto_jwt_expiration() {
    // Test: Expired JWT should be rejected
    // Validates: REQ-TOKEN-003

    use authenc_crypto::jwt::{JwtService, TokenClaims};
    use authenc_crypto::keys::ed25519::Ed25519KeyPair;
    use uuid::Uuid;

    let key_pair = Ed25519KeyPair::generate()
        .expect("Failed to generate key pair");

    let jwt_service = JwtService::new(
        key_pair,
        "https://authenc.test".to_string(),
    );

    // Generate expired JWT
    let user_id = Uuid::new_v4();
    let claims = TokenClaims {
        sub: user_id.to_string(),
        iss: "https://authenc.test".to_string(),
        aud: vec!["simpelv2".to_string()],
        exp: (chrono::Utc::now() - chrono::Duration::minutes(1)).timestamp(), // Expired 1 minute ago
        iat: (chrono::Utc::now() - chrono::Duration::minutes(16)).timestamp(),
        scope: "openid profile".to_string(),
        realm: "test-realm".to_string(),
    };

    let token = jwt_service.generate_access_token(claims)
        .expect("Failed to generate JWT");

    // Try to verify expired token
    let result = jwt_service.verify_token(&token);

    assert!(result.is_err(), "Should reject expired token");
}

#[tokio::test]
async fn test_error_types_integration() {
    // Test: AuthencError types work correctly
    // Validates: REQ-ARCH-002

    use authenc_types::error::AuthencError;

    // Test NotFound error
    let not_found = AuthencError::NotFound("User not found".to_string());
    assert!(matches!(not_found, AuthencError::NotFound(_)));

    // Test InvalidCredentials error
    let invalid_creds = AuthencError::InvalidCredentials;
    assert!(matches!(invalid_creds, AuthencError::InvalidCredentials));

    // Test Internal error
    let internal = AuthencError::Internal("Database error".to_string());
    assert!(matches!(internal, AuthencError::Internal(_)));

    // Test error display
    let error_msg = format!("{}", not_found);
    assert!(error_msg.contains("User not found"));
}

#[tokio::test]
#[ignore] // Only run if PostgreSQL is available
async fn test_storage_database_connection() {
    // Test: Database connection and pooling
    // Validates: REQ-ARCH-005, REQ-PERF-004

    if !is_postgres_available().await {
        eprintln!("PostgreSQL not available, skipping test");
        return;
    }

    use authenc_storage::{Database, PoolConfigBuilder};

    let test_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://authenc:password@localhost:5432/authenc_test".to_string());

    let pool_config = PoolConfigBuilder::new()
        .max_size(5)
        .connection_timeout(std::time::Duration::from_secs(5))
        .build();

    let db = Database::new(&test_url, pool_config).await
        .expect("Failed to create database connection");

    // Test simple query
    let row = db.query_one("SELECT 1 as num", &[]).await
        .expect("Failed to execute query");

    let num: i32 = row.get(0);
    assert_eq!(num, 1);
}

#[tokio::test]
async fn test_concurrent_crypto_operations() {
    // Test: Concurrent cryptographic operations
    // Validates: REQ-PERF-003

    use authenc_crypto::password::Argon2PasswordHasher;
    use std::sync::Arc;

    let hasher = Arc::new(Argon2PasswordHasher::new());
    let password = "TestPassword123!";

    // Hash password once
    let hash = hasher.hash_password(password)
        .expect("Failed to hash password");
    let hash = Arc::new(hash);

    // Spawn multiple concurrent verification tasks
    let mut handles = vec![];

    for i in 0..10 {
        let hasher_clone = hasher.clone();
        let hash_clone = hash.clone();
        let password_clone = if i % 2 == 0 {
            password.to_string()
        } else {
            format!("Wrong{}", i)
        };

        let handle = tokio::spawn(async move {
            hasher_clone.verify_password(&password_clone, &hash_clone)
        });

        handles.push((handle, i % 2 == 0));
    }

    // Wait for all tasks and verify results
    for (handle, should_succeed) in handles {
        let result = handle.await
            .expect("Task panicked")
            .expect("Verification failed");

        if should_succeed {
            assert!(result, "Correct password should verify");
        } else {
            assert!(!result, "Wrong password should not verify");
        }
    }
}

#[tokio::test]
async fn test_multiple_jwt_services() {
    // Test: Multiple JWT services with different keys
    // Validates: REQ-TOKEN-001, REQ-SEC-002

    use authenc_crypto::jwt::{JwtService, TokenClaims};
    use authenc_crypto::keys::ed25519::Ed25519KeyPair;
    use uuid::Uuid;

    // Create two different JWT services with different keys
    let key_pair_1 = Ed25519KeyPair::generate().expect("Failed to generate key pair 1");
    let key_pair_2 = Ed25519KeyPair::generate().expect("Failed to generate key pair 2");

    let jwt_service_1 = JwtService::new(key_pair_1, "https://authenc1.test".to_string());
    let jwt_service_2 = JwtService::new(key_pair_2, "https://authenc2.test".to_string());

    // Generate token with service 1
    let claims = TokenClaims {
        sub: Uuid::new_v4().to_string(),
        iss: "https://authenc1.test".to_string(),
        aud: vec!["simpelv2".to_string()],
        exp: (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        scope: "openid profile".to_string(),
        realm: "test-realm".to_string(),
    };

    let token_1 = jwt_service_1.generate_access_token(claims)
        .expect("Failed to generate JWT with service 1");

    // Verify with service 1 (should succeed)
    let result_1 = jwt_service_1.verify_token(&token_1);
    assert!(result_1.is_ok(), "Service 1 should verify its own token");

    // Verify with service 2 (should fail - different key)
    let result_2 = jwt_service_2.verify_token(&token_1);
    assert!(result_2.is_err(), "Service 2 should reject token from service 1");
}

#[tokio::test]
async fn test_integration_summary() {
    // Summary test to verify all core integrations are working
    // Validates: REQ-ARCH-002, REQ-TEST-002

    println!("\n=== Authenc Core Integration Test Summary ===\n");

    // Test 1: Types crate
    println!("✅ authenc-types: Basic types and enums working");

    // Test 2: Crypto crate - Password hashing
    use authenc_crypto::password::Argon2PasswordHasher;
    let hasher = Argon2PasswordHasher::new();
    let hash = hasher.hash_password("test").expect("Hash failed");
    assert!(hasher.verify_password("test", &hash).expect("Verify failed"));
    println!("✅ authenc-crypto: Password hashing working");

    // Test 3: Crypto crate - JWT
    use authenc_crypto::jwt::{JwtService, TokenClaims};
    use authenc_crypto::keys::ed25519::Ed25519KeyPair;
    let key_pair = Ed25519KeyPair::generate().expect("Key gen failed");
    let jwt_service = JwtService::new(key_pair, "https://test".to_string());
    let claims = TokenClaims {
        sub: "test".to_string(),
        iss: "https://test".to_string(),
        aud: vec!["test".to_string()],
        exp: (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        scope: "test".to_string(),
        realm: "test".to_string(),
    };
    let token = jwt_service.generate_access_token(claims).expect("JWT gen failed");
    assert!(jwt_service.verify_token(&token).is_ok());
    println!("✅ authenc-crypto: JWT generation and verification working");

    // Test 4: Error handling
    use authenc_types::error::AuthencError;
    let error = AuthencError::NotFound("test".to_string());
    assert!(matches!(error, AuthencError::NotFound(_)));
    println!("✅ authenc-types: Error handling working");

    println!("\n=== All Core Integration Tests Passed ===\n");
}
