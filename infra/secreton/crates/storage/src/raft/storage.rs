//! OpenRaft Storage Implementation
//!
//! Implements the storage layer for OpenRaft consensus.

use super::state_machine::{SecretonStateMachine, StateMachineSnapshot};
use super::types::{Entry, LogId, NodeId, SecretonTypeConfig, Vote};
use async_trait::async_trait;
use openraft::storage::{LogState, RaftLogStorage, RaftStateMachine, Snapshot};
use openraft::{
    ErrorSubject, ErrorVerb, RaftLogReader, RaftSnapshotBuilder, SnapshotMeta, StorageError,
    StoredMembership,
};
use std::fmt::Debug;
use std::io::Cursor;
use std::ops::RangeBounds;
use std::sync::Arc;
use tokio::sync::RwLock;

/// OpenRaft storage implementation for Secreton
pub struct SecretonStorage {
    /// Last purged log ID
    last_purged_log_id: Arc<RwLock<Option<LogId>>>,

    /// Committed log entries
    log: Arc<RwLock<Vec<Entry>>>,

    /// Current vote information
    vote: Arc<RwLock<Option<Vote>>>,

    /// Snapshot storage
    snapshot: Arc<RwLock<Option<Snapshot<SecretonTypeConfig>>>>,

    /// State machine
    state_machine: Arc<RwLock<SecretonStateMachine>>,
}

impl SecretonStorage {
    /// Create a new storage instance
    pub fn new() -> Self {
        Self {
            last_purged_log_id: Arc::new(RwLock::new(None)),
            log: Arc::new(RwLock::new(Vec::new())),
            vote: Arc::new(RwLock::new(None)),
            snapshot: Arc::new(RwLock::new(None)),
            state_machine: Arc::new(RwLock::new(SecretonStateMachine::new())),
        }
    }

    /// Get log entry by index
    async fn get_log_entry(&self, index: u64) -> Option<Entry> {
        let log = self.log.read().await;
        log.iter().find(|e| e.log_id.index == index).cloned()
    }
}

impl Default for SecretonStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RaftLogReader<SecretonTypeConfig> for SecretonStorage {
    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + Send + Sync>(
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

#[async_trait]
impl RaftLogStorage<SecretonTypeConfig> for SecretonStorage {
    type LogReader = Self;

    async fn get_log_state(
        &mut self,
    ) -> Result<LogState<SecretonTypeConfig>, StorageError<NodeId>> {
        let log = self.log.read().await;
        let last_purged = *self.last_purged_log_id.read().await;

        let last_log_id = log.last().map(|e| e.log_id);

        Ok(LogState {
            last_purged_log_id: last_purged,
            last_log_id,
        })
    }

    async fn save_vote(&mut self, vote: &Vote) -> Result<(), StorageError<NodeId>> {
        *self.vote.write().await = Some(*vote);
        Ok(())
    }

    async fn read_vote(&mut self) -> Result<Option<Vote>, StorageError<NodeId>> {
        Ok(*self.vote.read().await)
    }

    async fn append<I>(
        &mut self,
        entries: I,
        callback: openraft::storage::LogFlushed<SecretonTypeConfig>,
    ) -> Result<(), StorageError<NodeId>>
    where
        I: IntoIterator<Item = Entry> + Send,
        I::IntoIter: Send,
    {
        let mut log = self.log.write().await;
        for entry in entries {
            log.push(entry);
        }
        callback.log_io_completed(Ok(()));
        Ok(())
    }

    async fn truncate(&mut self, log_id: LogId) -> Result<(), StorageError<NodeId>> {
        let mut log = self.log.write().await;
        log.retain(|e| e.log_id.index <= log_id.index);
        Ok(())
    }

    async fn purge(&mut self, log_id: LogId) -> Result<(), StorageError<NodeId>> {
        {
            let mut log = self.log.write().await;
            log.retain(|e| e.log_id.index > log_id.index);
        }
        *self.last_purged_log_id.write().await = Some(log_id);
        Ok(())
    }

    async fn get_log_reader(&mut self) -> Self::LogReader {
        self.clone()
    }
}

// Implement Clone for SecretonStorage
impl Clone for SecretonStorage {
    fn clone(&self) -> Self {
        Self {
            last_purged_log_id: Arc::clone(&self.last_purged_log_id),
            log: Arc::clone(&self.log),
            vote: Arc::clone(&self.vote),
            snapshot: Arc::clone(&self.snapshot),
            state_machine: Arc::clone(&self.state_machine),
        }
    }
}

#[async_trait]
impl RaftStateMachine<SecretonTypeConfig> for SecretonStorage {
    type SnapshotBuilder = SecretonStateMachine;

    async fn applied_state(
        &mut self,
    ) -> Result<(Option<LogId>, StoredMembership<SecretonTypeConfig>), StorageError<NodeId>> {
        let sm = self.state_machine.read().await;
        let last_applied = *sm.last_applied_log.read().await;
        let last_membership = sm.last_membership.read().await.clone();
        Ok((last_applied, last_membership))
    }

    async fn apply<I>(
        &mut self,
        entries: I,
    ) -> Result<Vec<openraft::raft::AppResponse<SecretonTypeConfig>>, StorageError<NodeId>>
    where
        I: IntoIterator<Item = Entry> + Send,
        I::IntoIter: Send,
    {
        let mut sm = self.state_machine.write().await;
        sm.apply(entries).await
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        let sm = self.state_machine.read().await;
        sm.clone()
    }

    async fn begin_receiving_snapshot(
        &mut self,
    ) -> Result<Box<Cursor<Vec<u8>>>, StorageError<NodeId>> {
        Ok(Box::new(Cursor::new(Vec::new())))
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMeta<SecretonTypeConfig>,
        snapshot: Box<Cursor<Vec<u8>>>,
    ) -> Result<(), StorageError<NodeId>> {
        let data = snapshot.into_inner();

        // Deserialize snapshot
        let (snapshot_data, _): (StateMachineSnapshot, _) =
            bincode::decode_from_slice(&data, bincode::config::standard()).map_err(|e| {
                StorageError::from_io_error(
                    ErrorSubject::Snapshot(Some(meta.signature())),
                    ErrorVerb::Read,
                    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()),
                )
            })?;

        // Restore state machine
        let sm = self.state_machine.read().await;
        sm.restore_snapshot(snapshot_data).await;

        // Store snapshot
        *self.snapshot.write().await = Some(Snapshot {
            meta: meta.clone(),
            snapshot: Box::new(Cursor::new(data)),
        });

        Ok(())
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<Snapshot<SecretonTypeConfig>>, StorageError<NodeId>> {
        Ok(self.snapshot.read().await.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_creation() {
        let storage = SecretonStorage::new();
        assert!(storage.log.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_vote_storage() {
        let mut storage = SecretonStorage::new();
        let vote = Vote::new(1, 1);

        storage.save_vote(&vote).await.unwrap();
        let read_vote = storage.read_vote().await.unwrap();

        assert_eq!(read_vote, Some(vote));
    }
}
