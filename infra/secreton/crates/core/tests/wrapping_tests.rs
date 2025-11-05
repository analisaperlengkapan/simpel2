//! Integration tests for Response Wrapping Service
//!
//! Tests the complete lifecycle of wrapped tokens including:
//! - Wrap/unwrap operations
//! - One-time use enforcement
//! - TTL expiration
//! - Namespace isolation
//! - Size limits
//! - Cleanup operations
//!
//! # Prerequisites
//!
//! These tests require a PostgreSQL database running with the following configuration:
//! - Host: localhost
//! - Port: 5432
//! - Database: secreton_test
//! - User: postgres
//! - Password: postgres
//!
//! To run these tests:
//! ```bash
//! # Start PostgreSQL (if using Docker)
//! docker run -d --name secreton-test-db \
//!   -e POSTGRES_DB=secreton_test \
//!   -e POSTGRES_USER=postgres \
//!   -e POSTGRES_PASSWORD=postgres \
//!   -p 5432:5432 \
//!   postgres:15
//!
//! # Run tests
//! cargo test --package secreton-core --test wrapping_tests -- --test-threads=1
//! ```
//!
//! Note: Tests are run with --test-threads=1 to avoid database conflicts.

use chrono::Utc;
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use secreton_core::services::wrapping::{TokenStatus, WrapRequest, WrappingError, WrappingService};
use serde_json::json;
use std::time::Duration;
use tokio_postgres::NoTls;

