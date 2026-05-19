# Secreton Backup Manager

Automated backup and restore functionality for Secreton, providing comprehensive data protection and disaster recovery capabilities.

## Features

- **Automated Scheduled Backups**: Cron-based scheduling for automated backups
- **Complete Data Backup**: Backs up both Raft state and PostgreSQL data
- **Encryption**: All backups are encrypted with ChaCha20-Poly1305
- **Compression**: Optional gzip compression to reduce storage space
- **Pluggable Storage**: Support for multiple storage backends (local, S3, Azure, GCS)
- **Backup Verification**: Automatic verification with checksum validation
- **Point-in-Time Recovery**: Restore from any backup
- **Retention Policy**: Automatic cleanup of old backups
- **Metadata Tracking**: Comprehensive metadata for each backup

## Usage

### Creating a Backup Manager

```rust
use secreton_backup::prelude::*;
use secreton_backup::types::{BackupConfig, LocalStorageConfig, StorageConfig, StorageType};

#[tokio::main]
async fn main() -> Result<()> {
    let config = BackupConfig {
        enabled: true,
        schedule: "0 2 * * *".to_string(), // Daily at 2 AM
        retention_days: 30,
        storage_type: StorageType::Local,
        storage_config: StorageConfig::Local(LocalStorageConfig {
            path: "/var/backups/secreton".to_string(),
        }),
        encryption_key: None, // Will generate a new key
        compression_enabled: true,
        compression_level: 6,
        verify_after_backup: true,
    };

    let manager = BackupManager::new(config).await?;
    Ok(())
}
```

### Manual Backup

```rust
// Create a backup manually
let backup_id = manager.create_backup().await?;
println!("Backup created: {}", backup_id);
```

### Automated Backups

```rust
// Start automated backup scheduler
let mut manager = BackupManager::new(config).await?;
manager.start_scheduler().await?;

// Scheduler will run backups according to cron schedule
// ...

// Stop scheduler when done
manager.stop_scheduler().await?;
```

### Listing Backups

```rust
let backups = manager.list_backups().await?;

for backup in backups {
    println!("Backup ID: {}", backup.id);
    println!("Timestamp: {}", backup.timestamp);
    println!("Size: {}", backup.human_readable_size());
    println!("Compression ratio: {:.2}x", backup.compression_ratio());
    println!("Verified: {}", backup.is_verified());
}
```

### Verifying a Backup

```rust
// Verify backup integrity
manager.verify_backup(&backup_id).await?;
println!("Backup verified successfully");
```

### Restoring from Backup

```rust
use secreton_backup::types::RestoreOptions;

let options = RestoreOptions {
    backup_id: "backup-id-here".to_string(),
    restore_raft: true,
    restore_postgres: true,
    skip_verification: false,
    force: false,
};

manager.restore_backup(options).await?;
println!("Backup restored successfully");
```

### Deleting a Backup

```rust
manager.delete_backup(&backup_id).await?;
println!("Backup deleted");
```

## Configuration

### Cron Schedule Format

The backup schedule uses standard cron syntax:

```
┌───────────── minute (0 - 59)
│ ┌───────────── hour (0 - 23)
│ │ ┌───────────── day of month (1 - 31)
│ │ │ ┌───────────── month (1 - 12)
│ │ │ │ ┌───────────── day of week (0 - 6) (Sunday to Saturday)
│ │ │ │ │
│ │ │ │ │
* * * * *
```

Examples:

- `0 2 * * *` - Daily at 2:00 AM
- `0 */6 * * *` - Every 6 hours
- `0 0 * * 0` - Weekly on Sunday at midnight
- `0 0 1 * *` - Monthly on the 1st at midnight

### Storage Backends

#### Local Filesystem

```rust
StorageConfig::Local(LocalStorageConfig {
    path: "/var/backups/secreton".to_string(),
})
```

#### S3-Compatible Storage

```rust
use secreton_backup::types::{S3StorageConfig, StorageConfig, StorageType};

StorageConfig::S3(S3StorageConfig {
    bucket: "secreton-backups".to_string(),
    region: "us-east-1".to_string(),
    endpoint: Some("https://s3.amazonaws.com".to_string()), // Optional, for S3-compatible services
    access_key_id: Some("YOUR_ACCESS_KEY".to_string()),
    secret_access_key: Some("YOUR_SECRET_KEY".to_string()),
    prefix: Some("backups/".to_string()),
    force_path_style: false, // Set to true for MinIO
})
```

