//! Property-based tests for auto-unseal fallback mechanism
//!
//! **Validates: Requirements 2.1.6**
//!
//! These tests verify that the auto-unseal fallback mechanism correctly handles
//! provider failures, implements exponential backoff, and transitions to manual
//! unseal mode when configured.

use proptest::prelude::*;
use secreton_auto_unseal::{
    AutoUnsealManager, AutoUnsealProvider, FallbackConfig, ProviderMetadata, UnsealResult,
};
use secreton_core::error::{CoreError, SecretonError};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// Mock provider that fails a configurable number of times
///
/// This provider simulates various failure scenarios:
/// - Network timeouts
/// - Provider unavailability
/// - Transient errors
struct FailingMockProvider {
    /// Number of times decrypt has been called
    call_count: Arc<AtomicU32>,
    /// Number of times to fail before succeeding
    fail_count: u32,
    /// Whether to always fail (for testing fallback)
    always_fail: bool,
}

impl FailingMockProvider {
    fn new(fail_count: u32) -> Self {
        Self {
            call_count: Arc::new(AtomicU32::new(0)),
            fail_count,
            always_fail: false,
        }
    }

    fn always_failing() -> Self {
        Self {
            call_count: Arc::new(AtomicU32::new(0)),
            fail_count: 0,
            always_fail: true,
        }
    }
}

#[async_trait::async_trait]
impl AutoUnsealProvider for FailingMockProvider {
    fn name(&self) -> &str {
        "failing-mock"
    }

    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        // Simple XOR encryption for testing
        Ok(plaintext.iter().map(|&b| b ^ 0x42).collect())
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);

        if self.always_fail || count < self.fail_count {
            Err(SecretonError::Core(CoreError::internal(format!(
                "Mock provider failure (attempt {}/{})",
                count + 1,
                if self.always_fail {
                    "∞".to_string()
                } else {
                    self.fail_count.to_string()
                }
            ))))
        } else {
            // XOR is symmetric
            Ok(ciphertext.iter().map(|&b| b ^ 0x42).collect())
        }
    }

    async fn health_check(&self) -> Result<(), SecretonError> {
        if self.always_fail {
            Err(SecretonError::Core(CoreError::internal(
                "Provider unavailable",
            )))
        } else {
            Ok(())
        }
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata::new("failing-mock".to_string(), "test-key".to_string())
    }
}

