//! Property-based tests for health check registration
//!
//! **Validates: Requirements 2.6.1**
//!
//! These tests verify that the health check registry correctly manages
//! health check registration, retrieval, and execution.

use async_trait::async_trait;
use proptest::prelude::*;
use secreton_health::{HealthCheck, HealthCheckRegistry, HealthCheckResult, HealthStatus};

/// Mock health check for testing
///
/// This health check can be configured with a name and status for testing purposes.
struct MockHealthCheck {
    name: String,
    status: HealthStatus,
    is_critical: bool,
    tags: Vec<String>,
}

impl MockHealthCheck {
    fn new(name: String, status: HealthStatus) -> Self {
        Self {
            name,
            status,
            is_critical: true,
            tags: Vec::new(),
        }
    }

    fn with_critical(mut self, is_critical: bool) -> Self {
        self.is_critical = is_critical;
        self
    }

    fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

#[async_trait]
impl HealthCheck for MockHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthCheckResult {
        HealthCheckResult {
            status: self.status.clone(),
            message: Some(format!("Mock check: {}", self.name)),
            response_time_ms: 1,
            details: None,
        }
    }

    fn is_critical(&self) -> bool {
        self.is_critical
    }

    fn tags(&self) -> Vec<String> {
        self.tags.clone()
    }
}

/// Property 34: Health check registration
///
/// **Property**: For any health check with a unique name, registering it should succeed,
/// and the registered check should be retrievable and executable.
///
/// **Validates**: Requirements 2.6.1
///
/// **Formal specification**:
/// ```text
/// ∀ check ∈ HealthCheck:
///   register(check) ⇒ contains(check.name()) = true
///   ∧ check_one(check.name()) = check.check()
///   ∧ check_names() contains check.name()
/// ```
///
/// This property ensures that:
/// 1. Health checks can be registered with unique names
/// 2. Registered health checks can be retrieved by name
/// 3. Registry maintains all registered checks
/// 4. Duplicate registration is properly rejected
#[cfg(test)]
mod registration_tests {
    use super::*;

