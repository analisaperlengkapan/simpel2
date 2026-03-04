use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub portal_url: String,
    pub authenc_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            portal_url: "http://localhost:3000".to_string(),
            authenc_url: "http://localhost:8080".to_string(),
        }
    }
}

pub static APP_CONFIG: Lazy<RwLock<AppConfig>> = Lazy::new(|| RwLock::new(AppConfig::default()));

pub async fn load_config() {
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        // Coba load dari config.json
        if let Ok(response) = Request::get("/portal/config.json").send().await {
            if response.ok() {
                if let Ok(config) = response.json::<AppConfig>().await {
                    if let Ok(mut c) = APP_CONFIG.write() {
                        *c = config;
                        return;
                    }
                }
            }
        }
    }
}

pub fn get_config() -> AppConfig {
    APP_CONFIG.read().map(|c| c.clone()).unwrap_or_default()
}
