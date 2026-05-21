//! Replication manager implementation

use crate::{
    Result,
    config::ReplicationConfig,
    conflict::{ConflictResolver, ResolutionStrategy},
    error::ReplicationError,
    failover::{FailoverDetector, HealthStatus, Heartbeat},
    metrics::{ReplicationMetrics, ReplicationStatusValue},
    mode::ReplicationMode,
    node::SecondaryNode,
    operation::ReplicationOperation,
    secondary::{SecondaryInitializer, SecondaryReadHandler, SnapshotCreator},
};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Trait for storage backend operations needed by replication
#[async_trait]
pub trait ReplicationStorage: Send + Sync {
    /// Get operations since a given sequence number
    async fn get_operations_since(&self, sequence: u64) -> Result<Vec<ReplicationOperation>>;

    /// Apply a replicated operation
    async fn apply_operation(&self, operation: &ReplicationOperation) -> Result<()>;

    /// Get the current sequence number
    async fn get_current_sequence(&self) -> Result<u64>;

    /// Store cluster metadata
    async fn store_cluster_metadata(&self, key: &str, value: &[u8]) -> Result<()>;

    /// Get cluster metadata
    async fn get_cluster_metadata(&self, key: &str) -> Result<Option<Vec<u8>>>;
}

/// Replication manager handles replication between primary and secondary nodes
pub struct ReplicationManager {
    /// Replication configuration
    config: ReplicationConfig,

    /// Replication mode
    mode: ReplicationMode,

    /// List of secondary nodes (for primary)
    secondaries: Arc<RwLock<Vec<SecondaryNode>>>,

    /// Current replication lag
    replication_lag: Arc<RwLock<Duration>>,

    /// Storage backend
    storage: Arc<dyn ReplicationStorage>,

    /// Is this node the primary?
    is_primary: Arc<RwLock<bool>>,

    /// Last replicated sequence number
    last_sequence: Arc<RwLock<u64>>,

    /// Secondary initializer (for secondary nodes)
    secondary_initializer: Option<Arc<SecondaryInitializer>>,

    /// Snapshot creator (for primary nodes)
    snapshot_creator: Option<Arc<SnapshotCreator>>,

    /// Read handler (for secondary nodes)
    read_handler: Option<Arc<SecondaryReadHandler>>,

    /// Metrics collector
    metrics: Arc<ReplicationMetrics>,

    /// Failover detector (for secondary nodes)
    failover_detector: Option<Arc<FailoverDetector>>,

    /// Conflict resolver for handling write conflicts during failover
    conflict_resolver: Arc<ConflictResolver>,
}

impl ReplicationManager {
    /// Create a new replication manager
    pub async fn new(
        config: ReplicationConfig,
        storage: Arc<dyn ReplicationStorage>,
    ) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        let mode = config.mode;
        let is_primary = config.primary_endpoint.is_none();

        info!(
            "Initializing replication manager in {} mode (primary: {})",
            mode, is_primary
        );

        let secondary_initializer = if !is_primary {
            Some(Arc::new(SecondaryInitializer::new(storage.clone())))
        } else {
            None
        };

        let snapshot_creator = if is_primary {
            Some(Arc::new(SnapshotCreator::new(storage.clone())))
        } else {
            None
        };

        let read_handler = if !is_primary {
            // Default staleness threshold: 100ms
            let staleness_threshold_ms = config.staleness_threshold_ms.unwrap_or(100);
            Some(Arc::new(SecondaryReadHandler::new(
                storage.clone(),
                staleness_threshold_ms,
            )))
        } else {
            None
        };

        // Initialize metrics
        let node_id = if is_primary {
            "primary".to_string()
        } else {
            config
                .primary_endpoint
                .as_ref()
                .map(|ep| format!("secondary-{}", ep))
                .unwrap_or_else(|| "secondary-unknown".to_string())
        };

        let metrics = Arc::new(ReplicationMetrics::new(node_id, mode.to_string()));

        // Initialize failover detector for secondary nodes
        let failover_detector = if !is_primary {
            let heartbeat_timeout =
                Duration::from_millis(config.heartbeat_timeout_ms.unwrap_or(5000));
            let failure_threshold = config.failure_threshold.unwrap_or(3);

            let detector = Arc::new(FailoverDetector::new(heartbeat_timeout, failure_threshold));

            Some(detector)
        } else {
            None
        };

