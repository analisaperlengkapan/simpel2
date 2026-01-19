//! Context-Aware Challenge Selection and Personalization
//!
//! Implements intelligent challenge selection based on user behavior history and effectiveness analytics

use super::error::CaptchaError;
use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// User challenge history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserChallengeHistory {
    /// User session or identifier
    pub user_id: String,
    /// Total challenges attempted
    pub total_attempts: u32,
    /// Successful completions
    pub successful_completions: u32,
    /// Failed attempts
    pub failed_attempts: u32,
    /// Success rate by challenge type
    pub success_rate_by_type: HashMap<String, f64>,
    /// Average completion time by challenge type (in seconds)
    pub avg_completion_time: HashMap<String, f64>,
    /// Preferred challenge types (based on success and speed)
    pub preferred_types: Vec<String>,
    /// Challenge types to avoid (low success rate)
    pub avoid_types: Vec<String>,
    /// Current difficulty level
    pub current_difficulty: u8,
    /// Last challenge timestamp
    pub last_challenge_time: Option<std::time::SystemTime>,
}

impl UserChallengeHistory {
    /// Create new user challenge history
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            total_attempts: 0,
            successful_completions: 0,
            failed_attempts: 0,
            success_rate_by_type: HashMap::new(),
            avg_completion_time: HashMap::new(),
            preferred_types: Vec::new(),
            avoid_types: Vec::new(),
            current_difficulty: 3, // Start at medium-low difficulty
            last_challenge_time: None,
        }
    }

    /// Calculate overall success rate
    pub fn overall_success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            return 0.0;
        }
        self.successful_completions as f64 / self.total_attempts as f64
    }

    /// Update history with challenge result
    pub fn update_with_result(
        &mut self,
        challenge_type: &str,
        success: bool,
        completion_time_secs: f64,
    ) {
        self.total_attempts += 1;

        if success {
            self.successful_completions += 1;
        } else {
            self.failed_attempts += 1;
        }

        // Update success rate for this challenge type
        let type_attempts = self
            .success_rate_by_type
            .get(challenge_type)
            .unwrap_or(&0.0);
        let new_success_rate = if success {
            (*type_attempts * 0.8) + 0.2 // Weighted moving average
        } else {
            *type_attempts * 0.8
        };
        self.success_rate_by_type
            .insert(challenge_type.to_string(), new_success_rate);

        // Update average completion time
        let current_avg = self
            .avg_completion_time
            .get(challenge_type)
            .unwrap_or(&30.0);
        let new_avg = (*current_avg * 0.7) + (completion_time_secs * 0.3); // Weighted moving average
        self.avg_completion_time
            .insert(challenge_type.to_string(), new_avg);

        self.last_challenge_time = Some(std::time::SystemTime::now());

        // Update preferred and avoid lists
        self.update_preferences();
    }

    fn update_preferences(&mut self) {
        self.preferred_types.clear();
        self.avoid_types.clear();

        for (challenge_type, success_rate) in &self.success_rate_by_type {
            if *success_rate >= 0.7 {
                // High success rate - preferred
                self.preferred_types.push(challenge_type.clone());
            } else if *success_rate < 0.3 {
                // Low success rate - avoid
                self.avoid_types.push(challenge_type.clone());
            }
        }
    }

    /// Adjust difficulty based on recent performance
    /// Update preferred and avoid challenge types based on performance
    pub fn adjust_difficulty(&mut self) {
        let success_rate = self.overall_success_rate();

        if success_rate >= 0.8 && self.current_difficulty < 10 {
            // High success rate - increase difficulty
            self.current_difficulty = (self.current_difficulty + 1).min(10);
        } else if success_rate < 0.4 && self.current_difficulty > 1 {
            // Low success rate - decrease difficulty
            self.current_difficulty = (self.current_difficulty - 1).max(1);
        }
    }
}

/// Challenge type effectiveness metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeTypeEffectiveness {
    /// Challenge type identifier
    pub challenge_type: String,
    /// Total times this type was used
    pub usage_count: u32,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average completion time in seconds
    pub avg_completion_time: f64,
    /// Bot detection rate (0.0 to 1.0)
    pub bot_detection_rate: f64,
    /// User satisfaction score (0.0 to 1.0)
    pub user_satisfaction: f64,
    /// Effectiveness score (composite metric)
    pub effectiveness_score: f64,
}

impl ChallengeTypeEffectiveness {
    /// Create new effectiveness metrics
    pub fn new(challenge_type: String) -> Self {
        Self {
            challenge_type,
            usage_count: 0,
            success_rate: 0.5,
            avg_completion_time: 30.0,
            bot_detection_rate: 0.5,
            user_satisfaction: 0.5,
            effectiveness_score: 0.5,
        }
    }

