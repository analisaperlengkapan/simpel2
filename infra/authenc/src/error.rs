use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

/// Type alias for Results in this crate
pub type Result<T> = std::result::Result<T, AuthencError>;

/// Legacy alias for backward compatibility
pub type AuthenceResult<T> = Result<T>; // backward compat

/// Backwards-compatibility alias for older optimized error type name
pub type OptimizedAuthencError = AuthencError;

/// Comprehensive error types for the Authence application
#[derive(Error, Debug)]
pub enum AuthencError {
    // Authentication and authorization errors
    /// Authentication failed due to invalid credentials or session
    #[error("Authentication failed")]
    AuthenticationFailed,

    /// Access denied due to insufficient permissions
    #[error("Access denied: insufficient permissions")]
    AccessDenied,

    /// Invalid credentials provided during authentication
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// JWT token has expired and needs refresh
    #[error("Token expired")]
    TokenExpired,

    /// JWT token is malformed or invalid
    #[error("Invalid token")]
    InvalidToken,

    /// Account locked due to too many failed authentication attempts
    #[error("Account locked: {reason}")]
    AccountLocked {
        /// The reason for the account lockout
        reason: String,
        /// When the account will be unlocked
        locked_until: std::time::Instant,
    },

    /// Unauthorized access attempt with custom message
    #[error("Unauthorized: {message}")]
    Unauthorized {
        /// The custom error message describing the unauthorized access
        message: String,
    },

    /// Forbidden operation with custom message
    #[error("Forbidden: {message}")]
    Forbidden {
        /// The custom error message describing the forbidden operation
        message: String,
    },

    /// UMA 2.0 authorization error with structured response
    #[error("UMA error: {0}")]
    Uma(crate::services::uma::UmaError),

    // Validation errors
    /// Input validation failed with custom message
    #[error("Invalid input: {message}")]
    ValidationError {
        /// The custom error message describing the validation failure
        message: String,
    },

    /// Required field is missing from input
    #[error("Required field missing: {field}")]
    MissingField {
        /// The name of the missing required field
        field: String,
    },

    /// Field has invalid format
    #[error("Invalid format: {field}")]
    InvalidFormat {
        /// The name of the field with invalid format
        field: String,
    },

    // Resource errors
    /// User account not found in the system
    #[error("User not found")]
    UserNotFound,

    /// Requested resource does not exist
    #[error("Resource not found: {resource}")]
    ResourceNotFound {
        /// The identifier or name of the resource that was not found
        resource: String,
    },

    /// Resource already exists and cannot be created again
    #[error("Resource already exists: {resource}")]
    ResourceExists {
        /// The identifier or name of the resource that already exists
        resource: String,
    },

    /// Operation not permitted due to resource state conflict
    #[error("Operation not permitted on resource: {resource}")]
    ResourceConflict {
        /// The identifier or name of the resource with the state conflict
        resource: String,
    },

    // System errors
    /// Database operation failed with custom message
    #[error("Database error: {message}")]
    DatabaseError {
        /// The detailed error message from the database operation
        message: String,
    },

    /// Configuration is invalid or missing
    #[error("Configuration error: {message}")]
    ConfigurationError {
        /// The detailed error message describing the configuration issue
        message: String,
    },

    /// External service dependency failed
    #[error("External service error: {service}")]
    ExternalServiceError {
        /// The name or identifier of the external service that failed
        service: String,
    },

    /// Rate limit exceeded for the operation
    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded {
        /// The detailed error message
        message: String,
    },

    /// Service is temporarily unavailable
    #[error("Service temporarily unavailable")]
    ServiceUnavailable,

    // Internal errors
    /// Internal server error with custom message
    #[error("Internal server error: {message}")]
    InternalError {
        /// The detailed error message describing the internal server error
        message: String,
    },

    /// Data serialization/deserialization failed
    #[error("Serialization error: {message}")]
    SerializationError {
        /// The detailed error message from the serialization/deserialization operation
        message: String,
    },

    /// Cryptographic operation failed
    #[error("Cryptographic operation failed")]
    CryptographicError,

    /// Network communication failed
    #[error("Network error: {message}")]
    NetworkError {
        /// The detailed error message describing the network communication failure
        message: String,
    },

    // Secreton integration errors
    /// Secreton communication failed with retry capability
    #[error("Secreton communication failed: {message}")]
    SecretonCommunicationError {
        /// The detailed error message from secreton communication
        message: String,
        /// Whether this error should trigger a retry
        retryable: bool,
    },

    /// Secret access denied by secreton
    #[error("Secret access denied: {path}")]
    SecretAccessDenied {
        /// The path of the secret that was denied access
        path: String,
    },

