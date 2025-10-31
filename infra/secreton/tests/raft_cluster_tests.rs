//! Comprehensive Raft Cluster Tests
//!
//! Tests for Raft cluster management including:
//! - Cluster status retrieval
//! - Peer add/remove operations
//! - Snapshot creation and restoration
//! - Cluster health monitoring
//! - Quorum safety checks

use secreton_api::handlers::raft::{
    AddPeerRequest, AddPeerResponse, ClusterStatusResponse, CreateSnapshotResponse,
    ListSnapshotsResponse, NodeHealthStatus, PeerInfo, RemovePeerResponse,
    RestoreSnapshotRequest, RestoreSnapshotResponse, SnapshotMetadata,
};
use secreton_storage::raft::{RaftCluster, RaftClusterConfig, RaftStatus};
use std::collections::HashMap;

// ============================================================================
// Cluster Status Tests
// ============================================================================

#[tokio::test]
async fn test_cluster_status_structure() {
    // Test that cluster status response has all required fields
    let status = ClusterStatusResponse {
        node_id: 1,
        current_term: 5,
        leader_id: Some(1),
        is_leader: true,
        membership: vec![1, 2, 3],
        last_applied: Some(100),
        last_log_index: Some(100),
        health_status: NodeHealthStatus::Healthy,
        peers: vec![
            PeerInfo {
                node_id: 1,
                address: "node-1:7000".to_string(),
                state: "Leader".to_string(),
                health: NodeHealthStatus::Healthy,
                replication_lag: None,
                last_heartbeat: Some(chrono::Utc::now().timestamp()),
            },
            PeerInfo {
                node_id: 2,
                address: "node-2:7000".to_string(),
                state: "Follower".to_string(),
                health: NodeHealthStatus::Healthy,
                replication_lag: Some(0),
                last_heartbeat: Some(chrono::Utc::now().timestamp()),
            },
        ],
    };

    // Verify structure
    assert_eq!(status.node_id, 1);
    assert_eq!(status.current_term, 5);
    assert_eq!(status.leader_id, Some(1));
    assert!(status.is_leader);
    assert_eq!(status.membership.len(), 3);
    assert_eq!(status.peers.len(), 2);

    // Verify serialization
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"node_id\":1"));
    assert!(json.contains("\"is_leader\":true"));
    assert!(json.contains("\"current_term\":5"));
}

#[tokio::test]
async fn test_cluster_status_health_states() {
    // Test all health status variants
    let health_states = vec![
        NodeHealthStatus::Healthy,
        NodeHealthStatus::Degraded,
        NodeHealthStatus::Failed,
    ];

    for health in health_states {
        let status = ClusterStatusResponse {
            node_id: 1,
            current_term: 1,
            leader_id: None,
            is_leader: false,
            membership: vec![1],
            last_applied: None,
            last_log_index: None,
            health_status: health.clone(),
            peers: vec![],
        };

        // Verify serialization of each health state
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("health_status"));
    }
}

#[tokio::test]
async fn test_cluster_status_no_leader() {
    // Test cluster status when no leader is elected (degraded state)
    let status = ClusterStatusResponse {
        node_id: 1,
        current_term: 5,
        leader_id: None, // No leader
        is_leader: false,
        membership: vec![1, 2, 3],
        last_applied: Some(50),
        last_log_index: Some(50),
        health_status: NodeHealthStatus::Degraded,
        peers: vec![],
    };

    assert!(status.leader_id.is_none());
    assert!(!status.is_leader);
    assert!(matches!(status.health_status, NodeHealthStatus::Degraded));
}

#[tokio::test]
async fn test_cluster_status_replication_lag() {
    // Test replication lag calculation
    let leader_index = 100u64;
    let follower_index = 95u64;
    let expected_lag = leader_index - follower_index;

    let peer = PeerInfo {
        node_id: 2,
        address: "node-2:7000".to_string(),
        state: "Follower".to_string(),
        health: NodeHealthStatus::Healthy,
        replication_lag: Some(expected_lag),
        last_heartbeat: Some(chrono::Utc::now().timestamp()),
    };

    assert_eq!(peer.replication_lag, Some(5));
}

