//! Challenge Validator
//!
//! Validates CAPTCHA responses and manages progressive penalties

use async_trait::async_trait;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::analyzer::{BehavioralAnalyzer, BehavioralAnalyzerTrait};
use super::error::CaptchaError;
use super::rate_limiting::CaptchaRateLimitState;
use super::security_monitoring::{
    CaptchaSecurityEvent, CaptchaSecurityEventData, CaptchaSecurityMonitoringState,
};
use super::types::*;
use crate::utils::crypto::password::{hash_password, verify_password};

/// Challenge validator trait
#[async_trait]
pub trait ChallengeValidatorTrait: Send + Sync {
    /// Validate a challenge response
    async fn validate_response(
        &self,
        challenge: &Challenge,
        answer: &str,
    ) -> Result<bool, CaptchaError>;

    /// Calculate confidence score for validation
    async fn calculate_confidence_score(
        &self,
        challenge: &Challenge,
        answer: &str,
        behavioral_data: Option<&BehavioralMetrics>,
    ) -> Result<f64, CaptchaError>;

    /// Determine next difficulty level
    async fn determine_next_difficulty(
        &self,
        current_difficulty: u8,
        validation_success: bool,
        risk_level: RiskLevel,
        attempt_count: u32,
    ) -> u8;

    /// Check if retry is allowed
    async fn is_retry_allowed(&self, attempt_count: u32, risk_level: RiskLevel) -> bool;
}

/// Validation attempt tracking
#[derive(Debug, Clone)]
struct ValidationAttempt {
    /// Total number of validation attempts
    pub count: u32,
    /// Timestamp of the last validation attempt
    pub last_attempt: SystemTime,
    /// Number of consecutive validation failures
    pub consecutive_failures: u32,
    /// Current risk level based on failure patterns
    pub risk_level: RiskLevel,
}

impl ValidationAttempt {
/// Fungsi `new(`.
    pub fn new() -> Self {
        Self {
            count: 0,
            last_attempt: SystemTime::now(),
            consecutive_failures: 0,
            risk_level: RiskLevel::Low,
        }
    }
/// Fungsi `increment_failure(`.

    pub fn increment_failure(&mut self) {
        self.count += 1;
        self.consecutive_failures += 1;
        self.last_attempt = SystemTime::now();

        // Escalate risk level based on consecutive failures
        self.risk_level = match self.consecutive_failures {
            1..=2 => RiskLevel::Low,
            3..=5 => RiskLevel::Medium,
            6..=10 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };
/// Fungsi `reset_on_success(`.
    }

    pub fn reset_on_success(&mut self) {
        self.count += 1;
        self.consecutive_failures = 0;
        self.last_attempt = SystemTime::now();
/// Fungsi `is_locked_out(`.
        self.risk_level = RiskLevel::Low;
    }

    pub fn is_locked_out(&self) -> bool {
        match self.risk_level {
            RiskLevel::Critical => {
                // Lock out for 1 hour after 10+ consecutive failures
                if let Ok(elapsed) = self.last_attempt.elapsed() {
                    elapsed < Duration::from_secs(3600)
                } else {
                    false
                }
            }
            RiskLevel::High => {
                // Lock out for 15 minutes after 6-10 consecutive failures
                if let Ok(elapsed) = self.last_attempt.elapsed() {
                    elapsed < Duration::from_secs(900)
                } else {
                    false
                }
            }
/// Fungsi `get_lockout_duration(`.
            _ => false,
        }
    }

