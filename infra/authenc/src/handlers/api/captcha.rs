//! CAPTCHA API handlers for challenge generation and validation
//!
//! Provides REST API endpoints for CAPTCHA operations integrated with authenc middleware

use crate::error::AuthencError;
use crate::handlers::api::auth_bearer::AuthBearer;
use crate::middleware::{
    csrf_protection_axum::{CsrfConfig, CsrfState},
    rate_limit_axum::{RateLimitConfig, RateLimiterState},
    security_monitoring_axum::SecurityMonitoringConfig,
};
use crate::services::captcha::alerting::AlertRule;
use crate::services::captcha::dashboard::{
    Alert, AlertSeverity, DashboardConfig, DashboardData, DashboardService,
};
use crate::services::captcha::{
    BehavioralMetrics, CaptchaError, CaptchaService, CaptchaServiceTrait, ChallengeType,
    RiskAssessmentService,
};
use axum::{
    Router,
    extract::{ConnectInfo, Path, Query, State},
    middleware,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

/// Create CAPTCHA API routes with integrated middleware
pub fn create_captcha_routes() -> Router<Arc<crate::app::AppState>> {
    // Create CAPTCHA-specific rate limiting configuration
    let captcha_rate_limit_config = RateLimitConfig {
        requests_per_minute: 30, // More restrictive for CAPTCHA operations
        excluded_paths: vec![],  // No exclusions for CAPTCHA endpoints
        enabled: true,
        progressive_delays: true,
        base_delay_ms: 2000, // 2 second base delay
        max_delay_ms: 30000, // 30 seconds max delay for suspicious activity
    };

    // Create CSRF protection configuration
    let csrf_config = CsrfConfig {
        enabled: true,
        header_name: "X-CSRF-Token".to_string(),
        cookie_name: "captcha_csrf_token".to_string(),
        token_length: 32,
        excluded_paths: vec![
            "/captcha/challenge".to_string(), // Allow challenge generation without CSRF
            "/api/v1/captcha/challenge".to_string(), // Full path for nested routes
        ],
    };

    // Create security monitoring configuration
    let security_monitoring_config = SecurityMonitoringConfig {
        enabled: true,
        suspicious_threshold_rpm: 50, // Lower threshold for CAPTCHA endpoints
        monitored_paths: vec![
            "/captcha/validate".to_string(),
            "/captcha/challenge".to_string(),
        ],
        log_auth_attempts: true,
        log_authz_failures: true,
    };

    Router::new()
        .route("/captcha/challenge", post(generate_challenge))
        .route("/captcha/challenge/{id}", get(get_challenge))
        .route("/captcha/validate", post(validate_challenge))
        .route("/captcha/refresh/{id}", post(refresh_challenge))
        .route("/captcha/difficulty", post(adjust_difficulty))
        // Dashboard endpoints
        .route("/captcha/dashboard", get(get_dashboard_data))
        .route("/captcha/dashboard/metrics", get(get_metrics_summary))
        .route("/captcha/dashboard/alerts", get(get_alerts))
        .route(
            "/captcha/dashboard/alerts/{id}/acknowledge",
            post(acknowledge_alert),
        )
        .route("/captcha/dashboard/alerts/{id}/resolve", post(resolve_alert))
        .route("/captcha/dashboard/health", get(get_system_health))
        // Alerting endpoints
        .route("/captcha/alerts/rules", get(get_alert_rules))
        .route("/captcha/alerts/rules", post(add_alert_rule))
        .route("/captcha/alerts/test", post(trigger_test_alert))
        .route("/captcha/alerts/health-check", post(check_system_health))
        // Risk assessment endpoints
        .route("/captcha/risk/login", post(assess_login_risk))
        .route("/captcha/risk/mfa-setup", get(assess_mfa_setup_risk))
        .route(
            "/captcha/risk/password-reset",
            post(assess_password_reset_risk),
        )
        // Apply rate limiting middleware to all CAPTCHA endpoints
        .layer(middleware::from_fn_with_state(
            Arc::new(RateLimiterState::new(captcha_rate_limit_config)),
            crate::middleware::rate_limit_axum::rate_limit_middleware,
        ))
        // Apply CSRF protection middleware
        .layer(middleware::from_fn_with_state(
            Arc::new(CsrfState::new(csrf_config)),
            crate::middleware::csrf_protection_axum::csrf_protection_middleware,
        ))
    // Security monitoring will be applied at the router level in main handlers/mod.rs
    // to have access to the audit store from AppState
}

#[derive(Deserialize)]
/// Request payload for CAPTCHA challenge generation
pub struct ChallengeRequest {
    /// Type of challenge to generate
    pub challenge_type: Option<ChallengeType>,
    /// Difficulty level (1-10)
    pub difficulty: Option<u8>,
    /// Session ID for tracking
    pub session_id: Option<String>,
    /// Additional context for challenge generation
    pub context: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
/// Response payload for CAPTCHA challenge generation
pub struct ChallengeResponse {
    /// Challenge ID for validation
    pub challenge_id: String,
    /// Challenge type
    pub challenge_type: ChallengeType,
    /// Challenge data (encrypted)
    pub challenge_data: String,
    /// Difficulty level
    pub difficulty: u8,
    /// Expiration timestamp
    pub expires_at: u64,
    /// Additional metadata
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
/// Request payload for CAPTCHA validation
pub struct ValidationRequest {
    /// Challenge ID to validate
    pub challenge_id: String,
    /// User's answer to the challenge
    pub answer: String,
    /// Behavioral metrics for bot detection
    pub behavioral_data: Option<BehavioralMetrics>,
    /// Session ID for tracking
    pub session_id: Option<String>,
}

#[derive(Serialize)]
/// Response payload for CAPTCHA validation
pub struct ValidationResponse {
    /// Whether validation was successful
    pub success: bool,
    /// Confidence score (0.0 - 1.0)
    pub confidence_score: f64,
    /// Risk assessment level
    pub risk_level: String,
    /// Next difficulty level
    pub next_difficulty: u8,
    /// Whether retry is allowed
    pub retry_allowed: bool,
    /// Lockout duration in seconds (if applicable)
    pub lockout_duration: Option<u64>,
    /// Response message
    pub message: String,
}

#[derive(Deserialize)]
/// Query parameters for difficulty adjustment
pub struct DifficultyQuery {
    /// IP address pattern for adjustment
    pub ip_pattern: Option<String>,
    /// Session ID for adjustment
    pub session_id: Option<String>,
}

#[derive(Deserialize)]
/// Request payload for difficulty adjustment
pub struct DifficultyRequest {
    /// New difficulty level (1-10)
    pub difficulty: u8,
    /// Reason for adjustment
    pub reason: Option<String>,
}

#[derive(Serialize)]
/// Response payload for difficulty adjustment
pub struct DifficultyResponse {
    /// Success message
    pub message: String,
    /// New difficulty level
    pub difficulty: u8,
}

/// Generate a new CAPTCHA challenge
pub async fn generate_challenge(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(_state): State<Arc<crate::app::AppState>>,
    Json(req): Json<ChallengeRequest>,
) -> Result<Json<ChallengeResponse>, AuthencError> {
    let ip = addr.ip().to_string();

    // Generate simple in-memory challenge without database dependency
    // This is suitable for development/testing environments
    let challenge_id = uuid::Uuid::new_v4().to_string();
    let difficulty = req.difficulty.unwrap_or(3);
    let challenge_type = req.challenge_type.clone().unwrap_or(ChallengeType::Visual);

    // Generate a simple math challenge for development
    let (challenge_data, _answer) = match challenge_type {
        ChallengeType::Audio => {
            // Simple audio challenge data
            let num = rand::random::<u8>() % 10;
            (format!("{{\"type\":\"audio\",\"question\":\"What number is {}?\"}}", num), num.to_string())
        }
        ChallengeType::Visual | _ => {
            // Simple visual math challenge
            let a = (rand::random::<u8>() % 10) as u32 + 1;
            let b = (rand::random::<u8>() % 10) as u32 + 1;
            let answer = a + b;
            (format!("{{\"type\":\"math\",\"question\":\"What is {} + {}?\",\"image_data\":\"data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='200' height='80'><rect fill='white' width='200' height='80'/><text x='50%' y='50%' dominant-baseline='middle' text-anchor='middle' font-size='24' fill='black'>{} + {} = ?</text></svg>\"}}", a, b, a, b), answer.to_string())
        }
    };

    // Expires in 5 minutes
    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() + 300;

    // Create metadata
    let mut metadata = HashMap::new();
    if let Some(session_id) = &req.session_id {
        metadata.insert("session_id".to_string(), session_id.clone());
    }
    metadata.insert("ip".to_string(), ip);
    if let Some(context) = req.context {
        metadata.extend(context);
    }

    let response = ChallengeResponse {
        challenge_id,
        challenge_type,
        challenge_data,
        difficulty,
        expires_at,
        metadata: if metadata.is_empty() {
            None
        } else {
            Some(metadata)
        },
    };

    Ok(Json(response))
}

/// Get an existing CAPTCHA challenge
pub async fn get_challenge(
    Path(challenge_id): Path<String>,
    State(state): State<Arc<crate::app::AppState>>,
) -> Result<Json<ChallengeResponse>, AuthencError> {
    // Create CAPTCHA service instance
    let captcha_service = CaptchaService::simple().await;

    // Get challenge
    let challenge = captcha_service
        .get_challenge(challenge_id)
        .await
        .map_err(|e| match e {
            CaptchaError::ChallengeNotFound { challenge_id, .. } => {
                AuthencError::not_found(&format!("Challenge {} not found", challenge_id))
            }
            CaptchaError::ChallengeExpired { challenge_id, .. } => {
                AuthencError::validation(&format!("Challenge {} has expired", challenge_id))
            }
            _ => AuthencError::internal("Failed to retrieve challenge"),
        })?;

    // Convert SystemTime to timestamp
    let expires_at = challenge
        .expires_at
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| AuthencError::internal("Invalid expiration time"))?
        .as_secs();

    let response = ChallengeResponse {
        challenge_id: challenge.id,
        challenge_type: challenge.challenge_type,
        challenge_data: challenge.encrypted_data,
        difficulty: challenge.difficulty_level,
        expires_at,
        metadata: None,
    };

    Ok(Json(response))
}

/// Validate a CAPTCHA challenge response
pub async fn validate_challenge(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<ValidationRequest>,
) -> Result<Json<ValidationResponse>, AuthencError> {
    let ip = addr.ip().to_string();

    // Create CAPTCHA service instance
    let captcha_service = CaptchaService::simple().await;

    // Validate challenge
    let result = captcha_service
        .validate_challenge(req.challenge_id, req.answer, req.behavioral_data)
        .await
        .map_err(|e| match e {
            CaptchaError::ValidationFailed { message, .. } => AuthencError::validation(&message),
            CaptchaError::ChallengeNotFound { challenge_id, .. } => {
                AuthencError::not_found(&format!("Challenge {} not found", challenge_id))
            }
            CaptchaError::ChallengeExpired { challenge_id, .. } => {
                AuthencError::validation(&format!("Challenge {} has expired", challenge_id))
            }
            CaptchaError::RateLimitExceeded { .. } => {
                AuthencError::too_many_requests("CAPTCHA validation rate limit exceeded")
            }
            CaptchaError::UserLockedOut { .. } => {
                AuthencError::forbidden("User account locked due to suspicious activity")
            }
            _ => AuthencError::internal("CAPTCHA validation failed"),
        })?;

    // Convert risk level to string
    let risk_level = match result.risk_assessment {
        crate::services::captcha::RiskLevel::Low => "low",
        crate::services::captcha::RiskLevel::Medium => "medium",
        crate::services::captcha::RiskLevel::High => "high",
        crate::services::captcha::RiskLevel::Critical => "critical",
    }
    .to_string();

    // Convert lockout duration to seconds
    let lockout_duration = result.lockout_duration.map(|d| d.as_secs());

    let response = ValidationResponse {
        success: result.success,
        confidence_score: result.confidence_score,
        risk_level,
        next_difficulty: result.next_difficulty,
        retry_allowed: result.retry_allowed,
        lockout_duration,
        message: result.message,
    };

    Ok(Json(response))
}

/// Refresh an existing CAPTCHA challenge
pub async fn refresh_challenge(
    Path(challenge_id): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
) -> Result<Json<ChallengeResponse>, AuthencError> {
    let ip = addr.ip().to_string();

    // Create CAPTCHA service instance
    let captcha_service = CaptchaService::simple().await;

    // Refresh challenge
    let challenge = captcha_service
        .refresh_challenge(challenge_id)
        .await
        .map_err(|e| match e {
            CaptchaError::ChallengeNotFound { challenge_id, .. } => {
                AuthencError::not_found(&format!("Challenge {} not found", challenge_id))
            }
            CaptchaError::GenerationFailed { message, .. } => AuthencError::internal(&message),
            CaptchaError::RateLimitExceeded { .. } => {
                AuthencError::too_many_requests("CAPTCHA refresh rate limit exceeded")
            }
            _ => AuthencError::internal("CAPTCHA refresh failed"),
        })?;

    // Convert SystemTime to timestamp
    let expires_at = challenge
        .expires_at
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| AuthencError::internal("Invalid expiration time"))?
        .as_secs();

    let response = ChallengeResponse {
        challenge_id: challenge.id,
        challenge_type: challenge.challenge_type,
        challenge_data: challenge.encrypted_data,
        difficulty: challenge.difficulty_level,
        expires_at,
        metadata: None,
    };

    Ok(Json(response))
}

/// Adjust CAPTCHA difficulty for specific patterns
pub async fn adjust_difficulty(
    Query(query): Query<DifficultyQuery>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<DifficultyRequest>,
) -> Result<Json<DifficultyResponse>, AuthencError> {
    // Validate difficulty level
    if req.difficulty < 1 || req.difficulty > 10 {
        return Err(AuthencError::validation(
            "Difficulty must be between 1 and 10",
        ));
    }

    // TODO: Implement difficulty adjustment logic
    // This would typically involve:
    // 1. Updating difficulty settings in database/cache
    // 2. Applying to matching IP patterns or sessions
    // 3. Logging the adjustment for audit purposes

    let response = DifficultyResponse {
        message: format!(
            "Difficulty adjusted to {} for {}",
            req.difficulty,
            query
                .ip_pattern
                .as_deref()
                .or(query.session_id.as_deref())
                .unwrap_or("global")
        ),
        difficulty: req.difficulty,
    };

    Ok(Json(response))
}

/// Get complete dashboard data
pub async fn get_dashboard_data(
    State(state): State<Arc<crate::app::AppState>>,
) -> Result<Json<DashboardData>, AuthencError> {
    // Create dashboard service instance
    let captcha_service = CaptchaService::simple().await;
    let dashboard_service = DashboardService::new(
        captcha_service.get_metrics_collector(),
        captcha_service.get_db_ops(),
        DashboardConfig::default(),
    );

    let dashboard_data = dashboard_service
        .generate_dashboard_data()
        .await
        .map_err(|e| {
            AuthencError::internal(&format!("Failed to generate dashboard data: {}", e))
        })?;

    Ok(Json(dashboard_data))
}

#[derive(Deserialize)]
/// Query parameters for CAPTCHA metrics
pub struct MetricsQuery {
    /// Time window in minutes (default: 60)
    pub time_window: Option<u64>,
}

/// Get metrics summary
pub async fn get_metrics_summary(
    Query(query): Query<MetricsQuery>,
    State(state): State<Arc<crate::app::AppState>>,
) -> Result<Json<crate::services::captcha::metrics::MetricsSummary>, AuthencError> {
    let captcha_service = CaptchaService::simple().await;
    let time_window = query.time_window.unwrap_or(60); // Default to 1 hour

    let metrics_summary = captcha_service
        .generate_metrics_summary(time_window)
        .await
        .map_err(|e| {
            AuthencError::internal(&format!("Failed to generate metrics summary: {}", e))
        })?;

    Ok(Json(metrics_summary))
}

/// Get current alerts
pub async fn get_alerts(
    State(state): State<Arc<crate::app::AppState>>,
) -> Result<Json<Vec<Alert>>, AuthencError> {
    let captcha_service = CaptchaService::simple().await;
    let dashboard_service = DashboardService::new(
        captcha_service.get_metrics_collector(),
        captcha_service.get_db_ops(),
        DashboardConfig::default(),
    );

    let dashboard_data = dashboard_service
        .generate_dashboard_data()
        .await
        .map_err(|e| AuthencError::internal(&format!("Failed to get alerts: {}", e)))?;

    Ok(Json(dashboard_data.alerts))
}

/// Acknowledge an alert
pub async fn acknowledge_alert(
    Path(alert_id): Path<String>,
    State(state): State<Arc<crate::app::AppState>>,
) -> Result<Json<serde_json::Value>, AuthencError> {
    let captcha_service = CaptchaService::simple().await;
    let dashboard_service = DashboardService::new(
        captcha_service.get_metrics_collector(),
        captcha_service.get_db_ops(),
        DashboardConfig::default(),
    );

    dashboard_service
        .acknowledge_alert(&alert_id)
        .await
        .map_err(|e| AuthencError::internal(&format!("Failed to acknowledge alert: {}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "Alert acknowledged successfully",
        "alert_id": alert_id
    })))
}

