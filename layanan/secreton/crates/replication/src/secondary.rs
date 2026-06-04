//! Secondary node initialization and management
//!
//! This module handles the bootstrap process for new secondary nodes, including:
//! - Initial snapshot transfer from primary
//! - WAL streaming setup after snapshot
//! - State synchronization

use crate::{
    Result,
    error::ReplicationError,
    manager::ReplicationStorage,
    node::{ReplicationStatus, SecondaryNode},
    operation::ReplicationOperation,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Represents a snapshot of the current state for initial replication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationSnapshot {
    /// Snapshot identifier
    pub id: String,

    /// Timestamp when snapshot was created
    pub timestamp: chrono::DateTime<Utc>,

    /// Sequence number at snapshot time
    pub sequence: u64,

    /// Serialized state data
    pub data: Vec<u8>,

    /// Metadata about the snapshot
    pub metadata: SnapshotMetadata,
}

/// Metadata about a replication snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Size of the snapshot in bytes
    pub size_bytes: usize,

    /// Number of operations included
    pub operation_count: usize,

    /// Compression algorithm used (if any)
    pub compression: Option<String>,

    /// Checksum for integrity verification
    pub checksum: String,
}

/// Manages secondary node initialization
pub struct SecondaryInitializer {
    /// Storage backend
    storage: Arc<dyn ReplicationStorage>,

    /// Current initialization state
    state: Arc<RwLock<InitializationState>>,
}

/// Manages read request handling on secondary nodes
pub struct SecondaryReadHandler {
    /// Storage backend
    storage: Arc<dyn ReplicationStorage>,

    /// Last applied sequence number
    last_applied_sequence: Arc<RwLock<u64>>,

    /// Primary's last known sequence number
    primary_sequence: Arc<RwLock<u64>>,

    /// Staleness threshold in milliseconds
    staleness_threshold_ms: u64,

    /// Max time to wait for read-after-write consistency, in ms (default 5000).
    max_wait_ms: u64,

    /// Last sync timestamp
    last_sync: Arc<RwLock<chrono::DateTime<Utc>>>,
}

/// Result of a read request on secondary
#[derive(Debug, Clone)]
pub struct SecondaryReadResult {
    /// The requested data
    pub data: Vec<u8>,

    /// Sequence number when data was read
    pub sequence: u64,

    /// Replication lag in milliseconds
    pub lag_ms: u64,

    /// Whether the data might be stale
    pub is_stale: bool,
}

/// Error types specific to secondary read operations
#[derive(Debug, Clone)]
pub enum SecondaryReadError {
    /// Secondary is too far behind primary
    TooStale { lag_ms: u64, threshold_ms: u64 },

    /// Data not found
    NotFound { path: String },

    /// Read-after-write consistency violation
    ConsistencyViolation {
        expected_sequence: u64,
        actual_sequence: u64,
    },

    /// Storage error
    StorageError(String),
}

/// State of secondary initialization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitializationState {
    /// Not started
    NotStarted,

    /// Requesting snapshot from primary
    RequestingSnapshot,

    /// Transferring snapshot
    TransferringSnapshot { progress: u64, total: u64 },

    /// Applying snapshot to local storage
    ApplyingSnapshot,

    /// Starting WAL streaming
    StartingWALStream,

    /// Fully initialized and streaming
    Streaming,

    /// Initialization failed
    Failed { reason: String },
}

impl SecondaryInitializer {
    /// Create a new secondary initializer
    pub fn new(storage: Arc<dyn ReplicationStorage>) -> Self {
        Self {
            storage,
            state: Arc::new(RwLock::new(InitializationState::NotStarted)),
        }
    }

    /// Initialize a new secondary node from scratch
    ///
    /// This performs the complete bootstrap process:
    /// 1. Request snapshot from primary
    /// 2. Transfer and apply snapshot
    /// 3. Start WAL streaming from snapshot sequence
    pub async fn initialize_from_primary(&self, primary_endpoint: &str) -> Result<SecondaryNode> {
        info!(
            "Starting secondary initialization from primary: {}",
            primary_endpoint
        );

        // Update state
        *self.state.write().await = InitializationState::RequestingSnapshot;

        // Step 1: Request snapshot from primary
        let snapshot = self.request_snapshot(primary_endpoint).await?;

        info!(
            "Received snapshot {} with sequence {} ({} bytes)",
            snapshot.id, snapshot.sequence, snapshot.metadata.size_bytes
        );

        // Step 2: Transfer and apply snapshot
        self.apply_snapshot(&snapshot).await?;

        info!("Snapshot applied successfully");

        // Step 3: Start WAL streaming
        let node = self
            .start_wal_streaming(primary_endpoint, snapshot.sequence)
            .await?;

        info!("Secondary initialization complete, now streaming from primary");

        // Update state
        *self.state.write().await = InitializationState::Streaming;

        Ok(node)
    }