    pub fn get_lockout_duration(&self) -> Option<Duration> {
        if self.is_locked_out() {
            match self.risk_level {
                RiskLevel::Critical => Some(Duration::from_secs(3600)), // 1 hour
                RiskLevel::High => Some(Duration::from_secs(900)),      // 15 minutes
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Core validation engine for CAPTCHA challenges
pub struct ValidationEngine {
    /// Behavioral analyzer for risk assessment
    behavioral_analyzer: Arc<dyn BehavioralAnalyzerTrait>,
    /// Optional rate limiting state for progressive restrictions
    rate_limit_state: Option<Arc<CaptchaRateLimitState>>,
    /// Optional security monitoring state for event tracking
    security_monitoring_state: Option<Arc<CaptchaSecurityMonitoringState>>,
    /// Thread-safe tracking of validation attempts per IP/session
    attempt_tracker: Arc<RwLock<HashMap<String, ValidationAttempt>>>,
    /// Maximum allowed attempts per session
    max_attempts_per_session: u32,
    /// Minimum confidence threshold for validation acceptance
    confidence_threshold: f64,
}

impl ValidationEngine {
    /// Create a new validation engine
    pub fn new() -> Self {
        Self {
            behavioral_analyzer: Arc::new(BehavioralAnalyzer::new()),
            rate_limit_state: None,
            security_monitoring_state: None,
            attempt_tracker: Arc::new(RwLock::new(HashMap::new())),
            max_attempts_per_session: 10,
            confidence_threshold: 0.7,
        }
    }

    /// Create a new validation engine with rate limiting
    pub fn with_rate_limiting(rate_limit_state: Arc<CaptchaRateLimitState>) -> Self {
        Self {
            behavioral_analyzer: Arc::new(BehavioralAnalyzer::new()),
            rate_limit_state: Some(rate_limit_state),
            security_monitoring_state: None,
            attempt_tracker: Arc::new(RwLock::new(HashMap::new())),
            max_attempts_per_session: 10,
            confidence_threshold: 0.7,
        }
    }

    /// Create a new validation engine with security monitoring
    pub fn with_security_monitoring(
        security_monitoring_state: Arc<CaptchaSecurityMonitoringState>,
    ) -> Self {
        Self {
            behavioral_analyzer: Arc::new(BehavioralAnalyzer::new()),
            rate_limit_state: None,
            security_monitoring_state: Some(security_monitoring_state),
            attempt_tracker: Arc::new(RwLock::new(HashMap::new())),
            max_attempts_per_session: 10,
            confidence_threshold: 0.7,
        }
    }

    /// Create a new validation engine with full integration
    pub fn with_full_integration(
        rate_limit_state: Arc<CaptchaRateLimitState>,
        security_monitoring_state: Arc<CaptchaSecurityMonitoringState>,
    ) -> Self {
        Self {
            behavioral_analyzer: Arc::new(BehavioralAnalyzer::new()),
            rate_limit_state: Some(rate_limit_state),
            security_monitoring_state: Some(security_monitoring_state),
            attempt_tracker: Arc::new(RwLock::new(HashMap::new())),
            max_attempts_per_session: 10,
            confidence_threshold: 0.7,
        }
    }

    /// Create a new validation engine with custom configuration
    pub fn with_config(
        behavioral_analyzer: Arc<dyn BehavioralAnalyzerTrait>,
        rate_limit_state: Option<Arc<CaptchaRateLimitState>>,
        security_monitoring_state: Option<Arc<CaptchaSecurityMonitoringState>>,
        max_attempts_per_session: u32,
        confidence_threshold: f64,
    ) -> Self {
        Self {
            behavioral_analyzer,
            rate_limit_state,
            security_monitoring_state,
            attempt_tracker: Arc::new(RwLock::new(HashMap::new())),
            max_attempts_per_session,
            confidence_threshold,
        }
    }

    /// Get tracking key for attempt tracking (IP + session)
    fn get_tracking_key(&self, ip_address: &str, session_id: Option<&str>) -> String {
        match session_id {
            Some(session) => format!("{}:{}", ip_address, session),
            None => ip_address.to_string(),
        }
    }

    /// Parse IP address string to SocketAddr for rate limiting
    fn parse_ip_address(&self, ip_address: &str) -> Option<std::net::SocketAddr> {
        // Try to parse as IP:port first
        if let Ok(addr) = ip_address.parse::<std::net::SocketAddr>() {
            return Some(addr);
        }

        // Try to parse as IP only and add default port
        if let Ok(ip) = ip_address.parse::<std::net::IpAddr>() {
            return Some(std::net::SocketAddr::new(ip, 80));
        }

        None
    }

    /// Hash an answer for comparison using authenc crypto module
    fn hash_answer(&self, answer: &str, salt: &str) -> Result<String, CaptchaError> {
        let input = format!("{}:{}", answer.trim().to_lowercase(), salt);
        hash_password(&input).map_err(|e| CaptchaError::ValidationFailed {
            message: format!("Hash generation failed: {}", e),
            attempts_remaining: 0,
            next_difficulty: 1,
        })
    }

    /// Verify answer hash
    fn verify_answer_hash(
        &self,
        answer: &str,
        salt: &str,
        expected_hash: &str,
    ) -> Result<bool, CaptchaError> {
        let input = format!("{}:{}", answer.trim().to_lowercase(), salt);
        verify_password(expected_hash, &input).map_err(|e| CaptchaError::ValidationFailed {
            message: format!("Hash verification failed: {}", e),
            attempts_remaining: 0,
            next_difficulty: 1,
        })
    }

    /// Calculate confidence score based on multiple factors
    async fn calculate_confidence_score(
        &self,
        challenge: &Challenge,
        answer: &str,
        behavioral_data: Option<&BehavioralMetrics>,
        answer_correct: bool,
    ) -> Result<f64, CaptchaError> {
        let mut confidence = if answer_correct { 0.8 } else { 0.0 };

        // Adjust based on behavioral analysis if available
        if let Some(behavioral) = behavioral_data {
            let analysis = self
                .behavioral_analyzer
                .analyze_behavior(behavioral)
                .await?;

            match analysis.classification {
                BehaviorClassification::Human => {
                    confidence += 0.2 * analysis.confidence;
                }
                BehaviorClassification::Suspicious => {
                    confidence -= 0.1 * analysis.confidence;
                }
                BehaviorClassification::Bot => {
                    confidence -= 0.3 * analysis.confidence;
                }
                BehaviorClassification::Unknown => {
                    confidence -= 0.05;
                }
            }

            debug!(
                "Behavioral analysis for challenge {}: classification={:?}, risk_score={:.2}, confidence={:.2}",
                challenge.id,
                analysis.classification,
                analysis.overall_risk_score,
                analysis.confidence
            );
        }

        // Adjust based on challenge difficulty
        let difficulty_factor = (challenge.difficulty_level as f64) / 10.0;
        confidence += difficulty_factor * 0.1;

        // Adjust based on response time (too fast might indicate automation)
        if let Ok(elapsed) = challenge.created_at.elapsed() {
            let response_time_secs = elapsed.as_secs_f64();

            // Penalize very fast responses (< 2 seconds for visual challenges)
            if matches!(
                challenge.challenge_type,
                ChallengeType::Visual | ChallengeType::Logical
            ) && response_time_secs < 2.0
            {
                confidence -= 0.2;
                warn!(
                    "Suspiciously fast response time: {:.2}s for challenge {}",
                    response_time_secs, challenge.id
                );
            }

            // Penalize very slow responses (> 5 minutes)
            if response_time_secs > 300.0 {
                confidence -= 0.1;
            }
        }

        Ok(confidence.clamp(0.0, 1.0))
    }

    /// Assess risk level based on validation history and behavioral data
    async fn assess_risk_level(
        &self,
        tracking_key: &str,
        behavioral_data: Option<&BehavioralMetrics>,
        validation_success: bool,
    ) -> Result<RiskLevel, CaptchaError> {
        let tracker = self.attempt_tracker.read().await;
        let attempt_data = tracker.get(tracking_key);

        let mut risk_level = RiskLevel::Low;

        // Check attempt history
        if let Some(attempts) = attempt_data {
            risk_level = attempts.risk_level.clone();

            // Escalate risk if too many recent attempts
            if attempts.count > self.max_attempts_per_session / 2 {
                risk_level = match risk_level {
                    RiskLevel::Low => RiskLevel::Medium,
                    RiskLevel::Medium => RiskLevel::High,
                    RiskLevel::High => RiskLevel::Critical,
                    RiskLevel::Critical => RiskLevel::Critical,
                };
            }
        }

        // Adjust based on behavioral analysis
        if let Some(behavioral) = behavioral_data {
            let analysis = self
                .behavioral_analyzer
                .analyze_behavior(behavioral)
                .await?;

            match analysis.classification {
                BehaviorClassification::Bot => {
                    risk_level = RiskLevel::Critical;
                }
                BehaviorClassification::Suspicious => {
                    risk_level = match risk_level {
                        RiskLevel::Low => RiskLevel::Medium,
                        other => other,
                    };
                }
                _ => {}
            }
        }

        Ok(risk_level)
    }

    /// Update attempt tracking
    async fn update_attempt_tracking(
        &self,
        tracking_key: &str,
        validation_success: bool,
        risk_level: RiskLevel,
    ) -> Result<(), CaptchaError> {
        let mut tracker = self.attempt_tracker.write().await;

        let attempt = tracker
            .entry(tracking_key.to_string())
            .or_insert_with(ValidationAttempt::new);

        if validation_success {
            attempt.reset_on_success();
            info!("Successful validation for key: {}", tracking_key);
        } else {
            attempt.increment_failure();
            attempt.risk_level = risk_level;
            warn!(
                "Failed validation for key: {} (failures: {}, risk: {:?})",
                tracking_key, attempt.consecutive_failures, attempt.risk_level
            );
        }

        Ok(())
    }

    /// Check if user is currently locked out
    async fn check_lockout_status(
        &self,
        tracking_key: &str,
    ) -> Result<Option<Duration>, CaptchaError> {
        let tracker = self.attempt_tracker.read().await;

        if let Some(attempts) = tracker.get(tracking_key) {
            if attempts.is_locked_out() {
                return Ok(attempts.get_lockout_duration());
            }
        }

        Ok(None)
    }

    /// Perform comprehensive challenge validation
    pub async fn validate_challenge(
        &self,
        challenge: &Challenge,
        answer: &str,
        behavioral_data: Option<&BehavioralMetrics>,
    ) -> Result<ValidationResult, CaptchaError> {
        let tracking_key =
            self.get_tracking_key(&challenge.ip_address, challenge.session_id.as_deref());

        // Check if challenge is expired
        if challenge.is_expired() {
            return Ok(ValidationResult::failure(
                RiskLevel::Low,
                challenge.difficulty_level,
                false,
                None,
                "Challenge has expired. Please request a new challenge.".to_string(),
            ));
        }

        // Check lockout status
        if let Some(lockout_duration) = self.check_lockout_status(&tracking_key).await? {
            return Ok(ValidationResult::failure(
                RiskLevel::Critical,
                challenge.difficulty_level.saturating_add(2),
                false,
                Some(lockout_duration),
                "Account temporarily locked due to suspicious activity.".to_string(),
            ));
        }

        // Validate the answer
        let answer_correct =
            self.verify_answer_hash(answer, &challenge.id, &challenge.expected_answer_hash)?;

        // Calculate confidence score
        let confidence_score = self
            .calculate_confidence_score(challenge, answer, behavioral_data, answer_correct)
            .await?;

        // Assess risk level
        let risk_level = self
            .assess_risk_level(&tracking_key, behavioral_data, answer_correct)
            .await?;

        // Determine if validation is successful based on answer correctness and confidence
        let validation_success = answer_correct && confidence_score >= self.confidence_threshold;

        // Update attempt tracking
        self.update_attempt_tracking(&tracking_key, validation_success, risk_level.clone())
            .await?;

        // Update rate limiting state if available
        if let Some(rate_limiter) = &self.rate_limit_state {
            if let Some(socket_addr) = self.parse_ip_address(&challenge.ip_address) {
                if validation_success {
                    rate_limiter.record_captcha_success(&socket_addr).await;
                } else {
                    rate_limiter
                        .record_captcha_failure(&socket_addr, risk_level.clone())
                        .await;
                }
            }
        }

        // Log security events if monitoring is available
        if let Some(security_monitor) = &self.security_monitoring_state {
            let event_type = if validation_success {
                CaptchaSecurityEvent::ChallengeValidated
            } else {
                CaptchaSecurityEvent::ValidationFailed
            };

            let mut event_data = CaptchaSecurityEventData::new(
                event_type,
                challenge.ip_address.clone(),
                challenge.session_id.clone(),
            )
            .with_challenge(challenge);

            // Add behavioral classification if available
            if let Some(behavioral) = behavioral_data {
                event_data =
                    event_data.with_behavior_classification(behavioral.classification.clone());
            }

            // Add attempt count
            let attempt_count = {
                let tracker = self.attempt_tracker.read().await;
                tracker.get(&tracking_key).map(|a| a.count).unwrap_or(1)
            };
            event_data = event_data.with_attempt_count(attempt_count);

            // Log the event asynchronously to avoid blocking validation
            let security_monitor_clone = security_monitor.clone();
            let event_data_clone = event_data.clone();
            tokio::spawn(async move {
                if let Err(e) = security_monitor_clone
                    .log_security_event(event_data_clone)
                    .await
                {
                    error!("Failed to log CAPTCHA security event: {}", e);
                }
            });

            // Check for bot detection and log separate event if needed
            if let Some(behavioral) = behavioral_data {
                if matches!(behavioral.classification, BehaviorClassification::Bot) {
                    let bot_event = CaptchaSecurityEventData::new(
                        CaptchaSecurityEvent::BotDetected,
                        challenge.ip_address.clone(),
                        challenge.session_id.clone(),
                    )
                    .with_challenge(challenge)
                    .with_behavior_classification(behavioral.classification.clone())
                    .with_additional_data(
                        "bot_risk_score".to_string(),
                        serde_json::json!(behavioral.risk_score),
                    );

                    let security_monitor_clone = security_monitor.clone();
                    tokio::spawn(async move {
                        if let Err(e) = security_monitor_clone.log_security_event(bot_event).await {
                            error!("Failed to log bot detection event: {}", e);
                        }
                    });
                }
            }
        }

        // Determine next difficulty level
        let next_difficulty = self
            .determine_next_difficulty(
                challenge.difficulty_level,
                validation_success,
                risk_level.clone(),
                {
                    let tracker = self.attempt_tracker.read().await;
                    tracker.get(&tracking_key).map(|a| a.count).unwrap_or(1)
                },
            )
            .await;

        // Check if retry is allowed
        let retry_allowed = self
            .is_retry_allowed(
                {
                    let tracker = self.attempt_tracker.read().await;
                    tracker
                        .get(&tracking_key)
                        .map(|a| a.consecutive_failures)
                        .unwrap_or(0)
                },
                risk_level.clone(),
            )
            .await;

        // Generate appropriate message
        let message = if validation_success {
            "Challenge completed successfully.".to_string()
        } else if !answer_correct {
            "Incorrect answer. Please try again.".to_string()
        } else {
            "Validation failed due to suspicious behavior patterns.".to_string()
        };

        // Get lockout duration if applicable
        let lockout_duration = if !retry_allowed {
            self.check_lockout_status(&tracking_key).await?
        } else {
            None
        };

        let result = if validation_success {
            ValidationResult::success(confidence_score, next_difficulty)
        } else {
            ValidationResult::failure(
                risk_level,
                next_difficulty,
                retry_allowed,
                lockout_duration,
                message,
            )
        };

        info!(
            "Challenge validation completed: id={}, success={}, confidence={:.2}, risk={:?}",
            challenge.id, result.success, result.confidence_score, result.risk_assessment
        );

        Ok(result)
    }
}

#[async_trait]
impl ChallengeValidatorTrait for ValidationEngine {
    async fn validate_response(
        &self,
        challenge: &Challenge,
        answer: &str,
    ) -> Result<bool, CaptchaError> {
        // TODO: Implement response validation
        // 1. Hash the provided answer
        // 2. Compare with expected answer hash
        // 3. Consider fuzzy matching for accessibility

        let answer_hash = self.hash_answer(answer, &challenge.id)?;
        Ok(answer_hash == challenge.expected_answer_hash)
    }

    async fn calculate_confidence_score(
        &self,
        challenge: &Challenge,
        answer: &str,
        behavioral_data: Option<&BehavioralMetrics>,
    ) -> Result<f64, CaptchaError> {
        // TODO: Implement confidence scoring
        // 1. Base score from answer correctness
        // 2. Behavioral analysis contribution
        // 3. Timing analysis
        // 4. Pattern recognition

        let mut confidence: f64 = 0.5; // Base confidence

        // Adjust based on behavioral data if available
        if let Some(behavioral) = behavioral_data {
            match behavioral.classification {
                BehaviorClassification::Human => confidence += 0.3,
                BehaviorClassification::Suspicious => confidence -= 0.2,
                BehaviorClassification::Bot => confidence -= 0.4,
                BehaviorClassification::Unknown => confidence -= 0.1,
            }
        }

        Ok(confidence.clamp(0.0, 1.0))
    }

    async fn determine_next_difficulty(
        &self,
        current_difficulty: u8,
        validation_success: bool,
        risk_level: RiskLevel,
        attempt_count: u32,
    ) -> u8 {
        let mut next_difficulty = current_difficulty;

        if !validation_success {
            // Increase difficulty on failure
            next_difficulty = match risk_level {
                RiskLevel::Low => current_difficulty + 1,
                RiskLevel::Medium => current_difficulty + 2,
                RiskLevel::High => current_difficulty + 3,
                RiskLevel::Critical => current_difficulty + 5,
            };
        } else if attempt_count == 1 {
            // Decrease difficulty slightly on first success
            next_difficulty = current_difficulty.saturating_sub(1);
        }

        // Clamp difficulty between 1 and 10
        next_difficulty.clamp(1, 10)
    }

    async fn is_retry_allowed(&self, attempt_count: u32, risk_level: RiskLevel) -> bool {
        let max_attempts = match risk_level {
            RiskLevel::Low => 5,
            RiskLevel::Medium => 3,
            RiskLevel::High => 2,
            RiskLevel::Critical => 1,
        };

        attempt_count < max_attempts
    }
}
