//! Combined RaftStorage implementation for Adaptor pattern
//!
//! This implements the older RaftStorage trait (combining log + state machine)
//! which is then wrapped by openraft::storage::Adaptor to provide v2 API compatibility.

use super::state_machine::{SecretonStateMachine, StateMachineResponse};
use super::types::{Entry, LogId, NodeId, SecretonTypeConfig, Vote};
use crate::{QueryParams, VaultEntry};
use openraft::storage::{LogState, RaftLogReader, RaftSnapshotBuilder, RaftStorage, Snapshot};
use openraft::{ErrorSubject, ErrorVerb, StorageError, StoredMembership};
use std::fmt::Debug;
use std::ops::RangeBounds;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Combined Raft storage implementing the old RaftStorage trait
/// This will be wrapped by Adaptor to provide v2 API
#[derive(Clone)]
pub struct SecretonRaftStorage {
    /// Last purged log ID
    last_purged_log_id: Arc<RwLock<Option<LogId>>>,

    /// Committed log entries
    log: Arc<RwLock<Vec<Entry>>>,

    /// Current vote information
    vote: Arc<RwLock<Option<Vote>>>,

    /// State machine
    state_machine: Arc<RwLock<SecretonStateMachine>>,
}

impl SecretonRaftStorage {
    /// Create a new combined storage instance
    pub fn new() -> Self {
        Self {
            last_purged_log_id: Arc::new(RwLock::new(None)),
            log: Arc::new(RwLock::new(Vec::new())),
            vote: Arc::new(RwLock::new(None)),
            state_machine: Arc::new(RwLock::new(SecretonStateMachine::new())),
        }
    }

    /// Get a vault entry by its ID from the replicated state machine
    pub async fn get_entry_by_id(&self, id: Uuid) -> Option<VaultEntry> {
        let sm = self.state_machine.read().await;
        let id_index = sm.id_index.read().await;

        if let Some(path) = id_index.get(&id) {
            let data = sm.data.read().await;
            data.get(path).cloned()
        } else {
            None
        }
    }

    /// Get a vault entry by its logical path from the replicated state machine
    pub async fn get_entry_by_path(&self, path: &str) -> Option<VaultEntry> {
        let sm = self.state_machine.read().await;
        let data = sm.data.read().await;
        data.get(path).cloned()
    }

    /// List entries matching the provided query parameters
    pub async fn list_entries(&self, params: &QueryParams) -> Vec<VaultEntry> {
        let sm = self.state_machine.read().await;
        let data = sm.data.read().await;

        let mut entries: Vec<VaultEntry> = data.values().cloned().collect();

        // Filter by path prefix
        if let Some(prefix) = &params.path_prefix {
            entries.retain(|entry| entry.path.starts_with(prefix));
        }

        // Filter by minimum security level
        if let Some(min_level) = params.security_level {
            entries.retain(|entry| entry.security_level >= min_level);
        }

        // Filter by owner
        if let Some(owner_id) = &params.owner_id {
            let owner_str = owner_id.to_string();
            entries.retain(|entry| entry.owner_id == owner_str);
        }

        // Filter by tags (all tags in params must be present)
        if !params.tags.is_empty() {
            entries.retain(|entry| params.tags.iter().all(|t| entry.tags.contains(t)));
        }

        // Filter expired entries unless explicitly included
        if !params.include_expired {
            entries.retain(|entry| !entry.is_expired());
        }

        // Simple metadata filters: key must exist and stringified value must match
        if !params.metadata_filters.is_empty() {
            entries.retain(|entry| {
                if let serde_json::Value::Object(map) = &entry.metadata {
                    params.metadata_filters.iter().all(|(k, v)| {
                        map.get(k)
                            .map(|val| val.to_string().trim_matches('"') == v.as_str())
                            .unwrap_or(false)
                    })
                } else {
                    false
                }
            });
        }

        // Apply offset and limit for simple pagination
        let start: usize = params.offset.unwrap_or(0) as usize;
        let start = start.min(entries.len());

        let end = if let Some(limit) = params.limit {
            let end = start.saturating_add(limit as usize);
            end.min(entries.len())
        } else {
            entries.len()
        };

        entries[start..end].to_vec()
    }

    /// Count entries matching the provided query parameters
    pub async fn count_entries(&self, params: &QueryParams) -> u64 {
        self.list_entries(params).await.len() as u64
    }

    /// Check if a path exists in the replicated state machine
    pub async fn exists_path(&self, path: &str) -> bool {
        self.get_entry_by_path(path).await.is_some()
    }
}

impl Default for SecretonRaftStorage {
    fn default() -> Self {
        Self::new()
    }
}

// Implement RaftLogReader for log queries
impl RaftLogReader<SecretonTypeConfig> for SecretonRaftStorage {
    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + Send>(
        &mut self,
        range: RB,
    ) -> Result<Vec<Entry>, StorageError<NodeId>> {
        let log = self.log.read().await;
        let entries: Vec<Entry> = log
            .iter()
            .filter(|e| range.contains(&e.log_id.index))
            .cloned()
            .collect();
        Ok(entries)
    }
}

