# Audit Logging & Token Management

## 📋 Overview

Sistem audit logging dan token management yang comprehensive untuk:

1. **Audit Trail** - Track semua API calls
2. **Token Management** - Manage dan track token lifecycle
3. **Monitoring** - Real-time monitoring dan analytics
4. **Compliance** - Meet audit requirements
5. **Debugging** - Easy troubleshooting

## 🗄️ Database Schema

### 1. API Call Logging

#### Table: `api_call_log`

Track semua API calls ke MonSAKTI dan MySIMKARI.

**Columns:**

- `module` - ADM, ANG, BEN, etc.
- `endpoint` - refAdmin, dataAng, etc.
- `kode_kl` - KL code
- `kdsatker` - Satker code
- `full_url` - Complete URL
- `response_status` - HTTP status code
- `response_time_ms` - Response time
- `record_count` - Records returned
- `success` - Success/failure
- `error_message` - Error details
- `retry_count` - Number of retries
- `token_refreshed` - Was token refreshed?
- `storage_strategy` - database/json/csv/both
- `data_saved` - Was data saved successfully?

**Indexes:**

- By module, endpoint, kode_kl, kdsatker
- By success status
- By timestamp
- Partial index for failed calls

**Usage:**

```rust
use monsakti_fetcher::ApiCallLog;

let mut log = ApiCallLog::new("ADM", "refAdmin", &url);
log.kode_kl = Some("006".to_string());
log.success = true;
log.record_count = Some(150);
log.save(&db).await?;
```

### 2. Batch Processing Logging

#### Table: `batch_processing_log`

Track batch operations.

**Columns:**

- `batch_type` - complete, satker, global, mysimkari
- `kode_kl` - KL code
- `total_satker` - Total satker to process
- `processed_satker` - Successfully processed
- `failed_satker` - Failed
- `total_api_calls` - Total API calls made
- `successful_calls` - Successful calls
- `failed_calls` - Failed calls
- `status` - running, completed, failed, cancelled
- `duration_seconds` - Total duration
- `storage_strategy` - Storage used
- `parallel_mode` - Parallel processing?

**Usage:**

```rust
use monsakti_fetcher::BatchProcessingLog;

let batch = BatchProcessingLog::new("complete", Some("006"));
let batch_id = batch.start(&db).await?;

// During processing
BatchProcessingLog::update(&db, batch_id, 10, 0, 100, 95, 5).await?;

// On completion
BatchProcessingLog::complete(&db, batch_id, "completed", None).await?;
```

### 3. Data Sync Logging

#### Table: `data_sync_log`

Track data synchronization to database.

**Columns:**

- `table_name` - Target table
- `module` - Module name
- `endpoint` - Endpoint name
- `records_fetched` - Records from API
- `records_inserted` - Records inserted
- `records_failed` - Failed records
- `batch_id` - Reference to batch
- `api_call_id` - Reference to API call
- `success` - Success/failure

**Usage:**

```rust
use monsakti_fetcher::DataSyncLog;

let mut sync_log = DataSyncLog::new("adm_pejabat", "adm", "pejabat");
sync_log.records_fetched = 150;
sync_log.records_inserted = 148;
sync_log.records_failed = 2;
sync_log.success = true;
sync_log.save(&db, Some(batch_id), Some(api_call_id)).await?;
```

### 4. Token Management

#### Table: `api_tokens`

Current active tokens for each module.

**Columns:**

- `module` - Module name (UNIQUE)
- `token_value` - Encrypted token
- `token_hash` - SHA256 hash
- `kode_kl` - KL code
- `is_active` - Active status
- `is_expired` - Expired status
- `usage_count` - Times used
- `last_used_at` - Last usage
- `consecutive_failures` - Failure count
- `refreshed_at` - Last refresh

**Security:**

- Token value should be encrypted at application level
- Token hash for verification without exposing token
- Only first 50 chars logged for tracking

#### Table: `api_token_history`

Complete history of token changes.

**Columns:**

- `token_id` - Reference to api_tokens
- `module` - Module name
- `event_type` - created, refreshed, expired, reset, revoked
- `event_reason` - Why this happened
- `triggered_by` - system, user, api, auto
- `old_token_hash` - Previous token hash
- `new_token_hash` - New token hash

**Auto-logging:**

- Trigger automatically logs token changes
- No manual intervention needed

#### Table: `token_reset_log`

Log of token reset operations.

**Columns:**

- `module` - Module name
- `kode_kl` - KL code
- `reset_reason` - expired, failed, manual, scheduled
- `reset_method` - auto, manual, api
- `success` - Success/failure
- `new_token_received` - Got new token?

**Usage:**