    /// Request a snapshot from the primary node
    async fn request_snapshot(&self, primary_endpoint: &str) -> Result<ReplicationSnapshot> {
        debug!("Requesting snapshot from primary: {}", primary_endpoint);

        // TODO: Implement actual gRPC call to primary
        // For now, create a mock snapshot
        let snapshot = ReplicationSnapshot {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            sequence: 0,
            data: Vec::new(),
            metadata: SnapshotMetadata {
                size_bytes: 0,
                operation_count: 0,
                compression: None,
                checksum: "mock-checksum".to_string(),
            },
        };

        Ok(snapshot)
    }

    /// Apply a snapshot to local storage
    async fn apply_snapshot(&self, snapshot: &ReplicationSnapshot) -> Result<()> {
        info!("Applying snapshot {} to local storage", snapshot.id);

        // Update state
        *self.state.write().await = InitializationState::ApplyingSnapshot;

        // Verify checksum
        self.verify_snapshot_checksum(snapshot)?;

        // Decompress if needed
        let data = if let Some(compression) = &snapshot.metadata.compression {
            self.decompress_snapshot(&snapshot.data, compression)?
        } else {
            snapshot.data.clone()
        };

        // Deserialize and apply operations
        let operations: Vec<ReplicationOperation> = serde_json::from_slice(&data).map_err(|e| {
            ReplicationError::snapshot(format!("Failed to deserialize snapshot: {}", e))
        })?;

        info!("Applying {} operations from snapshot", operations.len());

        for (i, operation) in operations.iter().enumerate() {
            if i % 100 == 0 {
                debug!("Applied {}/{} operations", i, operations.len());
            }

            self.storage.apply_operation(operation).await?;
        }

        // Store snapshot metadata
        self.storage
            .store_cluster_metadata("last_snapshot_sequence", &snapshot.sequence.to_le_bytes())
            .await?;

        info!("Snapshot applied successfully");

        Ok(())
    }

    /// Start WAL streaming from the primary after snapshot is applied
    async fn start_wal_streaming(
        &self,
        primary_endpoint: &str,
        start_sequence: u64,
    ) -> Result<SecondaryNode> {
        info!(
            "Starting WAL streaming from primary: {} (sequence: {})",
            primary_endpoint, start_sequence
        );

        // Update state
        *self.state.write().await = InitializationState::StartingWALStream;

        // Create secondary node
        let mut node = SecondaryNode::new(primary_endpoint.to_string());
        node.update_status(ReplicationStatus::Healthy);

        // TODO: Implement actual WAL streaming setup via gRPC
        // This would establish a long-lived streaming connection to the primary

        Ok(node)
    }

    /// Verify snapshot checksum for integrity
    fn verify_snapshot_checksum(&self, snapshot: &ReplicationSnapshot) -> Result<()> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(&snapshot.data);
        let computed_checksum = format!("{:x}", hasher.finalize());

        if computed_checksum != snapshot.metadata.checksum {
            return Err(ReplicationError::snapshot(format!(
                "Snapshot checksum mismatch: expected {}, got {}",
                snapshot.metadata.checksum, computed_checksum
            )));
        }

        debug!("Snapshot checksum verified");
        Ok(())
    }

    /// Decompress snapshot data
    fn decompress_snapshot(&self, data: &[u8], compression: &str) -> Result<Vec<u8>> {
        match compression {
            "gzip" => {
                use flate2::read::GzDecoder;
                use std::io::Read;

                let mut decoder = GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed).map_err(|e| {
                    ReplicationError::snapshot(format!("Gzip decompression failed: {}", e))
                })?;

                Ok(decompressed)
            }
            "zstd" => {
                use zstd::stream::decode_all;

                decode_all(data).map_err(|e| {
                    ReplicationError::snapshot(format!("Zstd decompression failed: {}", e))
                })
            }
            _ => Err(ReplicationError::snapshot(format!(
                "Unsupported compression algorithm: {}",
                compression
            ))),
        }
    }

    /// Get current initialization state
    pub async fn state(&self) -> InitializationState {
        self.state.read().await.clone()
    }

    /// Check if initialization is complete
    pub async fn is_initialized(&self) -> bool {
        matches!(*self.state.read().await, InitializationState::Streaming)
    }
}

