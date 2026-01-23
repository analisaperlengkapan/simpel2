//! Helper functions to reduce code duplication across handlers.

use axum::http::HeaderMap;
use secreton_core::audit::{AuditLog, AuditStatus};
use std::collections::HashMap;

/// Extract client IP address from request headers.
/// Prioritizes X-Forwarded-For, then X-Real-IP.
pub fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
}

/// Create an audit log entry with common fields pre-filled.
/// This helper reduces duplication by providing a consistent way to create
/// audit logs across all handlers.
/// # Example
/// ```rust,no_run
/// use secreton_api::helpers::create_audit_log;
/// let log = create_audit_log(
///     "secret_created",
///     "john_doe",
///     "secret",
///     "app/database",
/// );
/// ```
pub fn create_audit_log(
    action: impl Into<String>,
    actor: impl Into<String>,
    resource_type: impl Into<String>,
    resource_id: impl Into<String>,
) -> AuditLog {
    AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: action.into(),
        actor: Some(actor.into()),
        resource_type: resource_type.into(),
        resource_id: resource_id.into(),
        status: AuditStatus::Success,
        ip: None,
        user_agent: None,
        namespace: None,
        metadata: HashMap::new(),
    }
}

/// Create an audit log entry with failure status.
pub fn create_audit_log_failure(
    action: impl Into<String>,
    actor: impl Into<String>,
    resource_type: impl Into<String>,
    resource_id: impl Into<String>,
    error: impl Into<String>,
) -> AuditLog {
    let mut metadata = HashMap::new();
    metadata.insert("error".to_string(), error.into());

    AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: action.into(),
        actor: Some(actor.into()),
        resource_type: resource_type.into(),
        resource_id: resource_id.into(),
        status: AuditStatus::Failure,
        ip: None,
        user_agent: None,
        namespace: None,
        metadata,
    }
}

/// Create an audit log with custom metadata.
pub fn create_audit_log_with_metadata(
    action: impl Into<String>,
    actor: impl Into<String>,
    resource_type: impl Into<String>,
    resource_id: impl Into<String>,
    metadata: HashMap<String, String>,
) -> AuditLog {
    AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: action.into(),
        actor: Some(actor.into()),
        resource_type: resource_type.into(),
        resource_id: resource_id.into(),
        status: AuditStatus::Success,
        ip: None,
        user_agent: None,
        namespace: None,
        metadata,
    }
}
