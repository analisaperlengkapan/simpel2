//! Secreton gRPC client for secret management

use tonic::transport::Channel;
use tracing::info;

/// Client for Secreton secret manager
pub struct SecretonClient {
    #[allow(dead_code)]
    channel: Channel,
}

impl SecretonClient {
    /// Create a new Secreton client
    pub async fn new(grpc_url: &str) -> anyhow::Result<Self> {
        info!("Connecting to Secreton at {}", grpc_url);
        let channel = Channel::from_shared(grpc_url.to_string())?
            .connect_lazy();

        Ok(Self { channel })
    }

    /// Get a secret by path
    pub async fn get_secret(&self, _path: &str) -> anyhow::Result<Secret> {
        // TODO: Implement actual gRPC call
        Ok(Secret {
            path: "secret/data/portal".to_string(),
            data: serde_json::json!({
                "api_key": "***",
                "db_password": "***"
            }),
        })
    }

    /// List secrets in a path
    pub async fn list_secrets(&self, _path: &str) -> anyhow::Result<Vec<String>> {
        // TODO: Implement actual gRPC call
        Ok(vec![
            "secret/data/portal/api".to_string(),
            "secret/data/portal/db".to_string(),
        ])
    }
}

#[derive(Debug)]
pub struct Secret {
    pub path: String,
    pub data: serde_json::Value,
}
