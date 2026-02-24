//! Custom Resource Definition for SecretSync
//!
//! Defines the SecretSync CRD that allows users to declaratively specify
//! which Secreton secrets should be synchronized to Kubernetes Secrets.

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// SecretSync Custom Resource Definition
///
/// This CRD allows users to specify a Secreton secret path and have it
/// automatically synchronized to a Kubernetes Secret object.
///
/// # Example
///
/// ```yaml
/// apiVersion: secreton.cipherce.io/v1
/// kind: SecretSync
/// metadata:
///   name: database-credentials
///   namespace: production
/// spec:
///   secretonPath: /secret/data/database/prod
///   secretonNamespace: production
///   targetSecret: database-creds
///   refreshInterval: 300
///   secretonUrl: https://secreton.internal:8200
/// ```
#[derive(CustomResource, Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "secreton.cipherce.io",
    version = "v1",
    kind = "SecretSync",
    namespaced,
    status = "SecretSyncStatus",
    printcolumn = r#"{"name":"Path", "type":"string", "jsonPath":".spec.secretonPath"}"#,
    printcolumn = r#"{"name":"Target", "type":"string", "jsonPath":".spec.targetSecret"}"#,
    printcolumn = r#"{"name":"Status", "type":"string", "jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#
)]
pub struct SecretSyncSpec {
    /// Path to the secret in Secreton (e.g., /secret/data/myapp/config)
    #[serde(rename = "secretonPath")]
    pub secreton_path: String,

    /// Secreton namespace (optional, defaults to "default")
    #[serde(rename = "secretonNamespace", skip_serializing_if = "Option::is_none")]
    pub secreton_namespace: Option<String>,

    /// Name of the target Kubernetes Secret to create/update
    #[serde(rename = "targetSecret")]
    pub target_secret: String,

    /// Refresh interval in seconds (default: 300)
    #[serde(rename = "refreshInterval", skip_serializing_if = "Option::is_none")]
    pub refresh_interval: Option<u64>,

    /// Secreton server URL (optional, can be configured globally)
    #[serde(rename = "secretonUrl", skip_serializing_if = "Option::is_none")]
    pub secreton_url: Option<String>,

    /// Authentication method configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,

    /// Data transformation rules (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<TransformConfig>,
}

/// Authentication configuration for Secreton
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AuthConfig {
    /// Authentication method (token, kubernetes, approle)
    pub method: String,

    /// Kubernetes service account for authentication (when method=kubernetes)
    #[serde(rename = "serviceAccount", skip_serializing_if = "Option::is_none")]
    pub service_account: Option<String>,

    /// Token secret reference (when method=token)
    #[serde(rename = "tokenSecret", skip_serializing_if = "Option::is_none")]
    pub token_secret: Option<SecretReference>,

    /// AppRole configuration (when method=approle)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approle: Option<AppRoleConfig>,
}

/// Reference to a Kubernetes Secret
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SecretReference {
    /// Secret name
    pub name: String,

    /// Secret key
    pub key: String,
}

/// AppRole authentication configuration
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AppRoleConfig {
    /// Role ID
    #[serde(rename = "roleId")]
    pub role_id: String,

    /// Secret ID reference
    #[serde(rename = "secretId")]
    pub secret_id: SecretReference,
}

/// Data transformation configuration
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct TransformConfig {
    /// Key mappings (Secreton key -> Kubernetes Secret key)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mappings: Option<std::collections::HashMap<String, String>>,

    /// Template for generating secret data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

/// Status of the SecretSync resource
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SecretSyncStatus {
    /// Current phase (Pending, Syncing, Synced, Failed)
    pub phase: String,

    /// Last sync timestamp
    #[serde(rename = "lastSyncTime", skip_serializing_if = "Option::is_none")]
    pub last_sync_time: Option<String>,

    /// Last successful sync timestamp
    #[serde(rename = "lastSuccessfulSync", skip_serializing_if = "Option::is_none")]
    pub last_successful_sync: Option<String>,

    /// Error message (if phase=Failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Number of sync attempts
    #[serde(rename = "syncAttempts", skip_serializing_if = "Option::is_none")]
    pub sync_attempts: Option<u32>,

    /// Secret version from Secreton
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u64>,
}

impl Default for SecretSyncStatus {
    fn default() -> Self {
        Self {
            phase: "Pending".to_string(),
            last_sync_time: None,
            last_successful_sync: None,
            message: None,
            sync_attempts: Some(0),
            version: None,
        }
    }
}