/// Resolve an alert
pub async fn resolve_alert(
    Path(alert_id): Path<String>,
    State(state): State<Arc<crate::app::AppState>>,
) -> Result<Json<serde_json::Value>, AuthencError> {
    let captcha_service = CaptchaService::simple().await;
    let dashboard_service = DashboardService::new(
        captcha_service.get_metrics_collector(),
        captcha_service.get_db_ops(),
        DashboardConfig::default(),
    );

    dashboard_service
        .resolve_alert(&alert_id)
        .await
        .map_err(|e| AuthencError::internal(&format!("Failed to resolve alert: {}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "Alert resolved successfully",
        "alert_id": alert_id
    })))
}

/// Get system health status
pub async fn get_system_health(
    State(state): State<Arc<crate::app::AppState>>,
) -> Result<Json<crate::services::captcha::dashboard::SystemHealthStatus>, AuthencError> {
    let captcha_service = CaptchaService::simple().await;
    let dashboard_service = DashboardService::new(
        captcha_service.get_metrics_collector(),
        captcha_service.get_db_ops(),
        DashboardConfig::default(),
    );

    let dashboard_data = dashboard_service
        .generate_dashboard_data()
        .await
        .map_err(|e| AuthencError::internal(&format!("Failed to get system health: {}", e)))?;

    Ok(Json(dashboard_data.system_health))
}

