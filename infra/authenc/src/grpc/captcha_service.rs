//! CAPTCHA gRPC Service Implementation
//!
//! gRPC service layer that wraps the CAPTCHA service for remote access via gRPC

use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::app::AppState;
use crate::services::captcha::{
    BehavioralMetrics, CaptchaServiceTrait, ChallengeType as ServiceChallengeType,
};

// Include generated proto code
pub mod proto {
    pub mod authenc {
        pub mod v1 {
            tonic::include_proto!("authenc.v1");
        }
    }
}

use proto::authenc::v1::{
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
        // Extract IP from metadata or use a default
        let ip_address = request
            .remote_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "0.0.0.0".to_string());

        let req = request.into_inner();

        let challenge_type = Self::convert_challenge_type(req.challenge_type);
        // Generate dynamic difficulty based on risk
        let difficulty = self
            .state
            .risk_engine
            .calculate_difficulty(&ip_address, Some(req.difficulty as u8))
            .await;

        // Generate challenge
        let challenge = self
            .state
            .captcha_service
            .generate_challenge(
                challenge_type,
                Some(difficulty),
                Some(req.session_id.clone()),
                ip_address,
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to generate challenge: {}", e)))?;

        // Convert to proto response
        let response = CaptchaChallengeResponse {
            challenge_id: challenge.id,
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

        Ok(Response::new(response))
    }

    /// Verify CAPTCHA challenge (gRPC endpoint)
    pub async fn verify_captcha_challenge(
        &self,
        request: Request<CaptchaVerificationRequest>,
    ) -> Result<Response<CaptchaVerificationResponse>, Status> {
        // Get IP from metadata (must be done before consuming request)
        let ip_address = request
            .remote_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "0.0.0.0".to_string());

        let req = request.into_inner();

        // Get CAPTCHA service from app state
        let captcha_service = &self.state.captcha_service;

        // Parse behavioral data if provided
        let behavioral_data = if !req.behavioral_data.is_empty() {
            // Try to deserialize JSON behavioral data
            serde_json::from_slice::<BehavioralMetrics>(&req.behavioral_data).ok()
        } else {
            None
        };

        // Validate challenge
        let validation_result = captcha_service
            .validate_challenge(req.challenge_id.clone(), req.answer, behavioral_data)
            .await
            .map_err(|e| match e {
                crate::services::captcha::CaptchaError::ChallengeNotFound { .. } => {
                    Status::not_found("Challenge not found")
                }
                crate::services::captcha::CaptchaError::ChallengeExpired { .. } => {
                    Status::failed_precondition("Challenge expired")
                }
                _ => Status::internal(format!("Validation failed: {}", e)),
            })?;

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
            message: validation_result.message,
            verification_token,
            next_difficulty: validation_result.next_difficulty as u32,
            retry_allowed: validation_result.retry_allowed,
            lockout_duration,
        };

        Ok(Response::new(response))
    }
}
