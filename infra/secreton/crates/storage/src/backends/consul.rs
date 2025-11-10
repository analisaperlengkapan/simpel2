//! Consul Storage Backend
//!
//! Implements HashiCorp Vault-compatible Consul storage backend for high availability
//! and service discovery. Consul provides:
//!
//! - **High Availability**: Multi-node clustering with automatic leader election
//! - **Service Discovery**: Native service mesh integration
//! - **Distributed KV**: Strongly consistent key-value store
//! - **ACL Support**: Fine-grained access control
//! - **Health Checks**: Automatic node health monitoring
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
//! │  Secreton 1  │────▶│   Consul 1   │◀────│  Secreton 2  │
//! └──────────────┘     └──────┬───────┘     └──────────────┘
//!                             │
//!                      ┌──────┴───────┐
//!                      │   Consul 2   │
//!                      └──────┬───────┘
//!                             │
//!                      ┌──────┴───────┐
//!                      │   Consul 3   │
//!                      └──────────────┘
//! ```
//!
//! # Configuration
//!
//! ```toml
//! [storage]
//! backend = "consul"
//! address = "127.0.0.1:8500"
//! path = "secreton/"
//! scheme = "https"
//! token = "${CONSUL_TOKEN}"
//! tls_skip_verify = false
//! ```

use crate::{BackendMetrics, KvBackend, StorageError, StorageResult};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Consul storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsulConfig {
    /// Consul API address (e.g., "127.0.0.1:8500")
    pub address: String,

    /// Key prefix for all Secreton data
    pub path: String,

    /// HTTP scheme: "http" or "https"
    pub scheme: String,

    /// Consul ACL token for authentication
    pub token: Option<String>,

    /// TLS certificate verification
    pub tls_skip_verify: bool,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Maximum number of retries for failed requests
    pub max_retries: u32,

    /// Enable Consul health checks
    pub health_check: bool,

    /// Session TTL for distributed locks (seconds)
    pub session_ttl_secs: u64,
}

impl Default for ConsulConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:8500".to_string(),
            path: "secreton/".to_string(),
            scheme: "http".to_string(),
            token: None,
            tls_skip_verify: false,
            timeout_secs: 10,
            max_retries: 3,
            health_check: true,
            session_ttl_secs: 15,
        }
    }
}

/// Consul KV response structure
#[derive(Debug, Deserialize)]
struct ConsulKvEntry {
    #[serde(rename = "Key")]
    key: String,

    #[serde(rename = "Value")]
    value: Option<String>,

    #[serde(rename = "Flags")]
    flags: u64,

    #[serde(rename = "CreateIndex")]
    create_index: u64,

    #[serde(rename = "ModifyIndex")]
    modify_index: u64,
}

/// Consul storage backend implementation
pub struct ConsulBackend {
    config: ConsulConfig,
    client: Client,
    base_url: String,
    metrics: Arc<RwLock<BackendMetrics>>,
}

impl ConsulBackend {
    /// Create a new Consul storage backend
    pub async fn new(config: ConsulConfig) -> StorageResult<Self> {
        let mut client_builder = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs));

        if config.tls_skip_verify {
            warn!("TLS certificate verification is disabled - not recommended for production");
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        let client = client_builder.build().map_err(|e| StorageError::ConnectionError {
            backend: "consul".to_string(),
            message: format!("Failed to create HTTP client: {}", e),
        })?;

        let base_url = format!("{}://{}/v1/kv/{}", config.scheme, config.address, config.path);

        let backend = Self {
            config: config.clone(),
            client,
            base_url,
            metrics: Arc::new(RwLock::new(BackendMetrics::default())),
        };

        // Verify connectivity
        backend.health_check().await?;

        info!("Consul storage backend initialized at {}", config.address);
        Ok(backend)
    }

    /// Construct full Consul key path
    fn key_path(&self, key: &str) -> String {
        format!("{}{}", self.base_url, key.trim_start_matches('/'))
    }

    /// Perform health check on Consul cluster
    async fn health_check(&self) -> StorageResult<()> {
        let health_url = format!("{}://{}/v1/health/service/consul",
            self.config.scheme, self.config.address);

        let response = self.client
            .get(&health_url)
            .send()
            .await
            .map_err(|e| StorageError::ConnectionError {
                backend: "consul".to_string(),
                message: format!("Health check failed: {}", e),
            })?;

        if response.status().is_success() {
            debug!("Consul health check passed");
            Ok(())
        } else {
            Err(StorageError::ConnectionError {
                backend: "consul".to_string(),
                message: format!("Consul unhealthy: {}", response.status()),
            })
        }
    }

    /// Create Consul session for distributed locking
    async fn create_session(&self) -> StorageResult<String> {
        let session_url = format!("{}://{}/v1/session/create",
            self.config.scheme, self.config.address);

        let session_payload = serde_json::json!({
            "Name": "secreton-lock",
            "TTL": format!("{}s", self.config.session_ttl_secs),
            "Behavior": "delete"
        });

        let mut request = self.client.put(&session_url).json(&session_payload);
        if let Some(ref token) = self.config.token {
            request = request.header("X-Consul-Token", token);
        }

        let response = request.send().await.map_err(|e| StorageError::BackendError {
            backend: "consul".to_string(),
            message: format!("Failed to create session: {}", e),
        })?;

        #[derive(Deserialize)]
        struct SessionResponse {
            #[serde(rename = "ID")]
            id: String,
        }

        let session: SessionResponse = response.json().await.map_err(|e| {
            StorageError::SerializationError {
                message: format!("Failed to parse session response: {}", e),
            }
        })?;

        Ok(session.id)
    }
}

