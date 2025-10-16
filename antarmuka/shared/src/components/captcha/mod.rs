//! CAPTCHA Components
//!
//! AI-resistant CAPTCHA components for SIMPelv2 authentication system.
//! Integrates with authenc and secreton for secure challenge generation and validation.

pub mod accessibility;
pub mod behavioral;
pub mod behavioral_collector;
pub mod behavioral_tracker;
pub mod component;
pub mod fingerprint_collector;
pub mod types;
pub mod validation_feedback;

// Re-exports - using specific imports to avoid conflicts
pub use accessibility::AudioChallenge;
pub use behavioral::{KeystrokeAnalyzer, MouseTracker};
pub use behavioral_collector::*;
pub use behavioral_tracker::*;
pub use component::Captcha;
pub use fingerprint_collector::*;
pub use types::{BehaviorClassification, CaptchaProps, CaptchaState, ChallengeType, RiskLevel};
pub use validation_feedback::*;
