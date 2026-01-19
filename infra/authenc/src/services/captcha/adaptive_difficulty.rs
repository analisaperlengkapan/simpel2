//! Adaptive difficulty system for CAPTCHA challenges
//!
//! Dynamically adjusts challenge difficulty based on user behavior and threat assessment

use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// User behavior history for difficulty calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBehaviorHistory {
    /// User session ID
    pub session_id: String,
    /// IP address
    pub ip_address: String,
    /// Number of failed attempts
    pub failed_attempts: u32,
    /// Number of successful attempts
    pub successful_attempts: u32,
    /// Average completion time in seconds
    pub avg_completion_time: f64,
    /// Behavioral risk scores over time
    pub risk_scores: Vec<f64>,
    /// Last challenge timestamp
    pub last_challenge_at: SystemTime,
    /// Consecutive failures
    pub consecutive_failures: u32,
    /// User agent string
    pub user_agent: Option<String>,
}

/// Threat assessment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAssessment {
    /// Overall risk level
    pub risk_level: RiskLevel,
    /// Risk score (0.0 to 1.0)
    pub risk_score: f64,
    /// Threat indicators
    pub indicators: Vec<ThreatIndicator>,
    /// Assessment timestamp
    pub assessed_at: SystemTime,
}

/// Individual threat indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    /// Type of indicator
    pub indicator_type: String,
    /// Severity (0.0 to 1.0)
    pub severity: f64,
    /// Description
    pub description: String,
}

/// Difficulty calculation parameters
#[derive(Debug, Clone)]
pub struct DifficultyParameters {
    /// Base difficulty level (1-10)
    pub base_difficulty: u8,
    /// Maximum difficulty level (1-10)
    pub max_difficulty: u8,
    /// Minimum difficulty level (1-10)
    pub min_difficulty: u8,
    /// Failure penalty multiplier
    pub failure_penalty: f64,
    /// Success reward multiplier
    pub success_reward: f64,
    /// Risk score weight in calculation
    pub risk_weight: f64,
    /// Time-based decay factor
    pub decay_factor: f64,
}

impl Default for DifficultyParameters {
    fn default() -> Self {
        Self {
            base_difficulty: 3,
            max_difficulty: 10,
            min_difficulty: 1,
            failure_penalty: 1.5,
            success_reward: 0.8,
            risk_weight: 2.0,
            decay_factor: 0.95,
        }
    }
}

/// Adaptive difficulty calculator
pub struct AdaptiveDifficultyCalculator {
    params: DifficultyParameters,
    /// User behavior history cache
    behavior_cache: HashMap<String, UserBehaviorHistory>,
}

impl AdaptiveDifficultyCalculator {
    /// Create a new adaptive difficulty calculator
    /// Configuration parameters
    pub fn new(params: DifficultyParameters) -> Self {
        Self {
            params,
            behavior_cache: HashMap::new(),
        }
    }

    /// Calculate difficulty for a user based on their behavior and threat assessment
    pub fn calculate_difficulty(
        &mut self,
        session_id: &str,
        ip_address: &str,
        behavioral_metrics: Option<&BehavioralMetrics>,
        threat_assessment: Option<&ThreatAssessment>,
    ) -> u8 {
        // Get or create user behavior history
        let behavior_key = format!("{}:{}", session_id, ip_address);
        let behavior = self
            .behavior_cache
            .entry(behavior_key.clone())
            .or_insert_with(|| UserBehaviorHistory {
                session_id: session_id.to_string(),
                ip_address: ip_address.to_string(),
                failed_attempts: 0,
                successful_attempts: 0,
                avg_completion_time: 0.0,
                risk_scores: Vec::new(),
                last_challenge_at: SystemTime::now(),
                consecutive_failures: 0,
                user_agent: None,
            });

        // Start with base difficulty
        let mut difficulty = self.params.base_difficulty as f64;

        // Apply failure penalty
        if behavior.consecutive_failures > 0 {
            let penalty = (behavior.consecutive_failures as f64) * self.params.failure_penalty;
            difficulty += penalty;
        }

        // Apply success reward (reduce difficulty for consistent success)
        if behavior.successful_attempts > behavior.failed_attempts
            && behavior.consecutive_failures == 0
        {
            let reward = self.params.success_reward;
            difficulty -= reward;
        }

        // Apply behavioral risk assessment
        if let Some(metrics) = behavioral_metrics {
            let risk_adjustment = metrics.risk_score * self.params.risk_weight;
            difficulty += risk_adjustment;
        }

        // Apply threat assessment
        if let Some(assessment) = threat_assessment {
            let threat_adjustment = match assessment.risk_level {
                RiskLevel::Low => 0.0,
                RiskLevel::Medium => 1.0,
                RiskLevel::High => 2.5,
                RiskLevel::Critical => 4.0,
            };
            difficulty += threat_adjustment;
        }

        // Apply time-based decay (reduce difficulty over time for legitimate users)
        let time_since_last = SystemTime::now()
            .duration_since(behavior.last_challenge_at)
            .unwrap_or(Duration::from_secs(0))
            .as_secs() as f64;

        if time_since_last > 300.0 && behavior.consecutive_failures == 0 {
            // Reduce difficulty if user has been away and had no recent failures
            let decay = (time_since_last / 3600.0) * self.params.decay_factor;
            difficulty -= decay;
        }

        // Clamp to valid range
        let final_difficulty = difficulty
            .max(self.params.min_difficulty as f64)
            .min(self.params.max_difficulty as f64)
            .round() as u8;

        final_difficulty
    }

