//! Error handling for core operations.

use thiserror::Error;

/// Core system errors
#[derive(Error, Debug)]
/// Mewakili pub `CoreError`.
pub enum CoreError {
    #[error("Invalid configuration: {message}")]
    Configuration { message: String },

    #[error("Authentication failed: {message}")]
    Authentication { message: String },

    #[error("Authorization failed: {message}")]
    Authorization { message: String },

    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("Resource already exists: {resource}")]
    AlreadyExists { resource: String },

    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },

    #[error("Validation failed: {message}")]
    Validation { message: String },

    #[error("Service unavailable: {message}")]
    ServiceUnavailable { message: String },

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Quota exceeded: {message}")]
    QuotaExceeded { message: String },

    #[error("Timeout occurred: {operation}")]
    Timeout { operation: String },

    // Authenc integration errors
    #[error("Authenc token validation failed: {reason}")]
    AuthencTokenValidationFailed { reason: String },

    #[error("IAM permission denied for operation: {operation}")]
    IamPermissionDenied { operation: String },

    #[error("Authenc authentication failed: {message}")]
    AuthencAuthenticationFailed { message: String },

    #[error("Authenc communication timeout")]
    AuthencCommunicationTimeout,

    // Post-quantum specific errors
    #[error("Post-quantum key validation failed: {reason}")]
    PqKeyValidationFailed { reason: String },

    #[error("Post-quantum signature verification failed: {details}")]
    PqSignatureVerificationFailed { details: String },

    // Audit compliance errors for kejaksaan operations
    #[error("Audit compliance violation: {violation}")]
    AuditComplianceViolation { violation: String },

    #[error("Satker access denied: {satker_code}")]
    SatkerAccessDenied { satker_code: String },

    #[error("Hierarchical permission denied: required level {required}, current level {current}")]
    HierarchicalPermissionDenied { required: String, current: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Secreton-specific errors
#[derive(Error, Debug)]
/// Mewakili pub `SecretonError`.
pub enum SecretonError {
    #[error("Core error: {0}")]
    Core(#[from] CoreError),

    #[error("Key not found")]
    KeyNotFound,

    #[error("Provider not found")]
    ProviderNotFound,

    #[error("No suitable provider")]
    NoSuitableProvider,

    #[error("Seal provider unavailable")]
    SealProviderUnavailable,

    #[error("Seal provider not found")]
    SealProviderNotFound,

    #[error("Invalid multi-seal data")]
    InvalidMultiSealData,

    #[error("Parent namespace not found")]
    ParentNamespaceNotFound,

    #[error("Channel send error")]
    ChannelSend,

    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Insufficient entropy")]
    InsufficientEntropy,

    #[error("IO error: {0}")]
    IoError(String),
}

/// Mewakili pub `SecretonResult`.
pub type SecretonResult<T> = std::result::Result<T, SecretonError>;

impl From<std::io::Error> for SecretonError {
    fn from(err: std::io::Error) -> Self {
        SecretonError::IoError(err.to_string())
    }
}

impl CoreError {
    /// Create configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create authentication error
    pub fn authentication<S: Into<String>>(message: S) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }

    /// Create authorization error
    pub fn authorization<S: Into<String>>(message: S) -> Self {
        Self::Authorization {
            message: message.into(),
        }
    }

    /// Create not found error
    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create already exists error
    pub fn already_exists<S: Into<String>>(resource: S) -> Self {
        Self::AlreadyExists {
            resource: resource.into(),
        }
    }

    /// Create invalid operation error
    pub fn invalid_operation<S: Into<String>>(message: S) -> Self {
        Self::InvalidOperation {
            message: message.into(),
        }
    }

    /// Create network error
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: format!("Network error: {}", message.into()),
            source: None,
        }
    }

    /// Create database error
    pub fn database<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: format!("Database error: {}", message.into()),
            source: None,
        }
    }

    /// Create internal error with context
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    /// Create internal error with source
    pub fn internal_with_source<S: Into<String>, E: std::error::Error + Send + Sync + 'static>(
        message: S,
        source: E,
    ) -> Self {
        Self::Internal {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create service unavailable error
    pub fn service_unavailable<S: Into<String>>(message: S) -> Self {
        Self::ServiceUnavailable {
            message: message.into(),
        }
    }

    /// Create timeout error
    pub fn timeout<S: Into<String>>(operation: S) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// Create quota exceeded error
    pub fn quota_exceeded<S: Into<String>>(message: S) -> Self {
        Self::QuotaExceeded {
            message: message.into(),
        }
    }

    /// Create authenc token validation failed error
    pub fn authenc_token_validation_failed<S: Into<String>>(reason: S) -> Self {
        Self::AuthencTokenValidationFailed {
            reason: reason.into(),
        }
    }

    /// Create IAM permission denied error
    pub fn iam_permission_denied<S: Into<String>>(operation: S) -> Self {
        Self::IamPermissionDenied {
            operation: operation.into(),
        }
    }

    /// Create authenc authentication failed error
    pub fn authenc_authentication_failed<S: Into<String>>(message: S) -> Self {
        Self::AuthencAuthenticationFailed {
            message: message.into(),
        }
    }

    /// Create authenc communication timeout error
    pub fn authenc_communication_timeout() -> Self {
        Self::AuthencCommunicationTimeout
    }

    /// Create post-quantum key validation failed error
    pub fn pq_key_validation_failed<S: Into<String>>(reason: S) -> Self {
        Self::PqKeyValidationFailed {
            reason: reason.into(),
        }
    }

    /// Create post-quantum signature verification failed error
    pub fn pq_signature_verification_failed<S: Into<String>>(details: S) -> Self {
        Self::PqSignatureVerificationFailed {
            details: details.into(),
        }
    }

    /// Create audit compliance violation error
    pub fn audit_compliance_violation<S: Into<String>>(violation: S) -> Self {
        Self::AuditComplianceViolation {
            violation: violation.into(),
        }
    }

    /// Create satker access denied error
    pub fn satker_access_denied<S: Into<String>>(satker_code: S) -> Self {
        Self::SatkerAccessDenied {
            satker_code: satker_code.into(),
        }
    }

    /// Create hierarchical permission denied error
    pub fn hierarchical_permission_denied<S: Into<String>>(required: S, current: S) -> Self {
        Self::HierarchicalPermissionDenied {
            required: required.into(),
            current: current.into(),
        }
    }

    /// Check if error is related to authenc integration
    pub fn is_authenc_related(&self) -> bool {
        matches!(
            self,
            Self::AuthencTokenValidationFailed { .. }
                | Self::AuthencAuthenticationFailed { .. }
                | Self::AuthencCommunicationTimeout
                | Self::IamPermissionDenied { .. }
        )
    }

    /// Check if error requires reauthentication
    pub fn requires_reauthentication(&self) -> bool {
        matches!(
            self,
            Self::AuthencTokenValidationFailed { .. }
                | Self::AuthencAuthenticationFailed { .. }
                | Self::Authentication { .. }
        )
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ServiceUnavailable { .. }
                | Self::RateLimitExceeded
                | Self::Timeout { .. }
                | Self::AuthencCommunicationTimeout
        )
    }

    /// Check if error is a client error
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::Authentication { .. }
                | Self::Authorization { .. }
                | Self::NotFound { .. }
                | Self::AlreadyExists { .. }
                | Self::InvalidOperation { .. }
                | Self::Validation { .. }
                | Self::QuotaExceeded { .. }
                | Self::AuthencTokenValidationFailed { .. }
                | Self::IamPermissionDenied { .. }
                | Self::AuthencAuthenticationFailed { .. }
                | Self::PqKeyValidationFailed { .. }
                | Self::PqSignatureVerificationFailed { .. }
                | Self::AuditComplianceViolation { .. }
                | Self::SatkerAccessDenied { .. }
                | Self::HierarchicalPermissionDenied { .. }
        )
    }

    /// Check if error is a server error
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            Self::ServiceUnavailable { .. }
                | Self::RateLimitExceeded
                | Self::Timeout { .. }
                | Self::AuthencCommunicationTimeout
                | Self::Io(_)
                | Self::Serialization(_)
                | Self::Internal { .. }
        )
    }

    /// Get error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Authentication { .. } => ErrorCategory::Security,
            Self::Authorization { .. } => ErrorCategory::Security,
            Self::AuthencTokenValidationFailed { .. } => ErrorCategory::Security,
            Self::IamPermissionDenied { .. } => ErrorCategory::Security,
            Self::AuthencAuthenticationFailed { .. } => ErrorCategory::Security,
            Self::SatkerAccessDenied { .. } => ErrorCategory::Security,
            Self::HierarchicalPermissionDenied { .. } => ErrorCategory::Security,
            Self::PqKeyValidationFailed { .. } => ErrorCategory::Cryptography,
            Self::PqSignatureVerificationFailed { .. } => ErrorCategory::Cryptography,
            Self::AuditComplianceViolation { .. } => ErrorCategory::Compliance,
            Self::NotFound { .. } => ErrorCategory::NotFound,
            Self::AlreadyExists { .. } => ErrorCategory::Conflict,
            Self::InvalidOperation { .. } => ErrorCategory::Business,
            Self::Validation { .. } => ErrorCategory::Validation,
            Self::ServiceUnavailable { .. } => ErrorCategory::Service,
            Self::RateLimitExceeded => ErrorCategory::RateLimit,
            Self::QuotaExceeded { .. } => ErrorCategory::RateLimit,
            Self::Timeout { .. } => ErrorCategory::Timeout,
            Self::AuthencCommunicationTimeout => ErrorCategory::Timeout,
            Self::Configuration { .. } => ErrorCategory::Configuration,
            Self::Io(_) => ErrorCategory::System,
            Self::Serialization(_) => ErrorCategory::System,
            Self::Internal { .. } => ErrorCategory::System,
        }
    }
}

