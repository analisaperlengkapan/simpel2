//! MFA Audit Logger
//!
//! Specialized audit logging for MFA operations with enhanced security context
//! and correlation tracking for forensic analysis.

use crate::error::{AuthencError, Result};
use crate::models::events::EventType;
use crate::services::events::EventBuilder;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use lib_common::correlation::CorrelationId;

/// MFA audit event context with enhanced security information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaAuditContext {
    /// User ID involved in the MFA operation
    pub user_id: Uuid,
    /// Session ID if available
    pub session_id: Option<String>,
    /// Client IP address
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// MFA operation type
    pub operation: MfaOperation,
    /// Success or failure status
    pub success: bool,
    /// Error message if operation failed
    pub error_message: Option<String>,
    /// Additional security metadata
    pub security_metadata: HashMap<String, String>,
    /// Correlation ID for tracking related events
    pub correlation_id: String,
}

/// Types of MFA operations for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MfaOperation {
    /// MFA setup initiated
    SetupInitiated,
    /// MFA setup completed successfully
    SetupCompleted,
    /// MFA setup failed
    SetupFailed,
    /// MFA verification attempted
    VerificationAttempted,
    /// MFA verification succeeded
    VerificationSucceeded,
    /// MFA verification failed
    VerificationFailed,
    /// MFA disabled for user
    Disabled,
    /// MFA reset by administrator
    Reset,
    /// Backup code used
    BackupCodeUsed,
    /// Suspicious MFA activity detected
    SuspiciousActivity,
}

impl MfaOperation {
    /// Convert to event type for standard event logging
    pub fn to_event_type(&self) -> EventType {
        match self {
            MfaOperation::SetupInitiated | MfaOperation::SetupCompleted => EventType::MfaSetup,
            MfaOperation::SetupFailed => EventType::MfaSetupError,
            MfaOperation::VerificationAttempted | MfaOperation::VerificationSucceeded => {
                EventType::MfaVerification
            }
            MfaOperation::VerificationFailed => EventType::MfaVerificationError,
            MfaOperation::Disabled => EventType::MfaDisabled,
            MfaOperation::Reset => EventType::MfaDisabled, // Use same event type for reset
            MfaOperation::BackupCodeUsed => EventType::MfaVerification,
            MfaOperation::SuspiciousActivity => EventType::MfaVerificationError,
        }
    }

    /// Get operation description for audit logs
    pub fn description(&self) -> &'static str {
        match self {
            MfaOperation::SetupInitiated => "MFA setup initiated",
            MfaOperation::SetupCompleted => "MFA setup completed successfully",
            MfaOperation::SetupFailed => "MFA setup failed",
            MfaOperation::VerificationAttempted => "MFA verification attempted",
            MfaOperation::VerificationSucceeded => "MFA verification succeeded",
            MfaOperation::VerificationFailed => "MFA verification failed",
            MfaOperation::Disabled => "MFA disabled for user",
            MfaOperation::Reset => "MFA reset by administrator",
            MfaOperation::BackupCodeUsed => "MFA backup code used",
            MfaOperation::SuspiciousActivity => "Suspicious MFA activity detected",
        }
    }
}

/// MFA audit logger for enhanced security event tracking
pub struct MfaAuditLogger {
    /// Event manager for firing audit events
    event_manager: Arc<tokio::sync::RwLock<crate::services::events::EventManager>>,
    /// Realm ID for events
    realm_id: String,
}

impl MfaAuditLogger {
    /// Create a new MFA audit logger
    pub fn new(
        event_manager: Arc<tokio::sync::RwLock<crate::services::events::EventManager>>,
        realm_id: String,
    ) -> Self {
        Self {
            event_manager,
            realm_id,
        }
    }

