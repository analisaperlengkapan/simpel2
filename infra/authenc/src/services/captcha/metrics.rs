//! CAPTCHA Metrics Collection System
//!
//! Comprehensive metrics collection for CAPTCHA performance, security, and user experience

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::error::CaptchaError;
use super::types::*;

/// CAPTCHA performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Unique identifier for this metrics snapshot
    pub metric_id: String,
    /// Timestamp when metrics were collected
    pub timestamp: SystemTime,
    /// Average time to generate challenges in milliseconds
    pub challenge_generation_latency_ms: u64,
    /// Average time to validate challenges in milliseconds
    pub validation_latency_ms: u64,
    /// Rate of successful challenge completions (0.0 to 1.0)
    pub success_rate: f64,
    /// Rate of failed challenge attempts (0.0 to 1.0)
    pub failure_rate: f64,
    /// Average difficulty level of challenges
    pub average_difficulty: f64,
    /// Number of concurrent challenges being processed
    pub concurrent_challenges: u64,
    /// Memory usage in megabytes
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
}

/// Bot detection accuracy metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotDetectionMetrics {
    /// Unique identifier for this metrics snapshot
    pub metric_id: String,
    /// Timestamp when metrics were collected
    pub timestamp: SystemTime,
    /// Total number of bot detections attempted
    pub total_detections: u64,
    /// Number of correctly identified bots
    pub true_positives: u64,
    /// Number of incorrectly flagged humans as bots
    pub false_positives: u64,
    /// Number of correctly identified humans
    pub true_negatives: u64,
    /// Number of bots that were not detected
    pub false_negatives: u64,
    /// Overall accuracy rate (0.0 to 1.0)
    pub accuracy_rate: f64,
    /// Precision of bot detection (0.0 to 1.0)
    pub precision: f64,
    /// Recall of bot detection (0.0 to 1.0)
    pub recall: f64,
    /// F1 score combining precision and recall (0.0 to 1.0)
    pub f1_score: f64,
    /// Distribution of detected risks by level
    pub risk_distribution: HashMap<RiskLevel, u64>,
}

/// User experience metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserExperienceMetrics {
    /// Unique identifier for this metrics snapshot
    pub metric_id: String,
    /// Timestamp when metrics were collected
    pub timestamp: SystemTime,
    /// Average time users take to complete challenges in milliseconds
    pub average_completion_time_ms: u64,
    /// Rate of users abandoning challenges (0.0 to 1.0)
    pub abandonment_rate: f64,
    /// Rate of users retrying after failure (0.0 to 1.0)
    pub retry_rate: f64,
    /// Rate of accessibility features usage (0.0 to 1.0)
    pub accessibility_usage_rate: f64,
    /// User satisfaction score (0.0 to 1.0)
    pub user_satisfaction_score: f64,
    /// User preferences for different challenge types
    pub challenge_type_preferences: HashMap<ChallengeType, u64>,
    /// Distribution of challenge difficulties
    pub difficulty_distribution: HashMap<u8, u64>,
}

/// Security event metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEventMetrics {
    /// Unique identifier for this metrics snapshot
    pub metric_id: String,
    /// Timestamp when metrics were collected
    pub timestamp: SystemTime,
    /// Number of detected attack attempts
    pub attack_attempts: u64,
    /// Number of IP addresses blocked
    pub blocked_ips: u64,
    /// Number of rate limit triggers
    pub rate_limit_triggers: u64,
    /// Number of user account lockouts
    pub lockout_events: u64,
    /// Number of suspicious behavior detections
    pub suspicious_behavior_count: u64,
    /// Distribution of threats by risk level
    pub threat_level_distribution: HashMap<RiskLevel, u64>,
    /// Geographic distribution of security events
    pub geographic_distribution: HashMap<String, u64>,
}