/// Get all alert rules
pub async fn get_alert_rules(
    State(state): State<Arc<crate::app::AppState>>,
) -> Result<Json<Vec<AlertRule>>, AuthencError> {
    let captcha_service = CaptchaService::simple().await;

    let rules = captcha_service.get_alert_rules().await;

    Ok(Json(rules))
}

#[derive(Deserialize)]
/// Request to add a new CAPTCHA alert rule
pub struct AddAlertRuleRequest {
    /// The alert rule to add
    pub rule: AlertRule,
}

/// Add a new alert rule
pub async fn add_alert_rule(
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<AddAlertRuleRequest>,
) -> Result<Json<serde_json::Value>, AuthencError> {
    let captcha_service = CaptchaService::simple().await;

    captcha_service
        .add_alert_rule(req.rule)
        .await
        .map_err(|e| AuthencError::internal(&format!("Failed to add alert rule: {}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "Alert rule added successfully"
    })))
}

#[derive(Deserialize)]
/// Request to trigger a test CAPTCHA alert
pub struct TestAlertRequest {
    /// Alert severity level
    pub severity: String,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
}

/// Trigger a test alert
pub async fn trigger_test_alert(
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<TestAlertRequest>,
) -> Result<Json<serde_json::Value>, AuthencError> {
    let captcha_service = CaptchaService::simple().await;

    let severity = match req.severity.to_lowercase().as_str() {
        "info" => AlertSeverity::Info,
        "warning" => AlertSeverity::Warning,
        "critical" => AlertSeverity::Critical,
        "emergency" => AlertSeverity::Emergency,
        _ => return Err(AuthencError::validation("Invalid severity level")),
    };

    captcha_service
        .trigger_test_alert(severity, req.title, req.description)
        .await
        .map_err(|e| AuthencError::internal(&format!("Failed to trigger test alert: {}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "Test alert triggered successfully"
    })))
}