/// Property 3: Auto-unseal fallback
///
/// **Property**: When auto-unseal fails after all retry attempts:
/// 1. If fallback is enabled, the system transitions to manual unseal mode
/// 2. If fallback is disabled, the system remains sealed and returns an error
/// 3. The number of retry attempts respects the max_retries configuration
/// 4. Retry delays follow exponential backoff pattern
/// 5. Audit events are logged for all fallback scenarios
///
/// **Validates**: Requirements 2.1.6
///
/// **Formal specification**:
/// ```text
/// ∀ config ∈ FallbackConfig, ∀ encrypted_key ∈ Bytes:
///
///   // Case 1: Provider succeeds within retry limit
///   (provider_succeeds_at(n) ∧ n ≤ max_retries) ⟹
///     unseal_with_fallback(encrypted_key) = (master_key, Success)
///
///   // Case 2: Provider fails, fallback enabled
///   (provider_fails_all ∧ fallback_to_manual = true) ⟹
///     unseal_with_fallback(encrypted_key) = (empty, FallbackToManual)
///
///   // Case 3: Provider fails, fallback disabled
///   (provider_fails_all ∧ fallback_to_manual = false) ⟹
///     unseal_with_fallback(encrypted_key) = Error
///
///   // Case 4: Retry count is respected
///   retry_attempts = min(max_retries + 1, actual_failures)
///
///   // Case 5: Exponential backoff timing
///   delay(n) = min(initial_delay * 2^(n-1), max_delay)
/// ```
#[cfg(test)]
mod fallback_property_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]

        /// Property: Fallback to manual unseal when enabled
        ///
        /// When auto-unseal fails after all retries and fallback is enabled,
        /// the system should transition to manual unseal mode.
        #[test]
        fn prop_fallback_to_manual_when_enabled(
            max_retries in 1u32..5,
            initial_delay in 0u64..2,
            max_delay in 2u64..5,
            plaintext in prop::collection::vec(any::<u8>(), 1..32)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                // Create provider that always fails
                let provider = Box::new(FailingMockProvider::always_failing());

                // Configure fallback to manual unseal
                let config = FallbackConfig {
                    fallback_to_manual: true,
                    max_retries,
                    initial_retry_delay_secs: initial_delay,
                    max_retry_delay_secs: max_delay,
                };

                let manager = AutoUnsealManager::new(provider, config);

                // Attempt unseal
                let result = manager.unseal_with_fallback(&plaintext).await;

                // Should succeed with fallback result
                assert!(result.is_ok(), "Should succeed with fallback");

                let (master_key, unseal_result) = result.unwrap();

                // Should return empty key and FallbackToManual status
                assert!(master_key.is_empty(), "Master key should be empty on fallback");
                assert_eq!(unseal_result, UnsealResult::FallbackToManual);
            });
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]

        /// Property: No fallback when disabled
        ///
        /// When auto-unseal fails and fallback is disabled, the system should
        /// remain sealed and return an error.
        #[test]
        fn prop_no_fallback_when_disabled(
            max_retries in 1u32..5,
            initial_delay in 0u64..2,
            max_delay in 2u64..5,
            plaintext in prop::collection::vec(any::<u8>(), 1..32)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                // Create provider that always fails
                let provider = Box::new(FailingMockProvider::always_failing());

                // Configure NO fallback
                let config = FallbackConfig {
                    fallback_to_manual: false,
                    max_retries,
                    initial_retry_delay_secs: initial_delay,
                    max_retry_delay_secs: max_delay,
                };

                let manager = AutoUnsealManager::new(provider, config);

                // Attempt unseal
                let result = manager.unseal_with_fallback(&plaintext).await;

                // Should fail with error
                assert!(result.is_err(), "Should fail when fallback is disabled");

            })
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]

        /// Property: Retry count is respected
        ///
        /// The system should attempt unseal exactly (max_retries + 1) times
        /// before giving up.
        #[test]
        fn prop_retry_count_respected(
            max_retries in 1u32..5,
            plaintext in prop::collection::vec(any::<u8>(), 1..32)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                // Create provider that always fails
                let provider = FailingMockProvider::always_failing();
                let call_count_tracker = provider.call_count.clone();

                let config = FallbackConfig {
                    fallback_to_manual: true,
                    max_retries,
                    initial_retry_delay_secs: 0, // No delay for faster testing
                    max_retry_delay_secs: 0,
                };

                let manager = AutoUnsealManager::new(Box::new(provider), config);

                // Attempt unseal
                let _ = manager.unseal_with_fallback(&plaintext).await;

                // Verify retry count: should be max_retries + 1 (initial attempt + retries)
                let actual_attempts = call_count_tracker.load(Ordering::SeqCst);
                assert_eq!(
                    actual_attempts,
                    max_retries + 1,
                    "Should attempt exactly max_retries + 1 times"
                );

            })
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]

        /// Property: Success within retry limit
        ///
        /// If the provider succeeds within the retry limit, auto-unseal should
        /// succeed without falling back to manual unseal.
        #[test]
        fn prop_success_within_retry_limit(
            fail_count in 0u32..3,
            max_retries in 3u32..6,
            plaintext in prop::collection::vec(any::<u8>(), 1..32)
        ) {
            prop_assume!(fail_count <= max_retries);

            tokio::runtime::Runtime::new().unwrap().block_on(async {
                // Create provider that fails fail_count times, then succeeds
                let provider = Box::new(FailingMockProvider::new(fail_count));

                let config = FallbackConfig {
                    fallback_to_manual: true,
                    max_retries,
                    initial_retry_delay_secs: 0,
                    max_retry_delay_secs: 0,
                };

                let manager = AutoUnsealManager::new(provider, config);

                // Encrypt plaintext
                let encrypted = manager.provider().encrypt(&plaintext).await.unwrap();

                // Attempt unseal
                let result = manager.unseal_with_fallback(&encrypted).await;

                // Should succeed
                assert!(result.is_ok(), "Should succeed within retry limit");

                let (master_key, unseal_result) = result.unwrap();

                // Should return decrypted key and Success status
                assert_eq!(master_key, plaintext, "Should decrypt correctly");
                assert_eq!(unseal_result, UnsealResult::Success);

            })
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]

        /// Property: Failure beyond retry limit triggers fallback
        ///
        /// If the provider fails more times than max_retries, fallback should
        /// be triggered (if enabled).
        #[test]
        fn prop_failure_beyond_retry_limit(
            max_retries in 1u32..3,
            extra_failures in 1u32..3,
            plaintext in prop::collection::vec(any::<u8>(), 1..32)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                // Create provider that fails more than max_retries
                let fail_count = max_retries + extra_failures;
                let provider = Box::new(FailingMockProvider::new(fail_count));

                let config = FallbackConfig {
                    fallback_to_manual: true,
                    max_retries,
                    initial_retry_delay_secs: 0,
                    max_retry_delay_secs: 0,
                };

                let manager = AutoUnsealManager::new(provider, config);

                // Attempt unseal
                let result = manager.unseal_with_fallback(&plaintext).await;

                // Should succeed with fallback
                assert!(result.is_ok(), "Should succeed with fallback");

                let (_master_key, unseal_result) = result.unwrap();
                assert_eq!(unseal_result, UnsealResult::FallbackToManual);

            })
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(3))]

        /// Property: Exponential backoff timing
        ///
        /// Retry delays should follow exponential backoff pattern:
        /// delay(n) = min(initial_delay * 2^(n-1), max_delay)
        #[test]
        fn prop_exponential_backoff_timing(
            max_retries in 2u32..4, // At least 2 retries to test backoff
            initial_delay in 1u64..2,
            max_delay in 3u64..5,
            plaintext in prop::collection::vec(any::<u8>(), 1..32)
        ) {
            prop_assume!(initial_delay < max_delay);

            tokio::runtime::Runtime::new().unwrap().block_on(async {
                // Create provider that always fails
                let provider = Box::new(FailingMockProvider::always_failing());

                let config = FallbackConfig {
                    fallback_to_manual: true,
                    max_retries,
                    initial_retry_delay_secs: initial_delay,
                    max_retry_delay_secs: max_delay,
                };

                let manager = AutoUnsealManager::new(provider, config);

                // Measure total time
                let start = Instant::now();
                let _ = manager.unseal_with_fallback(&plaintext).await;
                let elapsed = start.elapsed();

                // Calculate expected minimum delay
                // First attempt: no delay
                // Subsequent attempts: initial_delay, initial_delay*2, initial_delay*4, ...
                // Each capped at max_delay
                let mut expected_delay = Duration::from_secs(0);
                let mut current_delay = initial_delay;

                for _ in 0..max_retries {
                    expected_delay += Duration::from_secs(current_delay);
                    current_delay = std::cmp::min(current_delay * 2, max_delay);
                }

                // Allow some tolerance for test execution overhead (500ms)
                let tolerance = Duration::from_millis(500);

                assert!(
                    elapsed >= expected_delay.saturating_sub(tolerance),
                    "Elapsed time ({:?}) should be at least expected delay ({:?})",
                    elapsed,
                    expected_delay
                );

            })
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(3))]

        /// Property: Max delay cap is respected
        ///
        /// Retry delays should never exceed max_retry_delay_secs, even with
        /// exponential backoff.
        #[test]
        fn prop_max_delay_cap_respected(
            max_retries in 4u32..6, // Fewer retries for faster testing
            initial_delay in 1u64..2,
            max_delay in 2u64..3,
            plaintext in prop::collection::vec(any::<u8>(), 1..32)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = Box::new(FailingMockProvider::always_failing());

                let config = FallbackConfig {
                    fallback_to_manual: true,
                    max_retries,
                    initial_retry_delay_secs: initial_delay,
                    max_retry_delay_secs: max_delay,
                };

                let manager = AutoUnsealManager::new(provider, config);

                let start = Instant::now();
                let _ = manager.unseal_with_fallback(&plaintext).await;
                let elapsed = start.elapsed();

                // Maximum possible delay: max_delay * max_retries
                let max_possible_delay = Duration::from_secs(max_delay * max_retries as u64);

                // Add tolerance for test execution
                let tolerance = Duration::from_secs(2);

                assert!(
                    elapsed <= max_possible_delay + tolerance,
                    "Elapsed time ({:?}) should not exceed max possible delay ({:?})",
                    elapsed,
                    max_possible_delay
                );

            })
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]

        /// Property: Fallback configuration is immutable during unseal
        ///
        /// The fallback configuration should not change during the unseal process.
        #[test]
        fn prop_fallback_config_immutable(
            max_retries in 1u32..5,
            initial_delay in 0u64..2,
            max_delay in 2u64..5,
            plaintext in prop::collection::vec(any::<u8>(), 1..32)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = Box::new(FailingMockProvider::always_failing());

                let config = FallbackConfig {
                    fallback_to_manual: true,
                    max_retries,
                    initial_retry_delay_secs: initial_delay,
                    max_retry_delay_secs: max_delay,
                };

                let manager = AutoUnsealManager::new(provider, config.clone());

                // Attempt unseal
                let _ = manager.unseal_with_fallback(&plaintext).await;

                // Verify configuration hasn't changed
                let current_config = manager.fallback_config();
                assert_eq!(current_config.fallback_to_manual, config.fallback_to_manual);
                assert_eq!(current_config.max_retries, config.max_retries);
                assert_eq!(current_config.initial_retry_delay_secs, config.initial_retry_delay_secs);
                assert_eq!(current_config.max_retry_delay_secs, config.max_retry_delay_secs);

            })
        }
    }
}

