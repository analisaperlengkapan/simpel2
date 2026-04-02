//! Sealed Master Key Storage Operations
//!
//! This module provides storage operations for sealed master keys used in auto-unseal.
//! Sealed master keys are encrypted by external KMS providers (AWS KMS, GCP KMS, Azure Key Secreton, Transit)
//! and stored in PostgreSQL for retrieval during auto-unseal operations.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

/// Auto-unseal provider type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderType {
    /// AWS KMS
    AwsKms,
    /// GCP KMS
    GcpKms,
    /// Azure Key Secreton
    AzureKv,
    /// Transit engine (another Secreton instance)
    Transit,
}

impl ProviderType {
    pub fn as_str(&self) -> &str {
        match self {
            ProviderType::AwsKms => "aws-kms",
            ProviderType::GcpKms => "gcp-kms",
            ProviderType::AzureKv => "azure-kv",
            ProviderType::Transit => "transit",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "aws-kms" => Ok(ProviderType::AwsKms),
            "gcp-kms" => Ok(ProviderType::GcpKms),
            "azure-kv" => Ok(ProviderType::AzureKv),
            "transit" => Ok(ProviderType::Transit),
            _ => Err(format!("Invalid provider type: {}", s)),
        }
    }
}

/// Sealed master key entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedMasterKey {
    /// Unique identifier
    pub id: Uuid,

    /// Provider type
    pub provider_type: ProviderType,

    /// Provider-specific key identifier (KMS key ARN, key name, etc.)
    pub provider_key_id: String,

    /// Provider region (for AWS/GCP/Azure)
    pub provider_region: Option<String>,

    /// Provider endpoint (for custom endpoints or Transit)
    pub provider_endpoint: Option<String>,

    /// Encrypted master key (encrypted by the KMS provider)
    pub encrypted_master_key: Vec<u8>,

    /// SHA-256 checksum of encrypted data
    pub checksum: String,

    /// Whether this is the active sealed key
    pub is_active: bool,

    /// Version number for key rotation
    pub version: i32,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl SealedMasterKey {
    /// Create a new sealed master key entry
    pub fn new(
        provider_type: ProviderType,
        provider_key_id: String,
        provider_region: Option<String>,
        provider_endpoint: Option<String>,
        encrypted_master_key: Vec<u8>,
    ) -> Self {
        let checksum = Self::calculate_checksum(&encrypted_master_key);

        Self {
            id: Uuid::new_v4(),
            provider_type,
            provider_key_id,
            provider_region,
            provider_endpoint,
            encrypted_master_key,
            checksum,
            is_active: false,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }

    /// Calculate SHA-256 checksum of encrypted data
    fn calculate_checksum(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Verify checksum integrity
    pub fn verify_checksum(&self) -> bool {
        let calculated = Self::calculate_checksum(&self.encrypted_master_key);
        calculated == self.checksum
    }
}

/// Sealed master key storage trait
#[async_trait]
pub trait SealedKeyStorage: Send + Sync {
    /// Store a sealed master key
    async fn store_sealed_key(&self, key: &SealedMasterKey) -> Result<(), String>;

    /// Get the active sealed master key
    async fn get_active_sealed_key(&self) -> Result<Option<SealedMasterKey>, String>;

    /// Get a sealed master key by ID
    async fn get_sealed_key_by_id(&self, id: Uuid) -> Result<Option<SealedMasterKey>, String>;

    /// List all sealed master keys
    async fn list_sealed_keys(&self) -> Result<Vec<SealedMasterKey>, String>;

    /// Set a sealed master key as active (deactivates all others)
    async fn set_active_sealed_key(&self, id: Uuid) -> Result<(), String>;

    /// Delete a sealed master key
    async fn delete_sealed_key(&self, id: Uuid) -> Result<(), String>;

    /// Get sealed master keys by provider type
    async fn get_sealed_keys_by_provider(
        &self,
        provider_type: ProviderType,
    ) -> Result<Vec<SealedMasterKey>, String>;
}

/// PostgreSQL implementation of sealed key storage
pub struct PostgresSealedKeyStorage {
    pool: Arc<deadpool_postgres::Pool>,
}

impl PostgresSealedKeyStorage {
    /// Create a new PostgreSQL sealed key storage
    pub fn new(pool: Arc<deadpool_postgres::Pool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SealedKeyStorage for PostgresSealedKeyStorage {
    async fn store_sealed_key(&self, key: &SealedMasterKey) -> Result<(), String> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let query = r#"
            INSERT INTO sealed_master_keys (
                id, provider_type, provider_key_id, provider_region, provider_endpoint,
                encrypted_master_key, checksum, is_active, version, created_at, updated_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (id) DO UPDATE SET
                provider_type = EXCLUDED.provider_type,
                provider_key_id = EXCLUDED.provider_key_id,
                provider_region = EXCLUDED.provider_region,
                provider_endpoint = EXCLUDED.provider_endpoint,
                encrypted_master_key = EXCLUDED.encrypted_master_key,
                checksum = EXCLUDED.checksum,
                is_active = EXCLUDED.is_active,
                version = EXCLUDED.version,
                updated_at = EXCLUDED.updated_at,
                metadata = EXCLUDED.metadata
        "#;

        client
            .execute(
                query,
                &[
                    &key.id,
                    &key.provider_type.as_str(),
                    &key.provider_key_id,
                    &key.provider_region,
                    &key.provider_endpoint,
                    &key.encrypted_master_key,
                    &key.checksum,
                    &key.is_active,
                    &key.version,
                    &key.created_at,
                    &key.updated_at,
                    &key.metadata,
                ],
            )
            .await
            .map_err(|e| format!("Failed to store sealed key: {}", e))?;

        Ok(())
    }

    async fn get_active_sealed_key(&self) -> Result<Option<SealedMasterKey>, String> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let query = r#"
            SELECT id, provider_type, provider_key_id, provider_region, provider_endpoint,
                   encrypted_master_key, checksum, is_active, version, created_at, updated_at, metadata
            FROM sealed_master_keys
            WHERE is_active = true
            LIMIT 1
        "#;

        let row = client
            .query_opt(query, &[])
            .await
            .map_err(|e| format!("Failed to query active sealed key: {}", e))?;

        match row {
            Some(row) => Ok(Some(Self::row_to_sealed_key(row)?)),
            None => Ok(None),
        }
    }

    async fn get_sealed_key_by_id(&self, id: Uuid) -> Result<Option<SealedMasterKey>, String> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let query = r#"
            SELECT id, provider_type, provider_key_id, provider_region, provider_endpoint,
                   encrypted_master_key, checksum, is_active, version, created_at, updated_at, metadata
            FROM sealed_master_keys
            WHERE id = $1
        "#;

        let row = client
            .query_opt(query, &[&id])
            .await
            .map_err(|e| format!("Failed to query sealed key by ID: {}", e))?;

        match row {
            Some(row) => Ok(Some(Self::row_to_sealed_key(row)?)),
            None => Ok(None),
        }
    }

    async fn list_sealed_keys(&self) -> Result<Vec<SealedMasterKey>, String> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let query = r#"
            SELECT id, provider_type, provider_key_id, provider_region, provider_endpoint,
                   encrypted_master_key, checksum, is_active, version, created_at, updated_at, metadata
            FROM sealed_master_keys
            ORDER BY created_at DESC
        "#;

        let rows = client
            .query(query, &[])
            .await
            .map_err(|e| format!("Failed to list sealed keys: {}", e))?;

        rows.into_iter()
            .map(Self::row_to_sealed_key)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn set_active_sealed_key(&self, id: Uuid) -> Result<(), String> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let transaction = client
            .transaction()
            .await
            .map_err(|e| format!("Failed to start transaction: {}", e))?;

        // Deactivate all sealed keys
        transaction
            .execute(
                "UPDATE sealed_master_keys SET is_active = false, updated_at = NOW()",
                &[],
            )
            .await
            .map_err(|e| format!("Failed to deactivate sealed keys: {}", e))?;

        // Activate the specified key
        let rows_affected = transaction
            .execute(
                "UPDATE sealed_master_keys SET is_active = true, updated_at = NOW() WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| format!("Failed to activate sealed key: {}", e))?;

        if rows_affected == 0 {
            return Err(format!("Sealed key with ID {} not found", id));
        }

        transaction
            .commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(())
    }

    async fn delete_sealed_key(&self, id: Uuid) -> Result<(), String> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let rows_affected = client
            .execute("DELETE FROM sealed_master_keys WHERE id = $1", &[&id])
            .await
            .map_err(|e| format!("Failed to delete sealed key: {}", e))?;

        if rows_affected == 0 {
            return Err(format!("Sealed key with ID {} not found", id));
        }

        Ok(())
    }

    async fn get_sealed_keys_by_provider(
        &self,
        provider_type: ProviderType,
    ) -> Result<Vec<SealedMasterKey>, String> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let query = r#"
            SELECT id, provider_type, provider_key_id, provider_region, provider_endpoint,
                   encrypted_master_key, checksum, is_active, version, created_at, updated_at, metadata
            FROM sealed_master_keys
            WHERE provider_type = $1
            ORDER BY created_at DESC
        "#;

        let rows = client
            .query(query, &[&provider_type.as_str()])
            .await
            .map_err(|e| format!("Failed to query sealed keys by provider: {}", e))?;

        rows.into_iter()
            .map(Self::row_to_sealed_key)
            .collect::<Result<Vec<_>, _>>()
    }
}

