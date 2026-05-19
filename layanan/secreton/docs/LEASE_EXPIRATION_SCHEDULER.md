# Lease Expiration Scheduler

## Overview

The Lease Expiration Scheduler is a background task that automatically monitors and revokes expired leases. It runs periodically to ensure that expired leases are cleaned up promptly and optionally sends notifications before leases expire.

## Features

- **Automatic Expiration**: Periodically checks for expired leases and revokes them automatically
- **Configurable Interval**: Check interval can be configured (default: 60 seconds)
- **Expiration Notifications**: Optional notifications for leases expiring soon (default: 5 minutes before expiry)
- **Metrics Integration**: Exposes Prometheus metrics for monitoring
- **Graceful Error Handling**: Continues operation even if individual lease revocations fail
- **Cascade Revocation**: Automatically revokes child leases when parent lease expires

## Configuration

### Via Configuration File

Add the following to your `secreton.toml` configuration file:

```toml
[lease]
# Check interval in seconds (default: 60)
check_interval_secs = 60

# Notification threshold in seconds (default: 300 = 5 minutes)
notification_threshold_secs = 300

# Enable notifications before expiration (default: false)
enable_notifications = false
```

### Via Environment Variables

```bash
SECRETON__LEASE__CHECK_INTERVAL_SECS=60
SECRETON__LEASE__NOTIFICATION_THRESHOLD_SECS=300
SECRETON__LEASE__ENABLE_NOTIFICATIONS=false
```

### Programmatic Configuration

```rust
use secreton_core::services::lease::{LeaseManager, LeaseSchedulerConfig};
use std::sync::Arc;

// Create custom scheduler configuration
let scheduler_config = LeaseSchedulerConfig {
    check_interval_secs: 60,
    notification_threshold_secs: 300,
    enable_notifications: true,
};

// Create lease manager with custom config
let manager = Arc::new(LeaseManager::with_config(pool, scheduler_config));

// Optionally add metrics registry
let metrics_registry = Arc::new(MetricsRegistry::new());
let manager = manager.with_metrics(metrics_registry);

// Start the scheduler
let scheduler_handle = manager.start_expiration_scheduler();

// The scheduler runs in the background until the handle is dropped or aborted
// To stop the scheduler:
// scheduler_handle.abort();
```

## Usage Example

### Basic Usage

```rust
use secreton_core::services::lease::LeaseManager;
use deadpool_postgres::Pool;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Create database connection pool
    let pool = create_pool().await;

    // Create lease manager with default configuration
    let manager = Arc::new(LeaseManager::new(pool));

    // Start the expiration scheduler
    let scheduler_handle = manager.clone().start_expiration_scheduler();

    // Your application logic here...
    // The scheduler runs in the background

    // When shutting down, abort the scheduler
    scheduler_handle.abort();
}
```

### With Metrics

```rust
use secreton_core::services::lease::LeaseManager;
use secreton_core::services::metrics::MetricsRegistry;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let pool = create_pool().await;

    // Create metrics registry
    let metrics_registry = Arc::new(MetricsRegistry::new());

    // Create lease manager with metrics
    let manager = Arc::new(
        LeaseManager::new(pool)
            .with_metrics(metrics_registry.clone())
    );

    // Start scheduler
    let scheduler_handle = manager.clone().start_expiration_scheduler();

    // Metrics are automatically updated by the scheduler
    // Access metrics via the registry
    let prometheus_output = metrics_registry.export_prometheus().await;
    println!("{}", prometheus_output);
}
```

### With Custom Configuration

```rust
use secreton_core::services::lease::{LeaseManager, LeaseSchedulerConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let pool = create_pool().await;

    // Create custom configuration
    let config = LeaseSchedulerConfig {
        check_interval_secs: 30,        // Check every 30 seconds
        notification_threshold_secs: 600, // Notify 10 minutes before expiry
        enable_notifications: true,      // Enable notifications
    };

    // Create manager with custom config
    let manager = Arc::new(LeaseManager::with_config(pool, config));

    // Start scheduler
    let scheduler_handle = manager.clone().start_expiration_scheduler();

    // Application logic...
}
```

## Metrics

The scheduler exposes the following Prometheus metrics:

### Counters

- `secreton_leases_expired_total`: Total number of leases expired by scheduler
- `secreton_leases_revoked_total`: Total number of leases revoked by scheduler (includes children)
- `secreton_leases_expiration_notifications_total`: Total number of expiration notifications sent
- `secreton_leases_expiration_errors_total`: Total number of errors during expiration checks

### Gauges

- `secreton_leases_active`: Current number of active leases (updated by other operations)

### Example Prometheus Output

```
# HELP secreton_leases_expired_total Total number of leases expired by scheduler
# TYPE secreton_leases_expired_total counter
secreton_leases_expired_total 42

# HELP secreton_leases_revoked_total Total number of leases revoked by scheduler
# TYPE secreton_leases_revoked_total counter
secreton_leases_revoked_total 45

# HELP secreton_leases_expiration_notifications_total Total number of lease expiration notifications sent
# TYPE secreton_leases_expiration_notifications_total counter
secreton_leases_expiration_notifications_total 12

# HELP secreton_leases_expiration_errors_total Total number of errors during expiration checks
# TYPE secreton_leases_expiration_errors_total counter
secreton_leases_expiration_errors_total 0
```

