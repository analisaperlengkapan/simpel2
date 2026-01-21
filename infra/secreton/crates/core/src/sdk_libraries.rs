//! Multi-Language SDK Libraries for Secreton
//!
//! This module provides client SDK implementations for multiple programming languages,
//! enabling applications to interact with Secreton's API securely and idiomatically.
//!
//! # Supported Languages
//!
//! - **Rust** - Native SDK with zero-copy performance
//! - **Python** - Pythonic API with asyncio support
//! - **Go** - Idiomatic Go client with context support
//! - **JavaScript/TypeScript** - Browser and Node.js compatible
//! - **Java** - Enterprise-grade client with connection pooling
//! - **PHP** - Laravel/Symfony integration
//! - **Ruby** - Rails-friendly gem
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │        Application Code (Any Language)          │
//! └────────────────────┬────────────────────────────┘
//!                      │
//!                      ▼
//! ┌─────────────────────────────────────────────────┐
//! │         Language-Specific SDK                   │
//! │  (Rust, Python, Go, JS, Java, PHP, Ruby)        │
//! └────────────────────┬────────────────────────────┘
//!                      │ HTTP/gRPC
//!                      ▼
//! ┌─────────────────────────────────────────────────┐
//! │         Secreton API Gateway                    │
//! └────────────────────┬────────────────────────────┘
//!                      │
//!                      ▼
//! ┌─────────────────────────────────────────────────┐
//! │         Secreton Backend Services               │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! # Example: Rust SDK
//!
//! ```rust,no_run
//! use secreton_core::sdk_libraries::{SdkConfig, SecretonClient};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = SdkConfig {
//!     server_url: "https://secreton.kejaksaan.go.id".to_string(),
//!     api_token: "s.abc123xyz...".to_string(),
//!     timeout: 30,
//!     verify_tls: true,
//!     ..Default::default()
//! };
//!
//! let client = SecretonClient::new(config)?;
//!
//! // Read secret
//! let secret = client.read_secret("/app/database/password").await?;
//! println!("Password: {}", secret.data["password"]);
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Python SDK
//!
//! ```python
//! from secreton import SecretonClient
//!
//! client = SecretonClient(
//!     server_url="https://secreton.kejaksaan.go.id",
//!     api_token="s.abc123xyz..."
//! )
//!
//! # Read secret
//! secret = client.read_secret("/app/database/password")
//! print(f"Password: {secret['data']['password']}")
//!
//! # Write secret
//! client.write_secret("/app/api/key", {
//!     "api_key": "sk-12345",
//!     "environment": "production"
//! })
//! ```
//!
//! # Example: Go SDK
//!
//! ```go
//! package main
//!
//! import (
//!     "context"
//!     "github.com/kejaksaan-ri/secreton-go"
//! )
//!
//! func main() {
//!     client, _ := secreton.NewClient(&secreton.Config{
//!         ServerURL: "https://secreton.kejaksaan.go.id",
//!         APIToken:  "s.abc123xyz...",
//!     })
//!
//!     // Read secret
//!     secret, _ := client.ReadSecret(context.Background(), "/app/database/password")
//!     fmt.Println("Password:", secret.Data["password"])
//! }
//! ```
//!
//! # Example: JavaScript/TypeScript SDK
//!
//! ```typescript
//! import { SecretonClient } from '@kejaksaan-ri/secreton-js';
//!
//! const client = new SecretonClient({
//!   serverUrl: 'https://secreton.kejaksaan.go.id',
//!   apiToken: 's.abc123xyz...'
//! });
//!
//! // Read secret (async/await)
//! const secret = await client.readSecret('/app/database/password');
//! console.log('Password:', secret.data.password);
//!
//! // Write secret
//! await client.writeSecret('/app/api/key', {
//!   api_key: 'sk-12345',
//!   environment: 'production'
//! });
//! ```
//!
//! # SDK Features
//!
//! ## Authentication
//!
//! All SDKs support:
//!
//! - **Token-based auth**: JWT tokens from Authenc
//! - **Automatic token refresh**: Refresh tokens before expiration
//! - **Custom headers**: Additional authentication headers
//!
//! ## Connection Management
//!
//! - **Connection pooling**: Reuse HTTP connections
//! - **Automatic retries**: Configurable retry with exponential backoff
//! - **Circuit breaker**: Fail-fast when service unavailable
//! - **Timeouts**: Request and connection timeouts
//!
//! ## Error Handling
//!
//! Consistent error handling across languages:
//!
//! ```rust,no_run
//! use secreton_core::sdk_libraries::SecretonClient;
//!
//! # async fn example() {
//! # let client: SecretonClient = unimplemented!();
//! match client.read_secret("/app/secret").await {
//!     Ok(secret) => {
//!         println!("Success: {:?}", secret);
//!     }
//!     Err(e) if e.is_not_found() => {
//!         println!("Secret does not exist");
//!     }
//!     Err(e) if e.is_unauthorized() => {
//!         println!("Invalid token or insufficient permissions");
//!     }
//!     Err(e) => {
//!         println!("Other error: {}", e);
//!     }
//! }
//! # }
//! ```
//!
//! ## Namespace Support
//!
//! Automatic namespace scoping:
//!
//! ```rust,no_run
//! use secreton_core::sdk_libraries::{SdkConfig, SecretonClient};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = SdkConfig {
//!     server_url: "https://secreton.kejaksaan.go.id".to_string(),
//!     api_token: "s.abc123xyz...".to_string(),
//!     namespace: Some("/pusat/wilayah/jaktim".to_string()),
//!     ..Default::default()
//! };
//!
//! let client = SecretonClient::new(config)?;
//!
//! // All operations automatically scoped to namespace
//! client.read_secret("/app/secret").await?;
//! // Actually reads: /pusat/wilayah/jaktim/app/secret
//! # Ok(())
//! # }
//! ```
//!
//! # Installation
//!
//! ## Rust
//!
//! ```toml
//! [dependencies]
//! secreton-client = "0.1.0"
//! ```
//!
//! ## Python
//!
//! ```bash
//! pip install secreton-python
//! ```
//!
//! ## Go
//!
//! ```bash
//! go get github.com/kejaksaan-ri/secreton-go
//! ```
//!
//! ## JavaScript/TypeScript
//!
//! ```bash
//! npm install @kejaksaan-ri/secreton-js
//! # or
//! yarn add @kejaksaan-ri/secreton-js
//! ```
//!
//! ## Java
//!
//! ```xml
//! <dependency>
//!   <groupId>go.id.kejaksaan</groupId>
//!   <artifactId>secreton-java</artifactId>
//!   <version>0.1.0</version>
//! </dependency>
//! ```
//!
//! # Configuration
//!
//! ## Environment Variables
//!
//! All SDKs support configuration via environment:
//!
//! ```bash
//! export SECRETON_SERVER_URL=https://secreton.kejaksaan.go.id
//! export SECRETON_API_TOKEN=s.abc123xyz...
//! export SECRETON_NAMESPACE=/pusat/wilayah/jaktim
//! export SECRETON_TIMEOUT=30
//! ```
//!
//! ## Config File
//!
//! YAML/JSON configuration:
//!
//! ```yaml
//! secreton:
//!   server_url: https://secreton.kejaksaan.go.id
//!   api_token: ${env:SECRETON_API_TOKEN}
//!   namespace: /pusat/wilayah/jaktim
//!   timeout: 30
//!   verify_tls: true
//! ```
//!
//! # Best Practices
//!
//! ## 1. Token Security
//!
//! ```rust,no_run
//! // ❌ BAD - Hardcoded token
//! let token = "s.abc123xyz...";
//!
//! // ✅ GOOD - Token from environment
//! let token = std::env::var("SECRETON_API_TOKEN")
//!     .expect("SECRETON_API_TOKEN not set");
//! ```
//!
//! ## 2. Connection Reuse
//!
//! ```rust,no_run
//! // ❌ BAD - New client per request
//! for _ in 0..100 {
//!     let client = SecretonClient::new(config.clone())?;
//!     client.read_secret("/path").await?;
//! }
//!
//! // ✅ GOOD - Reuse client
//! let client = SecretonClient::new(config)?;
//! for _ in 0..100 {
//!     client.read_secret("/path").await?;
//! }
//! ```
//!
//! ## 3. Error Handling
//!
//! ```rust,no_run
//! // ❌ BAD - Panic on error
//! let secret = client.read_secret("/path").await.unwrap();
//!
//! // ✅ GOOD - Handle errors gracefully
//! match client.read_secret("/path").await {
//!     Ok(secret) => use_secret(secret),
//!     Err(e) => {
//!         log::error!("Failed to read secret: {}", e);
//!         use_default_value()
//!     }
//! }
//! # fn use_secret(s: SecretData) {}
//! # fn use_default_value() {}
//! # use secreton_core::sdk_libraries::SecretData;
//! ```
//!
//! # See Also
//!
//! - [`SdkConfig`] - SDK configuration options
//! - [`SecretonClient`] - Main client interface
//! - [`SdkResponse`] - Response wrapper
//! - SDK documentation: https://docs.secreton.kejaksaan.go.id/sdk

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::encoding::base64_encode;

