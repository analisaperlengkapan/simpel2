//! CAPTCHA Service Implementation
//!
//! Main service orchestrating CAPTCHA operations with authenc integration

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use super::analyzer::BehavioralAnalyzerTrait;
use super::error::CaptchaError;
use super::generator::ChallengeGeneratorTrait;
use super::validator::ChallengeValidatorTrait;
use super::types::*;

/// CAPTCHA service trait defining core operations
#[async_trait]
pub trait CaptchaServiceTrait: Send + Sync {
    /// Generate a new CAPTCHA challenge
    async fn generate_challenge(
        &self,
        challenge_type: ChallengeType,
        difficulty: Option<u8>,
        session_id: Option<String>,
        ip_address: String,
    ) -> Result<Challenge, CaptchaError>;

    /// Validate a CAPTCHA response
    async fn validate_challenge(
        &self,
        challenge_id: String,
        answer: String,
        behavioral_data: Option<BehavioralMetrics>,
    ) -> Result<ValidationResult, CaptchaError>;

    /// Refresh an existing challenge
    async fn refresh_challenge(
        &self,
        challenge_id: String,
    ) -> Result<Challenge, CaptchaError>;

    /// Get challenge by ID
    async fn get_challenge(
        &self,
        challenge_id: String,
    ) -> Result<Challenge, CaptchaError>;

    /// Clean up expired challenges
    async fn cleanup_expired_challenges(&self) -> Result<u64, CaptchaError>;
}

/// CAPTCHA service implementation
pub struct CaptchaService {
    /// Database operations for CAPTCHA challenges
    db_ops: Arc<crate::database::CaptchaOperations>,
    /// Challenge generator for creating CAPTCHA challenges
    generator: Arc<super::generator::ChallengeGenerator>,
    /// Validation engine for checking CAPTCHA responses
    validator: Arc<super::validator::ValidationEngine>,
    /// Behavioral analyzer for risk assessment
    analyzer: Arc<super::analyzer::BehavioralAnalyzer>,
    /// Metrics collector for performance monitoring
    metrics_collector: Arc<super::metrics::MetricsCollector>,
    /// Alert manager for security notifications
    alert_manager: Arc<super::alerting::AlertManager>,
}

impl CaptchaService {
    /// Create a new CAPTCHA service with all required components
    pub fn new(
        db_ops: Arc<crate::database::CaptchaOperations>,
        generator: Arc<super::generator::ChallengeGenerator>,
        validator: Arc<super::validator::ValidationEngine>,
        analyzer: Arc<super::analyzer::BehavioralAnalyzer>,
        metrics_collector: Arc<super::metrics::MetricsCollector>,
        alert_manager: Arc<super::alerting::AlertManager>,
    ) -> Self {
        Self {
            db_ops,
            generator,
            validator,
            analyzer,
            metrics_collector,
            alert_manager,
        }
    }

    /// Create a simple service instance for testing
    pub async fn simple() -> Self {
        // This would be used for testing or when full dependencies aren't available
        let db = crate::database::Database::mock().await;
        let db_ops = Arc::new(crate::database::CaptchaOperations::new(db));
        let generator = Arc::new(super::generator::ChallengeGenerator::new());
        let validator = Arc::new(super::validator::ValidationEngine::new());
        let analyzer = Arc::new(super::analyzer::BehavioralAnalyzer::new());
        let metrics_collector = Arc::new(super::metrics::MetricsCollector::new(db_ops.clone()));
        let alert_manager = Arc::new(super::alerting::AlertManager::new(metrics_collector.clone()));

        Self::new(db_ops, generator, validator, analyzer, metrics_collector, alert_manager)
    }
}