/// Aggregated metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Timestamp when summary was generated
    pub timestamp: SystemTime,
    /// Time window covered by this summary in minutes
    pub time_window_minutes: u64,
    /// Performance metrics for the time window
    pub performance: PerformanceMetrics,
    /// Bot detection metrics for the time window
    pub bot_detection: BotDetectionMetrics,
    /// User experience metrics for the time window
    pub user_experience: UserExperienceMetrics,
    /// Security event metrics for the time window
    pub security_events: SecurityEventMetrics,
}

/// Real-time metrics collector
pub struct MetricsCollector {
    performance_buffer: Arc<RwLock<Vec<PerformanceMetrics>>>,
    bot_detection_buffer: Arc<RwLock<Vec<BotDetectionMetrics>>>,
    user_experience_buffer: Arc<RwLock<Vec<UserExperienceMetrics>>>,
    security_events_buffer: Arc<RwLock<Vec<SecurityEventMetrics>>>,
    db_ops: Arc<crate::database::CaptchaOperations>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(db_ops: Arc<crate::database::CaptchaOperations>) -> Self {
        Self {
            performance_buffer: Arc::new(RwLock::new(Vec::new())),
            bot_detection_buffer: Arc::new(RwLock::new(Vec::new())),
            user_experience_buffer: Arc::new(RwLock::new(Vec::new())),
            security_events_buffer: Arc::new(RwLock::new(Vec::new())),
            db_ops,
        }
    }

    /// Record challenge generation metrics
    pub async fn record_challenge_generation(
        &self,
        generation_time: Duration,
        challenge_type: &ChallengeType,
        difficulty: u8,
    ) -> Result<(), CaptchaError> {
        let metric = PerformanceMetrics {
            metric_id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            challenge_generation_latency_ms: generation_time.as_millis() as u64,
            validation_latency_ms: 0, // Will be updated during validation
            success_rate: 0.0, // Will be calculated in aggregation
            failure_rate: 0.0, // Will be calculated in aggregation
            average_difficulty: difficulty as f64,
            concurrent_challenges: self.get_concurrent_challenges_count().await?,
            memory_usage_mb: self.get_memory_usage(),
            cpu_usage_percent: self.get_cpu_usage(),
        };

        let mut buffer = self.performance_buffer.write().await;
        buffer.push(metric);

        // Store in database for persistence
        self.db_ops.store_performance_metrics(&buffer.last().unwrap()).await
            .map_err(|e| CaptchaError::DatabaseError {
                message: e.to_string(),
                transient: true,
                retry_after: Some(Duration::from_secs(2)),
            })?;

        Ok(())
    }

    /// Record challenge validation metrics
    pub async fn record_challenge_validation(
        &self,
        validation_time: Duration,
        success: bool,
        risk_level: &RiskLevel,
        behavioral_classification: &BehaviorClassification,
    ) -> Result<(), CaptchaError> {
        // Update performance metrics
        let performance_metric = PerformanceMetrics {
            metric_id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            challenge_generation_latency_ms: 0,
            validation_latency_ms: validation_time.as_millis() as u64,
            success_rate: if success { 1.0 } else { 0.0 },
            failure_rate: if success { 0.0 } else { 1.0 },
            average_difficulty: 0.0, // Will be calculated in aggregation
            concurrent_challenges: self.get_concurrent_challenges_count().await?,
            memory_usage_mb: self.get_memory_usage(),
            cpu_usage_percent: self.get_cpu_usage(),
        };

        let mut perf_buffer = self.performance_buffer.write().await;
        perf_buffer.push(performance_metric);

        // Record bot detection metrics
        let is_bot = matches!(behavioral_classification, BehaviorClassification::Bot);
        let is_suspicious = matches!(behavioral_classification, BehaviorClassification::Suspicious);

        let bot_metric = BotDetectionMetrics {
            metric_id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            total_detections: 1,
            true_positives: if is_bot && !success { 1 } else { 0 },
            false_positives: if is_bot && success { 1 } else { 0 },
            true_negatives: if !is_bot && success { 1 } else { 0 },
            false_negatives: if !is_bot && !success { 1 } else { 0 },
            accuracy_rate: 0.0, // Will be calculated in aggregation
            precision: 0.0, // Will be calculated in aggregation
            recall: 0.0, // Will be calculated in aggregation
            f1_score: 0.0, // Will be calculated in aggregation
            risk_distribution: {
                let mut dist = HashMap::new();
                dist.insert(risk_level.clone(), 1);
                dist
            },
        };

        let mut bot_buffer = self.bot_detection_buffer.write().await;
        bot_buffer.push(bot_metric);

        // Store in database
        self.db_ops.store_bot_detection_metrics(&bot_buffer.last().unwrap()).await
            .map_err(|e| CaptchaError::DatabaseError {
                message: e.to_string(),
                transient: true,
                retry_after: Some(Duration::from_secs(30)),
            })?;

        Ok(())
    }

