//! Service modules for Secreton engine system
//!
//! This module contains all core business logic services for the Secreton engine.
//! Services are organized by functional domain:
//!
//! # Core Services
//! - [`health`] - System health checks and readiness probes
//! - [`lease`] - Lease management for temporary secrets
//! - [`metrics`] - Performance and usage metrics
//! - [`mfa`] - Multi-factor authentication
//! - [`policy`] - RBAC policy engine
//! - [`rate_limit`] - API rate limiting
//! - [`seal`] - Engine seal/unseal operations
//! - [`token`] - Token generation and validation
//! - [`wrapping`] - Response wrapping for secure secret delivery
//!
//! # Secrets Management
//! - [`secrets`] - Secret storage, versioning, and encryption engines
//!
//! # Architecture
//! All services follow a consistent pattern:
//! - Async/await for non-blocking I/O
//! - Trait-based design for extensibility
//! - Storage backend abstraction via [`crate::storage::StorageBackend`]
//! - Comprehensive audit logging via [`crate::audit::AuditLogger`]
//!
//! # Example
//!
//! Initialising the engine hands back the Shamir shares and deliberately leaves
//! it SEALED — the operator must unseal with `threshold` shares afterwards. That
//! is a security property, so the example asserts it rather than describing it.
//!
//! ```rust
//! use secreton_core::services::seal::{SealConfig, SealService, SealState};
//!
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let seal = SealService::new(SealConfig::default()); // 5 shares, threshold 3
//! assert!(seal.is_sealed().await);
//!
//! let shares = seal.initialize().await.unwrap();
//! assert_eq!(shares.len(), 5);
//!
//! let status = seal.status().await;
//! assert_eq!(status.state, SealState::Sealed); // init does NOT auto-unseal
//! assert_eq!(status.threshold, 3);
//! # });
//! ```

// Core services (essential)
pub mod classification;
pub mod health;
pub mod key_hierarchy;
pub mod lease;
pub mod metrics;
pub mod mfa;
pub mod policy;
pub mod rate_limit;
pub mod revocation;
pub mod rotation;
pub mod seal;
pub mod token;
pub mod wrapping;
pub mod zero_knowledge;

// Secrets management — including dynamic credentials. There is no separate
// `dynamic` module: `secrets::database` is the one engine that actually
// provisions and revokes, and it is what both api/handlers/dynamic.rs and
// grpc/server.rs call.
pub mod secrets;

// Refactored services from API crate
pub mod namespace_persistence;
pub mod seal_adapter;
