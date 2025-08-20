use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub ai_service_url: String,
    pub ai_service_api_key: Option<String>,
    pub storage_path: String,
    pub max_file_size: u64,
    pub allowed_extensions: Vec<String>,
    pub server_port: u16,
    pub server_host: String,
    pub encryption_key: String,
    pub cors_origins: Vec<String>,
    pub log_level: String,
    pub metrics_port: u16,
    pub sentry_dsn: Option<String>,
    pub clamav_host: Option<String>,
    pub clamav_port: Option<u16>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL wajib di-set"),
            ai_service_url: env::var("AI_SERVICE_URL").expect("AI_SERVICE_URL wajib di-set"),
            ai_service_api_key: env::var("AI_SERVICE_API_KEY").ok(),
            storage_path: env::var("STORAGE_PATH").unwrap_or_else(|_| "./storage".to_string()),
            max_file_size: parse_size(&env::var("MAX_FILE_SIZE").unwrap_or_else(|_| "100MB".to_string())),
            allowed_extensions: env::var("ALLOWED_EXTENSIONS").unwrap_or_else(|_| "pdf,doc,docx,jpg,png".to_string()).split(',').map(|s| s.trim().to_lowercase()).collect(),
            server_port: env::var("SERVER_PORT").unwrap_or_else(|_| "3003".to_string()).parse().unwrap_or(3003),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            encryption_key: env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY wajib di-set"),
            cors_origins: env::var("CORS_ORIGINS").unwrap_or_else(|_| "*".to_string()).split(',').map(|s| s.trim().to_string()).collect(),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            metrics_port: env::var("METRICS_PORT").unwrap_or_else(|_| "9090".to_string()).parse().unwrap_or(9090),
            sentry_dsn: env::var("SENTRY_DSN").ok(),
            clamav_host: env::var("CLAMAV_HOST").ok(),
            clamav_port: env::var("CLAMAV_PORT").ok().and_then(|v| v.parse().ok()),
        }
    }
}

fn parse_size(s: &str) -> u64 {
    let s = s.trim().to_uppercase();
    if let Some(mb) = s.strip_suffix("MB") {
        mb.trim().parse::<u64>().unwrap_or(100) * 1024 * 1024
    } else if let Some(gb) = s.strip_suffix("GB") {
        gb.trim().parse::<u64>().unwrap_or(1) * 1024 * 1024 * 1024
    } else {
        s.parse::<u64>().unwrap_or(100 * 1024 * 1024)
    }
} 