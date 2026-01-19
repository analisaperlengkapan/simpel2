use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::timeout;
use tracing::{debug, error, warn};

use crate::error::{AuthencError, Result};

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout_seconds: u64,
    /// Request timeout in seconds
    pub request_timeout_seconds: u64,
    /// Keep-alive timeout in seconds
    pub keep_alive_timeout_seconds: u64,
    /// Maximum idle time before connection is closed
    pub max_idle_seconds: u64,
    /// Whether to enable connection reuse
    pub enable_connection_reuse: bool,
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            connection_timeout_seconds: 30,
            request_timeout_seconds: 60,
            keep_alive_timeout_seconds: 90,
            max_idle_seconds: 300,
            enable_connection_reuse: true,
            max_retries: 3,
        }
    }
}

/// Connection pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolStats {
    /// Total number of connections created
    pub total_connections: u64,
    /// Number of active connections
    pub active_connections: usize,
    /// Number of idle connections
    pub idle_connections: usize,
    /// Number of failed connection attempts
    pub failed_connections: u64,
    /// Average connection time in milliseconds
    pub avg_connection_time_ms: f64,
    /// Total requests made
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
}

impl Default for ConnectionPoolStats {
    fn default() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            idle_connections: 0,
            failed_connections: 0,
            avg_connection_time_ms: 0.0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
        }
    }
}

/// HTTP connection pool for external service communication
pub struct HttpConnectionPool {
    client: Client,
    semaphore: Arc<Semaphore>,
    config: ConnectionPoolConfig,
    stats: Arc<Mutex<ConnectionPoolStats>>,
}