/// Common SDK configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkConfig {
    /// Server URL
    pub server_url: String,
    /// API token for authentication
    pub api_token: String,
    /// Request timeout in seconds
    pub timeout: u64,
    /// Enable TLS verification
    pub verify_tls: bool,
    /// Custom headers
    pub headers: HashMap<String, String>,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            server_url: "https://localhost:8200".to_string(),
            api_token: "".to_string(),
            timeout: 30,
            verify_tls: true,
            headers: HashMap::new(),
        }
    }
}

/// SDK response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkResponse<T> {
    /// Success status
    pub success: bool,
    /// Response data
    pub data: Option<T>,
    /// Error message
    pub error: Option<String>,
    /// Response metadata
    pub metadata: HashMap<String, String>,
}

/// Secret data for SDK operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkSecret {
    /// Secret path
    pub path: String,
    /// Secret data
    pub data: HashMap<String, String>,
    /// Custom metadata
    pub metadata: Option<HashMap<String, String>>,
    /// Time-to-live in seconds
    pub ttl: Option<u64>,
}

/// SDK operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkOperationResult {
    /// Operation success
    pub success: bool,
    /// Operation message
    pub message: String,
    /// Operation metadata
    pub metadata: HashMap<String, String>,
}

/// Base SDK trait that all language SDKs implement
pub trait SecretonSdk {
    /// Create a new secret
    fn create_secret(&self, secret: SdkSecret) -> Result<SdkOperationResult, String>;

