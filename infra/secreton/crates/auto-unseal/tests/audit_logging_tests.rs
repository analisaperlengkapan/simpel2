//! Tests for auto-unseal audit logging
//!
//! This test suite verifies that all auto-unseal operations are properly audited
//! according to requirement AC 2.1.7: "Audit log records all unseal attempts (auto and manual)"

use secreton_auto_unseal::{
    AutoUnsealManager, AutoUnsealProvider, ProviderMetadata, UnsealResult,
    config::FallbackConfig,
};
use secreton_core::audit::{AuditLogger, AuditQuery, AuditStatus, MemoryBackend};
use secreton_core::error::SecretonError;
use async_trait::async_trait;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Mock provider for testing audit logging
struct MockProvider {
    fail_count: Arc<AtomicU32>,
    max_failures: u32,
}

impl MockProvider {
    fn new(max_failures: u32) -> Self {
        Self {
            fail_count: Arc::new(AtomicU32::new(0)),
            max_failures,
        }
    }
}

#[async_trait]
impl AutoUnsealProvider for MockProvider {
    fn name(&self) -> &str {
        "mock-provider"
    }

    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        Ok(plaintext.to_vec())
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        let count = self.fail_count.fetch_add(1, Ordering::SeqCst);

        if count < self.max_failures {
            Err(SecretonError::Core(secreton_core::error::CoreError::internal(
                format!("Mock failure {}/{}", count + 1, self.max_failures),
            )))
        } else {
            Ok(ciphertext.to_vec())
        }
    }

    async fn health_check(&self) -> Result<(), SecretonError> {
        Ok(())
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata::new("mock-provider".to_string(), "test-key-123".to_string())
            .with_region("us-east-1".to_string())
            .with_endpoint("https://mock.example.com".to_string())
    }
}

#[tokio::test]
async fn test_audit_unseal_success() {
    // Setup: Create audit backend and logger
    let backend = Arc::new(MemoryBackend::default());
    let audit_logger = Arc::new(AuditLogger::new(vec![backend.clone()]));

    // Setup: Create provider that succeeds immediately
    let provider = Box::new(MockProvider::new(0));
    let config = FallbackConfig::default();
    let manager = AutoUnsealManager::with_audit_logger(provider, config, audit_logger);

    // Execute: Attempt auto-unseal
    let encrypted = b"test-encrypted-key";
    let (master_key, result) = manager
        .unseal_with_fallback(encrypted)
        .await
        .expect("Unseal should succeed");

    // Verify: Unseal succeeded
    assert_eq!(master_key, encrypted);
    assert_eq!(result, UnsealResult::Success);

    // Verify: Audit logs were created
    let logs = backend.logs();
    assert!(!logs.is_empty(), "Audit logs should be created");

    // Verify: "initiated" event was logged
    let initiated_logs: Vec<_> = logs
        .iter()
        .filter(|log| log.action == "auto_unseal.initiated")
        .collect();
    assert_eq!(initiated_logs.len(), 1, "Should have one 'initiated' log");
    assert_eq!(initiated_logs[0].status, AuditStatus::Success);
    assert_eq!(initiated_logs[0].resource_type, "seal");
    assert_eq!(initiated_logs[0].resource_id, "master_key");
    assert!(initiated_logs[0].metadata.contains_key("correlation_id"));
    assert_eq!(initiated_logs[0].metadata.get("provider_type").unwrap(), "mock-provider");
    assert_eq!(initiated_logs[0].metadata.get("key_id").unwrap(), "test-key-123");

    // Verify: "success" event was logged
    let success_logs: Vec<_> = logs
        .iter()
        .filter(|log| log.action == "auto_unseal.success")
        .collect();
    assert_eq!(success_logs.len(), 1, "Should have one 'success' log");
    assert_eq!(success_logs[0].status, AuditStatus::Success);

    // Verify: Correlation IDs match
    let correlation_id_initiated = initiated_logs[0].metadata.get("correlation_id").unwrap();
    let correlation_id_success = success_logs[0].metadata.get("correlation_id").unwrap();
    assert_eq!(
        correlation_id_initiated, correlation_id_success,
        "Correlation IDs should match across events"
    );
}

