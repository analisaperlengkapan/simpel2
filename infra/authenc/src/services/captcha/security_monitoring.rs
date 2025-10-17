//! CAPTCHA Security Monitoring Integration
//!
//! Integrates CAPTCHA validation with authenc security monitoring middleware
//! to provide comprehensive audit logging and bot detection alerts

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::middleware::security_monitoring_axum::SecurityMonitoringConfig;
use crate::models::audit_log::AuditLog;
use crate::services::pg_audit_log_store::PgAuditLogStore;
use super::error::CaptchaError;
use super::types::{BehaviorClassification, Challenge, RiskLevel, ValidationResult};

/// CAPTCHA security monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaSecurityConfig {
    /// Base security monitoring configuration
    pub base_config: SecurityMonitoringConfig,
    /// Whether to log all CAPTCHA attempts
    pub log_all_attempts: bool,
    /// Whether to log successful validations
    pub log_successful_validations: bool,
    /// Whether to log failed validations
    pub log_failed_validations: bool,
    /// Whether to log bot detection events
    pub log_bot_detections: bool,
    /// Threshold for bot detection alerts (bot probability)
    pub bot_alert_threshold: f64,
    /// Threshold for suspicious activity alerts (attempts per minute)
    pub suspicious_activity_threshold: u32,
    /// Whether to enable real-time alerting
    pub enable_real_time_alerts: bool,
}

impl Default for CaptchaSecurityConfig {
    fn default() -> Self {
        Self {
            base_config: SecurityMonitoringConfig {
                enabled: true,
                suspicious_threshold_rpm: 50,
                monitored_paths: vec![
                    "/api/v1/captcha/challenge".to_string(),
                    "/api/v1/captcha/validate".to_string(),
                    "/captcha/generate".to_string(),
                    "/captcha/verify".to_string(),
                ],
                log_auth_attempts: true,
                log_authz_failures: true,
            },
            log_all_attempts: true,
            log_successful_validations: true,
            log_failed_validations: true,
            log_bot_detections: true,
            bot_alert_threshold: 0.8,
            suspicious_activity_threshold: 20,
            enable_real_time_alerts: true,
        }
    }
}

/// CAPTCHA security event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptchaSecurityEvent {
    /// A new CAPTCHA challenge was generated
    ChallengeGenerated,
    /// A CAPTCHA challenge was successfully validated
    ChallengeValidated,
    /// A CAPTCHA validation attempt failed
    ValidationFailed,
    /// Bot-like behavior was detected
    BotDetected,
    /// Suspicious activity pattern detected
    SuspiciousActivity,
    /// Rate limit was exceeded
    RateLimitExceeded,
    /// Account was locked out due to security policy
    AccountLockout,
    /// General security alert triggered
    SecurityAlert,
}

impl std::fmt::Display for CaptchaSecurityEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptchaSecurityEvent::ChallengeGenerated => write!(f, "captcha_challenge_generated"),
            CaptchaSecurityEvent::ChallengeValidated => write!(f, "captcha_challenge_validated"),
            CaptchaSecurityEvent::ValidationFailed => write!(f, "captcha_validation_failed"),
            CaptchaSecurityEvent::BotDetected => write!(f, "captcha_bot_detected"),
            CaptchaSecurityEvent::SuspiciousActivity => write!(f, "captcha_suspicious_activity"),
            CaptchaSecurityEvent::RateLimitExceeded => write!(f, "captcha_rate_limit_exceeded"),
            CaptchaSecurityEvent::AccountLockout => write!(f, "captcha_account_lockout"),
            CaptchaSecurityEvent::SecurityAlert => write!(f, "captcha_security_alert"),
        }
    }
}

/// CAPTCHA security event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaSecurityEventData {
    /// Type of security event that occurred
    pub event_type: CaptchaSecurityEvent,
    /// ID of the CAPTCHA challenge involved
    pub challenge_id: Option<String>,
    /// IP address of the client
    pub ip_address: String,
    /// Session ID if available
    pub session_id: Option<String>,
    /// User agent string from the request
    pub user_agent: Option<String>,
    /// Risk level assessment
    pub risk_level: Option<RiskLevel>,
    /// Behavioral classification result
    pub behavior_classification: Option<BehaviorClassification>,
    /// Confidence score for the assessment
    pub confidence_score: Option<f64>,
    /// Number of attempts made
    pub attempt_count: Option<u32>,
    /// Additional event-specific data
    pub additional_data: HashMap<String, serde_json::Value>,
}

