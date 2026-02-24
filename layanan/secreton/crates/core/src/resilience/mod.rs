//! Resilience patterns module
//!
//! Provides resilience patterns for external service integration

pub mod circuit_breaker;

pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState};