impl SecondaryReadHandler {
    /// Create a new secondary read handler
    pub fn new(storage: Arc<dyn ReplicationStorage>, staleness_threshold_ms: u64) -> Self {
        Self {
            storage,
            last_applied_sequence: Arc::new(RwLock::new(0)),
            primary_sequence: Arc::new(RwLock::new(0)),
            staleness_threshold_ms,
            max_wait_ms: 5000,
            last_sync: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Override the read-after-write consistency wait timeout (default 5000ms).
    /// Mainly for tests that need a short, deterministic timeout instead of
    /// blocking the full 5s per call.
    pub fn with_max_wait_ms(mut self, max_wait_ms: u64) -> Self {
        self.max_wait_ms = max_wait_ms;
        self
    }

    /// Handle a read request on the secondary node
    ///
    /// This routes the read to local storage and ensures consistency
    pub async fn handle_read(
        &self,
        path: &str,
    ) -> std::result::Result<SecondaryReadResult, SecondaryReadError> {
        // Check staleness first
        self.check_staleness().await?;

        // Get current sequence
        let sequence = *self.last_applied_sequence.read().await;

        // Read from local storage
        // TODO: Implement actual storage read operation
        // For now, return mock data
        let data = self.read_from_storage(path).await?;

        // Calculate replication lag
        let lag_ms = self.calculate_lag().await;

        // Determine if data is stale
        let is_stale = lag_ms > self.staleness_threshold_ms;

        Ok(SecondaryReadResult {
            data,
            sequence,
            lag_ms,
            is_stale,
        })
    }

    /// Handle a read request with read-after-write consistency
    ///
    /// Ensures that reads see writes up to the specified sequence number
    pub async fn handle_read_with_consistency(
        &self,
        path: &str,
        min_sequence: u64,
    ) -> std::result::Result<SecondaryReadResult, SecondaryReadError> {
        // Wait for replication to catch up to min_sequence
        self.wait_for_sequence(min_sequence).await?;

        // Perform the read
        self.handle_read(path).await
    }

    /// Check if the secondary is too stale to serve reads
    pub async fn check_staleness(&self) -> std::result::Result<(), SecondaryReadError> {
        let lag_ms = self.calculate_lag().await;

        if lag_ms > self.staleness_threshold_ms {
            return Err(SecondaryReadError::TooStale {
                lag_ms,
                threshold_ms: self.staleness_threshold_ms,
            });
        }

        Ok(())
    }

    /// Calculate current replication lag in milliseconds
    pub async fn calculate_lag(&self) -> u64 {
        let last_sync = *self.last_sync.read().await;
        let now = Utc::now();
        let duration = now - last_sync;

        duration.num_milliseconds().max(0) as u64
    }

    /// Wait for replication to catch up to a specific sequence number
    ///
    /// This implements read-after-write consistency
    pub async fn wait_for_sequence(
        &self,
        min_sequence: u64,
    ) -> std::result::Result<(), SecondaryReadError> {
        use tokio::time::{Duration, sleep};

        let max_wait_ms = self.max_wait_ms; // configurable; default 5000 (5s)
        const POLL_INTERVAL_MS: u64 = 10; // Check every 10ms

        let start = std::time::Instant::now();

        loop {
            let current_sequence = *self.last_applied_sequence.read().await;

            if current_sequence >= min_sequence {
                debug!(
                    "Read-after-write consistency satisfied: current={}, required={}",
                    current_sequence, min_sequence
                );
                return Ok(());
            }

            // Check timeout
            if start.elapsed().as_millis() as u64 > max_wait_ms {
                return Err(SecondaryReadError::ConsistencyViolation {
                    expected_sequence: min_sequence,
                    actual_sequence: current_sequence,
                });
            }

            // Wait before next check
            sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
        }
    }

    /// Read data from local storage
    async fn read_from_storage(
        &self,
        path: &str,
    ) -> std::result::Result<Vec<u8>, SecondaryReadError> {
        // TODO: Implement actual storage read
        // This would query the local PostgreSQL or Raft storage
        // For now, return an error indicating not implemented
        Err(SecondaryReadError::NotFound {
            path: path.to_string(),
        })
    }

    /// Update the last applied sequence number
    ///
    /// Called by the replication receiver when operations are applied
    pub async fn update_sequence(&self, sequence: u64) {
        *self.last_applied_sequence.write().await = sequence;
        *self.last_sync.write().await = Utc::now();
    }

    /// Update the primary's sequence number
    ///
    /// Called when receiving heartbeats from primary
    pub async fn update_primary_sequence(&self, sequence: u64) {
        *self.primary_sequence.write().await = sequence;
    }

    /// Get current replication lag metrics
    pub async fn get_lag_metrics(&self) -> ReplicationLagMetrics {
        let last_applied = *self.last_applied_sequence.read().await;
        let primary_seq = *self.primary_sequence.read().await;
        let lag_ms = self.calculate_lag().await;

        ReplicationLagMetrics {
            last_applied_sequence: last_applied,
            primary_sequence: primary_seq,
            sequence_lag: primary_seq.saturating_sub(last_applied),
            time_lag_ms: lag_ms,
            is_stale: lag_ms > self.staleness_threshold_ms,
        }
    }

    /// Get staleness threshold
    pub fn staleness_threshold_ms(&self) -> u64 {
        self.staleness_threshold_ms
    }

    /// Set staleness threshold
    pub fn set_staleness_threshold_ms(&mut self, threshold_ms: u64) {
        self.staleness_threshold_ms = threshold_ms;
    }

    /// Get last applied sequence (for testing)
    pub fn last_applied_sequence(&self) -> Arc<RwLock<u64>> {
        self.last_applied_sequence.clone()
    }

    /// Get last sync time (for testing)
    pub fn last_sync(&self) -> Arc<RwLock<chrono::DateTime<Utc>>> {
        self.last_sync.clone()
    }
}

/// Replication lag metrics
#[derive(Debug, Clone)]
pub struct ReplicationLagMetrics {
    /// Last sequence number applied on secondary
    pub last_applied_sequence: u64,

    /// Primary's current sequence number
    pub primary_sequence: u64,

    /// Difference in sequence numbers
    pub sequence_lag: u64,

    /// Time lag in milliseconds
    pub time_lag_ms: u64,

    /// Whether the secondary is considered stale
    pub is_stale: bool,
}

/// Manages snapshot creation on the primary node
pub struct SnapshotCreator {
    /// Storage backend
    storage: Arc<dyn ReplicationStorage>,
}

impl SnapshotCreator {
    /// Create a new snapshot creator
    pub fn new(storage: Arc<dyn ReplicationStorage>) -> Self {
        Self { storage }
    }

    /// Create a snapshot of the current state
    pub async fn create_snapshot(&self) -> Result<ReplicationSnapshot> {
        info!("Creating replication snapshot");

        // Get current sequence number
        let current_sequence = self.storage.get_current_sequence().await?;

        // Get all operations up to current sequence
        let operations = self.storage.get_operations_since(0).await?;

        info!(
            "Snapshot includes {} operations up to sequence {}",
            operations.len(),
            current_sequence
        );

        // Serialize operations using JSON for compatibility
        let data = serde_json::to_vec(&operations).map_err(|e| {
            ReplicationError::snapshot(format!("Failed to serialize operations: {}", e))
        })?;

        // Compress data
        let (compressed_data, compression) = self.compress_data(&data)?;

        // Calculate checksum
        let checksum = self.calculate_checksum(&compressed_data);

        let snapshot = ReplicationSnapshot {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            sequence: current_sequence,
            data: compressed_data.clone(),
            metadata: SnapshotMetadata {
                size_bytes: compressed_data.len(),
                operation_count: operations.len(),
                compression: Some(compression),
                checksum,
            },
        };

        info!(
            "Snapshot created: {} ({} bytes, {} operations)",
            snapshot.id, snapshot.metadata.size_bytes, snapshot.metadata.operation_count
        );

        Ok(snapshot)
    }

    /// Compress snapshot data
    fn compress_data(&self, data: &[u8]) -> Result<(Vec<u8>, String)> {
        use zstd::stream::encode_all;

        // Use zstd compression with level 3 (balanced speed/compression)
        let compressed = encode_all(data, 3)
            .map_err(|e| ReplicationError::snapshot(format!("Compression failed: {}", e)))?;

        debug!(
            "Compressed snapshot from {} to {} bytes ({:.1}% reduction)",
            data.len(),
            compressed.len(),
            (1.0 - (compressed.len() as f64 / data.len() as f64)) * 100.0
        );

        Ok((compressed, "zstd".to_string()))
    }

    /// Calculate checksum for snapshot data
    fn calculate_checksum(&self, data: &[u8]) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}

// Implement Clone for SecondaryReadHandler for testing
impl Clone for SecondaryReadHandler {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            last_applied_sequence: self.last_applied_sequence.clone(),
            primary_sequence: self.primary_sequence.clone(),
            staleness_threshold_ms: self.staleness_threshold_ms,
            max_wait_ms: self.max_wait_ms,
            last_sync: self.last_sync.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::{OperationData, OperationType};
    use async_trait::async_trait;

    // Mock storage for testing
    struct MockStorage {
        operations: Arc<RwLock<Vec<ReplicationOperation>>>,
        sequence: Arc<RwLock<u64>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                operations: Arc::new(RwLock::new(Vec::new())),
                sequence: Arc::new(RwLock::new(0)),
            }
        }
    }

