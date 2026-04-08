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
        // In WASM, derive URLs from browser origin so the app works
        // behind reverse proxies (Istio/Nginx) without config.json
        #[cfg(target_arch = "wasm32")]
        {
            let origin = web_sys::window()
                .and_then(|w| w.location().origin().ok())
                .unwrap_or_else(|| "http://localhost:8080".to_string());
            Self {
                portal_url: origin.clone(),
                authenc_url: format!("{}/api/v1/auth", origin),
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                portal_url: "http://localhost:3000".to_string(),
                authenc_url: "http://localhost:8080/api/v1/auth".to_string(),
            }
        }
    }
}

pub static APP_CONFIG: Lazy<RwLock<AppConfig>> = Lazy::new(|| RwLock::new(AppConfig::default()));

pub async fn load_config() {
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        // Coba load dari config.json
        if let Ok(response) = Request::get("/portal/config.json").send().await
            && response.ok()
            && let Ok(config) = response.json::<AppConfig>().await
            && let Ok(mut c) = APP_CONFIG.write()
        {
            *c = config;
        }
    }
}

pub fn get_config() -> AppConfig {
    APP_CONFIG.read().map(|c| c.clone()).unwrap_or_default()
}
