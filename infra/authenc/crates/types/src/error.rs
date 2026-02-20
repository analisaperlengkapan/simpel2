//! Error types for Authenc
//!
//! This module defines the error types used throughout the Authenc system.

use thiserror::Error;

/// Main error type for Authenc operations
#[derive(Debug, Error)]
pub enum AuthencError {
    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Authorization failed
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    /// User not found
    #[error("User not found: {0}")]
    UserNotFound(String),

    /// Realm not found
    #[error("Realm not found: {0}")]
    RealmNotFound(String),

    /// Client not found
    #[error("Client not found: {0}")]
    ClientNotFound(String),

    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    /// Invalid credentials
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// User disabled
    #[error("User account is disabled")]
    UserDisabled,

    /// Account locked
    #[error("Account is locked due to too many failed attempts. Username: {username}, locked until: {locked_until:?}")]
    AccountLocked {
        username: String,
        locked_until: Option<std::time::Instant>,
    },

    /// Realm disabled
    #[error("Realm is disabled")]
    RealmDisabled,

    /// Invalid token
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    /// Token expired
    #[error("Token has expired")]
    TokenExpired,

    /// Invalid MFA code
    #[error("Invalid MFA code")]
    InvalidMfaCode,

    /// MFA required
    #[error("Multi-factor authentication is required")]
    MfaRequired,

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Cryptography error
    #[error("Cryptography error: {0}")]
    CryptoError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Conflict error (e.g., duplicate username)
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Username already exists
    #[error("Username already exists: {0}")]
    UsernameAlreadyExists(String),

    /// Email already exists
    #[error("Email already exists: {0}")]
    EmailAlreadyExists(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Unauthorized error
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// WebAuthn error
    #[error("WebAuthn error: {0}")]
    WebAuthnError(String),

    /// Not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// OAuth2 error
    #[error("OAuth2 error: {0}")]
    OAuth2Error(String),
}

impl AuthencError {
    /// Create a database error
    pub fn database<S: Into<String>>(msg: S) -> Self {
        Self::DatabaseError(msg.into())
    }

    /// Create a crypto error
    pub fn crypto<S: Into<String>>(msg: S) -> Self {
        Self::CryptoError(msg.into())
    }

    /// Create a config error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::ConfigError(msg.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::ValidationError(msg.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::InternalError(msg.into())
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create an unauthorized error
    pub fn unauthorized<S: Into<String>>(msg: S) -> Self {
        Self::Unauthorized(msg.into())
    }

    /// Create a WebAuthn error
    pub fn webauthn<S: Into<String>>(msg: S) -> Self {
        Self::WebAuthnError(msg.into())
    }
}

// Conversion from OAuth2Error to AuthencError
impl From<crate::domain_types::OAuth2Error> for AuthencError {
    fn from(err: crate::domain_types::OAuth2Error) -> Self {
        let msg = if let Some(desc) = err.error_description {
            format!("{}: {}", err.error, desc)
        } else {
            err.error
        };
        Self::OAuth2Error(msg)
    }
}

// Conversion from tokio_postgres::Error to AuthencError
impl From<tokio_postgres::Error> for AuthencError {
    fn from(err: tokio_postgres::Error) -> Self {
        Self::DatabaseError(err.to_string())
    }
}
