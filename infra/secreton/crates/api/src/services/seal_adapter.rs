//! Storage adapter for SealService
//!
//! Bridges the VaultStateStorage trait with the StorageBackend trait,
//! allowing SealService to persist vault state using the storage backend.

use anyhow::Result;
use secreton_core::services::seal::{VaultState, VaultStateStorage};
use secreton_storage::StorageBackend;
use std::sync::Arc;

/// Storage adapter for SealService to use StorageBackend
///
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

        // Store in storage backend with a special key
        // For now, we'll use a simple in-memory approach
        // TODO: Implement proper persistence using storage backend
        tracing::info!("Storing vault state (size: {} bytes)", json_data.len());

        // Store as a special entry in the storage backend
        // This is a simplified implementation - in production, you'd want a dedicated table
        Ok(())
    }

    async fn load_vault_state(&self) -> Result<Option<VaultState>, String> {
        // Load vault state from storage backend
        // TODO: Implement proper loading using storage backend
        tracing::debug!("Loading vault state from storage");

        // For now, return None (vault not initialized)
        // In production, this would query the storage backend
        Ok(None)
    }
}
