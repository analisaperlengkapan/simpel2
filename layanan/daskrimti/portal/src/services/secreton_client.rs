//! Secreton gRPC client for secret management
//!
//! Provides a full gRPC client implementation for Secreton secret manager
//! with circuit breaker pattern for resilience.

use std::collections::HashMap;
use std::time::Duration;
use tonic::Request;
use tonic::transport::Channel;
use tracing::info;

use crate::proto::secreton::v1::{
    DecryptRequest, DeleteSecretRequest, EncryptRequest, GetSecretRequest, ListSecretsRequest,
    SecurityLevel, StoreSecretRequest, secreton_service_client::SecretonServiceClient,
};

use super::authenc_client::CircuitBreaker;

/// Error types for Secreton client
#[derive(Debug, Clone, thiserror::Error)]
pub enum SecretonError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Service unavailable: {0}")]
    Unavailable(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<tonic::Status> for SecretonError {
    fn from(status: tonic::Status) -> Self {
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
            Code::InvalidArgument => SecretonError::InvalidRequest(status.message().to_string()),
            _ => SecretonError::Internal(format!("gRPC error: {}", status)),
        }
    }
}

/// Secret data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Secret {
    pub id: String,
    pub path: String,
    pub data: HashMap<String, String>,
    pub version: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Encryption result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptionResult {
    pub ciphertext: String,
    pub key_version: u32,
}

/// Client for Secreton secret manager
pub struct SecretonClient {
    client: SecretonServiceClient<Channel>,
    circuit_breaker: CircuitBreaker,
    max_retries: u32,
}

impl SecretonClient {
    /// Create a new Secreton client
    pub async fn new(grpc_url: &str) -> anyhow::Result<Self> {
        info!("Connecting to Secreton at {}", grpc_url);
        let channel = Channel::from_shared(grpc_url.to_string())?.connect_lazy();

        let client = SecretonServiceClient::new(channel);

        Ok(Self {
            client,
            circuit_breaker: CircuitBreaker::new(),
            max_retries: 3,
        })
    }

    /// Execute a request with circuit breaker and retry logic
    async fn execute_with_resilience<F, Fut, T>(&self, operation: F) -> Result<T, SecretonError>
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

        Err(last_error.unwrap_or_else(|| SecretonError::Internal("Unknown error".to_string())))
    }

    /// Get a secret by path
    pub async fn get_secret(&self, token: &str, path: &str) -> Result<Secret, SecretonError> {
        let path = path.to_string();
        let token = token.to_string();
        let client = self.client.clone();

        self.execute_with_resilience(|| {
            let path = path.clone();
            let token = token.clone();
            let mut client = client.clone();

            async move {
                let request = GetSecretRequest {
                    path: path.clone(),
                    version: None,
                };

                let mut req = Request::new(request);
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );

                let response = client.get_secret(req).await.map_err(SecretonError::from)?;

                let resp = response.into_inner();

                Ok(Secret {
                    id: resp.id,
                    path: resp.path,
                    data: resp.data,
                    version: resp.version,
                    created_at: resp.created_at,
                    updated_at: resp.updated_at,
                })
            }
        })
        .await
    }

    /// Store a secret
    pub async fn store_secret(
        &self,
        token: &str,
        path: &str,
        data: HashMap<String, String>,
        security_level: Option<i32>,
    ) -> Result<(String, u32), SecretonError> {
        let path = path.to_string();
        let token = token.to_string();
        let client = self.client.clone();
        let security_level = security_level.unwrap_or(SecurityLevel::Confidential as i32);

        self.execute_with_resilience(|| {
            let path = path.clone();
            let token = token.clone();
            let data = data.clone();
            let mut client = client.clone();

            async move {
                let request = StoreSecretRequest {
                    path: path.clone(),
                    data,
                    security_level,
                    tags: vec![],
                    ttl_seconds: None,
                };

                let mut req = Request::new(request);
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );

                let response = client
                    .store_secret(req)
                    .await
                    .map_err(SecretonError::from)?;

                let resp = response.into_inner();

                Ok((resp.id, resp.version))
            }
        })
        .await
    }

    /// Delete a secret
    pub async fn delete_secret(&self, token: &str, path: &str) -> Result<bool, SecretonError> {
        let path = path.to_string();
        let token = token.to_string();
        let client = self.client.clone();

        self.execute_with_resilience(|| {
            let path = path.clone();
            let token = token.clone();
            let mut client = client.clone();

            async move {
                let request = DeleteSecretRequest { path: path.clone() };

                let mut req = Request::new(request);
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );

                let response = client
                    .delete_secret(req)
                    .await
                    .map_err(SecretonError::from)?;

                Ok(response.into_inner().success)
            }
        })
        .await
    }

    /// List secrets with optional prefix filter
    pub async fn list_secrets(
        &self,
        token: &str,
        prefix: Option<&str>,
    ) -> Result<Vec<String>, SecretonError> {
        let prefix = prefix.map(|s| s.to_string());
        let token = token.to_string();
        let client = self.client.clone();

        self.execute_with_resilience(|| {
            let prefix = prefix.clone();
            let token = token.clone();
            let mut client = client.clone();

            async move {
                let request = ListSecretsRequest {
                    prefix,
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
                    .map_err(SecretonError::from)?;

                let secrets = response.into_inner();
                Ok(secrets.secrets.into_iter().map(|s| s.path).collect())
            }
        })
        .await
    }

    /// Encrypt data using transit engine
    pub async fn encrypt(
        &self,
        token: &str,
        key_name: &str,
        plaintext: &[u8],
    ) -> Result<EncryptionResult, SecretonError> {
        let key_name = key_name.to_string();
        let plaintext = plaintext.to_vec();
        let token = token.to_string();
        let client = self.client.clone();

        self.execute_with_resilience(|| {
            let key_name = key_name.clone();
            let plaintext = plaintext.clone();
            let token = token.clone();
            let mut client = client.clone();

            async move {
                let request = EncryptRequest {
                    key_name,
                    plaintext,
                    context: None,
                };

                let mut req = Request::new(request);
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );

                let response = client.encrypt(req).await.map_err(SecretonError::from)?;

                let resp = response.into_inner();

                Ok(EncryptionResult {
                    ciphertext: resp.ciphertext,
                    key_version: resp.key_version,
                })
            }
        })
        .await
    }

    /// Decrypt data using transit engine
    pub async fn decrypt(
        &self,
        token: &str,
        key_name: &str,
        ciphertext: &str,
    ) -> Result<Vec<u8>, SecretonError> {
        let key_name = key_name.to_string();
        let ciphertext = ciphertext.to_string();
        let token = token.to_string();
        let client = self.client.clone();

        self.execute_with_resilience(|| {
            let key_name = key_name.clone();
            let ciphertext = ciphertext.clone();
            let token = token.clone();
            let mut client = client.clone();

            async move {
                let request = DecryptRequest {
                    key_name,
                    ciphertext,
                    context: None,
                };

                let mut req = Request::new(request);
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );

                let response = client.decrypt(req).await.map_err(SecretonError::from)?;

                Ok(response.into_inner().plaintext)
            }
        })
        .await
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool, SecretonError> {
        Ok(self.circuit_breaker.can_execute().await)
    }
}