/// Setup test database pool
async fn setup_test_pool() -> Pool {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.port = Some(5432);
    cfg.dbname = Some("secreton_test".to_string());
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("postgres".to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create pool")
}

/// Setup test database schema
async fn setup_test_schema(pool: &Pool) {
    let client = pool.get().await.expect("Failed to get client");

    // Drop and recreate table for clean state
    client
        .execute("DROP TABLE IF EXISTS wrapping_tokens CASCADE", &[])
        .await
        .expect("Failed to drop table");

    // Create table
    client
        .execute(
            "CREATE TABLE wrapping_tokens (
                token VARCHAR(255) PRIMARY KEY,
                encrypted_data BYTEA NOT NULL,
                encryption_metadata JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                expires_at TIMESTAMPTZ NOT NULL,
                namespace VARCHAR(255) NOT NULL DEFAULT 'default',
                status VARCHAR(50) NOT NULL DEFAULT 'Active',
                data_size INTEGER NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            &[],
        )
        .await
        .expect("Failed to create table");

    // Create indexes
    client
        .execute(
            "CREATE INDEX idx_wrapping_tokens_expires_at ON wrapping_tokens(expires_at)",
            &[],
        )
        .await
        .expect("Failed to create index");
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
async fn test_wrap_and_unwrap_success() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;

    let service = WrappingService::new(pool);

    // Wrap data
    let data = json!({
        "username": "admin",
        "password": "secret123",
        "host": "db.example.com"
    });

    let request = WrapRequest {
        data: data.clone(),
        ttl: Duration::from_secs(300),
        namespace: "default".to_string(),
    };

    let response = service.wrap(request).await.expect("Failed to wrap data");

    assert!(response.token.starts_with("wrap_"));
    assert_eq!(response.ttl, 300);
    assert!(response.expires_at > Utc::now());

    // Unwrap data
    let unwrapped = service
        .unwrap(&response.token, "default")
        .await
        .expect("Failed to unwrap data");

    assert_eq!(unwrapped, data);
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
async fn test_one_time_use_enforcement() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;

    let service = WrappingService::new(pool);

    // Wrap data
    let request = WrapRequest {
        data: json!({"secret": "value"}),
        ttl: Duration::from_secs(300),
        namespace: "default".to_string(),
    };

    let response = service.wrap(request).await.expect("Failed to wrap");

    // First unwrap should succeed
    let _ = service
        .unwrap(&response.token, "default")
        .await
        .expect("First unwrap should succeed");

    // Second unwrap should fail (token deleted)
    let result = service.unwrap(&response.token, "default").await;
    assert!(matches!(result, Err(WrappingError::TokenNotFound(_))));
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
async fn test_ttl_expiration() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;

    let service = WrappingService::new(pool);

    // Wrap with very short TTL
    let request = WrapRequest {
        data: json!({"test": "data"}),
        ttl: Duration::from_secs(1),
        namespace: "default".to_string(),
    };

    let response = service.wrap(request).await.expect("Failed to wrap");

    // Wait for expiration
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Unwrap should fail due to expiration
    let result = service.unwrap(&response.token, "default").await;
    assert!(matches!(result, Err(WrappingError::TokenExpired(_))));
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
async fn test_namespace_isolation() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;

    let service = WrappingService::new(pool);

    // Wrap in namespace1
    let request = WrapRequest {
        data: json!({"secret": "value"}),
        ttl: Duration::from_secs(300),
        namespace: "namespace1".to_string(),
    };

    let response = service.wrap(request).await.expect("Failed to wrap");

    // Try to unwrap from different namespace
    let result = service.unwrap(&response.token, "namespace2").await;
    assert!(matches!(result, Err(WrappingError::InvalidNamespace(_))));

    // Unwrap from correct namespace should succeed
    let _ = service
        .unwrap(&response.token, "namespace1")
        .await
        .expect("Should succeed with correct namespace");
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
async fn test_invalid_ttl() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;

    let service = WrappingService::new(pool);

    // TTL too short (0 seconds)
    let request = WrapRequest {
        data: json!({"test": "data"}),
        ttl: Duration::from_secs(0),
        namespace: "default".to_string(),
    };

    let result = service.wrap(request).await;
    assert!(matches!(result, Err(WrappingError::InvalidTtl(_))));

    // TTL too long (more than 24 hours)
    let request = WrapRequest {
        data: json!({"test": "data"}),
        ttl: Duration::from_secs(86401), // 24 hours + 1 second
        namespace: "default".to_string(),
    };

    let result = service.wrap(request).await;
    assert!(matches!(result, Err(WrappingError::InvalidTtl(_))));
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
async fn test_data_size_limit() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;

    let service = WrappingService::new(pool);

    // Create data larger than 1MB
    let large_data = "x".repeat(1024 * 1024 + 1);

    let request = WrapRequest {
        data: json!({"large": large_data}),
        ttl: Duration::from_secs(300),
        namespace: "default".to_string(),
    };

    let result = service.wrap(request).await;
    assert!(matches!(result, Err(WrappingError::DataTooLarge(_, _))));
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
async fn test_lookup_metadata() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;

    let service = WrappingService::new(pool);

    // Wrap data
    let request = WrapRequest {
        data: json!({"secret": "value"}),
        ttl: Duration::from_secs(300),
        namespace: "default".to_string(),
    };

    let response = service.wrap(request).await.expect("Failed to wrap");

    // Lookup metadata
    let info = service
        .lookup(&response.token, "default")
        .await
        .expect("Failed to lookup");

    assert_eq!(info.token, response.token);
    assert_eq!(info.namespace, "default");
    assert_eq!(info.status, TokenStatus::Active);
    assert!(info.ttl_remaining > 0);
    assert!(info.ttl_remaining <= 300);

    // Lookup should not consume the token
    let _ = service
        .unwrap(&response.token, "default")
        .await
        .expect("Token should still be usable after lookup");
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
async fn test_cleanup_expired_tokens() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;

    let service = WrappingService::new(pool);

    // Create multiple tokens with short TTL
    for i in 0..5 {
        let request = WrapRequest {
            data: json!({"index": i}),
            ttl: Duration::from_secs(1),
            namespace: "default".to_string(),
        };
        service.wrap(request).await.expect("Failed to wrap");
    }

    // Wait for expiration
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Cleanup expired tokens
    let count = service.cleanup_expired().await.expect("Failed to cleanup");

    assert_eq!(count, 5, "Should cleanup all 5 expired tokens");
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
async fn test_token_not_found() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;

    let service = WrappingService::new(pool);

    // Try to unwrap non-existent token
    let result = service.unwrap("wrap_nonexistent", "default").await;
    assert!(matches!(result, Err(WrappingError::TokenNotFound(_))));

    // Try to lookup non-existent token
    let result = service.lookup("wrap_nonexistent", "default").await;
    assert!(matches!(result, Err(WrappingError::TokenNotFound(_))));
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
async fn test_complex_data_structures() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;

    let service = WrappingService::new(pool);

    // Wrap complex nested data
    let data = json!({
        "database": {
            "host": "db.example.com",
            "port": 5432,
            "credentials": {
                "username": "admin",
                "password": "secret123"
            },
            "ssl": {
                "enabled": true,
                "cert": "-----BEGIN CERTIFICATE-----\nMIIC..."
            }
        },
        "api_keys": ["key1", "key2", "key3"],
        "metadata": {
            "created_at": "2025-01-01T00:00:00Z",
            "expires_at": "2025-12-31T23:59:59Z"
        }
    });

    let request = WrapRequest {
        data: data.clone(),
        ttl: Duration::from_secs(300),
        namespace: "default".to_string(),
    };

    let response = service.wrap(request).await.expect("Failed to wrap");

    // Unwrap and verify structure is preserved
    let unwrapped = service
        .unwrap(&response.token, "default")
        .await
        .expect("Failed to unwrap");

    assert_eq!(unwrapped, data);
    assert_eq!(unwrapped["database"]["port"], 5432);
    assert_eq!(unwrapped["api_keys"][0], "key1");
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
async fn test_concurrent_wrap_operations() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;

    let service = std::sync::Arc::new(WrappingService::new(pool));

    // Spawn multiple concurrent wrap operations
    let mut handles = vec![];
    for i in 0..10 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let request = WrapRequest {
                data: json!({"index": i}),
                ttl: Duration::from_secs(300),
                namespace: "default".to_string(),
            };
            service_clone.wrap(request).await
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
async fn test_wrap_empty_data() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;

    let service = WrappingService::new(pool);

    // Wrap empty object
    let request = WrapRequest {
        data: json!({}),
        ttl: Duration::from_secs(300),
        namespace: "default".to_string(),
    };

    let response = service.wrap(request).await.expect("Failed to wrap");

    let unwrapped = service
        .unwrap(&response.token, "default")
        .await
        .expect("Failed to unwrap");

    assert_eq!(unwrapped, json!({}));
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database - run with: cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1"]
async fn test_multiple_namespaces() {
    let pool = setup_test_pool().await;
    setup_test_schema(&pool).await;

    let service = WrappingService::new(pool);

    // Wrap in different namespaces
    let namespaces = vec!["ns1", "ns2", "ns3"];
    let mut tokens = vec![];

    for ns in &namespaces {
        let request = WrapRequest {
            data: json!({"namespace": ns}),
            ttl: Duration::from_secs(300),
            namespace: ns.to_string(),
        };
        let response = service.wrap(request).await.expect("Failed to wrap");
        tokens.push((response.token, ns.to_string()));
    }

    // Unwrap from correct namespaces
    for (token, ns) in tokens {
        let unwrapped = service.unwrap(&token, &ns).await.expect("Failed to unwrap");
        assert_eq!(unwrapped["namespace"], ns);
    }
}
