//! Integration tests for LeaseManager
//!
//! These tests verify the complete lease lifecycle with PostgreSQL persistence.

use chrono::Utc;
use deadpool_postgres::{Config, Pool, Runtime};
use secreton_core::services::lease::{EnhancedLease, LeaseError, LeaseManager};
use std::collections::HashMap;
use tokio_postgres::NoTls;

/// Setup test database connection pool
async fn setup_test_pool() -> Pool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/secreton_test".to_string());

    let mut cfg = Config::new();
    cfg.url = Some(database_url);
    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create test pool")
}

/// Setup test database schema
async fn setup_test_schema(pool: &Pool) {
    let client = pool.get().await.expect("Failed to get client");

    // Create leases table if not exists
    client
        .execute(
            r#"
            CREATE TABLE IF NOT EXISTS leases (
                id VARCHAR(255) PRIMARY KEY,
                user_id VARCHAR(255) NOT NULL,
                resource VARCHAR(1024) NOT NULL,
                resource_type VARCHAR(50) NOT NULL,
                namespace VARCHAR(255) NOT NULL DEFAULT 'default',
                status VARCHAR(50) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'revoked', 'expired')),
                issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                expired_at TIMESTAMPTZ NOT NULL,
                last_renewed_at TIMESTAMPTZ,
                renewable BOOLEAN NOT NULL DEFAULT TRUE,
                max_ttl BIGINT NOT NULL DEFAULT 86400,
                renew_count INTEGER NOT NULL DEFAULT 0,
                max_renewals INTEGER,
                parent_id VARCHAR(255),
                revoke_callback VARCHAR(512),
                metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                CONSTRAINT fk_parent_lease
                    FOREIGN KEY (parent_id)
                    REFERENCES leases(id)
                    ON DELETE CASCADE
            )
            "#,
            &[],
        )
        .await
        .expect("Failed to create leases table");

    // Create indexes
    client
        .batch_execute(
            r#"
            CREATE INDEX IF NOT EXISTS idx_leases_user_id ON leases(user_id);
            CREATE INDEX IF NOT EXISTS idx_leases_resource ON leases(resource);
            CREATE INDEX IF NOT EXISTS idx_leases_namespace ON leases(namespace);
            CREATE INDEX IF NOT EXISTS idx_leases_status ON leases(status);
            CREATE INDEX IF NOT EXISTS idx_leases_expired_at ON leases(expired_at);
            "#,
        )
        .await
        .expect("Failed to create indexes");
}