```rust
use monsakti_fetcher::TokenResetLog;

let mut reset_log = TokenResetLog::new("ADM", "006", "expired", "auto");
reset_log.success = true;
reset_log.new_token_received = true;
reset_log.save(&db, Some(api_call_id)).await?;
```

#### Table: `token_rotation_policy`

Token rotation policies per module.

**Columns:**

- `module` - Module name (UNIQUE)
- `rotation_enabled` - Enable rotation?
- `rotation_interval_hours` - Rotate every N hours
- `max_usage_count` - Max uses before rotation
- `max_consecutive_failures` - Auto-reset threshold
- `notify_on_rotation` - Send notification?
- `next_rotation_at` - Next scheduled rotation

**Default Policy:**

- Rotation every 24 hours
- Auto-reset after 3 consecutive failures
- Notifications disabled by default

## 📊 Views for Analytics

### 1. `v_api_stats_by_module`

API call statistics by module and date.

```sql
SELECT * FROM v_api_stats_by_module
WHERE call_date >= CURRENT_DATE - 7
ORDER BY call_date DESC, module;
```

### 2. `v_recent_failed_calls`

Most recent 100 failed API calls.

```sql
SELECT * FROM v_recent_failed_calls;
```

**Rust Usage:**

```rust
use monsakti_fetcher::get_recent_failed_calls;

let failed_calls = get_recent_failed_calls(&db, 50).await?;
for call in failed_calls {
    println!("Failed: {} - {}", call["module"], call["error_message"]);
}
```

### 3. `v_token_health`

Health status of all tokens.

```sql
SELECT * FROM v_token_health
WHERE health_status IN ('critical', 'warning');
```

**Rust Usage:**

```rust
use monsakti_fetcher::get_token_health;

let health = get_token_health(&db).await?;
for token in health {
    if token["health_status"] == "critical" {
        println!("CRITICAL: {} has {} failures",
            token["module"], token["consecutive_failures"]);
    }
}
```

### 4. `v_batch_summary`

Summary of batch operations.

```sql
SELECT * FROM v_batch_summary
WHERE started_at >= CURRENT_DATE - 7
ORDER BY started_at DESC;
```

### 5. `v_daily_sync_stats`

Daily synchronization statistics.

```sql
SELECT * FROM v_daily_sync_stats
WHERE sync_date >= CURRENT_DATE - 30
ORDER BY sync_date DESC;
```

## 🔍 Common Queries

### Find Slow API Calls

```sql
SELECT module, endpoint, AVG(response_time_ms) as avg_time
FROM api_call_log
WHERE started_at >= CURRENT_DATE - 7
GROUP BY module, endpoint
HAVING AVG(response_time_ms) > 5000
ORDER BY avg_time DESC;
```

### Token Reset Frequency

```sql
SELECT module, COUNT(*) as reset_count,
       SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful
FROM token_reset_log
WHERE reset_at >= CURRENT_DATE - 30
GROUP BY module
ORDER BY reset_count DESC;
```

### Failed Satker Processing

```sql
SELECT kdsatker, COUNT(*) as failure_count,
       MAX(started_at) as last_failure
FROM api_call_log
WHERE success = false
  AND kdsatker IS NOT NULL
  AND started_at >= CURRENT_DATE - 7
GROUP BY kdsatker
HAVING COUNT(*) > 5
ORDER BY failure_count DESC;
```

### Batch Processing Performance

```sql
SELECT batch_type,
       AVG(duration_seconds) as avg_duration,
       AVG(total_api_calls) as avg_calls,
       AVG(CAST(successful_calls AS FLOAT) / NULLIF(total_api_calls, 0) * 100) as success_rate
FROM batch_processing_log
WHERE completed_at >= CURRENT_DATE - 30
  AND status = 'completed'
GROUP BY batch_type;
```

## 📈 Monitoring Dashboard Queries

### Real-time Status

```sql
-- Active batch operations
SELECT * FROM batch_processing_log
WHERE status = 'running'
ORDER BY started_at DESC;

-- Recent API calls (last hour)
SELECT module, COUNT(*) as calls,
       SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful
FROM api_call_log
WHERE started_at >= CURRENT_TIMESTAMP - INTERVAL '1 hour'
GROUP BY module;

-- Token health
SELECT * FROM v_token_health
WHERE health_status != 'healthy';
```

### Daily Report

```sql
-- Daily summary
SELECT
    DATE(started_at) as date,
    COUNT(*) as total_calls,
    SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful,
    SUM(CASE WHEN NOT success THEN 1 ELSE 0 END) as failed,
    AVG(response_time_ms) as avg_response_time,
    SUM(record_count) as total_records
FROM api_call_log
WHERE started_at >= CURRENT_DATE - 7
GROUP BY DATE(started_at)
ORDER BY date DESC;
```

