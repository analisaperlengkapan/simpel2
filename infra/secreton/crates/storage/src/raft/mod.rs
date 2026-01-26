use crate::{
    HealthStatus, QueryParams, SecretEntry, StorageBackend, StorageError, StorageResult,
    StorageStats, StorageTransaction,
};
#[cfg(feature = "metrics")]
use ::metrics::{counter, gauge};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub mod combined_storage;
pub mod config_builder;
pub mod metrics;
mod network;
mod state_machine;
mod types;

pub use combined_storage::SecretonRaftStorage;
pub use config_builder::RaftClusterConfigBuilder;
pub use metrics::{HealthStatus as RaftHealthStatus, MetricsCollector, RaftMetrics};

pub use network::{NetworkConfig, SecretonNetwork, SecretonNetworkFactory};
pub use state_machine::{SecretonStateMachine, StateMachineCommand, StateMachineResponse};
pub use types::{Config, Entry, LogId, Membership, NodeId, Raft, SecretonTypeConfig, Vote};

#[derive(Debug, Clone)]
pub struct RaftClusterConfig {
    pub node_id: NodeId,
    pub bind_address: String,
    pub peers: HashMap<NodeId, String>,
    pub election_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub max_payload_entries: u64,
    pub enable_tick: bool,
    /// Whether to bootstrap this node as the initial cluster
    pub bootstrap: bool,
}

impl Default for RaftClusterConfig {
    fn default() -> Self {
        let node_id = 1;
        Self {
            node_id: 1,
            bind_address: String::new(),
            peers: HashMap::new(),
            // Tuned for 5-second leader election guarantee
            // With election_timeout_max = 2x election_timeout_min
            // Worst case: 2 * 2500ms = 5000ms = 5 seconds
            election_timeout_ms: 2500,
            heartbeat_interval_ms: 750, // 30% of election timeout
            max_payload_entries: 1000,
            enable_tick: true,
            bootstrap: true, // Default to bootstrap for single-node
        }
    }
}

impl RaftClusterConfig {
    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.node_id == 0 {
            return Err("node_id cannot be 0".to_string());
        }

        if self.bind_address.trim().is_empty() {
            return Err("bind_address cannot be empty".to_string());
        }

        if self.heartbeat_interval_ms == 0 {
            return Err("heartbeat_interval_ms must be greater than 0".to_string());
        }

        if self.election_timeout_ms == 0 {
            return Err("election_timeout_ms must be greater than 0".to_string());
        }

        if self.heartbeat_interval_ms >= self.election_timeout_ms {
            return Err("heartbeat_interval_ms must be less than election_timeout_ms".to_string());
        }

        if self.max_payload_entries == 0 {
            return Err("max_payload_entries must be greater than 0".to_string());
        }

        // Check for self in peers list
        if self.peers.contains_key(&self.node_id) {
            return Err("peers list should not contain self (node_id)".to_string());
        }

        Ok(())
    }

    /// Create a builder for RaftClusterConfig
    pub fn builder(node_id: u64) -> RaftClusterConfigBuilder {
        RaftClusterConfigBuilder::new(node_id)
    }

    /// Create development configuration preset
    ///
    /// - Shorter timeouts for faster testing
    /// - Smaller payload for quick iteration
    /// - Leader election within 2 seconds worst case
    pub fn development(node_id: u64) -> Self {
        Self {
            node_id,
            bind_address: format!("127.0.0.1:{}", 8200 + node_id),
            peers: HashMap::new(),
            election_timeout_ms: 1000, // 2x = 2 seconds max
            heartbeat_interval_ms: 300,
            max_payload_entries: 100,
            enable_tick: true,
            bootstrap: true,
        }
    }

    /// Create staging configuration preset
    ///
    /// - Balanced timeouts
    /// - Moderate payload size
    /// - Leader election within 4 seconds worst case
    pub fn staging(node_id: u64) -> Self {
        Self {
            node_id,
            bind_address: format!("127.0.0.1:{}", 8200 + node_id),
            peers: HashMap::new(),
            election_timeout_ms: 2000, // 2x = 4 seconds max
            heartbeat_interval_ms: 600,
            max_payload_entries: 500,
            enable_tick: true,
            bootstrap: true,
        }
    }

    /// Create production configuration preset
    ///
    /// - Conservative timeouts for stability
    /// - Large payload for efficiency
    /// - Leader election within 5 seconds worst case (meets requirement 11.1)
    pub fn production(node_id: u64) -> Self {
        Self {
            node_id,
            bind_address: format!("127.0.0.1:{}", 8200 + node_id),
            peers: HashMap::new(),
            election_timeout_ms: 2500, // 2x = 5 seconds max
            heartbeat_interval_ms: 750,
            max_payload_entries: 2000, // Larger batches
            enable_tick: true,
            bootstrap: true,
        }
    }

    /// Create fast failover configuration preset
    ///
    /// - Optimized for rapid leader election
    /// - Leader election within 3 seconds worst case
    /// - Use in environments with reliable, low-latency networks
    pub fn fast_failover(node_id: u64) -> Self {
        Self {
            node_id,
            bind_address: format!("127.0.0.1:{}", 8200 + node_id),
            peers: HashMap::new(),
            election_timeout_ms: 1500, // 2x = 3 seconds max
            heartbeat_interval_ms: 450,
            max_payload_entries: 1000,
            enable_tick: true,
            bootstrap: true,
        }
    }

    pub fn to_openraft_config(&self) -> Config {
        Config {
            cluster_name: "secreton-cluster".to_string(),
            heartbeat_interval: self.heartbeat_interval_ms,
            election_timeout_min: self.election_timeout_ms,
            election_timeout_max: self.election_timeout_ms * 2,
            max_payload_entries: self.max_payload_entries,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct RaftStatus {
    pub node_id: NodeId,
    pub current_term: u64,
    pub leader_id: Option<NodeId>,
    pub is_leader: bool,
    pub membership: Vec<NodeId>,
    pub peer_addrs: HashMap<NodeId, String>,
    pub peer_lags: HashMap<NodeId, u64>,
    pub last_applied: Option<u64>,
    pub last_log_index: Option<u64>,
}

/// Leader election statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LeaderElectionStats {
    /// Total number of elections observed
    pub total_elections: u64,
    /// Duration of last election in milliseconds
    pub last_election_duration_ms: Option<u64>,
    /// Average election duration in milliseconds
    pub avg_election_duration_ms: Option<f64>,
    /// Whether last election met 5-second SLA
    pub meets_sla: bool,
}

/// Replication status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReplicationStatus {
    /// Whether replication is enabled
    pub enabled: bool,
    /// Replication targets status
    pub targets: Vec<ReplicationTargetStatus>,
    /// Current log index
    pub current_log_index: u64,
}

