//! OpenRaft State Machine Implementation
//!
//! Implements the state machine for Secreton's distributed storage using OpenRaft.

use super::types::{LogId, NodeId, SecretonTypeConfig};
use crate::SecretEntry;
use openraft::storage::RaftSnapshotBuilder;
use openraft::{BasicNode, SnapshotMeta, StorageError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Commands that can be applied to the state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateMachineCommand {
    /// Store a new engine entry
    Store(SecretEntry),

    /// Update an existing engine entry
    Update(SecretEntry),

    /// Delete a engine entry by ID
    Delete(Uuid),

    /// Delete a engine entry by path
    DeleteByPath(String),

    /// Delete expired entries
    DeleteExpired(chrono::DateTime<chrono::Utc>),
}

/// Response from state machine operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateMachineResponse {
    /// Operation succeeded
    Success,

    /// Entry was stored
    Stored(Uuid),

    /// Entry was updated
    Updated(Uuid),

    /// Entry was deleted
    Deleted(bool),

    /// Expired entries were deleted
    DeletedExpired(u64),

    /// Error occurred
    Error(String),
}

/// Snapshot of the state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachineSnapshot {
    /// Last applied log index
    pub last_applied_log: Option<LogId>,

    /// Last membership configuration
    pub last_membership: openraft::StoredMembership<NodeId, BasicNode>,

    /// All engine entries
    pub entries: HashMap<String, SecretEntry>,

    /// ID to path index
    pub id_index: HashMap<Uuid, String>,
}

/// OpenRaft state machine for Secreton storage
pub struct SecretonStateMachine {
    /// Last applied log ID
    pub last_applied_log: Arc<RwLock<Option<LogId>>>,

    /// Last membership configuration
    pub last_membership: Arc<RwLock<openraft::StoredMembership<NodeId, BasicNode>>>,

    /// Engine entries storage (path -> entry)
    pub data: Arc<RwLock<HashMap<String, SecretEntry>>>,

    /// ID to path index
    pub id_index: Arc<RwLock<HashMap<Uuid, String>>>,
}

impl SecretonStateMachine {
    /// Create a new state machine
    pub fn new() -> Self {
        Self {
            last_applied_log: Arc::new(RwLock::new(None)),
            last_membership: Arc::new(RwLock::new(openraft::StoredMembership::default())),
            data: Arc::new(RwLock::new(HashMap::new())),
            id_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Apply a command to the state machine (made public for combined storage)
    pub async fn apply_command(&mut self, cmd: StateMachineCommand) -> StateMachineResponse {
        match cmd {
            StateMachineCommand::Store(entry) => {
                let mut data = self.data.write().await;
                let mut id_index = self.id_index.write().await;

                let path = entry.path.clone();
                let id = entry.id;

                data.insert(path.clone(), entry);
                id_index.insert(id, path);

                StateMachineResponse::Stored(id)
            }

            StateMachineCommand::Update(entry) => {
                let mut data = self.data.write().await;

                if data.contains_key(&entry.path) {
                    data.insert(entry.path.clone(), entry.clone());
                    StateMachineResponse::Updated(entry.id)
                } else {
                    StateMachineResponse::Error("Entry not found".to_string())
                }
            }

            StateMachineCommand::Delete(id) => {
                let mut data = self.data.write().await;
                let mut id_index = self.id_index.write().await;

                if let Some(path) = id_index.remove(&id) {
                    data.remove(&path);
                    StateMachineResponse::Deleted(true)
                } else {
                    StateMachineResponse::Deleted(false)
                }
            }

            StateMachineCommand::DeleteByPath(path) => {
                let mut data = self.data.write().await;
                let mut id_index = self.id_index.write().await;

                if let Some(entry) = data.remove(&path) {
                    id_index.remove(&entry.id);
                    StateMachineResponse::Deleted(true)
                } else {
                    StateMachineResponse::Deleted(false)
                }
            }

            StateMachineCommand::DeleteExpired(timestamp) => {
                let mut data = self.data.write().await;
                let mut id_index = self.id_index.write().await;
                let mut count = 0;

                let keys_to_remove: Vec<String> = data
                    .iter()
                    .filter(|(_, entry)| {
                        if let Some(expires_at) = entry.expires_at {
                            expires_at < timestamp
                        } else {
                            false
                        }
                    })
                    .map(|(k, _)| k.clone())
                    .collect();

                for key in keys_to_remove {
                    if let Some(entry) = data.remove(&key) {
                        id_index.remove(&entry.id);
                        count += 1;
                    }
                }

                StateMachineResponse::DeletedExpired(count)
            }
        }
    }

    /// Get current snapshot
    async fn get_snapshot(&self) -> StateMachineSnapshot {
        let last_applied_log = *self.last_applied_log.read().await;
        let last_membership = self.last_membership.read().await.clone();
        let entries = self.data.read().await.clone();
        let id_index = self.id_index.read().await.clone();

        StateMachineSnapshot {
            last_applied_log,
            last_membership,
            entries,
            id_index,
        }
    }

    /// Restore from snapshot
    pub async fn restore_snapshot(&self, snapshot: StateMachineSnapshot) {
        *self.last_applied_log.write().await = snapshot.last_applied_log;
        *self.last_membership.write().await = snapshot.last_membership;
        *self.data.write().await = snapshot.entries;
        *self.id_index.write().await = snapshot.id_index;
    }

    /// Build a snapshot for OpenRaft
    pub async fn build_snapshot(&self) -> openraft::Snapshot<SecretonTypeConfig> {
        let data = self.get_snapshot().await;
        let snapshot_bytes = serde_json::to_vec(&data).unwrap_or_default();

        let meta = openraft::SnapshotMeta {
            last_log_id: data.last_applied_log,
            last_membership: data.last_membership,
            snapshot_id: uuid::Uuid::new_v4().to_string(),
        };

        openraft::Snapshot {
            meta,
            snapshot: Box::new(std::io::Cursor::new(snapshot_bytes)),
        }
    }
}

impl Default for SecretonStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

// RaftStateMachine implementation removed - functionality moved to combined_storage.rs

// Implement Clone for SecretonStateMachine
impl Clone for SecretonStateMachine {
    fn clone(&self) -> Self {
        Self {
            last_applied_log: Arc::clone(&self.last_applied_log),
            last_membership: Arc::clone(&self.last_membership),
            data: Arc::clone(&self.data),
            id_index: Arc::clone(&self.id_index),
        }
    }
}

impl RaftSnapshotBuilder<SecretonTypeConfig> for SecretonStateMachine {
    async fn build_snapshot(
        &mut self,
    ) -> Result<openraft::Snapshot<SecretonTypeConfig>, StorageError<NodeId>> {
        let snapshot = self.get_snapshot().await;

        // Serialize snapshot using JSON
        let data = serde_json::to_vec(&snapshot).map_err(|e| {
            openraft::StorageError::from_io_error(
                openraft::ErrorSubject::Snapshot(None),
                openraft::ErrorVerb::Write,
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()),
            )
        })?;

        let last_applied_log = snapshot.last_applied_log;
        let last_membership = snapshot.last_membership;

        let snapshot_id = format!(
            "snapshot-{}-{}",
            last_applied_log.map(|l| l.index).unwrap_or(0),
            chrono::Utc::now().timestamp()
        );

        let meta = SnapshotMeta {
            last_log_id: last_applied_log,
            last_membership,
            snapshot_id,
        };

        Ok(openraft::Snapshot {
            meta,
            snapshot: Box::new(std::io::Cursor::new(data)),
        })
    }
}