    /// Secreton authentication failed
    #[error("Secreton authentication failed: {reason}")]
    SecretonAuthenticationFailed {
        /// The reason for authentication failure
        reason: String,
    },

    /// Secret not found in secreton
    #[error("Secret not found in secreton: {path}")]
    SecretNotFound {
        /// The path of the secret that was not found
        path: String,
    },

    /// Secreton service unavailable
    #[error("Secreton service unavailable")]
    SecretonUnavailable,

    /// Secreton token validation failed
    #[error("Secreton token validation failed: {reason}")]
    SecretonTokenValidationFailed {
        /// The reason for token validation failure
        reason: String,
    },

    // Satker-specific errors
    /// Invalid satker code provided
    #[error("Invalid satker code: {satker_code}")]
    InvalidSatkerCode {
        /// The invalid satker code that was provided
        satker_code: String,
    },

    /// Satker access denied for operation
    #[error("Satker access denied: {satker_code} for operation: {operation}")]
    SatkerAccessDenied {
        /// The satker code that was denied access
        satker_code: String,
        /// The operation that was denied
        operation: String,
    },

    /// Cross-satker operation not permitted
    #[error("Cross-satker operation not permitted: {source_satker} -> {target_satker}")]
    CrossSatkerOperationDenied {
        /// The source satker attempting the operation
        source_satker: String,
        /// The target satker for the operation
        target_satker: String,
    },

    /// Satker hierarchy violation
    #[error("Satker hierarchy violation: {admin_level} cannot manage {target_satker}")]
    SatkerHierarchyViolation {
        /// The admin level attempting the operation
        admin_level: String,
        /// The target satker that cannot be managed
        target_satker: String,
    },

    /// Satker configuration error
    #[error("Satker configuration error: {satker_code} - {message}")]
    SatkerConfigurationError {
        /// The satker code with configuration issues
        satker_code: String,
        /// The detailed error message
        message: String,
    },

    // Compliance violation errors
    /// Audit compliance violation detected
    #[error("Audit compliance violation: {violation_type}")]
    AuditComplianceViolation {
        /// The type of compliance violation
        violation_type: String,
        /// Additional context about the violation
        context: Option<String>,
    },

    /// Security policy violation
    #[error("Security policy violation: {policy} - {details}")]
    SecurityPolicyViolation {
        /// The security policy that was violated
        policy: String,
        /// Detailsthe violation
        details: String,
    },

    /// Compliance requirement not met
    #[error("Compliance requirement not met: {requirement}")]
    ComplianceRequirementNotMet {
        /// The compliance requirement that was not met
        requirement: String,
        /// Additional details about the requirement
        details: Option<String>,
    },

    /// Data retention policy violation
    #[error("Data retention policy violation: {policy} - {resource}")]
    DataRetentionViolation {
        /// The data retention policy that was violated
        policy: String,
        /// The resource affected by the violation
        resource: String,
    },

    /// Unauthorized compliance access attempt
    #[error("Unauthorized compliance access: {operation} requires {required_level}")]
    UnauthorizedComplianceAccess {
        /// The operation that was attempted
        operation: String,
        /// The required compliance level
        required_level: String,
    },

    // Role-based access control errors
    /// Role assignment violation
    #[error("Role assignment violation: {role} cannot be assigned to {user_nip} in {satker_code}")]
    RoleAssignmentViolation {
        /// The role that cannot be assigned
        role: String,
        /// The NIP of the user
        user_nip: String,
        /// The satker code where assignment was attempted
        satker_code: String,
    },

    /// Insufficient admin privileges
    #[error("Insufficient admin privileges: {admin_level} required for {operation}")]
    InsufficientAdminPrivileges {
        /// The admin level required for the operation
        admin_level: String,
        /// The operation that requires higher privileges
        operation: String,
    },

    /// Role scope violation
    #[error("Role scope violation: {role} scope {scope} conflicts with operation")]
    RoleScopeViolation {
        /// The role with scope issues
        role: String,
        /// The scope that conflicts
        scope: String,
    },

    // MFA-specific errors
    /// MFA is not enabled for the user
    #[error("MFA is not enabled for this user")]
    MfaNotEnabled,

    /// MFA is already enabled for the user
    #[error("MFA is already enabled for this user")]
    MfaAlreadyEnabled,

    /// Invalid OTP code provided
    #[error("Invalid OTP code")]
    InvalidOtpCode,

    /// MFA setup is required but not completed
    #[error("MFA setup is required")]
    MfaSetupRequired,

    /// MFA verification is required
    #[error("MFA verification is required")]
    MfaVerificationRequired,

