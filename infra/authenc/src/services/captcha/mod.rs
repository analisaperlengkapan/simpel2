//! CAPTCHA Service Module
//!
//! AI-resistant CAPTCHA system integrated with authenc security infrastructure

pub mod adaptive_difficulty;
/// Modul `alerting`.
pub mod alerting;
/// Modul `audio_challenges`.
pub mod analyzer;
/// Modul `challenge_selector`.
pub mod audio_challenges;
/// Modul `enhanced_service`.
pub mod bot_detection;
/// Modul `fallback`.
pub mod challenge_selector;
/// Modul `generator`.
pub mod dashboard;
/// Modul `metrics`.
pub mod enhanced_service;
/// Modul `retry`.
pub mod error;
/// Modul `secreton_integration`.
pub mod fallback;
/// Modul `service`.
pub mod fingerprinting;
/// Modul `validator`.
pub mod generator;
/// Modul `image_challenges`.
pub mod image_challenges;
/// Modul `rate_limiting`.
pub mod metrics;
/// Modul `rate_limiting`.
pub mod rate_limiting;
/// Modul `risk_assessment`.
/// Modul `tests`.
pub mod retry;
/// Modul `service`.
pub mod risk_assessment;
/// Modul `validator`.
pub mod secreton_integration;
/// Modul `service`.
pub mod security_monitoring;
pub mod service;
pub mod types;
/// Modul `tests`.
pub mod validator;

#[cfg(test)]
/// Modul `tests`.
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
