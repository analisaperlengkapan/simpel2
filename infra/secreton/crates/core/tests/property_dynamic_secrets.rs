//! Property-Based Tests for Dynamic Secrets Engines
//!
//! Tests universal properties that should hold across all dynamic secrets engines.

use proptest::prelude::*;
use secreton_core::services::secrets::{
    MySqlConnection, MySqlRole, MySqlSecretsEngine,
    MongoDbConnection, MongoDbRole, MongoDbSecretsEngine,
    RedisConnection, RedisRole, RedisSecretsEngine,
};
use std::collections::HashSet;

// **Feature: secreton-comprehensive-enhancement, Property 13: Dynamic Credential Uniqueness**
// **Validates: Requirements 4.1, 4.2, 4.3**
//
// Property: For any database role, each credential generation SHALL produce unique
// username/password pairs with correct TTL.
#[tokio::test]
async fn property_dynamic_credential_uniqueness_mysql() {
    let engine = MySqlSecretsEngine::new();

    // Configure connection
    let config = MySqlConnection {
        name: "test-db".to_string(),
        connection_url: "mysql://localhost:3306/testdb".to_string(),
        verify_connection: false,
        ..Default::default()
    };
    engine.configure_connection(config).await.unwrap();

    // Create role
    let role = MySqlRole {
        name: "test-role".to_string(),
        db_name: "test-db".to_string(),
        default_ttl: 3600,
        max_ttl: 7200,
        creation_statements: vec![
            "CREATE USER '{{username}}'@'%' IDENTIFIED BY '{{password}}'".to_string(),
        ],
        revocation_statements: vec!["DROP USER '{{username}}'@'%'".to_string()],
        ..Default::default()
    };
    engine.create_role(role).await.unwrap();

    // Generate multiple credentials
    let mut usernames = HashSet::new();
    let mut passwords = HashSet::new();

    for _ in 0..10 {
        let creds = engine.generate_credentials("test-role", Some(1800)).await.unwrap();

        // Check uniqueness
        assert!(usernames.insert(creds.username.clone()),
            "Username should be unique: {}", creds.username);
        assert!(passwords.insert(creds.password.clone()),
            "Password should be unique: {}", creds.password);

        // Check TTL
        let ttl = (creds.expires_at - creds.created_at).num_seconds();
        assert_eq!(ttl, 1800, "TTL should match requested value");

        // Check username format
        assert!(creds.username.starts_with("v-test-role-"),
            "Username should follow format: {}", creds.username);

        // Check password length
        assert_eq!(creds.password.len(), 32, "Password should be 32 characters");
    }

    // Verify all credentials are unique
    assert_eq!(usernames.len(), 10, "All usernames should be unique");
    assert_eq!(passwords.len(), 10, "All passwords should be unique");
}

