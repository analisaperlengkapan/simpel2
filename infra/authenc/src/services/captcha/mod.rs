//! CAPTCHA Service Module
//!
//! AI-resistant CAPTCHA system integrated with authenc security infrastructure

pub mod adaptive_difficulty;
pub mod alerting;
pub mod analyzer;
pub mod bot_detection;
pub mod dashboard;
pub mod enhanced_service;
pub mod error;
pub mod fallback;
pub mod fingerprinting;
pub mod generator;
pub mod metrics;
pub mod rate_limiting;
pub mod retry;
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
pub use bot_detection::*;
pub use dashboard::*;
pub use enhanced_service::*;
pub use error::*;
pub use fallback::*;
pub use fingerprinting::*;
pub use generator::*;
pub use metrics::*;
pub use rate_limiting::*;
pub use retry::*;
pub use secreton_integration::*;
pub use security_monitoring::*;
pub use service::*;
pub use types::*;
pub use validator::*;