    #[async_trait]
    impl ReplicationStorage for MockStorage {
        async fn get_operations_since(&self, sequence: u64) -> Result<Vec<ReplicationOperation>> {
            let ops = self.operations.read().await;
            Ok(ops
                .iter()
                .filter(|op| op.sequence > sequence)
                .cloned()
                .collect())
        }

        async fn apply_operation(&self, operation: &ReplicationOperation) -> Result<()> {
            self.operations.write().await.push(operation.clone());
            *self.sequence.write().await = operation.sequence;
            Ok(())
        }

        async fn get_current_sequence(&self) -> Result<u64> {
            Ok(*self.sequence.read().await)
        }

        async fn store_cluster_metadata(&self, _key: &str, _value: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn get_cluster_metadata(&self, _key: &str) -> Result<Option<Vec<u8>>> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_snapshot_creation() {
        let storage = Arc::new(MockStorage::new());

        // Add some operations
        for i in 1..=10 {
            let op = ReplicationOperation::new(
                OperationType::SecretWrite,
                OperationData::Secret {
                    path: format!("/secret/test{}", i),
                    data: vec![1, 2, 3],
                    metadata: serde_json::json!({}),
                },
                i,
            );
            storage.apply_operation(&op).await.unwrap();
        }

        let creator = SnapshotCreator::new(storage);
        let snapshot = creator.create_snapshot().await.unwrap();

        assert_eq!(snapshot.sequence, 10);
        assert_eq!(snapshot.metadata.operation_count, 10);
        assert!(snapshot.metadata.size_bytes > 0);
        assert_eq!(snapshot.metadata.compression, Some("zstd".to_string()));
        assert!(!snapshot.metadata.checksum.is_empty());
    }

    #[tokio::test]
    async fn test_snapshot_apply() {
        let primary_storage = Arc::new(MockStorage::new());
        let secondary_storage = Arc::new(MockStorage::new());

        // Create operations on primary
        for i in 1..=5 {
            let op = ReplicationOperation::new(
                OperationType::SecretWrite,
                OperationData::Secret {
                    path: format!("/secret/test{}", i),
                    data: vec![1, 2, 3],
                    metadata: serde_json::json!({}),
                },
                i,
            );
            primary_storage.apply_operation(&op).await.unwrap();
        }

        // Create snapshot on primary
        let creator = SnapshotCreator::new(primary_storage);
        let snapshot = creator.create_snapshot().await.unwrap();

        // Apply snapshot on secondary
        let initializer = SecondaryInitializer::new(secondary_storage.clone());
        initializer.apply_snapshot(&snapshot).await.unwrap();

        // Verify secondary has all operations
        let secondary_sequence = secondary_storage.get_current_sequence().await.unwrap();
        assert_eq!(secondary_sequence, 5);

        let secondary_ops = secondary_storage.get_operations_since(0).await.unwrap();
        assert_eq!(secondary_ops.len(), 5);
    }

    #[tokio::test]
    async fn test_initialization_state_transitions() {
        let storage = Arc::new(MockStorage::new());
        let initializer = SecondaryInitializer::new(storage);

        // Initial state
        assert_eq!(initializer.state().await, InitializationState::NotStarted);
        assert!(!initializer.is_initialized().await);

        // Update to streaming
        *initializer.state.write().await = InitializationState::Streaming;
        assert_eq!(initializer.state().await, InitializationState::Streaming);
        assert!(initializer.is_initialized().await);
    }

    #[tokio::test]
    async fn test_checksum_verification() {
        let storage = Arc::new(MockStorage::new());
        let initializer = SecondaryInitializer::new(storage);

        let mut snapshot = ReplicationSnapshot {
            id: "test".to_string(),
            timestamp: Utc::now(),
            sequence: 1,
            data: vec![1, 2, 3, 4, 5],
            metadata: SnapshotMetadata {
                size_bytes: 5,
                operation_count: 1,
                compression: None,
                checksum: "invalid-checksum".to_string(),
            },
        };

        // Should fail with invalid checksum
        assert!(initializer.verify_snapshot_checksum(&snapshot).is_err());

        // Calculate correct checksum
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&snapshot.data);
        snapshot.metadata.checksum = format!("{:x}", hasher.finalize());

        // Should succeed with valid checksum
        assert!(initializer.verify_snapshot_checksum(&snapshot).is_ok());
    }