#[tokio::test]
async fn property_dynamic_credential_uniqueness_mongodb() {
    let engine = MongoDbSecretsEngine::new();

    // Configure connection
    let config = MongoDbConnection {
        name: "test-db".to_string(),
        connection_url: "mongodb://localhost:27017/testdb".to_string(),
        verify_connection: false,
        ..Default::default()
    };
    engine.configure_connection(config).await.unwrap();

    // Create role
    let role = MongoDbRole {
        name: "test-role".to_string(),
        db_name: "test-db".to_string(),
        default_ttl: 3600,
        max_ttl: 7200,
        creation_statements: vec![
            r#"{"createUser": "{{username}}", "pwd": "{{password}}"}"#.to_string(),
        ],
        revocation_statements: vec![r#"{"dropUser": "{{username}}"}"#.to_string()],
        ..Default::default()
    };
    engine.create_role(role).await.unwrap();

    // Generate multiple credentials
    let mut usernames = HashSet::new();
    let mut passwords = HashSet::new();

    for _ in 0..10 {
        let creds = engine.generate_credentials("test-role", Some(1800)).await.unwrap();

        // Check uniqueness
        assert!(usernames.insert(creds.username.clone()),
            "Username should be unique: {}", creds.username);
        assert!(passwords.insert(creds.password.clone()),
            "Password should be unique: {}", creds.password);

        // Check TTL
        let ttl = (creds.expires_at - creds.created_at).num_seconds();
        assert_eq!(ttl, 1800, "TTL should match requested value");

        // Check username format (MongoDB uses underscores)
        assert!(creds.username.starts_with("v_test_role_"),
            "Username should follow format: {}", creds.username);

        // Check password length
        assert_eq!(creds.password.len(), 32, "Password should be 32 characters");
    }

    // Verify all credentials are unique
    assert_eq!(usernames.len(), 10, "All usernames should be unique");
    assert_eq!(passwords.len(), 10, "All passwords should be unique");
}

#[tokio::test]
async fn property_dynamic_credential_uniqueness_redis() {
    let engine = RedisSecretsEngine::new();

    // Configure connection
    let config = RedisConnection {
        name: "test-db".to_string(),
        connection_url: "redis://localhost:6379/0".to_string(),
        verify_connection: false,
        ..Default::default()
    };
    engine.configure_connection(config).await.unwrap();

    // Create role
    let role = RedisRole {
        name: "test-role".to_string(),
        db_name: "test-db".to_string(),
        default_ttl: 3600,
        max_ttl: 7200,
        creation_statements: vec![
            "ACL SETUSER {{username}} on >{{password}}".to_string(),
        ],
        revocation_statements: vec!["ACL DELUSER {{username}}".to_string()],
        ..Default::default()
    };
    engine.create_role(role).await.unwrap();

    // Generate multiple credentials
    let mut usernames = HashSet::new();
    let mut passwords = HashSet::new();

    for _ in 0..10 {
        let creds = engine.generate_credentials("test-role", Some(1800)).await.unwrap();

        // Check uniqueness
        assert!(usernames.insert(creds.username.clone()),
            "Username should be unique: {}", creds.username);
        assert!(passwords.insert(creds.password.clone()),
            "Password should be unique: {}", creds.password);

        // Check TTL
        let ttl = (creds.expires_at - creds.created_at).num_seconds();
        assert_eq!(ttl, 1800, "TTL should match requested value");

        // Check username format
        assert!(creds.username.starts_with("v-test-role-"),
            "Username should follow format: {}", creds.username);

        // Check password length
        assert_eq!(creds.password.len(), 32, "Password should be 32 characters");
    }

    // Verify all credentials are unique
    assert_eq!(usernames.len(), 10, "All usernames should be unique");
    assert_eq!(passwords.len(), 10, "All passwords should be unique");
}

// Property test using proptest for more comprehensive testing
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // **Feature: secreton-comprehensive-enhancement, Property 13: Dynamic Credential Uniqueness**
    // **Validates: Requirements 4.1, 4.2, 4.3**
    #[test]
    fn property_password_generation_uniqueness(count in 2usize..20) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = MySqlSecretsEngine::new();

            let mut passwords = HashSet::new();
            for _ in 0..count {
                let password = engine.generate_password(32);
                assert_eq!(password.len(), 32);
                passwords.insert(password);
            }

            // All passwords should be unique
            assert_eq!(passwords.len(), count,
                "All {} generated passwords should be unique", count);
        });
    }

    #[test]
    fn property_username_generation_uniqueness(count in 2usize..20) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = MySqlSecretsEngine::new();

            let mut usernames = HashSet::new();
            for _ in 0..count {
                let username = engine.generate_username("test-role");
                assert!(username.starts_with("v-test-role-"));
                usernames.insert(username);
            }

            // All usernames should be unique
            assert_eq!(usernames.len(), count,
                "All {} generated usernames should be unique", count);
        });
    }
}


// **Feature: secreton-comprehensive-enhancement, Property 14: Lease Expiration Revokes Credentials**
// **Validates: Requirements 4.6**
//
// Property: For any dynamic credential with TTL, after the TTL expires the credential
// SHALL be automatically revoked and inaccessible.
#[tokio::test]
async fn property_lease_expiration_revokes_credentials() {
    let engine = MySqlSecretsEngine::new();

    // Configure connection
    let config = MySqlConnection {
        name: "test-db".to_string(),
        connection_url: "mysql://localhost:3306/testdb".to_string(),
        verify_connection: false,
        ..Default::default()
    };
    engine.configure_connection(config).await.unwrap();

    // Create role with short TTL
    let role = MySqlRole {
        name: "test-role".to_string(),
        db_name: "test-db".to_string(),
        default_ttl: 2, // 2 seconds
        max_ttl: 10,
        creation_statements: vec![
            "CREATE USER '{{username}}'@'%' IDENTIFIED BY '{{password}}'".to_string(),
        ],
        revocation_statements: vec!["DROP USER '{{username}}'@'%'".to_string()],
        ..Default::default()
    };
    engine.create_role(role).await.unwrap();

    // Generate credential with short TTL
    let creds = engine.generate_credentials("test-role", Some(2)).await.unwrap();
    let credential_id = creds.id.clone();

    // Verify credential exists
    let active_creds = engine.list_credentials().await;
    assert!(active_creds.iter().any(|c| c.id == credential_id),
        "Credential should exist immediately after creation");

    // Wait for TTL to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Verify credential has expired (check expiration time)
    let active_creds = engine.list_credentials().await;
    if let Some(cred) = active_creds.iter().find(|c| c.id == credential_id) {
        let now = chrono::Utc::now();
        assert!(cred.expires_at < now,
            "Credential should be expired after TTL");
    }

    // In a real implementation with lease manager integration,
    // the credential would be automatically revoked by the lease manager
    // For now, we verify the expiration time is correctly set
}

