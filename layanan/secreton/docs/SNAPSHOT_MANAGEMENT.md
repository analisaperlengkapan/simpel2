# Snapshot Management Implementation

## Overview

This document describes the implementation of Raft snapshot management for Secreton, providing backup and restore capabilities for the distributed cluster.

## Features

### 1. Snapshot Creation
- **Endpoint**: `POST /v1/sys/raft/snapshot`
- **gRPC**: `CreateSnapshot`
- **Security**: Admin-only access
- **Process**:
  1. Trigger Raft snapshot creation
  2. Compress snapshot data using gzip
  3. Encrypt compressed data using CryptoEngine
  4. Calculate SHA-256 checksum
  5. Sign with Ed25519
  6. Store in PostgreSQL database
  7. Automatic cleanup of old snapshots (retention: 10 snapshots)

### 2. Snapshot Download
- **Endpoint**: `GET /v1/sys/raft/snapshot?snapshot_id={id}`
- **Security**: Admin-only access
- **Process**:
  1. Retrieve encrypted snapshot from database
  2. Verify checksum integrity
  3. Verify Ed25519 signature
  4. Return encrypted snapshot as binary download
  5. Audit log the download operation

### 3. Snapshot Listing
- **Endpoint**: `GET /v1/sys/raft/snapshots`
- **gRPC**: `ListSnapshots`
- **Security**: Admin-only access
- **Returns**: List of all available snapshots with metadata

### 4. Snapshot Restore
- **Endpoint**: `POST /v1/sys/raft/restore`
- **gRPC**: `RestoreSnapshot`
- **Security**: Admin-only access, leader-only operation
- **Process**:
  1. Validate snapshot exists
  2. Verify checksum and signature
  3. Decrypt snapshot data
  4. Decompress snapshot data
  5. Validate snapshot format
  6. Create backup of current state
  7. Restore state from snapshot
  8. Audit log the restore operation

## Database Schema

```sql
CREATE TABLE raft_snapshots (
    snapshot_id VARCHAR(255) PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    size_bytes BIGINT NOT NULL,
    compressed_size_bytes BIGINT NOT NULL,
    last_included_index BIGINT NOT NULL,
    last_included_term BIGINT NOT NULL,
    checksum VARCHAR(64) NOT NULL,
    encrypted_data BYTEA NOT NULL,
    signature VARCHAR(128),
    metadata JSONB DEFAULT '{}'::jsonb
);
```

## Security Features

### Encryption
- All snapshots are encrypted using the CryptoEngine
- Encryption happens after compression for efficiency
- Encrypted data is stored in the database

### Integrity Verification
- SHA-256 checksum calculated on encrypted data
- Checksum verified before any restore operation
- Prevents corruption and tampering

### Authenticity Verification
- Ed25519 signature on encrypted data
- Signature verified before restore
- Ensures snapshot authenticity

### Access Control
- All operations require admin-level access
- Restore operations only allowed on leader node
- Prevents unauthorized backup/restore

## Automatic Cleanup

The system automatically cleans up old snapshots to prevent unbounded growth:

- **Retention Policy**: Keep 10 most recent snapshots
- **Trigger**: After each snapshot creation
- **Process**: Asynchronous cleanup in background
- **Metrics**: Tracks number of snapshots cleaned

## Metrics

The following Prometheus metrics are exposed:

- `secreton_raft_snapshots_created`: Counter of snapshots created
- `secreton_raft_snapshot_size_bytes`: Gauge of original snapshot size
- `secreton_raft_snapshot_compressed_size_bytes`: Gauge of compressed size
- `secreton_raft_restores_performed`: Counter of successful restores
- `secreton_raft_restores_attempted`: Counter of restore attempts
- `secreton_raft_restore_failures`: Counter of restore failures (by reason)
- `secreton_raft_snapshots_cleaned`: Counter of snapshots cleaned up

## Audit Logging

All snapshot operations are logged to the audit system:

- **snapshot_created**: When a snapshot is created
- **snapshot_downloaded**: When a snapshot is downloaded
- **snapshot_restored**: When a snapshot is restored

Each audit log includes:
- Snapshot ID
- Size information
- Raft metadata (index, term)
- Timestamp
- User information (when available)

## Error Handling

### Checksum Mismatch
- **Error**: "Snapshot integrity check failed: checksum mismatch"
- **Metric**: `secreton_raft_restore_failures{reason="checksum_mismatch"}`
- **Action**: Snapshot is rejected, restore aborted

### Signature Verification Failure
- **Error**: "Snapshot signature verification failed"
- **Metric**: `secreton_raft_restore_failures{reason="signature_verification"}`
- **Action**: Snapshot is rejected, restore aborted

### Decryption Failure
- **Error**: "Failed to decrypt snapshot"
- **Metric**: `secreton_raft_restore_failures{reason="decryption_failed"}`
- **Action**: Snapshot is rejected, restore aborted

### Decompression Failure
- **Error**: "Failed to decompress snapshot"
- **Metric**: `secreton_raft_restore_failures{reason="decompression_failed"}`
- **Action**: Snapshot is rejected, restore aborted

