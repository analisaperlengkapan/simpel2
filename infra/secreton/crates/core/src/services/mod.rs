//! Service modules for Secreton vault system
//!
//! This module contains core services for the vault system.

// Core services (essential)
pub mod health;
pub mod identity;
// TODO: key_manager needs refactoring to use proper StorageBackend API
// pub mod key_manager;
pub mod lease;
pub mod metrics;
pub mod mfa;
pub mod policy;
pub mod rate_limit;
pub mod rbac;
pub mod seal;
pub mod token;
pub mod wrapping;

// Auth services
pub mod auth;

// Dynamic secrets
pub mod dynamic;

// Secrets management
pub mod secrets;