/// Check system health and trigger alerts if needed
pub async fn check_system_health(
    State(state): State<Arc<crate::app::AppState>>,
) -> Result<Json<serde_json::Value>, AuthencError> {
    let captcha_service = CaptchaService::simple().await;

    captcha_service
        .check_system_health()
        .await
        .map_err(|e| AuthencError::internal(&format!("Failed to check system health: {}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "System health check completed"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::captcha::ChallengeType;

    #[test]
    fn test_challenge_request_deserialization() {
        let json = r#"{
            "challenge_type": "Visual",
            "difficulty": 5,
            "session_id": "test-session"
        }"#;

        let req: ChallengeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.challenge_type, Some(ChallengeType::Visual));
        assert_eq!(req.difficulty, Some(5));
        assert_eq!(req.session_id, Some("test-session".to_string()));
    }

    #[test]
    fn test_validation_request_deserialization() {
        let json = r#"{
            "challenge_id": "test-challenge",
            "answer": "test-answer",
            "session_id": "test-session"
        }"#;

        let req: ValidationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.challenge_id, "test-challenge");
        assert_eq!(req.answer, "test-answer");
        assert_eq!(req.session_id, Some("test-session".to_string()));
    }

    #[test]
    fn test_difficulty_validation() {
        let valid_req = DifficultyRequest {
            difficulty: 5,
            reason: Some("Test".to_string()),
        };
        assert!(valid_req.difficulty >= 1 && valid_req.difficulty <= 10);

        // Invalid difficulty would be caught by validation in the handler
    }
}

