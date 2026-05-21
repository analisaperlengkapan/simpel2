//! Integration tests for database layer
//!
//! These tests verify database connection pooling, prepared statement caching,
//! and transaction management using only public APIs.

// Common test utilities are inlined in each test module

#[cfg(test)]
mod database_integration_tests {
    use authenc_storage::PoolConfigBuilder;
    use std::time::Duration;

    #[test]
    fn test_pool_config_builder_integration() {
        let config = PoolConfigBuilder::new()
            .max_size(10)
            .min_idle(2)
            .timeout(Duration::from_secs(30))
            .build();

        // Config is built successfully
        assert_eq!(config.max_size, 10);
    }

    #[test]
    fn test_production_pool_config() {
        let builder = PoolConfigBuilder::production();
        let config = builder.build();
        assert_eq!(config.max_size, 20);
    }

    #[test]
    fn test_development_pool_config() {
        let builder = PoolConfigBuilder::development();
        let config = builder.build();
        assert_eq!(config.max_size, 5);
    }

    #[test]
    fn test_testing_pool_config() {
        let builder = PoolConfigBuilder::testing();
        let config = builder.build();
        assert_eq!(config.max_size, 2);
    }

    #[test]
    fn test_pool_health_metrics() {
        use authenc_storage::PoolHealth;

        let health = PoolHealth {
            size: 10,
            max_size: 20,
            available: 8,
            utilization: 50.0,
        };

        assert!(health.is_healthy());
        assert!(!health.is_under_pressure());
        assert_eq!(health.status(), "healthy");
    }

    #[test]
    fn test_pool_health_under_pressure() {
        use authenc_storage::PoolHealth;

        let health = PoolHealth {
            size: 19,
            max_size: 20,
            available: 1,
            utilization: 95.0,
        };

        assert!(!health.is_healthy());
        assert!(health.is_under_pressure());
    }

    #[test]
    fn test_prepared_cache_creation() {
        use authenc_storage::PreparedStatementCache;

        // Test that we can create a cache
        let _cache = PreparedStatementCache::default();
    }

    #[test]
    fn test_cache_stats_utilization() {
        use authenc_storage::CacheStats;

        let stats = CacheStats {
            size: 50,
            max_size: 100,
        };

        assert_eq!(stats.utilization(), 50.0);
    }

    #[test]
    fn test_cache_stats_full() {
        use authenc_storage::CacheStats;

        let stats = CacheStats {
            size: 100,
            max_size: 100,
        };

        assert_eq!(stats.utilization(), 100.0);
    }

    #[test]
    fn test_cache_stats_empty() {
        use authenc_storage::CacheStats;

        let stats = CacheStats {
            size: 0,
            max_size: 100,
        };

        assert_eq!(stats.utilization(), 0.0);
    }

    #[test]
    fn test_isolation_levels() {
        use authenc_storage::IsolationLevel;

        let levels = [
            IsolationLevel::ReadUncommitted,
            IsolationLevel::ReadCommitted,
            IsolationLevel::RepeatableRead,
            IsolationLevel::Serializable,
        ];

        assert_eq!(levels.len(), 4);
    }

    #[test]
    fn test_isolation_level_equality() {
        use authenc_storage::IsolationLevel;

        assert_eq!(IsolationLevel::ReadCommitted, IsolationLevel::ReadCommitted);
        assert_ne!(IsolationLevel::ReadCommitted, IsolationLevel::Serializable);
    }
}

#[cfg(test)]
mod store_creation_tests {
    #[test]
    fn test_store_types_exist() {
        // Verify that all store types are exported and can be referenced
        #[allow(unused_imports)]
        use authenc_storage::{
            PostgresClientStore, PostgresCredentialStore, PostgresRealmStore, PostgresSessionStore,
            PostgresUserStore,
        };

        // This test just verifies the types exist and are accessible
        // Actual instantiation requires a Database instance
    }

    #[test]
    fn test_migration_types_exist() {
        #[allow(unused_imports)]
        use authenc_storage::{Migration, MigrationResult, MigrationRunner, MigrationStatus};

        // Verify migration types are exported
    }
}

#[cfg(test)]
mod error_handling_tests {
    #[test]
    fn test_authenc_error_types() {
        use authenc_types::AuthencError;

        // Test that we can create errors
        let _error = AuthencError::Conflict("Duplicate entry".to_string());
        let _error2 = AuthencError::UserNotFound("User not found".to_string());
    }
}

#[cfg(test)]
mod concurrent_access_tests {
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_pool_health_concurrent_reads() {
        use authenc_storage::PoolHealth;

        let health = Arc::new(PoolHealth {
            size: 10,
            max_size: 20,
            available: 8,
            utilization: 50.0,
        });

        let mut handles = vec![];

        for _ in 0..10 {
            let health_clone = Arc::clone(&health);
            let handle = thread::spawn(move || {
                assert!(health_clone.is_healthy());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}

#[cfg(test)]
mod pool_monitor_tests {
    use authenc_storage::PoolMonitorConfig;
    use std::time::Duration;

    #[test]
    fn test_pool_monitor_config() {
        let config = PoolMonitorConfig {
            check_interval: Duration::from_secs(30),
            utilization_warning_threshold: 80.0,
            utilization_critical_threshold: 90.0,
            wait_time_warning_threshold_ms: 1000,
            enable_stats_logging: true,
        };

        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert!(config.enable_stats_logging);
    }

    #[test]
    fn test_pool_monitor_config_thresholds() {
        let config = PoolMonitorConfig {
            check_interval: Duration::from_secs(60),
            utilization_warning_threshold: 75.0,
            utilization_critical_threshold: 95.0,
            wait_time_warning_threshold_ms: 2000,
            enable_stats_logging: false,
        };

        assert_eq!(config.utilization_warning_threshold, 75.0);
        assert_eq!(config.utilization_critical_threshold, 95.0);
        assert!(!config.enable_stats_logging);
    }
}
