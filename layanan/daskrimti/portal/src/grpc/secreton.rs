//! Secreton gRPC Client
//!
//! Client wrapper for Secreton Secret Management service providing:
//! - Secret storage and retrieval
//! - Transit encryption/decryption
//! - Key management

use super::generated::secreton_v1::{
    secreton_service_client::SecretonServiceClient,
    StoreSecretRequest, StoreSecretResponse,
    GetSecretRequest, GetSecretResponse,
    DeleteSecretRequest, DeleteSecretResponse,
    ListSecretsRequest, ListSecretsResponse,
    CreateKeyRequest, CreateKeyResponse,
    EncryptRequest, EncryptResponse,
    DecryptRequest, DecryptResponse,
    SecurityLevel, KeyType,
};
use std::collections::HashMap;
use tonic::transport::{Channel, Endpoint};
use tonic::metadata::MetadataValue;
use tracing::{debug, info};

/// Error type for Secreton client operations
#[derive(Debug, thiserror::Error)]
pub enum SecretonError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),
}

/// Secreton gRPC Client wrapper
#[derive(Debug, Clone)]
pub struct SecretonClient {
    client: SecretonServiceClient<Channel>,
    auth_token: Option<String>,
}

impl SecretonClient {
    /// Create a new Secreton client
    pub async fn new(endpoint: &str) -> Result<Self, SecretonError> {
        info!("Connecting to Secreton gRPC at {}", endpoint);

        let endpoint = Endpoint::from_shared(endpoint.to_string())
            .map_err(|e| SecretonError::Connection(e.to_string()))?;

        let channel = endpoint
            .connect()
            .await
            .map_err(|e| SecretonError::Connection(e.to_string()))?;

        Ok(Self {
            client: SecretonServiceClient::new(channel),
            auth_token: None,
        })
    }

    /// Set authentication token for subsequent requests
    pub fn set_auth_token(&mut self, token: &str) {
        self.auth_token = Some(token.to_string());
    }

    /// Create a request with auth header if token is set
    fn create_request<T>(&self, message: T) -> tonic::Request<T> {
        let mut request = tonic::Request::new(message);

        if let Some(ref token) = self.auth_token {
            if let Ok(value) = format!("Bearer {}", token).parse::<MetadataValue<_>>() {
                request.metadata_mut().insert("authorization", value);
            }
        }

        request
    }

    /// Store a secret
    pub async fn store_secret(
        &mut self,
        path: &str,
        data: HashMap<String, String>,
        security_level: Option<i32>,
        tags: Vec<String>,
        ttl_seconds: Option<i64>,
    ) -> Result<StoreSecretResponse, SecretonError> {
        debug!("Storing secret at path: {}", path);

        let request = self.create_request(StoreSecretRequest {
            path: path.to_string(),
            data,
            security_level: security_level.unwrap_or(SecurityLevel::Confidential as i32),
            tags,
            ttl_seconds,
        });

        let response = self.client.store_secret(request).await?;
        info!("Secret stored at path: {}", path);
        Ok(response.into_inner())
    }

    /// Get a secret by path
    pub async fn get_secret(
        &mut self,
        path: &str,
        version: Option<u32>,
    ) -> Result<GetSecretResponse, SecretonError> {
        debug!("Getting secret at path: {}", path);

        let request = self.create_request(GetSecretRequest {
            path: path.to_string(),
            version,
        });

        let response = self.client.get_secret(request).await?;
        Ok(response.into_inner())
    }

    /// Delete a secret
    pub async fn delete_secret(
        &mut self,
        path: &str,
    ) -> Result<DeleteSecretResponse, SecretonError> {
        debug!("Deleting secret at path: {}", path);

        let request = self.create_request(DeleteSecretRequest {
            path: path.to_string(),
        });

        let response = self.client.delete_secret(request).await?;
        info!("Secret deleted at path: {}", path);
        Ok(response.into_inner())
    }

    /// List secrets with optional prefix filter
    pub async fn list_secrets(
        &mut self,
        prefix: Option<&str>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<ListSecretsResponse, SecretonError> {
        debug!("Listing secrets with prefix: {:?}", prefix);

        let request = self.create_request(ListSecretsRequest {
            prefix: prefix.map(|s| s.to_string()),
            limit,
            offset,
        });

        let response = self.client.list_secrets(request).await?;
        Ok(response.into_inner())
    }

    /// Create an encryption key
    pub async fn create_key(
        &mut self,
        name: &str,
        key_type: Option<i32>,
    ) -> Result<CreateKeyResponse, SecretonError> {
        debug!("Creating key: {}", name);

        let request = self.create_request(CreateKeyRequest {
            name: name.to_string(),
            key_type: key_type.unwrap_or(KeyType::Aes256Gcm as i32),
        });

        let response = self.client.create_key(request).await?;
        info!("Key created: {}", name);
        Ok(response.into_inner())
    }

    /// Encrypt data using a named key
    pub async fn encrypt(
        &mut self,
        key_name: &str,
        plaintext: Vec<u8>,
        context: Option<Vec<u8>>,
    ) -> Result<EncryptResponse, SecretonError> {
        debug!("Encrypting data with key: {}", key_name);

        let request = self.create_request(EncryptRequest {
            key_name: key_name.to_string(),
            plaintext,
            context,
        });

        let response = self.client.encrypt(request).await?;
        Ok(response.into_inner())
    }

    /// Decrypt data using a named key
    pub async fn decrypt(
        &mut self,
        key_name: &str,
        ciphertext: &str,
        context: Option<Vec<u8>>,
    ) -> Result<DecryptResponse, SecretonError> {
        debug!("Decrypting data with key: {}", key_name);

        let request = self.create_request(DecryptRequest {
            key_name: key_name.to_string(),
            ciphertext: ciphertext.to_string(),
            context,
        });

        let response = self.client.decrypt(request).await?;
        Ok(response.into_inner())
    }
}