    /// Update user behavior after challenge completion
    pub fn update_behavior(
        &mut self,
        session_id: &str,
        ip_address: &str,
        success: bool,
        completion_time: Duration,
        risk_score: Option<f64>,
        user_agent: Option<String>,
    ) {
        let behavior_key = format!("{}:{}", session_id, ip_address);
        let behavior =
            self.behavior_cache
                .entry(behavior_key)
                .or_insert_with(|| UserBehaviorHistory {
                    session_id: session_id.to_string(),
                    ip_address: ip_address.to_string(),
                    failed_attempts: 0,
                    successful_attempts: 0,
                    avg_completion_time: 0.0,
                    risk_scores: Vec::new(),
                    last_challenge_at: SystemTime::now(),
                    consecutive_failures: 0,
                    user_agent: None,
                });

        // Update attempt counts
        if success {
            behavior.successful_attempts += 1;
            behavior.consecutive_failures = 0;
        } else {
            behavior.failed_attempts += 1;
            behavior.consecutive_failures += 1;
        }

        // Update average completion time
        let completion_secs = completion_time.as_secs_f64();
        if behavior.avg_completion_time == 0.0 {
            behavior.avg_completion_time = completion_secs;
        } else {
            // Exponential moving average
            behavior.avg_completion_time =
                0.7 * behavior.avg_completion_time + 0.3 * completion_secs;
        }

        // Update risk scores
        if let Some(score) = risk_score {
            behavior.risk_scores.push(score);
            // Keep only last 10 scores
            if behavior.risk_scores.len() > 10 {
                behavior.risk_scores.remove(0);
            }
        }

        // Update metadata
        behavior.last_challenge_at = SystemTime::now();
        if user_agent.is_some() {
            behavior.user_agent = user_agent;
        }
    }