    /// Log MFA audit event with enhanced context
    pub async fn log_mfa_event(&self, context: MfaAuditContext) -> Result<()> {
        let mut event_builder =
            EventBuilder::new(context.operation.to_event_type(), self.realm_id.clone())
                .user_id(context.user_id.to_string())
                .client_id("api".to_string())
                .detail("operation", context.operation.description())
                .detail("success", context.success.to_string())
                .detail("correlation_id", context.correlation_id.clone());

        // Add session ID if available
        if let Some(session_id) = &context.session_id {
            event_builder = event_builder.session_id(session_id.clone());
        }

        // Add IP address if available
        if let Some(ip_address) = &context.ip_address {
            event_builder = event_builder.ip_address(ip_address.clone());
        }

        // Add user agent as detail
        if let Some(user_agent) = &context.user_agent {
            event_builder = event_builder.detail("user_agent", user_agent.clone());
        }

        // Add error message if operation failed
        if let Some(error_message) = &context.error_message {
            event_builder = event_builder.error(error_message.clone());
        }

        // Add all security metadata as details
        for (key, value) in &context.security_metadata {
            event_builder = event_builder.detail(format!("security_{}", key), value.clone());
        }

        // Add timestamp details for forensic analysis
        event_builder = event_builder
            .detail("timestamp_iso", Utc::now().to_rfc3339())
            .detail("timestamp_unix", Utc::now().timestamp().to_string());

        let event = event_builder.build();

        // Fire the event through the event manager
        if let Err(e) = self.event_manager.write().await.fire_event(event).await {
            tracing::error!("Failed to fire MFA audit event: {}", e);
            return Err(AuthencError::internal(format!(
                "Failed to log MFA audit event: {}",
                e
            )));
        }

        // Log to application logs as well for immediate visibility
        match context.operation {
            MfaOperation::SetupCompleted => {
                tracing::info!(
                    user_id = %context.user_id,
                    correlation_id = %context.correlation_id,
                    ip_address = ?context.ip_address,
                    "MFA setup completed successfully"
                );
            }
            MfaOperation::VerificationSucceeded => {
                tracing::info!(
                    user_id = %context.user_id,
                    correlation_id = %context.correlation_id,
                    ip_address = ?context.ip_address,
                    "MFA verification succeeded"
                );
            }
            MfaOperation::VerificationFailed => {
                tracing::warn!(
                    user_id = %context.user_id,
                    correlation_id = %context.correlation_id,
                    ip_address = ?context.ip_address,
                    error = ?context.error_message,
                    "MFA verification failed"
                );
            }
            MfaOperation::SuspiciousActivity => {
                tracing::error!(
                    user_id = %context.user_id,
                    correlation_id = %context.correlation_id,
                    ip_address = ?context.ip_address,
                    security_metadata = ?context.security_metadata,
                    "Suspicious MFA activity detected"
                );
            }
            _ => {
                tracing::info!(
                    user_id = %context.user_id,
                    correlation_id = %context.correlation_id,
                    operation = ?context.operation,
                    success = context.success,
                    "MFA audit event logged"
                );
            }
        }

        Ok(())
    }

    /// Log MFA setup event
    pub async fn log_setup_event(
        &self,
        user_id: Uuid,
        success: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
        error_message: Option<String>,
    ) -> Result<()> {
        let operation = if success {
            MfaOperation::SetupCompleted
        } else {
            MfaOperation::SetupFailed
        };

        let context = MfaAuditContext {
            user_id,
            session_id: None,
            ip_address,
            user_agent,
            operation,
            success,
            error_message,
            security_metadata: HashMap::new(),
            correlation_id: CorrelationId::new().to_string(),
        };

        self.log_mfa_event(context).await
    }

    /// Log MFA verification event
    pub async fn log_verification_event(
        &self,
        user_id: Uuid,
        success: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
        session_id: Option<String>,
        error_message: Option<String>,
        security_metadata: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let operation = if success {
            MfaOperation::VerificationSucceeded
        } else {
            MfaOperation::VerificationFailed
        };

        let context = MfaAuditContext {
            user_id,
            session_id,
            ip_address,
            user_agent,
            operation,
            success,
            error_message,
            security_metadata: security_metadata.unwrap_or_default(),
            correlation_id: CorrelationId::new().to_string(),
        };

        self.log_mfa_event(context).await
    }

    /// Log suspicious MFA activity
    pub async fn log_suspicious_activity(
        &self,
        user_id: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
        session_id: Option<String>,
        reason: String,
        security_metadata: HashMap<String, String>,
    ) -> Result<()> {
        let mut metadata = security_metadata;
        metadata.insert("suspicious_reason".to_string(), reason);

        let context = MfaAuditContext {
            user_id,
            session_id,
            ip_address,
            user_agent,
            operation: MfaOperation::SuspiciousActivity,
            success: false,
            error_message: Some("Suspicious MFA activity detected".to_string()),
            security_metadata: metadata,
            correlation_id: CorrelationId::new().to_string(),
        };

        self.log_mfa_event(context).await
    }

    /// Generate correlation ID for tracking related MFA events
    pub fn generate_correlation_id() -> String {
        CorrelationId::new().to_string()
    }
}

/// Helper function to create MFA audit logger from app state
pub fn create_mfa_audit_logger(
    event_manager: Arc<tokio::sync::RwLock<crate::services::events::EventManager>>,
    realm_id: Option<String>,
) -> MfaAuditLogger {
    MfaAuditLogger::new(
        event_manager,
        realm_id.unwrap_or_else(|| "master".to_string()),
    )
}
