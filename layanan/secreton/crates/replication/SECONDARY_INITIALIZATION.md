# Secondary Node Initialization

This document describes the secondary node initialization process for Secreton replication.

## Overview

Secondary node initialization is the process of bootstrapping a new secondary node from scratch. This involves:

1. **Snapshot Transfer**: The primary creates a snapshot of the current state and transfers it to the secondary
2. **Snapshot Application**: The secondary applies the snapshot to its local storage
3. **WAL Streaming**: After the snapshot is applied, the secondary starts streaming write-ahead log (WAL) entries from the primary

## Architecture

```
Primary Node                          Secondary Node
┌──────────────┐                     ┌──────────────┐
│              │                     │              │
│  1. Create   │                     │  2. Request  │
│  Snapshot    │◀────────────────────│  Snapshot    │
│              │                     │              │
│  ┌────────┐  │                     │              │
│  │ State  │  │                     │              │
│  │ Data   │  │                     │              │
│  └────────┘  │                     │              │
│      │       │                     │              │
│      ▼       │                     │              │
│  Serialize   │                     │              │
│  Compress    │                     │              │
│  Checksum    │                     │              │
│      │       │                     │              │
│      ▼       │                     │              │
│  3. Transfer │─────────────────────▶  4. Receive  │
│  Snapshot    │      (gRPC)         │  Snapshot    │
│              │                     │              │
│              │                     │  5. Verify   │
│              │                     │  Decompress  │
│              │                     │  Apply       │
│              │                     │      │       │
│              │                     │      ▼       │
│              │                     │  ┌────────┐  │
│              │                     │  │ State  │  │
│              │                     │  │ Data   │  │
│              │                     │  └────────┘  │
│              │                     │              │
│  6. Stream   │─────────────────────▶  7. Apply   │
│  WAL Entries │      (gRPC)         │  WAL        │
│              │                     │              │
└──────────────┘                     └──────────────┘
```

## Components

### SecondaryInitializer

The `SecondaryInitializer` manages the initialization process on the secondary node.

```rust
use secreton_replication::{SecondaryInitializer, ReplicationStorage};

let initializer = SecondaryInitializer::new(storage);

// Initialize from primary
let node = initializer
    .initialize_from_primary("https://primary:50051")
    .await?;

// Check initialization state
let state = initializer.state().await;
println!("Initialization state: {:?}", state);
```

### SnapshotCreator

The `SnapshotCreator` creates snapshots on the primary node.

```rust
use secreton_replication::{SnapshotCreator, ReplicationStorage};

let creator = SnapshotCreator::new(storage);

// Create a snapshot
let snapshot = creator.create_snapshot().await?;

println!("Snapshot created: {} ({} bytes, {} operations)",
    snapshot.id,
    snapshot.metadata.size_bytes,
    snapshot.metadata.operation_count
);
```

### ReplicationManager Integration

The `ReplicationManager` provides high-level methods for secondary initialization.

```rust
use secreton_replication::{ReplicationManager, ReplicationConfig, ReplicationMode};

// On secondary node
let config = ReplicationConfig {
    mode: ReplicationMode::Performance,
    primary_endpoint: Some("https://primary:50051".to_string()),
    ..Default::default()
};

let manager = ReplicationManager::new(config, storage).await?;

// Initialize secondary
manager.initialize_secondary().await?;

// Check if initialized
if manager.is_secondary_initialized().await {
    println!("Secondary is fully initialized and streaming");
}

// On primary node
let config = ReplicationConfig {
    mode: ReplicationMode::Performance,
    primary_endpoint: None,
    secondary_endpoints: vec!["https://secondary:50051".to_string()],
    ..Default::default()
};

let manager = ReplicationManager::new(config, storage).await?;

// Create snapshot for new secondary
let snapshot = manager.create_snapshot().await?;
```

## Initialization States

The secondary node goes through several states during initialization:

1. **NotStarted**: Initial state before initialization begins
2. **RequestingSnapshot**: Requesting snapshot from primary
3. **TransferringSnapshot**: Receiving snapshot data from primary
4. **ApplyingSnapshot**: Applying snapshot to local storage
5. **StartingWALStream**: Setting up WAL streaming connection
6. **Streaming**: Fully initialized and streaming WAL entries
7. **Failed**: Initialization failed (with error reason)

