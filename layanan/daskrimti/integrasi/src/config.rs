use anyhow::Result;
use lib_common::config::BaseServiceConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;

/// Configuration for the Integrasi service
/// Extends BaseServiceConfig with fields specific to MonSAKTI, MySIMKARI, and SIMAN
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Common service configuration (database, server, logging)
    #[serde(flatten)]
    pub base: BaseServiceConfig,

    /// MonSAKTI API Base URL
    #[serde(default = "default_monsakti_base_url")]
    pub base_url: String,

    /// MySIMKARI API Base URL
    #[serde(default = "default_mysimkari_base_url")]
    pub mysimkari_base_url: String,

    /// Output directory for file exports
    #[serde(default = "default_output_dir")]
    pub output_dir: String,

    /// Initial tokens for modules
    #[serde(default)]
    pub tokens: HashMap<String, String>,

    /// Database connection string (optional override or duplicate of base.database_url)
    /// Used by client logic to determine if DB features should be enabled
    pub db_config: Option<String>,

    // === SIMAN Configuration ===
    /// SIMAN OAuth2 Client ID
    pub siman_client_id: Option<String>,

    /// SIMAN OAuth2 Client Secret
    pub siman_client_secret: Option<String>,

    /// SIMAN Token Endpoint
    #[serde(default = "default_siman_token_url")]
    pub siman_token_url: String,

    /// SIMAN BA Key (Unit Key)
    pub siman_ba_key: Option<String>,

    /// SIMAN API Base URL
    #[serde(default = "default_siman_base_url")]
    pub siman_base_url: String,

    /// Concurrency limit for SIMAN batch operations
    #[serde(default = "default_concurrency_limit")]
    pub siman_concurrency_limit: usize,
}

// Allow accessing BaseServiceConfig fields directly
impl Deref for Config {
    type Target = BaseServiceConfig;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

// Defaults
fn default_monsakti_base_url() -> String {
    "https://monsakti.kemenkeu.go.id".to_string()
}

fn default_mysimkari_base_url() -> String {
    "https://mysimkari.kejaksaan.go.id".to_string()
}

fn default_output_dir() -> String {
    "output".to_string()
}

fn default_siman_token_url() -> String {
    "https://sso.kemenkeu.go.id/connect/token".to_string()
}

fn default_siman_base_url() -> String {
    "https://api.kemenkeu.go.id".to_string()
}

fn default_concurrency_limit() -> usize {
    5
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        // Load .env file
        let _ = dotenvy::dotenv();

        // Load base config
        let base = BaseServiceConfig::from_env();

        // Load specific config
        let base_url =
            std::env::var("MONSAKTI_BASE_URL").unwrap_or_else(|_| default_monsakti_base_url());

        let mysimkari_base_url =
            std::env::var("MYSIMKARI_BASE_URL").unwrap_or_else(|_| default_mysimkari_base_url());

        let output_dir = std::env::var("OUTPUT_DIR").unwrap_or_else(|_| default_output_dir());

        // Helper to load tokens from generic env vars if needed
        let mut tokens = HashMap::new();
        if let Ok(token) = std::env::var("MONSAKTI_TOKEN_DEFAULT") {
            tokens.insert("default".to_string(), token);
        }
        // Specific modules can be added logic here if needed, e.g. MONSAKTI_TOKEN_AST

        // Set db_config from base.database_url if available
        let db_config = if !base.database_url.is_empty() {
            Some(base.database_url.clone())
        } else {
            None
        };

        // SIMAN
        let siman_client_id = std::env::var("SIMAN_CLIENT_ID").ok();
        let siman_client_secret = std::env::var("SIMAN_CLIENT_SECRET").ok();
        let siman_token_url =
            std::env::var("SIMAN_TOKEN_URL").unwrap_or_else(|_| default_siman_token_url());
        let siman_ba_key = std::env::var("SIMAN_BA_KEY").ok();
        let siman_base_url =
            std::env::var("SIMAN_BASE_URL").unwrap_or_else(|_| default_siman_base_url());
        let siman_concurrency_limit = std::env::var("SIMAN_CONCURRENCY_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(default_concurrency_limit);

        Ok(Self {
            base,
            base_url,
            mysimkari_base_url,
            output_dir,
            tokens,
            db_config,
            siman_client_id,
            siman_client_secret,
            siman_token_url,
            siman_ba_key,
            siman_base_url,
            siman_concurrency_limit,
        })
    }
}