#[async_trait]
impl KvBackend for ConsulBackend {
    async fn get(&self, key: &str) -> StorageResult<Option<Vec<u8>>> {
        let url = self.key_path(key);

        let mut request = self.client.get(&url);
        if let Some(ref token) = self.config.token {
            request = request.header("X-Consul-Token", token);
        }

        let response = request.send().await.map_err(|e| {
            StorageError::ConnectionError {
                backend: "consul".to_string(),
                message: format!("GET request failed: {}", e),
            }
        })?;

        match response.status() {
            StatusCode::OK => {
                let entries: Vec<ConsulKvEntry> = response.json().await.map_err(|e| {
                    StorageError::SerializationError {
                        message: format!("Failed to parse response: {}", e),
                    }
                })?;

                if let Some(entry) = entries.first() {
                    if let Some(ref value_b64) = entry.value {
                        let decoded = general_purpose::STANDARD.decode(value_b64).map_err(|e| {
                            StorageError::SerializationError {
                                message: format!("Failed to decode base64: {}", e),
                            }
                        })?;

                        let mut metrics = self.metrics.write().await;
                        metrics.reads += 1;
                        metrics.bytes_read += decoded.len() as u64;

                        Ok(Some(decoded))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            StatusCode::NOT_FOUND => Ok(None),
            status => Err(StorageError::BackendError {
                backend: "consul".to_string(),
                message: format!("Unexpected status: {}", status),
            }),
        }
    }

    async fn put(&self, key: &str, value: &[u8]) -> StorageResult<()> {
        let url = self.key_path(key);

        let mut request = self.client.put(&url).body(value.to_vec());
        if let Some(ref token) = self.config.token {
            request = request.header("X-Consul-Token", token);
        }

        let response = request.send().await.map_err(|e| {
            StorageError::ConnectionError {
                backend: "consul".to_string(),
                message: format!("PUT request failed: {}", e),
            }
        })?;

        if response.status().is_success() {
            let mut metrics = self.metrics.write().await;
            metrics.writes += 1;
            metrics.bytes_written += value.len() as u64;
            Ok(())
        } else {
            Err(StorageError::BackendError {
                backend: "consul".to_string(),
                message: format!("PUT failed with status: {}", response.status()),
            })
        }
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let url = self.key_path(key);

        let mut request = self.client.delete(&url);
        if let Some(ref token) = self.config.token {
            request = request.header("X-Consul-Token", token);
        }

        let response = request.send().await.map_err(|e| {
            StorageError::ConnectionError {
                backend: "consul".to_string(),
                message: format!("DELETE request failed: {}", e),
            }
        })?;

        if response.status().is_success() {
            let mut metrics = self.metrics.write().await;
            metrics.deletes += 1;
            Ok(())
        } else {
            Err(StorageError::BackendError {
                backend: "consul".to_string(),
                message: format!("DELETE failed with status: {}", response.status()),
            })
        }
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let url = format!("{}?keys&separator=/", self.key_path(prefix));

        let mut request = self.client.get(&url);
        if let Some(ref token) = self.config.token {
            request = request.header("X-Consul-Token", token);
        }

        let response = request.send().await.map_err(|e| {
            StorageError::ConnectionError {
                backend: "consul".to_string(),
                message: format!("LIST request failed: {}", e),
            }
        })?;

        match response.status() {
            StatusCode::OK => {
                let keys: Vec<String> = response.json().await.map_err(|e| {
                    StorageError::SerializationError {
                        message: format!("Failed to parse keys: {}", e),
                    }
                })?;

                // Strip prefix from keys
                let stripped_keys: Vec<String> = keys
                    .into_iter()
                    .map(|k| k.trim_start_matches(&self.config.path).to_string())
                    .collect();

                Ok(stripped_keys)
            }
            StatusCode::NOT_FOUND => Ok(vec![]),
            status => Err(StorageError::BackendError {
                backend: "consul".to_string(),
                message: format!("LIST failed with status: {}", status),
            }),
        }
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        Ok(self.get(key).await?.is_some())
    }

    async fn metrics(&self) -> StorageResult<BackendMetrics> {
        Ok(self.metrics.read().await.clone())
    }

    async fn health_check(&self) -> StorageResult<crate::HealthStatus> {
        use std::time::Instant;
        
        let start = Instant::now();
        
        // Check Consul health endpoint
        let health_url = format!("{}/v1/agent/self", self.config.address);
        let is_healthy = match self.client.get(&health_url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        };
        
        let response_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        Ok(crate::HealthStatus {
            is_healthy,
            response_time_ms,
            connections_active: 1, // HTTP connection pool
            connections_idle: 0,
            last_error: if is_healthy { None } else { 
                Some("Consul agent not reachable".to_string()) 
            },
            uptime_seconds: 0, // Not tracked
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running Consul instance
    async fn test_consul_backend() {
        let config = ConsulConfig::default();
        let backend = ConsulBackend::new(config).await.unwrap();

        // Test put
        backend.put("test/key", b"test-value").await.unwrap();

        // Test get
        let value = backend.get("test/key").await.unwrap();
        assert_eq!(value.unwrap(), b"test-value");

        // Test delete
        backend.delete("test/key").await.unwrap();
        let value = backend.get("test/key").await.unwrap();
        assert!(value.is_none());
    }
}