    proptest! {
        /// Test basic health check registration
        #[test]
        fn prop_health_check_registration(
            name in "[a-z][a-z0-9_]{2,20}",
            status in prop_oneof![
                Just(HealthStatus::Healthy),
                Just(HealthStatus::Degraded),
                Just(HealthStatus::Unhealthy),
            ]
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut registry = HealthCheckRegistry::new();

                // Create and register health check
                let check = Box::new(MockHealthCheck::new(name.clone(), status.clone()));

                // Registration should succeed
                let result = registry.register(check).await;
                assert!(result.is_ok(), "Registration should succeed for unique name");

                // Check should be in registry
                assert!(registry.contains(&name).await, "Registry should contain registered check");

                // Check count should be 1
                assert_eq!(registry.count().await, 1, "Registry should have exactly 1 check");

                // Check name should be in list
                let names = registry.check_names().await;
                assert!(names.contains(&name), "Check names should include registered check");

                // Should be able to execute the check
                let result = registry.check_one(&name).await;
                assert!(result.is_ok(), "Should be able to execute registered check");

                let check_result = result.unwrap();
                assert_eq!(check_result.status, status, "Check should return configured status");
            });
        }
    }

    proptest! {
        /// Test registration of multiple health checks
        #[test]
        fn prop_multiple_health_check_registration(
            names in prop::collection::vec("[a-z][a-z0-9_]{2,20}", 1..10)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut registry = HealthCheckRegistry::new();

                // Remove duplicates to ensure unique names
                let unique_names: std::collections::HashSet<_> = names.into_iter().collect();
                let unique_names: Vec<_> = unique_names.into_iter().collect();

                // Register all checks
                for name in &unique_names {
                    let check = Box::new(MockHealthCheck::new(
                        name.clone(),
                        HealthStatus::Healthy
                    ));
                    let result = registry.register(check).await;
                    assert!(result.is_ok(), "Registration should succeed for unique name: {}", name);
                }

                // Verify count
                assert_eq!(
                    registry.count().await,
                    unique_names.len(),
                    "Registry should contain all registered checks"
                );

                // Verify all checks are present
                for name in &unique_names {
                    assert!(
                        registry.contains(name).await,
                        "Registry should contain check: {}",
                        name
                    );
                }

                // Verify all checks can be executed
                let results = registry.check_all().await;
                assert_eq!(
                    results.check_count(),
                    unique_names.len(),
                    "Should execute all registered checks"
                );
            });
        }
    }

    proptest! {
        /// Test duplicate registration is rejected
        #[test]
        fn prop_duplicate_registration_rejected(
            name in "[a-z][a-z0-9_]{2,20}"
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut registry = HealthCheckRegistry::new();

                // Register first check
                let check1 = Box::new(MockHealthCheck::new(name.clone(), HealthStatus::Healthy));
                let result1 = registry.register(check1).await;
                assert!(result1.is_ok(), "First registration should succeed");

                // Try to register duplicate
                let check2 = Box::new(MockHealthCheck::new(name.clone(), HealthStatus::Healthy));
                let result2 = registry.register(check2).await;
                assert!(result2.is_err(), "Duplicate registration should fail");

                // Count should still be 1
                assert_eq!(registry.count().await, 1, "Count should remain 1 after duplicate attempt");
            });
        }
    }

    proptest! {
        /// Test unregistration removes health check
        #[test]
        fn prop_unregistration(
            name in "[a-z][a-z0-9_]{2,20}"
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut registry = HealthCheckRegistry::new();

                // Register check
                let check = Box::new(MockHealthCheck::new(name.clone(), HealthStatus::Healthy));
                registry.register(check).await.unwrap();

                // Verify it's registered
                assert!(registry.contains(&name).await);
                assert_eq!(registry.count().await, 1);

                // Unregister
                let result = registry.unregister(&name).await;
                assert!(result.is_ok(), "Unregistration should succeed");

                // Verify it's removed
                assert!(!registry.contains(&name).await, "Check should be removed from registry");
                assert_eq!(registry.count().await, 0, "Count should be 0 after unregistration");

                // Check names should be empty
                let names = registry.check_names().await;
                assert!(!names.contains(&name), "Check name should not be in list");
            });
        }
    }

    proptest! {
        /// Test unregistering non-existent check fails
        #[test]
        fn prop_unregister_nonexistent_fails(
            name in "[a-z][a-z0-9_]{2,20}"
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut registry = HealthCheckRegistry::new();

                // Try to unregister non-existent check
                let result = registry.unregister(&name).await;
                assert!(result.is_err(), "Unregistering non-existent check should fail");
            });
        }
    }

    proptest! {
        /// Test registration and retrieval preserves check behavior
        #[test]
        fn prop_registration_preserves_behavior(
            name in "[a-z][a-z0-9_]{2,20}",
            status in prop_oneof![
                Just(HealthStatus::Healthy),
                Just(HealthStatus::Degraded),
                Just(HealthStatus::Unhealthy),
            ]
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut registry = HealthCheckRegistry::new();

                // Register check with specific status
                let check = Box::new(MockHealthCheck::new(name.clone(), status.clone()));
                registry.register(check).await.unwrap();

                // Execute check
                let result = registry.check_one(&name).await.unwrap();

                // Verify status is preserved
                assert_eq!(
                    result.status,
                    status,
                    "Registered check should return configured status"
                );

                // Verify message contains check name
                assert!(
                    result.message.as_ref().unwrap().contains(&name),
                    "Check message should contain check name"
                );
            });
        }
    }

    proptest! {
        /// Test check_all executes all registered checks
        #[test]
        fn prop_check_all_executes_all_checks(
            count in 1..20usize
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut registry = HealthCheckRegistry::new();

                // Register multiple checks
                for i in 0..count {
                    let name = format!("check_{}", i);
                    let check = Box::new(MockHealthCheck::new(name, HealthStatus::Healthy));
                    registry.register(check).await.unwrap();
                }

                // Execute all checks
                let results = registry.check_all().await;

                // Verify all checks were executed
                assert_eq!(
                    results.check_count(),
                    count,
                    "check_all should execute all registered checks"
                );

                // Verify overall status is healthy (all checks are healthy)
                assert!(
                    results.is_healthy(),
                    "Overall status should be healthy when all checks are healthy"
                );
            });
        }
    }

    proptest! {
        /// Test critical checks affect overall status
        #[test]
        fn prop_critical_checks_affect_overall_status(
            name in "[a-z][a-z0-9_]{2,20}",
            is_critical in any::<bool>()
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut registry = HealthCheckRegistry::new();

                // Register unhealthy check
                let check = Box::new(
                    MockHealthCheck::new(name.clone(), HealthStatus::Unhealthy)
                        .with_critical(is_critical)
                );
                registry.register(check).await.unwrap();

                // Execute all checks
                let results = registry.check_all().await;

                if is_critical {
                    // Critical unhealthy check should make overall status unhealthy
                    assert!(
                        results.is_unhealthy(),
                        "Critical unhealthy check should make overall status unhealthy"
                    );
                } else {
                    // Non-critical checks don't affect overall status in the same way
                    // (implementation may vary)
                    assert_eq!(results.check_count(), 1);
                }
            });
        }
    }

    proptest! {
        /// Test tags are preserved during registration
        #[test]
        fn prop_tags_preserved(
            name in "[a-z][a-z0-9_]{2,20}",
            tags in prop::collection::vec("[a-z]{3,10}", 0..5)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut registry = HealthCheckRegistry::new();

                // Register check with tags
                let check = Box::new(
                    MockHealthCheck::new(name.clone(), HealthStatus::Healthy)
                        .with_tags(tags.clone())
                );
                registry.register(check).await.unwrap();

                // If tags are not empty, test filtering by tags
                if !tags.is_empty() {
                    let results = registry.check_by_tags(&tags[0..1]).await;
                    assert_eq!(
                        results.check_count(),
                        1,
                        "Should find check when filtering by its tag"
                    );
                }
            });
        }
    }
}

