use chrono::Duration;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub uptime_seconds: u64,
    pub total_users: u64,
    pub active_sessions: u64,
    pub total_secrets: u64,
    pub total_keys: u64,
    pub storage_usage_bytes: u64,
    pub cache_hit_rate: f64,
    pub requests_per_minute: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretData {
    pub path: String,
    pub data: std::collections::HashMap<String, String>,
    pub metadata: SecretMetadata,
    pub version: u32,
    pub created_at: String, // Simplified for JSON
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecretMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
    pub classification: Option<String>,
}

pub async fn fetch_system_stats() -> Result<SystemStats, String> {
    // In a real environment, we'd use the token from context
    // For this plan, we assume the browser cookie handles auth or we are in a dev mode
    let resp = Request::get("/api/v1/sys/stats")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Failed to fetch stats: {}", resp.status()));
    }

    resp.json::<SystemStats>().await.map_err(|e| e.to_string())
}

pub async fn list_secrets(prefix: Option<&str>) -> Result<Vec<String>, String> {
    let url = if let Some(p) = prefix {
        format!("/api/v1/secrets?list=true&prefix={}", p)
    } else {
        "/api/v1/secrets?list=true".to_string()
    };

    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Failed to list secrets: {}", resp.status()));
    }

    // The API returns keys as a list of strings
    resp.json::<Vec<String>>().await.map_err(|e| e.to_string())
}

pub async fn get_secret(path: &str) -> Result<SecretData, String> {
    let resp = Request::get(&format!("/api/v1/secrets/{}", path))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Failed to get secret: {}", resp.status()));
    }

    resp.json::<SecretData>().await.map_err(|e| e.to_string())
}

pub async fn create_secret(path: &str, data: std::collections::HashMap<String, String>) -> Result<SecretData, String> {
    let resp = Request::post(&format!("/api/v1/secrets/{}", path))
        .json(&data)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Failed to create secret: {}", resp.status()));
    }

    resp.json::<SecretData>().await.map_err(|e| e.to_string())
}
