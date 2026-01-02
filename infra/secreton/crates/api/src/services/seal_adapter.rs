//! Storage adapter for SealService
//!
//! Bridges the VaultStateStorage trait with the StorageBackend trait,
//! allowing SealService to persist vault state using the storage backend.

use anyhow::Result;
use secreton_core::services::seal::{VaultState, VaultStateStorage};
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