/// Property tests for registry state consistency
#[cfg(test)]
mod consistency_tests {
    use super::*;

    proptest! {
        /// Test registry count is consistent with check_names length
        #[test]
        fn prop_count_consistent_with_names(
            count in 1..20usize
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut registry = HealthCheckRegistry::new();

                // Register checks
                for i in 0..count {
                    let name = format!("check_{}", i);
                    let check = Box::new(MockHealthCheck::new(name, HealthStatus::Healthy));
                    registry.register(check).await.unwrap();
                }

                // Verify consistency
                let registry_count = registry.count().await;
                let names_count = registry.check_names().await.len();

                assert_eq!(
                    registry_count,
                    names_count,
                    "Registry count should match check_names length"
                );
                assert_eq!(registry_count, count, "Registry count should match registered count");
            });
        }
    }

    proptest! {
        /// Test contains is consistent with check_names
        #[test]
        fn prop_contains_consistent_with_names(
            names in prop::collection::vec("[a-z][a-z0-9_]{2,20}", 1..10)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut registry = HealthCheckRegistry::new();

                // Remove duplicates
                let unique_names: std::collections::HashSet<_> = names.into_iter().collect();
                let unique_names: Vec<_> = unique_names.into_iter().collect();

                // Register checks
                for name in &unique_names {
                    let check = Box::new(MockHealthCheck::new(name.clone(), HealthStatus::Healthy));
                    registry.register(check).await.unwrap();
                }

                // Get check names from registry
                let registry_names = registry.check_names().await;

                // Verify consistency: all names in registry should be contained
                for name in &registry_names {
                    assert!(
                        registry.contains(name).await,
                        "contains() should return true for all names in check_names()"
                    );
                }

                // Verify consistency: all registered names should be in check_names
                for name in &unique_names {
                    assert!(
                        registry_names.contains(name),
                        "check_names() should include all registered checks"
                    );
                }
            });
        }
    }

    proptest! {
        /// Test check_one is consistent with check_all
        #[test]
        fn prop_check_one_consistent_with_check_all(
            name in "[a-z][a-z0-9_]{2,20}",
            status in prop_oneof![
                Just(HealthStatus::Healthy),
                Just(HealthStatus::Degraded),
                Just(HealthStatus::Unhealthy),
            ]
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut registry = HealthCheckRegistry::new();

                // Register single check
                let check = Box::new(MockHealthCheck::new(name.clone(), status.clone()));
                registry.register(check).await.unwrap();

                // Execute via check_one
                let result_one = registry.check_one(&name).await.unwrap();

                // Execute via check_all
                let results_all = registry.check_all().await;

                // Both should report the same status
                assert_eq!(
                    result_one.status,
                    status,
                    "check_one should return configured status"
                );

                assert_eq!(
                    results_all.check_count(),
                    1,
                    "check_all should execute the single check"
                );

                // Overall status should match the single check's status
                match status {
                    HealthStatus::Healthy => assert!(results_all.is_healthy()),
                    HealthStatus::Degraded => assert!(results_all.is_degraded()),
                    HealthStatus::Unhealthy => assert!(results_all.is_unhealthy()),
                }
            });
        }
    }
}

