use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of CAPTCHA challenge
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChallengeType {
    /// Visual alphanumeric CAPTCHA
    Visual,
    /// Logical math CAPTCHA
    Logical,
}

/// Representation of a generated CAPTCHA challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    /// Unique identifier for the challenge
    pub id: Uuid,
    /// The type of challenge
    pub challenge_type: ChallengeType,
    /// Hash of the correct answer
    pub answer_hash: String,
    /// Difficulty level of the challenge
    pub difficulty: u32,
    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Whether the challenge has already been verified
    pub verified: bool,
    /// Optional session identifier associated with the request
    pub session_id: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Record of an attempt to validate a CAPTCHA challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationAttempt {
    /// Unique identifier for the attempt
    pub id: Uuid,
    /// Identifier of the challenge being verified
    pub challenge_id: Uuid,
    /// The answer submitted by the user
    pub answer_submitted: String,
    /// Whether verification succeeded
    pub success: bool,
    /// IP address of the user making the request
    pub ip_address: Option<String>,
    /// User Agent of the client browser
    pub user_agent: Option<String>,
    /// Verification process duration in milliseconds
    pub duration_ms: Option<i64>,
    /// AI/Heuristic calculated risk score (0.0 to 1.0)
    pub risk_score: Option<f32>,
    /// Timestamp when validation attempt occurred
    pub created_at: DateTime<Utc>,
}