    #[tokio::test]
    async fn test_secondary_read_handler_staleness_detection() {
        let storage = Arc::new(MockStorage::new());
        let handler = SecondaryReadHandler::new(storage, 100); // 100ms threshold

        // Initially, lag should be minimal
        let lag = handler.calculate_lag().await;
        assert!(lag < 100);

        // Simulate old sync time
        *handler.last_sync.write().await = Utc::now() - chrono::Duration::milliseconds(200);

        // Now lag should exceed threshold
        let lag = handler.calculate_lag().await;
        assert!(lag >= 200);

        // Check staleness should fail
        let result = handler.check_staleness().await;
        assert!(matches!(result, Err(SecondaryReadError::TooStale { .. })));
    }

    #[tokio::test]
    async fn test_secondary_read_handler_sequence_update() {
        let storage = Arc::new(MockStorage::new());
        let handler = SecondaryReadHandler::new(storage, 1000);

        // Initial sequence should be 0
        assert_eq!(*handler.last_applied_sequence.read().await, 0);

        // Update sequence
        handler.update_sequence(42).await;

        // Sequence should be updated
        assert_eq!(*handler.last_applied_sequence.read().await, 42);

        // Last sync should be recent
        let last_sync = *handler.last_sync.read().await;
        let now = Utc::now();
        let diff = (now - last_sync).num_milliseconds();
        assert!(diff < 100); // Should be within 100ms
    }

