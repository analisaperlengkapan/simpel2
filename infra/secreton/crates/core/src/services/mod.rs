//! Service modules for Secreton engine system
//!
//! This module contains all core business logic services for the Secreton engine.
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
//! - [`seal`] - Engine seal/unseal operations
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
//! // Unseal engine with threshold shares
//! seal_manager.unseal(&shares[0]).await?;
//! # Ok(())
//! # }
//! ```

// Core services (essential)
pub mod classification;
pub mod health;
pub mod identity;
pub mod key_hierarchy;
// TODO: key_manager needs refactoring to use proper StorageBackend API
// pub mod key_manager;
pub mod lease;
pub mod metrics;
pub mod mfa;
pub mod policy;
pub mod policy_service;
pub mod rate_limit;
pub mod rbac;
pub mod revocation;
pub mod rotation;
pub mod seal;
pub mod token;
pub mod wrapping;
pub mod zero_knowledge;

// Auth services
pub mod auth;

// Dynamic secrets
pub mod dynamic;

// Secrets management
pub mod secrets;

// Refactored services from API crate
pub mod admin_service;
pub mod auth_service;
pub mod container;
pub mod namespace_persistence;
pub mod seal_adapter;
pub mod secret_service;