## Snapshot Format

### ReplicationSnapshot

```rust
pub struct ReplicationSnapshot {
    /// Unique identifier
    pub id: String,

    /// Creation timestamp
    pub timestamp: DateTime<Utc>,

    /// Sequence number at snapshot time
    pub sequence: u64,

    /// Serialized and compressed data
    pub data: Vec<u8>,

    /// Snapshot metadata
    pub metadata: SnapshotMetadata,
}
```

### SnapshotMetadata

```rust
pub struct SnapshotMetadata {
    /// Size in bytes (after compression)
    pub size_bytes: usize,

    /// Number of operations included
    pub operation_count: usize,

    /// Compression algorithm ("zstd" or "gzip")
    pub compression: Option<String>,

    /// SHA-256 checksum for integrity verification
    pub checksum: String,
}
```

## Data Flow

### 1. Snapshot Creation (Primary)

```rust
// Get all operations from storage
let operations = storage.get_operations_since(0).await?;

// Serialize to JSON
let data = serde_json::to_vec(&operations)?;

// Compress with zstd
let compressed = zstd::encode_all(&data[..], 3)?;

// Calculate checksum
let checksum = sha256(&compressed);

// Create snapshot
let snapshot = ReplicationSnapshot {
    id: Uuid::new_v4().to_string(),
    timestamp: Utc::now(),
    sequence: current_sequence,
    data: compressed,
    metadata: SnapshotMetadata {
        size_bytes: compressed.len(),
        operation_count: operations.len(),
        compression: Some("zstd".to_string()),
        checksum,
    },
};
```

### 2. Snapshot Transfer (gRPC)

The snapshot is transferred from primary to secondary using gRPC streaming:

```protobuf
service ReplicationService {
    rpc GetSnapshot(SnapshotRequest) returns (stream SnapshotChunk);
}

message SnapshotRequest {
    string node_id = 1;
}

message SnapshotChunk {
    string snapshot_id = 1;
    uint64 sequence = 2;
    bytes data = 3;
    uint64 offset = 4;
    uint64 total_size = 5;
    SnapshotMetadata metadata = 6;
}
```

### 3. Snapshot Application (Secondary)

```rust
// Verify checksum
verify_checksum(&snapshot)?;

// Decompress
let data = zstd::decode_all(&snapshot.data[..])?;

// Deserialize
let operations: Vec<ReplicationOperation> = serde_json::from_slice(&data)?;

// Apply operations
for operation in operations {
    storage.apply_operation(&operation).await?;
}

// Store snapshot metadata
storage.store_cluster_metadata(
    "last_snapshot_sequence",
    &snapshot.sequence.to_le_bytes()
).await?;
```

### 4. WAL Streaming Setup

After the snapshot is applied, the secondary starts streaming WAL entries:

```rust
// Start streaming from snapshot sequence
let start_sequence = snapshot.sequence;

// Establish gRPC streaming connection
let mut stream = client.stream_wal(StreamRequest {
    start_sequence,
    node_id: secondary_id,
}).await?;

// Process WAL entries
while let Some(entry) = stream.next().await {
    let operation = entry?;
    storage.apply_operation(&operation).await?;
}
```

## Error Handling

### Snapshot Checksum Mismatch

If the snapshot checksum doesn't match, the initialization fails:

```rust
if computed_checksum != snapshot.metadata.checksum {
    return Err(ReplicationError::snapshot(format!(
        "Snapshot checksum mismatch: expected {}, got {}",
        snapshot.metadata.checksum,
        computed_checksum
    )));
}
```

### Decompression Failure

If decompression fails, the initialization fails:

```rust
let data = zstd::decode_all(&snapshot.data[..])
    .map_err(|e| ReplicationError::snapshot(
        format!("Zstd decompression failed: {}", e)
    ))?;
```

### Deserialization Failure

If deserialization fails, the initialization fails:

```rust
let operations: Vec<ReplicationOperation> = serde_json::from_slice(&data)
    .map_err(|e| ReplicationError::snapshot(
        format!("Failed to deserialize snapshot: {}", e)
    ))?;
```

