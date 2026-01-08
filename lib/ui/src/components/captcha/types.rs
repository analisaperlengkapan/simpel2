//! CAPTCHA Types and Interfaces
//!
//! Core types for CAPTCHA component system

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// CAPTCHA challenge types supported by the system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChallengeType {
    /// Visual puzzle challenges
    Visual,
    /// Audio-based challenges for accessibility
    Audio,
    /// Behavioral analysis challenges
    Behavioral,
    /// Mathematical or logical puzzles
    Logical,
    /// Hybrid challenges combining multiple types
    Hybrid,
}

/// Risk level assessment for user behavior
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Behavioral classification results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BehaviorClassification {
    Human,
    Suspicious,
    Bot,
    Unknown,
}

/// CAPTCHA component properties
#[derive(Debug, Clone)]
pub struct CaptchaProps {
    /// Callback when CAPTCHA is successfully completed
    pub on_success: Callback<String>,
    /// Callback when CAPTCHA fails
    pub on_failure: Callback<String>,
    /// Initial difficulty level (1-10)
    pub difficulty: Option<u8>,
    /// Enable accessibility features
    pub accessibility_enabled: bool,
    /// Custom CSS classes
    pub class: Option<String>,
    /// Enable behavioral analysis
    pub behavioral_analysis: bool,
}

impl Default for CaptchaProps {
    fn default() -> Self {
        Self {
            on_success: Callback::new(|_| {}),
            on_failure: Callback::new(|_| {}),
            difficulty: Some(3),
            accessibility_enabled: true,
            class: None,
            behavioral_analysis: true,
        }
    }
}

/// CAPTCHA state management
#[derive(Debug, Clone, PartialEq)]
pub struct CaptchaState {
    /// Current challenge ID
    pub challenge_id: Option<String>,
    /// Current challenge type
    pub challenge_type: ChallengeType,
    /// Current difficulty level
    pub difficulty: u8,
    /// Loading state
    pub loading: bool,
    /// Error message if any
    pub error: Option<String>,
    /// Number of attempts made
    pub attempts: u32,
    /// Whether user is locked out
    pub locked_out: bool,
}

impl Default for CaptchaState {
    fn default() -> Self {
        Self {
            challenge_id: None,
            challenge_type: ChallengeType::Visual,
            difficulty: 3,
            loading: false,
            error: None,
            attempts: 0,
            locked_out: false,
        }
    }
}

/// CAPTCHA API response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub challenge_type: ChallengeType,
    pub challenge_data: String,
    #[serde(alias = "difficulty_level")]
    pub difficulty: u8,
    pub expires_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequest {
    pub challenge_id: String,
    pub answer: String,
    pub behavioral_data: Option<BehavioralData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub success: bool,
    pub message: String,
    pub next_difficulty: Option<u8>,
    pub retry_allowed: bool,
    pub lockout_duration: Option<u32>,
}

/// Behavioral data collected from user interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralData {
    pub mouse_movements: Vec<MouseEvent>,
    pub keystroke_timings: Vec<KeystrokeEvent>,
    pub interaction_duration: u64,
    pub browser_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    pub x: f64,
    pub y: f64,
    pub timestamp: u64,
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeEvent {
    pub key: String,
    pub timestamp: u64,
    pub duration: u64,
}