**MinIO Example:**

```rust
StorageConfig::S3(S3StorageConfig {
    bucket: "secreton-backups".to_string(),
    region: "us-east-1".to_string(),
    endpoint: Some("http://localhost:9000".to_string()),
    access_key_id: Some("minioadmin".to_string()),
    secret_access_key: Some("minioadmin".to_string()),
    prefix: Some("backups/".to_string()),
    force_path_style: true, // Required for MinIO
})
```

### Encryption

Backups are encrypted with ChaCha20-Poly1305 using a 32-byte encryption key. You can either:

1. **Provide your own key** (recommended for production):

   ```rust
   config.encryption_key = Some(your_32_byte_key);
   ```

2. **Let the manager generate a key** (for testing):

   ```rust
   config.encryption_key = None; // Will generate and log a warning
   ```

**Important**: Store the encryption key securely! Without it, backups cannot be restored.

### Compression

Compression is enabled by default with level 6 (balanced):

```rust
config.compression_enabled = true;
config.compression_level = 6; // 0-9, where 9 is maximum compression
```

Compression levels:

- `0`: No compression (fastest)
- `1-3`: Fast compression, lower ratio
- `4-6`: Balanced (recommended)
- `7-9`: Maximum compression, slower

## Backup Format

Each backup consists of:

1. **Raft Snapshot**: Complete Raft consensus state
2. **PostgreSQL Dump**: Full database dump
3. **Metadata**: Backup information and checksums

The backup process:

1. Create Raft snapshot
2. Dump PostgreSQL database
3. Compress data (if enabled)
4. Encrypt with ChaCha20-Poly1305
5. Calculate SHA-256 checksum
6. Upload to storage backend
7. Verify backup (if enabled)
8. Clean up old backups

## Metadata

Each backup includes comprehensive metadata:

```rust
pub struct BackupMetadata {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub original_size_bytes: u64,
    pub compressed_size_bytes: u64,
    pub encrypted_size_bytes: u64,
    pub raft_snapshot_size: u64,
    pub postgres_dump_size: u64,
    pub status: BackupStatus,
    pub checksum: String,
    pub encryption_algorithm: String,
    pub compression_algorithm: Option<String>,
    pub compression_level: Option<u32>,
    pub verified_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub tags: Vec<String>,
}
```

## Error Handling

The backup manager uses a comprehensive error type:

```rust
pub enum BackupError {
    Storage(String),
    Encryption(String),
    Decryption(String),
    Compression(String),
    Decompression(String),
    Serialization(String),
    Deserialization(String),
    NotFound(String),
    InvalidFormat(String),
    VerificationFailed(String),
    Scheduler(String),
    InvalidConfig(String),
    RaftSnapshot(String),
    PostgresDump(String),
    Restore(String),
    Io(std::io::Error),
    Other(String),
}
```

## Testing

Run the test suite:

```bash
cd crates/backup
cargo test
```

Run integration tests:

```bash
cargo test --test integration_tests
```

## Security Considerations

1. **Encryption Key Management**: Store encryption keys securely (e.g., in Secreton itself or a KMS)
2. **Storage Access**: Ensure backup storage has appropriate access controls
3. **Network Security**: Use TLS for remote storage backends
4. **Retention Policy**: Configure appropriate retention to balance storage costs and recovery needs
5. **Verification**: Always enable backup verification in production
6. **Disaster Recovery**: Test restore procedures regularly

## Performance

- **Compression**: Reduces backup size by 50-70% on average
- **Encryption**: Minimal overhead with ChaCha20-Poly1305
- **Parallel Operations**: Supports concurrent backup creation
- **Incremental**: Future versions will support incremental backups

## Roadmap

- [x] S3-compatible storage backend
- [ ] Azure Blob Storage backend
- [ ] Google Cloud Storage backend
- [ ] Incremental backups
- [ ] Backup encryption with KMS
- [ ] Backup streaming for large datasets
- [ ] Backup deduplication
- [ ] Multi-region replication
- [ ] Backup notifications (email, Slack, etc.)

## License

Apache-2.0

## Contributing

See the main Secreton repository for contribution guidelines.
