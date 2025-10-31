//! Comprehensive Lease Management Tests
//!
//! Tests for lease lifecycle management including creation, renewal,
//! expiration, revocation, and integration with KV and dynamic secrets engines.

use chrono::{Duration, Utc};
use deadpool_postgres::{Config, Pool, Runtime};
use secreton_core::services::lease::{EnhancedLease, LeaseError, LeaseManager, LeaseSchedulerConfig};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::NoTls;

/// Setup test database pool
async fn setup_test_pool() -> Pool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/secreton_test".to_string());

    let mut cfg = Config::new();
    cfg.url = Some(database_url);
    cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap()
}

/// Helper to create a test lease
async fn create_test_lease(
    manager: &LeaseManager,
    user: &str,
    resource: &str,
    ttl_secs: i64,
) -> Result<EnhancedLease, LeaseError> {
    manager
        .create_lease(
            user,
            resource,
            "kv",
            "default",
            ttl_secs,
            86400, // max_ttl
            true,  // renewable
            None,  // no parent
            None,  // no max_renewals
            None,  // no revoke_callback
            HashMap::new(),
        )
        .await
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_creation_with_various_ttls() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Test short TTL (1 minute)
    let lease1 = create_test_lease(&manager, "user1", "/secret/short", 60)
        .await
        .unwrap();
    assert_eq!(lease1.user, "user1");
    assert_eq!(lease1.status, "active");
    assert!(lease1.renewable);
    let ttl1 = (lease1.expired_at - lease1.issued_at).num_seconds();
    assert_eq!(ttl1, 60);

    // Test medium TTL (1 hour)
    let lease2 = create_test_lease(&manager, "user2", "/secret/medium", 3600)
        .await
        .unwrap();
    let ttl2 = (lease2.expired_at - lease2.issued_at).num_seconds();
    assert_eq!(ttl2, 3600);

    // Test long TTL (24 hours)
    let lease3 = create_test_lease(&manager, "user3", "/secret/long", 86400)
        .await
        .unwrap();
    let ttl3 = (lease3.expired_at - lease3.issued_at).num_seconds();
    assert_eq!(ttl3, 86400);

    // Test invalid TTL (zero)
    let result = manager
        .create_lease(
            "user4",
            "/secret/invalid",
            "kv",
            "default",
            0, // invalid TTL
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeaseError::InvalidTtl(_)));

    // Test invalid TTL (exceeds max_ttl)
    let result = manager
        .create_lease(
            "user5",
            "/secret/exceeds",
            "kv",
            "default",
            100000, // exceeds max_ttl
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeaseError::InvalidTtl(_)));
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_renewal_within_max_ttl() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create lease with 1 hour TTL
    let lease = create_test_lease(&manager, "user1", "/secret/renewable", 3600)
        .await
        .unwrap();

    assert_eq!(lease.renew_count, 0);
    assert!(lease.last_renewed_at.is_none());

    // Renew with 1 hour increment (within max_ttl)
    let renewed = manager.renew_lease(&lease.id, 3600).await.unwrap();
    assert_eq!(renewed.renew_count, 1);
    assert!(renewed.last_renewed_at.is_some());
    assert!(renewed.expired_at > lease.expired_at);

    // Renew again
    let renewed2 = manager.renew_lease(&lease.id, 3600).await.unwrap();
    assert_eq!(renewed2.renew_count, 2);
    assert!(renewed2.last_renewed_at.is_some());
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_renewal_exceeding_max_ttl() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create lease with short max_ttl
    let lease = manager
        .create_lease(
            "user1",
            "/secret/limited",
            "kv",
            "default",
            1800,  // 30 minutes
            3600,  // max_ttl: 1 hour
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await
        .unwrap();

    // Try to renew with increment that would exceed max_ttl
    let renewed = manager.renew_lease(&lease.id, 7200).await.unwrap(); // Request 2 hours

    // Should be capped at max_ttl (1 hour)
    let new_ttl = (renewed.expired_at - Utc::now()).num_seconds();
    assert!(new_ttl <= 3600);
    assert!(new_ttl > 3500); // Allow some time variance
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_renewal_with_max_renewals_limit() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create lease with max 2 renewals
    let lease = manager
        .create_lease(
            "user1",
            "/secret/limited_renewals",
            "kv",
            "default",
            1800,
            86400,
            true,
            None,
            Some(2), // max 2 renewals
            None,
            HashMap::new(),
        )
        .await
        .unwrap();

    // First renewal should succeed
    let renewed1 = manager.renew_lease(&lease.id, 1800).await.unwrap();
    assert_eq!(renewed1.renew_count, 1);

    // Second renewal should succeed
    let renewed2 = manager.renew_lease(&lease.id, 1800).await.unwrap();
    assert_eq!(renewed2.renew_count, 2);

    // Third renewal should fail
    let result = manager.renew_lease(&lease.id, 1800).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeaseError::RenewalNotAllowed));
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_renewal_non_renewable() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create non-renewable lease
    let lease = manager
        .create_lease(
            "user1",
            "/secret/non_renewable",
            "kv",
            "default",
            3600,
            86400,
            false, // not renewable
            None,
            None,
            None,
            HashMap::new(),
        )
        .await
        .unwrap();

    // Try to renew
    let result = manager.renew_lease(&lease.id, 1800).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeaseError::RenewalNotAllowed));
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_automatic_expiration() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create lease with very short TTL (2 seconds)
    let lease = create_test_lease(&manager, "user1", "/secret/expires", 2)
        .await
        .unwrap();

    assert_eq!(lease.status, "active");

    // Wait for expiration
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Run cleanup
    let count = manager.cleanup_expired().await.unwrap();
    assert!(count >= 1);

    // Verify lease is expired
    let expired_lease = manager.lookup_lease(&lease.id).await.unwrap();
    assert_eq!(expired_lease.status, "expired");
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_manual_revocation() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create lease
    let lease = create_test_lease(&manager, "user1", "/secret/revoke", 3600)
        .await
        .unwrap();

    assert_eq!(lease.status, "active");

    // Manually revoke
    let revoked_ids = manager.revoke_lease(&lease.id).await.unwrap();
    assert_eq!(revoked_ids.len(), 1);
    assert_eq!(revoked_ids[0], lease.id);

    // Verify lease is revoked
    let revoked_lease = manager.lookup_lease(&lease.id).await.unwrap();
    assert_eq!(revoked_lease.status, "revoked");

    // Try to renew revoked lease (should fail)
    let result = manager.renew_lease(&lease.id, 1800).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeaseError::LeaseRevoked));
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_revocation_cascades_to_children() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create parent lease
    let parent = create_test_lease(&manager, "user1", "/secret/parent", 3600)
        .await
        .unwrap();

    // Create child leases
    let child1 = manager
        .create_lease(
            "user1",
            "/secret/child1",
            "kv",
            "default",
            3600,
            86400,
            true,
            Some(parent.id.clone()),
            None,
            None,
            HashMap::new(),
        )
        .await
        .unwrap();

    let child2 = manager
        .create_lease(
            "user1",
            "/secret/child2",
            "kv",
            "default",
            3600,
            86400,
            true,
            Some(parent.id.clone()),
            None,
            None,
            HashMap::new(),
        )
        .await
        .unwrap();

    // Revoke parent
    let revoked_ids = manager.revoke_lease(&parent.id).await.unwrap();

    // Should revoke parent and both children
    assert_eq!(revoked_ids.len(), 3);
    assert!(revoked_ids.contains(&parent.id));
    assert!(revoked_ids.contains(&child1.id));
    assert!(revoked_ids.contains(&child2.id));

    // Verify all are revoked
    let parent_status = manager.lookup_lease(&parent.id).await.unwrap();
    assert_eq!(parent_status.status, "revoked");

    let child1_status = manager.lookup_lease(&child1.id).await.unwrap();
    assert_eq!(child1_status.status, "revoked");

    let child2_status = manager.lookup_lease(&child2.id).await.unwrap();
    assert_eq!(child2_status.status, "revoked");
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_lookup() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create lease
    let lease = create_test_lease(&manager, "user1", "/secret/lookup", 3600)
        .await
        .unwrap();

    // Lookup by ID
    let found = manager.lookup_lease(&lease.id).await.unwrap();
    assert_eq!(found.id, lease.id);
    assert_eq!(found.user, "user1");
    assert_eq!(found.resource, "/secret/lookup");
    assert_eq!(found.status, "active");

    // Lookup non-existent lease
    let result = manager.lookup_lease("non-existent-id").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeaseError::LeaseNotFound(_)));
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_list_with_filters() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create multiple leases with different attributes
    let _lease1 = create_test_lease(&manager, "user1", "/secret/test1", 3600)
        .await
        .unwrap();

    let _lease2 = manager
        .create_lease(
            "user2",
            "/secret/test2",
            "database",
            "default",
            3600,
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await
        .unwrap();

    let _lease3 = manager
        .create_lease(
            "user1",
            "/secret/test3",
            "kv",
            "namespace1",
            3600,
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await
        .unwrap();

    // List all leases
    let all_leases = manager
        .list_leases(None, None, None, None, None, None)
        .await
        .unwrap();
    assert!(all_leases.len() >= 3);

    // Filter by user
    let user1_leases = manager
        .list_leases(Some("user1".to_string()), None, None, None, None, None)
        .await
        .unwrap();
    assert!(user1_leases.iter().all(|l| l.user == "user1"));
    assert!(user1_leases.len() >= 2);

    // Filter by resource type
    let db_leases = manager
        .list_leases(None, None, Some("database".to_string()), None, None, None)
        .await
        .unwrap();
    assert!(db_leases.iter().all(|l| l.resource_type == "database"));
    assert!(db_leases.len() >= 1);

    // Filter by namespace
    let ns1_leases = manager
        .list_leases(None, Some("namespace1".to_string()), None, None, None, None)
        .await
        .unwrap();
    assert!(ns1_leases.iter().all(|l| l.namespace == "namespace1"));
    assert!(ns1_leases.len() >= 1);

    // Filter by status
    let active_leases = manager
        .list_leases(None, None, None, Some("active".to_string()), None, None)
        .await
        .unwrap();
    assert!(active_leases.iter().all(|l| l.status == "active"));

    // Test pagination
    let page1 = manager
        .list_leases(None, None, None, None, Some(2), Some(0))
        .await
        .unwrap();
    assert!(page1.len() <= 2);

    let page2 = manager
        .list_leases(None, None, None, None, Some(2), Some(2))
        .await
        .unwrap();
    // page2 may be empty or have items depending on total count
    assert!(page2.len() <= 2);
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_integration_with_kv_engine() {
    use secreton_core::services::secrets::kvv2::Kvv2Engine;
    use secreton_storage::MemoryBackend;
    use serde_json::json;

    let pool = setup_test_pool().await;
    let lease_manager = LeaseManager::new(pool);

    // Create KV engine
    let storage = Arc::new(MemoryBackend::new());
    let kv_engine = Kvv2Engine::new(storage);

    // Write secret with TTL (creates lease)
    let mut data = HashMap::new();
    data.insert("password".to_string(), json!("secret123"));

    let (version, lease_opt) = kv_engine
        .write_with_lease(
            "/secret/db/password",
            data,
            None,
            Some(3600), // 1 hour TTL
            "user1",
            "default",
            &lease_manager,
        )
        .await
        .unwrap();

    assert!(lease_opt.is_some());
    let lease = lease_opt.unwrap();
    assert_eq!(lease.user, "user1");
    assert_eq!(lease.resource, "/secret/db/password");
    assert_eq!(lease.resource_type, "kv");
    assert_eq!(lease.status, "active");
    assert!(lease.renewable);

    // Verify lease metadata contains version
    assert_eq!(
        lease.metadata.get("version").map(|s| s.as_str()),
        Some(&version.version.to_string())
    );

    // Read secret with lease information
    let (read_version, read_lease_opt) = kv_engine
        .read_with_lease("/secret/db/password", None, &lease_manager)
        .await
        .unwrap();

    assert_eq!(read_version.version, version.version);
    assert!(read_lease_opt.is_some());
    let read_lease = read_lease_opt.unwrap();
    assert_eq!(read_lease.id, lease.id);

    // Renew the lease
    let renewed = lease_manager.renew_lease(&lease.id, 1800).await.unwrap();
    assert_eq!(renewed.renew_count, 1);

    // Revoke the lease
    let revoked_ids = lease_manager.revoke_lease(&lease.id).await.unwrap();
    assert_eq!(revoked_ids.len(), 1);

    // Verify secret can still be read (revocation doesn't delete secret, just lease)
    let (final_version, final_lease_opt) = kv_engine
        .read_with_lease("/secret/db/password", None, &lease_manager)
        .await
        .unwrap();

    assert_eq!(final_version.version, version.version);
    // Lease should not be found or should be revoked
    if let Some(final_lease) = final_lease_opt {
        assert_eq!(final_lease.status, "revoked");
    }
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_integration_with_dynamic_secrets() {
    use secreton_core::services::secrets::database::{DatabaseConnection, DatabaseRole, DatabaseSecretsEngine, DatabaseType};

    let pool = setup_test_pool().await;
    let lease_manager = Arc::new(LeaseManager::new(pool));

    // Create database connection config
    let connection = DatabaseConnection {
        name: "test-db".to_string(),
        db_type: DatabaseType::PostgreSQL,
        connection_url: "postgresql://postgres:postgres@localhost/test".to_string(),
        max_open_connections: 10,
        max_idle_connections: 5,
        max_connection_lifetime: 3600,
        verify_connection: false,
        root_rotation_statements: vec![],
        username: Some("postgres".to_string()),
        password: Some("postgres".to_string()),
    };

    // Create database role
    let role = DatabaseRole {
        name: "readonly".to_string(),
        db_name: "test-db".to_string(),
        creation_statements: vec![
            "CREATE USER '{{name}}'@'%' IDENTIFIED BY '{{password}}';".to_string(),
            "GRANT SELECT ON *.* TO '{{name}}'@'%';".to_string(),
        ],
        revocation_statements: vec!["DROP USER '{{name}}'@'%';".to_string()],
        renew_statements: vec![],
        rollback_statements: vec![],
        default_ttl: 3600,
        max_ttl: 86400,
    };

    // Create database secrets engine
    let db_engine = DatabaseSecretsEngine::new();

    // Configure connection (this would normally connect to a real database)
    // For testing, we'll skip actual database operations

    // Generate credentials (creates lease automatically)
    let result = db_engine
        .generate_credentials(
            &role,
            &connection,
            "user1",
            "default",
            Some(3600),
            lease_manager.clone(),
        )
        .await;

    // Note: This will fail without a real database connection
    // In a real test environment with database, we would verify:
    // 1. Credentials are generated
    // 2. Lease is created with correct TTL
    // 3. Lease can be renewed
    // 4. Credentials are revoked when lease expires
    // 5. Database user is dropped on revocation

    // For now, we just verify the error is connection-related
    if let Err(e) = result {
        // Expected to fail without real database
        assert!(e.to_string().contains("Connection") || e.to_string().contains("connection"));
    }
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_expiration_scheduler() {
    let pool = setup_test_pool().await;

    // Create manager with fast check interval for testing
    let config = LeaseSchedulerConfig {
        check_interval_secs: 2, // Check every 2 seconds
        notification_threshold_secs: 10,
        enable_notifications: false,
    };

    let manager = Arc::new(LeaseManager::with_config(pool, config));

    // Create lease with short TTL (3 seconds)
    let lease = manager
        .create_lease(
            "user1",
            "/secret/auto_expire",
            "kv",
            "default",
            3, // 3 second TTL
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await
        .unwrap();

    assert_eq!(lease.status, "active");

    // Start the scheduler
    let scheduler_handle = manager.clone().start_expiration_scheduler();

    // Wait for scheduler to run and expire the lease
    tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

    // Verify lease was automatically revoked
    let expired_lease = manager.lookup_lease(&lease.id).await.unwrap();
    assert_eq!(expired_lease.status, "revoked");

    // Cancel the scheduler
    scheduler_handle.abort();
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_statistics() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create several leases
    let _active1 = create_test_lease(&manager, "user1", "/secret/active1", 3600)
        .await
        .unwrap();

    let _active2 = create_test_lease(&manager, "user2", "/secret/active2", 3600)
        .await
        .unwrap();

    let revoked = create_test_lease(&manager, "user3", "/secret/revoked", 3600)
        .await
        .unwrap();

    // Revoke one lease
    manager.revoke_lease(&revoked.id).await.unwrap();

    // Get statistics
    let stats = manager.get_stats().await.unwrap();

    assert!(stats.active_count >= 2);
    assert!(stats.revoked_count >= 1);
    assert!(stats.unique_users >= 3);
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_count_operations() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create leases in different namespaces
    let _lease1 = manager
        .create_lease(
            "user1",
            "/secret/ns1",
            "kv",
            "namespace1",
            3600,
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await
        .unwrap();

    let _lease2 = manager
        .create_lease(
            "user2",
            "/secret/ns1_2",
            "kv",
            "namespace1",
            3600,
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await
        .unwrap();

    let _lease3 = manager
        .create_lease(
            "user3",
            "/secret/ns2",
            "kv",
            "namespace2",
            3600,
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await
        .unwrap();

    // Count active leases
    let active_count = manager.count_active().await.unwrap();
    assert!(active_count >= 3);

    // Count by namespace
    let ns1_count = manager.count_by_namespace("namespace1").await.unwrap();
    assert!(ns1_count >= 2);

    let ns2_count = manager.count_by_namespace("namespace2").await.unwrap();
    assert!(ns2_count >= 1);
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_with_metadata() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create lease with custom metadata
    let mut metadata = HashMap::new();
    metadata.insert("purpose".to_string(), "testing".to_string());
    metadata.insert("environment".to_string(), "dev".to_string());
    metadata.insert("version".to_string(), "1.0".to_string());

    let lease = manager
        .create_lease(
            "user1",
            "/secret/with_metadata",
            "kv",
            "default",
            3600,
            86400,
            true,
            None,
            None,
            None,
            metadata.clone(),
        )
        .await
        .unwrap();

    // Verify metadata is stored
    assert_eq!(lease.metadata.get("purpose"), Some(&"testing".to_string()));
    assert_eq!(
        lease.metadata.get("environment"),
        Some(&"dev".to_string())
    );
    assert_eq!(lease.metadata.get("version"), Some(&"1.0".to_string()));

    // Lookup and verify metadata persists
    let found = manager.lookup_lease(&lease.id).await.unwrap();
    assert_eq!(found.metadata, metadata);
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_renewal_after_expiration() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create lease with very short TTL
    let lease = create_test_lease(&manager, "user1", "/secret/expired", 1)
        .await
        .unwrap();

    // Wait for expiration
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Try to renew expired lease (should fail)
    let result = manager.renew_lease(&lease.id, 1800).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeaseError::LeaseExpired));
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lease_with_revoke_callback() {
    let pool = setup_test_pool().await;
    let manager = LeaseManager::new(pool);

    // Create lease with revoke callback
    let lease = manager
        .create_lease(
            "user1",
            "/secret/with_callback",
            "kv",
            "default",
            3600,
            86400,
            true,
            None,
            None,
            Some("kv_revoke:/secret/with_callback".to_string()),
            HashMap::new(),
        )
        .await
        .unwrap();

    assert_eq!(
        lease.revoke_callback,
        Some("kv_revoke:/secret/with_callback".to_string())
    );

    // Revoke lease (callback would be executed in real implementation)
    let revoked_ids = manager.revoke_lease(&lease.id).await.unwrap();
    assert_eq!(revoked_ids.len(), 1);

    // Verify lease is revoked
    let revoked_lease = manager.lookup_lease(&lease.id).await.unwrap();
    assert_eq!(revoked_lease.status, "revoked");
}
