//! Audit logging for vault operations
//! Tracks all secret access, modifications, and deletions for compliance

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Audit event type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    /// Secret created
    SecretCreated,
    /// Secret read/accessed
    SecretRead,
    /// Secret updated
    SecretUpdated,
    /// Secret deleted
    SecretDeleted,
    /// Secret rotated
    SecretRotated,
    /// Secret listed
    SecretListed,
    /// Health check
    HealthCheck,
    /// Authentication attempt
    AuthAttempt,
    /// Authorization check
    AuthzCheck,
}

/// Audit event entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: AuditEventType,
    /// User/client identifier
    pub principal: String,
    /// Realm/tenant
    pub realm: Option<String>,
    /// Secret key (masked for security)
    pub secret_key: Option<String>,
    /// Operation result (success/failure)
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Client IP address
    pub client_ip: Option<String>,
    /// Request ID for tracing
    pub request_id: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(event_type: AuditEventType, principal: String, success: bool) -> Self {
        Self {
            timestamp: Utc::now(),
            event_type,
            principal,
            realm: None,
            secret_key: None,
            success,
            error: None,
            client_ip: None,
            request_id: None,
        }
    }

    /// Add realm information
    pub fn with_realm(mut self, realm: String) -> Self {
        self.realm = Some(realm);
        self
    }

    /// Add secret key (masked for security)
    pub fn with_secret_key(mut self, key: String) -> Self {
        // Mask the key for security (show only first 3 chars)
        let masked = if key.len() > 3 {
            format!("{}***", &key[..3])
        } else {
            "***".to_string()
        };
        self.secret_key = Some(masked);
        self
    }

    /// Add error information
    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }

    /// Add client IP
    pub fn with_client_ip(mut self, ip: String) -> Self {
        self.client_ip = Some(ip);
        self
    }

    /// Add request ID
    pub fn with_request_id(mut self, id: String) -> Self {
        self.request_id = Some(id);
        self
    }
}

/// Audit logger for vault operations (API-specific wrapper)
#[derive(Clone)]
pub struct AuditLogger {
    /// Core audit logger
    #[allow(dead_code)]
    core_logger: Arc<crate::audit::AuditLogger>,
    /// In-memory event buffer for API-specific events
    events: Arc<RwLock<VecDeque<AuditEvent>>>,
    /// Maximum events to keep in memory
    max_events: usize,
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(10000)
    }
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(max_events: usize) -> Self {
        // Create core logger with memory backend
        let core_logger = crate::audit::AuditLogger::new(vec![Arc::new(
            crate::audit::MemoryBackend::default(),
        )]);

        Self {
            core_logger: Arc::new(core_logger),
            events: Arc::new(RwLock::new(VecDeque::with_capacity(max_events))),
            max_events,
        }
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) {
        // Log to tracing
        if event.success {
            info!(
                event_type = ?event.event_type,
                principal = %event.principal,
                realm = ?event.realm,
                secret_key = ?event.secret_key,
                "Audit event"
            );
        } else {
            warn!(
                event_type = ?event.event_type,
                principal = %event.principal,
                realm = ?event.realm,
                secret_key = ?event.secret_key,
                error = ?event.error,
                "Audit event failed"
            );
        }

        // Store in memory buffer
        let mut events = self.events.write().await;

        // Remove oldest if at capacity
        if events.len() >= self.max_events {
            events.pop_front();
        }

        events.push_back(event);
    }

    /// Get recent audit events
    pub async fn get_recent(&self, count: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(count).cloned().collect()
    }

    /// Get events by principal
    pub async fn get_by_principal(&self, principal: &str, count: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .rev()
            .filter(|e| e.principal == principal)
            .take(count)
            .cloned()
            .collect()
    }

    /// Get events by realm
    pub async fn get_by_realm(&self, realm: &str, count: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .rev()
            .filter(|e| e.realm.as_ref().is_some_and(|r| r == realm))
            .take(count)
            .cloned()
            .collect()
    }

    /// Get events by type
    pub async fn get_by_type(&self, event_type: &AuditEventType, count: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .rev()
            .filter(|e| &e.event_type == event_type)
            .take(count)
            .cloned()
            .collect()
    }

    /// Get failed events
    pub async fn get_failed(&self, count: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .rev()
            .filter(|e| !e.success)
            .take(count)
            .cloned()
            .collect()
    }

    /// Get total event count
    pub async fn count(&self) -> usize {
        let events = self.events.read().await;
        events.len()
    }

    /// Clear all events (use with caution)
    pub async fn clear(&self) {
        let mut events = self.events.write().await;
        events.clear();
        info!("Audit log cleared");
    }

    /// Clean expired events from the buffer
    pub async fn cleanup_expired_events(&self, retention: Duration) -> usize {
        let cutoff = Utc::now() - retention;
        let mut events = self.events.write().await;
        let initial_len = events.len();

        // Retain only events newer than cutoff
        events.retain(|event| event.timestamp >= cutoff);

        let removed = initial_len - events.len();
        if removed > 0 {
            info!("Cleaned up {} expired audit events", removed);
        }
        removed
    }
}
