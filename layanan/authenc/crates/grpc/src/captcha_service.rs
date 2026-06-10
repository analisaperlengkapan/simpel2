//! CAPTCHA gRPC Service Implementation
//!
//! gRPC service layer that wraps the CAPTCHA service for remote access via gRPC.
//!
//! TODO: Implement when the following are available in authenc-core:
//! - `authenc_core::services::captcha::CaptchaServiceTrait`
//! - `authenc_core::services::captcha::BehavioralMetrics`
//! - `authenc_core::services::captcha::ChallengeType`
//! - `authenc_core::services::captcha::CaptchaError`
//! - A RiskEngine trait abstraction for dynamic dispatch
//!
//! The gRPC AuthencService already provides stubbed CAPTCHA endpoints
//! (generate_captcha_challenge / verify_captcha_challenge) in service.rs
//! that return `Status::unimplemented`.

/// CAPTCHA gRPC service (stub)
///
/// This will wrap a future `CaptchaServiceTrait` implementation and expose
/// CAPTCHA challenge generation and verification over gRPC.
///
/// TODO: Implement with real `CaptchaServiceTrait` and `RiskEngine` trait
/// once they are defined in `authenc-core`.
pub struct CaptchaGrpcService;

impl Default for CaptchaGrpcService {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptchaGrpcService {
    /// Create a new stub CAPTCHA gRPC service
    pub fn new() -> Self {
        Self
    }
}