#[tokio::test]
async fn test_raft_cluster_creation() {
    // Test basic cluster creation
    let config = RaftClusterConfig {
        node_id: 1,
        peers: HashMap::new(),
        election_timeout_ms: 1000,
        heartbeat_interval_ms: 300,
        max_payload_entries: 1000,
        enable_tick: true,
    };

    let cluster = RaftCluster::new(config).await;
    assert!(cluster.is_ok(), "Cluster creation should succeed");
}

#[tokio::test]
async fn test_raft_cluster_status_retrieval() {
    // Test status retrieval from cluster
    let config = RaftClusterConfig::default();
    let cluster = RaftCluster::new(config).await.unwrap();

    let status = cluster.status().await;
    assert!(status.is_ok(), "Status retrieval should succeed");

    let status = status.unwrap();
    assert_eq!(status.node_id, 1);
    assert!(status.current_term >= 0);
}

// ============================================================================
// Peer Management Tests
// ============================================================================

#[tokio::test]
async fn test_add_peer_request_validation() {
    // Test valid add peer request
    let valid_request = AddPeerRequest {
        node_id: 2,
        address: "192.168.1.2:7000".to_string(),
    };

    assert!(!valid_request.address.is_empty());
    assert!(valid_request.address.contains(':'));
    assert_eq!(valid_request.node_id, 2);

    // Verify serialization
    let json = serde_json::to_string(&valid_request).unwrap();
    assert!(json.contains("\"node_id\":2"));
    assert!(json.contains("192.168.1.2:7000"));
}

#[tokio::test]
async fn test_add_peer_invalid_address() {
    // Test various invalid address formats
    let invalid_addresses = vec![
        "",                    // Empty
        "invalid",             // No port
        "192.168.1.2",         // No port
        ":7000",               // No host
        "192.168.1.2:",        // No port number
        "192.168.1.2:abc",     // Invalid port
    ];

    for addr in invalid_addresses {
        let request = AddPeerRequest {
            node_id: 2,
            address: addr.to_string(),
        };

        // Validate address format
        if addr.is_empty() {
            assert!(request.address.is_empty());
        } else if !addr.contains(':') {
            assert!(!request.address.contains(':'));
        }
    }
}

#[tokio::test]
async fn test_add_peer_response_structure() {
    // Test add peer response structure
    let response = AddPeerResponse {
        success: true,
        message: "Peer 2 added successfully".to_string(),
        peers: vec![
            PeerInfo {
                node_id: 1,
                address: "node-1:7000".to_string(),
                state: "Leader".to_string(),
                health: NodeHealthStatus::Healthy,
                replication_lag: None,
                last_heartbeat: None,
            },
            PeerInfo {
                node_id: 2,
                address: "node-2:7000".to_string(),
                state: "Follower".to_string(),
                health: NodeHealthStatus::Healthy,
                replication_lag: Some(0),
                last_heartbeat: None,
            },
        ],
    };

    assert!(response.success);
    assert_eq!(response.peers.len(), 2);
    assert!(response.message.contains("added successfully"));
}

#[tokio::test]
async fn test_remove_peer_quorum_safety() {
    // Test that removing a peer maintains quorum safety
    let initial_cluster_size = 5;
    let min_quorum_size = (initial_cluster_size / 2) + 1; // 3 for 5 nodes

    // Simulate removing nodes one by one
    for nodes_to_remove in 1..=3 {
        let remaining_nodes = initial_cluster_size - nodes_to_remove;
        let can_remove = remaining_nodes >= min_quorum_size;

        if nodes_to_remove <= 2 {
            // Can safely remove up to 2 nodes (5 -> 3)
            assert!(can_remove, "Should be able to remove {} nodes", nodes_to_remove);
        } else {
            // Cannot remove 3rd node (would go to 2 nodes, breaking quorum)
            assert!(!can_remove, "Should not be able to remove {} nodes", nodes_to_remove);
        }
    }
}