/// Replication target status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReplicationTargetStatus {
    /// Target region name
    pub region: String,
    /// Target endpoint
    pub endpoint: String,
    /// Last replicated log index
    pub last_replicated_index: u64,
    /// Replication lag in entries
    pub replication_lag: u64,
    /// Whether target is healthy (lag within tolerance)
    pub is_healthy: bool,
}

pub struct RaftCluster {
    config: RaftClusterConfig,
    raft: Arc<Raft>,
    storage: Arc<SecretonRaftStorage>,
    /// Snapshot configuration
    snapshot_config: SnapshotConfig,
    /// Leader election monitoring
    election_monitor: Arc<tokio::sync::RwLock<LeaderElectionMonitor>>,
    /// Cross-region replication configuration
    replication_config: Arc<tokio::sync::RwLock<ReplicationConfig>>,
    /// Metrics collector for tracking performance
    metrics_collector: Arc<tokio::sync::RwLock<MetricsCollector>>,
    /// Shutdown signal sender
    shutdown_tx: tokio::sync::watch::Sender<bool>,
}

/// Monitor for tracking leader election performance
#[derive(Debug, Clone)]
struct LeaderElectionMonitor {
    /// Last time a leader was elected
    last_election_time: Option<std::time::Instant>,
    /// Time taken for last election (milliseconds)
    last_election_duration_ms: Option<u64>,
    /// Total number of elections observed
    total_elections: u64,
    /// Average election time (milliseconds)
    avg_election_time_ms: f64,
}

/// Snapshot configuration
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    /// Enable automatic snapshots
    pub enabled: bool,
    /// Snapshot interval in seconds
    pub interval_secs: u64,
    /// Log entries threshold for triggering snapshot
    pub log_entries_threshold: u64,
    /// Maximum number of snapshots to retain
    pub max_snapshots: usize,
}

/// Cross-region replication configuration
#[derive(Debug, Clone)]
pub struct ReplicationConfig {
    /// Enable cross-region replication
    pub enabled: bool,
    /// Replication targets (region name -> endpoint URL)
    pub targets: HashMap<String, ReplicationTarget>,
    /// Maximum replication lag tolerance in seconds
    pub max_lag_secs: u64,
    /// Replication batch size
    pub batch_size: usize,
}

/// Replication target configuration
#[derive(Debug, Clone)]
pub struct ReplicationTarget {
    /// Target region name
    pub region: String,
    /// Target endpoint URL
    pub endpoint: String,
    /// Authentication token
    pub auth_token: Option<String>,
    /// Enable TLS
    pub tls_enabled: bool,
    /// Last replicated log index
    pub last_replicated_index: u64,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 3600, // 1 hour
            log_entries_threshold: 10000,
            max_snapshots: 5,
        }
    }
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            targets: HashMap::new(),
            max_lag_secs: 30, // 30 seconds default tolerance
            batch_size: 100,
        }
    }
}

impl RaftCluster {
    pub async fn new(config: RaftClusterConfig) -> StorageResult<Self> {
        // Create combined storage
        let storage = SecretonRaftStorage::new();
        let raft_config = Arc::new(config.to_openraft_config());

        // Create network factory with peer addresses
        let network_factory = crate::raft::SecretonNetworkFactory::new(
            config.peers.clone(),
            crate::raft::NetworkConfig::default(),
        );

        // Use Adaptor to split storage into log_store and state_machine
        let (log_store, state_machine) = openraft::storage::Adaptor::new(storage.clone());

        // Create Raft instance with split stores
        let raft = openraft::Raft::new(
            config.node_id,
            raft_config,
            network_factory,
            log_store,
            state_machine,
        )
        .await
        .map_err(|e| StorageError::BackendError {
            backend: "raft".to_string(),
            message: format!("Failed to create Raft node: {}", e),
        })?;

        let election_monitor = Arc::new(tokio::sync::RwLock::new(LeaderElectionMonitor {
            last_election_time: None,
            last_election_duration_ms: None,
            total_elections: 0,
            avg_election_time_ms: 0.0,
        }));

        let replication_config = Arc::new(tokio::sync::RwLock::new(ReplicationConfig::default()));
        let metrics_collector = Arc::new(tokio::sync::RwLock::new(MetricsCollector::new()));
        let (shutdown_tx, _) = tokio::sync::watch::channel(false);

        let cluster = Self {
            config: config.clone(),
            raft: Arc::new(raft),
            storage: Arc::new(storage),
            snapshot_config: SnapshotConfig::default(),
            election_monitor: election_monitor.clone(),
            replication_config: replication_config.clone(),
            metrics_collector: metrics_collector.clone(),
            shutdown_tx,
        };

        // Bootstrap the Raft cluster with initial membership
        // For single-node or first node, initialize with self as voter
        if config.bootstrap {
            cluster.bootstrap_cluster().await?;
        }

        // Start automatic snapshot task if enabled
        if cluster.snapshot_config.enabled {
            cluster.start_snapshot_automation();
        }

        // Start leader election monitoring
        cluster.start_election_monitoring();

        // Start node health monitoring
        cluster.start_node_health_monitoring();

        // Start cross-region replication if enabled
        cluster.start_cross_region_replication();

        Ok(cluster)
    }

