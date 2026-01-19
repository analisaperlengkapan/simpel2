//! Authentication methods for Secreton
//!
//! This module provides a unified interface for various authentication mechanisms
//! through the `AuthMethod` trait. Each auth method implements this trait to provide
//! consistent authentication, role management, and configuration interfaces.

use async_trait::async_trait;
use serde_json::Value;
use std::error::Error;

/// Common trait for all authentication methods
/// Each auth method (OIDC, Kubernetes, LDAP, etc.) implements this trait
/// to provide a consistent interface for:
/// - User authentication with method-specific credentials
/// - Role creation and management
/// - Configuration updates
/// # Associated Types
/// - `Request`: Authentication request type (e.g., JWT, username/password, certificate)
/// - `Response`: Authentication response with user info and metadata
/// - `Error`: Method-specific error type
#[async_trait]
/// Mewakili pub `AuthMethod`.
pub trait AuthMethod: Send + Sync {
    /// Authentication request type (method-specific)
    type Request;

    /// Authentication response type (typically contains UserInfo)
    type Response;

    /// Error type for this auth method
    type Error: Error + Send + Sync + 'static;

    /// Authenticate a user with method-specific credentials
    ///
    /// # Arguments
    /// * `request` - Authentication request (type varies by method)
    ///
    /// # Returns
    /// Authentication response with user identity and metadata
    async fn authenticate(&self, request: Self::Request) -> Result<Self::Response, Self::Error>;

    /// Create a new role with given configuration
    ///
    /// # Arguments
    /// * `name` - Role name
    /// * `config` - Role configuration (JSON, method-specific schema)
    async fn create_role(&self, name: String, config: Value) -> Result<(), Self::Error>;

    /// Get role configuration by name
    ///
    /// # Returns
    /// Role configuration JSON, or None if not found
    async fn get_role(&self, name: &str) -> Result<Option<Value>, Self::Error>;

    /// List all role names
    async fn list_roles(&self) -> Result<Vec<String>, Self::Error>;

    /// Delete a role by name
    async fn delete_role(&self, name: &str) -> Result<(), Self::Error>;

    /// Get auth method name (e.g., "oidc", "kubernetes", "ldap")
    fn name(&self) -> &str;
}

pub mod approle;
/// Mewakili pub `kubernetes`.
pub mod kubernetes;
/// Mewakili pub `ldap`.
pub mod ldap;
/// Mewakili pub `oidc`.
pub mod oidc;
// pub mod token;  // Missing file
/// Mewakili pub `aws`.
pub mod aws;
/// Mewakili pub `certificate`.
pub mod certificate;
/// Mewakili pub `github`.
pub mod github;
// pub mod okta;  // Missing file
// pub mod radius;  // Missing file
/// Mewakili pub `userpass`.
pub mod userpass;
