# Database Connection Pool Monitoring

## Overview

The database connection pool monitoring system provides comprehensive metrics and health checks for the PostgreSQL connection pool. This ensures optimal performance and early detection of connection issues.

## Features

### 1. Connection Pool Metrics

The following metrics are exposed via the `/metrics` endpoint in Prometheus format:

- **authenc_db_pool_size** - Current pool size
- **authenc_db_pool_max_size** - Maximum pool size (configured: 50)
- **authenc_db_pool_available** - Available connections
- **authenc_db_pool_waiting** - Requests waiting for connections
- **authenc_db_pool_utilization** - Pool utilization percentage
- **authenc_db_connections_acquired_total** - Total connections acquired
- **authenc_db_connections_failures_total** - Total acquisition failures
- **authenc_db_connections_created_total** - New connections created
- **authenc_db_connections_reused_total** - Connections reused from pool
- **authenc_db_connection_reuse_rate** - Reuse rate percentage
- **authenc_db_acquisition_time_avg_microseconds** - Average acquisition time
- **authenc_db_wait_time_avg_microseconds** - Average wait time
- **authenc_db_health_checks_total** - Total health checks performed
- **authenc_db_health_check_success_rate** - Health check success ra
thenc_db_pool_healthy** - Pool health status (1 = healthy, 0 = unhealthy)

### 2. Optimized Pool Configuration

The connection pool is configured with production-optimized settings:

```rust
PoolConfigBuilder::default()
    .max_size(50)              // Maximum connections
    .min_idle(10)              // Minimum idle connections
    .timeout(Duration::from_secs(30))  // Connection timeout
    .idle_timeout(Duration::from_secs(600))  // 10 minutes
    .max_lifetime(Duration::from_secs(1800)) // 30 minutes
    .recycling_method(RecyclingMethod::Verified)  // Health verification
```

### 3. Periodic Health Monitoring

The `PoolMonitor` service performs periodic health checks:

- **Check Interval**: 30 seconds (configurable)
- **Connection Validation**: Executes `SELECT 1` to verify connections
- **Threshold Monitoring**:
  - Warning at 80% utilization
  - Critical at 90% utilization
  - Wait time warning at 100ms average

### 4. Connection Reuse Tracking

The system tracks connection reuse to optimize pool efficiency:

- Monitors new connection creation vs. reuse
- Calculates reuse rate percentage
- Helps identify connection churn issues

## Usage

### Starting the Pool Monitor

Add the pool monitor to your application startup:

```rust
use authenc::database::{PoolMonitor, PoolMonitorConfig};
use std::sync::Arc;

// In your application initialization
let database = Arc::new(Database::new(&config.database).await?);

// Start pool monitor as background task
let monitor = PoolMonitor::with_defaults(database.clone());
tokio::spawn(async move {
    monitor.start().await;
});
```

### Custom Monitor Configuration

```rust
let monitor_config = PoolMonitorConfig {
    check_interval: Duration::from_secs(60),  // Check every minute
    utilization_warning_threshold: 75.0,      // Warn at 75%
    utilization_critical_threshold: 85.0,     // Critical at 85%
    wait_time_warning_threshold_ms: 50,       // Warn at 50ms
    enable_stats_logging: true,
};

let monitor = PoolMonitor::new(database.clone(), monitor_config);
```

### Accessing Metrics

#### Prometheus Endpoint

```bash
curl http://localhost:8088/metrics
```

#### Health Check with Metrics

```bash
curl http://localhost:8088/health/metrics
```

Response:
```json
{
  "status": "healthy",
  "pool": {
    "size": 15,
    "max_size": 50,
    "available": 12,
    "waiting": 0,
    "utilization": "30.0%"
  },
  "metrics": {
    "total_acquired": 1523,
    "total_failures": 0,
    "reuse_rate": "95.2%",
    "avg_acquisition_time_us": 125,
    "avg_wait_time_us": 45,
    "health_check_success_rate": "100.0%"
  }
}
```

### Programmatic Access