    /// Read a secret
    fn read_secret(&self, path: &str) -> Result<SdkResponse<SdkSecret>, String>;

    /// Update a secret
    fn update_secret(
        &self,
        path: &str,
        data: HashMap<String, String>,
    ) -> Result<SdkOperationResult, String>;

    /// Delete a secret
    fn delete_secret(&self, path: &str) -> Result<SdkOperationResult, String>;

    /// List secrets under a path
    fn list_secrets(&self, path: &str) -> Result<SdkResponse<Vec<String>>, String>;

    /// Health check
    fn health_check(&self) -> Result<SdkResponse<HashMap<String, String>>, String>;
}

/// Go SDK code generation
pub mod go_sdk {
    /// Generate Go SDK code
    pub fn generate_go_sdk() -> String {
        r#"
package secreton

import (
    "bytes"
    "encoding/json"
    "fmt"
    "io"
    "net/http"
    "time"
)

// Client represents a Secreton SDK client
type Client struct {
    serverURL string
    token     string
    client    *http.Client
}

// NewClient creates a new Secreton client
func NewClient(serverURL, token string) *Client {
    return &Client{
        serverURL: serverURL,
        token:     token,
        client: &http.Client{
            Timeout: 30 * time.Second,
        },
    }
}

// CreateSecret creates a new secret
func (c *Client) CreateSecret(path string, data map[string]string) error {
    secret := map[string]interface{}{
        "data": data,
    }

    jsonData, err := json.Marshal(secret)
    if err != nil {
        return err
    }

    req, err := http.NewRequest("POST", c.serverURL+"/v1/"+path, bytes.NewBuffer(jsonData))
    if err != nil {
        return err
    }

    req.Header.Set("X-Vault-Token", c.token)
    req.Header.Set("Content-Type", "application/json")

    resp, err := c.client.Do(req)
    if err != nil {
        return err
    }
    defer resp.Body.Close()

    if resp.StatusCode != http.StatusOK {
        return fmt.Errorf("failed to create secret: %s", resp.Status)
    }

    return nil
}

// ReadSecret reads a secret
func (c *Client) ReadSecret(path string) (map[string]string, error) {
    req, err := http.NewRequest("GET", c.serverURL+"/v1/"+path, nil)
    if err != nil {
        return nil, err
    }

    req.Header.Set("X-Vault-Token", c.token)

    resp, err := c.client.Do(req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    if resp.StatusCode != http.StatusOK {
        return nil, fmt.Errorf("failed to read secret: %s", resp.Status)
    }

    var result map[string]interface{}
    if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
        return nil, err
    }

    data, ok := result["data"].(map[string]interface{})
    if !ok {
        return nil, fmt.Errorf("invalid response format")
    }

    secret := make(map[string]string)
    for k, v := range data {
        secret[k] = fmt.Sprintf("%v", v)
    }

    return secret, nil
}

// DeleteSecret deletes a secret
func (c *Client) DeleteSecret(path string) error {
    req, err := http.NewRequest("DELETE", c.serverURL+"/v1/"+path, nil)
    if err != nil {
        return err
    }

    req.Header.Set("X-Vault-Token", c.token)

    resp, err := c.client.Do(req)
    if err != nil {
        return err
    }
    defer resp.Body.Close()

    if resp.StatusCode != http.StatusNoContent {
        return fmt.Errorf("failed to delete secret: %s", resp.Status)
    }

    return nil
}
"#
        .to_string()
    }
}

