//! Property-Based Tests for Webhook Retry Backoff
//!
//! **Feature: secreton-comprehensive-enhancement, Property 15: Webhook Retry Exponential Backoff**
//! **Validates: Requirements 5.5**
//!
//! This test verifies that webhook retry intervals follow exponential backoff
//! (1s, 2s, 4s, 8s, 16s) up to max 60s.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    /// Retry configuration for testing
    #[derive(Debug, Clone)]
    struct RetryConfig {
        max_attempts: u32,
        initial_delay: u64,
        max_delay: u64,
        backoff_multiplier: f64,
    }

    impl Default for RetryConfig {
        fn default() -> Self {
            Self {
                max_attempts: 5,
                initial_delay: 1,
                max_delay: 60,
                backoff_multiplier: 2.0,
            }
        }
    }

    /// Calculate retry delays with exponential backoff
    fn calculate_retry_delays(config: &RetryConfig) -> Vec<u64> {
        let mut delays = Vec::new();
        let mut delay = config.initial_delay;

        for _ in 0..config.max_attempts {
            delays.push(delay);
            delay = ((delay as f64) * config.backoff_multiplier) as u64;
            delay = delay.min(config.max_delay);
        }

        delays
    }

    /// Property: Exponential backoff follows expected pattern
    ///
    /// For any valid retry configuration with exponential backoff (multiplier 2.0),
    /// the retry delays should follow the pattern: 1s, 2s, 4s, 8s, 16s, ...
    /// up to the maximum delay.
    #[test]
    fn property_exponential_backoff_pattern() {
        proptest!(|(
            initial_delay in 1u64..=10,
            max_delay in 30u64..=120,
            max_attempts in 3u32..=10,
        )| {
            let config = RetryConfig {
                max_attempts,
                initial_delay,
                max_delay,
                backoff_multiplier: 2.0,
            };

            let delays = calculate_retry_delays(&config);

            // Verify we have the correct number of delays
            prop_assert_eq!(delays.len(), max_attempts as usize);

            // Verify first delay is the initial delay
            prop_assert_eq!(delays[0], initial_delay);

            // Verify exponential growth (each delay is double the previous, or capped at max)
            for i in 1..delays.len() {
                let expected = (delays[i - 1] * 2).min(max_delay);
                prop_assert_eq!(
                    delays[i],
                    expected,
                    "Delay at index {} should be {} but was {}",
                    i,
                    expected,
                    delays[i]
                );
            }

            // Verify no delay exceeds max_delay
            for (i, &delay) in delays.iter().enumerate() {
                prop_assert!(
                    delay <= max_delay,
                    "Delay at index {} ({}) exceeds max_delay ({})",
                    i,
                    delay,
                    max_delay
                );
            }
        });
    }

    /// Property: Delays are monotonically increasing until max
    ///
    /// For any retry configuration, each delay should be greater than or equal
    /// to the previous delay (monotonically increasing) until the max delay is reached.
    #[test]
    fn property_delays_monotonically_increasing() {
        proptest!(|(
            initial_delay in 1u64..=10,
            max_delay in 30u64..=120,
            max_attempts in 3u32..=10,
            backoff_multiplier in 1.5f64..=3.0,
        )| {
            let config = RetryConfig {
                max_attempts,
                initial_delay,
                max_delay,
                backoff_multiplier,
            };

            let delays = calculate_retry_delays(&config);

            // Verify delays are monotonically increasing
            for i in 1..delays.len() {
                prop_assert!(
                    delays[i] >= delays[i - 1],
                    "Delay at index {} ({}) is less than previous delay ({})",
                    i,
                    delays[i],
                    delays[i - 1]
                );
            }
        });
    }

    /// Property: Default configuration produces expected sequence
    ///
    /// The default configuration should produce the sequence: 1, 2, 4, 8, 16
    #[test]
    fn property_default_config_sequence() {
        let config = RetryConfig::default();
        let delays = calculate_retry_delays(&config);

        assert_eq!(delays.len(), 5);
        assert_eq!(delays[0], 1);
        assert_eq!(delays[1], 2);
        assert_eq!(delays[2], 4);
        assert_eq!(delays[3], 8);
        assert_eq!(delays[4], 16);
    }

    /// Property: Max delay is respected
    ///
    /// For any configuration, no delay should exceed the max_delay.
    #[test]
    fn property_max_delay_respected() {
        proptest!(|(
            max_delay in 10u64..=200,
            max_attempts in 1u32..=20,
            backoff_multiplier in 1.1f64..=5.0,
        )| {
            // Ensure initial_delay is always <= max_delay
            let initial_delay = (max_delay / 2).max(1);

            let config = RetryConfig {
                max_attempts,
                initial_delay,
                max_delay,
                backoff_multiplier,
            };

            let delays = calculate_retry_delays(&config);

            for (i, &delay) in delays.iter().enumerate() {
                prop_assert!(
                    delay <= max_delay,
                    "Delay at index {} ({}) exceeds max_delay ({})",
                    i,
                    delay,
                    max_delay
                );
            }
        });
    }

    /// Property: Initial delay is always first
    ///
    /// For any configuration, the first delay should always equal the initial_delay.
    #[test]
    fn property_initial_delay_first() {
        proptest!(|(
            max_delay in 100u64..=1000,
            max_attempts in 1u32..=20,
            backoff_multiplier in 1.1f64..=5.0,
        )| {
            // Ensure initial_delay is always <= max_delay
            let initial_delay = (max_delay / 2).max(1);

            let config = RetryConfig {
                max_attempts,
                initial_delay,
                max_delay,
                backoff_multiplier,
            };

            let delays = calculate_retry_delays(&config);

            prop_assert!(!delays.is_empty(), "Delays should not be empty");
            prop_assert_eq!(
                delays[0],
                initial_delay,
                "First delay should equal initial_delay"
            );
        });
    }

    /// Property: Number of delays equals max_attempts
    ///
    /// For any configuration, the number of calculated delays should equal max_attempts.
    #[test]
    fn property_delay_count_equals_attempts() {
        proptest!(|(
            max_delay in 100u64..=1000,
            max_attempts in 1u32..=20,
            backoff_multiplier in 1.1f64..=5.0,
        )| {
            // Ensure initial_delay is always <= max_delay
            let initial_delay = (max_delay / 2).max(1);

            let config = RetryConfig {
                max_attempts,
                initial_delay,
                max_delay,
                backoff_multiplier,
            };

            let delays = calculate_retry_delays(&config);

            prop_assert_eq!(
                delays.len(),
                max_attempts as usize,
                "Number of delays should equal max_attempts"
            );
        });
    }

    /// Property: Backoff multiplier affects growth rate
    ///
    /// For any two configurations with different backoff multipliers,
    /// the one with the higher multiplier should reach max_delay faster.
    #[test]
    fn property_multiplier_affects_growth() {
        proptest!(|(
            initial_delay in 1u64..=10,
            max_delay in 100u64..=1000,
            max_attempts in 5u32..=10,
        )| {
            let config_slow = RetryConfig {
                max_attempts,
                initial_delay,
                max_delay,
                backoff_multiplier: 1.5,
            };

            let config_fast = RetryConfig {
                max_attempts,
                initial_delay,
                max_delay,
                backoff_multiplier: 3.0,
            };

            let delays_slow = calculate_retry_delays(&config_slow);
            let delays_fast = calculate_retry_delays(&config_fast);

            // Find the index where each reaches max_delay
            let slow_max_index = delays_slow.iter().position(|&d| d == max_delay);
            let fast_max_index = delays_fast.iter().position(|&d| d == max_delay);

            // If both reach max_delay, fast should reach it sooner or at the same time
            if let (Some(slow_idx), Some(fast_idx)) = (slow_max_index, fast_max_index) {
                prop_assert!(
                    fast_idx <= slow_idx,
                    "Higher multiplier should reach max_delay sooner or at same time"
                );
            }
        });
    }

    /// Property: Exponential backoff with multiplier 2.0 doubles each time until max
    ///
    /// For any configuration with multiplier 2.0, each delay should be exactly
    /// double the previous delay until max_delay is reached.
    #[test]
    fn property_multiplier_2_doubles_delay() {
        proptest!(|(
            initial_delay in 1u64..=10,
            max_delay in 100u64..=1000,
            max_attempts in 3u32..=10,
        )| {
            let config = RetryConfig {
                max_attempts,
                initial_delay,
                max_delay,
                backoff_multiplier: 2.0,
            };

            let delays = calculate_retry_delays(&config);

            for i in 1..delays.len() {
                let expected = (delays[i - 1] * 2).min(max_delay);
                prop_assert_eq!(
                    delays[i],
                    expected,
                    "With multiplier 2.0, delay should double (or cap at max)"
                );
            }
        });
    }

    /// Property: Once max_delay is reached, all subsequent delays are max_delay
    ///
    /// For any configuration, once a delay reaches max_delay, all subsequent
    /// delays should also be max_delay.
    #[test]
    fn property_stays_at_max_delay() {
        proptest!(|(
            initial_delay in 1u64..=10,
            max_delay in 20u64..=100,
            max_attempts in 5u32..=15,
            backoff_multiplier in 2.0f64..=5.0,
        )| {
            let config = RetryConfig {
                max_attempts,
                initial_delay,
                max_delay,
                backoff_multiplier,
            };

            let delays = calculate_retry_delays(&config);

            // Find the first index where max_delay is reached
            if let Some(max_index) = delays.iter().position(|&d| d == max_delay) {
                // All subsequent delays should also be max_delay
                for d in delays.iter().skip(max_index) {
                    prop_assert_eq!(
                        *d,
                        max_delay,
                        "Once max_delay is reached, all subsequent delays should be max_delay"
                    );
                }
            }
        });
    }

    /// Unit test: Specific example from requirements (1s, 2s, 4s, 8s, 16s, max 60s)
    #[test]
    fn test_requirements_example() {
        let config = RetryConfig {
            max_attempts: 7,
            initial_delay: 1,
            max_delay: 60,
            backoff_multiplier: 2.0,
        };

        let delays = calculate_retry_delays(&config);

        assert_eq!(delays.len(), 7);
        assert_eq!(delays[0], 1); // 1s
        assert_eq!(delays[1], 2); // 2s
        assert_eq!(delays[2], 4); // 4s
        assert_eq!(delays[3], 8); // 8s
        assert_eq!(delays[4], 16); // 16s
        assert_eq!(delays[5], 32); // 32s
        assert_eq!(delays[6], 60); // 60s (capped at max)
    }

    /// Unit test: Edge case with initial_delay equal to max_delay
    #[test]
    fn test_initial_equals_max() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay: 60,
            max_delay: 60,
            backoff_multiplier: 2.0,
        };

        let delays = calculate_retry_delays(&config);

        assert_eq!(delays.len(), 5);
        // All delays should be 60
        for delay in delays {
            assert_eq!(delay, 60);
        }
    }

    /// Unit test: Edge case with multiplier 1.0 (no growth)
    #[test]
    fn test_multiplier_one() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay: 10,
            max_delay: 100,
            backoff_multiplier: 1.0,
        };

        let delays = calculate_retry_delays(&config);

        assert_eq!(delays.len(), 5);
        // All delays should be 10 (no growth)
        for delay in delays {
            assert_eq!(delay, 10);
        }
    }
} // end of tests module
