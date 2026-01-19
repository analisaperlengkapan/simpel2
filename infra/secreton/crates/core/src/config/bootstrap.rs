//! Bootstrap Configuration
//!
//! Minimal configuration required to start Secreton. This includes:
//! - Storage backend (where to store encrypted data)
//! - Listeners (HTTP/gRPC endpoints)
//! - Seal configuration (how to protect the master key)
//! - Telemetry settings
//!
//! NO APPLICATION SECRETS should be in this file - they are stored
//! encrypted in the storage backend and loaded after unsealing.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Re-export SealConfig from services module for compatibility
use crate::services::seal::SealConfig;

/// Bootstrap configuration loaded from secreton.toml
#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `BootstrapConfig`.
pub struct BootstrapConfig {
    pub storage: StorageConfig,
    pub listener: ListenerConfig,
    pub seal: SealConfigBootstrap,

    #[serde(default)]
    pub telemetry: TelemetryConfig,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default = "default_log_format")]
    pub log_format: String,
}

impl BootstrapConfig {
    /// Load from TOML file
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))?;

        let config: Self =
            toml::from_str(&content).map_err(|e| anyhow::anyhow!("Failed to parse TOML: {}", e))?;

        config.validate()?;

        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        self.storage.validate()?;
        self.listener.validate()?;
        self.seal.validate()?;
        Ok(())
    }

    /// Convert to existing SealConfig for compatibility with SealService
    pub fn to_seal_config(&self) -> SealConfig {
        SealConfig {
            seal_type: self.seal.seal_type.to_string(),
            secret_shares: self.seal.get_shares(),
            secret_threshold: self.seal.get_threshold(),
            created_at: Utc::now(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "json".to_string()
}

// ================================
// Storage Configuration
// ================================

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `StorageConfig`.
pub struct StorageConfig {
    pub backend: StorageBackend,

    #[serde(flatten)]
    pub config: StorageBackendConfig,
}

impl StorageConfig {
    /// Mewakili pub `validate(`.
    pub fn validate(&self) -> anyhow::Result<()> {
        match (&self.backend, &self.config) {
            (StorageBackend::Raft, StorageBackendConfig::Raft(cfg)) => cfg.validate(),
            (StorageBackend::File, StorageBackendConfig::File(cfg)) => cfg.validate(),
            (StorageBackend::Postgres, StorageBackendConfig::Postgres(cfg)) => cfg.validate(),
            (StorageBackend::Memory, StorageBackendConfig::Memory) => Ok(()),
            _ => Err(anyhow::anyhow!("Storage backend type mismatch")),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
/// Mewakili pub `StorageBackend`.
pub enum StorageBackend {
    Raft,
    File,
    Postgres,
    Memory,
}

impl ToString for StorageBackend {
    fn to_string(&self) -> String {
        match self {
            StorageBackend::Raft => "raft".to_string(),
            StorageBackend::File => "file".to_string(),
            StorageBackend::Postgres => "postgres".to_string(),
            StorageBackend::Memory => "memory".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
/// Mewakili pub `StorageBackendConfig`.
pub enum StorageBackendConfig {
    Raft(RaftStorageConfig),
    File(FileStorageConfig),
    Postgres(PostgresStorageConfig),
    Memory,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `RaftStorageConfig`.
pub struct RaftStorageConfig {
    pub path: PathBuf,
    pub node_id: String,

    #[serde(default)]
    pub retry_join: Vec<RaftRetryJoin>,

    #[serde(default)]
    pub performance: RaftPerformanceConfig,
}

impl RaftStorageConfig {
    fn validate(&self) -> anyhow::Result<()> {
        if self.node_id.is_empty() {
            return Err(anyhow::anyhow!("Raft node_id cannot be empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `RaftRetryJoin`.
pub struct RaftRetryJoin {
    pub leader_api_addr: String,
    pub leader_ca_cert_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `RaftPerformanceConfig`.
pub struct RaftPerformanceConfig {
    #[serde(default = "default_election_timeout_ms")]
    pub election_timeout_ms: u64,

    #[serde(default = "default_heartbeat_interval_ms")]
    pub heartbeat_interval_ms: u64,

    #[serde(default = "default_snapshot_interval_secs")]
    pub snapshot_interval_secs: u64,

    #[serde(default = "default_max_appending_entries")]
    pub max_appending_entries: u64,
}

impl Default for RaftPerformanceConfig {
    fn default() -> Self {
        Self {
            election_timeout_ms: default_election_timeout_ms(),
            heartbeat_interval_ms: default_heartbeat_interval_ms(),
            snapshot_interval_secs: default_snapshot_interval_secs(),
            max_appending_entries: default_max_appending_entries(),
        }
    }
}

fn default_election_timeout_ms() -> u64 {
    1000
}
fn default_heartbeat_interval_ms() -> u64 {
    300
}
fn default_snapshot_interval_secs() -> u64 {
    120
}
fn default_max_appending_entries() -> u64 {
    64
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `FileStorageConfig`.
pub struct FileStorageConfig {
    pub path: PathBuf,

    #[serde(default = "default_sync_writes")]
    pub sync_writes: bool,
}

impl FileStorageConfig {
    fn validate(&self) -> anyhow::Result<()> {
        // Path will be created if it doesn't exist
        Ok(())
    }
}

fn default_sync_writes() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `PostgresStorageConfig`.
pub struct PostgresStorageConfig {
    // Connection URL from environment variable SECRETON_STORAGE_URL
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

impl PostgresStorageConfig {
    fn validate(&self) -> anyhow::Result<()> {
        if self.max_connections == 0 {
            return Err(anyhow::anyhow!("max_connections must be > 0"));
        }
        Ok(())
    }
}

fn default_max_connections() -> u32 {
    50
}

// ================================
// Listener Configuration
// ================================

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `ListenerConfig`.
pub struct ListenerConfig {
    pub http: HttpListenerConfig,

    #[serde(default)]
    pub grpc: GrpcListenerConfig,
}

impl ListenerConfig {
    fn validate(&self) -> anyhow::Result<()> {
        self.http.validate()?;
        self.grpc.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `HttpListenerConfig`.
pub struct HttpListenerConfig {
    pub address: String,

    #[serde(default)]
    pub tls_enabled: bool,

    #[serde(default)]
    pub tls: Option<TlsConfig>,
}

impl HttpListenerConfig {
    fn validate(&self) -> anyhow::Result<()> {
        if self.address.is_empty() {
            return Err(anyhow::anyhow!("HTTP address cannot be empty"));
        }

        if self.tls_enabled && self.tls.is_none() {
            return Err(anyhow::anyhow!("TLS enabled but no TLS config provided"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `TlsConfig`.
pub struct TlsConfig {
    pub cert_file: PathBuf,
    pub key_file: PathBuf,

    #[serde(default = "default_tls_min_version")]
    pub min_version: String,

    #[serde(default)]
    pub cipher_suites: Vec<String>,
}

fn default_tls_min_version() -> String {
    "1.3".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `GrpcListenerConfig`.
pub struct GrpcListenerConfig {
    #[serde(default = "default_grpc_enabled")]
    pub enabled: bool,

    #[serde(default = "default_grpc_address")]
    pub address: String,
}

impl GrpcListenerConfig {
    fn validate(&self) -> anyhow::Result<()> {
        if self.enabled && self.address.is_empty() {
            return Err(anyhow::anyhow!("gRPC enabled but address is empty"));
        }
        Ok(())
    }
}

impl Default for GrpcListenerConfig {
    fn default() -> Self {
        Self {
            enabled: default_grpc_enabled(),
            address: default_grpc_address(),
        }
    }
}

fn default_grpc_enabled() -> bool {
    true
}
fn default_grpc_address() -> String {
    "0.0.0.0:8201".to_string()
}

// ================================
// Seal Configuration
// ================================

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `SealConfigBootstrap`.
pub struct SealConfigBootstrap {
    #[serde(rename = "type")]
    pub seal_type: SealType,

    #[serde(flatten)]
    pub config: SealTypeConfig,
}

impl SealConfigBootstrap {
    fn validate(&self) -> anyhow::Result<()> {
        match (&self.seal_type, &self.config) {
            (SealType::Shamir, SealTypeConfig::Shamir(cfg)) => cfg.validate(),
            (SealType::AwsKms, SealTypeConfig::AwsKms(cfg)) => cfg.validate(),
            (SealType::GcpKms, SealTypeConfig::GcpKms(cfg)) => cfg.validate(),
            (SealType::AzureKv, SealTypeConfig::AzureKv(cfg)) => cfg.validate(),
            _ => Err(anyhow::anyhow!("Seal type mismatch")),
        }
    }

    /// Mewakili pub `get_shares(`.
    pub fn get_shares(&self) -> usize {
        match &self.config {
            SealTypeConfig::Shamir(cfg) => cfg.shares,
            _ => 5, // Default for auto-unseal
        }
    }

    /// Mewakili pub `get_threshold(`.
    pub fn get_threshold(&self) -> usize {
        match &self.config {
            SealTypeConfig::Shamir(cfg) => cfg.threshold,
            _ => 3, // Default for auto-unseal
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
/// Mewakili pub `SealType`.
pub enum SealType {
    Shamir,
    AwsKms,
    GcpKms,
    AzureKv,
}

impl ToString for SealType {
    fn to_string(&self) -> String {
        match self {
            SealType::Shamir => "shamir".to_string(),
            SealType::AwsKms => "aws-kms".to_string(),
            SealType::GcpKms => "gcp-kms".to_string(),
            SealType::AzureKv => "azure-kv".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
/// Mewakili pub `SealTypeConfig`.
pub enum SealTypeConfig {
    Shamir(ShamirSealConfig),
    AwsKms(AwsKmsSealConfig),
    GcpKms(GcpKmsSealConfig),
    AzureKv(AzureKvSealConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `ShamirSealConfig`.
pub struct ShamirSealConfig {
    #[serde(default = "default_shamir_shares")]
    pub shares: usize,

    #[serde(default = "default_shamir_threshold")]
    pub threshold: usize,
}

impl ShamirSealConfig {
    fn validate(&self) -> anyhow::Result<()> {
        if self.threshold > self.shares {
            return Err(anyhow::anyhow!(
                "Threshold ({}) cannot be greater than shares ({})",
                self.threshold,
                self.shares
            ));
        }
        if self.threshold < 1 {
            return Err(anyhow::anyhow!("Threshold must be at least 1"));
        }
        if self.shares < 1 {
            return Err(anyhow::anyhow!("Shares must be at least 1"));
        }
        Ok(())
    }
}

fn default_shamir_shares() -> usize {
    5
}
fn default_shamir_threshold() -> usize {
    3
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `AwsKmsSealConfig`.
pub struct AwsKmsSealConfig {
    pub region: String,
    pub kms_key_id: String,
}

impl AwsKmsSealConfig {
    fn validate(&self) -> anyhow::Result<()> {
        if self.region.is_empty() {
            return Err(anyhow::anyhow!("AWS region cannot be empty"));
        }
        if self.kms_key_id.is_empty() {
            return Err(anyhow::anyhow!("KMS key ID cannot be empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `GcpKmsSealConfig`.
pub struct GcpKmsSealConfig {
    pub project: String,
    pub region: String,
    pub key_ring: String,
    pub crypto_key: String,
}

impl GcpKmsSealConfig {
    fn validate(&self) -> anyhow::Result<()> {
        if self.project.is_empty() {
            return Err(anyhow::anyhow!("GCP project cannot be empty"));
        }
        if self.key_ring.is_empty() {
            return Err(anyhow::anyhow!("GCP key ring cannot be empty"));
        }
        if self.crypto_key.is_empty() {
            return Err(anyhow::anyhow!("GCP crypto key cannot be empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `AzureKvSealConfig`.
pub struct AzureKvSealConfig {
    pub vault_name: String,
    pub key_name: String,
}

impl AzureKvSealConfig {
    fn validate(&self) -> anyhow::Result<()> {
        if self.vault_name.is_empty() {
            return Err(anyhow::anyhow!("Azure vault name cannot be empty"));
        }
        if self.key_name.is_empty() {
            return Err(anyhow::anyhow!("Azure key name cannot be empty"));
        }
        Ok(())
    }
}

// ================================
// Telemetry Configuration
// ================================

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Mewakili pub `TelemetryConfig`.
pub struct TelemetryConfig {
    #[serde(default = "default_prometheus_enabled")]
    pub prometheus_enabled: bool,

    #[serde(default = "default_metrics_path")]
    pub metrics_path: String,

    #[serde(default)]
    pub disable_hostname: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            prometheus_enabled: default_prometheus_enabled(),
            metrics_path: default_metrics_path(),
            disable_hostname: false,
        }
    }
}

fn default_prometheus_enabled() -> bool {
    true
}
fn default_metrics_path() -> String {
    "/metrics".to_string()
}

// ================================
// Tests
// ================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shamir_seal_validation() {
        let config = ShamirSealConfig {
            shares: 5,
            threshold: 3,
        };
        assert!(config.validate().is_ok());

        let bad_config = ShamirSealConfig {
            shares: 3,
            threshold: 5, // threshold > shares
        };
        assert!(bad_config.validate().is_err());
    }

    #[test]
    fn test_bootstrap_config_parse() {
        let toml = r#"
[storage]
backend = "raft"
path = "/var/lib/secreton/raft"
node_id = "node1"

[listener.http]
address = "0.0.0.0:8200"
tls_enabled = false

[seal]
type = "shamir"
shares = 5
threshold = 3
"#;

        let config: BootstrapConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.storage.backend, StorageBackend::Raft);
        assert_eq!(config.seal.seal_type, SealType::Shamir);
        assert!(config.validate().is_ok());
    }
}
