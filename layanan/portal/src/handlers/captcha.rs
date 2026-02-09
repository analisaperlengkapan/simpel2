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
    #[serde(default)]
    pub session_id: Option<String>,
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
    #[serde(default)]
    pub session_id: Option<String>,
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
    // Generate session_id if not provided
    let session_id = request
        .session_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    info!("CAPTCHA challenge request: session_id={}", session_id);

    // Call Authenc via gRPC to generate CAPTCHA
    match state
        .authenc
        .generate_captcha(&request.challenge_type, request.difficulty, &session_id)
        .await
    {
        Ok(challenge) => {
            info!("CAPTCHA challenge generated: {}", challenge.challenge_id);
            (StatusCode::OK, Json(challenge)).into_response()
        }
        Err(err) => {
            error!("Failed to generate CAPTCHA: {}", err);
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
    info!(
        "CAPTCHA verify request: challenge_id={}",
        request.challenge_id
    );

    // Use session_id if provided, otherwise use a default based on challenge_id
    let session_id = request
        .session_id
        .unwrap_or_else(|| request.challenge_id.clone());

    // Call Authenc via gRPC to verify CAPTCHA
    match state
        .authenc
        .verify_captcha(
            &request.challenge_id,
            &request.answer,
            &session_id,
            request.behavioral_data,
        )
        .await
    {
        Ok(token_opt) => {
            info!("CAPTCHA verification result: {:?}", token_opt.is_some());

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
            error!("Failed to verify CAPTCHA: {}", err);
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
