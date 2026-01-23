// **Feature: secreton-comprehensive-enhancement, Property 31: Rate Limiting Enforcement**
// **Validates: Requirements 13.4**
//
// Property: For any client exceeding rate limit, subsequent requests SHALL receive HTTP 429 with Retry-After header.

use proptest::prelude::*;
use secreton_core::services::rate_limit::{
    RateLimitConfig, RateLimitError, RateLimitStrategy, RateLimiter,
};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 31: Rate Limiting Enforcement
    ///
    /// This test verifies that when a client exceeds the configured rate limit,
    /// subsequent requests are rejected with a rate limit error.
    ///
    /// Test strategy:
    /// 1. Create a rate limiter with a small capacity
    /// 2. Make requests up to the limit
    /// 3. Verify that requests within limit succeed
    /// 4. Verify that requests exceeding limit fail with RateLimitError
    #[test]
    fn property_rate_limit_enforcement(
        capacity in 1u32..20,
        refill_rate in 1u32..10,
        num_requests in 1usize..50,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let config = RateLimitConfig {
                strategy: RateLimitStrategy::TokenBucket {
                    capacity,
                    refill_rate,
                },
                enabled: true,
            };

            let limiter = RateLimiter::new(config);
            let client_id = "test_client";

            let mut successful_requests = 0;
            let mut rate_limited_requests = 0;

            // Make requests
            for _ in 0..num_requests {
                match limiter.check(client_id).await {
                    Ok(true) => successful_requests += 1,
                    Err(RateLimitError::LimitExceeded(_)) => rate_limited_requests += 1,
                    _ => {}
                }
            }

            // Verify that we allowed up to capacity requests
            prop_assert!(successful_requests <= capacity as usize,
                "Allowed {} requests but capacity is {}", successful_requests, capacity);

            // If we made more requests than capacity, some should be rate limited
            if num_requests > capacity as usize {
                prop_assert!(rate_limited_requests > 0,
                    "Expected rate limiting after {} requests with capacity {}",
                    num_requests, capacity);
            }

            // Total requests should match
            prop_assert_eq!(successful_requests + rate_limited_requests, num_requests,
                "Request count mismatch");

            Ok(())
        })?;
    }

    /// Property 31.1: Rate Limit Per-Client Isolation
    ///
    /// This test verifies that rate limits are enforced per-client,
    /// and one client's usage doesn't affect another client's quota.
    #[test]
    fn property_rate_limit_per_client_isolation(
        capacity in 2u32..10,
        num_clients in 2usize..5,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let config = RateLimitConfig {
                strategy: RateLimitStrategy::TokenBucket {
                    capacity,
                    refill_rate: 1,
                },
                enabled: true,
            };

            let limiter = RateLimiter::new(config);

            // Each client should be able to make 'capacity' requests
            for client_idx in 0..num_clients {
                let client_id = format!("client_{}", client_idx);

                // Each client should be allowed up to capacity requests
                for req_idx in 0..capacity {
                    let result = limiter.check(&client_id).await;
                    prop_assert!(result.is_ok(),
                        "Client {} request {} should succeed (capacity: {})",
                        client_idx, req_idx, capacity);
                }

                // Next request should be rate limited
                let result = limiter.check(&client_id).await;
                prop_assert!(matches!(result, Err(RateLimitError::LimitExceeded(_))),
                    "Client {} should be rate limited after {} requests",
                    client_idx, capacity);
            }

            Ok(())
        })?;
    }

    /// Property 31.2: Sliding Window Rate Limiting
    ///
    /// This test verifies that sliding window rate limiting correctly
    /// enforces the maximum requests within the time window.
    #[test]
    fn property_sliding_window_rate_limiting(
        max_requests in 2u32..20,
        window_seconds in 1u64..10,
        num_requests in 1usize..50,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let config = RateLimitConfig {
                strategy: RateLimitStrategy::SlidingWindow {
                    max_requests,
                    window_seconds,
                },
                enabled: true,
            };

            let limiter = RateLimiter::new(config);
            let client_id = "test_client";

            let mut successful_requests = 0;
            let mut rate_limited_requests = 0;

            // Make requests
            for _ in 0..num_requests {
                match limiter.check(client_id).await {
                    Ok(true) => successful_requests += 1,
                    Err(RateLimitError::LimitExceeded(_)) => rate_limited_requests += 1,
                    _ => {}
                }
            }

            // Verify that we allowed up to max_requests
            prop_assert!(successful_requests <= max_requests as usize,
                "Allowed {} requests but max is {}", successful_requests, max_requests);

            // If we made more requests than max, some should be rate limited
            if num_requests > max_requests as usize {
                prop_assert!(rate_limited_requests > 0,
                    "Expected rate limiting after {} requests with max {}",
                    num_requests, max_requests);
            }

            Ok(())
        })?;
    }

    /// Property 31.3: Rate Limit Reset Functionality
    ///
    /// This test verifies that resetting a client's rate limit
    /// allows them to make requests again.
    #[test]
    fn property_rate_limit_reset(
        capacity in 2u32..10,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let config = RateLimitConfig {
                strategy: RateLimitStrategy::TokenBucket {
                    capacity,
                    refill_rate: 1,
                },
                enabled: true,
            };

            let limiter = RateLimiter::new(config);
            let client_id = "test_client";

            // Exhaust the rate limit
            for _ in 0..capacity {
                limiter.check(client_id).await.unwrap();
            }

            // Next request should be rate limited
            let result = limiter.check(client_id).await;
            prop_assert!(matches!(result, Err(RateLimitError::LimitExceeded(_))),
                "Should be rate limited after {} requests", capacity);

            // Reset the rate limit
            limiter.reset(client_id).await;

            // Should be able to make requests again
            let result = limiter.check(client_id).await;
            prop_assert!(result.is_ok(),
                "Should be able to make requests after reset");

            Ok(())
        })?;
    }

    /// Property 31.4: Rate Limit with Custom Cost
    ///
    /// This test verifies that rate limiting with custom cost
    /// correctly consumes the specified number of tokens.
    #[test]
    fn property_rate_limit_custom_cost(
        capacity in 10u32..50,
        cost in 1u32..10,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let config = RateLimitConfig {
                strategy: RateLimitStrategy::TokenBucket {
                    capacity,
                    refill_rate: 1,
                },
                enabled: true,
            };

            let limiter = RateLimiter::new(config);
            let client_id = "test_client";

            // Calculate how many requests we can make with this cost
            let expected_requests = capacity / cost;

            let mut successful_requests = 0;

            // Make requests with custom cost
            for _ in 0..=expected_requests {
                match limiter.check_with_cost(client_id, cost).await {
                    Ok(true) => successful_requests += 1,
                    Err(RateLimitError::LimitExceeded(_)) => break,
                    _ => {}
                }
            }

            // We should be able to make at least expected_requests
            prop_assert!(successful_requests >= expected_requests as usize,
                "Expected at least {} requests with cost {} and capacity {}, got {}",
                expected_requests, cost, capacity, successful_requests);

            // We shouldn't be able to make significantly more
            prop_assert!(successful_requests <= (expected_requests + 1) as usize,
                "Made too many requests: {} (expected ~{})",
                successful_requests, expected_requests);

            Ok(())
        })?;
    }

    /// Property 31.5: Disabled Rate Limiting
    ///
    /// This test verifies that when rate limiting is disabled,
    /// all requests are allowed regardless of count.
    #[test]
    fn property_disabled_rate_limiting(
        num_requests in 1usize..100,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let config = RateLimitConfig {
                strategy: RateLimitStrategy::TokenBucket {
                    capacity: 1,  // Very low capacity
                    refill_rate: 1,
                },
                enabled: false,  // But disabled
            };

            let limiter = RateLimiter::new(config);
            let client_id = "test_client";

            // All requests should succeed when disabled
            for _ in 0..num_requests {
                let result = limiter.check(client_id).await;
                prop_assert!(result.is_ok(),
                    "All requests should succeed when rate limiting is disabled");
            }

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_basic() {
        let config = RateLimitConfig {
            strategy: RateLimitStrategy::TokenBucket {
                capacity: 3,
                refill_rate: 1,
            },
            enabled: true,
        };

        let limiter = RateLimiter::new(config);

        // First 3 requests should succeed
        assert!(limiter.check("client1").await.is_ok());
        assert!(limiter.check("client1").await.is_ok());
        assert!(limiter.check("client1").await.is_ok());

        // 4th request should be rate limited
        assert!(matches!(
            limiter.check("client1").await,
            Err(RateLimitError::LimitExceeded(_))
        ));
    }

    #[tokio::test]
    async fn test_rate_limit_different_clients() {
        let config = RateLimitConfig {
            strategy: RateLimitStrategy::TokenBucket {
                capacity: 2,
                refill_rate: 1,
            },
            enabled: true,
        };

        let limiter = RateLimiter::new(config);

        // Client 1 exhausts limit
        assert!(limiter.check("client1").await.is_ok());
        assert!(limiter.check("client1").await.is_ok());
        assert!(matches!(
            limiter.check("client1").await,
            Err(RateLimitError::LimitExceeded(_))
        ));

        // Client 2 should still have quota
        assert!(limiter.check("client2").await.is_ok());
        assert!(limiter.check("client2").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limit_reset() {
        let config = RateLimitConfig {
            strategy: RateLimitStrategy::TokenBucket {
                capacity: 1,
                refill_rate: 1,
            },
            enabled: true,
        };

        let limiter = RateLimiter::new(config);

        // Exhaust limit
        assert!(limiter.check("client1").await.is_ok());
        assert!(matches!(
            limiter.check("client1").await,
            Err(RateLimitError::LimitExceeded(_))
        ));

        // Reset
        limiter.reset("client1").await;

        // Should work again
        assert!(limiter.check("client1").await.is_ok());
    }

    #[tokio::test]
    async fn test_sliding_window() {
        let config = RateLimitConfig {
            strategy: RateLimitStrategy::SlidingWindow {
                max_requests: 3,
                window_seconds: 60,
            },
            enabled: true,
        };

        let limiter = RateLimiter::new(config);

        // First 3 requests should succeed
        assert!(limiter.check("client1").await.is_ok());
        assert!(limiter.check("client1").await.is_ok());
        assert!(limiter.check("client1").await.is_ok());

        // 4th request should be rate limited
        assert!(matches!(
            limiter.check("client1").await,
            Err(RateLimitError::LimitExceeded(_))
        ));
    }

    #[tokio::test]
    async fn test_disabled_rate_limiting() {
        let config = RateLimitConfig {
            strategy: RateLimitStrategy::TokenBucket {
                capacity: 1,
                refill_rate: 1,
            },
            enabled: false,
        };

        let limiter = RateLimiter::new(config);

        // Should allow unlimited requests when disabled
        for _ in 0..100 {
            assert!(limiter.check("client1").await.is_ok());
        }
    }
}