impl HttpConnectionPool {
    /// Create a new HTTP connection pool
    pub fn new(config: ConnectionPoolConfig) -> Result<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(config.request_timeout_seconds))
            .connect_timeout(Duration::from_secs(config.connection_timeout_seconds))
            .pool_idle_timeout(Duration::from_secs(config.max_idle_seconds))
            .pool_max_idle_per_host(config.max_connections / 4) // 25% of max connections as idle
            .tcp_keepalive(Duration::from_secs(config.keep_alive_timeout_seconds))
            .http2_keep_alive_interval(Duration::from_secs(30))
            .http2_keep_alive_timeout(Duration::from_secs(10))
            .http2_keep_alive_while_idle(true)
            .build()
            .map_err(|e| AuthencError::internal(&format!("Failed to create HTTP client: {}", e)))?;

        let semaphore = Arc::new(Semaphore::new(config.max_connections));
        let stats = Arc::new(Mutex::new(ConnectionPoolStats::default()));

        Ok(Self {
            client,
            semaphore,
            config,
            stats,
        })
    }

    /// Execute an HTTP GET request with connection pooling
    pub async fn get(&self, url: &str) -> Result<reqwest::Response> {
        self.execute_request(|| self.client.get(url)).await
    }

    /// Execute an HTTP POST request with connection pooling
    pub async fn post(
        &self,
        url: &str,
        body: impl Into<reqwest::Body> + Clone,
    ) -> Result<reqwest::Response> {
        self.execute_request(|| self.client.post(url).body(body.clone()))
            .await
    }

    /// Execute an HTTP PUT request with connection pooling
    pub async fn put(
        &self,
        url: &str,
        body: impl Into<reqwest::Body> + Clone,
    ) -> Result<reqwest::Response> {
        self.execute_request(|| self.client.put(url).body(body.clone()))
            .await
    }

    /// Execute an HTTP DELETE request with connection pooling
    pub async fn delete(&self, url: &str) -> Result<reqwest::Response> {
        self.execute_request(|| self.client.delete(url)).await
    }

    async fn execute_request<F>(&self, request_builder: F) -> Result<reqwest::Response>
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| AuthencError::internal("Failed to acquire connection permit"))?;

        let start_time = Instant::now();
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            let request = request_builder()
                .build()
                .map_err(|e| AuthencError::internal(&format!("Failed to build request: {}", e)))?;

            match timeout(
                Duration::from_secs(self.config.request_timeout_seconds),
                self.client.execute(request),
            )
            .await
            {
                Ok(Ok(response)) => {
                    let elapsed = start_time.elapsed();
                    self.update_stats(true, elapsed).await;

                    debug!(
                        "HTTP request successful after {} attempts, took {}ms",
                        attempt + 1,
                        elapsed.as_millis()
                    );

                    return Ok(response);
                }
                Ok(Err(e)) => {
                    last_error = Some(AuthencError::internal(&format!(
                        "HTTP request failed: {}",
                        e
                    )));
                    warn!("HTTP request attempt {} failed: {}", attempt + 1, e);
                }
                Err(_) => {
                    last_error = Some(AuthencError::internal("HTTP request timed out"));
                    warn!("HTTP request attempt {} timed out", attempt + 1);
                }
            }

            // Wait before retry (exponential backoff)
            if attempt < self.config.max_retries {
                let delay = Duration::from_millis(100 * (2_u64.pow(attempt)));
                tokio::time::sleep(delay).await;
            }
        }

        let elapsed = start_time.elapsed();
        self.update_stats(false, elapsed).await;

        error!(
            "HTTP request failed after {} attempts, took {}ms",
            self.config.max_retries + 1,
            elapsed.as_millis()
        );

        Err(last_error
            .unwrap_or_else(|| AuthencError::internal("HTTP request failed after all retries")))
    }

    /// Update connection pool statistics
    async fn update_stats(&self, success: bool, elapsed: Duration) {
        let mut stats = self.stats.lock().await;

        stats.total_requests += 1;
        if success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }

        // Update average connection time (simple moving average)
        let new_time_ms = elapsed.as_millis() as f64;
        if stats.total_requests == 1 {
            stats.avg_connection_time_ms = new_time_ms;
        } else {
            stats.avg_connection_time_ms =
                (stats.avg_connection_time_ms * (stats.total_requests - 1) as f64 + new_time_ms)
                    / stats.total_requests as f64;
        }

        stats.active_connections = self.config.max_connections - self.semaphore.available_permits();
    }

    /// Get current connection pool statistics
    pub async fn get_stats(&self) -> ConnectionPoolStats {
        let stats = self.stats.lock().await;
        let mut result = stats.clone();
        result.active_connections =
            self.config.max_connections - self.semaphore.available_permits();
        result.idle_connections = self.semaphore.available_permits();
        result
    }

    /// Get the underlying HTTP client (for advanced usage)
    /// Execute a request with retry logic and connection management
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get connection pool configuration
    pub fn config(&self) -> &ConnectionPoolConfig {
        &self.config
    }
}

/// Specialized connection pool for Secreton service communication
pub struct SecretonConnectionPool {
    pool: HttpConnectionPool,
    base_url: String,
    auth_token: Option<String>,
}

impl SecretonConnectionPool {
    /// Create a new Secreton connection pool
    pub fn new(
        base_url: String,
        auth_token: Option<String>,
        config: ConnectionPoolConfig,
    ) -> Result<Self> {
        let pool = HttpConnectionPool::new(config)?;

        Ok(Self {
            pool,
            base_url,
            auth_token,
        })
    }

    /// Execute a GET request to Secreton service
    pub async fn get(&self, path: &str) -> Result<reqwest::Response> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        let mut request_builder = self.pool.client().get(&url);

        if let Some(token) = &self.auth_token {
            request_builder = request_builder.bearer_auth(token);
        }

        let request = request_builder.build().map_err(|e| {
            AuthencError::internal(&format!("Failed to build Secreton request: {}", e))
        })?;

