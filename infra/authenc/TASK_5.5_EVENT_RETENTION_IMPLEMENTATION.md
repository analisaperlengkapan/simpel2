# Task 5.5: Event Retention Management Implementation

## Overview

Implemented comprehensive event retention management with cold storage archiving support for Authenc IAM system.

## Implementation Summary

### 1. Dependencies Added

Added AWS SDK dependencies to `Cargo.toml` for S3/MinIO integration:
- `aws-sdk-s3 = "1.90.0"`
- `aws-config = "1.5.15"`
- `aws-credential-types = "1.2.1"`

### 2. Configuration Enhancements

Enhanced `EventsConfig` in `src/config/mod.rs`:

**New Configuration Structure**:
```rust
pub struct EventsConfig {
    pub enabled: bool,
    pub user_event_retention_days: u32,      // Default: 90 days
    pub admin_event_retention_days: u32,     // Default: 365 days
    pub max_cleanup_batch_size: u32,         // Default: 10,000
    pub cleanup_interval_hours: u32,         // Default: 24 hours
    pub archive_before_delete: bool,
    pub archive_directory: Option<String>,
    pub cold_storage: Option<ColdStorageConfig>,  // NEW
}

pub struct ColdStorageConfig {
    pub enabled: bool,
    pub storage_type: String,                // "s3" or "minio"
    pub endpoint: String,                    // Custom endpoint for MinIO
    pub region: String,                      // AWS region
    pub bucket: String,                      // Bucket name
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub path_prefix: String,                 // Path prefix in bucket
    pub force_path_style: bool,              // Required for MinIO
}
```

### 3. Event Retention Service Enhancements

Enhanced `src/services/event_retention.rs` with:

#### Cold Storage Archiving
- **S3/MinIO Integration**: Automatic archiving to object storage before deletion
- **Batch Processing**: Archives events in configurable batches (default: 10,000)
- **JSON Format**: Events stored as JSON for easy retrieval
- **Organized Structure**: Archives organized by date and table name
  ```
  authenc/events/archive/
  ├── events/
  │   ├── 2024-01-15-0.json
  │   ├── 2024-01-15-10000.json
  │   └── 2024-01-15-20000.json
  └── admin_events/
      ├── 2024-01-15-0.json
      └── 2024-01-15-10000.json
  ```

#### Enhanced Metrics
Added comprehensive Prometheus metrics:
- `authenc.event_retention.cleanup_total`: Total cleanup operations
- `authenc.event_retention.events_deleted_total`: Total events deleted
- `authenc.event_retention.events_archived_total`: Total events archived
- `authenc.event_retention.cleanup_duration_ms`: Cleanup operation duration
- `authenc.event_retention.archive_batches_total`: Archive batches uploaded
- `authenc.event_retention.total_user_events`: Current user events count
- `authenc.event_retention.total_admin_events`: Current admin events count
- `authenc.event_retention.expired_user_events`: Expired user events count
- `authenc.event_retention.expired_admin_events`: Expired admin events count

#### Improved Result Tracking
Enhanced `RetentionCleanupResult`:
```rust
pub struct RetentionCleanupResult {
    pub user_events_deleted: usize,
    pub admin_events_deleted: usize,
    pub total_events_deleted: usize,
    pub user_events_archived: usize,      // NEW
    pub admin_events_archived: usize,     // NEW
    pub total_events_archived: usize,     // NEW
    pub cleanup_time: DateTime<Utc>,
    pub duration_ms: u64,                 // NEW
}
```

#### Enhanced Statistics
Enhanced `RetentionStats`:
```rust
pub struct RetentionStats {
    // ... existing fields ...
    pub cold_storage_enabled: bool,       // NEW
    pub cold_storage_bucket: Option<String>,  // NEW
}
```

### 4. Key Features Implemented

#### Configurable Retention Policies
- Separate retention periods for user events (90 days) and admin events (365 days)
- Configurable cleanup interval (default: daily)
- Configurable batch size for efficient processing

#### Cold Storage Archiving
- Archive events to S3/MinIO before deletion
- Support for both AWS S3 and MinIO
- Automatic credential handling (IAM roles or explicit credentials)
- Path-style addressing support for MinIO
- Batch upload for efficiency

#### Automated Cleanup Scheduler
- Daily execution (configurable)
- Non-blocking background operation
- Error handling and logging
- Graceful shutdown support

#### Comprehensive Monitoring
- Prometheus metrics for all operations
- Detailed logging of cleanup operations
- Performance tracking (duration, batch counts)
- Current state monitoring (event counts, expired events)

### 5. Documentation Created

#### Configuration Example
Created `config/event_retention.example.toml` with:
- Complete configuration examples
- AWS S3 configuration
- MinIO configuration
- Detailed comments and explanations

#### Comprehensive Documentation
Created `docs/EVENT_RETENTION.md` with:
- Architecture overview
- Configuration guide
- Feature descriptions
- Usage examples
- Deployment guide
- Monitoring and alerting
- Best practices
- Troubleshooting guide
- Compliance considerations

### 6. Testing

