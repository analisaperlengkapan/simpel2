//! Shared health check implementations
//!
//! This module provides unified health check logic that can be used by both
//! HTTP/REST endpoints (handlers/health.rs) and gRPC endpoints (grpc/health.rs).
//!
//! # Architecture
//!
//! - `checks`: Actual health check implementations for dependencies
//! - `types`: Shared types and enums for health status

pub mod checks;
pub mod types;

pub use checks::{DatabaseHealthCheck, KafkaHealthCheck, RedisHealthCheck, SecretonHealthCheck};
pub use types::{DependencyHealth, HealthStatus};
