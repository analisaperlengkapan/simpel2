//! CAPTCHA Service Module
//!
//! AI-resistant CAPTCHA system integrated with authenc security infrastructure

pub mod adaptive_difficulty;
pub mod alerting;
pub mod analyzer;
pub mod audio_challenges;
pub mod bot_detection;
pub mod challenge_selector;
pub mod dashboard;
pub mod enhanced_service;
pub mod error;
pub mod fallback;
pub mod fingerprinting;
pub mod generator;
pub mod image_challenges;
pub mod metrics;
pub mod rate_limiting;
pub mod retry;
pub mod risk_assessment;
pub mod secreton_integration;
pub mod security_monitoring;
pub mod service;
pub mod types;
pub mod validator;

#[cfg(test)]
mod test_generator;

#[cfg(test)]
pub mod tests;

// Re-exports
pub use adaptive_difficulty::*;
pub use alerting::*;
pub use analyzer::*;
pub use audio_challenges::*;
pub use bot_detection::*;
pub use challenge_selector::*;
pub use dashboard::*;
pub use enhanced_service::*;
pub use error::*;
pub use fallback::*;
pub use fingerprinting::*;
pub use generator::*;
pub use image_challenges::*;
pub use metrics::*;
pub use rate_limiting::*;
pub use retry::*;
pub use risk_assessment::*;
pub use secreton_integration::*;
pub use security_monitoring::*;
pub use service::*;
pub use types::*;
pub use validator::*;
