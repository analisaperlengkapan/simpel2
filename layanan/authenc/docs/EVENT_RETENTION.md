# Event Retention Management

## Overview

The Event Retention Management system provides automated lifecycle management for audit events and logs in Authenc. It supports:

- **Configurable retention policies** for user and admin events
- **Cold storage archiving** to S3/MinIO before deletion
- **Automated cleanup scheduler** that runs daily
- **Comprehensive metrics** for monitoring retention operations
- **Batch processing** for efficient large-scale operations

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  Event Retention Service                     │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐      ┌──────────────┐                     │
│  │   Scheduler  │─────▶│   Cleanup    │                     │
│  │  (Daily Run) │      │   Process    │                     │
│  └──────────────┘      └──────┬───────┘                     │
│                               │                              │
│                               ▼                              │
│                    ┌──────────────────┐                      │
│                    │  Archive Events  │                      │
│                    │  (if enabled)    │                      │
│                    └────────┬─────────┘                      │
│                             │                                │
│                             ▼                                │
│                    ┌──────────────────┐                      │
│                    │  Upload to S3/   │                      │
│                    │  MinIO (batches) │                      │
│                    └────────┬─────────┘                      │
│                             │                                │
│                             ▼                                │
│                    ┌──────────────────┐                      │
│                    │  Delete Events   │                      │
│                    │  from Database   │                      │
│                    └──────────────────┘                      │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Configuration

### Basic Configuration

```toml
[events]
enabled = true
user_event_retention_days = 90
admin_event_retention_days = 365
max_cleanup_batch_size = 10000
cleanup_interval_hours = 24
archive_before_delete = true
```

### Cold Storage Configuration (S3)

```toml
[events.cold_storage]
enabled = true
storage_type = "s3"
region = "us-east-1"
bucket = "authenc-events-archive"
path_prefix = "authenc/events/archive"
force_path_style = false
```

### Cold Storage Configuration (MinIO)

```toml
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

## Features

### 1. Configurable Retention Policies

Different retention periods for different event types:

- **User Events**: Default 90 days (login, logout, MFA operations)
- **Admin Events**: Default 365 days (administrative actions, configuration changes)

### 2. Cold Storage Archiving

Before deletion, events are archived to S3/MinIO:

- **Batch Processing**: Events are archived in batches (configurable size)
- **JSON Format**: Events are stored as JSON for easy retrieval and analysis
- **Organized Structure**: Archives are organized by date and table name
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

### 3. Automated Cleanup Scheduler

- **Daily Execution**: Runs every 24 hours (configurable)
- **Non-blocking**: Runs in background without affecting service performance
- **Error Handling**: Failures are logged and don't stop the service
- **Graceful Shutdown**: Waits for ongoing cleanup to complete

### 4. Comprehensive Metrics

Prometheus metrics for monitoring:

```
# Cleanup operations
authenc.event_retention.cleanup_total
authenc.event_retention.cleanup_duration_ms

# Event counts
authenc.event_retention.events_deleted_total
authenc.event_retention.events_archived_total
authenc.event_retention.archive_batches_total

# Current state
authenc.event_retention.total_user_events
authenc.event_retention.total_admin_events
authenc.event_retention.expired_user_events
authenc.event_retention.expired_admin_events
```

## Usage

### Initialization

```rust
use authenc::services::event_retention::EventRetentionService;
use authenc::config::EventsConfig;

// Create service
let retention_service = EventRetentionService::new(
    events_config,
    database,
    event_store,
).await?;

// Start automated cleanup scheduler
let service_arc = Arc::new(retention_service);
service_arc.clone().start_cleanup_task();
```

### Manual Cleanup

```rust
// Perform manual cleanup
let result = retention_service.perform_cleanup().await?;

println!("Deleted: {} events", result.total_events_deleted);
println!("Archived: {} events", result.total_events_archived);
println!("Duration: {}ms", result.duration_ms);
```

### Get Retention Statistics

```rust
// Get current retention statistics
let stats = retention_service.get_retention_stats().await?;

println!("Total user events: {}", stats.total_user_events);
println!("Expired user events: {}", stats.expired_user_events);
println!("Cold storage enabled: {}", stats.cold_storage_enabled);
```

## Deployment

### Kubernetes Configuration

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: authenc-config
data:
  events.toml: |
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
```

