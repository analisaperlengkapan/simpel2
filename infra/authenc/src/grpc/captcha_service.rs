//! CAPTCHA gRPC Service Implementation
//!
//! gRPC service layer that wraps the CAPTCHA service for remote access via gRPC

use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::app::AppState;
use crate::services::captcha::{
    BehavioralMetrics, CaptchaServiceTrait, ChallengeType as ServiceChallengeType,
};

use crate::grpc::proto::authenc::v1 as proto;
use proto::{
    CaptchaChallengeRequest, CaptchaChallengeResponse, CaptchaVerificationRequest,
    CaptchaVerificationResponse, ChallengeType,
};

/// CAPTCHA gRPC service implementation
pub struct CaptchaGrpcService {
    state: Arc<AppState>,
}

impl CaptchaGrpcService {
    /// Create a new CAPTCHA gRPC service
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    /// Convert proto ChallengeType to service ChallengeType
    fn convert_challenge_type(proto_type: i32) -> ServiceChallengeType {
        match proto_type {
            1 => ServiceChallengeType::Visual,
            2 => ServiceChallengeType::Audio,
            3 => ServiceChallengeType::Behavioral,
            4 => ServiceChallengeType::Logical,
            5 => ServiceChallengeType::Hybrid,
            _ => ServiceChallengeType::Visual, // Default
        }
    }

    /// Convert service ChallengeType to proto ChallengeType
    fn convert_challenge_type_to_proto(service_type: &ServiceChallengeType) -> i32 {
        match service_type {
            ServiceChallengeType::Visual => ChallengeType::Visual as i32,
            ServiceChallengeType::Audio => ChallengeType::Audio as i32,
            ServiceChallengeType::Behavioral => ChallengeType::Behavioral as i32,
            ServiceChallengeType::Logical => ChallengeType::Logical as i32,
            ServiceChallengeType::Hybrid => ChallengeType::Hybrid as i32,
        }
    }
}

impl CaptchaGrpcService {
    /// Generate CAPTCHA challenge (gRPC endpoint)
    pub async fn generate_captcha_challenge(
        &self,
        request: Request<CaptchaChallengeRequest>,
    ) -> Result<Response<CaptchaChallengeResponse>, Status> {
        let start_time = std::time::Instant::now();

        // Extract IP from metadata or use a default
        let ip_address = request
            .remote_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "0.0.0.0".to_string());

        let req = request.into_inner();

        tracing::info!(
            "gRPC CAPTCHA generate request: session_id={}, type={}, ip={}",
            req.session_id,
            req.challenge_type,
            ip_address
        );

        let challenge_type = Self::convert_challenge_type(req.challenge_type);

        // Track risk engine calculation time
        let risk_start = std::time::Instant::now();
        let difficulty = self
            .state
            .risk_engine
            .calculate_difficulty(&ip_address, Some(req.difficulty as u8))
            .await;
        let risk_elapsed = risk_start.elapsed();

        if risk_elapsed.as_millis() > 1000 {
            tracing::warn!(
                "Slow risk engine calculation: session_id={}, elapsed={:?}",
                req.session_id,
                risk_elapsed
            );
        }

        // Track challenge generation time
        let gen_start = std::time::Instant::now();
        let challenge = self
            .state
            .captcha_service
            .generate_challenge(
                challenge_type,
                Some(difficulty),
                Some(req.session_id.clone()),
                ip_address.clone(),
            )
            .await
            .map_err(|e| {
                tracing::error!(
                    "Failed to generate challenge: session_id={}, error={}",
                    req.session_id,
                    e
                );
                Status::internal(format!("Failed to generate challenge: {}", e))
            })?;
        let gen_elapsed = gen_start.elapsed();

        if gen_elapsed.as_millis() > 5000 {
            tracing::warn!(
                "Slow CAPTCHA generation: session_id={}, challenge_id={}, elapsed={:?}",
                req.session_id,
                challenge.id,
                gen_elapsed
            );
        }