    /// MFA secret encryption failed
    #[error("Failed to encrypt MFA secret")]
    MfaSecretEncryptionFailed,

    /// MFA secret decryption failed
    #[error("Failed to decrypt MFA secret")]
    MfaSecretDecryptionFailed,

    /// QR code generation failed
    #[error("QR code generation failed: {message}")]
    QrCodeGenerationFailed {
        /// The detailed error message
        message: String,
    },

    /// Backup code verification failed
    #[error("Invalid backup code")]
    InvalidBackupCode,

    /// No backup codes available
    #[error("No backup codes available")]
    NoBackupCodesAvailable,

    /// Recovery code already used
    #[error("Recovery code has already been used")]
    RecoveryCodeAlreadyUsed,

    /// MFA rate limit exceeded
    #[error("MFA verification rate limit exceeded")]
    MfaRateLimitExceeded,

    /// MFA account locked due to too many failed attempts
    #[error("Account locked due to too many failed MFA attempts")]
    MfaAccountLocked,
}

impl AuthencError {
    /// Create a validation error with custom message
    pub fn validation<T: Into<String>>(message: T) -> Self {
        Self::ValidationError {
            message: message.into(),
        }
    }

    /// Create a missing field error
    pub fn missing_field<T: Into<String>>(field: T) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    /// Create a resource not found error
    pub fn resource_not_found<T: Into<String>>(resource: T) -> Self {
        Self::ResourceNotFound {
            resource: resource.into(),
        }
    }

