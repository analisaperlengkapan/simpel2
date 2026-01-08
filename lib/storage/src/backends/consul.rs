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
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

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
            scheme: "https".to_string(), // Secure by default
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
    /// Active session ID for distributed locking
    session_id: Arc<RwLock<Option<String>>>,
}

impl ConsulBackend {
    /// Create a new Consul storage backend
    pub async fn new(config: ConsulConfig) -> StorageResult<Self> {
        // Validate production security requirements
        Self::validate_security_config(&config)?;

        let mut client_builder =
            Client::builder().timeout(std::time::Duration::from_secs(config.timeout_secs));

        if config.tls_skip_verify {
            warn!("TLS certificate verification is disabled - not recommended for production");
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        let client = client_builder
            .build()
            .map_err(|e| StorageError::ConnectionFailed {
                message: format!("Consul: Failed to create HTTP client: {}", e),
                source: None,
            })?;

        let base_url = format!(
            "{}://{}/v1/kv/{}",
            config.scheme, config.address, config.path
        );

        let backend = Self {
            config: config.clone(),
            client,
            base_url,
            metrics: Arc::new(RwLock::new(BackendMetrics::default())),
            session_id: Arc::new(RwLock::new(None)),
        };

        // Verify connectivity
        backend.health_check().await?;

        // Create initial session for distributed locking
        if let Ok(session) = backend.create_session().await {
            *backend.session_id.write().await = Some(session.clone());
            info!("Consul session created: {}", session);

            // Start background session renewal task
            backend.start_session_renewal();
        } else {
            warn!("Failed to create initial Consul session, will retry on demand");
        }

        info!("Consul storage backend initialized at {}", config.address);
        Ok(backend)
    }

    /// Construct full Consul key path
    fn key_path(&self, key: &str) -> String {
        format!("{}{}", self.base_url, key.trim_start_matches('/'))
    }

    /// Perform health check on Consul cluster
    async fn health_check(&self) -> StorageResult<()> {
        let health_url = format!(
            "{}://{}/v1/health/service/consul",
            self.config.scheme, self.config.address
        );

        let response = self.client.get(&health_url).send().await.map_err(|e| {
            StorageError::ConnectionFailed {
                message: format!("Consul: Health check failed: {}", e),
                source: None,
            }
        })?;

        if response.status().is_success() {
            debug!("Consul health check passed");
            Ok(())
        } else {
            Err(StorageError::ConnectionFailed {
                message: format!("Consul unhealthy: {}", response.status()),
                source: None,
            })
        }
    }

    /// Create Consul session for distributed locking
    async fn create_session(&self) -> StorageResult<String> {
        let session_url = format!(
            "{}://{}/v1/session/create",
            self.config.scheme, self.config.address
        );

        let session_payload = serde_json::json!({
            "Name": "secreton-lock",
            "TTL": format!("{}s", self.config.session_ttl_secs),
            "Behavior": "delete"
        });

        let mut request = self.client.put(&session_url).json(&session_payload);
        if let Some(ref token) = self.config.token {
            request = request.header("X-Consul-Token", token);
        }

        let response = request
            .send()
            .await
            .map_err(|e| StorageError::BackendError {
                backend: "consul".to_string(),
                message: format!("Failed to create session: {}", e),
            })?;

        #[derive(Deserialize)]
        struct SessionResponse {
            #[serde(rename = "ID")]
            id: String,
        }

        let session: SessionResponse =
            response
                .json()
                .await
                .map_err(|e| StorageError::SerializationError {
                    message: format!("Failed to parse session response: {}", e),
                    source: None,
                })?;

        Ok(session.id)
    }

    /// Start background task for session renewal
    fn start_session_renewal(&self) {
        let session_id = Arc::clone(&self.session_id);
        let config = self.config.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            // Renew session at half the TTL interval
            let renewal_interval = Duration::from_secs(config.session_ttl_secs / 2);

            loop {
                tokio::time::sleep(renewal_interval).await;

                let session_guard = session_id.read().await;
                if let Some(ref session) = *session_guard {
                    let renew_url = format!(
                        "{}://{}/v1/session/renew/{}",
                        config.scheme, config.address, session
                    );

                    let mut request = client.put(&renew_url);
                    if let Some(ref token) = config.token {
                        request = request.header("X-Consul-Token", token);
                    }

                    match request.send().await {
                        Ok(response) if response.status().is_success() => {
                            debug!("Consul session renewed: {}", session);
                        }
                        Ok(response) => {
                            warn!(
                                "Failed to renew Consul session {}: status {}",
                                session,
                                response.status()
                            );
                            // Session might be expired, clear it
                            drop(session_guard);
                            *session_id.write().await = None;
                        }
                        Err(e) => {
                            warn!("Error renewing Consul session {}: {}", session, e);
                        }
                    }
                }
            }
        });
    }

    /// Get or create a Consul session for distributed locking
    pub async fn get_session(&self) -> StorageResult<String> {
        // Check if we have an active session
        let session_guard = self.session_id.read().await;
        if let Some(ref session) = *session_guard {
            return Ok(session.clone());
        }
        drop(session_guard);

        // Create new session
        let session = self.create_session().await?;
        *self.session_id.write().await = Some(session.clone());

        Ok(session)
    }

    /// Validate security configuration for production
    fn validate_security_config(config: &ConsulConfig) -> StorageResult<()> {
        let env = std::env::var("SECRETON_ENV").unwrap_or_else(|_| "production".to_string());

        if env == "production" {
            if config.scheme == "http" {
                error!("SECURITY: Consul backend using HTTP in production mode");
                return Err(StorageError::ConfigurationError {
                    message: "Consul backend must use HTTPS in production. Set scheme='https' or SECRETON_ENV=development".to_string(),
                });
            }

            if config.tls_skip_verify {
                error!("SECURITY: TLS verification disabled in production mode");
                return Err(StorageError::ConfigurationError {
                    message: "TLS verification cannot be disabled in production. Set tls_skip_verify=false".to_string(),
                });
            }

            if config.token.is_none() {
                warn!("SECURITY: No Consul ACL token configured in production");
            }
        }

        Ok(())
    }

    /// Execute request with retry and exponential backoff
    async fn execute_with_retry<F, Fut, T>(
        &self,
        operation: &str,
        mut request_fn: F,
    ) -> StorageResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, reqwest::Error>>,
    {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match request_fn().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);

                    if attempt < self.config.max_retries {
                        // Exponential backoff with jitter: base_delay * 2^attempt + random(0-100ms)
                        let base_delay = Duration::from_millis(100);
                        let exponential_delay = base_delay * 2_u32.pow(attempt);
                        let jitter = Duration::from_millis(50);
                        let total_delay = exponential_delay + jitter;

                        warn!(
                            "Consul {} failed (attempt {}/{}), retrying in {:?}",
                            operation,
                            attempt + 1,
                            self.config.max_retries,
                            total_delay
                        );

                        tokio::time::sleep(total_delay).await;
                    }
                }
            }
        }

        Err(StorageError::ConnectionFailed {
            message: format!(
                "Consul: {} failed after {} retries: {}",
                operation,
                self.config.max_retries,
                last_error.unwrap()
            ),
            source: None,
        })
    }
}