/// Error categories for grouping and handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Mewakili pub `ErrorCategory`.
pub enum ErrorCategory {
    /// Security-related errors (authentication, authorization)
    Security,

    /// Cryptography-related errors (post-quantum, key validation)
    Cryptography,

    /// Compliance and audit errors
    Compliance,

    /// Resource not found errors
    NotFound,

    /// Resource conflict errors (already exists)
    Conflict,

    /// Business logic errors
    Business,

    /// Input validation errors
    Validation,

    /// Service availability errors
    Service,

    /// Rate limiting errors
    RateLimit,

    /// Timeout errors
    Timeout,

    /// Configuration errors
    Configuration,

    /// System-level errors (IO, serialization, etc.)
    System,
}

impl ErrorCategory {
    /// Get category name
    pub fn name(&self) -> &'static str {
        match self {
            ErrorCategory::Security => "security",
            ErrorCategory::Cryptography => "cryptography",
            ErrorCategory::Compliance => "compliance",
            ErrorCategory::NotFound => "not_found",
            ErrorCategory::Conflict => "conflict",
            ErrorCategory::Business => "business",
            ErrorCategory::Validation => "validation",
            ErrorCategory::Service => "service",
            ErrorCategory::RateLimit => "rate_limit",
            ErrorCategory::Timeout => "timeout",
            ErrorCategory::Configuration => "configuration",
            ErrorCategory::System => "system",
        }
    }
}