#[tokio::test]
async fn test_audit_retry_attempts() {
    // Setup: Create audit backend and logger
    let backend = Arc::new(MemoryBackend::default());
    let audit_logger = Arc::new(AuditLogger::new(vec![backend.clone()]));

    // Setup: Create provider that fails 2 times before succeeding
    let provider = Box::new(MockProvider::new(2));
    let config = FallbackConfig {
        fallback_to_manual: true,
        max_retries: 5,
        initial_retry_delay_secs: 0, // No delay for testing
        max_retry_delay_secs: 0,
    };
    let manager = AutoUnsealManager::with_audit_logger(provider, config, audit_logger);

    // Execute: Attempt auto-unseal
    let encrypted = b"test-encrypted-key";
    let (_master_key, result) = manager
        .unseal_with_fallback(encrypted)
        .await
        .expect("Unseal should succeed after retries");

    // Verify: Unseal succeeded
    assert_eq!(result, UnsealResult::Success);

    // Verify: Audit logs were created
    let logs = backend.logs();

    // Verify: "retry_failed" events were logged (2 failures)
    let retry_failed_logs: Vec<_> = logs
        .iter()
        .filter(|log| log.action == "auto_unseal.retry_failed")
        .collect();
    assert_eq!(retry_failed_logs.len(), 2, "Should have two 'retry_failed' logs");

    for (i, log) in retry_failed_logs.iter().enumerate() {
        assert_eq!(log.status, AuditStatus::Failure);
        assert!(log.metadata.contains_key("attempt"));
        assert!(log.metadata.contains_key("error"));
        assert_eq!(log.metadata.get("attempt").unwrap(), &(i + 1).to_string());
    }

    // Verify: "retry_success" event was logged (3rd attempt succeeded)
    let retry_success_logs: Vec<_> = logs
        .iter()
        .filter(|log| log.action == "auto_unseal.retry_success")
        .collect();
    assert_eq!(retry_success_logs.len(), 1, "Should have one 'retry_success' log");
    assert_eq!(retry_success_logs[0].status, AuditStatus::Success);
    assert_eq!(retry_success_logs[0].metadata.get("attempt").unwrap(), "3");

    // Verify: All events have the same correlation ID
    let correlation_ids: Vec<_> = logs
        .iter()
        .filter_map(|log| log.metadata.get("correlation_id"))
        .collect();
    assert!(correlation_ids.len() > 1, "Should have multiple events with correlation IDs");
    let first_id = correlation_ids[0];
    assert!(
        correlation_ids.iter().all(|id| *id == first_id),
        "All events should have the same correlation ID"
    );
}

#[tokio::test]
async fn test_audit_fallback_to_manual() {
    // Setup: Create audit backend and logger
    let backend = Arc::new(MemoryBackend::default());
    let audit_logger = Arc::new(AuditLogger::new(vec![backend.clone()]));

    // Setup: Create provider that always fails
    let provider = Box::new(MockProvider::new(10));
    let config = FallbackConfig {
        fallback_to_manual: true,
        max_retries: 3,
        initial_retry_delay_secs: 0,
        max_retry_delay_secs: 0,
    };
    let manager = AutoUnsealManager::with_audit_logger(provider, config, audit_logger);

    // Execute: Attempt auto-unseal
    let encrypted = b"test-encrypted-key";
    let (master_key, result) = manager
        .unseal_with_fallback(encrypted)
        .await
        .expect("Should fall back to manual");

    // Verify: Fell back to manual unseal
    assert!(master_key.is_empty());
    assert_eq!(result, UnsealResult::FallbackToManual);

    // Verify: Audit logs were created
    let logs = backend.logs();

    // Verify: "initiated" event was logged
    let initiated_logs: Vec<_> = logs
        .iter()
        .filter(|log| log.action == "auto_unseal.initiated")
        .collect();
    assert_eq!(initiated_logs.len(), 1);

    // Verify: Multiple "retry_failed" events were logged
    let retry_failed_logs: Vec<_> = logs
        .iter()
        .filter(|log| log.action == "auto_unseal.retry_failed")
        .collect();
    assert_eq!(retry_failed_logs.len(), 4, "Should have 4 retry_failed logs (initial + 3 retries)");

    // Verify: "fallback_to_manual" event was logged
    let fallback_logs: Vec<_> = logs
        .iter()
        .filter(|log| log.action == "auto_unseal.fallback_to_manual")
        .collect();
    assert_eq!(fallback_logs.len(), 1, "Should have one 'fallback_to_manual' log");
    assert_eq!(fallback_logs[0].status, AuditStatus::Failure);
    assert!(fallback_logs[0].metadata.contains_key("error"));
    assert_eq!(fallback_logs[0].metadata.get("fallback_enabled").unwrap(), "true");
}

#[tokio::test]
async fn test_audit_failure_no_fallback() {
    // Setup: Create audit backend and logger
    let backend = Arc::new(MemoryBackend::default());
    let audit_logger = Arc::new(AuditLogger::new(vec![backend.clone()]));

    // Setup: Create provider that always fails
    let provider = Box::new(MockProvider::new(10));
    let config = FallbackConfig {
        fallback_to_manual: false, // Fallback disabled
        max_retries: 2,
        initial_retry_delay_secs: 0,
        max_retry_delay_secs: 0,
    };
    let manager = AutoUnsealManager::with_audit_logger(provider, config, audit_logger);

    // Execute: Attempt auto-unseal
    let encrypted = b"test-encrypted-key";
    let result = manager.unseal_with_fallback(encrypted).await;

    // Verify: Unseal failed
    assert!(result.is_err(), "Should fail when fallback is disabled");

    // Verify: Audit logs were created
    let logs = backend.logs();

    // Verify: "initiated" event was logged
    let initiated_logs: Vec<_> = logs
        .iter()
        .filter(|log| log.action == "auto_unseal.initiated")
        .collect();
    assert_eq!(initiated_logs.len(), 1);
    assert_eq!(initiated_logs[0].metadata.get("fallback_enabled").unwrap(), "false");

    // Verify: "failed" event was logged (not "fallback_to_manual")
    let failed_logs: Vec<_> = logs
        .iter()
        .filter(|log| log.action == "auto_unseal.failed")
        .collect();
    assert_eq!(failed_logs.len(), 1, "Should have one 'failed' log");
    assert_eq!(failed_logs[0].status, AuditStatus::Failure);
    assert!(failed_logs[0].metadata.contains_key("error"));
    assert_eq!(failed_logs[0].metadata.get("fallback_enabled").unwrap(), "false");

    // Verify: No "fallback_to_manual" event
    let fallback_logs: Vec<_> = logs
        .iter()
        .filter(|log| log.action == "auto_unseal.fallback_to_manual")
        .collect();
    assert_eq!(fallback_logs.len(), 0, "Should not have 'fallback_to_manual' log");
}