### IAM Policy for S3 (AWS)

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:PutObject",
        "s3:GetObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::kejaksaan-authenc-events/*",
        "arn:aws:s3:::kejaksaan-authenc-events"
      ]
    }
  ]
}
```

### MinIO Setup

```bash
# Create bucket
mc mb minio/authenc-events

# Set bucket policy (optional)
mc policy set download minio/authenc-events

# Create access credentials
mc admin user add minio authenc-service <password>
mc admin policy attach minio readwrite --user authenc-service
```

## Monitoring

### Grafana Dashboard Queries

**Events Deleted Over Time**:
```promql
rate(authenc_event_retention_events_deleted_total[5m])
```

**Events Archived Over Time**:
```promql
rate(authenc_event_retention_events_archived_total[5m])
```

**Cleanup Duration**:
```promql
histogram_quantile(0.95, rate(authenc_event_retention_cleanup_duration_ms_bucket[5m]))
```

**Expired Events Pending Cleanup**:
```promql
authenc_event_retention_expired_user_events + authenc_event_retention_expired_admin_events
```

### Alerts

**High Number of Expired Events**:
```yaml
- alert: HighExpiredEvents
  expr: authenc_event_retention_expired_user_events > 100000
  for: 1h
  annotations:
    summary: "High number of expired events pending cleanup"
```

**Cleanup Failures**:
```yaml
- alert: CleanupFailures
  expr: increase(authenc_event_retention_cleanup_failures_total[1h]) > 3
  annotations:
    summary: "Event retention cleanup is failing"
```

## Best Practices

1. **Set Appropriate Retention Periods**
   - Balance compliance requirements with storage costs
   - User events: 90 days is typical for operational data
   - Admin events: 365 days or more for audit compliance

2. **Enable Cold Storage Archiving**
   - Always archive before deletion for compliance
   - Use lifecycle policies on S3 to move to cheaper storage tiers
   - Consider Glacier for long-term archival

3. **Monitor Cleanup Operations**
   - Set up alerts for cleanup failures
   - Monitor cleanup duration to detect performance issues
   - Track archived event counts for capacity planning

4. **Test Archival and Retrieval**
   - Periodically test retrieving archived events
   - Verify JSON format integrity
   - Document retrieval procedures for compliance audits

5. **Optimize Batch Size**
   - Larger batches = fewer S3 operations = lower costs
   - Smaller batches = less memory usage = more stable
   - Default 10,000 is a good balance for most deployments

## Troubleshooting

### Cleanup Not Running

Check if retention is enabled:
```toml
[events]
enabled = true
```

Check logs for scheduler startup:
```
Event retention cleanup task started with interval: 24 hours
```

### Archiving Failures

**S3 Connection Issues**:
- Verify endpoint URL and region
- Check IAM credentials or instance profile
- Test S3 connectivity: `aws s3 ls s3://bucket-name`

**MinIO Connection Issues**:
- Verify endpoint is accessible from pod
- Check access key and secret key
- Ensure `force_path_style = true` for MinIO

### High Memory Usage

Reduce batch size:
```toml
max_cleanup_batch_size = 5000
```

### Slow Cleanup Operations

- Check database indexes on `time` column
- Monitor S3/MinIO upload speeds
- Consider running cleanup during off-peak hours

## Compliance Considerations

### GDPR

- Retention periods must align with data minimization principles
- Archived events may need to support "right to erasure" requests
- Document retention policies in privacy policy

### ISO 27001

- Maintain audit trail of retention operations
- Regular review of retention policies
- Secure storage of archived events

### Industry-Specific

- Financial services: Often require 7+ years retention
- Healthcare: HIPAA may require specific retention periods
- Government: May have statutory retention requirements

## Future Enhancements

- [ ] Compression of archived events (gzip)
- [ ] Encryption of archived events at rest
- [ ] Selective archiving based on event type
- [ ] Archive retrieval API
- [ ] Integration with data lake for analytics
- [ ] Support for Azure Blob Storage
- [ ] Support for Google Cloud Storage