Created `src/services/event_retention_tests.rs` with unit tests for:
- `RetentionCleanupResult` default values
- `ColdStorageConfig` default values
- `EventsConfig` with cold storage
- `RetentionStats` metrics update

### 7. Integration

Updated `src/app.rs` to:
- Initialize `EventRetentionService` with async support
- Start automated cleanup scheduler
- Handle cold storage configuration

## Configuration Examples

### AWS S3 Configuration
```toml
[events]
enabled = true
user_event_retention_days = 90
admin_event_retention_days = 365
archive_before_delete = true

[events.cold_storage]
enabled = true
storage_type = "s3"
region = "ap-southeast-1"
bucket = "kejaksaan-authenc-events"
path_prefix = "production/events/archive"
force_path_style = false
```

### MinIO Configuration
```toml
[events]
enabled = true
user_event_retention_days = 90
admin_event_retention_days = 365
archive_before_delete = true

[events.cold_storage]
enabled = true
storage_type = "minio"
endpoint = "http://minio.simpelv2-infra.svc.cluster.local:9000"
region = "us-east-1"
bucket = "authenc-events"
access_key_id = "minioadmin"
secret_access_key = "minioadmin"
path_prefix = "archive"
force_path_style = true
```

## Metrics Exposed

All metrics are prefixed with `authenc.event_retention.`:

| Metric | Type | Description |
|--------|------|-------------|
| `cleanup_total` | Counter | Total number of cleanup operations |
| `events_deleted_total` | Counter | Total events deleted |
| `events_archived_total` | Counter | Total events archived |
| `cleanup_duration_ms` | Histogram | Cleanup operation duration |
| `archive_batches_total` | Counter | Archive batches uploaded (by table) |
| `total_user_events` | Gauge | Current user events count |
| `total_admin_events` | Gauge | Current admin events count |
| `expired_user_events` | Gauge | Expired user events count |
| `expired_admin_events` | Gauge | Expired admin events count |
| `user_retention_days` | Gauge | User event retention period |
| `admin_retention_days` | Gauge | Admin event retention period |

## Usage Example

```rust
// Initialize service
let retention_service = EventRetentionService::new(
    events_config,
    database,
    event_store,
).await?;

// Start automated cleanup
let service_arc = Arc::new(retention_service);
service_arc.clone().start_cleanup_task();

// Manual cleanup
let result = service_arc.perform_cleanup().await?;
println!("Archived: {}, Deleted: {}",
    result.total_events_archived,
    result.total_events_deleted);

// Get statistics
let stats = service_arc.get_retention_stats().await?;
println!("Expired events: {}",
    stats.expired_user_events + stats.expired_admin_events);
```

## Deployment Considerations

### AWS S3
- Use IAM roles for EC2/EKS instances (no credentials needed)
- Set up S3 lifecycle policies for cost optimization
- Consider S3 Glacier for long-term archival

### MinIO
- Deploy MinIO in Kubernetes cluster
- Use persistent volumes for data storage
- Configure access policies appropriately

### Monitoring
- Set up Grafana dashboards for metrics
- Configure alerts for cleanup failures
- Monitor expired event counts

## Compliance

The implementation supports:
- **GDPR**: Configurable retention periods, right to erasure
- **ISO 27001**: Audit trail, secure storage
- **Industry-specific**: Flexible retention periods (90-365+ days)

## Requirements Fulfilled

✅ **Requirement 10.3**: Event archiving to cold storage (S3/MinIO)
✅ **Requirement 10.3**: Configurable retention policies (default: 90 days)
✅ **Requirement 10.3**: Cleanup scheduler (runs daily)
✅ **Requirement 10.3**: Event retention metrics

## Files Modified

1. `infra/authenc/Cargo.toml` - Added AWS SDK dependencies
2. `infra/authenc/src/config/mod.rs` - Enhanced EventsConfig with ColdStorageConfig
3. `infra/authenc/src/services/event_retention.rs` - Implemented cold storage archiving
4. `infra/authenc/src/app.rs` - Updated initialization to await async new()
5. `infra/authenc/src/services/event_publisher.rs` - Fixed typo (ckoff_ms -> initial_backoff_ms)

## Files Created

1. `infra/authenc/src/services/event_retention_tests.rs` - Unit tests
2. `infra/authenc/config/event_retention.example.toml` - Configuration example
3. `infra/authenc/docs/EVENT_RETENTION.md` - Comprehensive documentation
4. `infra/authenc/TASK_5.5_EVENT_RETENTION_IMPLEMENTATION.md` - This summary

## Next Steps

1. **Testing**: Run integration tests with actual S3/MinIO instance
2. **Monitoring**: Set up Grafana dashboards for retention metrics
3. **Deployment**: Deploy with cold storage configuration
4. **Documentation**: Update main README with event retention section
5. **Compliance**: Document retention policies for audit purposes

## Notes

- The implementation is backward compatible (cold storage is optional)
- Default configuration has cold storage disabled
- All operations are logged for audit trail
- Graceful error handling prevents service disruption
- Batch processing ensures efficient resource usage