#[tokio::test]
async fn test_audit_no_sensitive_data() {
    // Setup: Create audit backend and logger
    let backend = Arc::new(MemoryBackend::default());
    let audit_logger = Arc::new(AuditLogger::new(vec![backend.clone()]));

    // Setup: Create provider that succeeds
    let provider = Box::new(MockProvider::new(0));
    let config = FallbackConfig::default();
    let manager = AutoUnsealManager::with_audit_logger(provider, config, audit_logger);

    // Execute: Attempt auto-unseal with "sensitive" data
    let encrypted = b"super-secret-master-key-do-not-log";
    let (_master_key, _result) = manager
        .unseal_with_fallback(encrypted)
        .await
        .expect("Unseal should succeed");

    // Verify: Audit logs do not contain sensitive data
    let logs = backend.logs();

    for log in logs {
        // Convert log to JSON to check all fields
        let log_json = serde_json::to_string(&log).expect("Should serialize");

        // Verify: Master key is not in the log
        assert!(
            !log_json.contains("super-secret-master-key"),
            "Audit log should not contain master key plaintext"
        );

        // Verify: Encrypted data is not in the log
        assert!(
            !log_json.contains("do-not-log"),
            "Audit log should not contain encrypted data"
        );
    }
}

#[tokio::test]
async fn test_audit_query_by_action() {
    // Setup: Create audit backend and logger
    let backend = Arc::new(MemoryBackend::default());
    let audit_logger = Arc::new(AuditLogger::new(vec![backend.clone()]));

    // Setup: Create provider that fails once then succeeds
    let provider = Box::new(MockProvider::new(1));
    let config = FallbackConfig {
        fallback_to_manual: true,
        max_retries: 3,
        initial_retry_delay_secs: 0,
        max_retry_delay_secs: 0,
    };
    let manager = AutoUnsealManager::with_audit_logger(provider, config, audit_logger.clone());

    // Execute: Attempt auto-unseal
    let encrypted = b"test-encrypted-key";
    let (_master_key, _result) = manager
        .unseal_with_fallback(encrypted)
        .await
        .expect("Unseal should succeed");

    // Verify: Can query audit logs by action
    let query = AuditQuery::new().action("auto_unseal.initiated");
    let results = audit_logger.query(&query).await.expect("Query should succeed");
    assert_eq!(results.len(), 1, "Should find one 'initiated' event");

    let query = AuditQuery::new().action("auto_unseal.retry_failed");
    let results = audit_logger.query(&query).await.expect("Query should succeed");
    assert_eq!(results.len(), 1, "Should find one 'retry_failed' event");

    let query = AuditQuery::new().action("auto_unseal.retry_success");
    let results = audit_logger.query(&query).await.expect("Query should succeed");
    assert_eq!(results.len(), 1, "Should find one 'retry_success' event");
}

#[tokio::test]
async fn test_audit_query_by_status() {
    // Setup: Create audit backend and logger
    let backend = Arc::new(MemoryBackend::default());
    let audit_logger = Arc::new(AuditLogger::new(vec![backend.clone()]));

    // Setup: Create provider that fails twice then succeeds
    let provider = Box::new(MockProvider::new(2));
    let config = FallbackConfig {
        fallback_to_manual: true,
        max_retries: 5,
        initial_retry_delay_secs: 0,
        max_retry_delay_secs: 0,
    };
    let manager = AutoUnsealManager::with_audit_logger(provider, config, audit_logger.clone());

    // Execute: Attempt auto-unseal
    let encrypted = b"test-encrypted-key";
    let (_master_key, _result) = manager
        .unseal_with_fallback(encrypted)
        .await
        .expect("Unseal should succeed");

    // Verify: Can query successful events
    let query = AuditQuery::new().status(AuditStatus::Success);
    let results = audit_logger.query(&query).await.expect("Query should succeed");
    assert!(results.len() >= 2, "Should have at least 2 successful events (initiated + success)");

    // Verify: Can query failed events
    let query = AuditQuery::new().status(AuditStatus::Failure);
    let results = audit_logger.query(&query).await.expect("Query should succeed");
    assert_eq!(results.len(), 2, "Should have 2 failed events (2 retry failures)");
}