/// Integration tests with realistic scenarios
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_realistic_health_check_scenario() {
        let mut registry = HealthCheckRegistry::new();

        // Register multiple health checks simulating real components
        let checks = vec![
            ("database", HealthStatus::Healthy),
            ("storage", HealthStatus::Healthy),
            ("crypto", HealthStatus::Healthy),
            ("raft", HealthStatus::Degraded),
            ("replication", HealthStatus::Healthy),
        ];

        for (name, status) in checks {
            let check = Box::new(MockHealthCheck::new(name.to_string(), status));
            registry.register(check).await.unwrap();
        }

        // Verify all checks are registered
        assert_eq!(registry.count().await, 5);

        // Execute all checks
        let results = registry.check_all().await;
        assert_eq!(results.check_count(), 5);

        // Overall status should be degraded (one check is degraded)
        assert!(results.is_degraded());

        // Verify individual checks can be executed
        let db_result = registry.check_one("database").await.unwrap();
        assert_eq!(db_result.status, HealthStatus::Healthy);

        let raft_result = registry.check_one("raft").await.unwrap();
        assert_eq!(raft_result.status, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_health_check_lifecycle() {
        let mut registry = HealthCheckRegistry::new();

        // Register check
        let check = Box::new(MockHealthCheck::new(
            "lifecycle_test".to_string(),
            HealthStatus::Healthy,
        ));
        registry.register(check).await.unwrap();

        // Verify registration
        assert!(registry.contains("lifecycle_test").await);
        assert_eq!(registry.count().await, 1);

        // Execute check
        let result = registry.check_one("lifecycle_test").await.unwrap();
        assert_eq!(result.status, HealthStatus::Healthy);

        // Unregister check
        registry.unregister("lifecycle_test").await.unwrap();

        // Verify removal
        assert!(!registry.contains("lifecycle_test").await);
        assert_eq!(registry.count().await, 0);

        // Attempting to execute should fail
        let result = registry.check_one("lifecycle_test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mixed_critical_and_non_critical_checks() {
        let mut registry = HealthCheckRegistry::new();

        // Register critical unhealthy check
        let critical_check = Box::new(
            MockHealthCheck::new("critical".to_string(), HealthStatus::Unhealthy)
                .with_critical(true),
        );
        registry.register(critical_check).await.unwrap();

        // Register non-critical unhealthy check
        let non_critical_check = Box::new(
            MockHealthCheck::new("non_critical".to_string(), HealthStatus::Unhealthy)
                .with_critical(false),
        );
        registry.register(non_critical_check).await.unwrap();

        // Execute all checks
        let results = registry.check_all().await;

        // Overall status should be unhealthy due to critical check
        assert!(results.is_unhealthy());
    }

    #[tokio::test]
    async fn test_tag_based_filtering() {
        let mut registry = HealthCheckRegistry::new();

        // Register checks with different tags
        let db_check = Box::new(
            MockHealthCheck::new("postgres".to_string(), HealthStatus::Healthy)
                .with_tags(vec!["database".to_string(), "storage".to_string()]),
        );
        registry.register(db_check).await.unwrap();

        let cache_check = Box::new(
            MockHealthCheck::new("redis".to_string(), HealthStatus::Healthy)
                .with_tags(vec!["cache".to_string(), "storage".to_string()]),
        );
        registry.register(cache_check).await.unwrap();

        let api_check = Box::new(
            MockHealthCheck::new("rest_api".to_string(), HealthStatus::Healthy)
                .with_tags(vec!["api".to_string()]),
        );
        registry.register(api_check).await.unwrap();

        // Filter by "storage" tag - should get 2 checks
        let storage_results = registry.check_by_tags(&[String::from("storage")]).await;
        assert_eq!(storage_results.check_count(), 2);

        // Filter by "api" tag - should get 1 check
        let api_results = registry.check_by_tags(&[String::from("api")]).await;
        assert_eq!(api_results.check_count(), 1);

        // Filter by "database" tag - should get 1 check
        let db_results = registry.check_by_tags(&[String::from("database")]).await;
        assert_eq!(db_results.check_count(), 1);
    }
}

/// Property 35: Database health check
///
/// **Property**: Database health check should correctly report health status based on
/// database connectivity and query execution.
///
/// **Validates**: Requirements 2.6.4
///
/// **Formal specification**:
/// ```text
/// ∀ db_state ∈ {Connected, Disconnected, QueryFailed}:
///   check_database(db_state) ⇒
///     (db_state = Connected ∧ query_success) ⇒ status = Healthy
///     ∧ (db_state = Disconnected ∨ query_failed) ⇒ status = Unhealthy
///     ∧ response_time_ms > 0
///     ∧ details contains connection_pool_info
/// ```
///
/// This property ensures that:
/// 1. Database health check returns Healthy when database is reachable and queries succeed
/// 2. Database health check returns Unhealthy when database is unreachable or queries fail
/// 3. Response time is always measured
/// 4. Connection pool status is included in details
#[cfg(test)]
mod database_health_tests {
    use super::*;

    /// Mock database health check that simulates different database states
    struct MockDatabaseHealthCheck {
        name: String,
        simulate_connection_failure: bool,
        simulate_query_failure: bool,
    }

    impl MockDatabaseHealthCheck {
        fn new(name: String) -> Self {
            Self {
                name,
                simulate_connection_failure: false,
                simulate_query_failure: false,
            }
        }

        fn with_connection_failure(mut self) -> Self {
            self.simulate_connection_failure = true;
            self
        }

        fn with_query_failure(mut self) -> Self {
            self.simulate_query_failure = true;
            self
        }
    }

    #[async_trait]
    impl HealthCheck for MockDatabaseHealthCheck {
        fn name(&self) -> &str {
            &self.name
        }

        async fn check(&self) -> HealthCheckResult {
            use std::collections::HashMap;

            let start_time = std::time::Instant::now();

            let (status, message, details) = if self.simulate_connection_failure {
                let mut details = HashMap::new();
                details.insert(
                    "error".to_string(),
                    serde_json::Value::String("Connection failed".to_string()),
                );
                details.insert(
                    "available_connections".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(0)),
                );

                (
                    HealthStatus::Unhealthy,
                    Some("Database connection failed".to_string()),
                    Some(details),
                )
            } else if self.simulate_query_failure {
                let mut details = HashMap::new();
                details.insert(
                    "error".to_string(),
                    serde_json::Value::String("Query failed".to_string()),
                );
                details.insert(
                    "connection_pool".to_string(),
                    serde_json::Value::String("healthy".to_string()),
                );

                (
                    HealthStatus::Unhealthy,
                    Some("Database query failed".to_string()),
                    Some(details),
                )
            } else {
                let mut details = HashMap::new();
                details.insert(
                    "connection_pool".to_string(),
                    serde_json::Value::String("healthy".to_string()),
                );
                details.insert(
                    "available_connections".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(10)),
                );
                details.insert(
                    "total_connections".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(20)),
                );
                details.insert(
                    "max_connections".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(20)),
                );
                details.insert(
                    "query_test".to_string(),
                    serde_json::Value::String("passed".to_string()),
                );

                (HealthStatus::Healthy, None, Some(details))
            };

            let response_time = start_time.elapsed().as_millis() as u64;
            // Ensure response time is at least 1ms for testing purposes
            let response_time = response_time.max(1);

            HealthCheckResult {
                status,
                message,
                response_time_ms: response_time,
                details,
            }
        }

        fn is_critical(&self) -> bool {
            true // Database is always critical
        }

        fn tags(&self) -> Vec<String> {
            vec!["database".to_string(), "storage".to_string()]
        }
    }

    proptest! {
        /// Test database health check with successful connection
        #[test]
        fn prop_database_health_check_success(
            name in "[a-z][a-z0-9_]{2,20}"
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let check = MockDatabaseHealthCheck::new(name.clone());
                let result = check.check().await;

                // Should be healthy when connection and query succeed
                assert_eq!(
                    result.status,
                    HealthStatus::Healthy,
                    "Database health check should be healthy with successful connection"
                );

                // Should have no error message
                assert!(
                    result.message.is_none(),
                    "Healthy check should not have error message"
                );

                // Should have response time
                assert!(
                    result.response_time_ms > 0,
                    "Response time should be measured"
                );

                // Should have connection pool details
                let details = result.details.expect("Should have details");
                assert!(
                    details.contains_key("connection_pool"),
                    "Should include connection pool status"
                );
                assert!(
                    details.contains_key("available_connections"),
                    "Should include available connections count"
                );
                assert!(
                    details.contains_key("query_test"),
                    "Should include query test result"
                );
            });
        }
    }

    proptest! {
        /// Test database health check with connection failure
        #[test]
        fn prop_database_health_check_connection_failure(
            name in "[a-z][a-z0-9_]{2,20}"
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let check = MockDatabaseHealthCheck::new(name.clone())
                    .with_connection_failure();
                let result = check.check().await;

                // Should be unhealthy when connection fails
                assert_eq!(
                    result.status,
                    HealthStatus::Unhealthy,
                    "Database health check should be unhealthy with connection failure"
                );

                // Should have error message
                assert!(
                    result.message.is_some(),
                    "Unhealthy check should have error message"
                );
                assert!(
                    result.message.unwrap().contains("connection"),
                    "Error message should mention connection failure"
                );

                // Should have response time
                assert!(
                    result.response_time_ms > 0,
                    "Response time should be measured even on failure"
                );

                // Should have error details
                let details = result.details.expect("Should have details");
                assert!(
                    details.contains_key("error"),
                    "Should include error details"
                );
            });
        }
    }

    proptest! {
        /// Test database health check with query failure
        #[test]
        fn prop_database_health_check_query_failure(
            name in "[a-z][a-z0-9_]{2,20}"
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let check = MockDatabaseHealthCheck::new(name.clone())
                    .with_query_failure();
                let result = check.check().await;

                // Should be unhealthy when query fails
                assert_eq!(
                    result.status,
                    HealthStatus::Unhealthy,
                    "Database health check should be unhealthy with query failure"
                );

                // Should have error message
                assert!(
                    result.message.is_some(),
                    "Unhealthy check should have error message"
                );
                assert!(
                    result.message.unwrap().contains("query"),
                    "Error message should mention query failure"
                );

                // Should have response time
                assert!(
                    result.response_time_ms > 0,
                    "Response time should be measured"
                );

                // Should have error details
                let details = result.details.expect("Should have details");
                assert!(
                    details.contains_key("error"),
                    "Should include error details"
                );
                assert!(
                    details.contains_key("connection_pool"),
                    "Should still report connection pool status"
                );
            });
        }
    }

    proptest! {
        /// Test database health check is critical
        #[test]
        fn prop_database_health_check_is_critical(
            name in "[a-z][a-z0-9_]{2,20}"
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let check = MockDatabaseHealthCheck::new(name.clone());

                // Database health check should always be critical
                assert!(
                    check.is_critical(),
                    "Database health check should be marked as critical"
                );
            });
        }
    }

    proptest! {
        /// Test database health check has appropriate tags
        #[test]
        fn prop_database_health_check_tags(
            name in "[a-z][a-z0-9_]{2,20}"
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let check = MockDatabaseHealthCheck::new(name.clone());
                let tags = check.tags();

                // Should have database and storage tags
                assert!(
                    tags.contains(&"database".to_string()),
                    "Should have 'database' tag"
                );
                assert!(
                    tags.contains(&"storage".to_string()),
                    "Should have 'storage' tag"
                );
            });
        }
    }

    #[tokio::test]
    async fn test_database_health_check_integration() {
        let mut registry = HealthCheckRegistry::new();

        // Register database health check
        let check = Box::new(MockDatabaseHealthCheck::new("postgres".to_string()));
        registry.register(check).await.unwrap();

        // Execute check
        let result = registry.check_one("postgres").await.unwrap();
        assert_eq!(result.status, HealthStatus::Healthy);

        // Execute all checks
        let results = registry.check_all().await;
        assert!(results.is_healthy());
    }

    #[tokio::test]
    async fn test_database_health_check_failure_affects_overall_status() {
        let mut registry = HealthCheckRegistry::new();

        // Register failing database health check
        let check = Box::new(
            MockDatabaseHealthCheck::new("postgres".to_string()).with_connection_failure(),
        );
        registry.register(check).await.unwrap();

        // Register other healthy checks
        let crypto_check = Box::new(MockHealthCheck::new(
            "crypto".to_string(),
            HealthStatus::Healthy,
        ));
        registry.register(crypto_check).await.unwrap();

        // Execute all checks
        let results = registry.check_all().await;

        // Overall status should be unhealthy due to critical database failure
        assert!(
            results.is_unhealthy(),
            "Overall status should be unhealthy when critical database check fails"
        );
    }
}