    /// Create a database error
    pub fn database<T: Into<String>>(message: T) -> Self {
        Self::DatabaseError {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal<T: Into<String>>(message: T) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }

    /// Create an unauthorized error for authentication failures
    ///
    /// This constructor creates an `AuthencError::Unauthorized` variant to indicate
    /// that the request lacks valid authentication credentials or the provided
    /// credentials are invalid/expired.
    ///
    /// # Arguments
    /// * `message` - A descriptive message explaining the authentication failure
    ///
    /// # Returns
    /// An `AuthencError::Unauthorized` instance with the provided message
    ///
    /// # Example
    /// ```rust
    /// use authenc::error::AuthencError;
    ///
    /// let error = AuthencError::unauthorized("Invalid or expired authentication token");
    /// ```
    pub fn unauthorized<T: Into<String>>(message: T) -> Self {
        Self::Unauthorized {
            message: message.into(),
        }
    }

    /// Create a forbidden error for access denied scenarios
    ///
    /// This constructor creates an `AuthencError::Forbidden` variant to indicate
    /// that the authenticated user does not have sufficient permissions to access
    /// the requested resource, even though they are authenticated.
    ///
    /// # Arguments
    /// * `message` - A descriptive message explaining why access was denied
    ///
    /// # Returns
    /// An `AuthencError::Forbidden` instance with the provided message
    ///
    /// # Example
    /// ```rust
    /// use authenc::error::AuthencError;
    ///
    /// let error = AuthencError::forbidden("User lacks required role for this operation");
    /// ```
    pub fn forbidden<T: Into<String>>(message: T) -> Self {
        Self::Forbidden {
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found<T: Into<String>>(message: T) -> Self {
        Self::ResourceNotFound {
            resource: message.into(),
        }
    }

    /// Create a conflict error
    pub fn conflict<T: Into<String>>(message: T) -> Self {
        Self::ValidationError {
            message: format!("Conflict: {}", message.into()),
        }
    }

    // Secreton integration error constructors
    /// Create a secreton communication error
    pub fn secreton_communication<T: Into<String>>(message: T, retryable: bool) -> Self {
        Self::SecretonCommunicationError {
            message: message.into(),
            retryable,
        }
    }

    /// Create a secret access denied error
    pub fn secret_access_denied<T: Into<String>>(path: T) -> Self {
        Self::SecretAccessDenied { path: path.into() }
    }

    /// Create a secreton authentication failed error
    pub fn secreton_auth_failed<T: Into<String>>(reason: T) -> Self {
        Self::SecretonAuthenticationFailed {
            reason: reason.into(),
        }
    }

    /// Create a secret not found error
    pub fn secret_not_found<T: Into<String>>(path: T) -> Self {
        Self::SecretNotFound { path: path.into() }
    }

    /// Create a secreton token validation failed error
    pub fn secreton_token_validation_failed<T: Into<String>>(reason: T) -> Self {
        Self::SecretonTokenValidationFailed {
            reason: reason.into(),
        }
    }

    // Satker-specific error constructors
    /// Create an invalid satker code error
    pub fn invalid_satker_code<T: Into<String>>(satker_code: T) -> Self {
        Self::InvalidSatkerCode {
            satker_code: satker_code.into(),
        }
    }

    /// Create a satker access denied error
    pub fn satker_access_denied<T: Into<String>>(satker_code: T, operation: T) -> Self {
        Self::SatkerAccessDenied {
            satker_code: satker_code.into(),
            operation: operation.into(),
        }
    }

    /// Create a cross-satker operation denied error
    pub fn cross_satker_denied<T: Into<String>>(source_satker: T, target_satker: T) -> Self {
        Self::CrossSatkerOperationDenied {
            source_satker: source_satker.into(),
            target_satker: target_satker.into(),
        }
    }

    /// Create a satker hierarchy violation error
    pub fn satker_hierarchy_violation<T: Into<String>>(admin_level: T, target_satker: T) -> Self {
        Self::SatkerHierarchyViolation {
            admin_level: admin_level.into(),
            target_satker: target_satker.into(),
        }
    }

    /// Create a satker configuration error
    pub fn satker_config_error<T: Into<String>>(satker_code: T, message: T) -> Self {
        Self::SatkerConfigurationError {
            satker_code: satker_code.into(),
            message: message.into(),
        }
    }

    // Compliance violation error constructors
    /// Create an audit compliance violation error
    pub fn audit_compliance_violation<T: Into<String>>(
        violation_type: T,
        context: Option<T>,
    ) -> Self {
        Self::AuditComplianceViolation {
            violation_type: violation_type.into(),
            context: context.map(|c| c.into()),
        }
    }

    /// Create a security policy violation error
    pub fn security_policy_violation<T: Into<String>>(policy: T, details: T) -> Self {
        Self::SecurityPolicyViolation {
            policy: policy.into(),
            details: details.into(),
        }
    }

    /// Create a compliance requirement not met error
    pub fn compliance_requirement_not_met<T: Into<String>>(
        requirement: T,
        details: Option<T>,
    ) -> Self {
        Self::ComplianceRequirementNotMet {
            requirement: requirement.into(),
            details: details.map(|d| d.into()),
        }
    }

    /// Create a data retention violation error
    pub fn data_retention_violation<T: Into<String>>(policy: T, resource: T) -> Self {
        Self::DataRetentionViolation {
            policy: policy.into(),
            resource: resource.into(),
        }
    }

    /// Create an unauthorized compliance access error
    pub fn unauthorized_compliance_access<T: Into<String>>(
        operation: T,
        required_level: T,
    ) -> Self {
        Self::UnauthorizedComplianceAccess {
            operation: operation.into(),
            required_level: required_level.into(),
        }
    }

    // Role-based access control error constructors
    /// Create a role assignment violation error
    pub fn role_assignment_violation<T: Into<String>>(
        role: T,
        user_nip: T,
        satker_code: T,
    ) -> Self {
        Self::RoleAssignmentViolation {
            role: role.into(),
            user_nip: user_nip.into(),
            satker_code: satker_code.into(),
        }
    }

    /// Create an insufficient admin privileges error
    pub fn insufficient_admin_privileges<T: Into<String>>(admin_level: T, operation: T) -> Self {
        Self::InsufficientAdminPrivileges {
            admin_level: admin_level.into(),
            operation: operation.into(),
        }
    }

    /// Create a role scope violation error
    pub fn role_scope_violation<T: Into<String>>(role: T, scope: T) -> Self {
        Self::RoleScopeViolation {
            role: role.into(),
            scope: scope.into(),
        }
    }

    // MFA-specific error constructors
    /// Create an MFA not enabled error
    pub fn mfa_not_enabled() -> Self {
        Self::MfaNotEnabled
    }

    /// Create an MFA already enabled error
    pub fn mfa_already_enabled() -> Self {
        Self::MfaAlreadyEnabled
    }

    /// Create an invalid OTP code error
    pub fn invalid_otp_code() -> Self {
        Self::InvalidOtpCode
    }

    /// Create an MFA setup required error
    pub fn mfa_setup_required() -> Self {
        Self::MfaSetupRequired
    }

    /// Create an MFA verification required error
    pub fn mfa_verification_required() -> Self {
        Self::MfaVerificationRequired
    }

    /// Create an MFA secret encryption failed error
    pub fn mfa_secret_encryption_failed() -> Self {
        Self::MfaSecretEncryptionFailed
    }

    /// Create an MFA secret decryption failed error
    pub fn mfa_secret_decryption_failed() -> Self {
        Self::MfaSecretDecryptionFailed
    }

    /// Create a QR code generation failed error
    pub fn qr_code_generation_failed<T: Into<String>>(message: T) -> Self {
        Self::QrCodeGenerationFailed {
            message: message.into(),
        }
    }

    /// Create an invalid backup code error
    pub fn invalid_backup_code() -> Self {
        Self::InvalidBackupCode
    }

    /// Create a no backup codes available error
    pub fn no_backup_codes_available() -> Self {
        Self::NoBackupCodesAvailable
    }

    /// Create an MFA rate limit exceeded error
    pub fn mfa_rate_limit_exceeded() -> Self {
        Self::MfaRateLimitExceeded
    }

    /// Create a rate limit exceeded error
    pub fn rate_limit_exceeded() -> Self {
        Self::RateLimitExceeded {
            message: "Rate limit exceeded".to_string(),
        }
    }

    /// Create a too many requests error with custom message
    pub fn too_many_requests<T: Into<String>>(message: T) -> Self {
        Self::RateLimitExceeded {
            message: message.into(),
        }
    }

    /// Create an invalid recovery code error
    pub fn invalid_recovery_code() -> Self {
        Self::InvalidBackupCode
    }

    /// Create a recovery code already used error
    pub fn recovery_code_already_used() -> Self {
        Self::RecoveryCodeAlreadyUsed
    }

    /// Create an MFA account locked error
    pub fn mfa_account_locked() -> Self {
        Self::MfaAccountLocked
    }

    /// Check if the error should be logged as an error (vs warning)
    pub fn should_log_as_error(&self) -> bool {
        matches!(
            self,
            AuthencError::DatabaseError { .. }
                | AuthencError::ConfigurationError { .. }
                | AuthencError::ExternalServiceError { .. }
                | AuthencError::InternalError { .. }
                | AuthencError::CryptographicError
                | AuthencError::ServiceUnavailable
                | AuthencError::Unauthorized { .. }
                | AuthencError::Forbidden { .. }
                | AuthencError::SecretonCommunicationError { .. }
                | AuthencError::SecretonUnavailable
                | AuthencError::AuditComplianceViolation { .. }
                | AuthencError::SecurityPolicyViolation { .. }
                | AuthencError::SatkerHierarchyViolation { .. }
                | AuthencError::MfaSecretEncryptionFailed
                | AuthencError::MfaSecretDecryptionFailed
                | AuthencError::MfaAccountLocked
        )
    }

    /// Check if this is a secreton-related error
    pub fn is_secreton_related(&self) -> bool {
        matches!(
            self,
            AuthencError::SecretonCommunicationError { .. }
                | AuthencError::SecretAccessDenied { .. }
                | AuthencError::SecretonAuthenticationFailed { .. }
                | AuthencError::SecretNotFound { .. }
                | AuthencError::SecretonUnavailable
                | AuthencError::SecretonTokenValidationFailed { .. }
        )
    }

    /// Check if this error should trigger a retry for secreton operations
    pub fn should_retry_secreton(&self) -> bool {
        match self {
            AuthencError::SecretonCommunicationError { retryable, .. } => *retryable,
            AuthencError::SecretonUnavailable => true,
            AuthencError::NetworkError { .. } => true,
            _ => false,
        }
    }

    /// Check if this is a satker-related error
    pub fn is_satker_related(&self) -> bool {
        matches!(
            self,
            AuthencError::InvalidSatkerCode { .. }
                | AuthencError::SatkerAccessDenied { .. }
                | AuthencError::CrossSatkerOperationDenied { .. }
                | AuthencError::SatkerHierarchyViolation { .. }
                | AuthencError::SatkerConfigurationError { .. }
        )
    }

    /// Check if this is a compliance-related error
    pub fn is_compliance_related(&self) -> bool {
        matches!(
            self,
            AuthencError::AuditComplianceViolation { .. }
                | AuthencError::SecurityPolicyViolation { .. }
                | AuthencError::ComplianceRequirementNotMet { .. }
                | AuthencError::DataRetentionViolation { .. }
                | AuthencError::UnauthorizedComplianceAccess { .. }
        )
    }

    /// Check if this is a role-based access control error
    pub fn is_rbac_related(&self) -> bool {
        matches!(
            self,
            AuthencError::RoleAssignmentViolation { .. }
                | AuthencError::InsufficientAdminPrivileges { .. }
                | AuthencError::RoleScopeViolation { .. }
        )
    }

    /// Check if this is an MFA-related error
    pub fn is_mfa_related(&self) -> bool {
        matches!(
            self,
            AuthencError::MfaNotEnabled
                | AuthencError::MfaAlreadyEnabled
                | AuthencError::InvalidOtpCode
                | AuthencError::MfaSetupRequired
                | AuthencError::MfaVerificationRequired
                | AuthencError::MfaSecretEncryptionFailed
                | AuthencError::MfaSecretDecryptionFailed
                | AuthencError::QrCodeGenerationFailed { .. }
                | AuthencError::InvalidBackupCode
                | AuthencError::NoBackupCodesAvailable
                | AuthencError::RecoveryCodeAlreadyUsed
                | AuthencError::MfaRateLimitExceeded
                | AuthencError::MfaAccountLocked
        )
    }

    /// Get the retry delay in milliseconds for retryable errors
    pub fn retry_delay_ms(&self) -> Option<u64> {
        if self.should_retry_secreton() {
            match self {
                AuthencError::SecretonCommunicationError { .. } => Some(1000), // 1 second
                AuthencError::SecretonUnavailable => Some(5000),               // 5 seconds
                AuthencError::NetworkError { .. } => Some(2000),               // 2 seconds
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get the maximum number of retries for this error type
    pub fn max_retries(&self) -> Option<u32> {
        if self.should_retry_secreton() {
            match self {
                AuthencError::SecretonCommunicationError { .. } => Some(3),
                AuthencError::SecretonUnavailable => Some(2),
                AuthencError::NetworkError { .. } => Some(3),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get the error code for structured logging and monitoring
    pub fn error_code(&self) -> &'static str {
        match self {
            AuthencError::AuthenticationFailed => "AUTH_FAILED",
            AuthencError::AccessDenied => "ACCESS_DENIED",
            AuthencError::InvalidCredentials => "INVALID_CREDENTIALS",
            AuthencError::TokenExpired => "TOKEN_EXPIRED",
            AuthencError::InvalidToken => "INVALID_TOKEN",
            AuthencError::AccountLocked { .. } => "ACCOUNT_LOCKED",
            AuthencError::Unauthorized { .. } => "UNAUTHORIZED",
            AuthencError::Forbidden { .. } => "FORBIDDEN",
            AuthencError::ValidationError { .. } => "VALIDATION_ERROR",
            AuthencError::MissingField { .. } => "MISSING_FIELD",
            AuthencError::InvalidFormat { .. } => "INVALID_FORMAT",
            AuthencError::UserNotFound => "USER_NOT_FOUND",
            AuthencError::ResourceNotFound { .. } => "RESOURCE_NOT_FOUND",
            AuthencError::ResourceExists { .. } => "RESOURCE_EXISTS",
            AuthencError::ResourceConflict { .. } => "RESOURCE_CONFLICT",
            AuthencError::DatabaseError { .. } => "DATABASE_ERROR",
            AuthencError::ConfigurationError { .. } => "CONFIG_ERROR",
            AuthencError::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
            AuthencError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            AuthencError::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            AuthencError::InternalError { .. } => "INTERNAL_ERROR",
            AuthencError::CryptographicError => "CRYPTO_ERROR",
            AuthencError::SerializationError { .. } => "SERIALIZATION_ERROR",
            AuthencError::NetworkError { .. } => "NETWORK_ERROR",
            // Secreton integration errors
            AuthencError::SecretonCommunicationError { .. } => "SECRETON_COMM_ERROR",
            AuthencError::SecretAccessDenied { .. } => "SECRET_ACCESS_DENIED",
            AuthencError::SecretonAuthenticationFailed { .. } => "SECRETON_AUTH_FAILED",
            AuthencError::SecretNotFound { .. } => "SECRET_NOT_FOUND",
            AuthencError::SecretonUnavailable => "SECRETON_UNAVAILABLE",
            AuthencError::SecretonTokenValidationFailed { .. } => {
                "SECRETON_TOKEN_VALIDATION_FAILED"
            }
            // Satker-specific errors
            AuthencError::InvalidSatkerCode { .. } => "INVALID_SATKER_CODE",
            AuthencError::SatkerAccessDenied { .. } => "SATKER_ACCESS_DENIED",
            AuthencError::CrossSatkerOperationDenied { .. } => "CROSS_SATKER_DENIED",
            AuthencError::SatkerHierarchyViolation { .. } => "SATKER_HIERARCHY_VIOLATION",
            AuthencError::SatkerConfigurationError { .. } => "SATKER_CONFIG_ERROR",
            // Compliance violation errors
            AuthencError::AuditComplianceViolation { .. } => "AUDIT_COMPLIANCE_VIOLATION",
            AuthencError::SecurityPolicyViolation { .. } => "SECURITY_POLICY_VIOLATION",
            AuthencError::ComplianceRequirementNotMet { .. } => "COMPLIANCE_REQUIREMENT_NOT_MET",
            AuthencError::DataRetentionViolation { .. } => "DATA_RETENTION_VIOLATION",
            AuthencError::UnauthorizedComplianceAccess { .. } => "UNAUTHORIZED_COMPLIANCE_ACCESS",
            // Role-based access control errors
            AuthencError::RoleAssignmentViolation { .. } => "ROLE_ASSIGNMENT_VIOLATION",
            AuthencError::InsufficientAdminPrivileges { .. } => "INSUFFICIENT_ADMIN_PRIVILEGES",
            AuthencError::RoleScopeViolation { .. } => "ROLE_SCOPE_VIOLATION",
            // MFA-specific errors
            AuthencError::MfaNotEnabled => "MFA_NOT_ENABLED",
            AuthencError::MfaAlreadyEnabled => "MFA_ALREADY_ENABLED",
            AuthencError::InvalidOtpCode => "INVALID_OTP_CODE",
            AuthencError::MfaSetupRequired => "MFA_SETUP_REQUIRED",
            AuthencError::MfaVerificationRequired => "MFA_VERIFICATION_REQUIRED",
            AuthencError::MfaSecretEncryptionFailed => "MFA_SECRET_ENCRYPTION_FAILED",
            AuthencError::MfaSecretDecryptionFailed => "MFA_SECRET_DECRYPTION_FAILED",
            AuthencError::QrCodeGenerationFailed { .. } => "QR_CODE_GENERATION_FAILED",
            AuthencError::InvalidBackupCode => "INVALID_BACKUP_CODE",
            AuthencError::NoBackupCodesAvailable => "NO_BACKUP_CODES_AVAILABLE",
            AuthencError::RecoveryCodeAlreadyUsed => "RECOVERY_CODE_ALREADY_USED",
            AuthencError::MfaRateLimitExceeded => "MFA_RATE_LIMIT_EXCEEDED",
            AuthencError::MfaAccountLocked => "MFA_ACCOUNT_LOCKED",
            AuthencError::Uma(_) => "UMA_ERROR",
        }
    }
}

impl IntoResponse for AuthencError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        // Handle UMA errors specially - they have their own format
        if let AuthencError::Uma(ref uma_error) = self {
            let response_body = json!({
                "error": uma_error.error,
                "error_description": uma_error.error_description,
                "status": uma_error.status,
                "ticket": uma_error.ticket,
                "required_claims": uma_error.required_claims,
                "redirect_uri": uma_error.redirect_uri,
            });
            return (status, Json(response_body)).into_response();
        }

        // For internal errors, don't expose sensitive information
        let error_message = match &self {
            AuthencError::DatabaseError { .. }
            | AuthencError::ConfigurationError { .. }
            | AuthencError::InternalError { .. }
            | AuthencError::CryptographicError
            | AuthencError::SerializationError { .. }
            | AuthencError::NetworkError { .. } => {
                tracing::error!("Internal error occurred: {}", self);
                "An internal error occurred. Please try again later.".to_string()
            }
            _ => self.to_string(),
        };

        let response_body = json!({
            "error": {
                "code": self.error_code(),
                "message": error_message,
                "status": status.as_u16()
            }
        });

        (status, Json(response_body)).into_response()
    }
}

impl AuthencError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 400 Bad Request
            AuthencError::ValidationError { .. }
            | AuthencError::MissingField { .. }
            | AuthencError::InvalidFormat { .. }
            | AuthencError::InvalidCredentials
            | AuthencError::InvalidSatkerCode { .. }
            | AuthencError::SatkerConfigurationError { .. }
            | AuthencError::InvalidOtpCode
            | AuthencError::InvalidBackupCode
            | AuthencError::RecoveryCodeAlreadyUsed
            | AuthencError::QrCodeGenerationFailed { .. } => StatusCode::BAD_REQUEST,

            // 401 Unauthorized
            AuthencError::AuthenticationFailed
            | AuthencError::TokenExpired
            | AuthencError::InvalidToken
            | AuthencError::Unauthorized { .. }
            | AuthencError::SecretonAuthenticationFailed { .. }
            | AuthencError::SecretonTokenValidationFailed { .. }
            | AuthencError::MfaSetupRequired
            | AuthencError::MfaVerificationRequired => StatusCode::UNAUTHORIZED,

            // 403 Forbidden
            AuthencError::AccessDenied
            | AuthencError::AccountLocked { .. }
            | AuthencError::Forbidden { .. }
            | AuthencError::SecretAccessDenied { .. }
            | AuthencError::SatkerAccessDenied { .. }
            | AuthencError::CrossSatkerOperationDenied { .. }
            | AuthencError::SatkerHierarchyViolation { .. }
            | AuthencError::AuditComplianceViolation { .. }
            | AuthencError::SecurityPolicyViolation { .. }
            | AuthencError::ComplianceRequirementNotMet { .. }
            | AuthencError::DataRetentionViolation { .. }
            | AuthencError::UnauthorizedComplianceAccess { .. }
            | AuthencError::RoleAssignmentViolation { .. }
            | AuthencError::InsufficientAdminPrivileges { .. }
            | AuthencError::RoleScopeViolation { .. }
            | AuthencError::MfaAccountLocked => StatusCode::FORBIDDEN,

            // UMA errors - map based on status field
            AuthencError::Uma(uma_error) => {
                StatusCode::from_u16(uma_error.status.unwrap_or(403u16))
                    .unwrap_or(StatusCode::FORBIDDEN)
            }

            // 404 Not Found
            AuthencError::UserNotFound
            | AuthencError::ResourceNotFound { .. }
            | AuthencError::SecretNotFound { .. }
            | AuthencError::MfaNotEnabled
            | AuthencError::NoBackupCodesAvailable => StatusCode::NOT_FOUND,

            // 409 Conflict
            AuthencError::ResourceExists { .. }
            | AuthencError::ResourceConflict { .. }
            | AuthencError::MfaAlreadyEnabled => StatusCode::CONFLICT,

            // 429 Too Many Requests
            AuthencError::RateLimitExceeded { .. } | AuthencError::MfaRateLimitExceeded => {
                StatusCode::TOO_MANY_REQUESTS
            }

            // 503 Service Unavailable
            AuthencError::ServiceUnavailable | AuthencError::SecretonUnavailable => {
                StatusCode::SERVICE_UNAVAILABLE
            }

            // 500 Internal Server Error
            AuthencError::DatabaseError { .. }
            | AuthencError::ConfigurationError { .. }
            | AuthencError::ExternalServiceError { .. }
            | AuthencError::InternalError { .. }
            | AuthencError::CryptographicError
            | AuthencError::SerializationError { .. }
            | AuthencError::NetworkError { .. }
            | AuthencError::SecretonCommunicationError { .. }
            | AuthencError::MfaSecretEncryptionFailed
            | AuthencError::MfaSecretDecryptionFailed => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Convert common error types to AuthencError
impl From<anyhow::Error> for AuthencError {
    fn from(err: anyhow::Error) -> Self {
        AuthencError::internal(err.to_string())
    }
}

impl From<serde_json::Error> for AuthencError {
    fn from(err: serde_json::Error) -> Self {
        AuthencError::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<tokio_postgres::Error> for AuthencError {
    fn from(err: tokio_postgres::Error) -> Self {
        AuthencError::database(err.to_string())
    }
}

impl From<deadpool_postgres::PoolError> for AuthencError {
    fn from(err: deadpool_postgres::PoolError) -> Self {
        AuthencError::database(err.to_string())
    }
}

impl From<argon2::password_hash::Error> for AuthencError {
    fn from(_err: argon2::password_hash::Error) -> Self {
        AuthencError::CryptographicError
    }
}

impl From<std::num::ParseIntError> for AuthencError {
    fn from(err: std::num::ParseIntError) -> Self {
        AuthencError::ConfigurationError {
            message: format!("Failed to parse integer: {}", err),
        }
    }
}

impl From<std::str::ParseBoolError> for AuthencError {
    fn from(err: std::str::ParseBoolError) -> Self {
        AuthencError::ConfigurationError {
            message: format!("Failed to parse boolean: {}", err),
        }
    }
}

impl From<std::io::Error> for AuthencError {
    fn from(err: std::io::Error) -> Self {
        AuthencError::NetworkError {
            message: format!("IO error: {}", err),
        }
    }
}

impl From<uuid::Error> for AuthencError {
    fn from(err: uuid::Error) -> Self {
        AuthencError::ValidationError {
            message: format!("Invalid UUID: {}", err),
        }
    }
}

impl From<lib_common::error::CommonError> for AuthencError {
    fn from(err: lib_common::error::CommonError) -> Self {
        match err {
            lib_common::error::CommonError::Validation { message } => {
                Self::ValidationError { message }
            }
            lib_common::error::CommonError::ValidationErrors(e) => Self::ValidationError {
                message: format!("Validation errors: {}", e),
            },
            lib_common::error::CommonError::Cache(message) => Self::InternalError {
                message: format!("Cache error: {}", message),
            },
            lib_common::error::CommonError::Database(message) => Self::InternalError {
                message: format!("Database error: {}", message),
            },
            lib_common::error::CommonError::Serialization(message) => Self::InternalError {
                message: format!("Serialization error: {}", message),
            },
            lib_common::error::CommonError::Deserialization(message) => Self::InternalError {
                message: format!("Deserialization error: {}", message),
            },
            lib_common::error::CommonError::Internal(message) => Self::InternalError { message },
        }
    }
}
