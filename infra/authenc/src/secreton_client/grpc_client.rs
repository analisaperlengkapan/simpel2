//! gRPC client for Secreton service
//!
//! This module provides a gRPC-based implementation of the SecretonClientTrait
//! for improved performance and better context propagation compared to REST.

use super::{RotationResult, Secret, SecretonClientTrait, SecretonError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::{Request, Status};
use tracing::warn;

// Generated proto code - must be at module root for proper namespace resolution
/// Modul `common`.
pub mod common {
/// Modul `v1`.
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
/// Modul `secreton`.

/// Modul `v1`.
pub mod secreton {
    pub mod v1 {
        tonic::include_proto!("secreton.v1");
    }
}

use secreton::v1::{
    DeleteSecretRequest, GetSecretRequest, ListSecretsRequest, StoreSecretRequest,
    secreton_service_client::SecretonServiceClient,
};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for handling Secreton service failures
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    failure_threshold: u32,
    timeout: Duration,
    success_threshold: u32,
    success_count: Arc<RwLock<u32>>,
}
/// Fungsi `new(`.

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            success_threshold: 3,
            success_count: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn can_execute(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                drop(state);
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() >= self.timeout {
                        *self.state.write().await = CircuitBreakerState::HalfOpen;
                        *self.success_count.write().await = 0;
                        return true;
                    }
                }
                false
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    pub async fn record_success(&self) {
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::Closed => {
                *self.failure_count.write().await = 0;
            }
            CircuitBreakerState::HalfOpen => {
                drop(state);
                let mut success_count = self.success_count.write().await;
                *success_count += 1;
                if *success_count >= self.success_threshold {
                    *self.state.write().await = CircuitBreakerState::Closed;
                    *self.failure_count.write().await = 0;
                }
            }
            CircuitBreakerState::Open => {
                *self.state.write().await = CircuitBreakerState::Closed;
                *self.failure_count.write().await = 0;
            }
        }
    }

    pub async fn record_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;
        *self.last_failure_time.write().await = Some(Instant::now());

        if *failure_count >= self.failure_threshold {
            *self.state.write().await = CircuitBreakerState::Open;
        }
    }
}

/// gRPC-based Secreton client
pub struct GrpcSecretonClient {
    client: SecretonServiceClient<Channel>,
    token: String,
    circuit_breaker: CircuitBreaker,
    max_retries: u32,
}

impl GrpcSecretonClient {
    /// Create a new gRPC Secreton client
    ///
    /// # Arguments
    /// * `endpoint` - Secreton gRPC endpoint (e.g., "https://secreton.local:50051")
    /// * `token` - Authentication token
    pub async fn new(endpoint: String, token: String) -> Result<Self, SecretonError> {
        let channel = Channel::from_shared(endpoint.clone())
            .map_err(|e| SecretonError::Other(format!("Invalid endpoint: {}", e)))?
            .connect()
            .await
            .map_err(|e| SecretonError::Unavailable(format!("Failed to connect: {}", e)))?;

        let client = SecretonServiceClient::new(channel);

        Ok(Self {
            client,
            token,
            circuit_breaker: CircuitBreaker::new(),
            max_retries: 3,
        })
    }

    /// Create a new gRPC Secreton client with TLS
    pub async fn new_with_tls(
        endpoint: String,
        token: String,
        tls_config: ClientTlsConfig,
    ) -> Result<Self, SecretonError> {
        let channel = Channel::from_shared(endpoint.clone())
            .map_err(|e| SecretonError::Other(format!("Invalid endpoint: {}", e)))?
            .tls_config(tls_config)
            .map_err(|e| SecretonError::Other(format!("TLS config error: {}", e)))?
            .connect()
            .await
            .map_err(|e| SecretonError::Unavailable(format!("Failed to connect: {}", e)))?;

        let client = SecretonServiceClient::new(channel);

        Ok(Self {
            client,
            token,
            circuit_breaker: CircuitBreaker::new(),
            max_retries: 3,
        })
    }

