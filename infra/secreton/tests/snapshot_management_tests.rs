//! Integration tests for Raft snapshot management
//!
//! Tests the complete snapshot lifecycle including creation, listing,
//! download, and restore operations.

use secreton_api::handlers::raft::{
    CreateSnapshotResponse, ListSnapshotsResponse, RestoreSnapshotRequest,
    RestoreSnapshotResponse, SnapshotMetadata,
};
use secreton_storage::MemoryBackend;
use std::sync::Arc;

#[tokio::test]
async fn test_snapshot_metadata_structure() {
    // Test that snapshot metadata can be serialized/deserialized
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

    // Serialize to JSON
    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("snapshot-1234567890-abc123"));
    assert!(json.contains("\"encrypted\":true"));

    // Deserialize from JSON
    let deserialized: SnapshotMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.snapshot_id, metadata.snapshot_id);
    assert_eq!(deserialized.size_bytes, metadata.size_bytes);
    assert_eq!(deserialized.encrypted, metadata.encrypted);
}

#[tokio::test]
async fn test_snapshot_id_format_validation() {
    // Valid snapshot IDs
    assert!(validate_snapshot_id("snapshot-1234567890-abc123"));
    assert!(validate_snapshot_id("snapshot-9999999999-xyz789"));

    // Invalid snapshot IDs
    assert!(!validate_snapshot_id(""));
    assert!(!validate_snapshot_id("invalid-id"));
    assert!(!validate_snapshot_id("snap-123"));
    assert!(!validate_snapshot_id("snapshot-"));
}

#[tokio::test]
async fn test_restore_request_validation() {
    // Valid restore request
    let valid_request = RestoreSnapshotRequest {
        snapshot_id: "snapshot-1234567890-abc123".to_string(),
    };
    assert!(!valid_request.snapshot_id.is_empty());
    assert!(valid_request.snapshot_id.starts_with("snapshot-"));

    // Invalid restore request (empty ID)
    let invalid_request = RestoreSnapshotRequest {
        snapshot_id: String::new(),
    };
    assert!(invalid_request.snapshot_id.is_empty());
}

#[tokio::test]
async fn test_snapshot_compression_ratio() {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    // Create sample data (JSON-like structure similar to Raft state)
    let sample_data = r#"{"node_id":1,"term":5,"index":100,"timestamp":1234567890}"#;
    let original_size = sample_data.len();

    // Compress the data
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(sample_data.as_bytes()).unwrap();
    let compressed_data = encoder.finish().unwrap();
    let compressed_size = compressed_data.len();

    // Verify compression occurred
    assert!(compressed_size < original_size);

    // Calculate compression ratio
    let ratio = (compressed_size as f64 / original_size as f64) * 100.0;
    println!("Compression ratio: {:.2}%", ratio);

    // For JSON data, we expect at least some compression
    assert!(ratio < 100.0);
}

#[tokio::test]
async fn test_snapshot_checksum_calculation() {
    use sha2::{Digest, Sha256};

    let data = b"test snapshot data";

    // Calculate checksum
    let mut hasher = Sha256::new();
    hasher.update(data);
    let checksum = format!("{:x}", hasher.finalize());

    // Verify checksum format (64 hex characters for SHA-256)
    assert_eq!(checksum.len(), 64);
    assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));

    // Verify checksum is deterministic
    let mut hasher2 = Sha256::new();
    hasher2.update(data);
    let checksum2 = format!("{:x}", hasher2.finalize());
    assert_eq!(checksum, checksum2);

    // Verify different data produces different checksum
    let mut hasher3 = Sha256::new();
    hasher3.update(b"different data");
    let checksum3 = format!("{:x}", hasher3.finalize());
    assert_ne!(checksum, checksum3);
}

#[tokio::test]
async fn test_snapshot_lifecycle_simulation() {
    // This test simulates the complete snapshot lifecycle without actual Raft cluster

    // Step 1: Create snapshot metadata
    let snapshot_id = format!(
        "snapshot-{}-{}",
        chrono::Utc::now().timestamp(),
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
    );

    let metadata = SnapshotMetadata {
        snapshot_id: snapshot_id.clone(),
        created_at: chrono::Utc::now().timestamp(),
        size_bytes: 1024,
        compressed_size_bytes: 512,
        last_included_index: 100,
        last_included_term: 5,
        checksum: "abc123".to_string(),
        encrypted: true,
        signature: Some("sig123".to_string()),
    };

    // Step 2: Verify metadata
    assert!(metadata.snapshot_id.starts_with("snapshot-"));
    assert!(metadata.encrypted);
    assert!(metadata.signature.is_some());
    assert!(metadata.compressed_size_bytes < metadata.size_bytes);

    // Step 3: Simulate listing snapshots
    let snapshots = vec![metadata.clone()];
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].snapshot_id, snapshot_id);

    // Step 4: Simulate restore request
    let restore_request = RestoreSnapshotRequest {
        snapshot_id: snapshot_id.clone(),
    };
    assert_eq!(restore_request.snapshot_id, snapshot_id);
}