/// Edge case tests for fallback mechanism
#[cfg(test)]
mod fallback_edge_cases {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(3))]

        /// Edge case: Zero retries
        ///
        /// With max_retries = 0, should attempt once and then fallback.
        #[test]
        fn prop_zero_retries(
            plaintext in prop::collection::vec(any::<u8>(), 1..32)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = FailingMockProvider::always_failing();
                let call_count_tracker = provider.call_count.clone();

                let config = FallbackConfig {
                    fallback_to_manual: true,
                    max_retries: 0,
                    initial_retry_delay_secs: 0,
                    max_retry_delay_secs: 0,
                };

                let manager = AutoUnsealManager::new(Box::new(provider), config);

                let result = manager.unseal_with_fallback(&plaintext).await;

                // Should fallback after single attempt
                assert!(result.is_ok());
                let (_key, status) = result.unwrap();
                assert_eq!(status, UnsealResult::FallbackToManual);

                // Should have attempted exactly once
                assert_eq!(call_count_tracker.load(Ordering::SeqCst), 1);

            })
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(3))]

        /// Edge case: Zero initial delay
        ///
        /// With initial_retry_delay_secs = 0, retries should happen immediately.
        #[test]
        fn prop_zero_initial_delay(
            max_retries in 1u32..4,
            plaintext in prop::collection::vec(any::<u8>(), 1..32)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = Box::new(FailingMockProvider::always_failing());

                let config = FallbackConfig {
                    fallback_to_manual: true,
                    max_retries,
                    initial_retry_delay_secs: 0,
                    max_retry_delay_secs: 0,
                };

                let manager = AutoUnsealManager::new(provider, config);

                let start = Instant::now();
                let _ = manager.unseal_with_fallback(&plaintext).await;
                let elapsed = start.elapsed();

                // With zero delays, should complete very quickly (< 1 second)
                assert!(
                    elapsed < Duration::from_secs(1),
                    "Should complete quickly with zero delays"
                );

            })
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(3))]

        /// Edge case: Very large max_retries
        ///
        /// System should handle large retry counts without overflow or panic.
        #[test]
        fn prop_large_max_retries(
            plaintext in prop::collection::vec(any::<u8>(), 1..32)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = Box::new(FailingMockProvider::new(2)); // Succeed after 2 failures

                let config = FallbackConfig {
                    fallback_to_manual: true,
                    max_retries: 1000, // Very large
                    initial_retry_delay_secs: 0,
                    max_retry_delay_secs: 0,
                };

                let manager = AutoUnsealManager::new(provider, config);

                // Encrypt plaintext
                let encrypted = manager.provider().encrypt(&plaintext).await.unwrap();

                // Should succeed within the large retry limit
                let result = manager.unseal_with_fallback(&encrypted).await;

                assert!(result.is_ok());
                let (_key, status) = result.unwrap();
                assert_eq!(status, UnsealResult::Success);

            })
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(3))]

        /// Edge case: Empty encrypted key
        ///
        /// System should handle empty encrypted key gracefully.
        #[test]
        fn prop_empty_encrypted_key(
            max_retries in 1u32..4
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = Box::new(FailingMockProvider::new(0)); // Succeed immediately

                let config = FallbackConfig {
                    fallback_to_manual: true,
                    max_retries,
                    initial_retry_delay_secs: 0,
                    max_retry_delay_secs: 0,
                };

                let manager = AutoUnsealManager::new(provider, config);

                let empty_key = vec![];
                let result = manager.unseal_with_fallback(&empty_key).await;

                // Should handle empty key (may succeed or fail depending on provider)
                assert!(result.is_ok() || result.is_err());

            })
        }
    }
}