    /// Perform threat assessment based on behavioral patterns
    pub fn assess_threat(&self, session_id: &str, ip_address: &str) -> ThreatAssessment {
        let behavior_key = format!("{}:{}", session_id, ip_address);
        let mut indicators = Vec::new();
        let mut risk_score = 0.0;

        if let Some(behavior) = self.behavior_cache.get(&behavior_key) {
            // Check for high failure rate
            let total_attempts = behavior.successful_attempts + behavior.failed_attempts;
            if total_attempts > 0 {
                let failure_rate = behavior.failed_attempts as f64 / total_attempts as f64;
                if failure_rate > 0.7 {
                    indicators.push(ThreatIndicator {
                        indicator_type: "high_failure_rate".to_string(),
                        severity: failure_rate,
                        description: format!("High failure rate: {:.1}%", failure_rate * 100.0),
                    });
                    risk_score += failure_rate * 0.4;
                }
            }

            // Check for consecutive failures
            if behavior.consecutive_failures > 5 {
                let severity = (behavior.consecutive_failures as f64 / 20.0).min(1.0);
                indicators.push(ThreatIndicator {
                    indicator_type: "consecutive_failures".to_string(),
                    severity,
                    description: format!("Consecutive failures: {}", behavior.consecutive_failures),
                });
                risk_score += severity * 0.3;
            }

            // Check for unusually fast completion times
            if behavior.avg_completion_time > 0.0 && behavior.avg_completion_time < 2.0 {
                let severity = (2.0 - behavior.avg_completion_time) / 2.0;
                indicators.push(ThreatIndicator {
                    indicator_type: "fast_completion".to_string(),
                    severity,
                    description: format!(
                        "Unusually fast completion: {:.1}s",
                        behavior.avg_completion_time
                    ),
                });
                risk_score += severity * 0.3;
            }

            // Check behavioral risk scores
            if !behavior.risk_scores.is_empty() {
                let avg_risk =
                    behavior.risk_scores.iter().sum::<f64>() / behavior.risk_scores.len() as f64;
                if avg_risk > 0.7 {
                    indicators.push(ThreatIndicator {
                        indicator_type: "high_behavioral_risk".to_string(),
                        severity: avg_risk,
                        description: format!("High behavioral risk score: {:.2}", avg_risk),
                    });
                    risk_score += avg_risk * 0.4;
                }
            }
        }

        // Determine risk level
        let risk_level = match risk_score {
            x if x < 0.3 => RiskLevel::Low,
            x if x < 0.6 => RiskLevel::Medium,
            x if x < 0.8 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        ThreatAssessment {
            risk_level,
            risk_score: risk_score.min(1.0),
            indicators,
            assessed_at: SystemTime::now(),
        }
    }

    /// Clean up old behavior data
    pub fn cleanup_old_data(&mut self, max_age: Duration) {
        let cutoff = SystemTime::now() - max_age;
        self.behavior_cache
            .retain(|_, behavior| behavior.last_challenge_at > cutoff);
    }

    /// Get behavior statistics for monitoring
    pub fn get_behavior_stats(
        &self,
        session_id: &str,
        ip_address: &str,
    ) -> Option<&UserBehaviorHistory> {
        let behavior_key = format!("{}:{}", session_id, ip_address);
        self.behavior_cache.get(&behavior_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_difficulty_calculator_creation() {
        let params = DifficultyParameters::default();
        let calculator = AdaptiveDifficultyCalculator::new(params);

        assert_eq!(calculator.params.base_difficulty, 3);
        assert_eq!(calculator.params.max_difficulty, 10);
        assert_eq!(calculator.params.min_difficulty, 1);
    }

    #[test]
    fn test_difficulty_calculation_base_case() {
        let mut calculator = AdaptiveDifficultyCalculator::new(DifficultyParameters::default());

        let difficulty = calculator.calculate_difficulty("test_session", "127.0.0.1", None, None);

        // Should return base difficulty for new user
        assert_eq!(difficulty, 3);
    }

    #[test]
    fn test_difficulty_increases_with_failures() {
        let mut calculator = AdaptiveDifficultyCalculator::new(DifficultyParameters::default());

        // Simulate multiple failures
        for _ in 0..3 {
            calculator.update_behavior(
                "test_session",
                "127.0.0.1",
                false, // failure
                Duration::from_secs(10),
                Some(0.8),
                None,
            );
        }

        let difficulty = calculator.calculate_difficulty("test_session", "127.0.0.1", None, None);

        // Should be higher than base difficulty due to failures
        assert!(difficulty > 3);
    }

    #[test]
    fn test_threat_assessment() {
        let mut calculator = AdaptiveDifficultyCalculator::new(DifficultyParameters::default());

        // Simulate suspicious behavior
        for _ in 0..10 {
            calculator.update_behavior(
                "test_session",
                "127.0.0.1",
                false,                  // failure
                Duration::from_secs(1), // very fast
                Some(0.9),              // high risk
                None,
            );
        }

        let assessment = calculator.assess_threat("test_session", "127.0.0.1");

        assert!(matches!(
            assessment.risk_level,
            RiskLevel::High | RiskLevel::Critical
        ));
        assert!(assessment.risk_score > 0.5);
        assert!(!assessment.indicators.is_empty());
    }

    #[test]
    fn test_behavior_cleanup() {
        let mut calculator = AdaptiveDifficultyCalculator::new(DifficultyParameters::default());

        // Add some behavior data
        calculator.update_behavior(
            "test_session",
            "127.0.0.1",
            true,
            Duration::from_secs(5),
            Some(0.3),
            None,
        );

        // Verify data exists
        assert!(
            calculator
                .get_behavior_stats("test_session", "127.0.0.1")
                .is_some()
        );

        // Clean up with very short max age
        calculator.cleanup_old_data(Duration::from_millis(1));

        // Data should still exist since it was just created
        assert!(
            calculator
                .get_behavior_stats("test_session", "127.0.0.1")
                .is_some()
        );
    }
}