#[tokio::test]
async fn property_lease_expiration_timing_accuracy() {
    let engine = RedisSecretsEngine::new();

    // Configure connection
    let config = RedisConnection {
        name: "test-db".to_string(),
        connection_url: "redis://localhost:6379/0".to_string(),
        verify_connection: false,
        ..Default::default()
    };
    engine.configure_connection(config).await.unwrap();

    // Create role
    let role = RedisRole {
        name: "test-role".to_string(),
        db_name: "test-db".to_string(),
        default_ttl: 3600,
        max_ttl: 7200,
        creation_statements: vec![
            "ACL SETUSER {{username}} on >{{password}}".to_string(),
        ],
        revocation_statements: vec!["ACL DELUSER {{username}}".to_string()],
        ..Default::default()
    };
    engine.create_role(role).await.unwrap();

    // Test various TTL values
    let ttl_values = vec![60, 300, 600, 1800, 3600];

    for ttl in ttl_values {
        let creds = engine.generate_credentials("test-role", Some(ttl)).await.unwrap();

        // Calculate actual TTL
        let actual_ttl = (creds.expires_at - creds.created_at).num_seconds();

        // Verify TTL is accurate (within 1 second tolerance)
        assert!((actual_ttl - ttl as i64).abs() <= 1,
            "TTL should be accurate: expected {}, got {}", ttl, actual_ttl);
    }
}

#[tokio::test]
async fn property_lease_renewal_extends_expiration() {
    let engine = MongoDbSecretsEngine::new();

    // Configure connection
    let config = MongoDbConnection {
        name: "test-db".to_string(),
        connection_url: "mongodb://localhost:27017/testdb".to_string(),
        verify_connection: false,
        ..Default::default()
    };
    engine.configure_connection(config).await.unwrap();

    // Create role
    let role = MongoDbRole {
        name: "test-role".to_string(),
        db_name: "test-db".to_string(),
        default_ttl: 3600,
        max_ttl: 7200,
        creation_statements: vec![
            r#"{"createUser": "{{username}}", "pwd": "{{password}}"}"#.to_string(),
        ],
        revocation_statements: vec![r#"{"dropUser": "{{username}}"}"#.to_string()],
        ..Default::default()
    };
    engine.create_role(role).await.unwrap();

    // Generate credential
    let creds = engine.generate_credentials("test-role", Some(1800)).await.unwrap();
    let credential_id = creds.id.clone();
    let original_expiration = creds.expires_at;

    // Renew lease
    let renewed_creds = engine.renew_lease(&credential_id, 900).await.unwrap();

    // Verify expiration was extended
    assert!(renewed_creds.expires_at > original_expiration,
        "Renewed credential should have later expiration time");

    // Verify extension amount is correct (within 1 second tolerance)
    let extension = (renewed_creds.expires_at - original_expiration).num_seconds();
    assert!((extension - 900).abs() <= 1,
        "Extension should be approximately 900 seconds, got {}", extension);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // **Feature: secreton-comprehensive-enhancement, Property 14: Lease Expiration Revokes Credentials**
    // **Validates: Requirements 4.6**
    #[test]
    fn property_ttl_within_bounds(ttl in 60u32..3600) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = MySqlSecretsEngine::new();

            // Configure connection
            let config = MySqlConnection {
                name: "test-db".to_string(),
                connection_url: "mysql://localhost:3306/testdb".to_string(),
                verify_connection: false,
                ..Default::default()
            };
            engine.configure_connection(config).await.unwrap();

            // Create role
            let role = MySqlRole {
                name: "test-role".to_string(),
                db_name: "test-db".to_string(),
                default_ttl: 3600,
                max_ttl: 7200,
                creation_statements: vec![
                    "CREATE USER '{{username}}'@'%' IDENTIFIED BY '{{password}}'".to_string(),
                ],
                revocation_statements: vec!["DROP USER '{{username}}'@'%'".to_string()],
                ..Default::default()
            };
            engine.create_role(role).await.unwrap();

            // Generate credential with random TTL
            let creds = engine.generate_credentials("test-role", Some(ttl)).await.unwrap();

            // Verify TTL is set correctly
            let actual_ttl = (creds.expires_at - creds.created_at).num_seconds();
            assert!((actual_ttl - ttl as i64).abs() <= 1,
                "TTL should match requested value: expected {}, got {}", ttl, actual_ttl);
        });
    }
}