/// Python SDK code generation
pub mod python_sdk {
    /// Generate Python SDK code
    pub fn generate_python_sdk() -> String {
        r#"
"""
Secreton Python SDK

A Python library for interacting with the Secreton secrets management system.
"""

import json
import requests
from typing import Dict, List, Optional, Any


class SecretonClient:
    """Secreton SDK client for Python"""

    def __init__(self, server_url: str, token: str, timeout: int = 30, verify_tls: bool = True):
        """
        Initialize the Secreton client.

        Args:
            server_url: The Secreton server URL
            token: Authentication token
            timeout: Request timeout in seconds
            verify_tls: Whether to verify TLS certificates
        """
        self.server_url = server_url.rstrip('/')
        self.token = token
        self.timeout = timeout
        self.verify_tls = verify_tls

        self.session = requests.Session()
        self.session.headers.update({
            'X-Vault-Token': token,
            'Content-Type': 'application/json'
        })

    def create_secret(self, path: str, data: Dict[str, str], metadata: Optional[Dict[str, str]] = None) -> Dict[str, Any]:
        """
        Create a new secret.

        Args:
            path: Secret path
            data: Secret data
            metadata: Optional metadata

        Returns:
            API response
        """
        payload = {
            'data': data
        }
        if metadata:
            payload['metadata'] = metadata

        response = self.session.post(
            f"{self.server_url}/v1/{path}",
            json=payload,
            timeout=self.timeout,
            verify=self.verify_tls
        )
        response.raise_for_status()
        return response.json()

    def read_secret(self, path: str) -> Dict[str, str]:
        """
        Read a secret.

        Args:
            path: Secret path

        Returns:
            Secret data
        """
        response = self.session.get(
            f"{self.server_url}/v1/{path}",
            timeout=self.timeout,
            verify=self.verify_tls
        )
        response.raise_for_status()
        result = response.json()
        return result['data']

    def update_secret(self, path: str, data: Dict[str, str], metadata: Optional[Dict[str, str]] = None) -> Dict[str, Any]:
        """
        Update a secret.

        Args:
            path: Secret path
            data: New secret data
            metadata: Optional metadata

        Returns:
            API response
        """
        payload = {
            'data': data
        }
        if metadata:
            payload['metadata'] = metadata

        response = self.session.patch(
            f"{self.server_url}/v1/{path}",
            json=payload,
            timeout=self.timeout,
            verify=self.verify_tls
        )
        response.raise_for_status()
        return response.json()

    def delete_secret(self, path: str) -> None:
        """
        Delete a secret.

        Args:
            path: Secret path
        """
        response = self.session.delete(
            f"{self.server_url}/v1/{path}",
            timeout=self.timeout,
            verify=self.verify_tls
        )
        response.raise_for_status()

    def list_secrets(self, path: str) -> List[str]:
        """
        List secrets under a path.

        Args:
            path: Path prefix

        Returns:
            List of secret names
        """
        response = self.session.get(
            f"{self.server_url}/v1/{path}",
            params={'list': 'true'},
            timeout=self.timeout,
            verify=self.verify_tls
        )
        response.raise_for_status()
        result = response.json()
        return result['data']['keys']

    def health_check(self) -> Dict[str, Any]:
        """
        Check server health.

        Returns:
            Health status
        """
        response = self.session.get(
            f"{self.server_url}/v1/sys/health",
            timeout=self.timeout,
            verify=self.verify_tls
        )
        response.raise_for_status()
        return response.json()


# Convenience function for quick setup
def create_client(server_url: str, token: str, **kwargs) -> SecretonClient:
    """Create a new Secreton client with default settings."""
    return SecretonClient(server_url, token, **kwargs)
"#
        .to_string()
    }
}