    /// Record user experience metrics
    pub async fn record_user_experience(
        &self,
        completion_time: Duration,
        abandoned: bool,
        retry_count: u32,
        challenge_type: &ChallengeType,
        difficulty: u8,
        accessibility_used: bool,
    ) -> Result<(), CaptchaError> {
        let metric = UserExperienceMetrics {
            metric_id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            average_completion_time_ms: completion_time.as_millis() as u64,
            abandonment_rate: if abandoned { 1.0 } else { 0.0 },
            retry_rate: if retry_count > 0 { 1.0 } else { 0.0 },
            accessibility_usage_rate: if accessibility_used { 1.0 } else { 0.0 },
            user_satisfaction_score: self.calculate_satisfaction_score(completion_time, retry_count, abandoned),
            challenge_type_preferences: {
                let mut prefs = HashMap::new();
                prefs.insert(challenge_type.clone(), 1);
                prefs
            },
            difficulty_distribution: {
                let mut dist = HashMap::new();
                dist.insert(difficulty, 1);
                dist
            },
        };

        let mut buffer = self.user_experience_buffer.write().await;
        buffer.push(metric);

        // Store in database
        self.db_ops.store_user_experience_metrics(&buffer.last().unwrap()).await
            .map_err(|e| CaptchaError::DatabaseError {
                message: e.to_string(),
                transient: true,
                retry_after: Some(Duration::from_secs(30)),
            })?;

        Ok(())
    }

    /// Record security event
    pub async fn record_security_event(
        &self,
        event_type: SecurityEventType,
        ip_address: &str,
        risk_level: &RiskLevel,
        country_code: Option<&str>,
    ) -> Result<(), CaptchaError> {
        let metric = SecurityEventMetrics {
            metric_id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            attack_attempts: if matches!(event_type, SecurityEventType::AttackAttempt) { 1 } else { 0 },
            blocked_ips: if matches!(event_type, SecurityEventType::IpBlocked) { 1 } else { 0 },
            rate_limit_triggers: if matches!(event_type, SecurityEventType::RateLimitTriggered) { 1 } else { 0 },
            lockout_events: if matches!(event_type, SecurityEventType::UserLockedOut) { 1 } else { 0 },
            suspicious_behavior_count: if matches!(event_type, SecurityEventType::SuspiciousBehavior) { 1 } else { 0 },
            threat_level_distribution: {
                let mut dist = HashMap::new();
                dist.insert(risk_level.clone(), 1);
                dist
            },
            geographic_distribution: {
                let mut dist = HashMap::new();
                if let Some(country) = country_code {
                    dist.insert(country.to_string(), 1);
                }
                dist
            },
        };

        let mut buffer = self.security_events_buffer.write().await;
        buffer.push(metric);

        // Store in database
        self.db_ops.store_security_event_metrics(&buffer.last().unwrap()).await
            .map_err(|e| CaptchaError::DatabaseError {
                message: e.to_string(),
                transient: true,
                retry_after: Some(Duration::from_secs(30)),
            })?;

        Ok(())
    }

