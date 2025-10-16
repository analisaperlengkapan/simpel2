//! CAPTCHA Service Types
//!
//! Core types and traits for CAPTCHA service implementation

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// CAPTCHA challenge types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChallengeType {
    Visual,
    Audio,
    Behavioral,
    Logical,
    Hybrid,
}

/// Risk level assessment
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Behavioral classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BehaviorClassification {
    Human,
    Suspicious,
    Bot,
    Unknown,
}

/// CAPTCHA challenge model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: String,
    pub challenge_type: ChallengeType,
    #[serde(deserialize_with = "validate_difficulty")]
    pub difficulty_level: u8,
    pub encrypted_data: String,
    pub expected_answer_hash: String,
    #[serde(with = "systemtime_serde")]
    pub created_at: SystemTime,
    #[serde(with = "systemtime_serde")]
    pub expires_at: SystemTime,
    pub session_id: Option<String>,
    pub ip_address: String,
}

impl Challenge {
    pub fn new(
        challenge_type: ChallengeType,
        difficulty_level: u8,
        encrypted_data: String,
        expected_answer_hash: String,
        session_id: Option<String>,
        ip_address: String,
    ) -> Self {
        let now = SystemTime::now();
        let expires_at = now + Duration::from_secs(300); // 5 minutes

        Self {
            id: Uuid::new_v4().to_string(),
            challenge_type,
            difficulty_level,
            encrypted_data,
            expected_answer_hash,
            created_at: now,
            expires_at,
            session_id,
            ip_address,
        }
    }

    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
}
/// Behavioral metrics for bot detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralMetrics {
    pub session_id: String,
    pub mouse_movements: Vec<MouseEvent>,
    pub keystroke_dynamics: Vec<KeystrokeEvent>,
    pub timing_patterns: TimingAnalysis,
    pub browser_fingerprint: BrowserFingerprint,
    pub risk_score: f64,
    pub classification: BehaviorClassification,
}

/// Mouse event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    pub x: f64,
    pub y: f64,
    pub timestamp: u64,
    pub event_type: String,
    pub velocity: Option<f64>,
    pub acceleration: Option<f64>,
}

/// Keystroke event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeEvent {
    pub key: String,
    pub timestamp: u64,
    pub duration: u64,
    pub dwell_time: u64,
    pub flight_time: Option<u64>,
}

/// Timing analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingAnalysis {
    pub total_interaction_time: u64,
    pub pause_patterns: Vec<u64>,
    pub rhythm_consistency: f64,
    pub typing_speed: Option<f64>,
}

/// Browser fingerprint data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFingerprint {
    pub user_agent: String,
    pub screen_resolution: String,
    pub timezone: String,
    pub language: String,
    pub plugins: Vec<String>,
    pub canvas_fingerprint: Option<String>,
    pub webgl_fingerprint: Option<String>,
}

/// CAPTCHA validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub success: bool,
    pub confidence_score: f64,
    pub risk_assessment: RiskLevel,
    pub next_difficulty: u8,
    pub retry_allowed: bool,
    pub lockout_duration: Option<Duration>,
    pub message: String,
}
// Validation functions and serialization helpers
use serde::{Deserializer, Serializer};

/// Validate difficulty level is between 1 and 10
fn validate_difficulty<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let difficulty = u8::deserialize(deserializer)?;
    if difficulty < 1 || difficulty > 10 {
        return Err(serde::de::Error::custom(
            "Difficulty must be between 1 and 10",
        ));
    }
    Ok(difficulty)
}

/// SystemTime serialization module
mod systemtime_serde {
    use super::*;
    use std::time::UNIX_EPOCH;

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| serde::ser::Error::custom("SystemTime before UNIX_EPOCH"))?;
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success(confidence_score: f64, next_difficulty: u8) -> Self {
        Self {
            success: true,
            confidence_score,
            risk_assessment: RiskLevel::Low,
            next_difficulty,
            retry_allowed: true,
            lockout_duration: None,
            message: "Challenge completed successfully".to_string(),
        }
    }

    /// Create a failed validation result
    pub fn failure(
        risk_assessment: RiskLevel,
        next_difficulty: u8,
        retry_allowed: bool,
        lockout_duration: Option<Duration>,
        message: String,
    ) -> Self {
        Self {
            success: false,
            confidence_score: 0.0,
            risk_assessment,
            next_difficulty,
            retry_allowed,
            lockout_duration,
            message,
        }
    }
}

impl BehavioralMetrics {
    /// Create new behavioral metrics with default values
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            mouse_movements: Vec::new(),
            keystroke_dynamics: Vec::new(),
            timing_patterns: TimingAnalysis::default(),
            browser_fingerprint: BrowserFingerprint::default(),
            risk_score: 0.5,
            classification: BehaviorClassification::Unknown,
        }
    }

    /// Add a mouse event to the metrics
    pub fn add_mouse_event(&mut self, event: MouseEvent) {
        self.mouse_movements.push(event);
    }

    /// Add a keystroke event to the metrics
    pub fn add_keystroke_event(&mut self, event: KeystrokeEvent) {
        self.keystroke_dynamics.push(event);
    }
}

impl Default for TimingAnalysis {
    fn default() -> Self {
        Self {
            total_interaction_time: 0,
            pause_patterns: Vec::new(),
            rhythm_consistency: 0.0,
            typing_speed: None,
        }
    }
}

impl Default for BrowserFingerprint {
    fn default() -> Self {
        Self {
            user_agent: String::new(),
            screen_resolution: String::new(),
            timezone: String::new(),
            language: String::new(),
            plugins: Vec::new(),
            canvas_fingerprint: None,
            webgl_fingerprint: None,
        }
    }
}