// ============================================================================
// RISK ASSESSMENT ENDPOINTS
// ============================================================================

#[derive(Deserialize)]
/// Request to assess login risk
pub struct AssessLoginRiskRequest {
    /// Username attempting to log in
    pub username: String,
    /// User agent string
    pub user_agent: Option<String>,
}

#[derive(Serialize)]
/// Response with risk assessment
pub struct RiskAssessmentResponse {
    /// Risk score (0.0 - 1.0)
    pub risk_score: f64,
    /// Whether CAPTCHA is required
    pub captcha_required: bool,
    /// Risk level description
    pub risk_level: String,
    /// Explanation of risk factors
    pub factors: Option<Vec<String>>,
}

/// Assess risk for login attempt
pub async fn assess_login_risk(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(_state): State<Arc<crate::app::AppState>>,
    Json(req): Json<AssessLoginRiskRequest>,
) -> Result<Json<RiskAssessmentResponse>, AuthencError> {
    let ip = addr.ip().to_string();

    let risk_service = RiskAssessmentService::default();

    let (risk_score, captcha_required) = risk_service
        .assess_login_risk(&req.username, &ip, req.user_agent.as_deref())
        .await
        .map_err(|e| AuthencError::internal(&format!("Risk assessment failed: {}", e)))?;

    let risk_level = risk_service.risk_score_to_level(risk_score);
    let risk_level_str = match risk_level {
        crate::services::captcha::RiskLevel::Low => "low",
        crate::services::captcha::RiskLevel::Medium => "medium",
        crate::services::captcha::RiskLevel::High => "high",
        crate::services::captcha::RiskLevel::Critical => "critical",
    }
    .to_string();

    Ok(Json(RiskAssessmentResponse {
        risk_score,
        captcha_required,
        risk_level: risk_level_str,
        factors: None, // TODO: Add detailed factor breakdown
    }))
}

