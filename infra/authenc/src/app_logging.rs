//! Logging initialization helper
//!
//! Provides logging configuration for the Authenc application.
//! This module is extracted from app.rs to improve code organization.

use crate::config::AppConfig;
use crate::error::Result;

/// Initialize logging based on configuration
///
/// Sets up tracing subscribers with appropriate log levels and formatting.
/// Log level can be overridden via RUST_LOG environment variable.
pub fn initialize_logging(config: &AppConfig) -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let level = match config.observability.log_level.as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        "trace" => tracing::Level::TRACE,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| level.as_str().to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_logging_compiles() {
        // This test just ensures the function signature is correct
        // Actual logging initialization cannot be tested easily in unit tests
        // as it affects global state
    }
}