### Invalid Format
- **Error**: "Failed to parse snapshot data"
- **Metric**: `secreton_raft_restore_failures{reason="invalid_format"}`
- **Action**: Snapshot is rejected, restore aborted

## Usage Examples

### Create Snapshot

```bash
curl -X POST https://secreton.example.com/v1/sys/raft/snapshot \
  -H "Authorization: Bearer $TOKEN"
```

Response:
```json
{
  "success": true,
  "data": {
    "success": true,
    "message": "Snapshot snapshot-1234567890-abc123 created successfully",
    "snapshot": {
      "snapshot_id": "snapshot-1234567890-abc123",
      "created_at": 1234567890,
      "size_bytes": 1024,
      "compressed_size_bytes": 512,
      "last_included_index": 100,
      "last_included_term": 5,
      "checksum": "abc123...",
      "encrypted": true,
      "signature": "def456..."
    }
  }
}
```

### List Snapshots

```bash
curl https://secreton.example.com/v1/sys/raft/snapshots \
  -H "Authorization: Bearer $TOKEN"
```

Response:
```json
{
  "success": true,
  "data": {
    "snapshots": [
      {
        "snapshot_id": "snapshot-1234567890-abc123",
        "created_at": 1234567890,
        "size_bytes": 1024,
        "compressed_size_bytes": 512,
        "last_included_index": 100,
        "last_included_term": 5,
        "checksum": "abc123...",
        "encrypted": true,
        "signature": "def456..."
      }
    ],
    "total": 1
  }
}
```

### Download Snapshot

```bash
curl https://secreton.example.com/v1/sys/raft/snapshot?snapshot_id=snapshot-1234567890-abc123 \
  -H "Authorization: Bearer $TOKEN" \
  -o snapshot.enc
```

### Restore Snapshot

```bash
curl -X POST https://secreton.example.com/v1/sys/raft/restore \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"snapshot_id": "snapshot-1234567890-abc123"}'
```

Response:
```json
{
  "success": true,
  "data": {
    "success": true,
    "message": "Snapshot snapshot-1234567890-abc123 restored successfully",
    "snapshot": {
      "snapshot_id": "snapshot-1234567890-abc123",
      "created_at": 1234567890,
      "size_bytes": 1024,
      "compressed_size_bytes": 512,
      "last_included_index": 100,
      "last_included_term": 5,
      "checksum": "abc123...",
      "encrypted": true,
      "signature": "def456..."
    }
  }
}
```

## Production Considerations

### Backup Strategy
1. **Automated Snapshots**: Schedule regular snapshot creation (e.g., daily)
2. **Off-site Storage**: Download and store snapshots in external backup system
3. **Retention Policy**: Adjust retention count based on storage capacity
4. **Testing**: Regularly test restore procedures

### Performance Impact
- Snapshot creation is CPU-intensive (compression, encryption)
- Recommended to create snapshots during low-traffic periods
- Snapshot size depends on cluster state size
- Compression typically achieves 50-70% size reduction

### Security Best Practices
1. **Access Control**: Restrict snapshot operations to admin users only
2. **Encryption Keys**: Protect encryption keys used for snapshots
3. **Audit Logging**: Monitor all snapshot operations
4. **Signature Verification**: Always verify signatures before restore
5. **Secure Storage**: Store downloaded snapshots in secure location

## Future Enhancements

1. **Incremental Snapshots**: Only store changes since last snapshot
2. **Compression Algorithms**: Support multiple compression algorithms (zstd, lz4)
3. **External Storage**: Support S3, Azure Blob, GCS for snapshot storage
4. **Scheduled Snapshots**: Built-in scheduler for automatic snapshots
5. **Snapshot Metadata**: Enhanced metadata with tags, descriptions
6. **Snapshot Verification**: Automated integrity checks on stored snapshots
7. **Point-in-Time Recovery**: Restore to specific log index/term
8. **Snapshot Streaming**: Stream large snapshots instead of loading into memory

## Troubleshooting

### Snapshot Creation Fails
- Check disk space on database server
- Verify Raft cluster is healthy
- Check encryption service is available
- Review logs for specific error messages

### Restore Fails with Checksum Mismatch
- Snapshot may be corrupted
- Try downloading snapshot again
- Verify snapshot was not modified
- Check for storage corruption

### Cannot Create Snapshot on Non-Leader
- Only leader node can create snapshots
- Check cluster status to identify leader
- Connect to leader node for snapshot operations

### Snapshot Size Too Large
- Consider increasing compression level
- Review what data is being snapshotted
- Implement data cleanup before snapshot
- Consider incremental snapshots (future enhancement)

## References

- [Raft Consensus Algorithm](https://raft.github.io/)
- [OpenRaft Documentation](https://docs.rs/openraft/)
- [Secreton Architecture](./cluster-management.md)
- [Security Best Practices](../docs/SECURITY_VALIDATION_COMPLETE.md)
