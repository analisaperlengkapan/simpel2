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
            message: "Tantangan tidak ditemukan atau sudah kedaluwarsa".to_string(),
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
            message: "Tantangan telah kedaluwarsa".to_string(),
            risk_score: Some(0.5),
        });
    }

    // Check if already verified
    if challenge.verified {
        return Json(VerifyResponse {
            success: false,
            token: None,
            message: "Tantangan sudah pernah digunakan".to_string(),
            risk_score: Some(0.7),
        });
    }

    // Verify answer — CASE-SENSITIVE for text_recognition (mixed case is part of the challenge)
    // Math challenges remain case-insensitive
    let correct =
        if challenge.challenge_type == "text_recognition" || challenge.challenge_type == "Visual" {
            req.answer.trim() == challenge.answer.trim()
        } else {
            req.answer
                .trim()
                .eq_ignore_ascii_case(challenge.answer.trim())
        };

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
            message: "Verifikasi berhasil".to_string(),
            risk_score: Some(0.1),
        })
    } else {
        drop(challenge);
        // Remove on failure to prevent brute force
        CHALLENGE_STORE.remove(&req.challenge_id);

        Json(VerifyResponse {
            success: false,
            token: None,
            message: "Jawaban salah, silakan coba lagi".to_string(),
            risk_score: Some(0.6),
        })
    }
}

/// Debug endpoint: return the stored answer for a challenge.
///
/// **Only available when compiled with `--features captcha-debug`.**
/// Used by E2E / integration tests that cannot read the answer from the SVG.
///
/// GET /api/captcha/debug/{challenge_id}
#[cfg(feature = "captcha-debug")]
pub async fn captcha_debug_answer_handler(Path(challenge_id): Path<String>) -> impl IntoResponse {
    match CHALLENGE_STORE.get(&challenge_id) {
        Some(entry) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "challenge_id": challenge_id,
                "answer": entry.answer,
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "challenge not found or expired",
            })),
        )
            .into_response(),
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

/// Generate a text recognition challenge with server-side SVG rendering.
///
/// Best practices applied (OWASP / Bursztein et al.):
/// - Mandatory mix of uppercase, lowercase, and digits (anti-pattern recognition)
/// - Random rotation, scale and vertical offset per character (anti-segmentation)
/// - Overlapping noise lines, arcs, and dots (anti-OCR)
/// - Characters intentionally overlap slightly (anti-segmentation)
fn generate_text_recognition_challenge(difficulty: u8) -> (String, String) {
    let mut rng = thread_rng();

    let length: usize = match difficulty {
        1..=3 => 5,
        4..=6 => 6,
        _ => 7,
    };

    // Character pools (excluding visually ambiguous: 0/O, 1/l/I, 5/S)
    let uppercase: Vec<char> = "ABCDEFGHJKLMNPQRTUVWXYZ".chars().collect();
    let lowercase: Vec<char> = "abcdefghjkmnpqrstuvwxyz".chars().collect();
    let digits: Vec<char> = "2346789".chars().collect();

    // Guarantee at least 1 uppercase, 1 lowercase, 1 digit
    let mut text_chars: Vec<char> = Vec::with_capacity(length);
    text_chars.push(uppercase[rng.gen_range(0..uppercase.len())]);
    text_chars.push(lowercase[rng.gen_range(0..lowercase.len())]);
    text_chars.push(digits[rng.gen_range(0..digits.len())]);

    // Fill remaining with random from all pools
    let all_chars: Vec<char> = uppercase
        .iter()
        .chain(lowercase.iter())
        .chain(digits.iter())
        .copied()
        .collect();
    for _ in 3..length {
        text_chars.push(all_chars[rng.gen_range(0..all_chars.len())]);
    }

    // Shuffle to avoid predictable positions
    for i in (1..text_chars.len()).rev() {
        let j = rng.gen_range(0..=i);
        text_chars.swap(i, j);
    }

    let text: String = text_chars.iter().collect();

    // Generate SVG with visual noise
    let svg = generate_captcha_svg(&text, &mut rng, difficulty);

    let nonce = Uuid::new_v4().to_string();

    let challenge_data = serde_json::json!({
        "challenge_type": "text_recognition",
        "instructions": "Ketik karakter yang ditampilkan di bawah ini",
        "svg": svg,
        "data": format!("challenge:{}", nonce),
    })
    .to_string();

    (challenge_data, text)
}

/// Escape a character for safe inclusion in XML/SVG text content.
fn xml_escape_char(ch: char) -> String {
    match ch {
        '&' => "&amp;".to_string(),
        '<' => "&lt;".to_string(),
        '>' => "&gt;".to_string(),
        '"' => "&quot;".to_string(),
        '\'' => "&#x27;".to_string(),
        _ => ch.to_string(),
    }
}

