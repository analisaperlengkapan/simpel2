//! Resilience patterns for Secreton
//!
//! This module provides resilience patterns including circuit breaker and retry logic
//! to prevent cascade failures when integrating with external services like Authenc.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Mewakili pub `CircuitBreakerState`.
pub enum CircuitBreakerState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker for handling external service failures
///
/// Implements the circuit breaker pattern to prevent cascade failures.
/// When a service fails repeatedly, the circuit opens and blocks requests
/// for a timeout period, then enters half-open state to test recovery.
#[derive(Debug, Clone)]
/// Mewakili pub `CircuitBreaker`.
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
/// Mewakili pub `CircuitBreakerConfig`.
pub struct CircuitBreakerConfig {
    /// Maximum number of failures before opening circuit
    pub max_failures: u32,
    /// Timeout before attempting recovery (half-open state)
    pub timeout: Duration,
    /// Reset timeout after successful recovery
    pub reset_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            max_failures: 5,
            timeout: Duration::from_secs(30),
            reset_timeout: Duration::from_secs(60),
        }
    }
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default configuration
    pub fn new() -> Self {
        Self::with_config(CircuitBreakerConfig::default())
    }

    /// Create a new circuit breaker with custom configuration
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Check if a request can be executed
    ///
    /// Returns true if the circuit is closed or half-open, false if open.
    /// When circuit is open and timeout has elapsed, transitions to half-open.
    pub async fn can_execute(&self) -> bool {
        let state = *self.state.read().await;

        match state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if timeout has elapsed
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() >= self.config.timeout {
                        // Transition to half-open
                        *self.state.write().await = CircuitBreakerState::HalfOpen;
                        tracing::info!("Circuit breaker transitioning to HalfOpen state");
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    /// Record a successful request
    ///
    /// In half-open state, this closes the circuit.
    /// In closed state, this resets the failure count.
    pub async fn record_success(&self) {
        let mut state = self.state.write().await;

        match *state {
            CircuitBreakerState::Closed => {
                // Reset failure count on success
                *self.failure_count.write().await = 0;
            }
            CircuitBreakerState::HalfOpen => {
                // Recovery successful, close circuit
                *state = CircuitBreakerState::Closed;
                *self.failure_count.write().await = 0;
                tracing::info!("Circuit breaker recovered, transitioning to Closed state");
            }
            CircuitBreakerState::Open => {
                // Transition to closed after reset timeout
                *state = CircuitBreakerState::Closed;
                *self.failure_count.write().await = 0;
                tracing::info!("Circuit breaker force closed after timeout");
            }
        }
    }

    /// Record a failed request
    ///
    /// Increments failure count and opens circuit if threshold exceeded.
    pub async fn record_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;
        *self.last_failure_time.write().await = Some(Instant::now());

        if *failure_count >= self.config.max_failures {
            let mut state = self.state.write().await;
            *state = CircuitBreakerState::Open;
            tracing::warn!("Circuit breaker opened after {} failures", *failure_count);
        }
    }

    /// Get current state
    pub async fn state(&self) -> CircuitBreakerState {
        *self.state.read().await
    }

    /// Get current failure count
    pub async fn failure_count(&self) -> u32 {
        *self.failure_count.read().await
    }

    /// Reset the circuit breaker to closed state
    pub async fn reset(&self) {
        *self.state.write().await = CircuitBreakerState::Closed;
        *self.failure_count.write().await = 0;
        *self.last_failure_time.write().await = None;
        tracing::info!("Circuit breaker manually reset");
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_circuit_breaker_states() {
        let cb = CircuitBreaker::new();

        // Initially closed
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
        assert!(cb.can_execute().await);

        // Record failures
        for _ in 0..4 {
            cb.record_failure().await;
            assert_eq!(cb.state().await, CircuitBreakerState::Closed);
        }

        // 5th failure opens circuit
        cb.record_failure().await;
        assert_eq!(cb.state().await, CircuitBreakerState::Open);
        assert!(!cb.can_execute().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let config = CircuitBreakerConfig {
            max_failures: 2,
            timeout: Duration::from_millis(100),
            reset_timeout: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::with_config(config);

        // Open circuit
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.state().await, CircuitBreakerState::Open);

        // Wait for timeout
        sleep(Duration::from_millis(150)).await;

        // Should transition to half-open
        assert!(cb.can_execute().await);
        assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

        // Success should close circuit
        cb.record_success().await;
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let cb = CircuitBreaker::new();

        // Open circuit
        for _ in 0..5 {
            cb.record_failure().await;
        }
        assert_eq!(cb.state().await, CircuitBreakerState::Open);

        // Manual reset
        cb.reset().await;
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
        assert_eq!(cb.failure_count().await, 0);
    }

    #[tokio::test]
    async fn test_success_resets_failure_count() {
        let cb = CircuitBreaker::new();

        // Some failures
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.failure_count().await, 2);

        // Success resets
        cb.record_success().await;
        assert_eq!(cb.failure_count().await, 0);
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    }
}