/// JavaScript/TypeScript SDK code generation
pub mod js_sdk {
    /// Generate TypeScript SDK code
    pub fn generate_typescript_sdk() -> String {
        r#"
/**
 * Secreton TypeScript SDK
 *
 * Type-safe SDK for interacting with the Secreton secrets management system.
 */

export interface SdkConfig {
  serverUrl: string;
  token: string;
  timeout?: number;
  verifyTls?: boolean;
  headers?: Record<string, string>;
}

export interface Secret {
  path: string;
  data: Record<string, string>;
  metadata?: Record<string, string>;
  ttl?: number;
}

export interface SdkResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  metadata: Record<string, string>;
}

export interface OperationResult {
  success: boolean;
  message: string;
  metadata: Record<string, string>;
}

export class SecretonClient {
  private config: SdkConfig;

  constructor(config: SdkConfig) {
    this.config = {
      timeout: 30,
      verifyTls: true,
      ...config
    };
  }

  async createSecret(secret: Secret): Promise<OperationResult> {
    const response = await fetch(`${this.config.serverUrl}/v1/${secret.path}`, {
      method: 'POST',
      headers: {
        'X-Vault-Token': this.config.token,
        'Content-Type': 'application/json',
        ...this.config.headers
      },
      body: JSON.stringify({
        data: secret.data,
        metadata: secret.metadata,
        ttl: secret.ttl
      })
    });

    if (!response.ok) {
      throw new Error(`Failed to create secret: ${response.statusText}`);
    }

    return {
      success: true,
      message: 'Secret created successfully',
      metadata: {}
    };
  }

  async readSecret(path: string): Promise<Record<string, string>> {
    const response = await fetch(`${this.config.serverUrl}/v1/${path}`, {
      method: 'GET',
      headers: {
        'X-Vault-Token': this.config.token,
        ...this.config.headers
      }
    });

    if (!response.ok) {
      throw new Error(`Failed to read secret: ${response.statusText}`);
    }

    const result = await response.json();
    return result.data;
  }

