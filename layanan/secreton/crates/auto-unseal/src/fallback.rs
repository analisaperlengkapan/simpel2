//! Auto-unseal fallback mechanism
//!
//! This module implements the fallback logic for auto-unseal operations.
//! When auto-unseal fails, it can optionally fall back to manual unseal mode
//! after retrying with exponential backoff.

use crate::{AutoUnsealProvider, config::FallbackConfig};
use secreton_core::audit::{AuditLog, AuditLogger, AuditStatus};
use secreton_core::error::SecretonError;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Result of an auto-unseal attempt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnsealResult {
    /// Auto-unseal succeeded
    Success,
    /// Auto-unseal failed, fell back to manual unseal
    FallbackToManual,
    /// Auto-unseal failed, no fallback configured
    Failed,
}

/// Auto-unseal manager with fallback support
pub struct AutoUnsealManager {
    provider: Box<dyn AutoUnsealProvider>,
    fallback_config: FallbackConfig,
    audit_logger: Option<Arc<AuditLogger>>,
    correlation_id: Uuid,
}

impl AutoUnsealManager {
    /// Create a new auto-unseal manager
    pub fn new(provider: Box<dyn AutoUnsealProvider>, fallback_config: FallbackConfig) -> Self {
        Self {
            provider,
            fallback_config,
            audit_logger: None,
            correlation_id: Uuid::new_v4(),
        }
    }

    /// Create a new auto-unseal manager with audit logging
    pub fn with_audit_logger(
        provider: Box<dyn AutoUnsealProvider>,
        fallback_config: FallbackConfig,
        audit_logger: Arc<AuditLogger>,
    ) -> Self {
        Self {
            provider,
            fallback_config,
            audit_logger: Some(audit_logger),
            correlation_id: Uuid::new_v4(),
        }
    }

    /// Attempt to auto-unseal with retry and fallback logic
    ///
    /// This method will:
    /// 1. Attempt to decrypt the master key using the configured provider
    /// 2. Retry with exponential backoff if decryption fails
    /// 3. Fall back to manual unseal mode if all retries fail (if configured)
    ///
    /// # Arguments
    ///
    /// * `encrypted_master_key` - The encrypted master key to decrypt
    ///
    /// # Returns
    ///
    /// * `Ok((master_key, UnsealResult))` - The decrypted master key and the result status
    /// * `Err(SecretonError)` - If auto-unseal failed and fallback is disabled
    pub async fn unseal_with_fallback(
        &self,
        encrypted_master_key: &[u8],
    ) -> Result<(Vec<u8>, UnsealResult), SecretonError> {
        let provider_metadata = self.provider.metadata();

        info!(
            provider = self.provider.name(),
            max_retries = self.fallback_config.max_retries,
            fallback_enabled = self.fallback_config.fallback_to_manual,
            correlation_id = %self.correlation_id,
            "Starting auto-unseal with fallback"
        );

        // Audit: Auto-unseal attempt initiated
        self.audit_unseal_initiated(&provider_metadata).await;

        // Attempt auto-unseal with retries
        match self.attempt_unseal_with_retries(encrypted_master_key).await {
            Ok(master_key) => {
                info!(correlation_id = %self.correlation_id, "Auto-unseal succeeded");

                // Audit: Auto-unseal success
                self.audit_unseal_success(&provider_metadata).await;

                Ok((master_key, UnsealResult::Success))
            }
            Err(e) => {
                error!(
                    error = %e,
                    correlation_id = %self.correlation_id,
                    "Auto-unseal failed after all retries"
                );

                // Check if fallback is enabled
                if self.fallback_config.fallback_to_manual {
                    warn!(correlation_id = %self.correlation_id, "Falling back to manual unseal mode");

                    // Audit: Fallback to manual unseal
                    self.audit_fallback_to_manual(&provider_metadata, &e).await;

                    // Return empty key to signal manual unseal is required
                    Ok((Vec::new(), UnsealResult::FallbackToManual))
                } else {
                    error!(
                        correlation_id = %self.correlation_id,
                        "Fallback to manual unseal is disabled, remaining sealed"
                    );

                    // Audit: Auto-unseal failed (no fallback)
                    self.audit_unseal_failure(&provider_metadata, &e).await;

                    Err(e)
                }
            }
        }
    }

