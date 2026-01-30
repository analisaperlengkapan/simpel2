//! CAPTCHA handlers
//!
//! REST API handlers that proxy CAPTCHA requests to Authenc via gRPC

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use crate::state::AppState;

/// CAPTCHA challenge request
#[derive(Debug, Deserialize)]
pub struct CaptchaRequest {
    pub challenge_type: String,
    pub difficulty: u8,
    pub session_id: String,
}

/// CAPTCHA challenge response
#[derive(Debug, Serialize)]
pub struct CaptchaResponse {
    pub challenge_id: String,
    pub challenge_type: String,
    pub challenge_data: String,
    pub difficulty: u8,
    pub expires_at: i64,
}

/// CAPTCHA verification request
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub challenge_id: String,
    pub answer: String,
    pub session_id: String,
    pub behavioral_data: Option<serde_json::Value>,
}

/// CAPTCHA verification response
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// POST /api/captcha/challenge - Generate CAPTCHA challenge
pub async fn generate_challenge(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CaptchaRequest>,
) -> impl IntoResponse {
    let start_time = std::time::Instant::now();

    info!(
        "CAPTCHA challenge request: session_id={}, type={}, difficulty={}",
        request.session_id, request.challenge_type, request.difficulty
    );

    // Call Authenc via gRPC to generate CAPTCHA
    match state
        .authenc
        .generate_captcha(
            &request.challenge_type,
            request.difficulty,
            &request.session_id,
        )
        .await
    {
        Ok(challenge) => {
            let elapsed = start_time.elapsed();
            info!(
                "CAPTCHA challenge generated: challenge_id={}, elapsed={:?}",
                challenge.challenge_id, elapsed
            );

            // Warn if generation took too long
            if elapsed.as_secs() > 10 {
                tracing::warn!(
                    "Slow CAPTCHA generation detected: challenge_id={}, elapsed={:?}",
                    challenge.challenge_id, elapsed
                );
            }

            (StatusCode::OK, Json(challenge)).into_response()
        }
        Err(err) => {
            let elapsed = start_time.elapsed();
            error!(
                "Failed to generate CAPTCHA: error={}, session_id={}, elapsed={:?}",
                err, request.session_id, elapsed
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to generate CAPTCHA: {}", err),
                }),
            )
                .into_response()
        }
    }
}

/// POST /api/captcha/verify - Verify CAPTCHA response
pub async fn verify_captcha(
    State(state): State<Arc<AppState>>,
    Json(request): Json<VerifyRequest>,
) -> impl IntoResponse {
    let start_time = std::time::Instant::now();

    info!(
        "CAPTCHA verify request: challenge_id={}, session_id={}",
        request.challenge_id, request.session_id
    );

    // Call Authenc via gRPC to verify CAPTCHA
    match state
        .authenc
        .verify_captcha(
            &request.challenge_id,
            &request.answer,
            &request.session_id,
            request.behavioral_data,
        )
        .await
    {
        Ok(token_opt) => {
            let elapsed = start_time.elapsed();
            let success = token_opt.is_some();

            info!(
                "CAPTCHA verification result: challenge_id={}, success={}, elapsed={:?}",
                request.challenge_id, success, elapsed
            );

            // Warn if verification took too long
            if elapsed.as_secs() > 5 {
                tracing::warn!(
                    "Slow CAPTCHA verification detected: challenge_id={}, elapsed={:?}",
                    request.challenge_id, elapsed
                );
            }

            let response = if let Some(token) = token_opt {
                VerifyResponse {
                    success: true,
                    message: "CAPTCHA verified successfully".to_string(),
                    token: Some(token),
                }
            } else {
                VerifyResponse {
                    success: false,
                    message: "Invalid CAPTCHA response".to_string(),
                    token: None,
                }
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(err) => {
            let elapsed = start_time.elapsed();
            error!(
                "Failed to verify CAPTCHA: error={}, challenge_id={}, elapsed={:?}",
                err, request.challenge_id, elapsed
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to verify CAPTCHA: {}", err),
                }),
            )
                .into_response()
        }
    }
}

/// Routes for CAPTCHA handlers
pub fn routes() -> axum::Router<Arc<AppState>> {
    use axum::routing::post;

    axum::Router::new()
        .route("/challenge", post(generate_challenge))
        .route("/verify", post(verify_captcha))
}