#[async_trait]
impl CaptchaServiceTrait for CaptchaService {
    async fn generate_challenge(
        &self,
        challenge_type: ChallengeType,
        difficulty: Option<u8>,
        session_id: Option<String>,
        ip_address: String,
    ) -> Result<Challenge, CaptchaError> {
        let start_time = std::time::Instant::now();

        // 1. Determine difficulty (use provided or get adaptive difficulty)
        let difficulty = if let Some(d) = difficulty {
            d
        } else {
            self.db_ops
                .get_difficulty_for_ip(&ip_address, session_id.as_deref())
                .await
                .map_err(|e| CaptchaError::DatabaseError {
                    message: e.to_string(),
                    transient: true,
                    retry_after: Some(Duration::from_secs(30)),
                })?
        };

        // 2. Generate challenge using the generator
        let mut challenge = self
            .generator
            .generate_challenge(challenge_type.clone(), difficulty, session_id.clone())
            .await
            .map_err(|e| CaptchaError::GenerationFailed {
                message: e.to_string(),
                recoverable: true,
                retry_after: Some(Duration::from_secs(1)),
            })?;

        // 3. Update challenge with correct IP address
        challenge.ip_address = ip_address.to_string();

        // 4. Store challenge in database
        self.db_ops
            .store_challenge(&challenge)
            .await
            .map_err(|e| CaptchaError::DatabaseError {
                message: e.to_string(),
                transient: true,
                retry_after: Some(Duration::from_secs(5)),
            })?;

        // 5. Record metrics
        let generation_time = start_time.elapsed();
        if let Err(e) = self.metrics_collector
            .record_challenge_generation(generation_time, &challenge_type, difficulty)
            .await
        {
            // Log error but don't fail the challenge generation
            eprintln!("Failed to record challenge generation metrics: {}", e);
        }

        Ok(challenge)
    }

