use std::collections::HashMap;
use anyhow::Result;

#[derive(Clone)]
pub struct SecretonClient;

impl SecretonClient {
    pub async fn connect(_addr: String) -> Result<Self> {
        // Stub implementation due to build issues with tonic/axum integration (http::Request type mismatch)
        // In a real environment, this would connect to the Secreton gRPC service.
        Ok(Self)
    }

    pub async fn get_secret(&self, _path: &str) -> Result<HashMap<String, String>> {
        // Stub implementation. Returns empty map.
        // The calling code in main.rs handles this by falling back to environment variables
        // or failing if strictly required.
        Ok(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stub_client() {
        let client = SecretonClient::connect("http://localhost:50051".to_string()).await.unwrap();
        let secrets = client.get_secret("test").await.unwrap();
        assert!(secrets.is_empty());
    }
}
