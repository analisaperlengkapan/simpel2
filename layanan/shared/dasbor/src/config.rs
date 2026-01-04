use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub server_port: u16,
    pub server_host: String,
    pub api_key: String,
    pub cors_origins: Vec<String>,
    pub log_level: String,
    pub metrics_port: u16,
    pub sentry_dsn: Option<String>,
    pub dashboard_refresh_interval: u64,
    pub chart_cache_ttl: u64,
    pub real_time_update_interval: u64,
    pub max_concurrent_connections: u32,
    pub aggregator_batch_size: u32,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL wajib di-set"),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3007".to_string())
                .parse()
                .unwrap_or(3007),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            api_key: env::var("API_KEY").expect("API_KEY wajib di-set"),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "*".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            metrics_port: env::var("METRICS_PORT")
                .unwrap_or_else(|_| "9091".to_string())
                .parse()
                .unwrap_or(9091),
            sentry_dsn: env::var("SENTRY_DSN").ok(),
            dashboard_refresh_interval: env::var("DASHBOARD_REFRESH_INTERVAL")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            chart_cache_ttl: env::var("CHART_CACHE_TTL")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
            real_time_update_interval: env::var("REAL_TIME_UPDATE_INTERVAL")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            max_concurrent_connections: env::var("MAX_CONCURRENT_CONNECTIONS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            aggregator_batch_size: env::var("AGGREGATOR_BATCH_SIZE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
        }
    }
}