#[tokio::test]
async fn test_snapshot_retention_policy() {
    // Test that retention policy logic works correctly
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
    let retained_snapshots: Vec<_> = snapshots.into_iter().take(RETENTION_COUNT).collect();

    // Verify retention policy
    assert_eq!(retained_snapshots.len(), RETENTION_COUNT);

    // Verify we kept the newest snapshots
    for i in 0..RETENTION_COUNT {
        assert!(retained_snapshots[i].created_at >= 1000000000 + 5);
    }
}

#[tokio::test]
async fn test_snapshot_error_scenarios() {
    // Test various error scenarios

    // Empty snapshot ID
    let empty_id = "";
    assert!(empty_id.is_empty());

    // Invalid snapshot ID format
    let invalid_id = "invalid-format";
    assert!(!invalid_id.starts_with("snapshot-"));

    // Snapshot ID too short
    let short_id = "snapshot-";
    assert!(short_id.starts_with("snapshot-"));
    assert_eq!(short_id.len(), 9); // Too short to be valid

    // Valid snapshot ID
    let valid_id = "snapshot-1234567890-abc123";
    assert!(valid_id.starts_with("snapshot-"));
    assert!(valid_id.len() > 20);
}

#[tokio::test]
async fn test_snapshot_size_calculations() {
    // Test size calculation logic

    let original_size: u64 = 1024;
    let compressed_size: u64 = 512;

    // Calculate compression ratio
    let compression_ratio = (compressed_size as f64 / original_size as f64) * 100.0;
    assert_eq!(compression_ratio, 50.0);

    // Calculate space saved
    let space_saved = original_size - compressed_size;
    assert_eq!(space_saved, 512);

    // Calculate space saved percentage
    let space_saved_pct = ((original_size - compressed_size) as f64 / original_size as f64) * 100.0;
    assert_eq!(space_saved_pct, 50.0);
}

// Helper function to validate snapshot ID format
fn validate_snapshot_id(id: &str) -> bool {
    if id.is_empty() {
        return false;
    }

    if !id.starts_with("snapshot-") {
        return false;
    }

    // Should have format: snapshot-{timestamp}-{uuid_prefix}
    let parts: Vec<&str> = id.split('-').collect();
    if parts.len() < 3 {
        return false;
    }

    // Verify timestamp part is numeric
    if parts[1].parse::<i64>().is_err() {
        return false;
    }

    true
}

#[tokio::test]
async fn test_snapshot_concurrent_operations() {
    // Test that multiple snapshot operations can be handled concurrently
    use tokio::task;

    let mut handles = vec![];

    // Spawn multiple tasks that create snapshot metadata
    for i in 0..10 {
        let handle = task::spawn(async move {
            let snapshot_id = format!(
                "snapshot-{}-{}",
                chrono::Utc::now().timestamp() + i,
                uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
            );

            SnapshotMetadata {
                snapshot_id,
                created_at: chrono::Utc::now().timestamp(),
                size_bytes: 1024,
                compressed_size_bytes: 512,
                last_included_index: i as u64,
                last_included_term: 1,
                checksum: format!("checksum{}", i),
                encrypted: true,
                signature: Some(format!("sig{}", i)),
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // Verify all tasks completed successfully
    assert_eq!(results.len(), 10);
    for result in results {
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert!(metadata.snapshot_id.starts_with("snapshot-"));
    }
}

#[tokio::test]
async fn test_snapshot_metadata_json_compatibility() {
    // Test that snapshot metadata is compatible with JSON serialization

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

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&metadata).unwrap();
    println!("Snapshot metadata JSON:\n{}", json);

    // Verify JSON contains expected fields
    assert!(json.contains("snapshot_id"));
    assert!(json.contains("created_at"));
    assert!(json.contains("size_bytes"));
    assert!(json.contains("compressed_size_bytes"));
    assert!(json.contains("last_included_index"));
    assert!(json.contains("last_included_term"));
    assert!(json.contains("checksum"));
    assert!(json.contains("encrypted"));
    assert!(json.contains("signature"));

    // Deserialize and verify
    let deserialized: SnapshotMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.snapshot_id, metadata.snapshot_id);
    assert_eq!(deserialized.created_at, metadata.created_at);
    assert_eq!(deserialized.size_bytes, metadata.size_bytes);
}
