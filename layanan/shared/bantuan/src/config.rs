use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub ai_service_url: String,
    pub ai_service_api_key: Option<String>,
    pub server_port: u16,
    pub server_host: String,
    pub api_key: String,
    pub cors_origins: Vec<String>,
    pub search_index_path: String,
    pub search_max_results: u32,
    pub log_level: String,
    pub metrics_port: u16,
    pub sentry_dsn: Option<String>,
    pub rate_limit_ticket: u32,
    pub rate_limit_chatbot: u32,
    pub captcha_secret: String,
    pub redis_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL wajib di-set"),
            ai_service_url: env::var("AI_SERVICE_URL").expect("AI_SERVICE_URL wajib di-set"),
            ai_service_api_key: env::var("AI_SERVICE_API_KEY").ok(),
            server_port: env::var("SERVER_PORT").unwrap_or_else(|_| "3006".to_string()).parse().unwrap_or(3006),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            api_key: env::var("API_KEY").expect("API_KEY wajib di-set"),
            cors_origins: env::var("CORS_ORIGINS").unwrap_or_else(|_| "*".to_string()).split(',').map(|s| s.trim().to_string()).collect(),
            search_index_path: env::var("SEARCH_INDEX_PATH").unwrap_or_else(|_| "./search-index".to_string()),
            search_max_results: env::var("SEARCH_MAX_RESULTS").unwrap_or_else(|_| "50".to_string()).parse().unwrap_or(50),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            metrics_port: env::var("METRICS_PORT").unwrap_or_else(|_| "9090".to_string()).parse().unwrap_or(9090),
            sentry_dsn: env::var("SENTRY_DSN").ok(),
            rate_limit_ticket: env::var("RATE_LIMIT_TICKET").unwrap_or_else(|_| "30".to_string()).parse().unwrap_or(30),
            rate_limit_chatbot: env::var("RATE_LIMIT_CHATBOT").unwrap_or_else(|_| "60".to_string()).parse().unwrap_or(60),
            captcha_secret: env::var("CAPTCHA_SECRET").unwrap_or_else(|_| "changeme".to_string()),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        }
    }
} 