    #[tokio::test]
    async fn test_secondary_read_handler_lag_metrics() {
        let storage = Arc::new(MockStorage::new());
        let handler = SecondaryReadHandler::new(storage, 100);

        // Set sequences
        handler.update_sequence(10).await;
        handler.update_primary_sequence(15).await;

        // Get metrics
        let metrics = handler.get_lag_metrics().await;

        assert_eq!(metrics.last_applied_sequence, 10);
        assert_eq!(metrics.primary_sequence, 15);
        assert_eq!(metrics.sequence_lag, 5);
        assert!(metrics.time_lag_ms < 100); // Should be recent
        assert!(!metrics.is_stale); // Should not be stale
    }

    #[tokio::test]
    async fn test_secondary_read_handler_wait_for_sequence() {
        let storage = Arc::new(MockStorage::new());
        let handler = SecondaryReadHandler::new(storage, 1000);

        // Set initial sequence
        handler.update_sequence(5).await;

        // Spawn a task to update sequence after delay
        let handler_clone = handler.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            handler_clone.update_sequence(10).await;
        });

        // Wait for sequence 10 - should succeed
        let result = handler.wait_for_sequence(10).await;
        assert!(result.is_ok());

        // Current sequence should be at least 10
        assert!(*handler.last_applied_sequence.read().await >= 10);
    }

    #[tokio::test]
    async fn test_secondary_read_handler_wait_for_sequence_timeout() {
        let storage = Arc::new(MockStorage::new());
        // Short timeout so the test doesn't block the full default 5s.
        let handler = SecondaryReadHandler::new(storage, 1000).with_max_wait_ms(200);

        // Set initial sequence
        handler.update_sequence(5).await;

        // Wait for sequence 100 - should timeout
        let result = handler.wait_for_sequence(100).await;
        assert!(matches!(
            result,
            Err(SecondaryReadError::ConsistencyViolation { .. })
        ));
    }

    #[tokio::test]
    async fn test_secondary_read_handler_staleness_threshold() {
        let storage = Arc::new(MockStorage::new());
        let mut handler = SecondaryReadHandler::new(storage, 100);

        assert_eq!(handler.staleness_threshold_ms(), 100);

        handler.set_staleness_threshold_ms(500);
        assert_eq!(handler.staleness_threshold_ms(), 500);
    }
}