    /// Bootstrap the Raft cluster with initial membership
    ///
    /// For single-node clusters, this initializes the node as both voter and leader.
    /// For multi-node clusters, this initializes the first node as the bootstrap node.
    async fn bootstrap_cluster(&self) -> StorageResult<()> {
        use std::collections::BTreeMap;

        tracing::info!(
            "Bootstrapping Raft cluster, node_id={}",
            self.config.node_id
        );

        // Create initial membership with self as the only voter
        let mut nodes: BTreeMap<NodeId, openraft::BasicNode> = BTreeMap::new();
        nodes.insert(
            self.config.node_id,
            openraft::BasicNode {
                addr: self.config.bind_address.clone(),
            },
        );

        // Initialize the cluster with self as the first voter
        match self.raft.initialize(nodes).await {
            Ok(_) => {
                tracing::info!(
                    "Raft cluster bootstrapped successfully, node {} is now a voter",
                    self.config.node_id
                );

                // Wait for leader election
                for _ in 0..50 {
                    // 5 seconds max
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    let metrics = self.raft.metrics().borrow().clone();
                    if metrics.current_leader.is_some() {
                        tracing::info!("Leader elected: node {}", metrics.current_leader.unwrap());
                        return Ok(());
                    }
                }

                tracing::warn!("Leader election timed out, but cluster is initialized");
                Ok(())
            }
            Err(e) => {
                // If already initialized, this is fine
                let error_msg = format!("{:?}", e);
                if error_msg.contains("NotAllowed")
                    || error_msg.contains("already initialized")
                    || error_msg.contains("vote is not None")
                {
                    tracing::debug!("Raft cluster already initialized");
                    Ok(())
                } else {
                    Err(StorageError::BackendError {
                        backend: "raft".to_string(),
                        message: format!("Failed to bootstrap cluster: {:?}", e),
                    })
                }
            }
        }
    }

    /// Start cross-region replication task
    fn start_cross_region_replication(&self) {
        let raft = Arc::clone(&self.raft);
        let storage = Arc::clone(&self.storage);
        let replication_config = Arc::clone(&self.replication_config);
        let node_id = self.config.node_id;
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            loop {
                // Wait for interval or shutdown
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        tracing::info!("Stopping replication task");
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(5)) => {}
                }

                // Check if replication is enabled
                let config = replication_config.read().await;
                if !config.enabled || config.targets.is_empty() {
                    drop(config);
                    // Sleep is handled at start of loop
                    continue;
                }

                let targets = config.targets.clone();
                let batch_size = config.batch_size;
                let max_lag_secs = config.max_lag_secs;
                drop(config);

                // Only leader performs replication
                let metrics = raft.metrics().borrow().clone();
                if metrics.current_leader != Some(node_id) {
                    continue;
                }

