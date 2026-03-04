//! CAPTCHA REST Handlers for Authenc API
//!
//! Provides REST endpoints for CAPTCHA challenge generation and verification.
//! Replaces the legacy layanan-portal captcha proxy by serving challenges directly.
//!
//! ## Endpoints
//! - `POST /api/captcha/challenge` - Generate a new CAPTCHA challenge
//! - `POST /api/captcha/verify` - Verify a CAPTCHA answer
//! - `GET /api/captcha/image/{nonce}/{index}` - Serve captcha image (placeholder SVG)

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use dashmap::DashMap;
use rand::Rng;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::ApiState;

// ============================================
// Types
// ============================================

/// In-memory challenge store with expiry
/// Stored as extension in ApiState or as global
static CHALLENGE_STORE: std::sync::LazyLock<DashMap<String, StoredChallenge>> =
    std::sync::LazyLock::new(DashMap::new);

/// A stored challenge
#[derive(Clone, Debug)]
struct StoredChallenge {
    /// The correct answer
    answer: String,
    /// Challenge type
    challenge_type: String,
    /// Expiry timestamp (unix seconds)
    expires_at: u64,
    /// Session ID
    session_id: Option<String>,
    /// Whether already verified
    verified: bool,
}

/// Request to generate a new captcha challenge
#[derive(Debug, Deserialize)]
pub struct ChallengeRequest {
    /// Type of challenge: "Visual", "Logical", etc.
    pub challenge_type: Option<String>,
    /// Difficulty level (1-10)
    pub difficulty: Option<u8>,
    /// Session identifier
    pub session_id: Option<String>,
}

/// Response with captcha challenge data
#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    /// Unique challenge identifier
    pub challenge_id: String,
    /// Challenge type
    pub challenge_type: String,
    /// JSON-encoded challenge data for the frontend to render
    pub challenge_data: String,
    /// Difficulty level
    pub difficulty: u8,
    /// Expiry timestamp (unix seconds)
    pub expires_at: u64,
    /// Optional metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Request to verify a captcha answer
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    /// Challenge ID to verify
    pub challenge_id: String,
    /// User's answer
    pub answer: String,
    /// Session ID
    pub session_id: Option<String>,
    /// Behavioral data (optional, for future risk analysis)
    pub behavioral_data: Option<serde_json::Value>,
}

/// Response from captcha verification
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    /// Whether verification was successful
    pub success: bool,
    /// Token to include in subsequent requests (e.g., login)
    pub token: Option<String>,
    /// Error message if failed
    pub message: String,
    /// Risk score (0.0 - 1.0, lower is safer)
    pub risk_score: Option<f64>,
}

// ============================================
// Handlers
// ============================================

/// Generate a new CAPTCHA challenge
///
/// POST /api/captcha/challenge
pub async fn captcha_challenge_handler(
    State(_state): State<Arc<ApiState>>,
    Json(req): Json<ChallengeRequest>,
) -> Json<ChallengeResponse> {
    // Clean expired challenges periodically
    cleanup_expired_challenges();

    let difficulty = req.difficulty.unwrap_or(3).clamp(1, 10);
    let challenge_type = req
        .challenge_type
        .as_deref()
        .unwrap_or("Visual")
        .to_string();

    let challenge_id = Uuid::new_v4().to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let expires_at = now + 300; // 5 minutes

    // Generate challenge based on type
    let (challenge_data, answer) = match challenge_type.as_str() {
        "Logical" | "math" | "logical" => generate_math_challenge(difficulty),
        _ => generate_text_recognition_challenge(difficulty),
    };

    // Store the challenge
    CHALLENGE_STORE.insert(
        challenge_id.clone(),
        StoredChallenge {
            answer,
            challenge_type: challenge_type.clone(),
            expires_at,
            session_id: req.session_id,
            verified: false,
        },
    );

    Json(ChallengeResponse {
        challenge_id,
        challenge_type,
        challenge_data,
        difficulty,
        expires_at,
        metadata: None,
    })
}

