//! lib-backend: Backend infrastructure utilities for SIMPEL services
//!
//! This crate provides backend-only infrastructure including database pools,
//! caching, middleware, telemetry, JWT decoding, gRPC utils, and storage.

// Re-export lib-core for convenience
pub use lib_core;
pub use lib_core::error::{CommonError, Result};

// Backend modules (always available with tokio)
pub mod cache;
pub mod cache_middleware;
pub mod client_ip;
pub mod memory;

// Config loading from env
#[cfg(feature = "env-config")]
pub mod config;

// Error types with Axum IntoResponse
#[cfg(feature = "axum")]
pub mod error;

// Correlation and Context with Axum support
#[cfg(feature = "axum")]
pub mod correlation;

#[cfg(feature = "axum")]
pub mod context;

// Validation with db-dependent validators
#[cfg(feature = "db")]
pub mod validation;

// Audit logger with database persistence
#[cfg(feature = "db")]
pub mod audit;

#[cfg(feature = "storage")]
pub mod storage;

#[cfg(feature = "jwt")]
pub mod jwt;

#[cfg(feature = "telemetry")]
pub mod telemetry;

#[cfg(feature = "db")]
pub mod db;

#[cfg(feature = "grpc")]
pub mod grpc;

#[cfg(feature = "axum")]
pub mod middleware;
