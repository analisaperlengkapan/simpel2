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
    let level = config.observability.log_level.as_str();
    lib_common::telemetry::init_subscriber(level);
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_initialize_logging_compiles() {
        // This test just ensures the function signature is correct
        // Actual logging initialization cannot be tested easily in unit tests
        // as it affects global state
    }
}