/// Verify a CAPTCHA answer
///
/// POST /api/captcha/verify
pub async fn captcha_verify_handler(
    State(_state): State<Arc<ApiState>>,
    Json(req): Json<VerifyRequest>,
) -> Json<VerifyResponse> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Look up the challenge
    let Some(mut challenge) = CHALLENGE_STORE.get_mut(&req.challenge_id) else {
        return Json(VerifyResponse {
            success: false,
            token: None,
            message: "Challenge not found or expired".to_string(),
            risk_score: Some(0.8),
        });
    };

    // Check expiry
    if now > challenge.expires_at {
        drop(challenge);
        CHALLENGE_STORE.remove(&req.challenge_id);
        return Json(VerifyResponse {
            success: false,
            token: None,
            message: "Challenge expired".to_string(),
            risk_score: Some(0.5),
        });
    }

    // Check if already verified
    if challenge.verified {
        return Json(VerifyResponse {
            success: false,
            token: None,
            message: "Challenge already used".to_string(),
            risk_score: Some(0.7),
        });
    }

    // Verify answer (case-insensitive)
    let correct = req
        .answer
        .trim()
        .eq_ignore_ascii_case(challenge.answer.trim());

    if correct {
        challenge.verified = true;
        drop(challenge);

        // Generate a verification token
        let token = format!("captcha_{}_{}", Uuid::new_v4(), now);

        // Remove the challenge after successful verification
        CHALLENGE_STORE.remove(&req.challenge_id);

        Json(VerifyResponse {
            success: true,
            token: Some(token),
            message: "Verification successful".to_string(),
            risk_score: Some(0.1),
        })
    } else {
        drop(challenge);
        // Remove on failure to prevent brute force
        CHALLENGE_STORE.remove(&req.challenge_id);

        Json(VerifyResponse {
            success: false,
            token: None,
            message: "Incorrect answer".to_string(),
            risk_score: Some(0.6),
        })
    }
}

/// Serve a captcha image (placeholder SVG)
///
/// GET /api/captcha/image/{nonce}/{index}
pub async fn captcha_image_handler(
    Path((_nonce, index)): Path<(String, u32)>,
) -> impl IntoResponse {
    // Generate a simple SVG placeholder image
    // In production, this would serve actual challenge images from storage
    let colors = [
        "#4A90D9", "#D94A4A", "#4AD94A", "#D9D94A", "#4AD9D9", "#D94AD9", "#9A4AD9", "#D99A4A",
        "#4A9AD9",
    ];
    let color = colors[(index as usize) % colors.len()];

    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="150" height="150" viewBox="0 0 150 150">
  <rect width="150" height="150" fill="{color}" rx="8"/>
  <text x="75" y="85" font-family="monospace" font-size="48" fill="white" text-anchor="middle" dominant-baseline="middle">{index}</text>
</svg>"#,
    );

    (
        StatusCode::OK,
        [
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "no-cache, no-store"),
        ],
        svg,
    )
}

// ============================================
// Challenge Generators
// ============================================