impl CaptchaSecurityEventData {
    /// Create a new security event data
    pub fn new(
        event_type: CaptchaSecurityEvent,
        ip_address: String,
        session_id: Option<String>,
    ) -> Self {
        Self {
            event_type,
            challenge_id: None,
            ip_address,
            session_id,
            user_agent: None,
            risk_level: None,
            behavior_classification: None,
            confidence_score: None,
            attempt_count: None,
            additional_data: HashMap::new(),
        }
    }

    /// Add challenge information
    pub fn with_challenge(mut self, challenge: &Challenge) -> Self {
        self.challenge_id = Some(challenge.id.clone());
        self
    }

    /// Add validation result information
    pub fn with_validation_result(mut self, result: &ValidationResult) -> Self {
        self.risk_level = Some(result.risk_assessment.clone());
        self.confidence_score = Some(result.confidence_score);
        self
    }

    /// Add behavioral classification
    pub fn with_behavior_classification(mut self, classification: BehaviorClassification) -> Self {
        self.behavior_classification = Some(classification);
        self
    }

    /// Add user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Add attempt count
    pub fn with_attempt_count(mut self, count: u32) -> Self {
        self.attempt_count = Some(count);
        self
    }

    /// Add confidence score
    pub fn with_confidence_score(mut self, score: f64) -> Self {
        self.confidence_score = Some(score);
        self
    }

    /// Add additional data
    pub fn with_additional_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.additional_data.insert(key, value);
        self
    }
}

/// Activity tracking for security monitoring
#[derive(Debug, Clone)]
pub struct ActivityTracker {
    /// Total number of CAPTCHA attempts
    pub total_attempts: u32,
    /// Number of failed validation attempts
    pub failed_attempts: u32,
    /// Number of bot detections
    pub bot_detections: u32,
    /// Timestamp of the last activity
    pub last_activity: SystemTime,
    /// Recent security events with timestamps
    pub recent_events: Vec<(SystemTime, CaptchaSecurityEvent)>,
}

impl ActivityTracker {
    /// Create a new activity tracker with default values
    pub fn new() -> Self {
        Self {
            total_attempts: 0,
            failed_attempts: 0,
            bot_detections: 0,
            last_activity: SystemTime::now(),
            recent_events: Vec::new(),
        }
    }

    /// Record a security event and update tracking statistics
    pub fn record_event(&mut self, event: CaptchaSecurityEvent) {
        self.total_attempts += 1;
        self.last_activity = SystemTime::now();

        match event {
            CaptchaSecurityEvent::ValidationFailed => {
                self.failed_attempts += 1;
            }
            CaptchaSecurityEvent::BotDetected => {
                self.bot_detections += 1;
            }
            _ => {}
        }

        // Keep only recent events (last 100)
        self.recent_events.push((SystemTime::now(), event));
        if self.recent_events.len() > 100 {
            self.recent_events.remove(0);
        }
    }

    /// Get the rate of recent activity in the last minute
    pub fn get_recent_activity_rate(&self) -> u32 {
        let one_minute_ago = SystemTime::now() - std::time::Duration::from_secs(60);
        self.recent_events
            .iter()
            .filter(|(timestamp, _)| *timestamp > one_minute_ago)
            .count() as u32
    }

    /// Check if the activity rate exceeds the given threshold
    pub fn is_suspicious(&self, threshold: u32) -> bool {
        self.get_recent_activity_rate() > threshold
    }
}

/// CAPTCHA security monitoring state
#[derive(Clone)]
pub struct CaptchaSecurityMonitoringState {
    /// Configuration for security monitoring behavior
    config: CaptchaSecurityConfig,
    /// Optional audit log store for persistent logging
    audit_store: Option<Arc<PgAuditLogStore>>,
    /// Thread-safe activity tracking per IP/session
    activity_tracker: Arc<RwLock<HashMap<String, ActivityTracker>>>,
}

