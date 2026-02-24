//! Error types for the Kubernetes operator

use thiserror::Error;

/// Result type for operator operations
pub type Result<T> = std::result::Result<T, Error>;

/// Operator error types
#[derive(Debug, Error)]
pub enum Error {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("YAML serialization error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Template rendering error: {0}")]
    TemplateError(String),

    #[error("Secret not found in Secreton: {0}")]
    SecretNotFound(String),

    #[error("Invalid secret path: {0}")]
    InvalidSecretPath(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Reconciliation failed: {0}")]
    ReconciliationFailed(String),
}