        // Convert to proto response
        let response = CaptchaChallengeResponse {
            challenge_id: challenge.id.clone(),
            challenge_type: Self::convert_challenge_type_to_proto(&challenge.challenge_type),
            // Use plaintext data if available (Bug 17 fix), otherwise fallback to encrypted_data field
            challenge_data: challenge.plaintext_data.unwrap_or(challenge.encrypted_data),
            difficulty: challenge.difficulty_level as u32,
            expires_at: challenge
                .expires_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            metadata: req.metadata,
        };

        let total_elapsed = start_time.elapsed();
        tracing::info!(
            "gRPC CAPTCHA generated: challenge_id={}, total_elapsed={:?} (risk={:?}, gen={:?})",
            challenge.id,
            total_elapsed,
            risk_elapsed,
            gen_elapsed
        );

        Ok(Response::new(response))
    }

    /// Verify CAPTCHA challenge (gRPC endpoint)
    pub async fn verify_captcha_challenge(
        &self,
        request: Request<CaptchaVerificationRequest>,
    ) -> Result<Response<CaptchaVerificationResponse>, Status> {
        let start_time = std::time::Instant::now();

        // Get IP from metadata (must be done before consuming request)
        let ip_address = request
            .remote_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "0.0.0.0".to_string());

        let req = request.into_inner();

        tracing::info!(
            "gRPC CAPTCHA verify request: challenge_id={}, session_id={}, ip={}",
            req.challenge_id,
            req.session_id,
            ip_address
        );

        // Get CAPTCHA service from app state
        let captcha_service = &self.state.captcha_service;

        // Parse behavioral data if provided
        let behavioral_data = if !req.behavioral_data.is_empty() {
            // Try to deserialize JSON behavioral data
            serde_json::from_slice::<BehavioralMetrics>(&req.behavioral_data).ok()
        } else {
            None
        };

        // Track validation time
        let validation_start = std::time::Instant::now();
        let validation_result = captcha_service
            .validate_challenge(req.challenge_id.clone(), req.answer.clone(), behavioral_data)
            .await
            .map_err(|e| {
                tracing::error!(
                    "CAPTCHA validation error: challenge_id={}, error={}",
                    req.challenge_id,
                    e
                );
                match e {
                    crate::services::captcha::CaptchaError::ChallengeNotFound { .. } => {
                        Status::not_found("Challenge not found")
                    }
                    crate::services::captcha::CaptchaError::ChallengeExpired { .. } => {
                        Status::failed_precondition("Challenge expired")
                    }
                    _ => Status::internal(format!("Validation failed: {}", e)),
                }
            })?;
        let validation_elapsed = validation_start.elapsed();

        if validation_elapsed.as_millis() > 2000 {
            tracing::warn!(
                "Slow CAPTCHA validation: challenge_id={}, elapsed={:?}",
                req.challenge_id,
                validation_elapsed
            );
        }

        // Update risk score based on validation result
        if validation_result.success {
            self.state.risk_engine.record_success(&ip_address).await;
        } else {
            self.state.risk_engine.record_failure(&ip_address).await;
        }

        // Generate verification token if successful
        let verification_token = if validation_result.success {
            // Generate a one-time verification token
            format!(
                "captcha_verified_{}_{}",
                req.challenge_id,
                chrono::Utc::now().timestamp()
            )
        } else {
            String::new()
        };

        // Calculate lockout duration in seconds
        let lockout_duration = validation_result
            .lockout_duration
            .map(|d| d.as_secs() as u32);

        // Convert to proto response
        let response = CaptchaVerificationResponse {
            success: validation_result.success,
            message: validation_result.message.clone(),
            verification_token: verification_token.clone(),
            next_difficulty: validation_result.next_difficulty as u32,
            retry_allowed: validation_result.retry_allowed,
            lockout_duration,
        };

        let total_elapsed = start_time.elapsed();
        tracing::info!(
            "gRPC CAPTCHA verified: challenge_id={}, success={}, total_elapsed={:?} (validation={:?})",
            req.challenge_id,
            validation_result.success,
            total_elapsed,
            validation_elapsed
        );

        Ok(Response::new(response))
    }
}