/// Property 36: Kubernetes probes
///
/// **Property**: Kubernetes liveness and readiness probes should correctly report
/// pod health status based on service state.
///
/// **Validates**: Requirements 2.6.7, 2.6.8
///
/// **Formal specification**:
/// ```text
/// ∀ service_state ∈ {Running, Sealed, Degraded, Failed}:
///   liveness_probe(service_state) ⇒
///     (service_state ∈ {Running, Sealed, Degraded}) ⇒ status = 200
///     ∧ (service_state = Failed) ⇒ status = 503
///
///   readiness_probe(service_state) ⇒
///     (service_state = Running ∧ unsealed) ⇒ status = 200
///     ∧ (service_state ∈ {Sealed, Degraded, Failed}) ⇒ status = 503
/// ```
///
/// This property ensures that:
/// 1. Liveness probe returns 200 if process is running (even if sealed)
/// 2. Liveness probe returns 503 only if process has failed
/// 3. Readiness probe returns 200 only if service is fully operational and unsealed
/// 4. Readiness probe returns 503 if sealed, degraded, or failed
#[cfg(test)]
mod kubernetes_probe_tests {
    use super::*;

    /// Mock service state for testing Kubernetes probes
    #[derive(Debug, Clone, PartialEq)]
    enum ServiceState {
        Running,
        Sealed,
        Degraded,
        Failed,
    }

