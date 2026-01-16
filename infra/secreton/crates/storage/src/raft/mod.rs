//! Raft storage backend implementation

use crate::{
    HealthStatus, QueryParams, StorageBackend, StorageError, StorageResult, StorageStats,
    StorageTransaction, VaultEntry,
};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub use self::cluster::{RaftCluster, RaftClusterConfig};
pub use self::node::{Raft, SecretonRaftStorage};
pub use self::state_machine::{
    RaftStatus, SecretonStateMachine, StateMachineCommand, StateMachineResponse,
};

mod cluster;
mod network;
mod node;
mod state_machine;
mod store;

#[async_trait]
impl StorageBackend for RaftCluster {
    async fn store(&self, entry: &VaultEntry) -> StorageResult<()> {
        self.propose_command(StateMachineCommand::Store(entry.clone()))
            .await?;
        Ok(())
    }

    async fn get_by_id(&self, id: Uuid) -> StorageResult<Option<VaultEntry>> {
        // Read operations can be local if linearizable reads are supported,
        // or through the leader for strict consistency.
        // For now, we query the state machine directly.
        let state_machine = self.state_machine.read().await;
        Ok(state_machine.get_by_id(id))
    }

    async fn get_by_path(&self, path: &str) -> StorageResult<Option<VaultEntry>> {
        let state_machine = self.state_machine.read().await;
        Ok(state_machine.get_by_path(path))
    }

    async fn update(&self, entry: &VaultEntry) -> StorageResult<()> {
        self.propose_command(StateMachineCommand::Update(entry.clone()))
            .await?;
        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> StorageResult<bool> {
        let response = self
            .propose_command(StateMachineCommand::DeleteById(id))
            .await?;
        match response {
            StateMachineResponse::Deleted(deleted) => Ok(deleted),
            _ => Err(StorageError::BackendError {
                backend: "raft".to_string(),
                message: "Unexpected response type".to_string(),
            }),
        }
    }

    async fn delete_by_path(&self, path: &str) -> StorageResult<bool> {
        let response = self
            .propose_command(StateMachineCommand::DeleteByPath(path.to_string()))
            .await?;
        match response {
            StateMachineResponse::Deleted(deleted) => Ok(deleted),
            _ => Err(StorageError::BackendError {
                backend: "raft".to_string(),
                message: "Unexpected response type".to_string(),
            }),
        }
    }

    async fn list(&self, params: &QueryParams) -> StorageResult<Vec<VaultEntry>> {
        let state_machine = self.state_machine.read().await;
        Ok(state_machine.list(params))
    }

    async fn count(&self, params: &QueryParams) -> StorageResult<u64> {
        let state_machine = self.state_machine.read().await;
        Ok(state_machine.count(params))
    }

    async fn exists(&self, path: &str) -> StorageResult<bool> {
        let state_machine = self.state_machine.read().await;
        Ok(state_machine.exists(path))
    }

    async fn begin_transaction(&self) -> StorageResult<Box<dyn StorageTransaction>> {
        // Raft operations are atomic per command, but multi-command transactions
        // require more complex support not yet implemented.
        Err(StorageError::TransactionNotSupported {
            backend: "raft".to_string(),
        })
    }

    async fn health_check(&self) -> StorageResult<HealthStatus> {
        let metrics = self.raft.metrics().borrow().clone();
        let is_healthy = metrics.state != openraft::ServerState::Shutdown;

        Ok(HealthStatus {
            is_healthy,
            response_time_ms: 0.0, // TODO: Measure ping time
            connections_active: 0,
            connections_idle: 0,
            last_error: None,
            uptime_seconds: 0, // TODO: Track uptime
        })
    }

    async fn get_stats(&self) -> StorageResult<StorageStats> {
        let state_machine = self.state_machine.read().await;
        let metrics = self.raft.metrics().borrow().clone();

        Ok(StorageStats {
            backend_type: "raft".to_string(),
            total_entries: state_machine.count(&QueryParams::default()),
            total_size_bytes: 0, // TODO: Estimate size
            average_entry_size: 0.0,
            entries_by_security_level: HashMap::new(),
            entries_created_today: 0,
            entries_updated_today: 0,
            expired_entries: 0,
            last_backup: state_machine.last_snapshot_time,
            metadata: serde_json::json!({
                "role": format!("{:?}", metrics.state),
                "term": metrics.current_term,
                "last_log_index": metrics.last_log_index,
                "leader": metrics.current_leader,
            }),
        })
    }

    async fn migrate(&self) -> StorageResult<()> {
        // Raft state machine handles schema evolution internally via snapshots
        Ok(())
    }

    async fn delete_expired(&self) -> StorageResult<u64> {
        // For Raft, we propose a command to delete expired entries
        // This ensures all nodes execute it deterministically

        // Note: Ideally we would have a specific command for this.
        // For now, we iterate locally and issue individual delete commands.
        // This is not atomic but works. A better approach would be `StateMachineCommand::DeleteExpired`.

        let state_machine = self.state_machine.read().await;
        let expired_ids: Vec<Uuid> = state_machine
            .list(&QueryParams::default())
            .into_iter()
            .filter(|e| e.is_expired())
            .map(|e| e.id)
            .collect();

        // Drop read lock before proposing commands
        drop(state_machine);

        let mut count = 0;
        for id in expired_ids {
            match self.delete_by_id(id).await {
                Ok(true) => count += 1,
                _ => {}
            }
        }

        Ok(count)
    }

    async fn compact(&self) -> StorageResult<()> {
        // Trigger a snapshot to compact the Raft log
        self.raft
            .trigger_snapshot()
            .await
            .map_err(|e| StorageError::BackendError {
                backend: "raft".to_string(),
                message: format!("Failed to trigger snapshot: {}", e),
            })?;

        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