    /// Generate aggregated metrics summary
    pub async fn generate_summary(&self, time_window_minutes: u64) -> Result<MetricsSummary, CaptchaError> {
        let cutoff_time = SystemTime::now() - Duration::from_secs(time_window_minutes * 60);

        // Aggregate performance metrics
        let perf_buffer = self.performance_buffer.read().await;
        let recent_perf: Vec<_> = perf_buffer.iter()
            .filter(|m| m.timestamp > cutoff_time)
            .collect();

        let performance = self.aggregate_performance_metrics(&recent_perf);

        // Aggregate bot detection metrics
        let bot_buffer = self.bot_detection_buffer.read().await;
        let recent_bot: Vec<_> = bot_buffer.iter()
            .filter(|m| m.timestamp > cutoff_time)
            .collect();

        let bot_detection = self.aggregate_bot_detection_metrics(&recent_bot);

        // Aggregate user experience metrics
        let ux_buffer = self.user_experience_buffer.read().await;
        let recent_ux: Vec<_> = ux_buffer.iter()
            .filter(|m| m.timestamp > cutoff_time)
            .collect();

        let user_experience = self.aggregate_user_experience_metrics(&recent_ux);

        // Aggregate security event metrics
        let security_buffer = self.security_events_buffer.read().await;
        let recent_security: Vec<_> = security_buffer.iter()
            .filter(|m| m.timestamp > cutoff_time)
            .collect();

        let security_events = self.aggregate_security_event_metrics(&recent_security);

        Ok(MetricsSummary {
            timestamp: SystemTime::now(),
            time_window_minutes,
            performance,
            bot_detection,
            user_experience,
            security_events,
        })
    }

    /// Get current metrics for real-time monitoring
    pub async fn get_real_time_metrics(&self) -> Result<MetricsSummary, CaptchaError> {
        self.generate_summary(5).await // Last 5 minutes
    }

    /// Clear old metrics from memory buffers
    pub async fn cleanup_old_metrics(&self, retention_hours: u64) -> Result<(), CaptchaError> {
        let cutoff_time = SystemTime::now() - Duration::from_secs(retention_hours * 3600);

        let mut perf_buffer = self.performance_buffer.write().await;
        perf_buffer.retain(|m| m.timestamp > cutoff_time);

        let mut bot_buffer = self.bot_detection_buffer.write().await;
        bot_buffer.retain(|m| m.timestamp > cutoff_time);

        let mut ux_buffer = self.user_experience_buffer.write().await;
        ux_buffer.retain(|m| m.timestamp > cutoff_time);

        let mut security_buffer = self.security_events_buffer.write().await;
        security_buffer.retain(|m| m.timestamp > cutoff_time);

        Ok(())
    }

    // Private helper methods
    async fn get_concurrent_challenges_count(&self) -> Result<u64, CaptchaError> {
        self.db_ops.get_active_challenges_count().await
            .map_err(|e| CaptchaError::DatabaseError {
                message: e.to_string(),
                transient: true,
                retry_after: Some(Duration::from_secs(30)),
            })
    }

    fn get_memory_usage(&self) -> f64 {
        // Placeholder - would integrate with system monitoring
        0.0
    }

    fn get_cpu_usage(&self) -> f64 {
        // Placeholder - would integrate with system monitoring
        0.0
    }

    fn calculate_satisfaction_score(&self, completion_time: Duration, retry_count: u32, abandoned: bool) -> f64 {
        if abandoned {
            return 0.0;
        }

        let base_score = 1.0;
        let time_penalty = (completion_time.as_secs() as f64 / 60.0) * 0.1; // Penalty for long completion times
        let retry_penalty = retry_count as f64 * 0.2; // Penalty for retries

        (base_score - time_penalty - retry_penalty).max(0.0).min(1.0)
    }

