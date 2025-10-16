//! Comprehensive error handling for the transit engine

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Comprehensive cryptographic error types for the transit engine
#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CryptoError {
    // Key management errors
    #[error("Key already exists: {0}")]
    KeyAlreadyExists(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Key version not found: {0}")]
    KeyVersionNotFound(u32),

    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[error("Key rotation failed: {0}")]
    KeyRotationFailed(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    // Encryption/Decryption errors
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid ciphertext: {0}")]
    InvalidCiphertext(String),

    // Signing/Verification errors
    #[error("Signing failed: {0}")]
    SigningFailed(String),

    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    // General input/usage errors
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid usage: {0}")]
    InvalidUsage(String),

    // Parameter validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("Invalid nonce/IV length")]
    InvalidNonceLength,

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    // Usage and policy errors
    #[error("Invalid algorithm: {0}")]
    InvalidAlgorithm(String),

    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    // System errors
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Random generation failed")]
    RandomGenerationFailed,

    #[error("Hash operation failed: {0}")]
    HashFailed(String),

    // Audit and compliance errors
    #[error("Audit logging failed: {0}")]
    AuditLogFailed(String),

    #[error("Compliance check failed: {0}")]
    ComplianceFailed(String),

    // Timeout and resource errors
    #[error("Operation timeout")]
    OperationTimeout,

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Concurrent operation limit exceeded")]
    ConcurrencyLimitExceeded,

    // Batch operation errors
    #[error("Batch operation failed: {0}")]
    BatchOperationFailed(String),

    #[error("Batch size exceeded: {current} > {max}")]
    BatchSizeExceeded { current: usize, max: usize },

    // Authenc integration errors
    #[error("Authenc token validation failed: {reason}")]
    AuthencTokenValidationFailed { reason: String },

    #[error("Authenc authentication failed: {message}")]
    AuthencAuthenticationFailed { message: String },

    #[error("IAM permission denied for operation: {operation}")]
    IamPermissionDenied { operation: String },

    #[error("Authenc communication timeout")]
    AuthencCommunicationTimeout,

    #[error("Authenc service unavailable")]
    AuthencServiceUnavailable,

    #[error("Invalid authenc token format: {details}")]
    InvalidAuthencTokenFormat { details: String },

    #[error("Authenc token expired: {token_id}")]
    AuthencTokenExpired { token_id: String },

    // Role-based access control errors
    #[error("Role access denied: {role} cannot access {resource}")]
    RoleAccessDenied { role: String, resource: String },

    #[error("Insufficient role privileges: {current_role} requires {required_role}")]
    InsufficientRolePrivileges {
        current_role: String,
        required_role: String,
    },

    #[error("Role scope violation: {role} scope {scope} conflicts with {resource}")]
    RoleScopeViolation {
        role: String,
        scope: String,
        resource: String,
    },

    #[error("Role assignment error: {role} cannot be assigned to {user_nip}")]
    RoleAssignmentError { role: String, user_nip: String },

    #[error("Admin level insufficient: {admin_level} required for {operation}")]
    AdminLevelInsufficient {
        admin_level: String,
        operation: String,
    },

    #[error("Satker access violation: {satker_code} cannot access {target_satker} resources")]
    SatkerAccessViolation {
        satker_code: String,
        target_satker: String,
    },

    #[error("Cross-satker operation denied: {operation} from {source_satker} to {target_satker}")]
    CrossSatkerOperationDenied {
        operation: String,
        source_satker: String,
        target_satker: String,
    },

    // Post-quantum cryptography errors
    #[error("Post-quantum algorithm not supported: {algorithm}")]
    PostQuantumAlgorithmNotSupported { algorithm: String },

    #[error("Post-quantum key generation failed: {algorithm} - {reason}")]
    PostQuantumKeyGenerationFailed { algorithm: String, reason: String },

    #[error("Post-quantum signature verification failed: {algorithm}")]
    PostQuantumSignatureVerificationFailed { algorithm: String },

    #[error("Post-quantum encryption failed: {algorithm} - {reason}")]
    PostQuantumEncryptionFailed { algorithm: String, reason: String },

    #[error("Post-quantum decryption failed: {algorithm} - {reason}")]
    PostQuantumDecryptionFailed { algorithm: String, reason: String },

    #[error("Hybrid crypto mode error: {mode} - {details}")]
    HybridCryptoModeError { mode: String, details: String },

    #[error("ML-DSA operation failed: {operation} - {reason}")]
    MlDsaOperationFailed { operation: String, reason: String },

    #[error("ML-KEM operation failed: {operation} - {reason}")]
    MlKemOperationFailed { operation: String, reason: String },

    #[error("Post-quantum migration error: {from_algorithm} to {to_algorithm} - {reason}")]
    PostQuantumMigrationError {
        from_algorithm: String,
        to_algorithm: String,
        reason: String,
    },

    // Audit and compliance errors
    #[error("Audit trail violation: {violation_type} - {details}")]
    AuditTrailViolation {
        violation_type: String,
        details: String,
    },

    #[error("Compliance requirement not met: {requirement} - {context}")]
    ComplianceRequirementNotMet {
        requirement: String,
        context: String,
    },

    #[error("Audit log integrity check failed: {log_id}")]
    AuditLogIntegrityFailed { log_id: String },

    #[error("Compliance policy violation: {policy} - {violation_details}")]
    CompliancePolicyViolation {
        policy: String,
        violation_details: String,
    },

    #[error("Audit retention policy violation: {policy} - {resource}")]
    AuditRetentionPolicyViolation { policy: String, resource: String },

    #[error("Compliance monitoring failed: {monitor_type} - {reason}")]
    ComplianceMonitoringFailed {
        monitor_type: String,
        reason: String,
    },

    #[error("Audit signature verification failed: {audit_id}")]
    AuditSignatureVerificationFailed { audit_id: String },

    #[error("Compliance certification expired: {cert_type} - {expiry_date}")]
    ComplianceCertificationExpired {
        cert_type: String,
        expiry_date: String,
    },

    // SIMKARI-specific errors
    #[error("Satker configuration invalid: {satker_code} - {issue}")]
    SatkerConfigurationInvalid { satker_code: String, issue: String },

    #[error("NIP validation failed: {nip} - {reason}")]
    NipValidationFailed { nip: String, reason: String },

    #[error("Kejaksaan hierarchy violation: {level} cannot access {target_level}")]
    KejaksaanHierarchyViolation { level: String, target_level: String },

    #[error("Wilayah access restriction: {wilayah} cannot access {target_resource}")]
    WilayahAccessRestriction {
        wilayah: String,
        target_resource: String,
    },

    #[error("Pusat-level operation required: {operation} requires pusat privileges")]
    PusatLevelOperationRequired { operation: String },
}

/// Type alias for Results with CryptoError
pub type CryptoResult<T> = Result<T, CryptoError>;

impl CryptoError {
    /// Check if error is recoverable (temporary)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            CryptoError::NetworkError(_)
                | CryptoError::OperationTimeout
                | CryptoError::ResourceExhausted(_)
                | CryptoError::ConcurrencyLimitExceeded
                | CryptoError::AuthencCommunicationTimeout
                | CryptoError::AuthencServiceUnavailable
        )
    }

    /// Check if error should be retried
    pub fn should_retry(&self) -> bool {
        matches!(
            self,
            CryptoError::OperationTimeout
                | CryptoError::NetworkError(_)
                | CryptoError::ConcurrencyLimitExceeded
                | CryptoError::AuthencCommunicationTimeout
                | CryptoError::AuthencServiceUnavailable
        )
    }

    /// Check if this is an authenc-related error
    pub fn is_authenc_related(&self) -> bool {
        matches!(
            self,
            CryptoError::AuthencTokenValidationFailed { .. }
                | CryptoError::AuthencAuthenticationFailed { .. }
                | CryptoError::IamPermissionDenied { .. }
                | CryptoError::AuthencCommunicationTimeout
                | CryptoError::AuthencServiceUnavailable
                | CryptoError::InvalidAuthencTokenFormat { .. }
                | CryptoError::AuthencTokenExpired { .. }
        )
    }

    /// Check if this error requires reauthentication
    pub fn requires_reauthentication(&self) -> bool {
        matches!(
            self,
            CryptoError::AuthencTokenValidationFailed { .. }
                | CryptoError::AuthencAuthenticationFailed { .. }
                | CryptoError::AuthencTokenExpired { .. }
        )
    }

    /// Check if this is a role-based access control error
    pub fn is_rbac_related(&self) -> bool {
        matches!(
            self,
            CryptoError::RoleAccessDenied { .. }
                | CryptoError::InsufficientRolePrivileges { .. }
                | CryptoError::RoleScopeViolation { .. }
                | CryptoError::RoleAssignmentError { .. }
                | CryptoError::AdminLevelInsufficient { .. }
                | CryptoError::SatkerAccessViolation { .. }
                | CryptoError::CrossSatkerOperationDenied { .. }
        )
    }

    /// Check if this is a post-quantum cryptography error
    pub fn is_post_quantum_related(&self) -> bool {
        matches!(
            self,
            CryptoError::PostQuantumAlgorithmNotSupported { .. }
                | CryptoError::PostQuantumKeyGenerationFailed { .. }
                | CryptoError::PostQuantumSignatureVerificationFailed { .. }
                | CryptoError::PostQuantumEncryptionFailed { .. }
                | CryptoError::PostQuantumDecryptionFailed { .. }
                | CryptoError::HybridCryptoModeError { .. }
                | CryptoError::MlDsaOperationFailed { .. }
                | CryptoError::MlKemOperationFailed { .. }
                | CryptoError::PostQuantumMigrationError { .. }
        )
    }

    /// Check if this is a compliance-related error
    pub fn is_compliance_related(&self) -> bool {
        matches!(
            self,
            CryptoError::AuditTrailViolation { .. }
                | CryptoError::ComplianceRequirementNotMet { .. }
                | CryptoError::AuditLogIntegrityFailed { .. }
                | CryptoError::CompliancePolicyViolation { .. }
                | CryptoError::AuditRetentionPolicyViolation { .. }
                | CryptoError::ComplianceMonitoringFailed { .. }
                | CryptoError::AuditSignatureVerificationFailed { .. }
                | CryptoError::ComplianceCertificationExpired { .. }
        )
    }

    /// Check if this is a SIMKARI-specific error
    pub fn is_simkari_related(&self) -> bool {
        matches!(
            self,
            CryptoError::SatkerConfigurationInvalid { .. }
                | CryptoError::NipValidationFailed { .. }
                | CryptoError::KejaksaanHierarchyViolation { .. }
                | CryptoError::WilayahAccessRestriction { .. }
                | CryptoError::PusatLevelOperationRequired { .. }
        )
    }
    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            CryptoError::Internal(_)
            | CryptoError::KeyGenerationFailed(_)
            | CryptoError::ConfigurationError(_)
            | CryptoError::PostQuantumKeyGenerationFailed { .. }
            | CryptoError::AuditLogIntegrityFailed { .. }
            | CryptoError::ComplianceCertificationExpired { .. } => ErrorSeverity::Critical,

            CryptoError::PolicyViolation(_)
            | CryptoError::PermissionDenied(_)
            | CryptoError::ComplianceFailed(_)
            | CryptoError::IamPermissionDenied { .. }
            | CryptoError::RoleAccessDenied { .. }
            | CryptoError::InsufficientRolePrivileges { .. }
            | CryptoError::AdminLevelInsufficient { .. }
            | CryptoError::SatkerAccessViolation { .. }
            | CryptoError::CrossSatkerOperationDenied { .. }
            | CryptoError::AuditTrailViolation { .. }
            | CryptoError::ComplianceRequirementNotMet { .. }
            | CryptoError::CompliancePolicyViolation { .. }
            | CryptoError::KejaksaanHierarchyViolation { .. }
            | CryptoError::PusatLevelOperationRequired { .. } => ErrorSeverity::High,

            CryptoError::EncryptionFailed(_)
            | CryptoError::DecryptionFailed(_)
            | CryptoError::SigningFailed(_)
            | CryptoError::VerificationFailed(_)
            | CryptoError::AuthencTokenValidationFailed { .. }
            | CryptoError::AuthencAuthenticationFailed { .. }
            | CryptoError::PostQuantumSignatureVerificationFailed { .. }
            | CryptoError::PostQuantumEncryptionFailed { .. }
            | CryptoError::PostQuantumDecryptionFailed { .. }
            | CryptoError::MlDsaOperationFailed { .. }
            | CryptoError::MlKemOperationFailed { .. }
            | CryptoError::AuditSignatureVerificationFailed { .. } => ErrorSeverity::Medium,

            CryptoError::InvalidParameter(_)
            | CryptoError::InvalidUsage(_)
            | CryptoError::InvalidInput(_)
            | CryptoError::KeyNotFound(_)
            | CryptoError::InvalidAuthencTokenFormat { .. }
            | CryptoError::RoleScopeViolation { .. }
            | CryptoError::RoleAssignmentError { .. }
            | CryptoError::PostQuantumAlgorithmNotSupported { .. }
            | CryptoError::SatkerConfigurationInvalid { .. }
            | CryptoError::NipValidationFailed { .. }
            | CryptoError::WilayahAccessRestriction { .. } => ErrorSeverity::Low,

            _ => ErrorSeverity::Medium,
        }
    }

    /// Get error category for metrics and monitoring
    pub fn category(&self) -> ErrorCategory {
        match self {
            CryptoError::KeyAlreadyExists(_)
            | CryptoError::KeyNotFound(_)
            | CryptoError::KeyVersionNotFound(_)
            | CryptoError::KeyGenerationFailed(_)
            | CryptoError::KeyRotationFailed(_)
            | CryptoError::KeyDerivationFailed(_)
            | CryptoError::PostQuantumKeyGenerationFailed { .. } => ErrorCategory::KeyManagement,

            CryptoError::EncryptionFailed(_)
            | CryptoError::DecryptionFailed(_)
            | CryptoError::InvalidCiphertext(_)
            | CryptoError::PostQuantumEncryptionFailed { .. }
            | CryptoError::PostQuantumDecryptionFailed { .. }
            | CryptoError::MlKemOperationFailed { .. } => ErrorCategory::Encryption,

            CryptoError::SigningFailed(_)
            | CryptoError::VerificationFailed(_)
            | CryptoError::InvalidSignature(_)
            | CryptoError::PostQuantumSignatureVerificationFailed { .. }
            | CryptoError::MlDsaOperationFailed { .. }
            | CryptoError::AuditSignatureVerificationFailed { .. } => ErrorCategory::Signing,

            CryptoError::InvalidParameter(_)
            | CryptoError::InvalidKeyLength { .. }
            | CryptoError::InvalidNonceLength
            | CryptoError::InvalidAlgorithm(_)
            | CryptoError::InvalidInput(_)
            | CryptoError::InvalidUsage(_)
            | CryptoError::InvalidAuthencTokenFormat { .. }
            | CryptoError::PostQuantumAlgorithmNotSupported { .. }
            | CryptoError::NipValidationFailed { .. } => ErrorCategory::Validation,

            CryptoError::PolicyViolation(_)
            | CryptoError::PermissionDenied(_)
            | CryptoError::RateLimitExceeded(_)
            | CryptoError::IamPermissionDenied { .. }
            | CryptoError::RoleAccessDenied { .. }
            | CryptoError::InsufficientRolePrivileges { .. }
            | CryptoError::RoleScopeViolation { .. }
            | CryptoError::AdminLevelInsufficient { .. }
            | CryptoError::SatkerAccessViolation { .. }
            | CryptoError::CrossSatkerOperationDenied { .. }
            | CryptoError::KejaksaanHierarchyViolation { .. }
            | CryptoError::WilayahAccessRestriction { .. }
            | CryptoError::PusatLevelOperationRequired { .. } => ErrorCategory::Security,

            CryptoError::Internal(_)
            | CryptoError::ConfigurationError(_)
            | CryptoError::NetworkError(_)
            | CryptoError::StorageError(_)
            | CryptoError::AuthencCommunicationTimeout
            | CryptoError::AuthencServiceUnavailable
            | CryptoError::SatkerConfigurationInvalid { .. } => ErrorCategory::System,

            CryptoError::AuthencTokenValidationFailed { .. }
            | CryptoError::AuthencAuthenticationFailed { .. }
            | CryptoError::AuthencTokenExpired { .. }
            | CryptoError::RoleAssignmentError { .. } => ErrorCategory::Authentication,

            CryptoError::PostQuantumMigrationError { .. }
            | CryptoError::HybridCryptoModeError { .. } => ErrorCategory::PostQuantum,

            CryptoError::AuditTrailViolation { .. }
            | CryptoError::ComplianceRequirementNotMet { .. }
            | CryptoError::AuditLogIntegrityFailed { .. }
            | CryptoError::CompliancePolicyViolation { .. }
            | CryptoError::AuditRetentionPolicyViolation { .. }
            | CryptoError::ComplianceMonitoringFailed { .. }
            | CryptoError::ComplianceCertificationExpired { .. } => ErrorCategory::Compliance,

            _ => ErrorCategory::Other,
        }
    }
}
/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "LOW"),
            ErrorSeverity::Medium => write!(f, "MEDIUM"),
            ErrorSeverity::High => write!(f, "HIGH"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Error categories for monitoring and metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    KeyManagement,
    Encryption,
    Signing,
    Validation,
    Security,
    System,
    Authentication,
    PostQuantum,
    Compliance,
    Other,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::KeyManagement => write!(f, "KEY_MANAGEMENT"),
            ErrorCategory::Encryption => write!(f, "ENCRYPTION"),
            ErrorCategory::Signing => write!(f, "SIGNING"),
            ErrorCategory::Validation => write!(f, "VALIDATION"),
            ErrorCategory::Security => write!(f, "SECURITY"),
            ErrorCategory::System => write!(f, "SYSTEM"),
            ErrorCategory::Authentication => write!(f, "AUTHENTICATION"),
            ErrorCategory::PostQuantum => write!(f, "POST_QUANTUM"),
            ErrorCategory::Compliance => write!(f, "COMPLIANCE"),
            ErrorCategory::Other => write!(f, "OTHER"),
        }
    }
}