/// Cleanup test data
async fn cleanup_test_data(pool: &Pool) {
    let client = pool.get().await.expect("Failed to get client");
    client
        .execute("DELETE FROM leases", &[])
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_create_lease_success() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;
    cleanup_test_data(&pool).await;

    let manager = LeaseManager::new(pool);

    let lease = manager
        .create_lease(
            "user1",
            "/secret/data/test",
            "kv",
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
        .expect("Failed to create lease");

    assert_eq!(lease.user, "user1");
    assert_eq!(lease.resource, "/secret/data/test");
    assert_eq!(lease.resource_type, "kv");
    assert_eq!(lease.namespace, "default");
    assert_eq!(lease.status, "active");
    assert!(lease.renewable);
    assert_eq!(lease.max_ttl, 86400);
    assert_eq!(lease.renew_count, 0);
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_create_lease_validation() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;
    cleanup_test_data(&pool).await;

    let manager = LeaseManager::new(pool);

    // Test invalid TTL (negative)
    let result = manager
        .create_lease(
            "user1",
            "/secret/data/test",
            "kv",
            "default",
            -100,
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await;
    assert!(matches!(result, Err(LeaseError::InvalidTtl(_))));

    // Test invalid TTL (exceeds max_ttl)
    let result = manager
        .create_lease(
            "user1",
            "/secret/data/test",
            "kv",
            "default",
            100000,
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await;
    assert!(matches!(result, Err(LeaseError::InvalidTtl(_))));

    // Test empty user
    let result = manager
        .create_lease(
            "",
            "/secret/data/test",
            "kv",
            "default",
            3600,
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await;
    assert!(matches!(result, Err(LeaseError::InvalidTtl(_))));
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_lookup_lease() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;
    cleanup_test_data(&pool).await;

    let manager = LeaseManager::new(pool);

    let created_lease = manager
        .create_lease(
            "user1",
            "/secret/data/test",
            "kv",
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
        .expect("Failed to create lease");

    // Lookup the lease
    let looked_up_lease = manager
        .lookup_lease(&created_lease.id)
        .await
        .expect("Failed to lookup lease");

    assert_eq!(looked_up_lease.id, created_lease.id);
    assert_eq!(looked_up_lease.user, created_lease.user);
    assert_eq!(looked_up_lease.resource, created_lease.resource);

    // Test lookup non-existent lease
    let result = manager.lookup_lease("non-existent-id").await;
    assert!(matches!(result, Err(LeaseError::LeaseNotFound(_))));
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_renew_lease_success() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;
    cleanup_test_data(&pool).await;

    let manager = LeaseManager::new(pool);

    let lease = manager
        .create_lease(
            "user1",
            "/secret/data/test",
            "kv",
            "default",
            1800,
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await
        .expect("Failed to create lease");

    let original_expired_at = lease.expired_at;

    // Renew the lease
    let renewed = manager
        .renew_lease(&lease.id, 3600)
        .await
        .expect("Failed to renew lease");

    assert_eq!(renewed.renew_count, 1);
    assert!(renewed.last_renewed_at.is_some());
    assert!(renewed.expired_at > original_expired_at);
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_renew_lease_max_renewals() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;
    cleanup_test_data(&pool).await;

    let manager = LeaseManager::new(pool);

    let lease = manager
        .create_lease(
            "user1",
            "/secret/data/test",
            "kv",
            "default",
            1800,
            86400,
            true,
            None,
            Some(2), // Max 2 renewals
            None,
            HashMap::new(),
        )
        .await
        .expect("Failed to create lease");

    // First renewal should succeed
    manager
        .renew_lease(&lease.id, 1800)
        .await
        .expect("First renewal failed");

    // Second renewal should succeed
    manager
        .renew_lease(&lease.id, 1800)
        .await
        .expect("Second renewal failed");

    // Third renewal should fail
    let result = manager.renew_lease(&lease.id, 1800).await;
    assert!(matches!(result, Err(LeaseError::RenewalNotAllowed)));
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_renew_non_renewable_lease() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;
    cleanup_test_data(&pool).await;

    let manager = LeaseManager::new(pool);

    let lease = manager
        .create_lease(
            "user1",
            "/secret/data/test",
            "kv",
            "default",
            1800,
            86400,
            false, // Not renewable
            None,
            None,
            None,
            HashMap::new(),
        )
        .await
        .expect("Failed to create lease");

    // Renewal should fail
    let result = manager.renew_lease(&lease.id, 1800).await;
    assert!(matches!(result, Err(LeaseError::RenewalNotAllowed)));
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_revoke_lease() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;
    cleanup_test_data(&pool).await;

    let manager = LeaseManager::new(pool);

    let lease = manager
        .create_lease(
            "user1",
            "/secret/data/test",
            "kv",
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
        .expect("Failed to create lease");

    // Revoke the lease
    let revoked_ids = manager
        .revoke_lease(&lease.id)
        .await
        .expect("Failed to revoke lease");

    assert_eq!(revoked_ids.len(), 1);
    assert_eq!(revoked_ids[0], lease.id);

    // Verify lease is revoked
    let revoked_lease = manager
        .lookup_lease(&lease.id)
        .await
        .expect("Failed to lookup revoked lease");
    assert_eq!(revoked_lease.status, "revoked");

    // Revoking again should return empty list
    let revoked_again = manager
        .revoke_lease(&lease.id)
        .await
        .expect("Failed to revoke again");
    assert_eq!(revoked_again.len(), 0);
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_parent_child_revocation() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;
    cleanup_test_data(&pool).await;

    let manager = LeaseManager::new(pool);

    // Create parent lease
    let parent = manager
        .create_lease(
            "user1",
            "/parent",
            "kv",
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
        .expect("Failed to create parent lease");

    // Create child lease
    let child = manager
        .create_lease(
            "user1",
            "/child",
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
        .expect("Failed to create child lease");

    // Revoke parent should revoke child too
    let revoked_ids = manager
        .revoke_lease(&parent.id)
        .await
        .expect("Failed to revoke parent");

    assert_eq!(revoked_ids.len(), 2);
    assert!(revoked_ids.contains(&parent.id));
    assert!(revoked_ids.contains(&child.id));

    // Verify both are revoked
    let parent_lease = manager.lookup_lease(&parent.id).await.unwrap();
    let child_lease = manager.lookup_lease(&child.id).await.unwrap();
    assert_eq!(parent_lease.status, "revoked");
    assert_eq!(child_lease.status, "revoked");
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_list_leases_with_filters() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;
    cleanup_test_data(&pool).await;

    let manager = LeaseManager::new(pool);

    // Create multiple leases
    manager
        .create_lease(
            "user1",
            "/secret/data/test1",
            "kv",
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

    manager
        .create_lease(
            "user2",
            "/secret/data/test2",
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
     .unwrap();

    manager
   .crea

            "user1",
            "/secret/data/test3",
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
    assert_eq!(all_leases.len(), 3);

    // List leases for user1
    let user1_leases = manager
        .list_leases(Some("user1".to_string()), None, None, None, None, None)
        .await
        .unwrap();
    assert_eq!(user1_leases.len(), 2);
    assert!(user1_leases.iter().all(|l| l.user == "user1"));

    // List database leases
    let db_leases = manager
        .list_leases(None, None, Some("database".to_string()), None, None, None)
        .await
        .unwrap();
    assert_eq!(db_leases.len(), 1);
    assert!(db_leases.iter().all(|l| l.resource_type == "database"));

    // List leases in namespace1
    let ns1_leases = manager
        .list_leases(None, Some("namespace1".to_string()), None, None, None, None)
        .await
        .unwrap();
    assert_eq!(ns1_leases.len(), 1);
    assert!(ns1_leases.iter().all(|l| l.namespace == "namespace1"));

    // Test pagination
    let page1 = manager
        .list_leases(None, None, None, None, Some(2), Some(0))
        .await
        .unwrap();
    assert_eq!(page1.len(), 2);

    let page2 = manager
        .list_leases(None, None, None, None, Some(2), Some(2))
        .await
        .unwrap();
    assert_eq!(page2.len(), 1);
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_cleanup_expired() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;
    cleanup_test_data(&pool).await;

    let manager = LeaseManager::new(pool);

    // Create a lease that expires immediately
    let lease = manager
        .create_lease(
            "user1",
            "/secret/data/test",
            "kv",
            "default",
            1, // 1 second TTL
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await
        .expect("Failed to create lease");

    // Wait for expiration
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Cleanup expired leases
    let count = manager
        .cleanup_expired()
        .await
        .expect("Failed to cleanup expired");
    assert!(count >= 1);

    // Verify lease is expired
    let expired_lease = manager.lookup_lease(&lease.id).await.unwrap();
    assert_eq!(expired_lease.status, "expired");
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_count_active_leases() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;
    cleanup_test_data(&pool).await;

    let manager = LeaseManager::new(pool);

    // Create active leases
    manager
        .create_lease(
            "user1",
            "/secret/data/test1",
            "kv",
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

    manager
        .create_lease(
            "user2",
            "/secret/data/test2",
            "kv",
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

    let count = manager.count_active().await.unwrap();
    assert_eq!(count, 2);
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_get_stats() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;
    cleanup_test_data(&pool).await;

    let manager = LeaseManager::new(pool);

    // Create various leases
    let lease1 = manager
        .create_lease(
            "user1",
            "/secret/data/test1",
            "kv",
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

    manager
        .create_lease(
            "user2",
            "/secret/data/test2",
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

    // Revoke one lease
    manager.revoke_lease(&lease1.id).await.unwrap();

    // Get stats
    let stats = manager.get_stats().await.unwrap();
    assert_eq!(stats.active_count, 1);
    assert_eq!(stats.revoked_count, 1);
    assert_eq!(stats.unique_users, 2);
    assert_eq!(stats.unique_namespaces, 2);
}