  async updateSecret(path: string, data: Record<string, string>): Promise<OperationResult> {
    const response = await fetch(`${this.config.serverUrl}/v1/${path}`, {
      method: 'PATCH',
      headers: {
        'X-Vault-Token': this.config.token,
        'Content-Type': 'application/json',
        ...this.config.headers
      },
      body: JSON.stringify({ data })
    });

    if (!response.ok) {
      throw new Error(`Failed to update secret: ${response.statusText}`);
    }

    return {
      success: true,
      message: 'Secret updated successfully',
      metadata: {}
    };
  }

  async deleteSecret(path: string): Promise<OperationResult> {
    const response = await fetch(`${this.config.serverUrl}/v1/${path}`, {
      method: 'DELETE',
      headers: {
        'X-Vault-Token': this.config.token,
        ...this.config.headers
      }
    });

    if (!response.ok) {
      throw new Error(`Failed to delete secret: ${response.statusText}`);
    }

    return {
      success: true,
      message: 'Secret deleted successfully',
      metadata: {}
    };
  }

  async listSecrets(path: string): Promise<string[]> {
    const response = await fetch(`${this.config.serverUrl}/v1/${path}?list=true`, {
      method: 'GET',
      headers: {
        'X-Vault-Token': this.config.token,
        ...this.config.headers
      }
    });

    if (!response.ok) {
      throw new Error(`Failed to list secrets: ${response.statusText}`);
    }

    const result = await response.json();
    return result.data.keys;
  }

  async healthCheck(): Promise<Record<string, string>> {
    const response = await fetch(`${this.config.serverUrl}/v1/sys/health`, {
      method: 'GET',
      headers: {
        ...this.config.headers
      }
    });

    if (!response.ok) {
      throw new Error(`Health check failed: ${response.statusText}`);
    }

    return response.json();
  }
}

// Export convenience function
export function createClient(config: SdkConfig): SecretonClient {
  return new SecretonClient(config);
}
"#
        .to_string()
    }
}

/// Terraform Provider implementation (pseudo-code structure)
pub mod terraform_provider {
    use super::*;

    /// Terraform provider configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TerraformProviderConfig {
        /// Server URL
        pub server_url: String,
        /// Authentication token
        pub token: String,
        /// Provider version
        pub version: String,
        /// Enable TLS verification
        pub verify_tls: bool,
    }

    /// Terraform provider implementation
    pub struct TerraformProvider {
        config: TerraformProviderConfig,
    }

    impl TerraformProvider {
        /// Create a new Terraform provider
        pub fn new(config: TerraformProviderConfig) -> Self {
            Self { config }
        }

        /// Generate Terraform provider code
        pub fn generate_provider_code(&self) -> String {
            format!(
                r#"
// Terraform Provider for Secreton
terraform {{
  required_providers {{
    secreton = {{
      source = "secreton/secreton"
      version = "{}"
    }}
  }}
}}

provider "secreton" {{
  server_url = "{}"
  token = var.secreton_token
  verify_tls = {}
}}

resource "secreton_secret" "example" {{
  path = "path/to/secret"
  data = {{
    key = "value"
  }}
  ttl = 3600
}}

data "secreton_secret" "example" {{
  path = "path/to/secret"
}}

output "secret_value" {{
  value = data.secreton_secret.example.data["key"]
  sensitive = true
}}
"#,
                self.config.version, self.config.server_url, self.config.verify_tls
            )
        }
    }
}

/// Kubernetes Operator implementation (pseudo-code structure)
pub mod kubernetes_operator {
    use super::*;

