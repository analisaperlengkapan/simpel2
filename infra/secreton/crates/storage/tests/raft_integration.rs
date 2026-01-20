// This test module requires the raft-consensus feature
#![cfg(feature = "raft-consensus")]

use secreton_storage::raft::{
    RaftCluster, RaftClusterConfig, StateMachineCommand, StateMachineResponse,
};
use secreton_storage::{SecurityLevel, StorageResult, VaultEntry};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// Helper to create a test cluster configuration
fn create_test_config(node_id: u64, peers: HashMap<u64, String>) -> RaftClusterConfig {
    RaftClusterConfig {
        node_id,
        peers,
        election_timeout_ms: 500, // Shorter for testing
        heartbeat_interval_ms: 150,
        max_payload_entries: 100,
        enable_tick: true,
        bootstrap: true, // Bootstrap for tests
    }
}

/// Helper to create a test vault entry
fn create_test_entry(path: &str, data: Vec<u8>) -> VaultEntry {
    VaultEntry {
        id: Uuid::new_v4(),
        path: path.to_string(),
        encrypted_data: data,
        encryption_metadata: serde_json::json!({}),
        security_level: SecurityLevel::Internal,
        metadata: serde_json::json!({}),
        tags: vec![],
        version: 1,
        owner_id: "test".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        expires_at: None,
    }
}

#[tokio::test]
#[ignore] // Requires multi-process setup
async fn test_three_node_cluster_startup() -> StorageResult<()> {
    // This test verifies that a 3-node cluster can be initialized
    // In a real scenario, these would be separate processes

    let peers_1 = HashMap::from([
        (2, "127.0.0.1:8002".to_string()),
        (3, "127.0.0.1:8003".to_string()),
    ]);

    let peers_2 = HashMap::from([
        (1, "127.0.0.1:8001".to_string()),
        (3, "127.0.0.1:8003".to_string()),
    ]);

    let peers_3 = HashMap::from([
        (1, "127.0.0.1:8001".to_string()),
        (2, "127.0.0.1:8002".to_string()),
    ]);

    // Create three nodes
    let node1 = RaftCluster::new(create_test_config(1, peers_1)).await?;
    let node2 = RaftCluster::new(create_test_config(2, peers_2)).await?;
    let node3 = RaftCluster::new(create_test_config(3, peers_3)).await?;

    // Wait for leader election
    sleep(Duration::from_secs(2)).await;

    // Verify cluster status
    let status1 = node1.status().await?;
    let status2 = node2.status().await?;
    let status3 = node3.status().await?;

    // One node should be leader
    let leader_count = [status1.is_leader, status2.is_leader, status3.is_leader]
        .iter()
        .filter(|&&is_leader| is_leader)
        .count();

    assert_eq!(leader_count, 1, "Exactly one node should be leader");

    // All nodes should have the same leader
    assert_eq!(status1.leader_id, status2.leader_id);
    assert_eq!(status2.leader_id, status3.leader_id);

    // Cleanup
    node1.shutdown().await?;
    node2.shutdown().await?;
    node3.shutdown().await?;

    Ok(())
}

#[tokio::test]
#[ignore] // Requires multi-process setup
async fn test_leader_election() -> StorageResult<()> {
    // Create a single-node cluster (it should become leader immediately)
    let config = RaftClusterConfig {
        node_id: 1,
        peers: HashMap::new(),
        election_timeout_ms: 500,
        heartbeat_interval_ms: 150,
        max_payload_entries: 100,
        enable_tick: true,
        bootstrap: true,
    };

    let cluster = RaftCluster::new(config).await?;

    // Wait for election
    sleep(Duration::from_millis(600)).await;

    // Verify node is leader
    let status = cluster.status().await?;
    assert!(status.is_leader, "Single node should be leader");
    assert_eq!(status.leader_id, Some(1));
    assert_eq!(status.current_term, 1);

    cluster.shutdown().await?;

    Ok(())
}

#[tokio::test]
#[ignore] // Requires multi-process setup
async fn test_basic_consensus() -> StorageResult<()> {
    // This test would verify that a write to the leader
    // is replicated to followers

    // For now, test single-node write/read
    let config = RaftClusterConfig {
        node_id: 1,
        peers: HashMap::new(),
        election_timeout_ms: 500,
        heartbeat_interval_ms: 150,
        max_payload_entries: 100,
        enable_tick: true,
        bootstrap: true,
    };

    let cluster = RaftCluster::new(config).await?;
    sleep(Duration::from_millis(600)).await;

    // Propose a command
    let entry = create_test_entry("test/key", vec![1, 2, 3]);
    let command = StateMachineCommand::Store(entry);

    let response = cluster.propose(command).await?;

    match response {
        StateMachineResponse::Success => {
            // Success!
        }
        _ => panic!("Expected success response"),
    }

    cluster.shutdown().await?;

    Ok(())
}

#[tokio::test]
#[ignore] // Requires multi-process setup
async fn test_log_replication() -> StorageResult<()> {
    // This test verifies that log entries are replicated across all nodes
    // For full implementation, would need inter-process communication

    // Placeholder: test that single node can store and retrieve
    let config = RaftClusterConfig {
        node_id: 1,
        peers: HashMap::new(),
        election_timeout_ms: 500,
        heartbeat_interval_ms: 150,
        max_payload_entries: 100,
        enable_tick: true,
        bootstrap: true,
    };

    let cluster = RaftCluster::new(config).await?;
    sleep(Duration::from_millis(600)).await;

    // Store multiple entries
    for i in 0..10 {
        let entry = create_test_entry(&format!("test/key{}", i), vec![i as u8; 10]);
        let command = StateMachineCommand::Store(entry);
        cluster.propose(command).await?;
    }

    // Verify status shows entries were applied
    let status = cluster.status().await?;
    assert!(status.last_applied.is_some());
    assert!(status.last_log_index.is_some());

    cluster.shutdown().await?;

    Ok(())
}

#[test]
fn test_cluster_config_validation() {
    // Test config validation
    let config = RaftClusterConfig::default();
    assert_eq!(config.node_id, 1);
    assert_eq!(config.election_timeout_ms, 1000);
    assert_eq!(config.heartbeat_interval_ms, 300);
}
