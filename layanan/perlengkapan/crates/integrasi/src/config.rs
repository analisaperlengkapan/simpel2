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

    // === SCHEDULER Configuration ===
    /// Enable/disable scheduler
    #[serde(default = "default_scheduler_enabled")]
    pub scheduler_enabled: bool,

    /// MonSAKTI cron schedule (default: "0 2 * * *" - daily at 02:00)
    #[serde(default = "default_monsakti_schedule")]
    pub monsakti_schedule: String,

    /// MySIMKARI cron schedule (default: "0 2 * * *" - daily at 02:00)
    #[serde(default = "default_mysimkari_schedule")]
    pub mysimkari_schedule: String,

    /// SIMAN cron schedule (default: "0 3 * * 0" - Sunday at 03:00)
    #[serde(default = "default_siman_schedule")]
    pub siman_schedule: String,

    /// Scheduler timezone (default: "Asia/Jakarta")
    #[serde(default = "default_scheduler_timezone")]
    pub scheduler_timezone: String,
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

fn default_scheduler_enabled() -> bool {
    true
}

fn default_monsakti_schedule() -> String {
    "0 2 * * *".to_string() // Daily at 02:00
}

fn default_mysimkari_schedule() -> String {
    "0 2 * * *".to_string() // Daily at 02:00
}

fn default_siman_schedule() -> String {
    "0 3 * * 0".to_string() // Sunday at 03:00
}

fn default_scheduler_timezone() -> String {
    "Asia/Jakarta".to_string()
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

        // Define MonSAKTI modules
        let monsakti_modules = vec!["ADM", "ANG", "AST", "BEN", "GLP", "KOM", "PEM", "PER"];

        // Load specific tokens for MonSAKTI
        for module in monsakti_modules {
            let env_var = format!("MONSAKTI_TOKEN_{}", module);
            if let Ok(token) = std::env::var(&env_var) {
                tokens.insert(module.to_string(), token);
            }
        }

        // Load MySIMKARI token separately (MYSIMKARI_TOKEN untuk get-satker)
        if let Ok(t) = std::env::var("MYSIMKARI_TOKEN") {
            tokens.insert("MYSIMKARI".to_string(), t);
        }

        // Load MySIMKARI pegawai token (MYSIMKARI_TOKEN_PEGAWAI untuk pegawai-satker, pegawai/{nip}, pegawai-aktif, pegawai-mutasi)
        if let Ok(t) = std::env::var("MYSIMKARI_TOKEN_PEGAWAI") {
            tokens.insert("MYSIMKARI_PEGAWAI".to_string(), t);
        }

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

        // Scheduler configuration
        let scheduler_enabled = std::env::var("SCHEDULER_ENABLED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(default_scheduler_enabled);

        let monsakti_schedule =
            std::env::var("MONSAKTI_SCHEDULE").unwrap_or_else(|_| default_monsakti_schedule());

        let mysimkari_schedule =
            std::env::var("MYSIMKARI_SCHEDULE").unwrap_or_else(|_| default_mysimkari_schedule());

        let siman_schedule =
            std::env::var("SIMAN_SCHEDULE").unwrap_or_else(|_| default_siman_schedule());

        let scheduler_timezone =
            std::env::var("SCHEDULER_TIMEZONE").unwrap_or_else(|_| default_scheduler_timezone());

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
            scheduler_enabled,
            monsakti_schedule,
            mysimkari_schedule,
            siman_schedule,
            scheduler_timezone,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base: BaseServiceConfig::default(),
            base_url: default_monsakti_base_url(),
            mysimkari_base_url: default_mysimkari_base_url(),
            output_dir: default_output_dir(),
            tokens: HashMap::new(),
            db_config: None,
            siman_client_id: None,
            siman_client_secret: None,
            siman_token_url: default_siman_token_url(),
            siman_ba_key: None,
            siman_base_url: default_siman_base_url(),
            siman_concurrency_limit: default_concurrency_limit(),
            scheduler_enabled: default_scheduler_enabled(),
            monsakti_schedule: default_monsakti_schedule(),
            mysimkari_schedule: default_mysimkari_schedule(),
            siman_schedule: default_siman_schedule(),
            scheduler_timezone: default_scheduler_timezone(),
        }
    }
}