## Performance Considerations

### Compression

Snapshots are compressed using zstd with level 3 (balanced speed/compression):

- **Level 3**: Good balance between compression ratio and speed
- **Typical compression ratio**: 60-80% reduction in size
- **Compression speed**: ~200-400 MB/s
- **Decompression speed**: ~800-1200 MB/s

### Serialization

Operations are serialized using JSON for compatibility:

- **Format**: JSON (human-readable, debuggable)
- **Alternative**: Could use bincode for better performance (binary format)
- **Trade-off**: JSON is slower but more compatible and debuggable

### Snapshot Size

For a typical Secreton deployment:

- **1,000 secrets**: ~100 KB uncompressed, ~20 KB compressed
- **10,000 secrets**: ~1 MB uncompressed, ~200 KB compressed
- **100,000 secrets**: ~10 MB uncompressed, ~2 MB compressed

### Transfer Time

Over a 1 Gbps network:

- **1 MB snapshot**: ~8 ms transfer time
- **10 MB snapshot**: ~80 ms transfer time
- **100 MB snapshot**: ~800 ms transfer time

## Best Practices

### 1. Snapshot Frequency

- Create snapshots only when adding new secondaries
- Don't create snapshots on a schedule (use WAL streaming for continuous replication)
- Snapshots are point-in-time and become stale quickly

### 2. Snapshot Retention

- Keep the last snapshot for disaster recovery
- Delete old snapshots after successful initialization
- Store snapshots in S3-compatible storage for durability

### 3. Network Considerations

- Use mTLS for snapshot transfer
- Consider network bandwidth when transferring large snapshots
- Use compression to reduce transfer time

### 4. Monitoring

Monitor the following metrics:

- **Snapshot creation time**: Should be < 1 second for typical deployments
- **Snapshot transfer time**: Should be < 10 seconds for typical deployments
- **Snapshot application time**: Should be < 5 seconds for typical deployments
- **Total initialization time**: Should be < 30 seconds for typical deployments

### 5. Error Recovery

If initialization fails:

1. Check network connectivity to primary
2. Verify primary is healthy and unsealed
3. Check secondary has sufficient disk space
4. Review error logs for specific failure reason
5. Retry initialization after fixing the issue

## Testing

### Unit Tests

```bash
# Run all replication tests
cargo test --package secreton-replication

# Run only secondary initialization tests
cargo test --package secreton-replication secondary::tests
```

### Integration Tests

```bash
# Run integration tests with real primary/secondary
cargo test --package secreton-replication --test integration
```

### Property-Based Tests

Property-based tests verify correctness properties:

- **Snapshot round-trip**: Snapshot creation and application preserves all operations
- **Checksum verification**: Invalid checksums are detected
- **Compression round-trip**: Compression and decompression preserves data
- **Serialization round-trip**: Serialization and deserialization preserves operations

## Future Enhancements

### 1. Incremental Snapshots

Instead of transferring all operations, transfer only operations since the last snapshot:

```rust
// Create incremental snapshot
let snapshot = creator.create_incremental_snapshot(last_sequence).await?;
```

### 2. Parallel Snapshot Transfer

Transfer snapshot in parallel chunks for faster initialization:

```rust
// Transfer snapshot in 10 MB chunks
let chunks = snapshot.split_into_chunks(10 * 1024 * 1024);
for chunk in chunks {
    transfer_chunk(chunk).await?;
}
```

### 3. Snapshot Verification

Verify snapshot integrity before application:

```rust
// Verify snapshot before applying
creator.verify_snapshot(&snapshot).await?;
initializer.apply_snapshot(&snapshot).await?;
```

### 4. Snapshot Encryption

Encrypt snapshots for security:

```rust
// Encrypt snapshot with AES-256-GCM
let encrypted_snapshot = encrypt_snapshot(&snapshot, &encryption_key)?;
```

## References

- [Replication Design Document](../../.kiro/specs/secreton-vault-parity/design.md)
- [Replication Requirements](../../.kiro/specs/secreton-vault-parity/requirements.md)
- [WAL Streaming Documentation](./WAL_STREAMING.md)
- [Replication Manager API](./src/manager.rs)
- [Secondary Initializer API](./src/secondary.rs)