        self.pool
            .client()
            .execute(request)
            .await
            .map_err(|e| AuthencError::internal(&format!("Secreton request failed: {}", e)))
    }

    /// Execute a POST request to Secreton service
    pub async fn post(&self, path: &str, body: impl Serialize) -> Result<reqwest::Response> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        let mut request_builder = self.pool.client().post(&url).json(&body);

        if let Some(token) = &self.auth_token {
            request_builder = request_builder.bearer_auth(token);
        }

        let request = request_builder.build().map_err(|e| {
            AuthencError::internal(&format!("Failed to build Secreton request: {}", e))
        })?;

        self.pool
            .client()
            .execute(request)
            .await
            .map_err(|e| AuthencError::internal(&format!("Secreton request failed: {}", e)))
    }

    /// Get connection pool statistics
    pub async fn get_stats(&self) -> ConnectionPoolStats {
        self.pool.get_stats().await
    }

    /// Update authentication token
    pub fn update_auth_token(&mut self, token: Option<String>) {
        self.auth_token = token;
    }
}

/// Connection pool manager for managing multiple connection pools
pub struct ConnectionPoolManager {
    secreton_pool: Option<SecretonConnectionPool>,
    default_pool: HttpConnectionPool,
}

impl ConnectionPoolManager {
    /// Create a new connection pool manager
    pub fn new(config: ConnectionPoolConfig) -> Result<Self> {
        let default_pool = HttpConnectionPool::new(config)?;

        Ok(Self {
            secreton_pool: None,
            default_pool,
        })
    }

    /// Configure Secreton connection pool
    pub fn configure_secreton_pool(
        &mut self,
        base_url: String,
        auth_token: Option<String>,
        config: ConnectionPoolConfig,
    ) -> Result<()> {
        self.secreton_pool = Some(SecretonConnectionPool::new(base_url, auth_token, config)?);
        Ok(())
    }

    /// Get Secreton connection pool
    pub fn secreton_pool(&self) -> Option<&SecretonConnectionPool> {
        self.secreton_pool.as_ref()
    }

    /// Get mutable Secreton connection pool
    pub fn secreton_pool_mut(&mut self) -> Option<&mut SecretonConnectionPool> {
        self.secreton_pool.as_mut()
    }

    /// Get default HTTP connection pool
    pub fn default_pool(&self) -> &HttpConnectionPool {
        &self.default_pool
    }

    /// Get combined statistics from all pools
    pub async fn get_combined_stats(&self) -> ConnectionPoolManagerStats {
        let default_stats = self.default_pool.get_stats().await;
        let secreton_stats = if let Some(pool) = &self.secreton_pool {
            Some(pool.get_stats().await)
        } else {
            None
        };

        ConnectionPoolManagerStats {
            default_pool: default_stats,
            secreton_pool: secreton_stats,
        }
    }
}

/// Combined statistics for all connection pools
#[derive(Debug, Clone)]
pub struct ConnectionPoolManagerStats {
    /// Statistics for the default PostgreSQL connection pool
    pub default_pool: ConnectionPoolStats,
    /// Statistics for the Secreton connection pool (if available)
    pub secreton_pool: Option<ConnectionPoolStats>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool_config() {
        let config = ConnectionPoolConfig::default();
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.connection_timeout_seconds, 30);
        assert!(config.enable_connection_reuse);
    }

    #[tokio::test]
    async fn test_http_connection_pool_creation() {
        let config = ConnectionPoolConfig::default();
        let pool = HttpConnectionPool::new(config);
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_connection_pool_stats() {
        let config = ConnectionPoolConfig::default();
        let pool = HttpConnectionPool::new(config).unwrap();

        let stats = pool.get_stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
    }

    #[tokio::test]
    async fn test_secreton_connection_pool() {
        let config = ConnectionPoolConfig::default();
        let pool = SecretonConnectionPool::new(
            "https://secreton.example.com".to_string(),
            Some("test_token".to_string()),
            config,
        );
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_connection_pool_manager() {
        let config = ConnectionPoolConfig::default();
        let mut manager = ConnectionPoolManager::new(config.clone()).unwrap();

        assert!(manager.secreton_pool().is_none());

        manager
            .configure_secreton_pool(
                "https://secreton.example.com".to_string(),
                Some("test_token".to_string()),
                config,
            )
            .unwrap();

        assert!(manager.secreton_pool().is_some());
    }
}