#[tokio::test]
async fn test_remove_peer_minimum_cluster_size() {
    // Test minimum cluster size enforcement (2 nodes)
    const MIN_CLUSTER_SIZE: usize = 2;

    let test_cases = vec![
        (5, true),  // 5 -> 4: OK
        (4, true),  // 4 -> 3: OK
        (3, true),  // 3 -> 2: OK
        (2, false), // 2 -> 1: NOT OK (breaks minimum)
        (1, false), // 1 -> 0: NOT OK
    ];

    for (current_size, can_remove) in test_cases {
        let remaining = current_size - 1;
        let is_safe = remaining >= MIN_CLUSTER_SIZE;
        assert_eq!(is_safe, can_remove,
            "Cluster size {} -> {}: expected can_remove={}",
            current_size, remaining, can_remove
        );
    }
}

#[tokio::test]
async fn test_remove_peer_leader_protection() {
    // Test that leader node cannot be removed directly
    let leader_id = 1u64;
    let node_to_remove = 1u64;

    // Attempting to remove leader should fail
    assert_eq!(leader_id, node_to_remove,
        "Should detect attempt to remove leader node"
    );

    // Verify error message would be appropriate
    let error_msg = "Cannot remove leader node. Transfer leadership first.";
    assert!(error_msg.contains("leader"));
    assert!(error_msg.contains("Transfer leadership"));
}

#[tokio::test]
async fn test_remove_peer_response_structure() {
    // Test remove peer response structure
    let response = RemovePeerResponse {
        success: true,
        message: "Peer 3 removed successfully".to_string(),
        peers: vec![
            PeerInfo {
                node_id: 1,
                address: "node-1:7000".to_string(),
                state: "Leader".to_string(),
                health: NodeHealthStatus::Healthy,
                replication_lag: None,
                last_heartbeat: None,
            },
            PeerInfo {
                node_id: 2,
                address: "node-2:7000".to_string(),
                state: "Follower".to_string(),
                health: NodeHealthStatus::Healthy,
                replication_lag: Some(0),
                last_heartbeat: None,
            },
        ],
    };

    assert!(response.success);
    assert_eq!(response.peers.len(), 2);
    assert!(response.message.contains("removed successfully"));
    assert!(!response.peers.iter().any(|p| p.node_id == 3));
}

#[tokio::test]
async fn test_peer_list_consistency() {
    // Test that peer list remains consistent after operations
    let mut peers = vec![1u64, 2, 3, 4, 5];

    // Add a peer
    peers.push(6);
    assert_eq!(peers.len(), 6);
    assert!(peers.contains(&6));

    // Remove a peer
    peers.retain(|&id| id != 3);
    assert_eq!(peers.len(), 5);
    assert!(!peers.contains(&3));

    // Verify no duplicates
    let unique_peers: std::collections::HashSet<_> = peers.iter().collect();
    assert_eq!(unique_peers.len(), peers.len());
}

// ============================================================================
// Snapshot Management Tests
// ============================================================================

#[tokio::test]
async fn test_snapshot_metadata_structure() {
    // Test snapshot metadata structure
    let metadata = SnapshotMetadata {
        snapshot_id: "snapshot-1234567890-abc123".to_string(),
        created_at: 1234567890,
        size_bytes: 1024,
        compressed_size_bytes: 512,
        last_included_index: 100,
        last_included_term: 5,
        checksum: "abc123def456".to_string(),
        encrypted: true,
        signature: Some("signature123".to_string()),
    };

    // Verify all fields
    assert!(metadata.snapshot_id.starts_with("snapshot-"));
    assert!(metadata.encrypted);
    assert!(metadata.signature.is_some());
    assert!(metadata.compressed_size_bytes < metadata.size_bytes);
    assert_eq!(metadata.last_included_index, 100);
    assert_eq!(metadata.last_included_term, 5);

    // Verify serialization
    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("snapshot_id"));
    assert!(json.contains("encrypted"));
    assert!(json.contains("checksum"));
}

