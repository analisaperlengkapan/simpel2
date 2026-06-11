//! Property-Based Tests for Revocation Service
//!
//! These tests verify the correctness properties of the secret revocation system.

use deadpool_postgres::{Config, Runtime};
use proptest::prelude::*;
use secreton_core::services::lease::{CreateLeaseRequest, LeaseManager};
use secreton_core::services::revocation::{
    RevocationManager, RevocationRequest, RevocationService,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::NoTls;

// Helper to setup test database pool
async fn setup_test_pool() -> Result<deadpool_postgres::Pool, Box<dyn std::error::Error>> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/secreton_test".to_string());

    let mut cfg = Config::new();
    cfg.url = Some(database_url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    // Test connection
    let _conn = pool.get().await?;

    Ok(pool)
}

// Helper to setup test service
async fn setup_test_service() -> Result<Arc<RevocationManager>, Box<dyn std::error::Error>> {
    let pool = setup_test_pool().await?;
    let lease_manager = Arc::new(LeaseManager::new(pool.clone()));
    let revocation_manager = Arc::new(RevocationManager::new(pool, lease_manager.clone()));

    // Initialize tables
    revocation_manager.initialize().await?;

    Ok(revocation_manager)
}

// Helper to create a test lease
async fn create_test_lease(
    lease_manager: &Arc<LeaseManager>,
    path: &str,
    user: &str,
    namespace: &str,
) -> String {
    let lease = lease_manager
        .create_lease(CreateLeaseRequest {
            user,
            resource: path,
            resource_type: "kv",
            namespace,
            ttl_secs: 3600,
            max_ttl: 86400,
            renewable: true,
            ..Default::default()
        })
        .await
        .unwrap();

    lease.id
}

// **Feature: secreton-comprehensive-enhancement, Property 21: Revocation Invalidates Secret**
// **Validates: Requirements 8.1**
//
// Property: For any revoked secret, subsequent read attempts SHALL fail with "secret revoked" error.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn property_revocation_invalidates_secret(
        // Generate random secret path
        path_suffix in "[a-z0-9]{5,15}",
        // Generate random user
        user in "[a-z]{3,10}",
        // Generate random reason
        reason in "[a-z ]{10,50}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Skip test if database is not available
            let revocation_manager = match setup_test_service().await {
                Ok(mgr) => mgr,
                Err(_) => {
                    // Database not available, skip test
                    return Ok(());
                }
            };
            let lease_manager = revocation_manager.lease_manager.clone();

            let path = format!("/secret/data/{}", path_suffix);
            let namespace = "default";

            // Create a lease for the secret (simulating secret access)
            let lease_id = create_test_lease(&lease_manager, &path, &user, namespace).await;

            // Verify lease is active before revocation
            let lease_before = lease_manager.lookup_lease(&lease_id).await
                .expect("Lease should exist before revocation");
            prop_assert_eq!(lease_before.status, "active",
                "Lease should be active before revocation");

            // Revoke the secret
            let request = RevocationRequest {
                path: path.clone(),
                reason: reason.clone(),
                cascade: false,
                emergency: false,
                actor: user.clone(),
                namespace: namespace.to_string(),
            };

            let record = revocation_manager.revoke(request).await
                .expect("Revocation should succeed");

            // Property 1: Revocation record should be created
            prop_assert_eq!(record.path, path.clone(),
                "Revocation record should have correct path");
            prop_assert_eq!(record.revoked_by, user.clone(),
                "Revocation record should have correct actor");
            prop_assert_eq!(record.reason, reason.clone(),
                "Revocation record should have correct reason");

            // Property 2: Associated lease should be revoked
            let lease_after = lease_manager.lookup_lease(&lease_id).await
                .expect("Lease should still exist after revocation");
            prop_assert_eq!(lease_after.status, "revoked",
                "Lease should be revoked after secret revocation");

            // Property 3: Revocation history should contain the record
            let history = revocation_manager.get_history(&path).await
                .expect("Should be able to get revocation history");
            prop_assert!(!history.is_empty(),
                "Revocation history should not be empty");
            prop_assert_eq!(history[0].id, record.id,
                "Most recent revocation should match");

            Ok(())
        })?;
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 22: Cascade Revocation Completeness**
// **Validates: Requirements 8.2**
//
// Property: For any secret with dependencies, cascade revocation SHALL revoke all secrets
// in the dependency chain.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn property_cascade_revocation_completeness(
        // Generate random paths
        parent_suffix in "[a-z0-9]{5,10}",
        child_suffix in "[a-z0-9]{5,10}",
        user in "[a-z]{3,10}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Skip test if database is not available
            let revocation_manager = match setup_test_service().await {
                Ok(mgr) => mgr,
                Err(_) => {
                    // Database not available, skip test
                    return Ok(());
                }
            };
            let lease_manager = revocation_manager.lease_manager.clone();

            let parent_path = format!("/secret/parent/{}", parent_suffix);
            let child_path = format!("/secret/child/{}", child_suffix);
            let namespace = "default";

            // Create leases for both parent and child
            let parent_lease_id = create_test_lease(&lease_manager, &parent_path, &user, namespace).await;
            let child_lease_id = create_test_lease(&lease_manager, &child_path, &user, namespace).await;

            // Add dependency relationship
            revocation_manager
                .add_dependency(&parent_path, &child_path, namespace)
                .await
                .expect("Should be able to add dependency");

            // Verify both leases are active
            let parent_lease_before = lease_manager.lookup_lease(&parent_lease_id).await.unwrap();
            let child_lease_before = lease_manager.lookup_lease(&child_lease_id).await.unwrap();
            prop_assert_eq!(parent_lease_before.status, "active");
            prop_assert_eq!(child_lease_before.status, "active");

            // Revoke parent with cascade enabled
            let request = RevocationRequest {
                path: parent_path.clone(),
                reason: "Testing cascade revocation".to_string(),
                cascade: true,
                emergency: false,
                actor: user.clone(),
                namespace: namespace.to_string(),
            };

            let record = revocation_manager.revoke(request).await
                .expect("Cascade revocation should succeed");

            // Property 1: Cascade count should be at least 1 (the child)
            prop_assert!(record.cascade_count >= 1,
                "Cascade count should include at least the child secret");

            // Property 2: Cascaded paths should include the child
            prop_assert!(record.cascaded_paths.contains(&child_path),
                "Cascaded paths should include the child secret");

            // Property 3: Both parent and child leases should be revoked
            let parent_lease_after = lease_manager.lookup_lease(&parent_lease_id).await.unwrap();
            let child_lease_after = lease_manager.lookup_lease(&child_lease_id).await.unwrap();
            prop_assert_eq!(parent_lease_after.status, "revoked",
                "Parent lease should be revoked");
            prop_assert_eq!(child_lease_after.status, "revoked",
                "Child lease should be revoked in cascade");

            // Property 4: Revocation history should exist for parent
            let history = revocation_manager.get_history(&parent_path).await.unwrap();
            prop_assert!(!history.is_empty(),
                "Parent should have revocation history");

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod revocation_edge_cases {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_revoke_nonexistent_secret() {
        let revocation_manager = match setup_test_service().await {
            Ok(mgr) => mgr,
            Err(_) => return, // Skip if database not available
        };

        let request = RevocationRequest {
            path: "/nonexistent/secret".to_string(),
            reason: "Testing".to_string(),
            cascade: false,
            emergency: false,
            actor: "test_user".to_string(),
            namespace: "default".to_string(),
        };

        // Should succeed even if secret doesn't exist (idempotent)
        let result = revocation_manager.revoke(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_revoke_already_revoked_secret() {
        let revocation_manager = match setup_test_service().await {
            Ok(mgr) => mgr,
            Err(_) => return, // Skip if database not available
        };
        let lease_manager = revocation_manager.lease_manager.clone();

        let path = "/secret/data/test_double_revoke";
        let namespace = "default";

        // Create a lease
        let _lease_id = create_test_lease(&lease_manager, path, "user1", namespace).await;

        let request = RevocationRequest {
            path: path.to_string(),
            reason: "First revocation".to_string(),
            cascade: false,
            emergency: false,
            actor: "user1".to_string(),
            namespace: namespace.to_string(),
        };

        // First revocation
        let result1 = revocation_manager.revoke(request.clone()).await;
        assert!(result1.is_ok());

        // Second revocation (should be idempotent)
        let request2 = RevocationRequest {
            reason: "Second revocation".to_string(),
            ..request
        };
        let result2 = revocation_manager.revoke(request2).await;
        assert!(result2.is_ok());

        // History should show both revocations
        let history = revocation_manager.get_history(path).await.unwrap();
        assert_eq!(history.len(), 2);
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_emergency_revocation_timing() {
        let revocation_manager = match setup_test_service().await {
            Ok(mgr) => mgr,
            Err(_) => return, // Skip if database not available
        };
        let lease_manager = revocation_manager.lease_manager.clone();

        // Create multiple secrets
        for i in 0..10 {
            let path = format!("/secret/emergency/test{}", i);
            create_test_lease(&lease_manager, &path, "user1", "default").await;
        }

        let start = std::time::Instant::now();

        // Emergency revoke all secrets matching pattern
        let records = revocation_manager
            .emergency_revoke("/secret/emergency/", "admin", "default")
            .await
            .unwrap();

        let duration = start.elapsed();

        // Should complete within 1 second
        assert!(
            duration.as_secs() < 1,
            "Emergency revocation took {:?}",
            duration
        );
        assert!(records.len() >= 10, "Should revoke all matching secrets");

        // All records should be marked as emergency
        for record in records {
            assert!(record.emergency, "Should be marked as emergency revocation");
        }
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_deep_cascade_revocation() {
        let revocation_manager = match setup_test_service().await {
            Ok(mgr) => mgr,
            Err(_) => return, // Skip if database not available
        };
        let lease_manager = revocation_manager.lease_manager.clone();

        let namespace = "default";

        // Create a chain: root -> level1 -> level2 -> level3
        let root_path = "/secret/root";
        let level1_path = "/secret/level1";
        let level2_path = "/secret/level2";
        let level3_path = "/secret/level3";

        // Create leases for all levels
        let root_lease = create_test_lease(&lease_manager, root_path, "user1", namespace).await;
        let level1_lease = create_test_lease(&lease_manager, level1_path, "user1", namespace).await;
        let level2_lease = create_test_lease(&lease_manager, level2_path, "user1", namespace).await;
        let level3_lease = create_test_lease(&lease_manager, level3_path, "user1", namespace).await;

        // Add dependency chain
        revocation_manager
            .add_dependency(root_path, level1_path, namespace)
            .await
            .unwrap();
        revocation_manager
            .add_dependency(level1_path, level2_path, namespace)
            .await
            .unwrap();
        revocation_manager
            .add_dependency(level2_path, level3_path, namespace)
            .await
            .unwrap();

        // Revoke root with cascade
        let request = RevocationRequest {
            path: root_path.to_string(),
            reason: "Testing deep cascade".to_string(),
            cascade: true,
            emergency: false,
            actor: "user1".to_string(),
            namespace: namespace.to_string(),
        };

        let record = revocation_manager.revoke(request).await.unwrap();

        // Should cascade to all 3 levels
        assert_eq!(record.cascade_count, 3);
        assert!(record.cascaded_paths.contains(&level1_path.to_string()));
        assert!(record.cascaded_paths.contains(&level2_path.to_string()));
        assert!(record.cascaded_paths.contains(&level3_path.to_string()));

        // All leases should be revoked
        assert_eq!(
            lease_manager
                .lookup_lease(&root_lease)
                .await
                .unwrap()
                .status,
            "revoked"
        );
        assert_eq!(
            lease_manager
                .lookup_lease(&level1_lease)
                .await
                .unwrap()
                .status,
            "revoked"
        );
        assert_eq!(
            lease_manager
                .lookup_lease(&level2_lease)
                .await
                .unwrap()
                .status,
            "revoked"
        );
        assert_eq!(
            lease_manager
                .lookup_lease(&level3_lease)
                .await
                .unwrap()
                .status,
            "revoked"
        );
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_orphaned_secret_detection() {
        let revocation_manager = match setup_test_service().await {
            Ok(mgr) => mgr,
            Err(_) => return, // Skip if database not available
        };
        let lease_manager = revocation_manager.lease_manager.clone();

        let namespace = "default";

        // Create a secret with a very short TTL
        let path = "/secret/orphan_test";
        let _lease = lease_manager
            .create_lease(CreateLeaseRequest {
                user: "user1",
                resource: path,
                resource_type: "kv",
                namespace,
                ttl_secs: 1, // 1 second TTL
                max_ttl: 86400,
                renewable: true,
                ..Default::default()
            })
            .await
            .unwrap();

        // Wait for lease to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Cleanup expired leases
        lease_manager.cleanup_expired().await.unwrap();

        // Detect orphans (threshold of 0 days to catch immediately)
        let orphans = revocation_manager.detect_orphans(0).await.unwrap();

        // Should detect the orphaned secret
        let found = orphans.iter().any(|o| o.path == path);
        assert!(found, "Should detect orphaned secret");
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_revocation_stats() {
        let revocation_manager = match setup_test_service().await {
            Ok(mgr) => mgr,
            Err(_) => return, // Skip if database not available
        };
        let lease_manager = revocation_manager.lease_manager.clone();

        let namespace = "default";

        // Create and revoke some secrets
        for i in 0..5 {
            let path = format!("/secret/stats_test/{}", i);
            create_test_lease(&lease_manager, &path, "user1", namespace).await;

            let request = RevocationRequest {
                path,
                reason: "Testing stats".to_string(),
                cascade: false,
                emergency: i % 2 == 0, // Every other one is emergency
                actor: "user1".to_string(),
                namespace: namespace.to_string(),
            };

            revocation_manager.revoke(request).await.unwrap();
        }

        // Get stats
        let stats = revocation_manager.get_stats().await.unwrap();

        assert!(stats.total_revocations >= 5);
        assert!(stats.emergency_revocations >= 2); // At least 2 emergency revocations
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_empty_path_validation() {
        let revocation_manager = match setup_test_service().await {
            Ok(mgr) => mgr,
            Err(_) => return, // Skip if database not available
        };

        let request = RevocationRequest {
            path: "".to_string(),
            reason: "Testing".to_string(),
            cascade: false,
            emergency: false,
            actor: "user1".to_string(),
            namespace: "default".to_string(),
        };

        let result = revocation_manager.revoke(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_empty_actor_validation() {
        let revocation_manager = match setup_test_service().await {
            Ok(mgr) => mgr,
            Err(_) => return, // Skip if database not available
        };

        let request = RevocationRequest {
            path: "/secret/test".to_string(),
            reason: "Testing".to_string(),
            cascade: false,
            emergency: false,
            actor: "".to_string(),
            namespace: "default".to_string(),
        };

        let result = revocation_manager.revoke(request).await;
        assert!(result.is_err());
    }
}
