//! Middleware module for pre-operation checks
//!
//! This module provides middleware functionality to check engine status
//! before executing operations, ensuring the engine is initialized and unsealed.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Engine status information
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct EngineStatus {
    pub state: String,
    pub seal_type: String,
    pub initialized: bool,
    pub total_shares: usize,
    pub threshold: usize,
    pub progress: usize,
    pub version: String,
}

/// Cached engine status with timestamp
#[derive(Debug, Clone)]
struct CachedStatus {
    status: EngineStatus,
    cached_at: Instant,
}

/// Error types for engine readiness checks
#[derive(Debug, thiserror::Error)]
pub enum EngineNotReadyError {
    #[error("Engine not initialized. Run 'secreton seal init' first.")]
    NotInitialized,

    #[error(
        "Engine is sealed (progress: {progress}/{threshold}). Run 'secreton seal unseal' to unseal."
    )]
    Sealed { progress: usize, threshold: usize },

    #[error("Failed to connect to engine: {0}")]
    Unreachable(String),

    #[error("API error: {0}")]
    ApiError(String),
}

/// Middleware for checking engine seal status before operations
pub struct SealChecker {
    client: reqwest::Client,
    server_url: String,
    cache: Option<CachedStatus>,
    cache_ttl: Duration,
}

impl SealChecker {
    /// Create a new SealChecker
    ///
    /// # Arguments
    /// * `server_url` - The Secreton server URL
    /// * `cache_ttl_seconds` - Cache TTL in seconds (default: 5)
    pub fn new(server_url: String, cache_ttl_seconds: Option<u64>) -> Self {
        Self {
            client: reqwest::Client::new(),
            server_url,
            cache: None,
            cache_ttl: Duration::from_secs(cache_ttl_seconds.unwrap_or(5)),
        }
    }