/// Result type alias for core operations
pub type Result<T> = std::result::Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        let auth_error = CoreError::authentication("invalid token");
        assert_eq!(auth_error.category(), ErrorCategory::Security);
        assert!(auth_error.is_client_error());
        assert!(!auth_error.is_retryable());

        let service_error = CoreError::service_unavailable("database down");
        assert_eq!(service_error.category(), ErrorCategory::Service);
        assert!(service_error.is_server_error());
        assert!(service_error.is_retryable());
    }

    #[test]
    fn test_error_construction() {
        let error = CoreError::not_found("user/123");
        match error {
            CoreError::NotFound { resource } => {
                assert_eq!(resource, "user/123");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_retryable_errors() {
        assert!(CoreError::service_unavailable("test").is_retryable());
        assert!(CoreError::RateLimitExceeded.is_retryable());
        assert!(CoreError::timeout("operation").is_retryable());
        assert!(CoreError::authenc_communication_timeout().is_retryable());

        assert!(!CoreError::authentication("test").is_retryable());
        assert!(!CoreError::validation("test").is_retryable());
    }

    #[test]
    fn test_authenc_related_errors() {
        let token_error = CoreError::authenc_token_validation_failed("expired token");
        assert!(token_error.is_authenc_related());
        assert!(token_error.requires_reauthentication());
        assert_eq!(token_error.category(), ErrorCategory::Security);

        let iam_error = CoreError::iam_permission_denied("read_secret");
        assert!(iam_error.is_authenc_related());
        assert!(!iam_error.requires_reauthentication());
        assert_eq!(iam_error.category(), ErrorCategory::Security);

        let auth_failed = CoreError::authenc_authentication_failed("invalid credentials");
        assert!(auth_failed.is_authenc_related());
        assert!(auth_failed.requires_reauthentication());

        let timeout = CoreError::authenc_communication_timeout();
        assert!(timeout.is_authenc_related());
        assert!(!timeout.requires_reauthentication());
        assert!(timeout.is_retryable());
    }

    #[test]
    fn test_post_quantum_errors() {
        let pq_key_error = CoreError::pq_key_validation_failed("invalid ML-KEM key");
        assert_eq!(pq_key_error.category(), ErrorCategory::Cryptography);
        assert!(pq_key_error.is_client_error());
        assert!(!pq_key_error.is_retryable());

        let pq_sig_error =
            CoreError::pq_signature_verification_failed("ML-DSA verification failed");
        assert_eq!(pq_sig_error.category(), ErrorCategory::Cryptography);
        assert!(pq_sig_error.is_client_error());
    }

    #[test]
    fn test_compliance_errors() {
        let audit_error = CoreError::audit_compliance_violation("missing audit trail");
        assert_eq!(audit_error.category(), ErrorCategory::Compliance);
        assert!(audit_error.is_client_error());

        let satker_error = CoreError::satker_access_denied("SATKER001");
        assert_eq!(satker_error.category(), ErrorCategory::Security);
        assert!(satker_error.is_client_error());

        let hierarchy_error =
            CoreError::hierarchical_permission_denied("AdminPusat", "AdminSatker");
        assert_eq!(hierarchy_error.category(), ErrorCategory::Security);
        assert!(hierarchy_error.is_client_error());
    }

    #[test]
    fn test_requires_reauthentication() {
        assert!(CoreError::authenc_token_validation_failed("test").requires_reauthentication());
        assert!(CoreError::authenc_authentication_failed("test").requires_reauthentication());
        assert!(CoreError::authentication("test").requires_reauthentication());

        assert!(!CoreError::iam_permission_denied("test").requires_reauthentication());
        assert!(!CoreError::authenc_communication_timeout().requires_reauthentication());
        assert!(!CoreError::validation("test").requires_reauthentication());
    }

    #[test]
    fn test_error_category_names() {
        assert_eq!(ErrorCategory::Security.name(), "security");
        assert_eq!(ErrorCategory::Cryptography.name(), "cryptography");
        assert_eq!(ErrorCategory::Compliance.name(), "compliance");
        assert_eq!(ErrorCategory::Timeout.name(), "timeout");
    }
}
