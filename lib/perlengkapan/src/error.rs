//! Unified error type for the Perlengkapan service.
//!
//! `ServiceError` is the single error type used across every module of the
//! `layanan-perlengkapan` backend service. It is intentionally lean (no `From`
//! impls for backend-only crates) so this library remains WASM-compatible.
//! Service-side code converts concrete errors (e.g. `tokio_postgres::Error`,
//! `tonic::Status`, `validator::ValidationErrors`) into `ServiceError` via
//! `.into()` helpers defined in the service crate's `shared::error` module.

use thiserror::Error;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Database pool error: {0}")]
    DatabasePool(String),

    #[error("gRPC error: {0}")]
    Grpc(String),

    #[error("Authenc service error: {0}")]
    Authenc(String),

    #[error("Secreton service error: {0}")]
    Secreton(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Business rule violation: {0}")]
    BusinessRule(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Integration error: {0}")]
    Integration(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ServiceError {
    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }
    pub fn business_rule(msg: impl Into<String>) -> Self {
        Self::BusinessRule(msg.into())
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
    pub fn integration(msg: impl Into<String>) -> Self {
        Self::Integration(msg.into())
    }
    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage(msg.into())
    }
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    /// Suggested HTTP status code for this error variant.
    /// Service-side code uses this when implementing `IntoResponse` for axum.
    pub fn http_status(&self) -> u16 {
        match self {
            Self::Validation(_) => 400,
            Self::Unauthorized(_) | Self::InvalidToken(_) => 401,
            Self::Forbidden(_) => 403,
            Self::NotFound(_) => 404,
            Self::Conflict(_) | Self::BusinessRule(_) => 409,
            Self::RateLimit(_) => 429,
            Self::ExternalService(_) | Self::Authenc(_) | Self::Secreton(_) | Self::Grpc(_) => 502,
            _ => 500,
        }
    }

    /// Stable machine-readable error code (frontend can branch on this).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Database(_) => "DATABASE_ERROR",
            Self::DatabasePool(_) => "DATABASE_POOL_ERROR",
            Self::Grpc(_) => "GRPC_ERROR",
            Self::Authenc(_) => "AUTHENC_ERROR",
            Self::Secreton(_) => "SECRETON_ERROR",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::BusinessRule(_) => "BUSINESS_RULE_VIOLATION",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::InvalidToken(_) => "INVALID_TOKEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
            Self::Integration(_) => "INTEGRATION_ERROR",
            Self::Storage(_) => "STORAGE_ERROR",
            Self::Cache(_) => "CACHE_ERROR",
            Self::Configuration(_) => "CONFIGURATION_ERROR",
            Self::RateLimit(_) => "RATE_LIMIT_EXCEEDED",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;
