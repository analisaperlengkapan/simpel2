//! CAPTCHA Service Types
//!
//! Core types and traits for CAPTCHA service implementation

use super::secreton_integration::EncryptedChallengeData;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// CAPTCHA challenge types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChallengeType {
    /// Visual CAPTCHA (image-based)
    Visual,
    /// Audio CAPTCHA (sound-based)
    Audio,
    /// Behavioral CAPTCHA (interaction-based)
    Behavioral,
    /// Logical CAPTCHA (puzzle-based)
    Logical,
    /// Hybrid CAPTCHA (multiple types combined)
    Hybrid,
}

/// Risk level assessment
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk level
    Low,
    /// Medium risk level
    Medium,
    /// High risk level
    High,
    /// Critical risk level
    Critical,
}

/// Behavioral classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BehaviorClassification {
    /// Classified as human behavior
    Human,
    /// Suspicious behavior detected
    Suspicious,
    /// Classified as bot behavior
    Bot,
    /// Unknown classification
    Unknown,
}

/// CAPTCHA challenge model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    /// Unique challenge identifier
    pub id: String,
    /// Type of CAPTCHA challenge
    pub challenge_type: ChallengeType,
    /// Difficulty level (1-10)
    pub difficulty_level: u8,
    /// Challenge data (legacy field for backward compatibility)
    pub encrypted_data: String,
    /// Hash of the expected answer
    pub expected_answer_hash: String,
    /// Timestamp when challenge was created
    pub created_at: SystemTime,
    /// Timestamp when challenge expires
    pub expires_at: SystemTime,
    /// Optional session identifier
    pub session_id: Option<String>,
    /// IP address of the client
    pub ip_address: String,
    /// Encrypted challenge data with metadata (if encryption enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_challenge_data: Option<EncryptedChallengeData>,
    /// Flag indicating if challenge is encrypted
    #[serde(default)]
    pub is_encrypted: bool,
    /// Transient field to hold plaintext data for immediate response (not stored/serialized)
    #[serde(skip)]
    pub plaintext_data: Option<String>,
}

impl Challenge {
    /// Create a new CAPTCHA challenge
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
            encrypted_challenge_data: None,
            is_encrypted: false,
            plaintext_data: None,
        }
    }

    /// Set encrypted challenge data with metadata
    pub fn with_encrypted_data(mut self, encrypted_data: Option<EncryptedChallengeData>) -> Self {
        self.encrypted_challenge_data = encrypted_data;
        self
    }

    /// Set encryption flag
    pub fn with_encryption_flag(mut self, is_encrypted: bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    /// Set a specific challenge ID (used when hash was computed with this ID)
    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    /// Set plaintext data (transient)
    pub fn with_plaintext_data(mut self, data: String) -> Self {
        self.plaintext_data = Some(data);
        self
    }

    /// Check if the challenge has expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
}
/// Behavioral metrics for bot detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralMetrics {
    /// Session identifier for tracking user behavior
    pub session_id: String,
    /// Recorded mouse movement events
    pub mouse_movements: Vec<MouseEvent>,
    /// Recorded keystroke dynamics
    pub keystroke_dynamics: Vec<KeystrokeEvent>,
    /// Analysis of timing patterns
    pub timing_patterns: TimingAnalysis,
    /// Browser fingerprint for device identification
    pub browser_fingerprint: BrowserFingerprint,
    /// Calculated risk score (0.0 to 1.0)
    pub risk_score: f64,
    /// Classification of behavior type
    pub classification: BehaviorClassification,
}

/// Mouse event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    /// X coordinate of mouse position
    pub x: f64,
    /// Y coordinate of mouse position
    pub y: f64,
    /// Timestamp of the event in milliseconds
    pub timestamp: u64,
    /// Type of mouse event (click, move, etc.)
    pub event_type: String,
    /// Mouse movement velocity if applicable
    pub velocity: Option<f64>,
    /// Mouse movement acceleration if applicable
    pub acceleration: Option<f64>,
}

/// Keystroke event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeEvent {
    /// The key that was pressed
    pub key: String,
    /// Timestamp when key was pressed in milliseconds
    pub timestamp: u64,
    /// Duration the key was held down in milliseconds
    pub duration: u64,
    /// Time the key was pressed down
    pub dwell_time: u64,
    /// Time between releasing this key and pressing the next
    pub flight_time: Option<u64>,
}

/// Timing analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingAnalysis {
    /// Total time spent on interaction in milliseconds
    pub total_interaction_time: u64,
    /// Patterns of pauses between actions
    pub pause_patterns: Vec<u64>,
    /// Consistency score of typing rhythm (0.0 to 1.0)
    pub rhythm_consistency: f64,
    /// Average typing speed in characters per minute
    pub typing_speed: Option<f64>,
}

/// Browser fingerprint data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFingerprint {
    /// User agent string of the browser
    pub user_agent: String,
    /// Screen resolution in format "widthxheight"
    pub screen_resolution: String,
    /// Timezone identifier
    pub timezone: String,
    /// Primary language setting
    pub language: String,
    /// List of installed browser plugins
    pub plugins: Vec<String>,
    /// Canvas rendering fingerprint hash
    pub canvas_fingerprint: Option<String>,
    /// WebGL rendering fingerprint hash
    pub webgl_fingerprint: Option<String>,
}

/// CAPTCHA validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the validation was successful
    pub success: bool,
    /// Confidence score of the validation (0.0 to 1.0)
    pub confidence_score: f64,
    /// Assessed risk level
    pub risk_assessment: RiskLevel,
    /// Recommended difficulty level for next challenge
    pub next_difficulty: u8,
    /// Whether retry is allowed after failure
    pub retry_allowed: bool,
    /// Duration to lock out user if applicable
    pub lockout_duration: Option<Duration>,
    /// Human-readable message about the result
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
