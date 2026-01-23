//! Axum middleware for audit logging
//!
//! # ⚠️ DEPRECATED MODULE
//!
//! **This module has been moved to `secreton-api` crate.**
//!
//! # Compilation
//!
//! This module requires the `legacy-axum-middleware` feature to compile.
//! It is disabled by default to remove web framework dependencies from core.

#![cfg(feature = "axum-middleware")]
//!
//! ## Why Deprecated?
//!
//! This middleware uses Axum framework types (Request, Response, Body, Next),
//! creating a dependency from core → axum. This violates clean architecture:
//! - **Core crate** should be framework-agnostic (domain logic only)
//! - **API crate** should contain framework-specific code (presentation logic)
//!
//! ## Migration Path
//!
//! **Before**:
//! ```rust,ignore
//! use secreton_core::audit::{audit_middleware, AuditExt};
//!
//! let app = Router::new()
//!     .layer(middleware::from_fn(audit_middleware));
//! ```
//!
//! **After**:
//! ```rust,ignore
//! use secreton_api::middleware::audit_middleware;
//! use secreton_core::audit::{AuditLogger, AuditLog}; // Domain types stay in core
//!
//! let app = Router::new()
//!     .layer(middleware::from_fn(audit_middleware));
//! ```
//!
//! ## What Stays in Core?
//!
//! - `AuditLogger` trait and implementations
//! - `AuditLog` data structure
//! - `AuditBackend` trait
//! - Backend implementations (PostgreSQL, Syslog, etc.)
//!
//! ## What Moved to API?
//!
//! - Axum middleware functions
//! - Request/Response extensions
//! - HTTP-specific audit logic
//!
//! ## Removal Timeline
//!
//! - **v1.1.0**: Deprecated (current)
//! - **v1.2.0**: Copy moved to api crate
//! - **v2.0.0**: This file removed from core

use axum::{body::Body, http::Request, middleware::Next, response::Response};
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

// TODO: Re-enable when auth module is implemented
// use crate::auth::auth_impl::Claims;

// Placeholder until auth module is implemented
#[derive(Debug, Clone)]
pub struct Claims {
    pub sub: String,
}

use super::*;

/// Extension trait for adding audit logging to requests
#[async_trait::async_trait]
pub trait AuditExt {
    /// Log an audit event for this request
    async fn audit_log(
        &self,
        action: impl Into<String> + Send,
        resource_type: impl Into<String> + Send,
        resource_id: impl Into<String> + Send,
        status: AuditStatus,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), AuditError>;
}

#[async_trait::async_trait]
impl<B> AuditExt for Request<B>
where
    B: Send + Sync + 'static,
{
    async fn audit_log(
        &self,
        action: impl Into<String> + Send,
        resource_type: impl Into<String> + Send,
        resource_id: impl Into<String> + Send,
        status: AuditStatus,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), AuditError> {
        let extensions = self.extensions();
        let logger = extensions.get::<AuditLogger>().ok_or_else(|| {
            AuditError::LoggingError("AuditLogger not found in request extensions".into())
        })?;

        let request_id = extensions
            .get::<String>()
            .cloned()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let claims = extensions.get::<Claims>();

        let mut metadata = metadata.unwrap_or_default();
        metadata.insert("request_id".into(), request_id);

        let entry = AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: action.into(),
            actor: claims.map(|c| c.sub.clone()),
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            status,
            ip: self
                .headers()
                .get("x-forwarded-for")
                .or_else(|| self.headers().get("x-real-ip"))
                .and_then(|h| h.to_str().ok())
                .map(|s| s.split(',').next().unwrap_or(s).trim().to_string()),
            user_agent: self
                .headers()
                .get("user-agent")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
            namespace: extract_namespace_from_path(self.uri().path()),
            metadata,
        };

        logger.log(entry).await
    }
}

/// Extract namespace from request path
/// Namespaces are typically in paths like:
/// - /v1/sys/namespaces/{ns}/...
/// - /{ns}/secrets/...
fn extract_namespace_from_path(path: &str) -> Option<String> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // Pattern 1: /v1/sys/namespaces/{ns}/...
    if segments.len() >= 4
        && segments[0] == "v1"
        && segments[1] == "sys"
        && segments[2] == "namespaces"
    {
        return Some(segments[3].to_string());
    }

    // Pattern 2: /{ns}/secrets/... or /{ns}/transit/...
    if segments.len() >= 2 {
        let second = segments[1];
        if second == "secrets" || second == "transit" || second == "keys" || second == "policies" {
            return Some(segments[0].to_string());
        }
    }

    None
}

/// Middleware for request logging
pub async fn audit_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, std::convert::Infallible> {
    let start = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    // Log the request
    if let Some(logger) = response.extensions().get::<AuditLogger>() {
        let status_code = status.as_u16();
        let _success = status_code < 400;
        let status = if status_code >= 500 {
            AuditStatus::Failure
        } else if status_code >= 400 {
            AuditStatus::Denied
        } else {
            AuditStatus::Success
        };

        let mut metadata = HashMap::new();
        metadata.insert("method".into(), method.to_string());
        metadata.insert("path".into(), path.clone());
        metadata.insert("status".into(), status_code.to_string());
        metadata.insert("duration_ms".into(), duration.as_millis().to_string());

        // Note: Body size extraction removed due to axum 0.8 API changes
        // This is deprecated middleware that will be removed in v2.0.0

        // Log the request
        let namespace = extract_namespace_from_path(&path);
        let _ = logger
            .log(AuditLog {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                action: format!("http_{}", method).to_lowercase(),
                actor: None, // Will be set by the auth middleware if available
                resource_type: "http_request".into(),
                resource_id: path,
                status,
                ip: None,         // Will be set by the AuditExt
                user_agent: None, // Will be set by the AuditExt
                namespace,
                metadata,
            })
            .await;
    }

    Ok(response)
}