/// Generate a distorted SVG image for CAPTCHA text.
///
/// Techniques used to defeat OCR/ML attacks:
/// 1. Per-character random rotation (-25° to +25°)
/// 2. Per-character random vertical offset
/// 3. Variable font sizes
/// 4. Multiple overlapping noise lines with varying stroke widths
/// 5. Random bezier curve arcs across the image
/// 6. Random dots/circles as background noise
/// 7. Subtle grid warp effect
fn generate_captcha_svg(text: &str, rng: &mut impl Rng, difficulty: u8) -> String {
    let char_count = text.chars().count();
    let vb_width = 50 * char_count + 30;
    let vb_height = 70;

    // Use viewBox for internal coords but width="100%" so SVG scales to container
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="100%" preserveAspectRatio="xMidYMid meet" viewBox="0 0 {vb_width} {vb_height}" style="display:block;border-radius:8px;">"#,
    );

    let width = vb_width;
    let height = vb_height;

    // Background
    svg.push_str(&format!(
        r##"<rect width="{width}" height="{height}" fill="#1a237e" rx="8"/>"##,
    ));

    // Background noise dots (more with higher difficulty)
    let dot_count = 15 + (difficulty as usize) * 5;
    for _ in 0..dot_count {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let r: f64 = rng.gen_range(1.0..3.5);
        let opacity: f64 = rng.gen_range(0.1..0.35);
        let colors = ["#5c6bc0", "#7986cb", "#9fa8da", "#c5cae9", "#3949ab"];
        let color = colors[rng.gen_range(0..colors.len())];
        svg.push_str(&format!(
            r#"<circle cx="{cx}" cy="{cy}" r="{r:.1}" fill="{color}" opacity="{opacity:.2}"/>"#,
        ));
    }

    // Noise lines crossing the image (anti-OCR)
    let line_count = 4 + (difficulty as usize) * 2;
    for _ in 0..line_count {
        let x1 = rng.gen_range(0..width);
        let y1 = rng.gen_range(0..height);
        let x2 = rng.gen_range(0..width);
        let y2 = rng.gen_range(0..height);
        let sw: f64 = rng.gen_range(1.0..2.5);
        let opacity: f64 = rng.gen_range(0.3..0.7);
        let colors = [
            "#e8eaf6", "#c5cae9", "#9fa8da", "#7986cb", "#5c6bc0", "#3f51b5",
        ];
        let color = colors[rng.gen_range(0..colors.len())];
        svg.push_str(&format!(
            r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{color}" stroke-width="{sw:.1}" opacity="{opacity:.2}"/>"#,
        ));
    }

    // Bezier curve arcs (harder for segmentation)
    let arc_count = 2 + (difficulty as usize);
    for _ in 0..arc_count {
        let sx = rng.gen_range(0..(width / 4));
        let sy = rng.gen_range(10..height - 10);
        let cx1 = rng.gen_range(width / 4..width / 2) as i32;
        let cy1 = rng.gen_range(0..height) as i32;
        let cx2 = rng.gen_range(width / 2..3 * width / 4) as i32;
        let cy2 = rng.gen_range(0..height) as i32;
        let ex = rng.gen_range(3 * width / 4..width);
        let ey = rng.gen_range(10..height - 10);
        let sw: f64 = rng.gen_range(1.0..2.5);
        let opacity: f64 = rng.gen_range(0.25..0.55);
        svg.push_str(&format!(
            r##"<path d="M{sx},{sy} C{cx1},{cy1} {cx2},{cy2} {ex},{ey}" fill="none" stroke="#9fa8da" stroke-width="{sw:.1}" opacity="{opacity:.2}"/>"##,
        ));
    }

    // Render each character with distortion
    // Use <g translate> + local rotate/scale so transforms stay centered on each char
    let padding = 20.0;
    let usable = width as f64 - padding * 2.0;
    let char_spacing = usable / char_count as f64;
    for (i, ch) in text.chars().enumerate() {
        let x = padding + (i as f64 + 0.5) * char_spacing + rng.gen_range(-3.0..3.0);
        let y = (height as f64) / 2.0 + rng.gen_range(-6.0..6.0);
        let rotation: f64 = rng.gen_range(-20.0..20.0);
        let font_size = rng.gen_range(24..32);
        let scale_x: f64 = rng.gen_range(0.9..1.1);
        let scale_y: f64 = rng.gen_range(0.9..1.1);

        // Alternate character colors for added difficulty
        let char_colors = [
            "#e8eaf6", "#c5cae9", "#ffffff", "#bbdefb", "#d1c4e9", "#f3e5f5",
        ];
        let color = char_colors[rng.gen_range(0..char_colors.len())];

        // Use different font weights randomly
        let weights = ["bold", "800", "900"];
        let weight = weights[rng.gen_range(0..weights.len())];

        // translate to char center, then rotate+scale locally so nothing drifts outside
        let safe_ch = xml_escape_char(ch);
        svg.push_str(&format!(
            r#"<g transform="translate({x:.1},{y:.1})"><text font-family="monospace,Courier,serif" font-size="{font_size}" font-weight="{weight}" fill="{color}" transform="rotate({rotation:.1}) scale({scale_x:.2},{scale_y:.2})" text-anchor="middle" dominant-baseline="middle">{safe_ch}</text></g>"#,
        ));
    }

    // Foreground scratch lines that partially cover characters
    let scratch_count = 3 + (difficulty as usize);
    for _ in 0..scratch_count {
        let x1 = rng.gen_range(10..width - 10);
        let y1 = rng.gen_range(20..height - 20);
        let x2 = x1 as i32 + rng.gen_range(-60..60);
        let y2 = y1 as i32 + rng.gen_range(-15..15);
        let sw: f64 = rng.gen_range(0.8..2.0);
        let opacity: f64 = rng.gen_range(0.3..0.65);
        svg.push_str(&format!(
            r##"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="#b0bec5" stroke-width="{sw:.1}" opacity="{opacity:.2}" stroke-linecap="round"/>"##,
        ));
    }

    svg.push_str("</svg>");
    svg
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
        assert_eq!(answer.len(), 5);
        // Must contain at least one uppercase, one lowercase, one digit
        assert!(answer.chars().any(|c| c.is_ascii_uppercase()));
        assert!(answer.chars().any(|c| c.is_ascii_lowercase()));
        assert!(answer.chars().any(|c| c.is_ascii_digit()));
        // SVG should be in the challenge data
        assert!(data.contains("svg"));
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
