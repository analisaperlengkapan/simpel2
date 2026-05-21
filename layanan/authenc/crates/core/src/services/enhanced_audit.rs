//! Enhanced audit logging service with comprehensive context capture

use crate::services::audit_signature::AuditSignatureService;
use authenc_storage::Database;
use authenc_types::domain::events::{AdminEvent, Event};
use authenc_types::{AuthencError, Result};
use axum::http::HeaderMap;
use serde_json::Value;
use std::sync::Arc;
use tracing::debug;

/// Minimal request context extracted from HTTP headers
#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: uuid::Uuid,
}

impl RequestContext {
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let ip_address = headers
            .get("x-forwarded-for")
            .or_else(|| headers.get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or(s).trim().to_string());
        let user_agent = headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        Self {
            ip_address,
            user_agent,
            request_id: uuid::Uuid::new_v4(),
        }
    }
}

/// Geolocation data associated with an IP address
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeolocationData {
    pub country: Option<String>,
    pub city: Option<String>,
}

/// Trait for geolocation lookup services
#[async_trait::async_trait]
pub trait GeolocationService: Send + Sync {
    async fn lookup(&self, ip: &str) -> Option<GeolocationData>;
}

/// No-op geolocation service for environments without geo lookup
pub struct SimpleGeolocationService;
impl Default for SimpleGeolocationService {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleGeolocationService {
    pub fn new() -> Self {
        Self
    }
}
#[async_trait::async_trait]
impl GeolocationService for SimpleGeolocationService {
    async fn lookup(&self, _ip: &str) -> Option<GeolocationData> {
        None
    }
}

/// Configuration for sanitizing sensitive fields in payloads
#[derive(Debug, Clone, Default)]
pub struct SanitizerConfig {
    pub sensitive_fields: Vec<String>,
}

/// Sanitize a JSON payload by removing sensitive fields
pub fn sanitize_payload(payload: &Value, config: &SanitizerConfig) -> Value {
    let mut result = payload.clone();
    if let Some(obj) = result.as_object_mut() {
        for field in &config.sensitive_fields {
            obj.remove(field);
        }
    }
    result
}

/// Enhanced audit context with full request details
#[derive(Debug, Clone)]
pub struct EnhancedAuditContext {
    pub request_context: RequestContext,
    pub geolocation: Option<GeolocationData>,
    pub request_payload: Option<Value>,
    pub response_payload: Option<Value>,
    pub session_id: Option<String>,
}

impl EnhancedAuditContext {
    pub async fn from_headers(
        headers: &HeaderMap,
        geolocation_service: Option<&Arc<dyn GeolocationService>>,
    ) -> Self {
        let request_context = RequestContext::from_headers(headers);

        let geolocation = if let Some(service) = geolocation_service {
            if let Some(ref ip) = request_context.ip_address {
                service.lookup(ip).await
            } else {
                None
            }
        } else {
            None
        };

        Self {
            request_context,
            geolocation,
            request_payload: None,
            response_payload: None,
            session_id: None,
        }
    }

    pub fn with_request_payload(mut self, payload: Value, config: &SanitizerConfig) -> Self {
        self.request_payload = Some(sanitize_payload(&payload, config));
        self
    }