```rust
// Get current pool statistics
let stats = database.pool_stats();
println!("Pool utilization: {:.1}%", stats.utilization);
println!("Reuse rate: {:.1}%", stats.reuse_rate);
println!("Avg wait time: {}μs", stats.avg_wait_time_us);

// Get pool health
let health = database.pool_health();
if !health.is_healthy() {
    warn!("Pool health: {}", health.status());
}

// Perform immediate validation
let validation = database.validate_connections().await?;
if !validation.healthy {
    error!("Pool validation failed: {:?}", validation.error);
}
```

## Monitoring Best Practices

### 1. Prometheus Alerts

Configure alerts for critical conditions:

```yaml
groups:
  - name: authenc_database
   - alert: HighPoolUtilization
        expr: authenc_db_pool_utilization > 90
        for: 5m
        annotations:
ummary: "Database pool utilization above 90%"

      - alert: HighConnectionWaitTime
        expr: authenc_db_wait_time_avg_microseconds > 100000
        for: 5m
        annotations:
          summary: "Average connection wait time above 100ms"

      - alert: LowConnectionReuseRate
        expr: authenc_db_connection_reuse_rate < 70
        for: 10m
        annotations:
          summary: "Connection reuse rate below 70%"

      - alert: PoolUnhealthy
        expr: authenc_db_pool_healthy == 0
        for: 1m
        annotations:
          summary: "Database connection pool is unhealthy"
```

### 2. Grafana Dashboard

Create a dashboard with panels for:

- Pool utilization over time (line graph)
- Available vs. waiting connections (stacked area)
- Connection acquisition time (histogram)
- Connection reuse rate (gauge)
- Health check success rate (gauge)

### 3. Log Analysis

Monitor logs for warnings:

```
WARNING: Connection pool utilization at 85.0% (threshold: 80.0%)
WARNING: Average connection wait time is 120ms (threshold: 100ms)
CRITICAL: Connection pool utilization at 92.0% (threshold: 90.0%)
```

## Troubleshooting

### High Pool Utilization

**Symptoms**: Pool utilization consistently above 80%

**Solutions**:
1. Increase `max_connections` in configuration
2. Optimize slow queries to release connections faster
3. Review application connection usage patterns
4. Consider connection pooling at application level

### High Wait Times

**Symptoms**: Average wait time above 100ms

**Solutions**:
1. Increase pool size
2. Reduce connection lifetime to force recycling
3. Check for connection leaks (not properly released)
4. Optimize database query performance

### Low Reuse Rate

**Symptoms**: Reuse rate below 70%

**Solutions**:
1. Increase `idle_timeout` to keep connections alive longer
2. Reduce `max_lifetime` if connections are being recycled too aggressively
3. Check for connection validation failures
4. Review connection recycling method

### Connection Failures

**Symptoms**: `total_failures` increasing

**Solutions**:
1. Check database server health
2. Verify network connectivity
3. Review connection timeout settings
4. Check PostgreSQL `max_connections` limit

## Performance Impact

The monitoring system is designed to have minimal performance impact:

- **Metrics Collection**: Atomic operations, no locks
- **Health Checks**: 30-second intervals (configurable)
- **Memory Overhead**: ~1KB per pool
- **CPU Overhead**: <0.1% on typical workloads

## Configuration Reference

### Database Configuration

```toml
[database]
host = "localhost"
port = 5432
username = "authenc"
password = "secret"
database = "authenc"
max_connections = 50          # Optimized for production
min_connections = 10          # Warm connections
connection_timeout = 30       # Seconds
idle_timeout = 600            # 10 minutes
max_lifetime = 1800           # 30 minutes
```

### Environment Variables

```bash
DATABASE_MAX_CONNECTIONS=50
DATABASE_MIN_CONNECTIONS=10
DATABASE_CONNECTION_TIMEOUT=30
DATABASE_IDLE_TIMEOUT=600
DATABASE_MAX_LIFETIME=1800
```

## Requirements Met

This implementation satisfies task 6.4 requirements:

✅ **Tune pool parameters**: max=50, min=10, timeout=30s
✅ **Add connection reuse metrics**: Tracks creation vs. reuse with percentage
✅ **Implement connection health monitoring**: Periodic validation with configurable intervals
✅ **Add connection wait time metrics**: Tracks and exposes average wait time

## Related Documentation

- [Database Optimization](./DATABASE_OPTIMIZATION.md)
- [Performance Tuning](./PERFORMANCE_TUNING.md)
- [Monitoring Guide](./MONITORING.md)