## Logging

The scheduler emits structured logs at various levels:

### Info Level

```
Starting lease expiration scheduler with check interval: 60s
Lease expiration check completed: 5 expired, 7 revoked, 2 notified
Lease abc123 for user user1 will expire in 4 minutes (resource: /secret/data/db)
```

### Debug Level

```
Running lease expiration check
Lease expiration check completed: no leases to process
Revoked expired lease: abc123 (and 2 children)
Sent 3 expiration notifications
```

### Warning Level

```
Failed to revoke expired lease abc123: Storage error: connection timeout
Failed to send expiration notifications: notification service unavailable
```

### Error Level

```
Lease expiration check failed: Database connection pool exhausted
```

## Behavior

### Expiration Check Cycle

Each check cycle performs the following steps:

1. **Query Expired Leases**: Finds all leases with `status = 'active'` and `expired_at < NOW()`
2. **Revoke Leases**: Calls `revoke_lease()` for each expired lease
3. **Cascade Revocation**: Automatically revokes child leases (depth-first)
4. **Execute Callbacks**: Executes revoke callbacks if specified
5. **Send Notifications**: If enabled, sends notifications for leases expiring soon
6. **Update Metrics**: Updates Prometheus metrics with operation results
7. **Log Results**: Logs summary of operations performed

### Error Handling

- Individual lease revocation failures are logged but don't stop the scheduler
- Database connection errors are logged and the scheduler continues on next cycle
- Metrics track the number of errors encountered
- The scheduler never crashes - it continues running even after errors

### Notification Mechanism

When `enable_notifications = true`:

1. Queries leases expiring within `notification_threshold_secs`
2. Logs notification messages (INFO level)
3. TODO: Implement actual notification delivery (webhook, email, etc.)
4. Updates notification metrics

## Performance Considerations

### Check Interval

- **Lower values** (e.g., 10-30 seconds): More responsive but higher database load
- **Higher values** (e.g., 60-300 seconds): Lower database load but less responsive
- **Recommended**: 60 seconds for most deployments

### Database Impact

Each check cycle performs:

- 1 query to find expired leases
- 1 query per expired lease to revoke it
- 1 query to find expiring-soon leases (if notifications enabled)
- Additional queries for child lease cascade

For 1000 active leases with 1% expiring per minute:

- ~10 expired leases per check
- ~10 revocation queries
- ~20 total queries per minute (with 60s interval)

### Memory Usage

- In-memory cache stores recently accessed leases
- Cache is automatically cleaned during expiration checks
- Minimal memory overhead from scheduler task itself

## Integration with Application

### Startup Integration

```rust
use secreton_core::services::lease::LeaseManager;
use std::sync::Arc;

pub struct Application {
    lease_manager: Arc<LeaseManager>,
    scheduler_handle: Option<tokio::task::JoinHandle<()>>,
}

impl Application {
    pub async fn start(&mut self) {
        // Start lease expiration scheduler
        let handle = self.lease_manager.clone().start_expiration_scheduler();
        self.scheduler_handle = Some(handle);

        tracing::info!("Lease expiration scheduler started");
    }

    pub async fn shutdown(&mut self) {
        // Abort scheduler on shutdown
        if let Some(handle) = self.scheduler_handle.take() {
            handle.abort();
            tracing::info!("Lease expiration scheduler stopped");
        }
    }
}
```

### Health Check Integration

```rust
pub async fn health_check(manager: Arc<LeaseManager>) -> Result<HealthStatus> {
    // Check if scheduler is running by verifying recent activity
    let stats = manager.get_stats().await?;

    Ok(HealthStatus {
        healthy: true,
        active_leases: stats.active_count,
        expired_leases: stats.expired_count,
    })
}
```

## Troubleshooting

### Scheduler Not Running

**Symptom**: Expired leases are not being revoked

**Solutions**:

1. Check logs for scheduler startup message
2. Verify scheduler handle is not dropped prematurely
3. Check for database connection issues in error logs

### High Database Load

**Symptom**: Database CPU/IO usage is high

**Solutions**:

1. Increase `check_interval_secs` to reduce query frequency
2. Add database indexes on `leases.status` and `leases.expired_at`
3. Consider batch revocation for large numbers of expired leases

### Notifications Not Sent

**Symptom**: No notification logs appearing

**Solutions**:

1. Verify `enable_notifications = true` in configuration
2. Check that leases are within notification threshold
3. Implement actual notification delivery mechanism (currently logs only)

### Memory Growth

**Symptom**: Application memory usage grows over time

**Solutions**:

1. Verify cache cleanup is working (check logs)
2. Monitor cache size with custom metrics
3. Reduce cache TTL if needed

## Future Enhancements

- [ ] Webhook notification delivery
- [ ] Email notification delivery
- [ ] Configurable notification templates
- [ ] Batch revocation for performance
- [ ] Distributed scheduler coordination (for multi-node deployments)
- [ ] Lease renewal reminders
- [ ] Custom revocation callbacks
- [ ] Lease expiration analytics dashboard

## See Also

- [Lease Management API](./LEASE_MANAGEMENT_API.md)
- [Metrics and Monitoring](./METRICS.md)
- [Configuration Guide](./CONFIGURATION.md)
