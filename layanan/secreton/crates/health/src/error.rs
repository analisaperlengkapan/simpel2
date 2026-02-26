//! Health check error types

use thiserror::Error;

/// Errors that can occur during health checks
#[derive(Debug, Error)]
pub enum HealthCheckError {
    /// Health check with the given name already exists
    #[error("Health check '{0}' already registered")]
    AlreadyRegistered(String),

    /// Health check with the given name was not found
    #[error("Health check '{0}' not found")]
    NotFound(String),

    /// Health check execution failed
    #[error("Health check execution failed: {0}")]
    ExecutionFailed(String),

    /// Health check timed out
    #[error("Health check timed out after {0}ms")]
    Timeout(u64),

    /// Invalid health check configuration
    #[error("Invalid health check configuration: {0}")]
    InvalidConfiguration(String),
}

impl HealthCheckError {
    /// Creates a new execution failed error
    pub fn execution_failed(message: impl Into<String>) -> Self {
        Self::ExecutionFailed(message.into())
    }

    /// Creates a new timeout error
    pub fn timeout(timeout_ms: u64) -> Self {
        Self::Timeout(timeout_ms)
    }

    /// Creates a new invalid configuration error
    pub fn invalid_configuration(message: impl Into<String>) -> Self {
        Self::InvalidConfiguration(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = HealthCheckError::AlreadyRegistered("test".to_string());
        assert_eq!(err.to_string(), "Health check 'test' already registered");

        let err = HealthCheckError::NotFound("test".to_string());
        assert_eq!(err.to_string(), "Health check 'test' not found");

        let err = HealthCheckError::execution_failed("Connection failed");
        assert_eq!(
            err.to_string(),
            "Health check execution failed: Connection failed"
        );

        let err = HealthCheckError::timeout(5000);
        assert_eq!(err.to_string(), "Health check timed out after 5000ms");

        let err = HealthCheckError::invalid_configuration("Missing required field");
        assert_eq!(
            err.to_string(),
            "Invalid health check configuration: Missing required field"
        );
    }
}