    async fn validate_challenge(
        &self,
        challenge_id: String,
        answer: String,
        behavioral_data: Option<BehavioralMetrics>,
    ) -> Result<ValidationResult, CaptchaError> {
        let start_time = std::time::Instant::now();

        // 1. Retrieve challenge from database
        let challenge = self
            .db_ops
            .get_challenge(&challenge_id)
            .await
            .map_err(|e| CaptchaError::DatabaseError {
                message: e.to_string(),
                transient: true,
                retry_after: Some(Duration::from_secs(5)),
            })?
            .ok_or_else(|| CaptchaError::ChallengeNotFound {
                challenge_id: challenge_id.clone(),
                expired: false,
            })?;

        // 2. Check if challenge is expired
        if challenge.is_expired() {
            return Err(CaptchaError::ChallengeExpired {
                challenge_id,
                expired_at: format!("{:?}", challenge.expires_at),
            });
        }

        // 3. Validate answer using the validator
        let validation_result = self
            .validator
            .validate_response(&challenge, &answer)
            .await
            .map_err(|e| CaptchaError::ValidationFailed {
                message: e.to_string(),
                attempts_remaining: 0,
                next_difficulty: challenge.difficulty_level,
            })?;

        // 4. Analyze behavioral data if provided
        let mut risk_assessment = RiskLevel::Medium;
        let mut confidence_score = if validation_result { 0.9 } else { 0.1 }; // Default confidence based on validation result
        let mut behavioral_classification = BehaviorClassification::Unknown;

        if let Some(behavioral_data) = &behavioral_data {
            let behavioral_analysis = self
                .analyzer
                .analyze_behavior(behavioral_data)
                .await
                .map_err(|e| CaptchaError::ValidationFailed {
                    message: e.to_string(),
                    attempts_remaining: 0,
                    next_difficulty: challenge.difficulty_level,
                })?;

            // Adjust confidence and risk based on behavioral analysis
            confidence_score = (confidence_score + behavioral_analysis.confidence) / 2.0;
            risk_assessment = match behavioral_analysis.classification {
                BehaviorClassification::Human => RiskLevel::Low,
                BehaviorClassification::Suspicious => RiskLevel::Medium,
                BehaviorClassification::Bot => RiskLevel::High,
                BehaviorClassification::Unknown => RiskLevel::Medium,
            };
            behavioral_classification = behavioral_data.classification.clone();

            // Store behavioral metrics
            let _metrics_id = self
                .db_ops
                .store_behavioral_metrics(&challenge_id, behavioral_data)
                .await
                .map_err(|e| CaptchaError::DatabaseError {
                    message: e.to_string(),
                    transient: true,
                    retry_after: Some(Duration::from_secs(5)),
                })?;
        }

        // 5. Record validation attempt
        let _attempt_id = self
            .db_ops
            .record_validation_attempt(
                &challenge_id,
                &challenge.ip_address,
                None, // user_agent would come from request context
                &answer,
                validation_result,
                Some(confidence_score),
                &risk_assessment,
                None, // behavioral_metrics_id would be set if stored above
            )
            .await
            .map_err(|e| CaptchaError::DatabaseError {
                message: e.to_string(),
                transient: true,
                retry_after: Some(Duration::from_secs(30)),
            })?;

        // 6. Update challenge status
        self.db_ops
            .update_challenge_status(&challenge_id, validation_result)
            .await
            .map_err(|e| CaptchaError::DatabaseError {
                message: e.to_string(),
                transient: true,
                retry_after: Some(Duration::from_secs(30)),
            })?;

        // 7. Record validation metrics
        let validation_time = start_time.elapsed();
        if let Err(e) = self.metrics_collector
            .record_challenge_validation(
                validation_time,
                validation_result,
                &risk_assessment,
                &behavioral_classification,
            )
            .await
        {
            // Log error but don't fail the validation
            eprintln!("Failed to record challenge validation metrics: {}", e);
        }

        // 8. Check for security events that should trigger alerts
        if matches!(behavioral_classification, BehaviorClassification::Bot) {
            if let Err(e) = self.alert_manager
                .record_security_event(
                    super::SecurityEventType::SuspiciousBehavior,
                    &challenge.ip_address,
                    &risk_assessment,
                    None, // Country code would be determined from IP geolocation
                )
                .await
            {
                eprintln!("Failed to record security event: {}", e);
            }
        }

        // Check for attack patterns
        if matches!(risk_assessment, RiskLevel::Critical | RiskLevel::High) && !validation_result {
            if let Err(e) = self.alert_manager
                .record_security_event(
                    super::SecurityEventType::AttackAttempt,
                    &challenge.ip_address,
                    &risk_assessment,
                    None,
                )
                .await
            {
                eprintln!("Failed to record attack attempt: {}", e);
            }
        }

        // 9. Determine next difficulty and retry policy
        let next_difficulty = if validation_result {
            std::cmp::max(1, challenge.difficulty_level.saturating_sub(1))
        } else {
            std::cmp::min(10, challenge.difficulty_level + 1)
        };

        let retry_allowed = match risk_assessment {
            RiskLevel::Critical => false,
            RiskLevel::High => confidence_score > 0.3,
            _ => true,
        };

        let lockout_duration = if !retry_allowed {
            Some(std::time::Duration::from_secs(300)) // 5 minutes
        } else {
            None
        };

        Ok(ValidationResult {
            success: validation_result,
            confidence_score,
            risk_assessment,
            next_difficulty,
            retry_allowed,
            lockout_duration,
            message: if validation_result {
                "Challenge completed successfully".to_string()
            } else {
                "Challenge validation failed".to_string()
            },
        })
    }

    async fn refresh_challenge(
        &self,
        challenge_id: String,
    ) -> Result<Challenge, CaptchaError> {
        // 1. Get existing challenge
        let existing_challenge = self
            .db_ops
            .get_challenge(&challenge_id)
            .await
            .map_err(|e| CaptchaError::DatabaseError {
                message: e.to_string(),
                transient: true,
                retry_after: Some(Duration::from_secs(5)),
            })?
            .ok_or_else(|| CaptchaError::ChallengeNotFound {
                challenge_id: challenge_id.clone(),
                expired: false,
            })?;

        // 2. Generate new challenge with same parameters but increased difficulty
        let new_difficulty = std::cmp::min(10, existing_challenge.difficulty_level + 1);

        self.generate_challenge(
            existing_challenge.challenge_type,
            Some(new_difficulty),
            existing_challenge.session_id,
            existing_challenge.ip_address,
        )
        .await
    }