/// Error context for detailed error reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub operation: String,
    pub key_name: Option<String>,
    pub algorithm: Option<String>,
    pub user: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: Option<String>,
    pub additional_data: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: String) -> Self {
        Self {
            operation,
            key_name: None,
            algorithm: None,
            user: None,
            timestamp: chrono::Utc::now(),
            request_id: None,
            additional_data: std::collections::HashMap::new(),
        }
    }

    pub fn with_key_name(mut self, key_name: String) -> Self {
        self.key_name = Some(key_name);
        self
    }

    pub fn with_algorithm(mut self, algorithm: String) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    pub fn with_user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    pub fn add_data(mut self, key: String, value: String) -> Self {
        self.additional_data.insert(key, value);
        self
    }
}

/// Enhanced error type with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualError {
    pub error: CryptoError,
    pub context: ErrorContext,
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (operation: {}", self.error, self.context.operation)?;

        if let Some(ref key_name) = self.context.key_name {
            write!(f, ", key: {}", key_name)?;
        }

        if let Some(ref user) = self.context.user {
            write!(f, ", user: {}", user)?;
        }

        write!(f, ")")
    }
}

impl std::error::Error for ContextualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Convert from various error types
impl From<std::io::Error> for CryptoError {
    fn from(err: std::io::Error) -> Self {
        CryptoError::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for CryptoError {
    fn from(err: serde_json::Error) -> Self {
        CryptoError::SerializationError(err.to_string())
    }
}

impl From<base64::DecodeError> for CryptoError {
    fn from(err: base64::DecodeError) -> Self {
        CryptoError::InvalidParameter(format!("Base64 decode error: {}", err))
    }
}
// Helper constructor methods for CryptoError
impl CryptoError {
    // Authenc integration error constructors
    /// Create an authenc token validation failed error
    pub fn authenc_token_validation_failed<T: Into<String>>(reason: T) -> Self {
        Self::AuthencTokenValidationFailed {
            reason: reason.into(),
        }
    }

    /// Create an authenc authentication failed error
    pub fn authenc_authentication_failed<T: Into<String>>(message: T) -> Self {
        Self::AuthencAuthenticationFailed {
            message: message.into(),
        }
    }

    /// Create an IAM permission denied error
    pub fn iam_permission_denied<T: Into<String>>(operation: T) -> Self {
        Self::IamPermissionDenied {
            operation: operation.into(),
        }
    }

    /// Create an invalid authenc token format error
    pub fn invalid_authenc_token_format<T: Into<String>>(details: T) -> Self {
        Self::InvalidAuthencTokenFormat {
            details: details.into(),
        }
    }

    /// Create an authenc token expired error
    pub fn authenc_token_expired<T: Into<String>>(token_id: T) -> Self {
        Self::AuthencTokenExpired {
            token_id: token_id.into(),
        }
    }

    // Role-based access control error constructors
    /// Create a role access denied error
    pub fn role_access_denied<T: Into<String>>(role: T, resource: T) -> Self {
        Self::RoleAccessDenied {
            role: role.into(),
            resource: resource.into(),
        }
    }

    /// Create an insufficient role privileges error
    pub fn insufficient_role_privileges<T: Into<String>>(
        current_role: T,
        required_role: T,
    ) -> Self {
        Self::InsufficientRolePrivileges {
            current_role: current_role.into(),
            required_role: required_role.into(),
        }
    }

    /// Create a role scope violation error
    pub fn role_scope_violation<T: Into<String>>(role: T, scope: T, resource: T) -> Self {
        Self::RoleScopeViolation {
            role: role.into(),
            scope: scope.into(),
            resource: resource.into(),
        }
    }

    /// Create a role assignment error
    pub fn role_assignment_error<T: Into<String>>(role: T, user_nip: T) -> Self {
        Self::RoleAssignmentError {
            role: role.into(),
            user_nip: user_nip.into(),
        }
    }

    /// Create an admin level insufficient error
    pub fn admin_level_insufficient<T: Into<String>>(admin_level: T, operation: T) -> Self {
        Self::AdminLevelInsufficient {
            admin_level: admin_level.into(),
            operation: operation.into(),
        }
    }

    /// Create a satker access violation error
    pub fn satker_access_violation<T: Into<String>>(satker_code: T, target_satker: T) -> Self {
        Self::SatkerAccessViolation {
            satker_code: satker_code.into(),
            target_satker: target_satker.into(),
        }
    }

    /// Create a cross-satker operation denied error
    pub fn cross_satker_operation_denied<T: Into<String>>(
        operation: T,
        source_satker: T,
        target_satker: T,
    ) -> Self {
        Self::CrossSatkerOperationDenied {
            operation: operation.into(),
            source_satker: source_satker.into(),
            target_satker: target_satker.into(),
        }
    }

    // Post-quantum cryptography error constructors
    /// Create a post-quantum algorithm not supported error
    pub fn post_quantum_algorithm_not_supported<T: Into<String>>(algorithm: T) -> Self {
        Self::PostQuantumAlgorithmNotSupported {
            algorithm: algorithm.into(),
        }
    }

    /// Create a post-quantum key generation failed error
    pub fn post_quantum_key_generation_failed<T: Into<String>>(algorithm: T, reason: T) -> Self {
        Self::PostQuantumKeyGenerationFailed {
            algorithm: algorithm.into(),
            reason: reason.into(),
        }
    }

    /// Create a post-quantum signature verification failed error
    pub fn post_quantum_signature_verification_failed<T: Into<String>>(algorithm: T) -> Self {
        Self::PostQuantumSignatureVerificationFailed {
            algorithm: algorithm.into(),
        }
    }

    /// Create a post-quantum encryption failed error
    pub fn post_quantum_encryption_failed<T: Into<String>>(algorithm: T, reason: T) -> Self {
        Self::PostQuantumEncryptionFailed {
            algorithm: algorithm.into(),
            reason: reason.into(),
        }
    }

    /// Create a post-quantum decryption failed error
    pub fn post_quantum_decryption_failed<T: Into<String>>(algorithm: T, reason: T) -> Self {
        Self::PostQuantumDecryptionFailed {
            algorithm: algorithm.into(),
            reason: reason.into(),
        }
    }

    /// Create a hybrid crypto mode error
    pub fn hybrid_crypto_mode_error<T: Into<String>>(mode: T, details: T) -> Self {
        Self::HybridCryptoModeError {
            mode: mode.into(),
            details: details.into(),
        }
    }

    /// Create an ML-DSA operation failed error
    pub fn ml_dsa_operation_failed<T: Into<String>>(operation: T, reason: T) -> Self {
        Self::MlDsaOperationFailed {
            operation: operation.into(),
            reason: reason.into(),
        }
    }

    /// Create an ML-KEM operation failed error
    pub fn ml_kem_operation_failed<T: Into<String>>(operation: T, reason: T) -> Self {
        Self::MlKemOperationFailed {
            operation: operation.into(),
            reason: reason.into(),
        }
    }

    /// Create a post-quantum migration error
    pub fn post_quantum_migration_error<T: Into<String>>(
        from_algorithm: T,
        to_algorithm: T,
        reason: T,
    ) -> Self {
        Self::PostQuantumMigrationError {
            from_algorithm: from_algorithm.into(),
            to_algorithm: to_algorithm.into(),
            reason: reason.into(),
        }
    }

    // Audit and compliance error constructors
    /// Create an audit trail violation error
    pub fn audit_trail_violation<T: Into<String>>(violation_type: T, details: T) -> Self {
        Self::AuditTrailViolation {
            violation_type: violation_type.into(),
            details: details.into(),
        }
    }

    /// Create a compliance requirement not met error
    pub fn compliance_requirement_not_met<T: Into<String>>(requirement: T, context: T) -> Self {
        Self::ComplianceRequirementNotMet {
            requirement: requirement.into(),
            context: context.into(),
        }
    }

    /// Create an audit log integrity failed error
    pub fn audit_log_integrity_failed<T: Into<String>>(log_id: T) -> Self {
        Self::AuditLogIntegrityFailed {
            log_id: log_id.into(),
        }
    }

    /// Create a compliance policy violation error
    pub fn compliance_policy_violation<T: Into<String>>(policy: T, violation_details: T) -> Self {
        Self::CompliancePolicyViolation {
            policy: policy.into(),
            violation_details: violation_details.into(),
        }
    }

    /// Create an audit retention policy violation error
    pub fn audit_retention_policy_violation<T: Into<String>>(policy: T, resource: T) -> Self {
        Self::AuditRetentionPolicyViolation {
            policy: policy.into(),
            resource: resource.into(),
        }
    }

    /// Create a compliance monitoring failed error
    pub fn compliance_monitoring_failed<T: Into<String>>(monitor_type: T, reason: T) -> Self {
        Self::ComplianceMonitoringFailed {
            monitor_type: monitor_type.into(),
            reason: reason.into(),
        }
    }

    /// Create an audit signature verification failed error
    pub fn audit_signature_verification_failed<T: Into<String>>(audit_id: T) -> Self {
        Self::AuditSignatureVerificationFailed {
            audit_id: audit_id.into(),
        }
    }

    /// Create a compliance certification expired error
    pub fn compliance_certification_expired<T: Into<String>>(cert_type: T, expiry_date: T) -> Self {
        Self::ComplianceCertificationExpired {
            cert_type: cert_type.into(),
            expiry_date: expiry_date.into(),
        }
    }

    // SIMKARI-specific error constructors
    /// Create a satker configuration invalid error
    pub fn satker_configuration_invalid<T: Into<String>>(satker_code: T, issue: T) -> Self {
        Self::SatkerConfigurationInvalid {
            satker_code: satker_code.into(),
            issue: issue.into(),
        }
    }

    /// Create a NIP validation failed error
    pub fn nip_validation_failed<T: Into<String>>(nip: T, reason: T) -> Self {
        Self::NipValidationFailed {
            nip: nip.into(),
            reason: reason.into(),
        }
    }

    /// Create a kejaksaan hierarchy violation error
    pub fn kejaksaan_hierarchy_violation<T: Into<String>>(level: T, target_level: T) -> Self {
        Self::KejaksaanHierarchyViolation {
            level: level.into(),
            target_level: target_level.into(),
        }
    }

    /// Create a wilayah access restriction error
    pub fn wilayah_access_restriction<T: Into<String>>(wilayah: T, target_resource: T) -> Self {
        Self::WilayahAccessRestriction {
            wilayah: wilayah.into(),
            target_resource: target_resource.into(),
        }
    }

    /// Create a pusat-level operation required error
    pub fn pusat_level_operation_required<T: Into<String>>(operation: T) -> Self {
        Self::PusatLevelOperationRequired {
            operation: operation.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_properties() {
        let error = CryptoError::NetworkError("Connection failed".to_string());
        assert!(error.is_recoverable());
        assert!(error.should_retry());
        assert_eq!(error.severity(), ErrorSeverity::Medium);
        assert_eq!(error.category(), ErrorCategory::System);
    }

    #[test]
    fn test_authenc_related_errors() {
        let error = CryptoError::authenc_token_validation_failed("Invalid token");
        assert!(error.is_authenc_related());
        assert!(error.requires_reauthentication());
        assert_eq!(error.category(), ErrorCategory::Authentication);
    }

    #[test]
    fn test_rbac_related_errors() {
        let error = CryptoError::role_access_denied("user", "secret");
        assert!(error.is_rbac_related());
        assert_eq!(error.severity(), ErrorSeverity::High);
        assert_eq!(error.category(), ErrorCategory::Security);
    }

    #[test]
    fn test_post_quantum_errors() {
        let error = CryptoError::post_quantum_algorithm_not_supported("ML-DSA");
        assert!(error.is_post_quantum_related());
        assert_eq!(error.severity(), ErrorSeverity::Low);
        assert_eq!(error.category(), ErrorCategory::Validation);
    }

    #[test]
    fn test_compliance_errors() {
        let error = CryptoError::audit_trail_violation("missing signature", "audit log corrupted");
        assert!(error.is_compliance_related());
        assert_eq!(error.severity(), ErrorSeverity::High);
        assert_eq!(error.category(), ErrorCategory::Compliance);
    }

    #[test]
    fn test_simkari_errors() {
        let error = CryptoError::satker_configuration_invalid("SATKER001", "missing config");
        assert!(error.is_simkari_related());
        assert_eq!(error.severity(), ErrorSeverity::Low);
        assert_eq!(error.category(), ErrorCategory::System);
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("encrypt".to_string())
            .with_key_name("test-key".to_string())
            .with_user("alice".to_string())
            .add_data("size".to_string(), "1024".to_string());

        assert_eq!(context.operation, "encrypt");
        assert_eq!(context.key_name, Some("test-key".to_string()));
        assert_eq!(context.user, Some("alice".to_string()));
        assert!(context.additional_data.contains_key("size"));
    }

    #[test]
    fn test_contextual_error() {
        let error = CryptoError::EncryptionFailed("Bad key".to_string());
        let context =
            ErrorContext::new("encrypt".to_string()).with_key_name("test-key".to_string());

        let contextual_error = ContextualError { error, context };
        let error_string = contextual_error.to_string();

        assert!(error_string.contains("Encryption failed"));
        assert!(error_string.contains("operation: encrypt"));
        assert!(error_string.contains("key: test-key"));
    }
}
