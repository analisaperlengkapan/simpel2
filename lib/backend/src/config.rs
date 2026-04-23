//! Configuration loading from environment variables

use lib_core::config::BaseServiceConfig;

/// Load BaseServiceConfig from environment variables
pub fn load_base_config() -> BaseServiceConfig {
    // Load .env file if present
    let _ = dotenvy::dotenv();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/perlengkapan".to_string()
    });

    let database_pool_size = std::env::var("DATABASE_POOL_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(lib_core::config::default_pool_size());

    let server_port = std::env::var("SERVER_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(lib_core::config::default_service_port());

    let server_host =
        std::env::var("SERVER_HOST").unwrap_or_else(|_| lib_core::config::default_service_host());

    let log_level =
        std::env::var("LOG_LEVEL").unwrap_or_else(|_| lib_core::config::default_log_level());

    BaseServiceConfig {
        database_url,
        database_pool_size,
        server_port,
        server_host,
        log_level,
    }
}