    async fn get_challenge(
        &self,
        challenge_id: String,
    ) -> Result<Challenge, CaptchaError> {
        self.db_ops
            .get_challenge(&challenge_id)
            .await
            .map_err(|e| CaptchaError::DatabaseError {
                message: e.to_string(),
                transient: true,
                retry_after: Some(Duration::from_secs(5)),
            })?
            .ok_or_else(|| CaptchaError::ChallengeNotFound {
                challenge_id,
                expired: false,
            })
    }

    async fn cleanup_expired_challenges(&self) -> Result<u64, CaptchaError> {
        self.db_ops
            .cleanup_expired_challenges()
            .await
            .map_err(|e| CaptchaError::DatabaseError {
                message: e.to_string(),
                transient: true,
                retry_after: Some(Duration::from_secs(30)),
            })
    }
}

impl CaptchaService {
    /// Get access to the metrics collector
    pub fn get_metrics_collector(&self) -> Arc<super::metrics::MetricsCollector> {
        self.metrics_collector.clone()
    }

    /// Get access to the database operations
    pub fn get_db_ops(&self) -> Arc<crate::database::CaptchaOperations> {
        self.db_ops.clone()
    }

    /// Get access to the alert manager
    pub fn get_alert_manager(&self) -> Arc<super::alerting::AlertManager> {
        self.alert_manager.clone()
    }

    /// Get real-time metrics summary
    pub async fn get_real_time_metrics(&self) -> Result<super::metrics::MetricsSummary, CaptchaError> {
        self.metrics_collector
            .get_real_time_metrics()
            .await
    }

    /// Generate metrics summary for a specific time window
    pub async fn generate_metrics_summary(&self, time_window_minutes: u64) -> Result<super::metrics::MetricsSummary, CaptchaError> {
        self.metrics_collector
            .generate_summary(time_window_minutes)
            .await
    }

    /// Record user experience metrics
    pub async fn record_user_experience(
        &self,
        completion_time: std::time::Duration,
        abandoned: bool,
        retry_count: u32,
        challenge_type: &ChallengeType,
        difficulty: u8,
        accessibility_used: bool,
    ) -> Result<(), CaptchaError> {
        self.metrics_collector
            .record_user_experience(
                completion_time,
                abandoned,
                retry_count,
                challenge_type,
                difficulty,
                accessibility_used,
            )
            .await
    }

    /// Record security event
    pub async fn record_security_event(
        &self,
        event_type: super::metrics::SecurityEventType,
        ip_address: &str,
        risk_level: &RiskLevel,
        country_code: Option<&str>,
    ) -> Result<(), CaptchaError> {
        self.metrics_collector
            .record_security_event(event_type, ip_address, risk_level, country_code)
            .await
    }

    /// Clean up old metrics from memory
    pub async fn cleanup_old_metrics(&self, retention_hours: u64) -> Result<(), CaptchaError> {
        self.metrics_collector
            .cleanup_old_metrics(retention_hours)
            .await
    }

    /// Start the alerting system
    pub async fn start_alerting(&self) -> Result<(), CaptchaError> {
        self.alert_manager.start().await
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<super::dashboard::Alert> {
        self.alert_manager
            .get_alerting_engine()
            .get_active_alerts()
            .await
    }

    /// Trigger a test alert
    pub async fn trigger_test_alert(
        &self,
        severity: super::dashboard::AlertSeverity,
        title: String,
        description: String,
    ) -> Result<(), CaptchaError> {
        self.alert_manager
            .trigger_test_alert(severity, title, description)
            .await
    }

    /// Check system health and trigger alerts if needed
    pub async fn check_system_health(&self) -> Result<(), CaptchaError> {
        self.alert_manager.check_system_health().await
    }

    /// Add a new alert rule
    pub async fn add_alert_rule(&self, rule: super::alerting::AlertRule) -> Result<(), CaptchaError> {
        self.alert_manager
            .get_alerting_engine()
            .add_rule(rule)
            .await
    }

    /// Get all alert rules
    pub async fn get_alert_rules(&self) -> Vec<super::alerting::AlertRule> {
        self.alert_manager
            .get_alerting_engine()
            .get_rules()
            .await
    }
}
