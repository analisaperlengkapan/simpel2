//! Storage adapter for SealService
//!
//! Bridges the VaultStateStorage trait with the StorageBackend trait,
//! allowing SealService to persist vault state using the storage backend.

use anyhow::Result;
use crate::services::seal::{VaultState, VaultStateStorage};
use secreton_storage::{StorageBackend, VaultEntry, SecurityLevel};
use std::sync::Arc;

const VAULT_STATE_PATH: &str = "sys/seal/state";
const VAULT_STATE_OWNER: &str = "system";

/// Storage adapter for SealService to use StorageBackend
/// This adapter implements the VaultStateStorage trait required by SealService
/// and delegates to the underlying StorageBackend implementation.
pub struct SealStorageAdapter {
    storage: Arc<dyn StorageBackend + Send + Sync>,
}

impl SealStorageAdapter {
    /// Create new seal storage adapter
    pub fn new(storage: Arc<dyn StorageBackend + Send + Sync>) -> Self {
        Self { storage }
    }
}

#[async_trait::async_trait]
impl VaultStateStorage for SealStorageAdapter {
    async fn store_vault_state(&self, state: &VaultState) -> Result<(), String> {
        // Serialize vault state to JSON
        let json_data = serde_json::to_vec(state)
            .map_err(|e| format!("Failed to serialize vault state: {}", e))?;

        tracing::info!("Storing vault state (size: {} bytes)", json_data.len());

        let entry = VaultEntry::new(
            VAULT_STATE_PATH.to_string(),
            json_data,
            serde_json::json!({}), // No additional encryption metadata at this level
            SecurityLevel::TopSecret,
            VAULT_STATE_OWNER.to_string(),
        );

        self.storage
            .store(&entry)
            .await
            .map_err(|e| format!("Failed to store vault state: {}", e))?;

        Ok(())
    }

    async fn load_vault_state(&self) -> Result<Option<VaultState>, String> {
        tracing::debug!("Loading vault state from storage");

        let entry = self
            .storage
            .get_by_path(VAULT_STATE_PATH)
            .await
            .map_err(|e| format!("Failed to load vault state: {}", e))?;

        if let Some(entry) = entry {
            let state: VaultState = serde_json::from_slice(&entry.encrypted_data)
                .map_err(|e| format!("Failed to deserialize vault state: {}", e))?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::seal::{SealConfig, EncryptionMetadata, KdfParams};
    use secreton_storage::MemoryBackend;
    use chrono::Utc;

    fn create_test_vault_state() -> VaultState {
        VaultState {
            encrypted_master_key: vec![1, 2, 3, 4],
            seal_config: SealConfig {
                seal_type: "shamir".to_string(),
                secret_shares: 5,
                secret_threshold: 3,
                created_at: Utc::now(),
            },
            shamir_commitments: vec![],
            encryption_metadata: EncryptionMetadata {
                algorithm: "aes-256-gcm".to_string(),
                nonce: vec![0; 12],
                salt: vec![0; 16],
                kdf: "argon2id".to_string(),
                kdf_params: KdfParams {
                    memory_cost: 1024,
                    time_cost: 1,
                    parallelism: 1,
                },
            },
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_seal_storage_adapter_round_trip() {
        // Setup
        let storage = Arc::new(MemoryBackend::new());
        let adapter = SealStorageAdapter::new(storage.clone());
        let state = create_test_vault_state();

        // Test Store
        adapter.store_vault_state(&state).await.expect("Failed to store vault state");

        // Verify storage content directly
        let stored_entry = storage.get_by_path(VAULT_STATE_PATH).await.unwrap();
        assert!(stored_entry.is_some());
        let entry = stored_entry.unwrap();
        assert_eq!(entry.security_level, SecurityLevel::TopSecret);
        assert_eq!(entry.owner_id, VAULT_STATE_OWNER);

        // Test Load
        let loaded_state = adapter.load_vault_state().await.expect("Failed to load vault state");
        assert!(loaded_state.is_some());
        let loaded = loaded_state.unwrap();

        // Verify content matches
        assert_eq!(loaded.encrypted_master_key, state.encrypted_master_key);
        assert_eq!(loaded.seal_config.seal_type, state.seal_config.seal_type);
        assert_eq!(loaded.version, state.version);
    }

    #[tokio::test]
    async fn test_seal_storage_adapter_empty() {
        let storage = Arc::new(MemoryBackend::new());
        let adapter = SealStorageAdapter::new(storage);

        let loaded_state = adapter.load_vault_state().await.expect("Failed to load empty state");
        assert!(loaded_state.is_none());
    }
}