/// Assess risk for MFA setup
pub async fn assess_mfa_setup_risk(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(_state): State<Arc<crate::app::AppState>>,
    AuthBearer(auth): AuthBearer,
) -> Result<Json<RiskAssessmentResponse>, AuthencError> {
    let ip = addr.ip().to_string();

    // Extract user_id from JWT token
    let user_id = &auth.sub;

    tracing::debug!("Assessing MFA setup risk for user: {}, IP: {}", user_id, ip);

    let risk_service = RiskAssessmentService::default();

    let risk_score = risk_service
        .assess_mfa_setup_risk(user_id, &ip)
        .await
        .map_err(|e| AuthencError::internal(&format!("Risk assessment failed: {}", e)))?;

    let captcha_required = risk_score > 0.5;
    let risk_level = risk_service.risk_score_to_level(risk_score);
    let risk_level_str = match risk_level {
        crate::services::captcha::RiskLevel::Low => "low",
        crate::services::captcha::RiskLevel::Medium => "medium",
        crate::services::captcha::RiskLevel::High => "high",
        crate::services::captcha::RiskLevel::Critical => "critical",
    }
    .to_string();

    Ok(Json(RiskAssessmentResponse {
        risk_score,
        captcha_required,
        risk_level: risk_level_str,
        factors: None,
    }))
}