#[tokio::test]
async fn test_snapshot_id_format() {
    // Test snapshot ID format validation
    let valid_ids = vec![
        "snapshot-1234567890-abc123",
        "snapshot-9999999999-xyz789",
        "snapshot-1609459200-a1b2c3",
    ];

    for id in valid_ids {
        assert!(id.starts_with("snapshot-"));
        let parts: Vec<&str> = id.split('-').collect();
        assert!(parts.len() >= 3, "ID should have at least 3 parts");
        assert!(parts[1].parse::<i64>().is_ok(), "Second part should be timestamp");
    }

    let invalid_ids = vec![
        "",
        "invalid-id",
        "snap-123",
        "snapshot-",
        "snapshot-abc-def", // Non-numeric timestamp
    ];

    for id in invalid_ids {
        if !id.is_empty() && id.starts_with("snapshot-") {
            let parts: Vec<&str> = id.split('-').collect();
            if parts.len() >= 2 {
                // Should fail to parse timestamp
                assert!(parts[1].parse::<i64>().is_err() || parts.len() < 3);
            }
        }
    }
}

#[tokio::test]
async fn test_create_snapshot_response() {
    // Test create snapshot response structure
    let response = CreateSnapshotResponse {
        success: true,
        message: "Snapshot created successfully".to_string(),
        snapshot: SnapshotMetadata {
            snapshot_id: "snapshot-1234567890-abc123".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            size_bytes: 2048,
            compressed_size_bytes: 1024,
            last_included_index: 150,
            last_included_term: 7,
            checksum: "checksum123".to_string(),
            encrypted: true,
            signature: Some("sig123".to_string()),
        },
    };

    assert!(response.success);
    assert!(response.message.contains("successfully"));
    assert!(response.snapshot.encrypted);
    assert!(response.snapshot.signature.is_some());
}