    /// Get the current engine status
    ///
    /// This method will use cached status if available and not expired.
    pub async fn get_status(&mut self) -> Result<EngineStatus, EngineNotReadyError> {
        // Check cache first
        if let Some(cached) = &self.cache
            && cached.cached_at.elapsed() < self.cache_ttl
        {
            return Ok(cached.status.clone());
        }

        // Fetch fresh status
        let url = format!("{}/v1/sys/seal-status", self.server_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| EngineNotReadyError::Unreachable(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(EngineNotReadyError::ApiError(format!(
                "{} - {}",
                status, error_text
            )));
        }

        let status: EngineStatus = response
            .json()
            .await
            .map_err(|e| EngineNotReadyError::ApiError(e.to_string()))?;

        // Update cache
        self.cache = Some(CachedStatus {
            status: status.clone(),
            cached_at: Instant::now(),
        });

        Ok(status)
    }

    /// Check if the engine is ready for operations
    ///
    /// Returns Ok(EngineStatus) if the engine is initialized and unsealed.
    /// Returns Err(EngineNotReadyError) otherwise.
    pub async fn check_ready(&mut self) -> Result<EngineStatus, EngineNotReadyError> {
        let status = self.get_status().await?;

        if !status.initialized {
            return Err(EngineNotReadyError::NotInitialized);
        }

        if status.state != "unsealed" {
            return Err(EngineNotReadyError::Sealed {
                progress: status.progress,
                threshold: status.threshold,
            });
        }

        Ok(status)
    }

    /// Require the engine to be unsealed
    ///
    /// This is a convenience method that returns an error if the engine is not unsealed.
    pub async fn require_unsealed(&mut self) -> Result<(), EngineNotReadyError> {
        self.check_ready().await?;
        Ok(())
    }

    /// Require the engine to be initialized
    ///
    /// This checks only if the engine is initialized, not if it's unsealed.
    pub async fn require_initialized(&mut self) -> Result<(), EngineNotReadyError> {
        let status = self.get_status().await?;

        if !status.initialized {
            return Err(EngineNotReadyError::NotInitialized);
        }

        Ok(())
    }

    /// Clear the cached status
    ///
    /// This forces the next status check to fetch fresh data from the server.
    pub fn clear_cache(&mut self) {
        self.cache = None;
    }

    /// Check if the cache is valid
    pub fn is_cache_valid(&self) -> bool {
        if let Some(cached) = &self.cache {
            cached.cached_at.elapsed() < self.cache_ttl
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seal_checker_creation() {
        let checker = SealChecker::new("http://127.0.0.1:8200".to_string(), None);
        assert_eq!(checker.server_url, "http://127.0.0.1:8200");
        assert_eq!(checker.cache_ttl, Duration::from_secs(5));
        assert!(checker.cache.is_none());
    }

    #[test]
    fn test_seal_checker_custom_ttl() {
        let checker = SealChecker::new("http://127.0.0.1:8200".to_string(), Some(10));
        assert_eq!(checker.cache_ttl, Duration::from_secs(10));
    }

    #[test]
    fn test_cache_validity() {
        let mut checker = SealChecker::new("http://127.0.0.1:8200".to_string(), Some(5));
        assert!(!checker.is_cache_valid());

        // Simulate cached status
        checker.cache = Some(CachedStatus {
            status: EngineStatus {
                state: "unsealed".to_string(),
                seal_type: "shamir".to_string(),
                initialized: true,
                total_shares: 5,
                threshold: 3,
                progress: 0,
                version: "1.0.0".to_string(),
            },
            cached_at: Instant::now(),
        });

        assert!(checker.is_cache_valid());
    }

    #[test]
    fn test_clear_cache() {
        let mut checker = SealChecker::new("http://127.0.0.1:8200".to_string(), None);

        checker.cache = Some(CachedStatus {
            status: EngineStatus {
                state: "unsealed".to_string(),
                seal_type: "shamir".to_string(),
                initialized: true,
                total_shares: 5,
                threshold: 3,
                progress: 0,
                version: "1.0.0".to_string(),
            },
            cached_at: Instant::now(),
        });

        assert!(checker.cache.is_some());

        checker.clear_cache();
        assert!(checker.cache.is_none());
    }

    #[test]
    fn test_engine_not_ready_error_display() {
        let err = EngineNotReadyError::NotInitialized;
        assert!(err.to_string().contains("not initialized"));
        assert!(err.to_string().contains("secreton seal init"));

        let err = EngineNotReadyError::Sealed {
            progress: 1,
            threshold: 3,
        };
        assert!(err.to_string().contains("sealed"));
        assert!(err.to_string().contains("1/3"));
        assert!(err.to_string().contains("secreton seal unseal"));

        let err = EngineNotReadyError::Unreachable("connection refused".to_string());
        assert!(err.to_string().contains("Failed to connect"));
    }

    // Property-based tests
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // Generator for engine status
        fn arb_engine_status() -> impl Strategy<Value = EngineStatus> {
            (
                prop::sample::select(vec!["sealed", "unsealing", "unsealed"]),
                prop::sample::select(vec!["shamir", "auto"]),
                any::<bool>(),
                1usize..=10,
                1usize..=10,
                0usize..=10,
                prop::string::string_regex("[0-9]+\\.[0-9]+\\.[0-9]+").unwrap(),
            )
                .prop_map(
                    |(
                        state,
                        seal_type,
                        initialized,
                        total_shares,
                        threshold,
                        progress,
                        version,
                    )| {
                        EngineStatus {
                            state: state.to_string(),
                            seal_type: seal_type.to_string(),
                            initialized,
                            total_shares,
                            threshold: threshold.min(total_shares),
                            progress: progress.min(threshold.min(total_shares)),
                            version,
                        }
                    },
                )
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            /// **Feature: secreton-cli-workflow-integration, Property 9: Sealed Engine Operation Rejection**
            /// **Validates: Requirements 6.2, 9.1, 10.1**
            ///
            /// For any engine status where the engine is sealed (state != "unsealed"),
            /// the SealChecker should reject operations with a Sealed error.
            #[test]
            fn prop_sealed_engine_rejection(status in arb_engine_status()) {
                // Only test sealed engines (initialized but not unsealed)
                prop_assume!(status.initialized && status.state != "unsealed");

                let mut checker = SealChecker::new("http://127.0.0.1:8200".to_string(), None);

                // Simulate cached status (to avoid network calls)
                checker.cache = Some(CachedStatus {
                    status: status.clone(),
                    cached_at: Instant::now(),
                });

                // Attempt to check if engine is ready
                let result = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(checker.check_ready());

                // Should return Sealed error
                prop_assert!(result.is_err());
                if let Err(e) = result {
                    match e {
                        EngineNotReadyError::Sealed { progress, threshold } => {
                            prop_assert_eq!(progress, status.progress);
                            prop_assert_eq!(threshold, status.threshold);
                        }
                        _ => prop_assert!(false, "Expected Sealed error, got: {:?}", e),
                    }
                }
            }

            /// **Feature: secreton-cli-workflow-integration, Property 10: Uninitialized Engine Operation Rejection**
            /// **Validates: Requirements 6.1, 9.2**
            ///
            /// For any engine status where the engine is not initialized,
            /// the SealChecker should reject operations with a NotInitialized error.
            #[test]
            fn prop_uninitialized_engine_rejection(status in arb_engine_status()) {
                // Only test uninitialized engines
                prop_assume!(!status.initialized);

                let mut checker = SealChecker::new("http://127.0.0.1:8200".to_string(), None);

                // Simulate cached status (to avoid network calls)
                checker.cache = Some(CachedStatus {
                    status: status.clone(),
                    cached_at: Instant::now(),
                });

                // Attempt to check if engine is ready
                let result = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(checker.check_ready());

                // Should return NotInitialized error
                prop_assert!(result.is_err());
                if let Err(e) = result {
                    match e {
                        EngineNotReadyError::NotInitialized => {
                            // Expected error
                        }
                        _ => prop_assert!(false, "Expected NotInitialized error, got: {:?}", e),
                    }
                }
            }

            /// Additional property: Ready engine acceptance
            ///
            /// For any engine status where the engine is initialized and unsealed,
            /// the SealChecker should accept operations (return Ok).
            #[test]
            fn prop_ready_engine_acceptance(status in arb_engine_status()) {
                // Only test ready engines (initialized and unsealed)
                prop_assume!(status.initialized && status.state == "unsealed");

                let mut checker = SealChecker::new("http://127.0.0.1:8200".to_string(), None);

                // Simulate cached status (to avoid network calls)
                checker.cache = Some(CachedStatus {
                    status: status.clone(),
                    cached_at: Instant::now(),
                });

                // Attempt to check if engine is ready
                let result = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(checker.check_ready());

                // Should return Ok with the status
                prop_assert!(result.is_ok());
                if let Ok(returned_status) = result {
                    prop_assert_eq!(returned_status.state, status.state);
                    prop_assert_eq!(returned_status.initialized, status.initialized);
                }
            }

            /// Additional property: require_initialized only checks initialization
            ///
            /// For any engine status, require_initialized should only check if the engine
            /// is initialized, not if it's unsealed.
            #[test]
            fn prop_require_initialized_ignores_seal_state(status in arb_engine_status()) {
                let mut checker = SealChecker::new("http://127.0.0.1:8200".to_string(), None);

                // Simulate cached status (to avoid network calls)
                checker.cache = Some(CachedStatus {
                    status: status.clone(),
                    cached_at: Instant::now(),
                });

                // Attempt to require initialized
                let result = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(checker.require_initialized());

                if status.initialized {
                    // Should succeed regardless of seal state
                    prop_assert!(result.is_ok());
                 } else {
                    // Should fail with NotInitialized
                    prop_assert!(result.is_err());
                    if let Err(e) = result {
                        match e {
                            EngineNotReadyError::NotInitialized => {
                                // Expected
                            }
                            _ => prop_assert!(false, "Expected NotInitialized, got: {:?}", e),
                        }
                    }
                }
            }
        }
    }
}
