use crate::error::IntegrationError;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub api_key: String,
    pub cors_origins: Vec<String>,
    pub log_level: String,
    pub metrics_port: u16,
    pub sentry_dsn: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, IntegrationError> {
        dotenvy::dotenv().ok();

        let port_str = env::var("SERVER_PORT").unwrap_or_else(|_| "3008".to_string());
        let port: u16 = port_str.parse().map_err(|_| {
            IntegrationError::Internal(format!("Invalid SERVER_PORT: {}", port_str))
        })?;

        let metrics_port_str = env::var("METRICS_PORT").unwrap_or_else(|_| "9092".to_string());
        let metrics_port: u16 = metrics_port_str.parse().map_err(|_| {
            IntegrationError::Internal(format!("Invalid METRICS_PORT: {}", metrics_port_str))
        })?;

        Ok(Self {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port,
            database_url: env::var("DATABASE_URL")
                .map_err(|_| IntegrationError::Internal("DATABASE_URL wajib di-set".to_string()))?,
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            api_key: env::var("API_KEY")
                .map_err(|_| IntegrationError::Internal("API_KEY wajib di-set".to_string()))?,
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "*".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            metrics_port,
            sentry_dsn: env::var("SENTRY_DSN").ok(),
        })
    }
}