// Implement the old RaftStorage trait
impl RaftStorage<SecretonTypeConfig> for SecretonRaftStorage {
    type LogReader = Self;
    type SnapshotBuilder = Self;

    async fn save_vote(&mut self, vote: &Vote) -> Result<(), StorageError<NodeId>> {
        *self.vote.write().await = Some(*vote);
        Ok(())
    }

    async fn read_vote(&mut self) -> Result<Option<Vote>, StorageError<NodeId>> {
        Ok(*self.vote.read().await)
    }

    async fn get_log_state(
        &mut self,
    ) -> Result<LogState<SecretonTypeConfig>, StorageError<NodeId>> {
        let log = self.log.read().await;
        let last_purged = *self.last_purged_log_id.read().await;
        let last = log.last().map(|entry| entry.log_id);

        let last_log_id = match last {
            None => last_purged,
            Some(x) => Some(x),
        };

        Ok(LogState {
            last_purged_log_id: last_purged,
            last_log_id,
        })
    }

    async fn get_log_reader(&mut self) -> Self::LogReader {
        self.clone()
    }

    async fn append_to_log<I>(&mut self, entries: I) -> Result<(), StorageError<NodeId>>
    where
        I: IntoIterator<Item = Entry> + Send,
    {
        let mut log = self.log.write().await;
        log.extend(entries);
        Ok(())
    }

    async fn delete_conflict_logs_since(
        &mut self,
        log_id: LogId,
    ) -> Result<(), StorageError<NodeId>> {
        let mut log = self.log.write().await;
        log.retain(|entry| entry.log_id.index < log_id.index);
        Ok(())
    }

    async fn purge_logs_upto(&mut self, log_id: LogId) -> Result<(), StorageError<NodeId>> {
        {
            let mut log = self.log.write().await;
            log.retain(|entry| entry.log_id.index > log_id.index);
        }
        *self.last_purged_log_id.write().await = Some(log_id);
        Ok(())
    }

    async fn last_applied_state(
        &mut self,
    ) -> Result<
        (
            Option<LogId>,
            openraft::StoredMembership<NodeId, openraft::BasicNode>,
        ),
        StorageError<NodeId>,
    > {
        let sm = self.state_machine.read().await;
        let last_applied = *sm.last_applied_log.read().await;
        let last_membership = sm.last_membership.read().await.clone();
        Ok((last_applied, last_membership))
    }

    async fn apply_to_state_machine(
        &mut self,
        entries: &[Entry],
    ) -> Result<Vec<StateMachineResponse>, StorageError<NodeId>> {
        let mut sm = self.state_machine.write().await;
        let mut responses = Vec::new();

        for entry in entries {
            let data = match &entry.payload {
                openraft::EntryPayload::Blank => continue,
                openraft::EntryPayload::Normal(data) => data,
                openraft::EntryPayload::Membership(m) => {
                    *sm.last_membership.write().await =
                        StoredMembership::new(Some(entry.log_id), m.clone());
                    *sm.last_applied_log.write().await = Some(entry.log_id);
                    continue;
                }
            };

            // data is already StateMachineCommand
            let cmd = data.clone();

            let response = sm.apply_command(cmd).await;
            responses.push(response);
            *sm.last_applied_log.write().await = Some(entry.log_id);
        }

        Ok(responses)
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        self.clone()
    }

    async fn begin_receiving_snapshot(
        &mut self,
    ) -> Result<Box<std::io::Cursor<Vec<u8>>>, StorageError<NodeId>> {
        // Return empty cursor for snapshot reception
        Ok(Box::new(std::io::Cursor::new(Vec::new())))
    }

    async fn install_snapshot(
        &mut self,
        _meta: &openraft::SnapshotMeta<NodeId, openraft::BasicNode>,
        snapshot: Box<std::io::Cursor<Vec<u8>>>,
    ) -> Result<(), StorageError<NodeId>> {
        let sm = self.state_machine.write().await;

        // Read snapshot data from cursor
        let data = snapshot.into_inner();
        let snapshot_data: super::state_machine::StateMachineSnapshot =
            serde_json::from_slice(&data).map_err(|e| {
                StorageError::from_io_error(
                    ErrorSubject::Store,
                    ErrorVerb::Read,
                    std::io::Error::new(std::io::ErrorKind::InvalidData, e),
                )
            })?;

        // Apply snapshot
        *sm.last_applied_log.write().await = snapshot_data.last_applied_log;
        *sm.last_membership.write().await = snapshot_data.last_membership;
        *sm.data.write().await = snapshot_data.entries;
        *sm.id_index.write().await = snapshot_data.id_index;

        Ok(())
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<Snapshot<SecretonTypeConfig>>, StorageError<NodeId>> {
        let sm = self.state_machine.read().await;
        let snapshot = sm.build_snapshot().await;
        Ok(Some(snapshot))
    }
}

// Implement RaftSnapshotBuilder
impl RaftSnapshotBuilder<SecretonTypeConfig> for SecretonRaftStorage {
    async fn build_snapshot(
        &mut self,
    ) -> Result<Snapshot<SecretonTypeConfig>, StorageError<NodeId>> {
        let sm = self.state_machine.read().await;
        Ok(sm.build_snapshot().await)
    }
}