    pub fn with_response_payload(mut self, payload: Value, config: &SanitizerConfig) -> Self {
        self.response_payload = Some(sanitize_payload(&payload, config));
        self
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// Enhanced audit logging service
pub struct EnhancedAuditService {
    db: Arc<Database>,
    signature_service: Arc<AuditSignatureService>,
    geolocation_service: Arc<dyn GeolocationService>,
    sanitizer_config: SanitizerConfig,
}

impl EnhancedAuditService {
    pub fn new(
        db: Arc<Database>,
        signature_service: Arc<AuditSignatureService>,
        geolocation_service: Option<Arc<dyn GeolocationService>>,
        sanitizer_config: Option<SanitizerConfig>,
    ) -> Self {
        Self {
            db,
            signature_service,
            geolocation_service: geolocation_service.unwrap_or_else(|| {
                Arc::new(SimpleGeolocationService::new()) as Arc<dyn GeolocationService>
            }),
            sanitizer_config: sanitizer_config.unwrap_or_default(),
        }
    }

    pub async fn log_user_event(
        &self,
        mut event: Event,
        context: &EnhancedAuditContext,
    ) -> Result<()> {
        if let Some(ref ip) = context.request_context.ip_address {
            event.ip_address = Some(ip.clone());
        }

        if let Some(ref session_id) = context.session_id {
            event.session_id = Some(session_id.clone());
        }

        if let Some(ref geo) = context.geolocation
            && let Ok(geo_json) = serde_json::to_value(geo)
        {
            event
                .details
                .insert("geolocation".to_string(), geo_json.to_string());
        }

        if let Some(ref ua) = context.request_context.user_agent {
            event.details.insert("user_agent".to_string(), ua.clone());
        }

        if let Some(ref req_payload) = context.request_payload
            && let Ok(payload_str) = serde_json::to_string(req_payload)
        {
            event
                .details
                .insert("request_payload".to_string(), payload_str);
        }

        if let Some(ref resp_payload) = context.response_payload
            && let Ok(payload_str) = serde_json::to_string(resp_payload)
        {
            event
                .details
                .insert("response_payload".to_string(), payload_str);
        }

        event.details.insert(
            "correlation_id".to_string(),
            context.request_context.request_id.to_string(),
        );

        debug!(
            "Logging enhanced user event: type={}, user_id={:?}, ip={:?}",
            event.event_type.as_str(),
            event.user_id,
            event.ip_address
        );

        // TODO: Implement via authenc_storage audit operations when available
        let _ = (&self.db, &self.signature_service);
        Err(AuthencError::internal(
            "Enhanced audit event storage not yet implemented",
        ))
    }

    pub async fn log_admin_event(
        &self,
        mut event: AdminEvent,
        context: &EnhancedAuditContext,
    ) -> Result<()> {
        if let Some(ref ip) = context.request_context.ip_address {
            event.auth_details.ip_address = Some(ip.clone());
        }

        if let Some(ref ua) = context.request_context.user_agent {
            event.auth_details.user_agent = Some(ua.clone());
        }

        if let Some(ref geo) = context.geolocation
            && let Ok(geo_json) = serde_json::to_value(geo)
            && let Some(ref mut repr) = event.representation
            && let Ok(mut repr_json) = serde_json::from_str::<Value>(repr)
            && let Some(obj) = repr_json.as_object_mut()
        {
            obj.insert("geolocation".to_string(), geo_json);
            if let Ok(updated) = serde_json::to_string(&repr_json) {
                event.representation = Some(updated);
            }
        }

        debug!(
            "Logging enhanced admin event: type={}, user_id={}, ip={:?}",
            event.operation_type.as_str(),
            event.auth_details.user_id,
            event.auth_details.ip_address
        );

        // TODO: Implement via authenc_storage audit operations when available
        let _ = (&self.db, &self.signature_service);
        Err(AuthencError::internal(
            "Enhanced audit admin event storage not yet implemented",
        ))
    }

    pub async fn create_context(&self, headers: &HeaderMap) -> EnhancedAuditContext {
        EnhancedAuditContext::from_headers(headers, Some(&self.geolocation_service)).await
    }

    pub fn sanitizer_config(&self) -> &SanitizerConfig {
        &self.sanitizer_config
    }
}

pub async fn create_audit_context(
    headers: &HeaderMap,
    session_id: Option<String>,
) -> EnhancedAuditContext {
    let mut context = EnhancedAuditContext::from_headers(headers, None).await;
    if let Some(sid) = session_id {
        context = context.with_session_id(sid);
    }
    context
}

pub fn extract_audit_details(headers: &HeaderMap) -> (Option<String>, Option<String>) {
    let ctx = RequestContext::from_headers(headers);
    (ctx.ip_address, ctx.user_agent)
}
