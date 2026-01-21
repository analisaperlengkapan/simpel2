use lib_common::config::BaseServiceConfig;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// Configuration for the Notifikasi service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Common service configuration
    #[serde(flatten)]
    pub base: BaseServiceConfig,

    /// SMTP configuration
    #[serde(default)]
    pub smtp_host: String,
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
    #[serde(default)]
    pub smtp_username: String,
    #[serde(default)]
    pub smtp_password: String,
    #[serde(default)]
    pub smtp_from: String,

    /// FCM configuration
    pub fcm_server_key: Option<String>,

    /// WhatsApp configuration
    pub whatsapp_api_url: Option<String>,
    pub whatsapp_access_token: Option<String>,
    pub whatsapp_phone_number_id: Option<String>,

    /// Redis configuration
    #[serde(default = "default_redis_url")]
    pub redis_url: String,

    /// API security
    #[serde(default)]
    pub api_key: String,
}

fn default_smtp_port() -> u16 {
    587
}

fn default_redis_url() -> String {
    "redis://localhost:6379".to_string()
}

// Allow accessing BaseServiceConfig fields directly
impl Deref for AppConfig {
    type Target = BaseServiceConfig;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            base: BaseServiceConfig::default(),
            smtp_host: String::new(),
            smtp_port: default_smtp_port(),
            smtp_username: String::new(),
            smtp_password: String::new(),
            smtp_from: String::new(),
            fcm_server_key: None,
            whatsapp_api_url: None,
            whatsapp_access_token: None,
            whatsapp_phone_number_id: None,
            redis_url: default_redis_url(),
            api_key: String::new(),
        }
    }
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();

        let base = BaseServiceConfig::from_env();

        Self {
            base,
            smtp_host: std::env::var("SMTP_HOST").unwrap_or_default(),
            smtp_port: std::env::var("SMTP_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default_smtp_port()),
            smtp_username: std::env::var("SMTP_USERNAME").unwrap_or_default(),
            smtp_password: std::env::var("SMTP_PASSWORD").unwrap_or_default(),
            smtp_from: std::env::var("SMTP_FROM").unwrap_or_default(),
            fcm_server_key: std::env::var("FCM_SERVER_KEY").ok(),
            whatsapp_api_url: std::env::var("WHATSAPP_API_URL").ok(),
            whatsapp_access_token: std::env::var("WHATSAPP_ACCESS_TOKEN").ok(),
            whatsapp_phone_number_id: std::env::var("WHATSAPP_PHONE_NUMBER_ID").ok(),
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| default_redis_url()),
            api_key: std::env::var("API_KEY").unwrap_or_default(),
        }
    }
}