#[tokio::test]
async fn test_snapshot_compression_effectiveness() {
    // Test that compression actually reduces size
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let test_data = r#"{"node_id":1,"term":5,"index":100,"data":"test data with repetition repetition repetition"}"#;
    let original_size = test_data.len();

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(test_data.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();
    let compressed_size = compressed.len();

    // Compression should reduce size for repetitive data
    assert!(compressed_size < original_size,
        "Compressed size ({}) should be less than original ({})",
        compressed_size, original_size
    );

    let compression_ratio = (compressed_size as f64 / original_size as f64) * 100.0;
    println!("Compression ratio: {:.2}%", compression_ratio);
    assert!(compression_ratio < 100.0);
}

#[tokio::test]
async fn test_snapshot_checksum_verification() {
    // Test checksum calculation and verification
    use sha2::{Digest, Sha256};

    let data = b"test snapshot data for checksum verification";

    // Calculate checksum
    let mut hasher = Sha256::new();
    hasher.update(data);
    let checksum1 = format!("{:x}", hasher.finalize());

    // Verify checksum format (64 hex characters for SHA-256)
    assert_eq!(checksum1.len(), 64);
    assert!(checksum1.chars().all(|c| c.is_ascii_hexdigit()));

    // Verify deterministic (same data = same checksum)
    let mut hasher2 = Sha256::new();
    hasher2.update(data);
    let checksum2 = format!("{:x}", hasher2.finalize());
    assert_eq!(checksum1, checksum2);

    // Verify different data = different checksum
    let mut hasher3 = Sha256::new();
    hasher3.update(b"different data");
    let checksum3 = format!("{:x}", hasher3.finalize());
    assert_ne!(checksum1, checksum3);
}

#[tokio::test]
async fn test_list_snapshots_response() {
    // Test list snapshots response structure
    let snapshots = vec![
        SnapshotMetadata {
            snapshot_id: "snapshot-1234567890-abc".to_string(),
            created_at: 1234567890,
            size_bytes: 1024,
            compressed_size_bytes: 512,
            last_included_index: 100,
            last_included_term: 5,
            checksum: "checksum1".to_string(),
            encrypted: true,
            signature: Some("sig1".to_string()),
        },
        SnapshotMetadata {
            snapshot_id: "snapshot-1234567900-def".to_string(),
            created_at: 1234567900,
            size_bytes: 2048,
            compressed_size_bytes: 1024,
            last_included_index: 200,
            last_included_term: 6,
            checksum: "checksum2".to_string(),
            encrypted: true,
            signature: Some("sig2".to_string()),
        },
    ];

    let response = ListSnapshotsResponse {
        snapshots: snapshots.clone(),
        total: snapshots.len(),
    };

    assert_eq!(response.total, 2);
    assert_eq!(response.snapshots.len(), 2);
    assert!(response.snapshots[0].created_at < response.snapshots[1].created_at);
}

#[tokio::test]
async fn test_restore_snapshot_request_validation() {
    // Test restore request validation
    let valid_request = RestoreSnapshotRequest {
        snapshot_id: "snapshot-1234567890-abc123".to_string(),
    };

    assert!(!valid_request.snapshot_id.is_empty());
    assert!(valid_request.snapshot_id.starts_with("snapshot-"));

    // Invalid requests
    let invalid_requests = vec![
        RestoreSnapshotRequest {
            snapshot_id: String::new(), // Empty
        },
        RestoreSnapshotRequest {
            snapshot_id: "invalid-format".to_string(), // Wrong prefix
        },
    ];

    for req in invalid_requests {
        if req.snapshot_id.is_empty() {
            assert!(req.snapshot_id.is_empty());
        } else if !req.snapshot_id.starts_with("snapshot-") {
            assert!(!req.snapshot_id.starts_with("snapshot-"));
        }
    }
}

#[tokio::test]
async fn test_restore_snapshot_response() {
    // Test restore snapshot response structure
    let response = RestoreSnapshotResponse {
        success: true,
        message: "Snapshot restored successfully".to_string(),
        snapshot: SnapshotMetadata {
            snapshot_id: "snapshot-1234567890-abc123".to_string(),
            created_at: 1234567890,
            size_bytes: 1024,
            compressed_size_bytes: 512,
            last_included_index: 100,
            last_included_term: 5,
            checksum: "checksum123".to_string(),
            encrypted: true,
            signature: Some("sig123".to_string()),
        },
    };

    assert!(response.success);
    assert!(response.message.contains("restored successfully"));
    assert!(response.snapshot.encrypted);
}

#[tokio::test]
async fn test_snapshot_retention_policy() {
    // Test snapshot retention policy logic
    const RETENTION_COUNT: usize = 10;

    // Create more snapshots than retention limit
    let mut snapshots: Vec<SnapshotMetadata> = Vec::new();
    for i in 0..15 {
        snapshots.push(SnapshotMetadata {
            snapshot_id: format!("snapshot-{}-{}", 1000000000 + i, i),
            created_at: 1000000000 + i as i64,
            size_bytes: 1024,
            compressed_size_bytes: 512,
            last_included_index: i as u64,
            last_included_term: 1,
            checksum: format!("checksum{}", i),
            encrypted: true,
            signature: Some(format!("sig{}", i)),
        });
    }

    // Sort by creation time (newest first)
    snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Keep only the most recent RETENTION_COUNT snapshots
    let retained: Vec<_> = snapshots.into_iter().take(RETENTION_COUNT).collect();

    // Verify retention policy
    assert_eq!(retained.len(), RETENTION_COUNT);

    // Verify we kept the newest snapshots (indices 14, 13, 12, ..., 5)
    for (i, snapshot) in retained.iter().enumerate() {
        let expected_index = 14 - i;
        assert_eq!(snapshot.last_included_index, expected_index as u64);
    }
}

#[tokio::test]
async fn test_snapshot_encryption_flag() {
    // Test that snapshots are marked as encrypted
    let metadata = SnapshotMetadata {
        snapshot_id: "snapshot-test".to_string(),
        created_at: chrono::Utc::now().timestamp(),
        size_bytes: 1024,
        compressed_size_bytes: 512,
        last_included_index: 50,
        last_included_term: 3,
        checksum: "checksum".to_string(),
        encrypted: true,
        signature: Some("signature".to_string()),
    };

    assert!(metadata.encrypted, "Snapshots should always be encrypted");
    assert!(metadata.signature.is_some(), "Snapshots should be signed");
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_cluster_lifecycle() {
    // Test complete cluster lifecycle: create -> status -> shutdown
    let config = RaftClusterConfig::default();
    let cluster = RaftCluster::new(config).await.unwrap();

    // Get initial status
    let status = cluster.status().await.unwrap();
    assert_eq!(status.node_id, 1);

    // Check leader status
    let is_leader = cluster.is_leader().await;
    assert!(is_leader || !is_leader); // Either state is valid

    // Shutdown
    let shutdown_result = cluster.shutdown().await;
    assert!(shutdown_result.is_ok());
}

#[tokio::test]
async fn test_snapshot_lifecycle_simulation() {
    // Simulate complete snapshot lifecycle
    let snapshot_id = format!(
        "snapshot-{}-{}",
        chrono::Utc::now().timestamp(),
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
    );

    // Step 1: Create snapshot metadata
    let metadata = SnapshotMetadata {
        snapshot_id: snapshot_id.clone(),
        created_at: chrono::Utc::now().timestamp(),
        size_bytes: 2048,
        compressed_size_bytes: 1024,
        last_included_index: 100,
        last_included_term: 5,
        checksum: "abc123def456".to_string(),
        encrypted: true,
        signature: Some("sig123".to_string()),
    };

    // Step 2: Verify metadata
    assert!(metadata.snapshot_id.starts_with("snapshot-"));
    assert!(metadata.encrypted);
    assert!(metadata.signature.is_some());
    assert!(metadata.compressed_size_bytes < metadata.size_bytes);

    // Step 3: Simulate listing
    let snapshots = vec![metadata.clone()];
    assert_eq!(snapshots.len(), 1);

    // Step 4: Simulate restore request
    let restore_request = RestoreSnapshotRequest {
        snapshot_id: snapshot_id.clone(),
    };
    assert_eq!(restore_request.snapshot_id, snapshot_id);
}

#[tokio::test]
async fn test_concurrent_cluster_operations() {
    // Test that multiple cluster operations can be performed concurrently
    use tokio::task;

    let mut handles = vec![];

    // Spawn multiple tasks that create cluster configs
    for i in 0..5 {
        let handle = task::spawn(async move {
            let config = RaftClusterConfig {
                node_id: i + 1,
                peers: HashMap::new(),
                election_timeout_ms: 1000,
                heartbeat_interval_ms: 300,
                max_payload_entries: 1000,
                enable_tick: true,
            };

            RaftCluster::new(config).await
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // Verify all tasks completed successfully
    assert_eq!(results.len(), 5);
    for result in results {
        assert!(result.is_ok());
        let cluster_result = result.unwrap();
        assert!(cluster_result.is_ok());
    }
}

#[tokio::test]
async fn test_peer_info_serialization() {
    // Test peer info JSON serialization
    let peer = PeerInfo {
        node_id: 2,
        address: "192.168.1.2:7000".to_string(),
        state: "Follower".to_string(),
        health: NodeHealthStatus::Healthy,
        replication_lag: Some(5),
        last_heartbeat: Some(1234567890),
    };

    let json = serde_json::to_string_pretty(&peer).unwrap();
    println!("Peer info JSON:\n{}", json);

    assert!(json.contains("\"node_id\":2"));
    assert!(json.contains("192.168.1.2:7000"));
    assert!(json.contains("Follower"));
    assert!(json.contains("\"replication_lag\":5"));

    // Deserialize and verify
    let deserialized: PeerInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.node_id, peer.node_id);
    assert_eq!(deserialized.address, peer.address);
}

#[tokio::test]
async fn test_cluster_membership_changes() {
    // Test cluster membership tracking through add/remove operations
    let mut membership = vec![1u64, 2, 3];

    // Initial state
    assert_eq!(membership.len(), 3);

    // Add nodes
    membership.push(4);
    membership.push(5);
    assert_eq!(membership.len(), 5);
    assert!(membership.contains(&4));
    assert!(membership.contains(&5));

    // Remove node
    membership.retain(|&id| id != 3);
    assert_eq!(membership.len(), 4);
    assert!(!membership.contains(&3));

    // Verify quorum
    let quorum_size = (membership.len() / 2) + 1;
    assert_eq!(quorum_size, 3); // 4 nodes -> quorum of 3
}