impl CaptchaSecurityMonitoringState {
    /// Create a new CAPTCHA security monitoring state
    pub fn new(
        config: CaptchaSecurityConfig,
        audit_store: Option<Arc<PgAuditLogStore>>,
    ) -> Self {
        let state = Self {
            config,
            audit_store,
            activity_tracker: Arc::new(RwLock::new(HashMap::new())),
        };

        // Spawn cleanup task for activity tracker
        let activity_tracker = state.activity_tracker.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // Clean every 5 minutes
            loop {
                interval.tick().await;
                let mut tracker = activity_tracker.write().await;
                let cutoff_time = SystemTime::now() - std::time::Duration::from_secs(3600); // Keep 1 hour of history

                tracker.retain(|_, activity_data| {
                    activity_data.last_activity > cutoff_time
                });
            }
        });

        state
    }

    /// Log a CAPTCHA security event
    pub async fn log_security_event(&self, event_data: CaptchaSecurityEventData) -> Result<(), CaptchaError> {
        if !self.config.base_config.enabled {
            return Ok(());
        }

        // Check if we should log this event type
        let should_log = match event_data.event_type {
            CaptchaSecurityEvent::ChallengeGenerated => self.config.log_all_attempts,
            CaptchaSecurityEvent::ChallengeValidated => self.config.log_successful_validations,
            CaptchaSecurityEvent::ValidationFailed => self.config.log_failed_validations,
            CaptchaSecurityEvent::BotDetected => self.config.log_bot_detections,
            _ => true, // Always log security alerts and suspicious activities
        };

        if !should_log {
            return Ok(());
        }

        // Update activity tracking
        let tracking_key = event_data.ip_address.clone();
        {
            let mut tracker = self.activity_tracker.write().await;
            let activity = tracker.entry(tracking_key.clone()).or_insert_with(ActivityTracker::new);
            activity.record_event(event_data.event_type.clone());
        }

        // Create audit log entry
        let audit_event = AuditLog {
            timestamp: chrono::Utc::now(),
            event: event_data.event_type.to_string(),
            user_id: event_data.session_id.clone(),
            client_id: None,
            status: match event_data.event_type {
                CaptchaSecurityEvent::ChallengeValidated => "success".to_string(),
                CaptchaSecurityEvent::ValidationFailed |
                CaptchaSecurityEvent::BotDetected |
                CaptchaSecurityEvent::SuspiciousActivity => "failure".to_string(),
                _ => "info".to_string(),
            },
            detail: Some(serde_json::to_string(&event_data).unwrap_or_default()),
        };

        // Store in audit log
        if let Some(audit_store) = &self.audit_store {
            if let Err(e) = audit_store.add_log(&audit_event).await {
                error!("Failed to store CAPTCHA security audit event: {}", e);
            }
        } else {
            // Fallback to structured logging
            info!(
                event_type = %event_data.event_type,
                ip_address = %event_data.ip_address,
                challenge_id = ?event_data.challenge_id,
                risk_level = ?event_data.risk_level,
                "CAPTCHA security event"
            );
        }

        // Check for suspicious activity and generate alerts
        if self.config.enable_real_time_alerts {
            self.check_and_generate_alerts(&tracking_key, &event_data).await?;
        }

        Ok(())
    }

    /// Check for suspicious activity and generate alerts
    async fn check_and_generate_alerts(
        &self,
        tracking_key: &str,
        event_data: &CaptchaSecurityEventData,
    ) -> Result<(), CaptchaError> {
        let tracker = self.activity_tracker.read().await;

        let activity = tracker.get(tracking_key).cloned();

        if let Some(activity) = activity {
            // Check for high activity rate
            if activity.is_suspicious(self.config.suspicious_activity_threshold) {
                let alert_data = CaptchaSecurityEventData::new(
                    CaptchaSecurityEvent::SuspiciousActivity,
                    event_data.ip_address.clone(),
                    event_data.session_id.clone(),
                )
                .with_attempt_count(activity.get_recent_activity_rate())
                .with_additional_data(
                    "alert_reason".to_string(),
                    json!("High activity rate detected"),
                );

                warn!(
                    "Suspicious CAPTCHA activity detected from {}: {} attempts in last minute",
                    event_data.ip_address, activity.get_recent_activity_rate()
                );

                // Log the alert directly without recursion
                warn!(
                    "CAPTCHA security alert: Suspicious activity detected from {}: {} attempts in last minute",
                    event_data.ip_address, activity.get_recent_activity_rate()
                );
            }

            // Check for bot detection threshold
            if let Some(confidence) = event_data.confidence_score {
                if confidence >= self.config.bot_alert_threshold {
                    let alert_data = CaptchaSecurityEventData::new(
                        CaptchaSecurityEvent::SecurityAlert,
                        event_data.ip_address.clone(),
                        event_data.session_id.clone(),
                    )
                    .with_confidence_score(confidence)
                    .with_additional_data(
                        "alert_reason".to_string(),
                        json!("High bot detection confidence"),
                    );

                    warn!(
                        "CAPTCHA security alert: High confidence bot detection from {}: confidence={:.2}",
                        event_data.ip_address, confidence
                    );
                }
            }
        }

        Ok(())
    }

    /// Get activity statistics for an IP address
    pub async fn get_activity_stats(&self, ip_address: &str) -> Option<ActivityTracker> {
        let tracker = self.activity_tracker.read().await;
        tracker.get(ip_address).cloned()
    }

    /// Check if IP address is currently flagged as suspicious
    pub async fn is_suspicious_ip(&self, ip_address: &str) -> bool {
        let tracker = self.activity_tracker.read().await;

        if let Some(activity) = tracker.get(ip_address) {
            activity.is_suspicious(self.config.suspicious_activity_threshold)
        } else {
            false
        }
    }
}