                // Replicate to each target
                for (region, target) in targets.iter() {
                    match replicate_to_target(&storage, target, batch_size).await {
                        Ok(replicated_count) => {
                            if replicated_count > 0 {
                                tracing::info!(
                                    "Replicated {} entries to region {}",
                                    replicated_count,
                                    region
                                );

                                #[cfg(feature = "metrics")]
                                {
                                    counter!("secreton_raft_replication_entries", "region" => region.clone())
                                        .increment(replicated_count as u64);
                                }
                            }

                            // Check replication lag
                            let current_index = metrics.last_log_index.unwrap_or(0);
                            let lag = current_index.saturating_sub(target.last_replicated_index);

                            if lag > (max_lag_secs * 10) {
                                // Assuming ~10 entries per second
                                tracing::warn!(
                                    "High replication lag to region {}: {} entries behind",
                                    region,
                                    lag
                                );

                                #[cfg(feature = "metrics")]
                                {
                                    gauge!("secreton_raft_replication_lag", "region" => region.clone())
                                        .set(lag as f64);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to replicate to region {}: {}", region, e);

                            #[cfg(feature = "metrics")]
                            {
                                counter!("secreton_raft_replication_errors", "region" => region.clone())
                                    .increment(1);
                            }
                        }
                    }
                }
            }
        });
    }

    /// Start monitoring node health and connectivity
    fn start_node_health_monitoring(&self) {
        let raft = Arc::clone(&self.raft);
        let _node_id = self.config.node_id;
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            let mut last_leader: Option<NodeId> = None;

            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        tracing::info!("Stopping node health monitoring");
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                }

                let metrics = raft.metrics().borrow().clone();
                let current_leader = metrics.current_leader;

                // Detect leader change
                if current_leader != last_leader {
                    if let Some(new_leader) = current_leader {
                        if last_leader.is_some() {
                            tracing::warn!("Leader changed: {:?} -> {}", last_leader, new_leader);

                            #[cfg(feature = "metrics")]
                            {
                                counter!("secreton_raft_leader_changes").increment(1);
                            }
                        }
                    } else if last_leader.is_some() {
                        tracing::error!("Leader lost, no current leader");

                        #[cfg(feature = "metrics")]
                        {
                            counter!("secreton_raft_leader_lost").increment(1);
                            gauge!("secreton_raft_has_leader").set(0.0);
                        }
                    }

                    last_leader = current_leader;
                }

                // Update leader presence metric
                #[cfg(feature = "metrics")]
                {
                    gauge!("secreton_raft_has_leader").set(if current_leader.is_some() {
                        1.0
                    } else {
                        0.0
                    });
                }

                // Check for stale metrics (potential node isolation)
                if let Some(last_applied) = metrics.last_applied {
                    let log_index = metrics.last_log_index.unwrap_or(0);
                    let lag = log_index.saturating_sub(last_applied.index);

                    if lag > 1000 {
                        tracing::warn!("High replication lag detected: {} entries behind", lag);

                        #[cfg(feature = "metrics")]
                        {
                            gauge!("secreton_raft_replication_lag").set(lag as f64);
                        }
                    }
                }
            }
        });
    }

    /// Start monitoring leader elections
    fn start_election_monitoring(&self) {
        let raft = Arc::clone(&self.raft);
        let monitor = Arc::clone(&self.election_monitor);
        let _node_id = self.config.node_id;
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            let mut last_term = 0u64;
            let mut election_start: Option<std::time::Instant> = None;

            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        tracing::info!("Stopping election monitoring");
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                }

                let metrics = raft.metrics().borrow().clone();
                let current_term = metrics.current_term;

                // Check for leader election completion
                if let Some(leader_id) = metrics.current_leader {
                    if let Some(start_time) = election_start {
                        // Election finished
                        let duration = start_time.elapsed();
                        let duration_ms = duration.as_millis() as u64;

                        tracing::info!(
                            "Leader elected: node {} in term {} (took {}ms)",
                            leader_id,
                            current_term,
                            duration_ms
                        );

                        // Update monitor
                        let mut mon = monitor.write().await;
                        mon.last_election_time = Some(std::time::Instant::now());
                        mon.last_election_duration_ms = Some(duration_ms);
                        mon.total_elections += 1;

                        // Update running average
                        if mon.total_elections == 1 {
                            mon.avg_election_time_ms = duration_ms as f64;
                        } else {
                            mon.avg_election_time_ms = (mon.avg_election_time_ms
                                * (mon.total_elections - 1) as f64
                                + duration_ms as f64)
                                / mon.total_elections as f64;
                        }

                        // Record metrics
                        #[cfg(feature = "metrics")]
                        {
                            gauge!("secreton_raft_last_election_duration_ms")
                                .set(duration_ms as f64);
                            gauge!("secreton_raft_avg_election_duration_ms")
                                .set(mon.avg_election_time_ms);
                            counter!("secreton_raft_elections_total").increment(1);

                            // Alert if election took too long (> 5 seconds)
                            if duration_ms > 5000 {
                                counter!("secreton_raft_slow_elections_total").increment(1);
                                tracing::warn!(
                                    "Slow leader election detected: {}ms (threshold: 5000ms)",
                                    duration_ms
                                );
                            }
                        }

                        // Reset for next election
                        election_start = None;
                    }
                }

                // Detect term change (potential election)
                if current_term > last_term {
                    if metrics.current_leader.is_none() {
                        // Election started
                        election_start = Some(std::time::Instant::now());
                        tracing::info!(
                            "Leader election started: term {} -> {}",
                            last_term,
                            current_term
                        );
                    } else {
                        // Leader already exists in new term (e.g. we joined as follower)
                        election_start = None;
                    }
                    last_term = current_term;
                } else if metrics.current_leader.is_none() && election_start.is_none() {
                    // No leader and no timer? Start one.
                    // This handles cases where we lose a leader in the same term
                    // or if we restart without a leader
                    election_start = Some(std::time::Instant::now());
                }
            }
        });
    }

    /// Start automatic snapshot creation task
    /// Start automatic snapshot creation task
    fn start_snapshot_automation(&self) {
        let raft = Arc::clone(&self.raft);
        let config = self.snapshot_config.clone();
        let _node_id = self.config.node_id;
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            let interval = Duration::from_secs(config.interval_secs);

            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        tracing::info!("Stopping snapshot automation");
                        break;
                    }
                    _ = tokio::time::sleep(interval) => {}
                }

                // All nodes create snapshots for log compaction
                let metrics = raft.metrics().borrow().clone();
                let log_size = metrics.last_log_index.unwrap_or(0);
                let last_snapshot_index = metrics.snapshot.as_ref().map(|s| s.index).unwrap_or(0);
                let entries_since_snapshot = log_size.saturating_sub(last_snapshot_index);

                if entries_since_snapshot >= config.log_entries_threshold {
                    tracing::info!(
                        "Creating automatic snapshot: {} entries since last snapshot (threshold: {})",
                        entries_since_snapshot,
                        config.log_entries_threshold
                    );

                    match raft.trigger().snapshot().await {
                        Ok(_) => {
                            tracing::info!("Snapshot created successfully at index {}", log_size);

                            #[cfg(feature = "metrics")]
                            {
                                counter!("secreton_raft_snapshots_created").increment(1);
                                gauge!("secreton_raft_last_snapshot_index").set(log_size as f64);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to create snapshot: {}", e);

                            #[cfg(feature = "metrics")]
                            {
                                counter!("secreton_raft_snapshot_errors").increment(1);
                            }
                        }
                    }
                } else {
                    tracing::debug!(
                        "Snapshot not needed: {} entries since last snapshot (threshold: {})",
                        entries_since_snapshot,
                        config.log_entries_threshold
                    );
                }
            }
        });
    }

    pub async fn status(&self) -> StorageResult<RaftStatus> {
        let metrics = self.raft.metrics().borrow().clone();

        // Extract peer addresses from membership
        let mut peer_addrs = HashMap::new();
        for (node_id, node) in metrics.membership_config.membership().nodes() {
            peer_addrs.insert(*node_id, node.addr.clone());
        }

        // Extract replication lag if leader
        let mut peer_lags = HashMap::new();
        if metrics.current_leader == Some(self.config.node_id) {
            if let Some(replication) = &metrics.replication {
                let current_index = metrics.last_log_index.unwrap_or(0);
                for (node_id, matched_log_id) in replication.iter() {
                    let matched_index = matched_log_id.map(|l| l.index).unwrap_or(0);
                    let lag = current_index.saturating_sub(matched_index);
                    peer_lags.insert(*node_id, lag);
                }
            }
        }

        Ok(RaftStatus {
            node_id: self.config.node_id,
            current_term: metrics.current_term,
            leader_id: metrics.current_leader,
            is_leader: metrics.current_leader == Some(self.config.node_id),
            membership: metrics.membership_config.membership().voter_ids().collect(),
            peer_addrs,
            peer_lags,
            last_applied: metrics.last_applied.map(|l| l.index),
            last_log_index: metrics.last_log_index,
        })
    }

    pub async fn is_leader(&self) -> bool {
        let metrics = self.raft.metrics().borrow().clone();
        metrics.current_leader == Some(self.config.node_id)
    }

    /// Get access to the underlying storage for metrics/monitoring
    pub fn get_storage(&self) -> Arc<SecretonRaftStorage> {
        Arc::clone(&self.storage)
    }

    /// Get metrics snapshot for monitoring
    pub async fn get_metrics(&self) -> StorageResult<RaftMetrics> {
        let raft_metrics = self.raft.metrics().borrow().clone();
        let avg_latency = self.metrics_collector.read().await.avg_commit_latency();

        Ok(RaftMetrics {
            node_id: self.config.node_id,
            current_term: raft_metrics.current_term,
            leader_id: raft_metrics.current_leader,
            is_leader: raft_metrics.current_leader == Some(self.config.node_id),
            leader_elections: raft_metrics.current_term, // Approximate
            last_applied_index: raft_metrics.last_applied.map(|l| l.index),
            last_log_index: raft_metrics.last_log_index,
            log_size: raft_metrics.last_log_index.unwrap_or(0),
            cluster_size: raft_metrics
                .membership_config
                .membership()
                .voter_ids()
                .count(),
            committed_entries: raft_metrics.last_applied.map(|l| l.index).unwrap_or(0),
            avg_commit_latency_ms: avg_latency,
            snapshots_created: 0, // TODO: Track snapshots
            last_snapshot_index: raft_metrics.snapshot.map(|meta| meta.index),
            health: if raft_metrics.current_leader.is_some() {
                metrics::HealthStatus::Healthy
            } else {
                metrics::HealthStatus::Degraded
            },
        })
    }

    pub async fn leader_id(&self) -> Option<NodeId> {
        let metrics = self.raft.metrics().borrow().clone();
        metrics.current_leader
    }

    /// Get leader election statistics
    pub async fn get_election_stats(&self) -> LeaderElectionStats {
        let monitor = self.election_monitor.read().await;
        LeaderElectionStats {
            total_elections: monitor.total_elections,
            last_election_duration_ms: monitor.last_election_duration_ms,
            avg_election_duration_ms: if monitor.total_elections > 0 {
                Some(monitor.avg_election_time_ms)
            } else {
                None
            },
            meets_sla: monitor
                .last_election_duration_ms
                .map(|d| d <= 5000)
                .unwrap_or(true),
        }
    }

    /// Check if cluster is healthy and can serve requests
    ///
    /// A cluster is considered healthy if:
    /// - A leader is elected
    /// - This node can communicate with the leader
    /// - Replication lag is acceptable
    pub async fn is_cluster_healthy(&self) -> bool {
        let metrics = self.raft.metrics().borrow().clone();

        // Check 1: Leader must be elected
        if metrics.current_leader.is_none() {
            tracing::warn!("Cluster unhealthy: no leader elected");
            return false;
        }

        // Check 2: Check replication lag
        if let Some(last_applied) = metrics.last_applied {
            let log_index = metrics.last_log_index.unwrap_or(0);
            let lag = log_index.saturating_sub(last_applied.index);

            // If lag is too high (> 10000 entries), cluster is degraded
            if lag > 10000 {
                tracing::warn!("Cluster degraded: high replication lag ({})", lag);
                return false;
            }
        }

        true
    }

    /// Get the leader address for client forwarding
    pub async fn get_leader_address(&self) -> Option<String> {
        let leader_id = self.leader_id().await?;

        // If this node is the leader, return None (no forwarding needed)
        if leader_id == self.config.node_id {
            return None;
        }

        // Look up leader address in dynamic membership
        let metrics = self.raft.metrics().borrow().clone();
        metrics
            .membership_config
            .membership()
            .nodes()
            .find(|(id, _)| **id == leader_id)
            .map(|(_, node)| node.addr.clone())
    }

    /// Add a replication target
    pub async fn add_replication_target(
        &self,
        region: String,
        endpoint: String,
        auth_token: Option<String>,
        tls_enabled: bool,
    ) -> StorageResult<()> {
        let mut config = self.replication_config.write().await;

        let target = ReplicationTarget {
            region: region.clone(),
            endpoint,
            auth_token,
            tls_enabled,
            last_replicated_index: 0,
        };

        config.targets.insert(region.clone(), target);

        tracing::info!("Added replication target for region: {}", region);

        Ok(())
    }

    /// Remove a replication target
    pub async fn remove_replication_target(&self, region: &str) -> StorageResult<()> {
        let mut config = self.replication_config.write().await;

        if config.targets.remove(region).is_some() {
            tracing::info!("Removed replication target for region: {}", region);
            Ok(())
        } else {
            Err(StorageError::NotFound {
                resource_type: "ReplicationTarget".to_string(),
                id: region.to_string(),
            })
        }
    }

    /// Enable cross-region replication
    pub async fn enable_replication(&self) -> StorageResult<()> {
        let mut config = self.replication_config.write().await;
        config.enabled = true;
        tracing::info!("Cross-region replication enabled");
        Ok(())
    }

    /// Disable cross-region replication
    pub async fn disable_replication(&self) -> StorageResult<()> {
        let mut config = self.replication_config.write().await;
        config.enabled = false;
        tracing::info!("Cross-region replication disabled");
        Ok(())
    }

    /// Get replication status
    pub async fn get_replication_status(&self) -> ReplicationStatus {
        let config = self.replication_config.read().await;
        let metrics = self.raft.metrics().borrow().clone();
        let current_index = metrics.last_log_index.unwrap_or(0);

        let targets: Vec<ReplicationTargetStatus> = config
            .targets
            .iter()
            .map(|(region, target)| {
                let lag = current_index.saturating_sub(target.last_replicated_index);
                ReplicationTargetStatus {
                    region: region.clone(),
                    endpoint: target.endpoint.clone(),
                    last_replicated_index: target.last_replicated_index,
                    replication_lag: lag,
                    is_healthy: lag <= (config.max_lag_secs * 10),
                }
            })
            .collect();

        ReplicationStatus {
            enabled: config.enabled,
            targets,
            current_log_index: current_index,
        }
    }

    pub async fn propose(&self, data: StateMachineCommand) -> StorageResult<StateMachineResponse> {
        self.propose_with_retry(data, 3).await
    }

    /// Propose a command with automatic retry and leader forwarding
    ///
    /// This method implements automatic request routing on node failure:
    /// - Retries on transient failures
    /// - Automatically forwards to leader if this node is not the leader
    /// - Handles leader election in progress
    pub async fn propose_with_retry(
        &self,
        data: StateMachineCommand,
        max_retries: usize,
    ) -> StorageResult<StateMachineResponse> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < max_retries {
            attempts += 1;

            let start = std::time::Instant::now();
            match self.raft.client_write(data.clone()).await {
                Ok(response) => {
                    let duration = start.elapsed();
                    let collector = self.metrics_collector.clone();
                    tokio::spawn(async move {
                        collector.write().await.record_commit_latency(duration);
                    });

                    #[cfg(feature = "metrics")]
                    {
                        counter!("secreton_raft_propose_success").increment(1);
                        if attempts > 1 {
                            counter!("secreton_raft_propose_retries_success").increment(1);
                        }
                    }
                    return Ok(response.data);
                }
                Err(e) => {
                    last_error = Some(e.to_string());

                    // Check if error is due to not being leader
                    let error_str = e.to_string();
                    if error_str.contains("not leader") || error_str.contains("ForwardToLeader") {
                        tracing::warn!(
                            "Node {} is not leader, waiting for leader election (attempt {}/{})",
                            self.config.node_id,
                            attempts,
                            max_retries
                        );

                        #[cfg(feature = "metrics")]
                        {
                            counter!("secreton_raft_forward_to_leader").increment(1);
                        }

                        // Wait for leader election
                        let leader_elected = self.wait_for_leader(Duration::from_secs(5)).await;

                        if !leader_elected {
                            tracing::error!(
                                "No leader elected after waiting (attempt {}/{})",
                                attempts,
                                max_retries
                            );

                            if attempts >= max_retries {
                                break;
                            }

                            // Exponential backoff
                            let backoff =
                                Duration::from_millis(100 * 2u64.pow(attempts as u32 - 1));
                            tokio::time::sleep(backoff).await;
                            continue;
                        }

                        // Leader elected, retry immediately
                        tracing::info!("Leader elected, retrying request");
                        continue;
                    }

                    // For other errors, use exponential backoff
                    tracing::warn!(
                        "Propose failed: {} (attempt {}/{})",
                        error_str,
                        attempts,
                        max_retries
                    );

                    if attempts >= max_retries {
                        break;
                    }

                    // Exponential backoff
                    let backoff = Duration::from_millis(100 * 2u64.pow(attempts as u32 - 1));
                    tokio::time::sleep(backoff).await;
                }
            }
        }

        #[cfg(feature = "metrics")]
        {
            counter!("secreton_raft_propose_failures").increment(1);
        }

        Err(StorageError::ReplicationError(format!(
            "Failed to propose command after {} attempts: {}",
            max_retries,
            last_error.unwrap_or_else(|| "unknown error".to_string())
        )))
    }

    /// Wait for a leader to be elected
    ///
    /// Returns true if a leader is elected within the timeout, false otherwise
    async fn wait_for_leader(&self, timeout: Duration) -> bool {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            let metrics = self.raft.metrics().borrow().clone();

            if metrics.current_leader.is_some() {
                tracing::info!(
                    "Leader elected: node {} in term {}",
                    metrics.current_leader.unwrap(),
                    metrics.current_term
                );
                return true;
            }

            // Check every 100ms
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        false
    }
    pub async fn add_node(&self, node_id: NodeId, address: String) -> StorageResult<()> {
        // First add as learner
        let node = openraft::BasicNode {
            addr: address.clone(),
        };
        self.raft
            .add_learner(node_id, node, true)
            .await
            .map_err(|e| StorageError::ReplicationError(format!("Failed to add learner: {}", e)))?;

        // Then update membership
        let mut members = self.get_current_members().await?;
        members.insert(node_id, openraft::BasicNode { addr: address });

        let member_ids: std::collections::BTreeSet<NodeId> = members.keys().cloned().collect();
        self.raft
            .change_membership(member_ids, false)
            .await
            .map_err(|e| StorageError::ReplicationError(format!("Failed to add node: {}", e)))?;
        Ok(())
    }

    pub async fn remove_node(&self, node_id: NodeId) -> StorageResult<()> {
        let mut members = self.get_current_members().await?;
        members.remove(&node_id);

        let member_ids: std::collections::BTreeSet<NodeId> = members.keys().cloned().collect();
        self.raft
            .change_membership(member_ids, false)
            .await
            .map_err(|e| StorageError::ReplicationError(format!("Failed to remove node: {}", e)))?;
        Ok(())
    }

    async fn get_current_members(&self) -> StorageResult<HashMap<NodeId, openraft::BasicNode>> {
        let metrics = self.raft.metrics().borrow().clone();
        let mut members = HashMap::new();

        // Use the addresses directly from the current membership configuration
        // This ensures dynamically added nodes are included
        for (node_id, node) in metrics.membership_config.membership().nodes() {
            members.insert(*node_id, node.clone());
        }

        Ok(members)
    }

    /// Manually trigger a snapshot
    pub async fn create_snapshot(&self) -> StorageResult<()> {
        tracing::info!("Manually triggering Raft snapshot");

        self.raft
            .trigger()
            .snapshot()
            .await
            .map_err(|e| StorageError::BackendError {
                backend: "raft".to_string(),
                message: format!("Failed to create snapshot: {}", e),
            })?;

        tracing::info!("Snapshot created successfully");
        Ok(())
    }

    pub async fn shutdown(&self) -> StorageResult<()> {
        // Signal background tasks to stop
        let _ = self.shutdown_tx.send(true);

        self.raft
            .shutdown()
            .await
            .map_err(|e| StorageError::BackendError {
                backend: "raft".to_string(),
                message: format!("Failed to shutdown Raft: {}", e),
            })?;
        Ok(())
    }

    /// Get reference to self as Any for downcasting
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl StorageBackend for RaftCluster {
    async fn store(&self, entry: &SecretEntry) -> StorageResult<()> {
        if entry.path.trim().is_empty() {
            return Err(StorageError::InvalidQuery {
                message: "Path cannot be empty".to_string(),
            });
        }

        let response = self
            .propose(StateMachineCommand::Store(entry.clone()))
            .await?;

        match response {
            StateMachineResponse::Stored(_) | StateMachineResponse::Success => Ok(()),
            StateMachineResponse::Error(msg) => Err(StorageError::ReplicationError(format!(
                "Raft store failed: {}",
                msg
            ))),
            other => Err(StorageError::ReplicationError(format!(
                "Unexpected Raft response for store: {:?}",
                other
            ))),
        }
    }

    async fn get_by_id(&self, id: Uuid) -> StorageResult<Option<SecretEntry>> {
        let entry = self.storage.get_entry_by_id(id).await;
        Ok(entry)
    }

    async fn get_by_path(&self, path: &str) -> StorageResult<Option<SecretEntry>> {
        let entry = self.storage.get_entry_by_path(path).await;
        Ok(entry)
    }

    async fn update(&self, entry: &SecretEntry) -> StorageResult<()> {
        let response = self
            .propose(StateMachineCommand::Update(entry.clone()))
            .await?;

        match response {
            StateMachineResponse::Updated(_) | StateMachineResponse::Success => Ok(()),
            StateMachineResponse::Error(msg) if msg == "Entry not found" => {
                Err(StorageError::NotFound {
                    resource_type: "SecretEntry".to_string(),
                    id: entry.id.to_string(),
                })
            }
            StateMachineResponse::Error(msg) => Err(StorageError::ReplicationError(format!(
                "Raft update failed: {}",
                msg
            ))),
            other => Err(StorageError::ReplicationError(format!(
                "Unexpected Raft response for update: {:?}",
                other
            ))),
        }
    }

    async fn delete_by_id(&self, id: Uuid) -> StorageResult<bool> {
        let response = self.propose(StateMachineCommand::Delete(id)).await?;

        match response {
            StateMachineResponse::Deleted(existed) => Ok(existed),
            StateMachineResponse::Error(msg) => Err(StorageError::ReplicationError(format!(
                "Raft delete_by_id failed: {}",
                msg
            ))),
            other => Err(StorageError::ReplicationError(format!(
                "Unexpected Raft response for delete_by_id: {:?}",
                other
            ))),
        }
    }

    async fn delete_by_path(&self, path: &str) -> StorageResult<bool> {
        let response = self
            .propose(StateMachineCommand::DeleteByPath(path.to_string()))
            .await?;

        match response {
            StateMachineResponse::Deleted(existed) => Ok(existed),
            StateMachineResponse::Error(msg) => Err(StorageError::ReplicationError(format!(
                "Raft delete_by_path failed: {}",
                msg
            ))),
            other => Err(StorageError::ReplicationError(format!(
                "Unexpected Raft response for delete_by_path: {:?}",
                other
            ))),
        }
    }

    async fn list(&self, params: &QueryParams) -> StorageResult<Vec<SecretEntry>> {
        let entries = self.storage.list_entries(params).await;
        Ok(entries)
    }

    async fn count(&self, params: &QueryParams) -> StorageResult<u64> {
        let count = self.storage.count_entries(params).await;
        Ok(count)
    }

    async fn exists(&self, path: &str) -> StorageResult<bool> {
        let exists = self.storage.exists_path(path).await;
        Ok(exists)
    }

    async fn begin_transaction(&self) -> StorageResult<Box<dyn StorageTransaction>> {
        Err(StorageError::TransactionNotSupported {
            backend: "raft".to_string(),
        })
    }

    async fn health_check(&self) -> StorageResult<HealthStatus> {
        let metrics = self.get_metrics().await?;

        let is_healthy = matches!(metrics.health, RaftHealthStatus::Healthy);

        Ok(HealthStatus {
            is_healthy,
            response_time_ms: 0.0,
            connections_active: 1,
            connections_idle: 0,
            last_error: None,
            uptime_seconds: 0,
        })
    }

    async fn get_stats(&self) -> StorageResult<StorageStats> {
        let entries = self.storage.list_entries(&QueryParams::new()).await;
        let total_entries = entries.len() as u64;
        let total_size_bytes: u64 = entries.iter().map(|e| e.encrypted_data.len() as u64).sum();

        let average_entry_size = if total_entries > 0 {
            total_size_bytes as f64 / total_entries as f64
        } else {
            0.0
        };

        let mut entries_by_security_level = HashMap::new();
        let mut expired_entries = 0u64;

        for entry in &entries {
            *entries_by_security_level
                .entry(entry.security_level)
                .or_insert(0) += 1;

            if entry.is_expired() {
                expired_entries += 1;
            }
        }

        Ok(StorageStats {
            backend_type: "raft".to_string(),
            total_entries,
            total_size_bytes,
            average_entry_size,
            entries_by_security_level,
            entries_created_today: 0,
            entries_updated_today: 0,
            expired_entries,
            last_backup: None,
            metadata: serde_json::json!({
                "node_id": self.config.node_id,
            }),
        })
    }

    async fn migrate(&self) -> StorageResult<()> {
        // Raft-based storage does not require schema migrations
        Ok(())
    }

    async fn delete_expired(&self) -> StorageResult<u64> {
        let response = self
            .propose(StateMachineCommand::DeleteExpired(chrono::Utc::now()))
            .await?;

        match response {
            StateMachineResponse::DeletedExpired(count) => Ok(count),
            StateMachineResponse::Error(msg) => Err(StorageError::ReplicationError(format!(
                "Raft delete_expired failed: {}",
                msg
            ))),
            other => Err(StorageError::ReplicationError(format!(
                "Unexpected Raft response for delete_expired: {:?}",
                other
            ))),
        }
    }

    async fn compact(&self) -> StorageResult<()> {
        self.create_snapshot().await
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Replicate entries to a target region
///
/// This is a simplified implementation that would need to be enhanced
/// for production use with proper error handling, retry logic, and
/// authentication.
async fn replicate_to_target(
    _storage: &Arc<SecretonRaftStorage>,
    target: &ReplicationTarget,
    _batch_size: usize,
) -> StorageResult<usize> {
    // In a real implementation, this would:
    // 1. Fetch entries from storage starting at target.last_replicated_index
    // 2. Send entries to target endpoint via HTTP/gRPC
    // 3. Update target.last_replicated_index on success
    // 4. Handle authentication with target.auth_token
    // 5. Use TLS if target.tls_enabled

    tracing::debug!(
        "Replicating to {} (last index: {})",
        target.region,
        target.last_replicated_index
    );

    // Placeholder: In production, implement actual replication logic
    // For now, return 0 to indicate no entries replicated
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cluster_creation() {
        let config = RaftClusterConfig::default();
        let cluster = RaftCluster::new(config).await;
        assert!(cluster.is_ok());
    }

    #[tokio::test]
    async fn test_cluster_status() {
        let config = RaftClusterConfig::default();
        let cluster = RaftCluster::new(config).await.unwrap();
        let status = cluster.status().await;
        assert!(status.is_ok());
    }

    #[tokio::test]
    async fn test_replication_config() {
        let config = RaftClusterConfig::default();
        let cluster = RaftCluster::new(config).await.unwrap();

        // Test adding replication target
        cluster
            .add_replication_target(
                "us-west-2".to_string(),
                "https://secreton-us-west-2.example.com".to_string(),
                Some("token123".to_string()),
                true,
            )
            .await
            .unwrap();

        // Test getting replication status
        let status = cluster.get_replication_status().await;
        assert_eq!(status.targets.len(), 1);
        assert_eq!(status.targets[0].region, "us-west-2");

        // Test removing replication target
        cluster
            .remove_replication_target("us-west-2")
            .await
            .unwrap();

        let status = cluster.get_replication_status().await;
        assert_eq!(status.targets.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_expired() {
        let config = RaftClusterConfig::development(1);
        let cluster = RaftCluster::new(config).await.unwrap();

        // Wait for leader election
        for _ in 0..50 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if cluster.is_leader().await {
                break;
            }
        }
        assert!(cluster.is_leader().await, "Cluster failed to elect leader");

        // Create expired entry
        let expired_entry = SecretEntry::new(
            "expired/path".to_string(),
            vec![1, 2, 3],
            serde_json::json!({}),
            crate::SecurityLevel::Internal,
            "system".to_string(),
        )
        .with_expiration(chrono::Utc::now() - chrono::Duration::hours(1));

        cluster.store(&expired_entry).await.unwrap();

        // Create valid entry
        let valid_entry = SecretEntry::new(
            "valid/path".to_string(),
            vec![4, 5, 6],
            serde_json::json!({}),
            crate::SecurityLevel::Internal,
            "system".to_string(),
        )
        .with_expiration(chrono::Utc::now() + chrono::Duration::hours(1));

        cluster.store(&valid_entry).await.unwrap();

        // Verify both exist
        assert!(cluster.exists("expired/path").await.unwrap());
        assert!(cluster.exists("valid/path").await.unwrap());

        // Delete expired
        let count = cluster.delete_expired().await.unwrap();
        assert_eq!(count, 1);

        // Verify expired is gone
        assert!(!cluster.exists("expired/path").await.unwrap());

        // Verify valid remains
        assert!(cluster.exists("valid/path").await.unwrap());
    }
}
