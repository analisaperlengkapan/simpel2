//! Example handler demonstrating enhanced audit logging usage
//!
//! This module provides examples of how to use the enhanced audit logging
//! service in handlers to capture comprehensive context.

use authenc::error::Result;
use authenc::models::events::{Event, EventType};
use authenc::services::enhanced_audit::{EnhancedAuditService, create_audit_context};
use authenc::utils::payload_sanitizer::{SanitizerConfig, sanitize_payload};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

/// Example request payload
#[derive(Debug, Deserialize)]
pub struct ExampleRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

/// Example response payload
#[derive(Debug, Serialize)]
pub struct ExampleResponse {
    pub success: bool,
    pub message: String,
    pub user_id: Option<String>,
}

/// Example handler demonstrating enhanced audit logging
/// This shows how to:
/// 1. Extract request context from headers
/// 2. Sanitize request/response payloads
/// 3. Log events with full context including geolocation
pub async fn example_audit_handler(
    State(audit_service): State<Arc<EnhancedAuditService>>,
    headers: HeaderMap,
    Json(request): Json<ExampleRequest>,
) -> Result<impl IntoResponse> {
    // Step 1: Create audit context from headers
    // This automatically extracts IP, user agent, request ID, and geolocation
    let mut audit_context = audit_service.create_context(&headers).await;

    // Step 2: Add sanitized request payload to context
    let request_json = json!({
        "username": request.username,
        "password": request.password,
        "email": request.email,
    });
    audit_context =
        audit_context.with_request_payload(request_json, audit_service.sanitizer_config());

    // Step 3: Perform business logic
    let user_id = Uuid::new_v4();
    let response = ExampleResponse {
        success: true,
        message: "Operation completed successfully".to_string(),
        user_id: Some(user_id.to_string()),
    };

    // Step 4: Add sanitized response payload to context
    let response_json = serde_json::to_value(&response).unwrap();
    audit_context =
        audit_context.with_response_payload(response_json, audit_service.sanitizer_config());

    // Step 5: Add session ID if available (from session management)
    // audit_context = audit_context.with_session_id("session-123".to_string());

    // Step 6: Create and log the audit event
    let event = Event {
        id: Uuid::new_v4().to_string(),
        time: Utc::now(),
        event_type: EventType::Login,
        realm_id: "default".to_string(),
        realm_name: Some("Default Realm".to_string()),
        client_id: Some("example-client".to_string()),
        user_id: Some(user_id.to_string()),
        session_id: None, // Will be set from context
        ip_address: None, // Will be set from context
        error: None,
        details: std::collections::HashMap::new(),
    };

    // Log the event with enhanced context
    audit_service.log_user_event(event, &audit_context).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// Simplified helper for common audit logging patterns
/// This demonstrates a more concise approach for simple cases
pub async fn simple_audit_example(headers: HeaderMap, session_id: Option<String>) -> Result<()> {
    // Create audit context with minimal boilerplate
    let audit_context = create_audit_context(&headers, session_id).await;

    // Use the context for logging
    // The context now contains:
    // - IP address (from X-Forwarded-For, X-Real-IP, etc.)
    // - User agent
    // - Request ID (for correlation)
    // - Session ID (if provided)

    // Example: Extract details for manual logging
    let ip = audit_context.request_context.ip_address;
    let user_agent = audit_context.request_context.user_agent;

    tracing::info!(
        ip = ?ip,
        user_agent = ?user_agent,
        "Audit context extracted"
    );

    Ok(())
}

/// Example of payload sanitization
pub fn example_payload_sanitization() {
    let config = SanitizerConfig::default();

    // Example payload with sensitive data
    let payload = json!({
        "username": "testuser",
        "password": "secret123",
        "email": "user@example.com",
        "api_key": "sk_live_abc123",
        "profile": {
            "name": "Test User",
            "phone": "+1234567890"
        }
    });

    // Sanitize the payload
    let sanitized = sanitize_payload(&payload, &config);

    // Result will have:
    // - password: "[REDACTED]"
    // - api_key: "[REDACTED]"
    // - email: "u***@example.com" (masked)
    // - phone: "+***" (masked)
    // - Other fields preserved

    println!(
        "Sanitized payload: {}",
        serde_json::to_string_pretty(&sanitized).unwrap()
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    use axum::http::HeaderMap;

    let headers = HeaderMap::new();
    // Run a simple example to ensure the audit helpers work
    simple_audit_example(headers, None).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[tokio::test]
    async fn test_audit_context_creation() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.1"));
        headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0"));

        let context = create_audit_context(&headers, Some("session-123".to_string())).await;

        assert_eq!(
            context.request_context.ip_address,
            Some("203.0.113.1".to_string())
        );
        assert_eq!(
            context.request_context.user_agent,
            Some("Mozilla/5.0".to_string())
        );
        assert_eq!(context.session_id, Some("session-123".to_string()));
    }

    #[test]
    fn test_payload_sanitization_example() {
        let config = SanitizerConfig::default();

        let payload = json!({
            "username": "testuser",
            "password": "secret123"
        });

        let sanitized = sanitize_payload(&payload, &config);

        assert_eq!(sanitized["username"], json!("testuser"));
        assert_eq!(sanitized["password"], json!("[REDACTED]"));
    }
}