    /// Execute a request with circuit breaker and retry logic
    async fn execute_with_circuit_breaker<F, Fut, T>(
        &self,
        operation: F,
    ) -> Result<T, SecretonError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, SecretonError>>,
    {
        if !self.circuit_breaker.can_execute().await {
            return Err(SecretonError::Unavailable(
                "Circuit breaker is open".to_string(),
            ));
        }

        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match operation().await {
                Ok(result) => {
                    self.circuit_breaker.record_success().await;
                    return Ok(result);
                }
                Err(err) => {
                    last_error = Some(err.clone());

                    // Don't retry on auth errors
                    if matches!(
                        err,
                        SecretonError::AuthenticationFailed(_) | SecretonError::Unauthorized(_)
                    ) {
                        self.circuit_breaker.record_failure().await;
                        return Err(err);
                    }

                    if attempt == self.max_retries {
                        self.circuit_breaker.record_failure().await;
                    }

                    // Exponential backoff
                    if attempt < self.max_retries {
                        let delay = Duration::from_millis(100 * (2_u64.pow(attempt)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| SecretonError::Other("Unknown error".to_string())))
    }

    /// Convert gRPC Status to SecretonError
    fn status_to_error(status: Status) -> SecretonError {
        use tonic::Code;

        match status.code() {
            Code::Unauthenticated => {
                SecretonError::AuthenticationFailed(status.message().to_string())
            }
            Code::PermissionDenied => SecretonError::Unauthorized(status.message().to_string()),
            Code::NotFound => SecretonError::NotFound(status.message().to_string()),
            Code::Unavailable | Code::DeadlineExceeded => {
                SecretonError::Unavailable(status.message().to_string())
            }
            Code::InvalidArgument => SecretonError::InvalidFormat(status.message().to_string()),
            _ => SecretonError::Other(format!("gRPC error: {}", status)),
        }
    }

    /// Create a gRPC request with authentication
    fn authenticated_request<T>(&self, message: T) -> Request<T> {
        let mut request = Request::new(message);
        request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", self.token).parse().unwrap(),
        );
        request
    }
}

#[async_trait]
impl SecretonClientTrait for GrpcSecretonClient {
    async fn get_secret(&self, key: &str, realm: Option<&str>) -> Option<Secret> {
        let operation = || {
            let key = key.to_string();
            let realm = realm.map(|s| s.to_string());
            let mut client = self.client.clone();
            let token = self.token.clone();

            async move {
                let path = if let Some(realm) = realm {
                    format!("{}/{}", realm, key)
                } else {
                    key.clone()
                };

                let request = GetSecretRequest {
                    path,
                    version: None,
                };

                let mut req = Request::new(request);
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );

                let response = client
                    .get_secret(req)
                    .await
                    .map_err(Self::status_to_error)?;

                let secret_resp = response.into_inner();

                Ok(Secret {
                    value: secret_resp
                        .data
                        .values()
                        .next()
                        .cloned()
                        .unwrap_or_default(),
                    metadata: Some(secret_resp.data),
                    version: Some(secret_resp.version),
                    created_at: Some(
                        chrono::DateTime::from_timestamp(secret_resp.created_at, 0)
                            .unwrap_or_default()
                            .with_timezone(&chrono::Utc),
                    ),
                    expires_at: None,
                })
            }
        };

        match self.execute_with_circuit_breaker(operation).await {
            Ok(secret) => Some(secret),
            Err(e) => {
                warn!("Failed to get secret: {}", e);
                None
            }
        }
    }

    async fn put_secret(
        &self,
        key: &str,
        value: &str,
        realm: Option<&str>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), SecretonError> {
        let operation = || {
            let key = key.to_string();
            let value = value.to_string();
            let realm = realm.map(|s| s.to_string());
            let metadata = metadata.clone();
            let mut client = self.client.clone();
            let token = self.token.clone();

            async move {
                let path = if let Some(realm) = realm {
                    format!("{}/{}", realm, key)
                } else {
                    key.clone()
                };

                let mut data = HashMap::new();
                data.insert("value".to_string(), value);

                if let Some(meta) = metadata {
                    data.extend(meta);
                }

                let request = StoreSecretRequest {
                    path,
                    data,
                    security_level: 3, // CONFIDENTIAL
                    tags: vec![],
                    ttl_seconds: None,
                };

                let mut req = Request::new(request);
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );

                client
                    .store_secret(req)
                    .await
                    .map_err(Self::status_to_error)?;

                Ok(())
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    async fn delete_secret(&self, key: &str, realm: Option<&str>) -> Result<(), SecretonError> {
        let operation = || {
            let key = key.to_string();
            let realm = realm.map(|s| s.to_string());
            let mut client = self.client.clone();
            let token = self.token.clone();

            async move {
                let path = if let Some(realm) = realm {
                    format!("{}/{}", realm, key)
                } else {
                    key.clone()
                };

                let request = DeleteSecretRequest { path };

                let mut req = Request::new(request);
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );

                client
                    .delete_secret(req)
                    .await
                    .map_err(Self::status_to_error)?;

                Ok(())
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    async fn list_secrets(&self, realm: Option<&str>) -> Result<Vec<String>, SecretonError> {
        let operation = || {
            let realm = realm.map(|s| s.to_string());
            let mut client = self.client.clone();
            let token = self.token.clone();

            async move {
                let request = ListSecretsRequest {
                    prefix: realm,
                    limit: Some(1000),
                    offset: Some(0),
                };

                let mut req = Request::new(request);
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );

                let response = client
                    .list_secrets(req)
                    .await
                    .map_err(Self::status_to_error)?;

                let secrets = response.into_inner();
                Ok(secrets.secrets.into_iter().map(|s| s.path).collect())
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    async fn rotate_secret(
        &self,
        key: &str,
        realm: Option<&str>,
        generator: Box<dyn Fn() -> String + Send>,
    ) -> Result<RotationResult, SecretonError> {
        // Get old secret
        let old_secret = self.get_secret(key, realm).await;

        // Generate new secret
        let new_value = generator();

        // Store new secret
        self.put_secret(key, &new_value, realm, None).await?;

        // Get the new secret details
        let new_secret = self
            .get_secret(key, realm)
            .await
            .ok_or_else(|| SecretonError::Other("Failed to retrieve rotated secret".to_string()))?;

        Ok(RotationResult {
            new_secret,
            old_secret,
            rotated_at: chrono::Utc::now(),
        })
    }

    async fn get_secret_versions(
        &self,
        _key: &str,
        _realm: Option<&str>,
    ) -> Result<Vec<Secret>, SecretonError> {
        // gRPC API would need a specific endpoint for this
        // For now, return unimplemented
        Err(SecretonError::Other(
            "Version history not yet implemented for gRPC client".to_string(),
        ))
    }

    async fn health_check(&self) -> Result<bool, SecretonError> {
        // Simple health check via any lightweight operation
        // In practice, use a dedicated health endpoint
        Ok(self.circuit_breaker.can_execute().await)
    }
}