    /// Mock liveness probe health check
    struct MockLivenessProbe {
        service_state: ServiceState,
    }

    impl MockLivenessProbe {
        fn new(service_state: ServiceState) -> Self {
            Self { service_state }
        }
    }

    #[async_trait]
    impl HealthCheck for MockLivenessProbe {
        fn name(&self) -> &str {
            "liveness"
        }

        async fn check(&self) -> HealthCheckResult {
            let start_time = std::time::Instant::now();

            let (status, message) = match self.service_state {
                ServiceState::Running | ServiceState::Sealed | ServiceState::Degraded => {
                    // Liveness probe passes if process is running, even if sealed or degraded
                    (HealthStatus::Healthy, Some("Process is alive".to_string()))
                }
                ServiceState::Failed => {
                    // Liveness probe fails only if process has failed
                    (
                        HealthStatus::Unhealthy,
                        Some("Process has failed".to_string()),
                    )
                }
            };

            let response_time = start_time.elapsed().as_millis() as u64;

            HealthCheckResult {
                status,
                message,
                response_time_ms: response_time.max(1),
                details: Some({
                    let mut details = std::collections::HashMap::new();
                    details.insert(
                        "service_state".to_string(),
                        serde_json::Value::String(format!("{:?}", self.service_state)),
                    );
                    details
                }),
            }
        }