impl PostgresSealedKeyStorage {
    /// Convert a database row to a SealedMasterKey
    fn row_to_sealed_key(row: tokio_postgres::Row) -> Result<SealedMasterKey, String> {
        let provider_type_str: String = row
            .try_get("provider_type")
            .map_err(|e| format!("Failed to get provider_type: {}", e))?;

        let provider_type = ProviderType::from_str(&provider_type_str)?;

        Ok(SealedMasterKey {
            id: row
                .try_get("id")
                .map_err(|e| format!("Failed to get id: {}", e))?,
            provider_type,
            provider_key_id: row
                .try_get("provider_key_id")
                .map_err(|e| format!("Failed to get provider_key_id: {}", e))?,
            provider_region: row
                .try_get("provider_region")
                .map_err(|e| format!("Failed to get provider_region: {}", e))?,
            provider_endpoint: row
                .try_get("provider_endpoint")
                .map_err(|e| format!("Failed to get provider_endpoint: {}", e))?,
            encrypted_master_key: row
                .try_get("encrypted_master_key")
                .map_err(|e| format!("Failed to get encrypted_master_key: {}", e))?,
            checksum: row
                .try_get("checksum")
                .map_err(|e| format!("Failed to get checksum: {}", e))?,
            is_active: row
                .try_get("is_active")
                .map_err(|e| format!("Failed to get is_active: {}", e))?,
            version: row
                .try_get("version")
                .map_err(|e| format!("Failed to get version: {}", e))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("Failed to get created_at: {}", e))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| format!("Failed to get updated_at: {}", e))?,
            metadata: row
                .try_get("metadata")
                .map_err(|e| format!("Failed to get metadata: {}", e))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_conversion() {
        assert_eq!(ProviderType::AwsKms.as_str(), "aws-kms");
        assert_eq!(ProviderType::GcpKms.as_str(), "gcp-kms");
        assert_eq!(ProviderType::AzureKv.as_str(), "azure-kv");
        assert_eq!(ProviderType::Transit.as_str(), "transit");

        assert_eq!(
            ProviderType::from_str("aws-kms").unwrap(),
            ProviderType::AwsKms
        );
        assert_eq!(
            ProviderType::from_str("gcp-kms").unwrap(),
            ProviderType::GcpKms
        );
        assert_eq!(
            ProviderType::from_str("azure-kv").unwrap(),
            ProviderType::AzureKv
        );
        assert_eq!(
            ProviderType::from_str("transit").unwrap(),
            ProviderType::Transit
        );

        assert!(ProviderType::from_str("invalid").is_err());
    }

    #[test]
    fn test_sealed_master_key_checksum() {
        let encrypted_data = vec![1, 2, 3, 4, 5];
        let key = SealedMasterKey::new(
            ProviderType::AwsKms,
            "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012"
                .to_string(),
            Some("us-east-1".to_string()),
            None,
            encrypted_data.clone(),
        );

        assert!(key.verify_checksum());

        // Modify encrypted data and verify checksum fails
        let mut modified_key = key.clone();
        modified_key.encrypted_master_key = vec![5, 4, 3, 2, 1];
        assert!(!modified_key.verify_checksum());
    }

    #[test]
    fn test_sealed_master_key_creation() {
        let key = SealedMasterKey::new(
            ProviderType::Transit,
            "auto-unseal-key".to_string(),
            None,
            Some("https://secreton.internal:8200".to_string()),
            vec![1, 2, 3, 4, 5],
        );

        assert_eq!(key.provider_type, ProviderType::Transit);
        assert_eq!(key.provider_key_id, "auto-unseal-key");
        assert_eq!(key.provider_region, None);
        assert_eq!(
            key.provider_endpoint,
            Some("https://secreton.internal:8200".to_string())
        );
        assert_eq!(key.encrypted_master_key, vec![1, 2, 3, 4, 5]);
        assert!(!key.is_active);
        assert_eq!(key.version, 1);
    }
}