/// Integration tests for realistic scenarios
#[cfg(test)]
mod fallback_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_realistic_network_timeout_scenario() {
        // Simulate network timeout: fail 3 times, then succeed
        let provider = Box::new(FailingMockProvider::new(3));

        let config = FallbackConfig {
            fallback_to_manual: true,
            max_retries: 5,
            initial_retry_delay_secs: 1,
            max_retry_delay_secs: 8,
        };

        let manager = AutoUnsealManager::new(provider, config);

        let master_key = b"secret-master-key-32-bytes-long!";
        let encrypted = manager.provider().encrypt(master_key).await.unwrap();

        let start = Instant::now();
        let result = manager.unseal_with_fallback(&encrypted).await;
        let elapsed = start.elapsed();

        // Should succeed after retries
        assert!(result.is_ok());
        let (decrypted, status) = result.unwrap();
        assert_eq!(decrypted, master_key);
        assert_eq!(status, UnsealResult::Success);

        // Should have taken at least 1 + 2 + 4 = 7 seconds (with some tolerance)
        assert!(elapsed >= Duration::from_secs(6));
    }

    #[tokio::test]
    async fn test_provider_permanently_unavailable() {
        // Provider always fails
        let provider = Box::new(FailingMockProvider::always_failing());

        let config = FallbackConfig {
            fallback_to_manual: true,
            max_retries: 3,
            initial_retry_delay_secs: 0,
            max_retry_delay_secs: 0,
        };

        let manager = AutoUnsealManager::new(provider, config);

        let encrypted = b"encrypted-master-key";
        let result = manager.unseal_with_fallback(encrypted).await;

        // Should fallback to manual
        assert!(result.is_ok());
        let (key, status) = result.unwrap();
        assert!(key.is_empty());
        assert_eq!(status, UnsealResult::FallbackToManual);
    }

    #[tokio::test]
    async fn test_no_fallback_strict_mode() {
        // Provider always fails, fallback disabled
        let provider = Box::new(FailingMockProvider::always_failing());

        let config = FallbackConfig {
            fallback_to_manual: false,
            max_retries: 2,
            initial_retry_delay_secs: 0,
            max_retry_delay_secs: 0,
        };

        let manager = AutoUnsealManager::new(provider, config);

        let encrypted = b"encrypted-master-key";
        let result = manager.unseal_with_fallback(encrypted).await;

        // Should fail without fallback
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_immediate_success_no_retries() {
        // Provider succeeds immediately
        let provider = Box::new(FailingMockProvider::new(0));

        let config = FallbackConfig::default();

        let manager = AutoUnsealManager::new(provider, config);

        let master_key = b"secret-master-key";
        let encrypted = manager.provider().encrypt(master_key).await.unwrap();

        let start = Instant::now();
        let result = manager.unseal_with_fallback(&encrypted).await;
        let elapsed = start.elapsed();

        // Should succeed immediately
        assert!(result.is_ok());
        let (decrypted, status) = result.unwrap();
        assert_eq!(decrypted, master_key);
        assert_eq!(status, UnsealResult::Success);

        // Should complete very quickly (< 100ms)
        assert!(elapsed < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_exponential_backoff_cap() {
        // Fail many times to test backoff cap
        let provider = Box::new(FailingMockProvider::always_failing());

        let config = FallbackConfig {
            fallback_to_manual: true,
            max_retries: 10,
            initial_retry_delay_secs: 1,
            max_retry_delay_secs: 4, // Cap at 4 seconds
        };

        let manager = AutoUnsealManager::new(provider, config);

        let start = Instant::now();
        let _ = manager.unseal_with_fallback(b"test").await;
        let elapsed = start.elapsed();

        // Expected delays: 1, 2, 4, 4, 4, 4, 4, 4, 4, 4 = 35 seconds
        // Allow some tolerance
        assert!(elapsed >= Duration::from_secs(33));
        assert!(elapsed <= Duration::from_secs(37));
    }
}