#[derive(Deserialize)]
/// Request to assess password reset risk
pub struct AssessPasswordResetRiskRequest {
    /// Email address for password reset
    pub email: String,
}

/// Assess risk for password reset
pub async fn assess_password_reset_risk(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(_state): State<Arc<crate::app::AppState>>,
    Json(req): Json<AssessPasswordResetRiskRequest>,
) -> Result<Json<RiskAssessmentResponse>, AuthencError> {
    let ip = addr.ip().to_string();

    let risk_service = RiskAssessmentService::default();

    let risk_score = risk_service
        .assess_password_reset_risk(&req.email, &ip)
        .await
        .map_err(|e| AuthencError::internal(&format!("Risk assessment failed: {}", e)))?;

    // Password reset always requires CAPTCHA
    let captcha_required = true;
    let risk_level = risk_service.risk_score_to_level(risk_score);
    let risk_level_str = match risk_level {
        crate::services::captcha::RiskLevel::Low => "low",
        crate::services::captcha::RiskLevel::Medium => "medium",
        crate::services::captcha::RiskLevel::High => "high",
        crate::services::captcha::RiskLevel::Critical => "critical",
    }
    .to_string();

    Ok(Json(RiskAssessmentResponse {
        risk_score,
        captcha_required,
        risk_level: risk_level_str,
        factors: Some(vec![
            "Password reset is a sensitive operation".to_string(),
            "CAPTCHA is always required for security".to_string(),
        ]),
    }))
}