    fn aggregate_performance_metrics(&self, metrics: &[&PerformanceMetrics]) -> PerformanceMetrics {
        if metrics.is_empty() {
            return PerformanceMetrics {
                metric_id: Uuid::new_v4().to_string(),
                timestamp: SystemTime::now(),
                challenge_generation_latency_ms: 0,
                validation_latency_ms: 0,
                success_rate: 0.0,
                failure_rate: 0.0,
                average_difficulty: 0.0,
                concurrent_challenges: 0,
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
            };
        }

        let count = metrics.len() as f64;
        let total_gen_latency: u64 = metrics.iter().map(|m| m.challenge_generation_latency_ms).sum();
        let total_val_latency: u64 = metrics.iter().map(|m| m.validation_latency_ms).sum();
        let total_success_rate: f64 = metrics.iter().map(|m| m.success_rate).sum();
        let total_failure_rate: f64 = metrics.iter().map(|m| m.failure_rate).sum();
        let total_difficulty: f64 = metrics.iter().map(|m| m.average_difficulty).sum();
        let avg_concurrent: u64 = (metrics.iter().map(|m| m.concurrent_challenges).sum::<u64>() as f64 / count) as u64;
        let avg_memory: f64 = metrics.iter().map(|m| m.memory_usage_mb).sum::<f64>() / count;
        let avg_cpu: f64 = metrics.iter().map(|m| m.cpu_usage_percent).sum::<f64>() / count;

        PerformanceMetrics {
            metric_id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            challenge_generation_latency_ms: (total_gen_latency as f64 / count) as u64,
            validation_latency_ms: (total_val_latency as f64 / count) as u64,
            success_rate: total_success_rate / count,
            failure_rate: total_failure_rate / count,
            average_difficulty: total_difficulty / count,
            concurrent_challenges: avg_concurrent,
            memory_usage_mb: avg_memory,
            cpu_usage_percent: avg_cpu,
        }
    }

    fn aggregate_bot_detection_metrics(&self, metrics: &[&BotDetectionMetrics]) -> BotDetectionMetrics {
        if metrics.is_empty() {
            return BotDetectionMetrics {
                metric_id: Uuid::new_v4().to_string(),
                timestamp: SystemTime::now(),
                total_detections: 0,
                true_positives: 0,
                false_positives: 0,
                true_negatives: 0,
                false_negatives: 0,
                accuracy_rate: 0.0,
                precision: 0.0,
                recall: 0.0,
                f1_score: 0.0,
                risk_distribution: HashMap::new(),
            };
        }

        let total_detections: u64 = metrics.iter().map(|m| m.total_detections).sum();
        let true_positives: u64 = metrics.iter().map(|m| m.true_positives).sum();
        let false_positives: u64 = metrics.iter().map(|m| m.false_positives).sum();
        let true_negatives: u64 = metrics.iter().map(|m| m.true_negatives).sum();
        let false_negatives: u64 = metrics.iter().map(|m| m.false_negatives).sum();

        let total = true_positives + false_positives + true_negatives + false_negatives;
        let accuracy_rate = if total > 0 {
            (true_positives + true_negatives) as f64 / total as f64
        } else {
            0.0
        };

        let precision = if (true_positives + false_positives) > 0 {
            true_positives as f64 / (true_positives + false_positives) as f64
        } else {
            0.0
        };

        let recall = if (true_positives + false_negatives) > 0 {
            true_positives as f64 / (true_positives + false_negatives) as f64
        } else {
            0.0
        };

        let f1_score = if (precision + recall) > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        // Aggregate risk distribution
        let mut risk_distribution = HashMap::new();
        for metric in metrics {
            for (risk_level, count) in &metric.risk_distribution {
                *risk_distribution.entry(risk_level.clone()).or_insert(0) += count;
            }
        }

        BotDetectionMetrics {
            metric_id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            total_detections,
            true_positives,
            false_positives,
            true_negatives,
            false_negatives,
            accuracy_rate,
            precision,
            recall,
            f1_score,
            risk_distribution,
        }
    }