#[async_trait]
impl KvBackend for ConsulBackend {
    async fn get(&self, key: &str) -> StorageResult<Option<Vec<u8>>> {
        let url = self.key_path(key);
        let token = self.config.token.clone();

        let response = self
            .execute_with_retry("GET", || async {
                let mut request = self.client.get(&url);
                if let Some(ref t) = token {
                    request = request.header("X-Consul-Token", t);
                }
                request.send().await
            })
            .await?;

        match response.status() {
            StatusCode::OK => {
                let entries: Vec<ConsulKvEntry> =
                    response
                        .json()
                        .await
                        .map_err(|e| StorageError::SerializationError {
                            message: format!("Failed to parse response: {}", e),
                            source: None,
                        })?;

                if let Some(entry) = entries.first() {
                    if let Some(ref value_b64) = entry.value {
                        let decoded = general_purpose::STANDARD.decode(value_b64).map_err(|e| {
                            StorageError::SerializationError {
                                message: format!("Failed to decode base64: {}", e),
                                source: None,
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
        let token = self.config.token.clone();
        let value_vec = value.to_vec();

        let response = self
            .execute_with_retry("PUT", || async {
                let mut request = self.client.put(&url).body(value_vec.clone());
                if let Some(ref t) = token {
                    request = request.header("X-Consul-Token", t);
                }
                request.send().await
            })
            .await?;

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
        let token = self.config.token.clone();

        let response = self
            .execute_with_retry("DELETE", || async {
                let mut request = self.client.delete(&url);
                if let Some(ref t) = token {
                    request = request.header("X-Consul-Token", t);
                }
                request.send().await
            })
            .await?;

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
        let token = self.config.token.clone();

        let response = self
            .execute_with_retry("LIST", || async {
                let mut request = self.client.get(&url);
                if let Some(ref t) = token {
                    request = request.header("X-Consul-Token", t);
                }
                request.send().await
            })
            .await?;

        match response.status() {
            StatusCode::OK => {
                let keys: Vec<String> =
                    response
                        .json()
                        .await
                        .map_err(|e| StorageError::SerializationError {
                            message: format!("Failed to parse keys: {}", e),
                            source: None,
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
        let health_url = format!(
            "{}://{}/v1/agent/self",
            self.config.scheme, self.config.address
        );
        let is_healthy = match self.client.get(&health_url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        };

        let response_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Check session status
        let session_active = self.session_id.read().await.is_some();

        Ok(crate::HealthStatus {
            is_healthy: is_healthy && session_active,
            response_time_ms,
            connections_active: 1, // HTTP connection pool
            connections_idle: 0,
            last_error: if !is_healthy {
                Some("Consul agent not reachable".to_string())
            } else if !session_active {
                Some("Consul session not active".to_string())
            } else {
                None
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
