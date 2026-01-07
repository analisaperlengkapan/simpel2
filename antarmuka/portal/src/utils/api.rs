use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("API error: {0}")]
    Api(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SystemMetrics {
    pub uptime: u64,
    pub memory_usage: MemoryMetrics,
    pub cpu_usage: CpuMetrics,
    pub disk_usage: DiskMetrics,
    pub network: NetworkMetrics,
    pub vault: VaultMetrics,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MemoryMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub cached: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CpuMetrics {
    pub cores: u32,
    pub usage_percent: f64,
    pub load_average: [f64; 3],
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DiskMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VaultMetrics {
    pub total_secrets: u64,
    pub total_keys: u64,
    pub total_policies: u64,
    pub active_sessions: u64,
    pub operations_per_second: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

pub async fn get_system_metrics() -> Result<SystemMetrics, ApiError> {
    // In a real app, this URL would come from config
    let url = "/api/v1/admin/metrics";

    // For now, if we are in SSR or test mode without a real backend, we might want to return mocks
    // But the goal is integration.

    // Using gloo_net for fetch
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        let response = Request::get(url)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if !response.ok() {
            return Err(ApiError::Api(format!("HTTP Error: {}", response.status())));
        }

        let api_response: ApiResponse<SystemMetrics> = response
            .json()
            .await
            .map_err(|e| ApiError::Serialization(e.to_string()))?;

        if let Some(data) = api_response.data {
            Ok(data)
        } else {
            Err(ApiError::Api(api_response.error.unwrap_or_else(|| "Unknown API error".to_string())))
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Mock data for SSR/Server side rendering to avoid network calls during hydration mismatch
        Ok(SystemMetrics::default())
    }
}

// Secrets API

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretListItem {
    pub path: String,
    pub version: u32,
    pub created_at: String, // Simplified for frontend display
    pub updated_at: String,
}

pub async fn list_secrets(prefix: Option<&str>) -> Result<Vec<SecretListItem>, ApiError> {
    let mut url = "/api/v1/secrets".to_string();
    if let Some(p) = prefix {
        url.push_str("?filter=");
        url.push_str(p);
    }

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if !response.ok() {
            return Err(ApiError::Api(format!("HTTP Error: {}", response.status())));
        }

        let api_response: ApiResponse<Vec<SecretListItem>> = response
            .json()
            .await
            .map_err(|e| ApiError::Serialization(e.to_string()))?;

        if let Some(data) = api_response.data {
            Ok(data)
        } else {
            Err(ApiError::Api(api_response.error.unwrap_or_else(|| "Unknown API error".to_string())))
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(vec![])
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSecretRequest {
    pub data: std::collections::HashMap<String, String>,
    pub metadata: Option<SecretMetadata>,
    pub ttl: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
    pub classification: Option<String>,
}

pub async fn create_secret(path: &str, data: std::collections::HashMap<String, String>) -> Result<(), ApiError> {
    let url = format!("/api/v1/data/{}", path);

    let request_body = CreateSecretRequest {
        data,
        metadata: Some(SecretMetadata {
            description: Some("Created via Portal".to_string()),
            tags: vec!["portal".to_string()],
            owner: None, // Will default to user
            classification: Some("confidential".to_string()),
        }),
        ttl: None,
    };

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        let response = Request::post(&url)
            .json(&request_body)
            .map_err(|e| ApiError::Serialization(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if !response.ok() {
            return Err(ApiError::Api(format!("HTTP Error: {}", response.status())));
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(())
    }
}
