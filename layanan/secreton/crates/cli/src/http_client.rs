//! HTTP client wrapper with automatic token injection
//!
//! This module provides an HTTP client that automatically injects
//! authentication tokens from TokenStore into all requests.

use anyhow::{Context, Result};
use reqwest::{Client, RequestBuilder, Response};

use crate::token_store::TokenStore;

/// HTTP client wrapper that auto-injects authentication tokens
pub struct AuthenticatedClient {
    client: Client,
    token_store: TokenStore,
}

impl AuthenticatedClient {
    /// Create a new authenticated client
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            token_store: TokenStore::new()?,
        })
    }

    /// Get the underlying reqwest client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Build a GET request with automatic token injection
    pub fn get(&self, url: &str) -> Result<RequestBuilder> {
        let token = self.get_token()?;
        Ok(self.client.get(url).header("X-Secreton-Token", token))
    }

    /// Build a POST request with automatic token injection
    pub fn post(&self, url: &str) -> Result<RequestBuilder> {
        let token = self.get_token()?;
        Ok(self.client.post(url).header("X-Secreton-Token", token))
    }

    /// Build a PUT request with automatic token injection
    pub fn put(&self, url: &str) -> Result<RequestBuilder> {
        let token = self.get_token()?;
        Ok(self.client.put(url).header("X-Secreton-Token", token))
    }

    /// Build a DELETE request with automatic token injection
    pub fn delete(&self, url: &str) -> Result<RequestBuilder> {
        let token = self.get_token()?;
        Ok(self.client.delete(url).header("X-Secreton-Token", token))
    }

    /// Get the authentication token from TokenStore
    ///
    /// Returns an error if no token is stored or if the token is expired.
    fn get_token(&self) -> Result<String> {
        let stored_token = self
            .token_store
            .load_token()?
            .context("Not authenticated. Run 'secreton login' first.")?;

        // Check if token is expired
        if self.token_store.is_token_expired() {
            anyhow::bail!("Token expired. Run 'secreton login' to re-authenticate.");
        }

        Ok(stored_token.token)
    }

    /// Execute a request and handle token expiration gracefully
    ///
    /// If the request fails with a 401 Unauthorized status, this will
    /// return a helpful error message prompting the user to re-authenticate.
    pub async fn execute(&self, request: RequestBuilder) -> Result<Response> {
        let response = request
            .send()
            .await
            .context("Failed to connect to Secreton server")?;

        // Check for authentication errors
        if response.status() == 401 {
            anyhow::bail!(
                "Authentication failed. Your token may have expired. Run 'secreton login' to re-authenticate."
            );
        }

        Ok(response)
    }
}

impl Default for AuthenticatedClient {
    fn default() -> Self {
        Self::new().expect("Failed to create AuthenticatedClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_store::TokenMetadata;
    use chrono::{Duration, Utc};
    use tempfile::TempDir;

    fn create_test_client() -> (AuthenticatedClient, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let token_file = config_dir.join("token");

        let token_store = TokenStore {
            config_dir,
            token_file,
        };

        let client = AuthenticatedClient {
            client: Client::new(),
            token_store,
        };

        (client, temp_dir)
    }

    #[test]
    fn test_get_token_not_authenticated() {
        let (client, _temp) = create_test_client();

        let result = client.get_token();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Not authenticated")
        );
    }

    #[test]
    fn test_get_token_expired() {
        let (client, _temp) = create_test_client();

        // Store an expired token
        let token = "stn.expired_token";
        let metadata = TokenMetadata {
            expires_at: Some(Utc::now() - Duration::hours(1)),
            policies: vec![],
            renewable: false,
            display_name: None,
        };

        client.token_store.store_token(token, &metadata).unwrap();

        let result = client.get_token();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expired"));
    }

    #[test]
    fn test_get_token_valid() {
        let (client, _temp) = create_test_client();

        // Store a valid token
        let token = "stn.valid_token";
        let metadata = TokenMetadata {
            expires_at: Some(Utc::now() + Duration::hours(1)),
            policies: vec![],
            renewable: false,
            display_name: None,
        };

        client.token_store.store_token(token, &metadata).unwrap();

        let result = client.get_token();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), token);
    }

    #[test]
    fn test_request_builders() {
        let (client, _temp) = create_test_client();

        // Store a valid token
        let token = "stn.test_token";
        let metadata = TokenMetadata {
            expires_at: Some(Utc::now() + Duration::hours(1)),
            policies: vec![],
            renewable: false,
            display_name: None,
        };

        client.token_store.store_token(token, &metadata).unwrap();

        // Test that request builders work
        let get_req = client.get("http://example.com");
        assert!(get_req.is_ok());

        let post_req = client.post("http://example.com");
        assert!(post_req.is_ok());

        let put_req = client.put("http://example.com");
        assert!(put_req.is_ok());

        let delete_req = client.delete("http://example.com");
        assert!(delete_req.is_ok());
    }
}