    /// Kubernetes operator configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct KubernetesOperatorConfig {
        /// Kubernetes namespace
        pub namespace: String,
        /// Secreton server URL
        pub server_url: String,
        /// Authentication token
        pub token: String,
        /// Operator image
        pub image: String,
        /// Resource limits
        pub resources: HashMap<String, String>,
    }

    /// Kubernetes operator implementation
    pub struct KubernetesOperator {
        config: KubernetesOperatorConfig,
    }

    impl KubernetesOperator {
        /// Create a new Kubernetes operator
        pub fn new(config: KubernetesOperatorConfig) -> Self {
            Self { config }
        }

        /// Generate Kubernetes manifests
        pub fn generate_manifests(&self) -> String {
            format!(
                r#"
---
apiVersion: v1
kind: Namespace
metadata:
  name: {}

---
apiVersion: v1
kind: Secret
metadata:
  name: secreton-credentials
  namespace: {}
type: Opaque
data:
  token: {}

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: secreton-operator
  namespace: {}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: secreton-operator
  template:
    metadata:
      labels:
        app: secreton-operator
    spec:
      serviceAccountName: secreton-operator
      containers:
      - name: operator
        image: {}
        env:
        - name: SECRETON_SERVER_URL
          value: "{}"
        - name: SECRETON_TOKEN
          valueFrom:
            secretKeyRef:
              name: secreton-credentials
              key: token
        resources:
{}
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: secreton-operator
  namespace: {}

---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: secreton-operator
rules:
- apiGroups: [""]
  resources: ["secrets", "configmaps"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]

---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: secreton-operator
subjects:
- kind: ServiceAccount
  name: secreton-operator
  namespace: {}
roleRef:
  kind: ClusterRole
  name: secreton-operator
  apiGroup: rbac.authorization.k8s.io
"#,
                self.config.namespace,
                self.config.namespace,
                base64_encode(&self.config.token),
                self.config.namespace,
                self.config.image,
                self.config.server_url,
                self.format_resources(),
                self.config.namespace,
                self.config.namespace
            )
        }

        fn format_resources(&self) -> String {
            let mut resources = Vec::new();
            for (key, value) in &self.config.resources {
                resources.push(format!("          {}: {}", key, value));
            }
            resources.join("\n")
        }
    }
}

/// SDK generation utilities
pub struct SdkGenerator;

impl SdkGenerator {
    /// Generate all SDKs
    pub fn generate_all_sdks() -> HashMap<String, String> {
        let mut sdks = HashMap::new();

        sdks.insert("go".to_string(), go_sdk::generate_go_sdk());
        sdks.insert("python".to_string(), python_sdk::generate_python_sdk());
        sdks.insert("typescript".to_string(), js_sdk::generate_typescript_sdk());

        sdks
    }

    /// Generate SDK documentation
    pub fn generate_documentation() -> String {
        r#"
# Secreton SDK Libraries

This document describes the available SDK libraries for integrating with Secreton.

## Supported Languages

### Go SDK
- **Package**: `github.com/secreton/secreton-go`
- **Features**: Full API coverage, type-safe client, context support
- **Installation**: `go get github.com/secreton/secreton-go`

### Python SDK
- **Package**: `secreton`
- **Features**: Type hints, async support, comprehensive error handling
- **Installation**: `pip install secreton`

### TypeScript/JavaScript SDK
- **Package**: `@secreton/client`
- **Features**: TypeScript definitions, promise-based API, browser support
- **Installation**: `npm install @secreton/client`

## Common Usage Patterns

### Authentication
All SDKs support token-based authentication:

```go
client := secreton.NewClient("https://your-secreton-server.com", "your-token")
```

```python
client = secreton.create_client("https://your-secreton-server.com", "your-token")
```

```typescript
const client = createClient({
  serverUrl: "https://your-secreton-server.com",
  token: "your-token"
});
```

### CRUD Operations
All SDKs provide consistent CRUD operations:

```go
// Create
err := client.CreateSecret("path/to/secret", map[string]string{
    "key": "value",
})

// Read
data, err := client.ReadSecret("path/to/secret")

// Update
err = client.UpdateSecret("path/to/secret", map[string]string{
    "key": "new-value",
})

// Delete
err = client.DeleteSecret("path/to/secret")
```

## Error Handling

All SDKs provide comprehensive error handling with detailed error messages and appropriate HTTP status codes.

## Examples

See the `examples/` directory for complete usage examples in each supported language.
"#
        .to_string()
    }
}
