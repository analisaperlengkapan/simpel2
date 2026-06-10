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

use axum::extract::ConnectInfo;
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use uuid::Uuid;

use super::auth::ErrorResponse;
use crate::state::ApiState;

// ============================================
// Types
// ============================================

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
// Helpers
// ============================================

/// Extract the real client IP address, handling proxy headers
fn get_client_ip(headers: &HeaderMap, addr: SocketAddr) -> IpAddr {
    // 1. Try X-Forwarded-For header
    if let Some(xff) = headers.get("x-forwarded-for")
        && let Ok(xff_str) = xff.to_str()
        && let Some(first_ip) = xff_str.split(',').next()
        && let Ok(ip) = first_ip.trim().parse::<IpAddr>()
    {
        return ip;
    }
    // 2. Try X-Real-IP header
    if let Some(xri) = headers.get("x-real-ip")
        && let Ok(xri_str) = xri.to_str()
        && let Ok(ip) = xri_str.trim().parse::<IpAddr>()
    {
        return ip;
    }
    // 3. Fallback to connection info IP
    addr.ip()
}

// ============================================
// Handlers
// ============================================

/// Generate a new CAPTCHA challenge
///
/// POST /api/captcha/challenge
pub async fn captcha_challenge_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<ChallengeRequest>,
) -> Result<Json<ChallengeResponse>, ErrorResponse> {
    let difficulty = req.difficulty.unwrap_or(3) as u32;
    let req_type_str = req.challenge_type.as_deref().unwrap_or("Visual");

    let challenge_type = match req_type_str {
        "Logical" | "math" | "logical" => authenc_types::domain::captcha::ChallengeType::Logical,
        _ => authenc_types::domain::captcha::ChallengeType::Visual,
    };

    let ip_address = get_client_ip(&headers, addr);

    let (challenge, challenge_data) = state
        .captcha_service
        .generate_challenge(challenge_type, difficulty, ip_address, req.session_id)
        .await
        .map_err(|e| ErrorResponse {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            error: "captcha_generation_failed".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(ChallengeResponse {
        challenge_id: challenge.id.to_string(),
        challenge_type: match challenge.challenge_type {
            authenc_types::domain::captcha::ChallengeType::Logical => "Logical".to_string(),
            authenc_types::domain::captcha::ChallengeType::Visual => "Visual".to_string(),
        },
        challenge_data,
        difficulty: challenge.difficulty as u8,
        expires_at: challenge.expires_at.timestamp() as u64,
        metadata: None,
    }))
}

/// Verify a CAPTCHA answer
///
/// POST /api/captcha/verify
pub async fn captcha_verify_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, ErrorResponse> {
    let challenge_uuid = Uuid::parse_str(&req.challenge_id).map_err(|_| ErrorResponse {
        status_code: StatusCode::BAD_REQUEST,
        error: "invalid_challenge_id".to_string(),
        message: "Format ID tantangan tidak valid".to_string(),
    })?;

    let ip_address = get_client_ip(&headers, addr);
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok().map(String::from));

    match state
        .captcha_service
        .verify_challenge(challenge_uuid, &req.answer, ip_address, user_agent)
        .await
    {
        Ok(true) => {
            // The solved challenge id IS the single-use login token (#49): the
            // login handler redeems it via `consume_solved_captcha`, which deletes
            // the row so it cannot be replayed. (Previously a cosmetic random
            // token was returned that the server never validated — captcha was not
            // actually enforced on login.)
            Ok(Json(VerifyResponse {
                success: true,
                token: Some(req.challenge_id.clone()),
                message: "Verifikasi berhasil".to_string(),
                risk_score: Some(0.1),
            }))
        }
        Ok(false) => Ok(Json(VerifyResponse {
            success: false,
            token: None,
            message: "Jawaban salah, silakan coba lagi".to_string(),
            risk_score: Some(0.6),
        })),
        Err(e) => {
            // CaptchaService verification failed with validation error (expired, already solved, etc)
            Ok(Json(VerifyResponse {
                success: false,
                token: None,
                message: e.to_string(),
                risk_score: Some(0.5),
            }))
        }
    }
}

/// Debug endpoint: return the stored answer for a challenge.
///
/// **Only available when compiled with `--features captcha-debug`.**
/// Used by E2E / integration tests that cannot read the answer from the SVG.
///
/// GET /api/captcha/debug/{challenge_id}
#[cfg(feature = "captcha-debug")]
pub async fn captcha_debug_answer_handler(
    State(state): State<Arc<ApiState>>,
    Path(challenge_id): Path<String>,
) -> impl IntoResponse {
    let challenge_uuid = match Uuid::parse_str(&challenge_id) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid UUID" })),
            )
                .into_response();
        }
    };

    let client = match state.database.get_connection().await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let row_opt = match client
        .query_opt(
            "SELECT challenge_type, raw_data FROM captcha_challenges WHERE id = $1",
            &[&challenge_uuid],
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let Some(row) = row_opt else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "challenge not found" })),
        )
            .into_response();
    };

    let challenge_type: String = row.get("challenge_type");
    let raw_data: String = row.get("raw_data");

    let answer = if challenge_type == "Logical" {
        // Parse JSON and extract data
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw_data) {
            if let Some(data_str) = val.get("data").and_then(|v| v.as_str()) {
                // e.g. "7 + 3:+:10"
                data_str.split(":+:").nth(2).unwrap_or("").to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        // Extract chars from SVG using a simple regex since SVG is structured consistently
        let re = regex::Regex::new(r#">([a-zA-Z0-9])</text>"#).unwrap();
        let mut ans = String::new();
        for cap in re.captures_iter(&raw_data) {
            ans.push_str(&cap[1]);
        }
        ans
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "challenge_id": challenge_id,
            "answer": answer,
        })),
    )
        .into_response()
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