    /// Update metrics with new data
    pub fn update(&mut self, success: bool, completion_time: f64, bot_detected: bool) {
        self.usage_count += 1;

        // Update success rate (weighted moving average)
        let success_value = if success { 1.0 } else { 0.0 };
        self.success_rate = (self.success_rate * 0.9) + (success_value * 0.1);

        // Update completion time
        self.avg_completion_time = (self.avg_completion_time * 0.9) + (completion_time * 0.1);

        // Update bot detection rate
        let bot_value = if bot_detected { 1.0 } else { 0.0 };
        self.bot_detection_rate = (self.bot_detection_rate * 0.9) + (bot_value * 0.1);

        // Calculate user satisfaction (based on success and reasonable completion time)
        let time_factor = if completion_time < 60.0 { 1.0 } else { 0.5 };
        let satisfaction = if success { time_factor } else { 0.3 };
        self.user_satisfaction = (self.user_satisfaction * 0.9) + (satisfaction * 0.1);

        // Calculate composite effectiveness score
        self.effectiveness_score = (self.success_rate * 0.3)
            + (self.bot_detection_rate * 0.4)
            + (self.user_satisfaction * 0.3);
    }
}

/// Context-aware challenge selector
pub struct ChallengeSelector {
    effectiveness_metrics: HashMap<String, ChallengeTypeEffectiveness>,
    /// User challenge histories
    user_histories: HashMap<String, UserChallengeHistory>,
}

impl ChallengeSelector {
    /// Create a new challenge selector
    /// Challenge type effectiveness metrics
    pub fn new() -> Self {
        let mut effectiveness_metrics = HashMap::new();

        // Initialize effectiveness metrics for all challenge types
        let challenge_types = vec![
            "Visual",
            "Audio",
            "Logical",
            "Behavioral",
            "Hybrid",
            "ImageJigsaw",
            "ImageRotation",
            "ImageObjectSelection",
            "ImageSequence",
            "AudioToneSequence",
            "AudioSpokenDigits",
            "AudioSpokenWords",
            "AudioPatternRecognition",
            "AudioSoundIdentification",
        ];

        for challenge_type in challenge_types {
            effectiveness_metrics.insert(
                challenge_type.to_string(),
                ChallengeTypeEffectiveness::new(challenge_type.to_string()),
            );
        }

        Self {
            effectiveness_metrics,
            user_histories: HashMap::new(),
        }
    }

    /// Get or create user challenge history
    pub fn get_user_history(&mut self, user_id: &str) -> &mut UserChallengeHistory {
        self.user_histories
            .entry(user_id.to_string())
            .or_insert_with(|| UserChallengeHistory::new(user_id.to_string()))
    }

    /// Select optimal challenge type for user
    pub fn select_challenge_type(
        &mut self,
        user_id: &str,
        behavioral_metrics: Option<&BehavioralMetrics>,
    ) -> Result<ChallengeType, CaptchaError> {
        // Get user's preferred and avoid types
        let (preferred_types, avoid_types) = {
            let user_history = self.get_user_history(user_id);
            (
                user_history.preferred_types.clone(),
                user_history.avoid_types.clone(),
            )
        };

        // Check if user has preferred types
        if !preferred_types.is_empty() {
            // Select from preferred types
            let preferred_type = &preferred_types[0];
            return self.map_string_to_challenge_type(preferred_type);
        }

        // If behavioral metrics indicate bot-like behavior, use harder challenges
        if let Some(metrics) = behavioral_metrics {
            if metrics.risk_score > 0.7 {
                // High risk - use hybrid or behavioral challenges
                return Ok(ChallengeType::Hybrid);
            }
        }

        // Select based on effectiveness metrics
        let mut best_type = "Visual";
        let mut best_score = 0.0;

        for (challenge_type, metrics) in &self.effectiveness_metrics {
            // Skip types in user's avoid list
            if avoid_types.contains(challenge_type) {
                continue;
            }

            if metrics.effectiveness_score > best_score {
                best_score = metrics.effectiveness_score;
                best_type = challenge_type;
            }
        }

        self.map_string_to_challenge_type(best_type)
    }

    /// Select difficulty level for user
    pub fn select_difficulty(&mut self, user_id: &str) -> u8 {
        let user_history = self.get_user_history(user_id);
        user_history.adjust_difficulty();
        user_history.current_difficulty
    }

