//! # Configuration
//!
//! Application configuration management

use std::env;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub run_migrations: bool,
    pub log_level: String,
    pub cors_origin: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| anyhow!("Invalid PORT value"))?;

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/simpelv2_perlengkapan".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());

        let run_migrations = env::var("RUN_MIGRATIONS")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "debug".to_string());

        let cors_origin = env::var("CORS_ORIGIN")
            .unwrap_or_else(|_| "*".to_string());

        Ok(Self {
            port,
            database_url,
            jwt_secret,
            run_migrations,
            log_level,
            cors_origin,
        })
    }
}
