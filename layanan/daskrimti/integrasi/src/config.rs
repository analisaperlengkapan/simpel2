use crate::error::MonsaktiError;
use std::collections::HashMap;

/// Konfigurasi untuk klien MonSAKTI
#[derive(Debug, Clone)]
pub struct Config {
    /// URL dasar API MonSAKTI
    pub base_url: String,
    /// URL dasar API MySIMKARI
    pub mysimkari_base_url: String,
    /// URL dasar API SIMAN (Gateway Kemenkeu)
    pub siman_base_url: String,
    /// URL endpoint SSO Kemenkeu untuk OAuth2 token
    pub siman_token_url: String,
    /// Client ID untuk SIMAN OAuth2
    pub siman_client_id: Option<String>,
    /// Client Secret untuk SIMAN OAuth2
    pub siman_client_secret: Option<String>,
    /// BA_KEY (Kode Satker) untuk SIMAN API
    pub siman_ba_key: Option<String>,
    /// Token autentikasi untuk setiap modul
    pub tokens: HashMap<String, String>,
    /// Direktori output untuk file
    pub output_dir: String,
    /// Konfigurasi database opsional
    pub db_config: Option<String>,
}

impl Config {
    /// Membuat konfigurasi dari environment variables
    pub fn from_env() -> Result<Self, MonsaktiError> {
        dotenvy::dotenv().ok();

        let base_url = std::env::var("MONSAKTI_BASE_URL").unwrap_or_else(|_| {
            "https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice".to_string()
        });

        let mysimkari_base_url = std::env::var("MYSIMKARI_BASE_URL")
            .unwrap_or_else(|_| "https://mysimkari.kejaksaan.go.id/api/anbut".to_string());

        // SIMAN API Configuration
        let siman_base_url = std::env::var("SIMAN_BASE_URL")
            .unwrap_or_else(|_| "https://api-gw.kemenkeu.go.id".to_string());

        let siman_token_url = std::env::var("SIMAN_TOKEN_URL")
            .unwrap_or_else(|_| "https://sso.kemenkeu.go.id/connect/token".to_string());

        let siman_client_id = std::env::var("SIMAN_CLIENT_ID").ok();
        let siman_client_secret = std::env::var("SIMAN_CLIENT_SECRET").ok();
        let siman_ba_key = std::env::var("SIMAN_BA_KEY").ok();

        let output_dir = std::env::var("OUTPUT_DIR").unwrap_or_else(|_| "./data".to_string());
        let db_config = std::env::var("DATABASE_URL").ok();

        let mut tokens = HashMap::new();
        for module in &[
            "ADM",
            "ANG",
            "PEM",
            "BEN",
            "KOM",
            "AST",
            "PER",
            "GLP",
            "MYSIMKARI",
        ] {
            let env_var = if module == &"MYSIMKARI" {
                "MYSIMKARI_TOKEN".to_string()
            } else {
                format!("MONSAKTI_TOKEN_{}", module)
            };
            if let Ok(token) = std::env::var(&env_var) {
                // Trim whitespace to avoid issues
                tokens.insert(module.to_string(), token.trim().to_string());
            }
        }

        Ok(Config {
            base_url,
            mysimkari_base_url,
            siman_base_url,
            siman_token_url,
            siman_client_id,
            siman_client_secret,
            siman_ba_key,
            tokens,
            output_dir,
            db_config,
        })
    }
}