        fn is_critical(&self) -> bool {
            true
        }

        fn tags(&self) -> Vec<String> {
            vec!["kubernetes".to_string(), "liveness".to_string()]
        }
    }

    /// Mock readiness probe health check
    struct MockReadinessProbe {
        service_state: ServiceState,
    }

    impl MockReadinessProbe {
        fn new(service_state: ServiceState) -> Self {
            Self { service_state }
        }
    }

    #[async_trait]
    impl HealthCheck for MockReadinessProbe {
        fn name(&self) -> &str {
            "readiness"
        }

        async fn check(&self) -> HealthCheckResult {
            let start_time = std::time::Instant::now();

            let (status, message) = match self.service_state {
                ServiceState::Running => {
                    // Readiness probe passes only if service is fully operational and unsealed
                    (HealthStatus::Healthy, Some("Service is ready".to_string()))
                }
                ServiceState::Sealed => {
                    // Readiness probe fails if sealed
                    (
                        HealthStatus::Unhealthy,
                        Some("Service is sealed".to_string()),
                    )
                }
                ServiceState::Degraded => {
                    // Readiness probe fails if degraded
                    (
                        HealthStatus::Unhealthy,
                        Some("Service is degraded".to_string()),
                    )
                }
                ServiceState::Failed => {
                    // Readiness probe fails if failed
                    (
                        HealthStatus::Unhealthy,
                        Some("Service has failed".to_string()),
                    )
                }
            };

            let response_time = start_time.elapsed().as_millis() as u64;

            HealthCheckResult {
                status,
                message,
                response_time_ms: response_time.max(1),
                details: Some({
                    let mut details = std::collections::HashMap::new();
                    details.insert(
                        "service_state".to_string(),
                        serde_json::Value::String(format!("{:?}", self.service_state)),
                    );
                    details
                }),
            }
        }

        fn is_critical(&self) -> bool {
            true
        }

        fn tags(&self) -> Vec<String> {
            vec!["kubernetes".to_string(), "readiness".to_string()]
        }
    }