/// Middleware function for CAPTCHA security monitoring
pub async fn captcha_security_monitoring_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(security_state): State<CaptchaSecurityMonitoringState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_string();
    let method = request.method().clone();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Check if this is a monitored CAPTCHA endpoint
    let is_monitored = security_state.config.base_config.monitored_paths
        .iter()
        .any(|monitored_path| path.contains(monitored_path));

    if !is_monitored {
        return Ok(next.run(request).await);
    }

    let start_time = SystemTime::now();

    // Process the request
    let response = next.run(request).await;

    // Log the request/response
    let event_type = match (method.as_str(), response.status()) {
        ("POST", status) if path.contains("/challenge") => {
            if status.is_success() {
                CaptchaSecurityEvent::ChallengeGenerated
            } else {
                CaptchaSecurityEvent::SecurityAlert
            }
        }
        ("POST", status) if path.contains("/validate") || path.contains("/verify") => {
            if status.is_success() {
                CaptchaSecurityEvent::ChallengeValidated
            } else {
                CaptchaSecurityEvent::ValidationFailed
            }
        }
        _ => CaptchaSecurityEvent::SecurityAlert,
    };

    let event_data = CaptchaSecurityEventData::new(
        event_type,
        addr.ip().to_string(),
        None, // Session ID would need to be extracted from request/response
    )
    .with_user_agent(user_agent.unwrap_or_default())
    .with_additional_data(
        "response_status".to_string(),
        json!(response.status().as_u16()),
    )
    .with_additional_data(
        "request_path".to_string(),
        json!(path),
    )
    .with_additional_data(
        "request_method".to_string(),
        json!(method.as_str()),
    );

    // Log the event asynchronously
    let security_state_clone = security_state.clone();
    tokio::spawn(async move {
        if let Err(e) = security_state_clone.log_security_event(event_data).await {
            error!("Failed to log CAPTCHA security event: {}", e);
        }
    });

    Ok(response)
}

/// Helper function to create CAPTCHA security monitoring layer
pub fn create_captcha_security_monitoring_layer(
    config: CaptchaSecurityConfig,
    audit_store: Option<Arc<PgAuditLogStore>>,
) -> impl tower::Layer<CaptchaSecurityMonitoringState> + Clone + Send + Sync + 'static {
    let state = CaptchaSecurityMonitoringState::new(config, audit_store);
    axum::middleware::from_fn_with_state::<_, CaptchaSecurityMonitoringState, (axum::extract::ConnectInfo<std::net::SocketAddr>, axum::extract::State<CaptchaSecurityMonitoringState>)>(state, captcha_security_monitoring_middleware)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_security_monitoring_state_creation() {
        let config = CaptchaSecurityConfig::default();
        let state = CaptchaSecurityMonitoringState::new(config, None);

        let ip = "127.0.0.1";
        let is_suspicious = state.is_suspicious_ip(ip).await;
        assert!(!is_suspicious); // Should not be suspicious initially
    }

    #[tokio::test]
    async fn test_event_logging() {
        let config = CaptchaSecurityConfig::default();
        let state = CaptchaSecurityMonitoringState::new(config, None);

        let event_data = CaptchaSecurityEventData::new(
            CaptchaSecurityEvent::ValidationFailed,
            "127.0.0.1".to_string(),
            Some("session123".to_string()),
        );

        let result = state.log_security_event(event_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_suspicious_activity_detection() {
        let mut config = CaptchaSecurityConfig::default();
        config.suspicious_activity_threshold = 5; // Low threshold for testing

        let state = CaptchaSecurityMonitoringState::new(config, None);

        // Generate multiple events to trigger suspicious activity
        for i in 0..10 {
            let event_data = CaptchaSecurityEventData::new(
                CaptchaSecurityEvent::ValidationFailed,
                "127.0.0.1".to_string(),
                Some(format!("session{}", i)),
            );

            let _ = state.log_security_event(event_data).await;
        }

        let is_suspicious = state.is_suspicious_ip("127.0.0.1").await;
        assert!(is_suspicious);
    }
}