## 🔧 Maintenance

### Cleanup Old Logs

```sql
-- Delete logs older than 90 days
DELETE FROM api_call_log
WHERE created_at < CURRENT_DATE - INTERVAL '90 days';

DELETE FROM batch_processing_log
WHERE created_at < CURRENT_DATE - INTERVAL '90 days';

-- Keep token history for 1 year
DELETE FROM api_token_history
WHERE created_at < CURRENT_DATE - INTERVAL '1 year';
```

### Archive Old Data

```sql
-- Archive to separate table
CREATE TABLE api_call_log_archive AS
SELECT * FROM api_call_log
WHERE created_at < CURRENT_DATE - INTERVAL '90 days';

-- Then delete from main table
DELETE FROM api_call_log
WHERE created_at < CURRENT_DATE - INTERVAL '90 days';
```

## 🚨 Alerts

### Critical Token Failures

```sql
-- Tokens with 3+ consecutive failures
SELECT module, consecutive_failures, last_failure_at
FROM api_tokens
WHERE consecutive_failures >= 3
  AND is_active = true;
```

### High Error Rate

```sql
-- Modules with >10% error rate today
SELECT module,
       COUNT(*) as total,
       SUM(CASE WHEN NOT success THEN 1 ELSE 0 END) as errors,
       ROUND(SUM(CASE WHEN NOT success THEN 1 ELSE 0 END)::NUMERIC / COUNT(*) * 100, 2) as error_rate
FROM api_call_log
WHERE started_at >= CURRENT_DATE
GROUP BY module
HAVING SUM(CASE WHEN NOT success THEN 1 ELSE 0 END)::NUMERIC / COUNT(*) > 0.1;
```

### Stuck Batch Operations

```sql
-- Batch operations running > 2 hours
SELECT id, batch_type, kode_kl,
       EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - started_at))/3600 as hours_running
FROM batch_processing_log
WHERE status = 'running'
  AND started_at < CURRENT_TIMESTAMP - INTERVAL '2 hours';
```

## 📝 Best Practices

### 1. Always Log API Calls

```rust
let start = std::time::Instant::now();
let mut log = ApiCallLog::new("ADM", "refAdmin", &url);

match client.fetch("ADM", "refAdmin", vars).await {
    Ok(response) => {
        log.success = true;
        log.response_status = Some(200);
        log.record_count = response.data.as_ref()
            .and_then(|d| d.as_array())
            .map(|a| a.len() as i32);
    }
    Err(e) => {
        log.success = false;
        log.error_message = Some(e.to_string());
    }
}

log.response_time_ms = Some(start.elapsed().as_millis() as i32);
log.save(&db).await?;
```

### 2. Track Batch Operations

```rust
let batch = BatchProcessingLog::new("complete", Some("006"));
let batch_id = batch.start(&db).await?;

// Your batch processing logic here

BatchProcessingLog::complete(&db, batch_id, "completed", None).await?;
```

### 3. Log Token Resets

```rust
let mut reset_log = TokenResetLog::new("ADM", "006", "expired", "auto");

match client.reset_token("ADM", "refAdmin", "KL006").await {
    Ok(new_token) => {
        reset_log.success = true;
        reset_log.new_token_received = true;
    }
    Err(e) => {
        reset_log.success = false;
        reset_log.error_message = Some(e.to_string());
    }
}

reset_log.save(&db, None).await?;
```

### 4. Monitor Token Health

```rust
// Check token health periodically
let health = get_token_health(&db).await?;

for token in health {
    if token["health_status"] == "critical" {
        // Send alert
        eprintln!("ALERT: Token {} is critical!", token["module"]);
    }
}
```

## 🎯 Benefits

### Audit & Compliance

- ✅ Complete audit trail
- ✅ Track who, what, when
- ✅ Meet compliance requirements
- ✅ Easy to generate reports

### Debugging

- ✅ Quick error identification
- ✅ Performance analysis
- ✅ Pattern detection
- ✅ Root cause analysis

### Monitoring

- ✅ Real-time status
- ✅ Health checks
- ✅ Performance metrics
- ✅ Proactive alerts

### Analytics

- ✅ Usage patterns
- ✅ Performance trends
- ✅ Capacity planning
- ✅ Optimization opportunities

## 🚀 Next Steps

1. **Setup Monitoring Dashboard**
   - Grafana for visualization
   - Prometheus for metrics
   - Alert manager for notifications

2. **Implement Automated Cleanup**
   - Cron job for old log cleanup
   - Archive strategy
   - Retention policies

3. **Add More Analytics**
   - Cost analysis
   - Usage forecasting
   - Anomaly detection

4. **Enhance Security**
   - Token encryption at rest
   - Audit log encryption
   - Access control for logs
