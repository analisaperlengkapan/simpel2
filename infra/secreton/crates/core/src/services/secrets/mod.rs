//! Secrets engines for Secreton
//!
//! This module provides a unified interface for various secret storage and
//! cryptographic engines through the `SecretEngine` trait. Each engine implements
//! this trait to provide consistent CRUD operations, key management, and
//! cryptographic operations.

use async_trait::async_trait;
use serde_json::Value;
use std::error::Error;

/// Common trait for all secret engines
///
/// Each secret engine (KV, Transit, Database, SSH, etc.) implements this trait
/// to provide a consistent interface for:
/// - Secret read/write/delete operations
/// - List operations with path support
/// - Key generation and rotation (for cryptographic engines)
/// - Configuration management
///
/// # Associated Types
///
/// - `Config`: Engine-specific configuration
/// - `Error`: Engine-specific error type
#[async_trait]
pub trait SecretEngine: Send + Sync {
    /// Configuration type for this engine
    type Config;

    /// Error type for this engine
    type Error: Error + Send + Sync + 'static;

    /// Write a secret at the given path
    ///
    /// # Arguments
    /// * `path` - Secret path (engine mount + key path)
    /// * `data` - Secret data (JSON)
    async fn write_secret(&self, path: &str, data: Value) -> Result<(), Self::Error>;

    /// Read a secret from the given path
    ///
    /// # Returns
    /// Secret data as JSON, or None if not found
    async fn read_secret(&self, path: &str) -> Result<Option<Value>, Self::Error>;

    /// Delete a secret at the given path
    async fn delete_secret(&self, path: &str) -> Result<(), Self::Error>;

    /// List secrets under a path prefix
    ///
    /// # Returns
    /// List of secret keys (not full paths)
    async fn list_secrets(&self, prefix: &str) -> Result<Vec<String>, Self::Error>;

    /// Generate a new encryption/signing key (for crypto engines)
    ///
    /// # Arguments
    /// * `name` - Key name
    /// * `config` - Key generation config (algorithm, key size, etc)
    ///
    /// # Returns
    /// Some(key_id) for crypto engines, None for non-crypto engines
    async fn generate_key(&self, name: &str, config: Value) -> Result<Option<String>, Self::Error> {
        let _ = (name, config);
        Ok(None)
    }

    /// Rotate an encryption key (for crypto engines)
    async fn rotate_key(&self, name: &str) -> Result<(), Self::Error> {
        let _ = name;
        Ok(())
    }

    /// List all key names (for crypto engines)
    async fn list_keys(&self) -> Result<Vec<String>, Self::Error> {
        Ok(vec![])
    }

    /// Update engine configuration
    async fn update_config(&self, config: Self::Config) -> Result<(), Self::Error>;

    /// Get current engine configuration
    async fn get_config(&self) -> Result<Self::Config, Self::Error>;

    /// Get engine name (e.g., "kv", "transit", "database")
    fn name(&self) -> &str;

    /// Get engine version (e.g., "v1", "v2")
    fn version(&self) -> &str {
        "v1"
    }
}

pub mod aws;
pub mod azure;
pub mod cubbyhole;
pub mod database;
pub mod enhanced;
pub mod gcp;
pub mod identity;
pub mod kafka;
pub mod kmip;
pub mod kubernetes;
pub mod kvv2;
pub mod ldap;
pub mod lease_integration;
pub mod mongodb;
pub mod mysql;
pub mod pki;
pub mod rabbitmq;
pub mod redis;
pub mod ssh;
pub mod totp;
pub mod transform;
pub mod transit;

pub use aws::*;
pub use azure::*;
pub use cubbyhole::*;
pub use database::*;
pub use enhanced::*;
pub use gcp::*;
pub use identity::*;
pub use kmip::*;
pub use kubernetes::*;
pub use kvv2::*;
pub use lease_integration::*;
pub use mongodb::*;
pub use mysql::*;
pub use pki::*;
pub use redis::*;
pub use ssh::*;
pub use totp::*;
pub use transform::*;
pub use transit::*;
