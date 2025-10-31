//! Enhanced audit logging service with comprehensive context capture

use crate::database::Database;
use crate::error::Result;
use crate::models::events::{AdminEvent, Event};
use crate::services::audit_signature::AuditSignatureService;
use crate::utils::{
    geolocation::{GeolocationData, GeolocationService, SimpleGeolocationService},
    payload_sanitizer::{sanitize_payload, SanitizerConfig},
    request_context::RequestContext,
};
use axum::http::HeaderMap;
use serde_json::Value;
use std::sync::Arc;
use tracing::debug;

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

        if let Some(ref geo) = context.geolocation {
            if let Ok(geo_json) = serde_json::to_value(geo) {
                event.details.insert("geolocation".to_string(), geo_json.to_string());
            }
        }

        if let Some(ref ua) = context.request_context.user_agent {
            event.details.insert("user_agent".to_string(), ua.clone());
        }

        if let Some(ref req_payload) = context.request_payload {
            if let Ok(payload_str) = serde_json::to_string(req_payload) {
                event
                    .details
                    .insert("request_payload".to_string(), payload_str);
            }
        }

        if let Some(ref resp_payload) = context.response_payload {
            if let Ok(payload_str) = serde_json::to_string(resp_payload) {
                event
                    .details
                    .insert("response_payload".to_string(), payload_str);
            }
        }

        if let Some(ref req_id) = context.request_context.request_id {
            event
                .details
                .insert("correlation_id".to_string(), req_id.clone());
        }

        debug!(
            "Logging enhanced user event: type={}, user_id={:?}, ip={:?}",
            event.event_type.as_str(),
            event.user_id,
            event.ip_address
        );

        crate::database::audit_operations::store_event_with_signature(
            &self.db,
            &event,
            &self.signature_service,
        )
        .await
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

        if let Some(ref geo) = context.geolocation {
            if let Ok(geo_json) = serde_json::to_value(geo) {
                if let Some(ref mut repr) = event.representation {
                    if let Ok(mut repr_json) = serde_json::from_str::<Value>(repr) {
                        if let Some(obj) = repr_json.as_object_mut() {
                            obj.insert("geolocation".to_string(), geo_json);
                            if let Ok(updated) = serde_json::to_string(&repr_json) {
                                event.representation = Some(updated);
                            }
                        }
                    }
                }
            }
        }

        debug!(
            "Logging enhanced admin event: type={}, user_id={}, ip={:?}",
            event.operation_type.as_str(),
            event.auth_details.user_id,
            event.auth_details.ip_address
        );

        crate::database::audit_operations::store_admin_event_with_signature(
            &self.db,
            &event,
            &self.signature_service,
        )
        .await
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
    let ip_address = crate::utils::request_context::extract_ip_address(headers);
    let user_agent = crate::utils::request_context::extract_user_agent(headers);
    (ip_address, user_agent)
}