    fn aggregate_user_experience_metrics(&self, metrics: &[&UserExperienceMetrics]) -> UserExperienceMetrics {
        if metrics.is_empty() {
            return UserExperienceMetrics {
                metric_id: Uuid::new_v4().to_string(),
                timestamp: SystemTime::now(),
                average_completion_time_ms: 0,
                abandonment_rate: 0.0,
                retry_rate: 0.0,
                accessibility_usage_rate: 0.0,
                user_satisfaction_score: 0.0,
                challenge_type_preferences: HashMap::new(),
                difficulty_distribution: HashMap::new(),
            };
        }

        let count = metrics.len() as f64;
        let avg_completion_time: u64 = (metrics.iter().map(|m| m.average_completion_time_ms).sum::<u64>() as f64 / count) as u64;
        let abandonment_rate: f64 = metrics.iter().map(|m| m.abandonment_rate).sum::<f64>() / count;
        let retry_rate: f64 = metrics.iter().map(|m| m.retry_rate).sum::<f64>() / count;
        let accessibility_usage_rate: f64 = metrics.iter().map(|m| m.accessibility_usage_rate).sum::<f64>() / count;
        let user_satisfaction_score: f64 = metrics.iter().map(|m| m.user_satisfaction_score).sum::<f64>() / count;

        // Aggregate challenge type preferences
        let mut challenge_type_preferences = HashMap::new();
        for metric in metrics {
            for (challenge_type, count) in &metric.challenge_type_preferences {
                *challenge_type_preferences.entry(challenge_type.clone()).or_insert(0) += count;
            }
        }

        // Aggregate difficulty distribution
        let mut difficulty_distribution = HashMap::new();
        for metric in metrics {
            for (difficulty, count) in &metric.difficulty_distribution {
                *difficulty_distribution.entry(*difficulty).or_insert(0) += count;
            }
        }

        UserExperienceMetrics {
            metric_id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            average_completion_time_ms: avg_completion_time,
            abandonment_rate,
            retry_rate,
            accessibility_usage_rate,
            user_satisfaction_score,
            challenge_type_preferences,
            difficulty_distribution,
        }
    }

    fn aggregate_security_event_metrics(&self, metrics: &[&SecurityEventMetrics]) -> SecurityEventMetrics {
        if metrics.is_empty() {
            return SecurityEventMetrics {
                metric_id: Uuid::new_v4().to_string(),
                timestamp: SystemTime::now(),
                attack_attempts: 0,
                blocked_ips: 0,
                rate_limit_triggers: 0,
                lockout_events: 0,
                suspicious_behavior_count: 0,
                threat_level_distribution: HashMap::new(),
                geographic_distribution: HashMap::new(),
            };
        }

        let attack_attempts: u64 = metrics.iter().map(|m| m.attack_attempts).sum();
        let blocked_ips: u64 = metrics.iter().map(|m| m.blocked_ips).sum();
        let rate_limit_triggers: u64 = metrics.iter().map(|m| m.rate_limit_triggers).sum();
        let lockout_events: u64 = metrics.iter().map(|m| m.lockout_events).sum();
        let suspicious_behavior_count: u64 = metrics.iter().map(|m| m.suspicious_behavior_count).sum();

        // Aggregate threat level distribution
        let mut threat_level_distribution = HashMap::new();
        for metric in metrics {
            for (threat_level, count) in &metric.threat_level_distribution {
                *threat_level_distribution.entry(threat_level.clone()).or_insert(0) += count;
            }
        }

        // Aggregate geographic distribution
        let mut geographic_distribution = HashMap::new();
        for metric in metrics {
            for (country, count) in &metric.geographic_distribution {
                *geographic_distribution.entry(country.clone()).or_insert(0) += count;
            }
        }

        SecurityEventMetrics {
            metric_id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            attack_attempts,
            blocked_ips,
            rate_limit_triggers,
            lockout_events,
            suspicious_behavior_count,
            threat_level_distribution,
            geographic_distribution,
        }
    }
}

/// Security event types for metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityEventType {
    /// Attempted attack on the CAPTCHA system
    AttackAttempt,
    /// IP address blocked due to suspicious activity
    IpBlocked,
    /// Rate limit triggered for excessive requests
    RateLimitTriggered,
    /// User account locked out due to security policy
    UserLockedOut,
    /// Suspicious behavior detected
    SuspiciousBehavior,
}
