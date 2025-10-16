use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from: String,
    pub whatsapp_api_url: Option<String>,
    pub whatsapp_access_token: Option<String>,
    pub whatsapp_phone_number_id: Option<String>,
    pub fcm_server_key: Option<String>,
    pub apns_key_id: Option<String>,
    pub apns_team_id: Option<String>,
    pub apns_private_key: Option<String>,
    pub server_port: u16,
    pub server_host: String,
    pub api_key: String,
    pub cors_origins: Vec<String>,
    pub rate_limit_emails: u32,
    pub rate_limit_whatsapp: u32,
    pub rate_limit_push: u32,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL wajib di-set"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL wajib di-set"),
            smtp_host: env::var("SMTP_HOST").expect("SMTP_HOST wajib di-set"),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587),
            smtp_username: env::var("SMTP_USERNAME").expect("SMTP_USERNAME wajib di-set"),
            smtp_password: env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD wajib di-set"),
            smtp_from: env::var("SMTP_FROM")
                .unwrap_or_else(|_| "noreply@simpelv2.go.id".to_string()),
            whatsapp_api_url: env::var("WHATSAPP_API_URL").ok(),
            whatsapp_access_token: env::var("WHATSAPP_ACCESS_TOKEN").ok(),
            whatsapp_phone_number_id: env::var("WHATSAPP_PHONE_NUMBER_ID").ok(),
            fcm_server_key: env::var("FCM_SERVER_KEY").ok(),
            apns_key_id: env::var("APNS_KEY_ID").ok(),
            apns_team_id: env::var("APNS_TEAM_ID").ok(),
            apns_private_key: env::var("APNS_PRIVATE_KEY").ok(),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3004".to_string())
                .parse()
                .unwrap_or(3004),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            api_key: env::var("API_KEY").expect("API_KEY wajib di-set"),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "*".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            rate_limit_emails: env::var("RATE_LIMIT_EMAILS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            rate_limit_whatsapp: env::var("RATE_LIMIT_WHATSAPP")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            rate_limit_push: env::var("RATE_LIMIT_PUSH")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
        }
    }
}
