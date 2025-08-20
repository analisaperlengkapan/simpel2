use crate::error::AppError;
use config::{Config as ConfigFile, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub vault_url: String,
    pub vault_token: String,
    pub server_port: u16,
    pub server_host: String,
    pub log_level: String,
    pub cors_origins: Vec<String>,
    pub rate_limit_requests: u32,
    pub rate_limit_duration: u64,
}

impl Config {
    pub fn load() -> Result<Self, AppError> {
        let config_file = ConfigFile::builder()
            // Start with default settings
            .add_source(File::with_name("config/default"))
            // Add environment-specific settings
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables with prefix "APP_"
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        let config: Config = config_file.try_deserialize()?;

        // Validate required fields
        if config.database_url.is_empty() {
            return Err(AppError::ConfigurationError("Database URL is required".to_string()));
        }

        if config.jwt_secret.is_empty() {
            return Err(AppError::ConfigurationError("JWT secret is required".to_string()));
        }

        if config.vault_url.is_empty() {
            return Err(AppError::ConfigurationError("Vault URL is required".to_string()));
        }

        Ok(config)
    }

    pub fn from_env() -> Result<Self, AppError> {
        dotenv::dotenv().ok();

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://simpel:simpel@localhost:5432/simpelv2".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string()),
            vault_url: env::var("VAULT_URL")
                .unwrap_or_else(|_| "http://localhost:8200".to_string()),
            vault_token: env::var("VAULT_TOKEN")
                .unwrap_or_else(|_| "your-vault-token".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .unwrap_or(3001),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,http://localhost:8080".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            rate_limit_requests: env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            rate_limit_duration: env::var("RATE_LIMIT_DURATION")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
        })
    }
} 