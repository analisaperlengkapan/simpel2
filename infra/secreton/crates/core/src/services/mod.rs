//! Service modules for Secreton vault system
//!
//! This module contains all core business logic services for the Secreton vault.
//! Services are organized by functional domain:
//!
//! # Core Services
//! - [`health`] - System health checks and readiness probes
//! - [`identity`] - Entity identity management
//! - [`lease`] - Lease management for temporary secrets
//! - [`metrics`] - Performance and usage metrics
//! - [`mfa`] - Multi-factor authentication
//! - [`policy`] - RBAC policy engine
//! - [`rate_limit`] - API rate limiting
//! - [`rbac`] - Role-based access control
//! - [`seal`] - Vault seal/unseal operations
//! - [`token`] - Token generation and validation
//! - [`wrapping`] - Response wrapping for secure secret delivery
//!
//! # Authentication Services
//! - [`auth`] - Authentication providers (userpass, certificate, LDAP, OAuth, etc.)
//!
//! # Dynamic Secrets
//! - [`dynamic`] - Dynamic secret generation for databases, cloud providers
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
//! ```rust,no_run
//! use secreton_core::services::seal::SealManager;
//! use secreton_core::storage::InMemoryStorage;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = Arc::new(InMemoryStorage::new());
//! let seal_manager = SealManager::new(storage, 3, 5).await?;
//!
//! // Initialize seal with shares
//! let shares = seal_manager.init().await?;
//! // Unseal vault with threshold shares
//! seal_manager.unseal(&shares[0]).await?;
//! # Ok(())
//! # }
//! ```

// Core services (essential)
pub mod health;
/// Mewakili pub `identity`.
pub mod identity;
// TODO: key_manager needs refactoring to use proper StorageBackend API
// pub mod key_manager;
/// Mewakili pub `lease`.
pub mod lease;
/// Mewakili pub `metrics`.
pub mod metrics;
/// Mewakili pub `mfa`.
pub mod mfa;
/// Mewakili pub `policy`.
pub mod policy;
/// Mewakili pub `rate_limit`.
pub mod rate_limit;
/// Mewakili pub `rbac`.
pub mod rbac;
/// Mewakili pub `rotation`.
pub mod rotation;
/// Mewakili pub `seal`.
pub mod seal;
/// Mewakili pub `token`.
pub mod token;
/// Mewakili pub `wrapping`.
pub mod wrapping;

// Auth services
/// Mewakili pub `auth`.
pub mod auth;

// Dynamic secrets
/// Mewakili pub `dynamic`.
pub mod dynamic;

// Secrets management
/// Mewakili pub `secrets`.
pub mod secrets;