    /// Attempt to unseal with exponential backoff retries
    async fn attempt_unseal_with_retries(
        &self,
        encrypted_master_key: &[u8],
    ) -> Result<Vec<u8>, SecretonError> {
        let mut attempt = 0;
        let mut delay_secs = self.fallback_config.initial_retry_delay_secs;
        let provider_metadata = self.provider.metadata();

        loop {
            attempt += 1;

            info!(
                attempt = attempt,
                max_retries = self.fallback_config.max_retries + 1,
                provider = self.provider.name(),
                correlation_id = %self.correlation_id,
                "Attempting auto-unseal"
            );

            // Attempt to decrypt the master key
            match self.provider.decrypt(encrypted_master_key).await {
                Ok(master_key) => {
                    info!(
                        attempt = attempt,
                        correlation_id = %self.correlation_id,
                        "Auto-unseal successful"
                    );

                    // Audit: Retry succeeded (if this wasn't the first attempt)
                    if attempt > 1 {
                        self.audit_retry_success(&provider_metadata, attempt).await;
                    }

                    return Ok(master_key);
                }
                Err(e) => {
                    error!(
                        attempt = attempt,
                        error = %e,
                        correlation_id = %self.correlation_id,
                        "Auto-unseal attempt failed"
                    );

                    // Audit: Retry attempt failed
                    self.audit_retry_failure(&provider_metadata, attempt, &e)
                        .await;

                    // Check if we've exhausted all retries
                    if attempt > self.fallback_config.max_retries {
                        error!(
                            correlation_id = %self.correlation_id,
                            "All auto-unseal retry attempts exhausted"
                        );
                        return Err(e);
                    }

                    // Log retry information
                    warn!(
                        attempt = attempt,
                        delay_secs = delay_secs,
                        remaining_attempts = self.fallback_config.max_retries + 1 - attempt,
                        correlation_id = %self.correlation_id,
                        "Retrying auto-unseal after delay"
                    );

                    // Wait before retrying (exponential backoff)
                    sleep(Duration::from_secs(delay_secs)).await;

                    // Calculate next delay with exponential backoff
                    delay_secs =
                        std::cmp::min(delay_secs * 2, self.fallback_config.max_retry_delay_secs);
                }
            }
        }
    }

    /// Get the configured provider
    pub fn provider(&self) -> &dyn AutoUnsealProvider {
        self.provider.as_ref()
    }

    /// Get the fallback configuration
    pub fn fallback_config(&self) -> &FallbackConfig {
        &self.fallback_config
    }