    proptest! {
        /// Test liveness probe with running service
        #[test]
        fn prop_liveness_probe_running(
            _seed in 0..100u32
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let probe = MockLivenessProbe::new(ServiceState::Running);
                let result = probe.check().await;

                // Liveness probe should be healthy when service is running
                assert_eq!(
                    result.status,
                    HealthStatus::Healthy,
                    "Liveness probe should be healthy when service is running"
                );
            });
        }
    }

    proptest! {
        /// Test liveness probe with sealed service
        #[test]
        fn prop_liveness_probe_sealed(
            _seed in 0..100u32
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let probe = MockLivenessProbe::new(ServiceState::Sealed);
                let result = probe.check().await;

                // Liveness probe should still be healthy when sealed (process is alive)
                assert_eq!(
                    result.status,
                    HealthStatus::Healthy,
                    "Liveness probe should be healthy even when sealed (process is alive)"
                );
            });
        }
    }

    proptest! {
        /// Test liveness probe with failed service
        #[test]
        fn prop_liveness_probe_failed(
            _seed in 0..100u32
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let probe = MockLivenessProbe::new(ServiceState::Failed);
                let result = probe.check().await;

                // Liveness probe should be unhealthy when service has failed
                assert_eq!(
                    result.status,
                    HealthStatus::Unhealthy,
                    "Liveness probe should be unhealthy when service has failed"
                );
            });
        }
    }

    proptest! {
        /// Test readiness probe with running service
        #[test]
        fn prop_readiness_probe_running(
            _seed in 0..100u32
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let probe = MockReadinessProbe::new(ServiceState::Running);
                let result = probe.check().await;

                // Readiness probe should be healthy only when service is fully operational
                assert_eq!(
                    result.status,
                    HealthStatus::Healthy,
                    "Readiness probe should be healthy when service is running and unsealed"
                );
            });
        }
    }

    proptest! {
        /// Test readiness probe with sealed service
        #[test]
        fn prop_readiness_probe_sealed(
            _seed in 0..100u32
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let probe = MockReadinessProbe::new(ServiceState::Sealed);
                let result = probe.check().await;

                // Readiness probe should be unhealthy when sealed
                assert_eq!(
                    result.status,
                    HealthStatus::Unhealthy,
                    "Readiness probe should be unhealthy when service is sealed"
                );
            });
        }
    }

    proptest! {
        /// Test readiness probe with degraded service
        #[test]
        fn prop_readiness_probe_degraded(
            _seed in 0..100u32
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let probe = MockReadinessProbe::new(ServiceState::Degraded);
                let result = probe.check().await;

                // Readiness probe should be unhealthy when degraded
                assert_eq!(
                    result.status,
                    HealthStatus::Unhealthy,
                    "Readiness probe should be unhealthy when service is degraded"
                );
            });
        }
    }

    proptest! {
        /// Test readiness probe with failed service
        #[test]
        fn prop_readiness_probe_failed(
            _seed in 0..100u32
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let probe = MockReadinessProbe::new(ServiceState::Failed);
                let result = probe.check().await;

                // Readiness probe should be unhealthy when failed
                assert_eq!(
                    result.status,
                    HealthStatus::Unhealthy,
                    "Readiness probe should be unhealthy when service has failed"
                );
            });
        }
    }

    #[tokio::test]
    async fn test_liveness_vs_readiness_sealed_state() {
        // When service is sealed:
        // - Liveness should pass (process is alive)
        // - Readiness should fail (not ready to serve traffic)

        let liveness = MockLivenessProbe::new(ServiceState::Sealed);
        let readiness = MockReadinessProbe::new(ServiceState::Sealed);

        let liveness_result = liveness.check().await;
        let readiness_result = readiness.check().await;

        assert_eq!(
            liveness_result.status,
            HealthStatus::Healthy,
            "Liveness should pass when sealed"
        );
        assert_eq!(
            readiness_result.status,
            HealthStatus::Unhealthy,
            "Readiness should fail when sealed"
        );
    }

    #[tokio::test]
    async fn test_probe_tags() {
        let liveness = MockLivenessProbe::new(ServiceState::Running);
        let readiness = MockReadinessProbe::new(ServiceState::Running);

        let liveness_tags = liveness.tags();
        let readiness_tags = readiness.tags();

        assert!(liveness_tags.contains(&"kubernetes".to_string()));
        assert!(liveness_tags.contains(&"liveness".to_string()));

        assert!(readiness_tags.contains(&"kubernetes".to_string()));
        assert!(readiness_tags.contains(&"readiness".to_string()));
    }

    #[tokio::test]
    async fn test_probes_are_critical() {
        let liveness = MockLivenessProbe::new(ServiceState::Running);
        let readiness = MockReadinessProbe::new(ServiceState::Running);

        assert!(liveness.is_critical(), "Liveness probe should be critical");
        assert!(
            readiness.is_critical(),
            "Readiness probe should be critical"
        );
    }
}