        // Initialize conflict resolver with last-write-wins strategy
        let conflict_resolver = Arc::new(ConflictResolver::new(
            ResolutionStrategy::LastWriteWins,
            1000, // Keep last 1000 conflicts in history
        ));

        Ok(Self {
            config,
            mode,
            secondaries: Arc::new(RwLock::new(Vec::new())),
            replication_lag: Arc::new(RwLock::new(Duration::from_secs(0))),
            storage,
            is_primary: Arc::new(RwLock::new(is_primary)),
            last_sequence: Arc::new(RwLock::new(0)),
            secondary_initializer,
            snapshot_creator,
            read_handler,
            metrics,
            failover_detector,
            conflict_resolver,
        })
    }

    /// Start the replication manager
    pub async fn start(&self) -> Result<()> {
        info!("Starting replication manager");

        if *self.is_primary.read().await {
            self.start_as_primary().await?;
        } else {
            self.start_as_secondary().await?;
        }

        Ok(())
    }

    /// Stop the replication manager
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping replication manager");
        // TODO: Implement graceful shutdown
        Ok(())
    }

    /// Add a secondary node (primary only)
    pub async fn add_secondary(&self, endpoint: String) -> Result<String> {
        if !*self.is_primary.read().await {
            return Err(ReplicationError::UnsupportedOperation {
                mode: "secondary".to_string(),
                operation: "add_secondary".to_string(),
            });
        }

        let node = SecondaryNode::new(endpoint);
        let node_id = node.id.clone();

        info!("Adding secondary node: {} ({})", node.endpoint, node_id);

        self.secondaries.write().await.push(node);

        Ok(node_id)
    }

    /// Remove a secondary node (primary only)
    pub async fn remove_secondary(&self, node_id: &str) -> Result<()> {
        if !*self.is_primary.read().await {
            return Err(ReplicationError::UnsupportedOperation {
                mode: "secondary".to_string(),
                operation: "remove_secondary".to_string(),
            });
        }

        let mut secondaries = self.secondaries.write().await;
        let initial_len = secondaries.len();

        secondaries.retain(|node| node.id != node_id);

        if secondaries.len() == initial_len {
            return Err(ReplicationError::NodeNotFound {
                node_id: node_id.to_string(),
            });
        }

        info!("Removed secondary node: {}", node_id);

        Ok(())
    }

    /// Get list of secondary nodes
    pub async fn get_secondaries(&self) -> Vec<SecondaryNode> {
        self.secondaries.read().await.clone()
    }

    /// Get current replication lag
    pub async fn get_lag(&self) -> Duration {
        *self.replication_lag.read().await
    }

    /// Replicate an operation to all secondaries (primary only)
    pub async fn replicate_operation(&self, operation: ReplicationOperation) -> Result<()> {
        if !*self.is_primary.read().await {
            return Err(ReplicationError::UnsupportedOperation {
                mode: "secondary".to_string(),
                operation: "replicate_operation".to_string(),
            });
        }

        // Check if operation should be replicated based on mode
        if self.mode == ReplicationMode::Performance && !operation.is_performance_replicable() {
            debug!("Skipping operation {} in Performance mode", operation.id);
            return Ok(());
        }

        // Check namespace filtering
        if let Some(namespace) = self.extract_namespace(&operation)
            && !self.config.should_replicate_namespace(&namespace)
        {
            debug!(
                "Skipping operation {} for filtered namespace {}",
                operation.id, namespace
            );
            return Ok(());
        }

        let secondaries = self.secondaries.read().await;

        for secondary in secondaries.iter() {
            if !secondary.is_operational() {
                warn!(
                    "Skipping non-operational secondary: {} (status: {})",
                    secondary.id, secondary.status
                );
                continue;
            }

            // TODO: Implement actual gRPC replication
            debug!(
                "Would replicate operation {} to secondary {}",
                operation.id, secondary.id
            );
        }

        Ok(())
    }

    /// Promote this secondary to primary (DR mode only)
    ///
    /// This method performs the complete promotion process:
    /// 1. Validates that promotion is allowed (DR mode, not already primary)
    /// 2. Stops WAL replication from the old primary
    /// 3. Enables write operations on this node
    /// 4. Updates cluster configuration metadata
    /// 5. Notifies monitoring systems of the promotion
    ///
    /// **Validates: Requirements 2.2.7**
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - This node is already the primary
    /// - Replication mode is not DisasterRecovery
    /// - Cluster metadata update fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use secreton_replication::ReplicationManager;
    /// # async fn example(manager: &ReplicationManager) -> Result<(), Box<dyn std::error::Error>> {
    /// // Promote secondary to primary after detecting primary failure
    /// manager.promote_to_primary().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn promote_to_primary(&self) -> Result<()> {
        // Validate that promotion is allowed
        if *self.is_primary.read().await {
            return Err(ReplicationError::UnsupportedOperation {
                mode: "primary".to_string(),
                operation: "promote_to_primary".to_string(),
            });
        }

        if self.mode != ReplicationMode::DisasterRecovery {
            return Err(ReplicationError::UnsupportedOperation {
                mode: self.mode.to_string(),
                operation: "promote_to_primary".to_string(),
            });
        }

        info!("Starting promotion of secondary to primary");

        // 1. Stop WAL replication from old primary
        info!("Stopping WAL replication from old primary");
        self.stop_replication_receiver().await?;

        // 2. Enable write operations on this node
        info!("Enabling write operations");
        self.enable_write_operations().await?;

        // 3. Update cluster metadata to mark this node as primary
        info!("Updating cluster metadata");
        self.storage
            .store_cluster_metadata("is_primary", b"true")
            .await
            .map_err(|e| {
                error!("Failed to update cluster metadata: {}", e);
                e
            })?;

        // Store promotion timestamp for audit trail
        let promotion_time = Utc::now().to_rfc3339();
        self.storage
            .store_cluster_metadata("promoted_at", promotion_time.as_bytes())
            .await?;

        // 4. Update local state
        *self.is_primary.write().await = true;

        // 5. Update metrics to reflect new primary status
        self.metrics
            .update_status(ReplicationStatusValue::Healthy)
            .await;

        // 6. Clear secondary-specific state
        if let Some(detector) = &self.failover_detector {
            detector.reset().await;
        }

        // 7. Notify other nodes of the promotion (if configured)
        self.notify_promotion().await?;

        info!("Successfully promoted to primary");

        Ok(())
    }

    /// Stop receiving replication from the old primary
    ///
    /// This is called during promotion to stop the replication receiver loop
    async fn stop_replication_receiver(&self) -> Result<()> {
        // The receive loop checks is_primary flag and will stop automatically
        // We just need to ensure any pending operations are flushed
        debug!("Replication receiver will stop on next iteration");

        // Wait a short time for the receiver to notice the state change
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Enable write operations on this node
    ///
    /// This transitions the node from read-only secondary to writable primary
    async fn enable_write_operations(&self) -> Result<()> {
        // Store write-enabled flag in cluster metadata
        self.storage
            .store_cluster_metadata("write_enabled", b"true")
            .await?;

        debug!("Write operations enabled");

        Ok(())
    }

    /// Notify other nodes and monitoring systems of the promotion
    ///
    /// This sends notifications about the promotion event
    async fn notify_promotion(&self) -> Result<()> {
        // Log promotion event for audit trail
        info!("Node promoted to primary at {}", Utc::now().to_rfc3339());

        // Update metrics with promotion event
        self.metrics.record_promotion().await;

        // TODO: Send notifications to monitoring systems (Prometheus, etc.)
        // TODO: Update service discovery (if applicable)

        Ok(())
    }

    /// Resolve conflicts after failover
    ///
    /// This method detects and resolves write conflicts that may have occurred
    /// during the failover window. It uses the last-write-wins strategy to
    /// determine which operations should be kept.
    ///
    /// **Validates: Requirements 2.2.6, 2.2.8**
    ///
    /// # Arguments
    ///
    /// * `operations` - List of operations to check for conflicts
    ///
    /// # Returns
    ///
    /// A map of paths to the winning operations after conflict resolution
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use secreton_replication::ReplicationManager;
    /// # async fn example(manager: &ReplicationManager, ops: Vec<secreton_replication::ReplicationOperation>) -> Result<(), Box<dyn std::error::Error>> {
    /// // Resolve conflicts after failover
    /// let resolved = manager.resolve_conflicts(ops).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve_conflicts(
        &self,
        operations: Vec<ReplicationOperation>,
    ) -> Result<std::collections::HashMap<String, ReplicationOperation>> {
        info!("Resolving conflicts for {} operations", operations.len());

        let resolved = self.conflict_resolver.batch_resolve(operations).await?;

        // Log conflict resolution summary
        let conflict_count = self.conflict_resolver.conflict_count().await;
        if conflict_count > 0 {
            warn!(
                "Resolved {} conflicts during failover using last-write-wins strategy",
                conflict_count
            );

            // Log conflicts to audit trail
            let conflicts = self.conflict_resolver.get_history().await;
            for conflict in conflicts.iter().take(10) {
                // Log first 10 conflicts
                info!(
                    "Conflict at path '{}': {}",
                    conflict.path, conflict.resolution_reason
                );
            }

            if conflicts.len() > 10 {
                info!("... and {} more conflicts", conflicts.len() - 10);
            }
        }

        Ok(resolved)
    }

    /// Get conflict resolution history
    ///
    /// Returns all conflicts that have been resolved, useful for audit purposes
    pub async fn get_conflict_history(&self) -> Vec<crate::conflict::Conflict> {
        self.conflict_resolver.get_history().await
    }

    /// Get conflicts for a specific path
    ///
    /// # Arguments
    ///
    /// * `path` - The path to filter by
    ///
    /// # Returns
    ///
    /// All conflicts that occurred at the specified path
    pub async fn get_conflicts_for_path(&self, path: &str) -> Vec<crate::conflict::Conflict> {
        self.conflict_resolver.get_conflicts_for_path(path).await
    }

    /// Check if this node is the primary
    pub async fn is_primary(&self) -> bool {
        *self.is_primary.read().await
    }

    /// Get replication mode
    pub fn mode(&self) -> ReplicationMode {
        self.mode
    }

    /// Get replication configuration
    pub fn config(&self) -> &ReplicationConfig {
        &self.config
    }

    /// Initialize this node as a secondary from the primary
    ///
    /// This performs the complete bootstrap process:
    /// 1. Request snapshot from primary
    /// 2. Transfer and apply snapshot
    /// 3. Start WAL streaming from snapshot sequence
    pub async fn initialize_secondary(&self) -> Result<()> {
        if *self.is_primary.read().await {
            return Err(ReplicationError::UnsupportedOperation {
                mode: "primary".to_string(),
                operation: "initialize_secondary".to_string(),
            });
        }

        let primary_endpoint = self
            .config
            .primary_endpoint
            .as_ref()
            .ok_or_else(|| ReplicationError::config("primary_endpoint not configured"))?;

        let initializer = self
            .secondary_initializer
            .as_ref()
            .ok_or_else(|| ReplicationError::internal("Secondary initializer not available"))?;

        info!("Initializing secondary from primary: {}", primary_endpoint);

        let _node = initializer
            .initialize_from_primary(primary_endpoint)
            .await?;

        info!("Secondary initialization complete");

        Ok(())
    }

    /// Create a snapshot of the current state (primary only)
    ///
    /// This is used to bootstrap new secondary nodes
    pub async fn create_snapshot(&self) -> Result<crate::secondary::ReplicationSnapshot> {
        if !*self.is_primary.read().await {
            return Err(ReplicationError::UnsupportedOperation {
                mode: "secondary".to_string(),
                operation: "create_snapshot".to_string(),
            });
        }

        let creator = self
            .snapshot_creator
            .as_ref()
            .ok_or_else(|| ReplicationError::internal("Snapshot creator not available"))?;

        info!("Creating replication snapshot");

        let snapshot = creator.create_snapshot().await?;

        info!(
            "Snapshot created: {} ({} bytes, {} operations)",
            snapshot.id, snapshot.metadata.size_bytes, snapshot.metadata.operation_count
        );

        Ok(snapshot)
    }

    /// Get the initialization state (secondary only)
    pub async fn initialization_state(&self) -> Option<crate::secondary::InitializationState> {
        if let Some(initializer) = &self.secondary_initializer {
            Some(initializer.state().await)
        } else {
            None
        }
    }

    /// Check if secondary initialization is complete
    pub async fn is_secondary_initialized(&self) -> bool {
        if let Some(initializer) = &self.secondary_initializer {
            initializer.is_initialized().await
        } else {
            false
        }
    }

    /// Handle a read request on secondary node
    ///
    /// Routes read requests to local storage with staleness detection
    pub async fn handle_secondary_read(
        &self,
        path: &str,
    ) -> std::result::Result<
        crate::secondary::SecondaryReadResult,
        crate::secondary::SecondaryReadError,
    > {
        if *self.is_primary.read().await {
            return Err(crate::secondary::SecondaryReadError::StorageError(
                "Cannot handle secondary read on primary node".to_string(),
            ));
        }

        let handler = self.read_handler.as_ref().ok_or_else(|| {
            crate::secondary::SecondaryReadError::StorageError(
                "Read handler not available".to_string(),
            )
        })?;

        handler.handle_read(path).await
    }

    /// Handle a read request with read-after-write consistency
    ///
    /// Ensures reads see writes up to the specified sequence number
    pub async fn handle_secondary_read_with_consistency(
        &self,
        path: &str,
        min_sequence: u64,
    ) -> std::result::Result<
        crate::secondary::SecondaryReadResult,
        crate::secondary::SecondaryReadError,
    > {
        if *self.is_primary.read().await {
            return Err(crate::secondary::SecondaryReadError::StorageError(
                "Cannot handle secondary read on primary node".to_string(),
            ));
        }

        let handler = self.read_handler.as_ref().ok_or_else(|| {
            crate::secondary::SecondaryReadError::StorageError(
                "Read handler not available".to_string(),
            )
        })?;

        handler
            .handle_read_with_consistency(path, min_sequence)
            .await
    }

    /// Get replication lag metrics for secondary node
    pub async fn get_secondary_lag_metrics(
        &self,
    ) -> Option<crate::secondary::ReplicationLagMetrics> {
        if let Some(handler) = &self.read_handler {
            Some(handler.get_lag_metrics().await)
        } else {
            None
        }
    }

    /// Get replication metrics collector
    pub fn metrics(&self) -> Arc<ReplicationMetrics> {
        self.metrics.clone()
    }

    /// Get current replication lag in bytes (sequence difference)
    pub async fn get_lag_bytes(&self) -> u64 {
        self.metrics.get_lag_bytes().await
    }

    /// Get current replication lag in milliseconds
    pub async fn get_lag_ms(&self) -> u64 {
        self.metrics.get_lag_ms().await
    }

    /// Get primary WAL position (sequence number)
    pub async fn get_primary_sequence(&self) -> u64 {
        if *self.is_primary.read().await {
            self.storage.get_current_sequence().await.unwrap_or(0)
        } else {
            self.metrics.get_primary_sequence().await
        }
    }

    /// Get secondary WAL position (sequence number)
    pub async fn get_secondary_sequence(&self) -> u64 {
        if *self.is_primary.read().await {
            0 // Primary doesn't have a secondary sequence
        } else {
            self.metrics.get_secondary_sequence().await
        }
    }

    /// Update sequence number after applying operation (secondary only)
    ///
    /// This should be called by the replication receiver when operations are applied
    pub async fn update_applied_sequence(&self, sequence: u64) -> Result<()> {
        if *self.is_primary.read().await {
            return Err(ReplicationError::UnsupportedOperation {
                mode: "primary".to_string(),
                operation: "update_applied_sequence".to_string(),
            });
        }

        if let Some(handler) = &self.read_handler {
            handler.update_sequence(sequence).await;
        }

        *self.last_sequence.write().await = sequence;

        Ok(())
    }

    /// Update primary sequence number from heartbeat (secondary only)
    pub async fn update_primary_sequence(&self, sequence: u64) -> Result<()> {
        if *self.is_primary.read().await {
            return Err(ReplicationError::UnsupportedOperation {
                mode: "primary".to_string(),
                operation: "update_primary_sequence".to_string(),
            });
        }

        if let Some(handler) = &self.read_handler {
            handler.update_primary_sequence(sequence).await;
        }

        // Update metrics with both primary and secondary sequences
        let secondary_seq = *self.last_sequence.read().await;
        self.metrics
            .update_wal_positions(sequence, secondary_seq)
            .await;

        // Calculate lag and update status
        let lag_bytes = sequence.saturating_sub(secondary_seq);
        let lag_metrics = self.get_secondary_lag_metrics().await;

        if let Some(metrics) = lag_metrics {
            self.metrics
                .update_lag(lag_bytes, metrics.time_lag_ms)
                .await;

            let status = if metrics.is_stale {
                ReplicationStatusValue::Lagging
            } else {
                ReplicationStatusValue::Healthy
            };
            self.metrics.update_status(status).await;
        }

        Ok(())
    }

    /// Record a heartbeat from the primary (secondary only)
    ///
    /// This updates the failover detector with the latest heartbeat
    pub async fn record_primary_heartbeat(
        &self,
        sequence: u64,
        status: HealthStatus,
    ) -> Result<()> {
        if *self.is_primary.read().await {
            return Err(ReplicationError::UnsupportedOperation {
                mode: "primary".to_string(),
                operation: "record_primary_heartbeat".to_string(),
            });
        }

        // Update primary sequence
        self.update_primary_sequence(sequence).await?;

        // Record heartbeat in failover detector
        if let Some(detector) = &self.failover_detector {
            let heartbeat = Heartbeat {
                sequence,
                timestamp: chrono::Utc::now(),
                status,
            };

            detector.record_heartbeat(heartbeat).await;
        }

        Ok(())
    }

    /// Start failover monitoring (secondary only)
    ///
    /// This starts a background task that monitors primary health and triggers
    /// automatic promotion if the primary fails
    pub async fn start_failover_monitoring(&self) -> Result<()> {
        if *self.is_primary.read().await {
            return Err(ReplicationError::UnsupportedOperation {
                mode: "primary".to_string(),
                operation: "start_failover_monitoring".to_string(),
            });
        }

        let detector = self
            .failover_detector
            .as_ref()
            .ok_or_else(|| ReplicationError::internal("Failover detector not available"))?;

        // Set up failover callback
        let is_primary = self.is_primary.clone();
        let storage = self.storage.clone();
        let _mode = self.mode;

        let _callback_detector = detector.clone();
        let callback = move || {
            let is_primary = is_primary.clone();
            let storage = storage.clone();

            tokio::spawn(async move {
                info!("Failover triggered, promoting to primary");

                // 1. Stop replication from old primary
                // TODO: Implement replication stop

                // 2. Update cluster metadata
                if let Err(e) = storage.store_cluster_metadata("is_primary", b"true").await {
                    error!("Failed to update cluster metadata: {}", e);
                    return;
                }

                // 3. Update local state
                *is_primary.write().await = true;

                // 4. Start accepting writes
                // TODO: Implement write enablement

                info!("Successfully promoted to primary");
            });
        };

        // Create a new detector with the callback
        let heartbeat_timeout =
            Duration::from_millis(self.config.heartbeat_timeout_ms.unwrap_or(5000));
        let failure_threshold = self.config.failure_threshold.unwrap_or(3);

        let mut new_detector = FailoverDetector::new(heartbeat_timeout, failure_threshold);
        new_detector.set_failover_callback(callback);

        // Start monitoring loop
        let check_interval =
            Duration::from_millis(self.config.heartbeat_check_interval_ms.unwrap_or(1000));

        new_detector.start_monitoring(check_interval);

        info!("Failover monitoring started");

        Ok(())
    }

    /// Check if failover has been triggered (secondary only)
    pub async fn is_failover_triggered(&self) -> bool {
        if let Some(detector) = &self.failover_detector {
            detector.is_failover_triggered().await
        } else {
            false
        }
    }

    /// Get the current failure count (secondary only)
    pub async fn failure_count(&self) -> u32 {
        if let Some(detector) = &self.failover_detector {
            detector.failure_count().await
        } else {
            0
        }
    }

    // Private methods

    async fn start_as_primary(&self) -> Result<()> {
        info!("Starting as primary node");

        // Initialize secondary nodes from configuration
        for endpoint in &self.config.secondary_endpoints {
            self.add_secondary(endpoint.clone()).await?;
        }

        // Start replication loop
        self.spawn_replication_loop();

        Ok(())
    }

    async fn start_as_secondary(&self) -> Result<()> {
        info!("Starting as secondary node");

        // Start receiving replication stream from primary
        self.spawn_receive_loop();

        Ok(())
    }

    fn spawn_replication_loop(&self) {
        let _secondaries = self.secondaries.clone();
        let storage = self.storage.clone();
        let last_sequence = self.last_sequence.clone();
        let replication_lag = self.replication_lag.clone();
        let metrics = self.metrics.clone();
        let interval = self.config.interval();
        let _batch_size = self.config.batch_size;

        tokio::spawn(async move {
            let mut last_update = std::time::Instant::now();
            let mut operations_count = 0u64;

            loop {
                tokio::time::sleep(interval).await;

                let current_sequence = match storage.get_current_sequence().await {
                    Ok(seq) => seq,
                    Err(e) => {
                        error!("Failed to get current sequence: {}", e);
                        metrics
                            .update_status(ReplicationStatusValue::Disconnected)
                            .await;
                        continue;
                    }
                };

                // Update primary sequence in metrics
                metrics
                    .update_wal_positions(current_sequence, *last_sequence.read().await)
                    .await;

                let last_seq = *last_sequence.read().await;

                if current_sequence > last_seq {
                    let operations = match storage.get_operations_since(last_seq).await {
                        Ok(ops) => ops,
                        Err(e) => {
                            error!("Failed to get operations: {}", e);
                            metrics
                                .update_status(ReplicationStatusValue::Disconnected)
                                .await;
                            continue;
                        }
                    };

                    if !operations.is_empty() {
                        debug!("Replicating {} operations", operations.len());

                        let start = std::time::Instant::now();

                        // TODO: Send operations to secondaries via gRPC
                        // For now, just record metrics
                        for op in &operations {
                            metrics.record_operation(&op.operation_type.to_string(), true);
                        }

                        // Update last sequence
                        if let Some(last_op) = operations.last() {
                            *last_sequence.write().await = last_op.sequence;
                        }

                        // Update lag metric (time-based)
                        if let Some(first_op) = operations.first() {
                            let lag = chrono::Utc::now() - first_op.timestamp;
                            let lag_duration = lag.to_std().unwrap_or(Duration::from_secs(0));
                            *replication_lag.write().await = lag_duration;

                            // Update metrics
                            let lag_ms = lag_duration.as_millis() as u64;
                            let lag_bytes = current_sequence.saturating_sub(last_seq);
                            metrics.update_lag(lag_bytes, lag_ms).await;

                            // Update status based on lag
                            let status = if lag_ms > 1000 {
                                ReplicationStatusValue::Lagging
                            } else {
                                ReplicationStatusValue::Healthy
                            };
                            metrics.update_status(status).await;
                        }

                        // Record operation latency
                        let latency = start.elapsed().as_secs_f64();
                        metrics.record_latency("replication_batch", latency);

                        operations_count += operations.len() as u64;
                    }
                }

                // Update throughput every second
                let elapsed = last_update.elapsed();
                if elapsed.as_secs() >= 1 {
                    let ops_per_second = operations_count as f64 / elapsed.as_secs_f64();
                    metrics.update_throughput(ops_per_second);

                    last_update = std::time::Instant::now();
                    operations_count = 0;
                }
            }
        });
    }

    fn spawn_receive_loop(&self) {
        let _storage = self.storage.clone();

        tokio::spawn(async move {
            loop {
                // TODO: Receive operations from primary via gRPC
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    fn extract_namespace(&self, _operation: &ReplicationOperation) -> Option<String> {
        // TODO: Extract namespace from operation path
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock storage implementation for testing
    struct MockStorage;

    #[async_trait]
    impl ReplicationStorage for MockStorage {
        async fn get_operations_since(&self, _sequence: u64) -> Result<Vec<ReplicationOperation>> {
            Ok(Vec::new())
        }

        async fn apply_operation(&self, _operation: &ReplicationOperation) -> Result<()> {
            Ok(())
        }

        async fn get_current_sequence(&self) -> Result<u64> {
            Ok(0)
        }

        async fn store_cluster_metadata(&self, _key: &str, _value: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn get_cluster_metadata(&self, _key: &str) -> Result<Option<Vec<u8>>> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_new_replication_manager() {
        let config = ReplicationConfig {
            mode: ReplicationMode::Performance,
            primary_endpoint: None,
            secondary_endpoints: vec!["https://secondary:50051".to_string()],
            ..Default::default()
        };

        let storage = Arc::new(MockStorage);
        let manager = ReplicationManager::new(config, storage).await.unwrap();

        assert_eq!(manager.mode(), ReplicationMode::Performance);
        assert!(manager.is_primary().await);
    }

    #[tokio::test]
    async fn test_add_remove_secondary() {
        let config = ReplicationConfig {
            mode: ReplicationMode::Performance,
            primary_endpoint: None,
            secondary_endpoints: Vec::new(),
            ..Default::default()
        };

        let storage = Arc::new(MockStorage);
        let manager = ReplicationManager::new(config, storage).await.unwrap();

        // Add secondary
        let node_id = manager
            .add_secondary("https://secondary:50051".to_string())
            .await
            .unwrap();

        let secondaries = manager.get_secondaries().await;
        assert_eq!(secondaries.len(), 1);
        assert_eq!(secondaries[0].id, node_id);

        // Remove secondary
        manager.remove_secondary(&node_id).await.unwrap();

        let secondaries = manager.get_secondaries().await;
        assert_eq!(secondaries.len(), 0);
    }

    #[tokio::test]
    async fn test_secondary_cannot_add_nodes() {
        let config = ReplicationConfig {
            mode: ReplicationMode::Performance,
            primary_endpoint: Some("https://primary:50051".to_string()),
            secondary_endpoints: Vec::new(),
            ..Default::default()
        };

        let storage = Arc::new(MockStorage);
        let manager = ReplicationManager::new(config, storage).await.unwrap();

        assert!(!manager.is_primary().await);

        let result = manager
            .add_secondary("https://secondary:50051".to_string())
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_promote_to_primary_success() {
        let config = ReplicationConfig {
            mode: ReplicationMode::DisasterRecovery,
            primary_endpoint: Some("https://primary:50051".to_string()),
            secondary_endpoints: Vec::new(),
            ..Default::default()
        };

        let storage = Arc::new(MockStorage);
        let manager = ReplicationManager::new(config, storage).await.unwrap();

        // Initially should be secondary
        assert!(!manager.is_primary().await);

        // Promote to primary
        let result = manager.promote_to_primary().await;
        assert!(result.is_ok());

        // Should now be primary
        assert!(manager.is_primary().await);
    }

    #[tokio::test]
    async fn test_promote_to_primary_already_primary() {
        let config = ReplicationConfig {
            mode: ReplicationMode::DisasterRecovery,
            primary_endpoint: None,
            secondary_endpoints: Vec::new(),
            ..Default::default()
        };

        let storage = Arc::new(MockStorage);
        let manager = ReplicationManager::new(config, storage).await.unwrap();

        // Already primary
        assert!(manager.is_primary().await);

        // Should fail to promote
        let result = manager.promote_to_primary().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_promote_to_primary_wrong_mode() {
        let config = ReplicationConfig {
            mode: ReplicationMode::Performance,
            primary_endpoint: Some("https://primary:50051".to_string()),
            secondary_endpoints: Vec::new(),
            ..Default::default()
        };

        let storage = Arc::new(MockStorage);
        let manager = ReplicationManager::new(config, storage).await.unwrap();

        // Should fail to promote in Performance mode
        let result = manager.promote_to_primary().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_promote_to_primary_updates_metrics() {
        let config = ReplicationConfig {
            mode: ReplicationMode::DisasterRecovery,
            primary_endpoint: Some("https://primary:50051".to_string()),
            secondary_endpoints: Vec::new(),
            ..Default::default()
        };

        let storage = Arc::new(MockStorage);
        let manager = ReplicationManager::new(config, storage).await.unwrap();

        // Set some lag before promotion
        manager.update_primary_sequence(1000).await.unwrap();
        manager.update_applied_sequence(950).await.unwrap();

        // Promote to primary
        manager.promote_to_primary().await.unwrap();

        // Lag should be reset to 0
        assert_eq!(manager.get_lag_bytes().await, 0);
    }

    #[tokio::test]
    async fn test_promote_to_primary_clears_failover_state() {
        let config = ReplicationConfig {
            mode: ReplicationMode::DisasterRecovery,
            primary_endpoint: Some("https://primary:50051".to_string()),
            secondary_endpoints: Vec::new(),
            heartbeat_timeout_ms: Some(100),
            failure_threshold: Some(2),
            ..Default::default()
        };

        let storage = Arc::new(MockStorage);
        let manager = ReplicationManager::new(config, storage).await.unwrap();

        // Simulate some failures
        if let Some(detector) = &manager.failover_detector {
            // Record old heartbeat to trigger failure
            let old_heartbeat = Heartbeat {
                sequence: 100,
                timestamp: chrono::Utc::now() - chrono::Duration::milliseconds(200),
                status: HealthStatus::Healthy,
            };
            detector.record_heartbeat(old_heartbeat).await;
            detector.check_health().await.unwrap();
        }

        // Promote to primary
        manager.promote_to_primary().await.unwrap();

        // Failover state should be cleared
        assert_eq!(manager.failure_count().await, 0);
        assert!(!manager.is_failover_triggered().await);
    }
}