    /// Get the correlation ID for this unseal session
    pub fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }

    // ========== Audit Logging Methods ==========

    /// Audit: Auto-unseal attempt initiated
    async fn audit_unseal_initiated(&self, provider_metadata: &crate::ProviderMetadata) {
        if let Some(audit_logger) = &self.audit_logger {
            let mut metadata = HashMap::new();
            metadata.insert(
                "correlation_id".to_string(),
                self.correlation_id.to_string(),
            );
            metadata.insert(
                "provider_type".to_string(),
                provider_metadata.provider_type.clone(),
            );
            metadata.insert("key_id".to_string(), provider_metadata.key_id.clone());

            if let Some(region) = &provider_metadata.region {
                metadata.insert("region".to_string(), region.clone());
            }
            if let Some(endpoint) = &provider_metadata.endpoint {
                metadata.insert("endpoint".to_string(), endpoint.clone());
            }

            metadata.insert(
                "max_retries".to_string(),
                self.fallback_config.max_retries.to_string(),
            );
            metadata.insert(
                "fallback_enabled".to_string(),
                self.fallback_config.fallback_to_manual.to_string(),
            );

            let log = AuditLog {
                id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "auto_unseal.initiated".to_string(),
                actor: Some("system".to_string()),
                resource_type: "seal".to_string(),
                resource_id: "master_key".to_string(),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata,
            };

            if let Err(e) = audit_logger.log(log).await {
                error!(error = %e, "Failed to write auto-unseal initiated audit log");
            }
        }
    }

    /// Audit: Auto-unseal succeeded
    async fn audit_unseal_success(&self, provider_metadata: &crate::ProviderMetadata) {
        if let Some(audit_logger) = &self.audit_logger {
            let mut metadata = HashMap::new();
            metadata.insert(
                "correlation_id".to_string(),
                self.correlation_id.to_string(),
            );
            metadata.insert(
                "provider_type".to_string(),
                provider_metadata.provider_type.clone(),
            );
            metadata.insert("key_id".to_string(), provider_metadata.key_id.clone());

            let log = AuditLog {
                id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "auto_unseal.success".to_string(),
                actor: Some("system".to_string()),
                resource_type: "seal".to_string(),
                resource_id: "master_key".to_string(),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata,
            };

            if let Err(e) = audit_logger.log(log).await {
                error!(error = %e, "Failed to write auto-unseal success audit log");
            }
        }
    }

    /// Audit: Auto-unseal failed (no fallback)
    async fn audit_unseal_failure(
        &self,
        provider_metadata: &crate::ProviderMetadata,
        error: &SecretonError,
    ) {
        if let Some(audit_logger) = &self.audit_logger {
            let mut metadata = HashMap::new();
            metadata.insert(
                "correlation_id".to_string(),
                self.correlation_id.to_string(),
            );
            metadata.insert(
                "provider_type".to_string(),
                provider_metadata.provider_type.clone(),
            );
            metadata.insert("key_id".to_string(), provider_metadata.key_id.clone());
            metadata.insert("error".to_string(), error.to_string());
            metadata.insert("fallback_enabled".to_string(), "false".to_string());

            let log = AuditLog {
                id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "auto_unseal.failed".to_string(),
                actor: Some("system".to_string()),
                resource_type: "seal".to_string(),
                resource_id: "master_key".to_string(),
                status: AuditStatus::Failure,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata,
            };

            if let Err(e) = audit_logger.log(log).await {
                error!(error = %e, "Failed to write auto-unseal failure audit log");
            }
        }
    }

    /// Audit: Fallback to manual unseal
    async fn audit_fallback_to_manual(
        &self,
        provider_metadata: &crate::ProviderMetadata,
        error: &SecretonError,
    ) {
        if let Some(audit_logger) = &self.audit_logger {
            let mut metadata = HashMap::new();
            metadata.insert(
                "correlation_id".to_string(),
                self.correlation_id.to_string(),
            );
            metadata.insert(
                "provider_type".to_string(),
                provider_metadata.provider_type.clone(),
            );
            metadata.insert("key_id".to_string(), provider_metadata.key_id.clone());
            metadata.insert("error".to_string(), error.to_string());
            metadata.insert("fallback_enabled".to_string(), "true".to_string());

            let log = AuditLog {
                id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "auto_unseal.fallback_to_manual".to_string(),
                actor: Some("system".to_string()),
                resource_type: "seal".to_string(),
                resource_id: "master_key".to_string(),
                status: AuditStatus::Failure,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata,
            };

            if let Err(e) = audit_logger.log(log).await {
                error!(error = %e, "Failed to write fallback to manual audit log");
            }
        }
    }

    /// Audit: Retry attempt failed
    async fn audit_retry_failure(
        &self,
        provider_metadata: &crate::ProviderMetadata,
        attempt: u32,
        error: &SecretonError,
    ) {
        if let Some(audit_logger) = &self.audit_logger {
            let mut metadata = HashMap::new();
            metadata.insert(
                "correlation_id".to_string(),
                self.correlation_id.to_string(),
            );
            metadata.insert(
                "provider_type".to_string(),
                provider_metadata.provider_type.clone(),
            );
            metadata.insert("key_id".to_string(), provider_metadata.key_id.clone());
            metadata.insert("attempt".to_string(), attempt.to_string());
            metadata.insert(
                "max_retries".to_string(),
                self.fallback_config.max_retries.to_string(),
            );
            metadata.insert("error".to_string(), error.to_string());

            let log = AuditLog {
                id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "auto_unseal.retry_failed".to_string(),
                actor: Some("system".to_string()),
                resource_type: "seal".to_string(),
                resource_id: "master_key".to_string(),
                status: AuditStatus::Failure,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata,
            };

            if let Err(e) = audit_logger.log(log).await {
                error!(error = %e, "Failed to write retry failure audit log");
            }
        }
    }

    /// Audit: Retry succeeded
    async fn audit_retry_success(&self, provider_metadata: &crate::ProviderMetadata, attempt: u32) {
        if let Some(audit_logger) = &self.audit_logger {
            let mut metadata = HashMap::new();
            metadata.insert(
                "correlation_id".to_string(),
                self.correlation_id.to_string(),
            );
            metadata.insert(
                "provider_type".to_string(),
                provider_metadata.provider_type.clone(),
            );
            metadata.insert("key_id".to_string(), provider_metadata.key_id.clone());
            metadata.insert("attempt".to_string(), attempt.to_string());

            let log = AuditLog {
                id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "auto_unseal.retry_success".to_string(),
                actor: Some("system".to_string()),
                resource_type: "seal".to_string(),
                resource_id: "master_key".to_string(),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata,
            };

            if let Err(e) = audit_logger.log(log).await {
                error!(error = %e, "Failed to write retry success audit log");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AutoUnsealProvider, ProviderMetadata};
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Mock provider that fails a specified number of times before succeeding
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
            "mock"
        }

        async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretonError> {
            Ok(plaintext.to_vec())
        }

        async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError> {
            let count = self.fail_count.fetch_add(1, Ordering::SeqCst);

            if count < self.max_failures {
                Err(SecretonError::Core(
                    secreton_core::error::CoreError::internal(format!(
                        "Mock failure {}/{}",
                        count + 1,
                        self.max_failures
                    )),
                ))
            } else {
                Ok(ciphertext.to_vec())
            }
        }

        async fn health_check(&self) -> Result<(), SecretonError> {
            Ok(())
        }

        fn metadata(&self) -> ProviderMetadata {
            ProviderMetadata::new("mock".to_string(), "test-key".to_string())
        }
    }

    #[tokio::test]
    async fn test_unseal_success_first_attempt() {
        let provider = Box::new(MockProvider::new(0));
        let config = FallbackConfig::default();
        let manager = AutoUnsealManager::new(provider, config);

        let encrypted = b"test-encrypted-key";
        let (master_key, result) = manager
            .unseal_with_fallback(encrypted)
            .await
            .expect("Unseal should succeed");

        assert_eq!(master_key, encrypted);
        assert_eq!(result, UnsealResult::Success);
    }

    #[tokio::test]
    async fn test_unseal_success_after_retries() {
        let provider = Box::new(MockProvider::new(2)); // Fail 2 times, succeed on 3rd
        let config = FallbackConfig {
            fallback_to_manual: true,
            max_retries: 5,
            initial_retry_delay_secs: 0, // No delay for testing
            max_retry_delay_secs: 0,
        };
        let manager = AutoUnsealManager::new(provider, config);

        let encrypted = b"test-encrypted-key";
        let (master_key, result) = manager
            .unseal_with_fallback(encrypted)
            .await
            .expect("Unseal should succeed after retries");

        assert_eq!(master_key, encrypted);
        assert_eq!(result, UnsealResult::Success);
    }

    #[tokio::test]
    async fn test_fallback_to_manual_when_enabled() {
        let provider = Box::new(MockProvider::new(10)); // Fail more than max_retries
        let config = FallbackConfig {
            fallback_to_manual: true,
            max_retries: 3,
            initial_retry_delay_secs: 0,
            max_retry_delay_secs: 0,
        };
        let manager = AutoUnsealManager::new(provider, config);

        let encrypted = b"test-encrypted-key";
        let (master_key, result) = manager
            .unseal_with_fallback(encrypted)
            .await
            .expect("Should fall back to manual");

        assert!(master_key.is_empty());
        assert_eq!(result, UnsealResult::FallbackToManual);
    }

    #[tokio::test]
    async fn test_no_fallback_when_disabled() {
        let provider = Box::new(MockProvider::new(10)); // Fail more than max_retries
        let config = FallbackConfig {
            fallback_to_manual: false,
            max_retries: 3,
            initial_retry_delay_secs: 0,
            max_retry_delay_secs: 0,
        };
        let manager = AutoUnsealManager::new(provider, config);

        let encrypted = b"test-encrypted-key";
        let result = manager.unseal_with_fallback(encrypted).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let provider = Box::new(MockProvider::new(3));
        let config = FallbackConfig {
            fallback_to_manual: true,
            max_retries: 5,
            initial_retry_delay_secs: 1,
            max_retry_delay_secs: 8,
        };
        let manager = AutoUnsealManager::new(provider, config);

        let encrypted = b"test-encrypted-key";
        let start = std::time::Instant::now();

        let (_master_key, result) = manager
            .unseal_with_fallback(encrypted)
            .await
            .expect("Should succeed after retries");

        let elapsed = start.elapsed();

        // Should have delays: 1s + 2s + 4s = 7s (approximately)
        // We allow some tolerance for test execution time
        assert!(elapsed.as_secs() >= 6);
        assert_eq!(result, UnsealResult::Success);
    }
}