    /// Record challenge result and update metrics
    pub fn record_challenge_result(
        &mut self,
        user_id: &str,
        challenge_type: &str,
        success: bool,
        completion_time_secs: f64,
        bot_detected: bool,
    ) {
        // Update user history
        let user_history = self.get_user_history(user_id);
        user_history.update_with_result(challenge_type, success, completion_time_secs);

        // Update effectiveness metrics
        if let Some(metrics) = self.effectiveness_metrics.get_mut(challenge_type) {
            metrics.update(success, completion_time_secs, bot_detected);
        }
    }

    /// Get effectiveness report for all challenge types
    pub fn get_effectiveness_report(&self) -> Vec<ChallengeTypeEffectiveness> {
        let mut report: Vec<_> = self.effectiveness_metrics.values().cloned().collect();
        report.sort_by(|a, b| {
            b.effectiveness_score
                .partial_cmp(&a.effectiveness_score)
                .unwrap()
        });
        report
    }

    /// Get user performance summary
    pub fn get_user_summary(&self, user_id: &str) -> Option<UserChallengeHistory> {
        self.user_histories.get(user_id).cloned()
    }

    /// Map string to ChallengeType enum
    fn map_string_to_challenge_type(&self, type_str: &str) -> Result<ChallengeType, CaptchaError> {
        match type_str {
            "Visual"
            | "ImageJigsaw"
            | "ImageRotation"
            | "ImageObjectSelection"
            | "ImageSequence" => Ok(ChallengeType::Visual),
            "Audio"
            | "AudioToneSequence"
            | "AudioSpokenDigits"
            | "AudioSpokenWords"
            | "AudioPatternRecognition"
            | "AudioSoundIdentification" => Ok(ChallengeType::Audio),
            "Logical" => Ok(ChallengeType::Logical),
            "Behavioral" => Ok(ChallengeType::Behavioral),
            "Hybrid" => Ok(ChallengeType::Hybrid),
            _ => Err(CaptchaError::GenerationFailed {
                message: format!("Unknown challenge type: {}", type_str),
                recoverable: true,
                retry_after: Some(Duration::from_secs(1)),
            }),
        }
    }
}

impl Default for ChallengeSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_history_creation() {
        let history = UserChallengeHistory::new("user123".to_string());
        assert_eq!(history.user_id, "user123");
        assert_eq!(history.total_attempts, 0);
        assert_eq!(history.current_difficulty, 3);
    }

    #[test]
    fn test_user_history_update() {
        let mut history = UserChallengeHistory::new("user123".to_string());

        history.update_with_result("Visual", true, 25.0);
        assert_eq!(history.total_attempts, 1);
        assert_eq!(history.successful_completions, 1);

        history.update_with_result("Visual", false, 45.0);
        assert_eq!(history.total_attempts, 2);
        assert_eq!(history.failed_attempts, 1);
    }

    #[test]
    fn test_difficulty_adjustment() {
        let mut history = UserChallengeHistory::new("user123".to_string());

        // Simulate high success rate
        for _ in 0..10 {
            history.update_with_result("Visual", true, 20.0);
        }

        let initial_difficulty = history.current_difficulty;
        history.adjust_difficulty();
        assert!(history.current_difficulty >= initial_difficulty);
    }

    #[test]
    fn test_challenge_selector_creation() {
        let selector = ChallengeSelector::new();
        assert!(!selector.effectiveness_metrics.is_empty());
    }

    #[test]
    fn test_challenge_type_selection() {
        let mut selector = ChallengeSelector::new();

        let challenge_type = selector.select_challenge_type("user123", None);
        assert!(challenge_type.is_ok());
    }

    #[test]
    fn test_difficulty_selection() {
        let mut selector = ChallengeSelector::new();

        let difficulty = selector.select_difficulty("user123");
        assert!(difficulty >= 1 && difficulty <= 10);
    }

    #[test]
    fn test_record_challenge_result() {
        let mut selector = ChallengeSelector::new();

        selector.record_challenge_result("user123", "Visual", true, 30.0, false);

        let user_summary = selector.get_user_summary("user123");
        assert!(user_summary.is_some());

        let summary = user_summary.unwrap();
        assert_eq!(summary.total_attempts, 1);
        assert_eq!(summary.successful_completions, 1);
    }

    #[test]
    fn test_effectiveness_report() {
        let mut selector = ChallengeSelector::new();

        // Record some results
        selector.record_challenge_result("user1", "Visual", true, 25.0, false);
        selector.record_challenge_result("user2", "Audio", false, 60.0, true);

        let report = selector.get_effectiveness_report();
        assert!(!report.is_empty());
    }

    #[test]
    fn test_effectiveness_metrics_update() {
        let mut metrics = ChallengeTypeEffectiveness::new("Visual".to_string());

        metrics.update(true, 30.0, false);
        assert_eq!(metrics.usage_count, 1);
        assert!(metrics.success_rate > 0.5);
    }
}