/// Generate a math challenge (e.g., "What is 7 + 3?")
fn generate_math_challenge(difficulty: u8) -> (String, String) {
    let mut rng = thread_rng();

    let (a, b, op, answer): (i32, i32, &str, String) = match difficulty {
        1..=3 => {
            // Simple addition/subtraction
            let a: i32 = rng.gen_range(1..=20);
            let b: i32 = rng.gen_range(1..=20);
            if rng.gen_bool(0.5) {
                (a, b, "+", (a + b).to_string())
            } else {
                let (a, b) = if a >= b { (a, b) } else { (b, a) };
                (a, b, "-", (a - b).to_string())
            }
        }
        4..=6 => {
            // Multiplication
            let a: i32 = rng.gen_range(2..=12);
            let b: i32 = rng.gen_range(2..=12);
            (a, b, "×", (a * b).to_string())
        }
        _ => {
            // Mixed operations with larger numbers
            let a: i32 = rng.gen_range(10..=50);
            let b: i32 = rng.gen_range(2..=20);
            if rng.gen_bool(0.5) {
                (a, b, "+", (a + b).to_string())
            } else {
                (a, b, "×", (a * b).to_string())
            }
        }
    };

    // Generate wrong options
    let correct: i32 = answer.parse::<i32>().unwrap_or(0);
    let mut options: Vec<i32> = vec![correct];
    while options.len() < 4 {
        let offset: i32 = rng.gen_range(1..=5) * if rng.gen_bool(0.5) { 1 } else { -1 };
        let wrong = correct + offset;
        if wrong >= 0 && !options.contains(&wrong) {
            options.push(wrong);
        }
    }

    // Shuffle options
    for i in (1..options.len()).rev() {
        let j = rng.gen_range(0..=i);
        options.swap(i, j);
    }

    let visual = format!("{} {} {}", a, op, b);
    let instructions = format!("Berapa hasil dari {} {} {}?", a, op, b);
    let options_json: Vec<serde_json::Value> =
        options.iter().map(|o| serde_json::json!(*o)).collect();

    let challenge_data = serde_json::json!({
        "challenge_type": "math",
        "instructions": instructions,
        "visual": visual,
        "options": options_json,
        "data": format!("{}:{}:{}", visual, op, answer),
    })
    .to_string();

    (challenge_data, answer)
}

/// Generate a text recognition challenge (e.g., "Type the characters: ABCD")
fn generate_text_recognition_challenge(difficulty: u8) -> (String, String) {
    let mut rng = thread_rng();

    let length = match difficulty {
        1..=3 => 4,
        4..=6 => 5,
        _ => 6,
    };

    // Generate random alphanumeric characters (avoiding confusing chars like 0/O, 1/l/I)
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();

    let text: String = (0..length)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect();

    let nonce = Uuid::new_v4().to_string();

    let challenge_data = serde_json::json!({
        "challenge_type": "text_recognition",
        "instructions": "Ketik karakter yang ditampilkan di bawah ini",
        "data": format!("{}:{}", text, nonce),
    })
    .to_string();

    (challenge_data, text)
}

/// Clean up expired captcha challenges
fn cleanup_expired_challenges() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Only clean up every ~100 requests (probabilistic)
    let mut rng = thread_rng();
    if rng.gen_range(0..100u32) == 0 {
        CHALLENGE_STORE.retain(|_, v| v.expires_at > now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_math_challenge() {
        let (data, answer) = generate_math_challenge(3);
        assert!(!data.is_empty());
        assert!(!answer.is_empty());
        // Answer should be a valid number
        assert!(answer.parse::<i32>().is_ok());
    }

    #[test]
    fn test_generate_text_recognition_challenge() {
        let (data, answer) = generate_text_recognition_challenge(3);
        assert!(!data.is_empty());
        assert_eq!(answer.len(), 4);
        // All chars should be uppercase alphanumeric
        assert!(answer.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_challenge_store_operations() {
        let id = "test-challenge-1".to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        CHALLENGE_STORE.insert(
            id.clone(),
            StoredChallenge {
                answer: "42".to_string(),
                challenge_type: "math".to_string(),
                expires_at: now + 300,
                session_id: None,
                verified: false,
            },
        );

        assert!(CHALLENGE_STORE.contains_key(&id));

        // Verify answer
        let challenge = CHALLENGE_STORE.get(&id).unwrap();
        assert_eq!(challenge.answer, "42");
        assert!(!challenge.verified);

        drop(challenge);
        CHALLENGE_STORE.remove(&id);
        assert!(!CHALLENGE_STORE.contains_key(&id));
    }
}
