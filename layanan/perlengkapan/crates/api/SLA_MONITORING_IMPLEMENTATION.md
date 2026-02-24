# SLA Monitoring with Notification Implementation

## Overview

This document describes the implementation of SLA (Service Level Agreement) monitoring with automatic escalation notifications for the SIMPEL workflow engine.

**Requirements:** REQ-W003, REQ-N008, NFR-M004

**Status:** ✅ Complete

## Architecture

### Components

1. **SLA Monitor** (`workflow/sla.rs`)
   - Detects SLA breaches for workflow entities
   - Sends escalation notifications via notification service
   - Records metrics for monitoring

2. **Notification Client** (`workflow/notifikasi_client.rs`)
   - gRPC client for notification service
   - Supports multiple notification types including SLA breach

3. **Prometheus Metrics** (`metrics.rs`)
   - `workflow_sla_breaches_total` - Counter for SLA breaches
   - `workflow_sla_breach_duration_minutes` - Histogram for breach duration
   - `workflow_escalations_total` - Counter for escalations

4. **Grafana Dashboard** (`grafana/sla_breach_dashboard.json`)
   - Real-time SLA breach monitoring
   - Breach rate and duration visualization
   - Escalation success rate tracking

## Features

### 1. SLA Breach Detection

The SLA monitor checks workflow entities against configured SLA limits:

```rust
// Check SLA for a specific entity
let breach = monitor.check_sla(entity_id).await?;

if let Some(breach_info) = breach {
    // SLA breached - escalate
    monitor.escalate_sla_breach(&breach_info).await?;
}
```

**SLA Limits (Kebutuhan BMN):**
- SUBMITTED: 2 days (2880 minutes)
- REVIEWED: 1 day (1440 minutes)
- APPROVED: 3 days (4320 minutes)

### 2. Automatic Escalation

When an SLA breach is detected, the system automatically:

1. **Sends escalation notification to approver** (Urgent priority)
   - Includes entity details
   - Shows SLA deadline and breach duration
   - Provides action link to approve/reject

2. **Sends informational notification to requester** (Normal priority)
   - Notifies that their request is delayed
   - Shows current status

3. **Logs escalation in workflow activity**
   - Creates audit trail
   - Records breach duration

4. **Records Prometheus metrics**
   - Increments breach counter
   - Records breach duration
   - Tracks escalation success/failure

### 3. Notification Types

#### SLA Breach Escalation (to Approver)

```json
{
  "type": "sla_breach_escalation",
  "entity_type": "kebutuhan_bmn",
  "entity_id": "uuid",
  "current_state": "SUBMITTED",
  "sla_deadline": "2026-02-15T10:00:00Z",
  "breach_duration_minutes": 120,
  "days_overdue": 0
}
```

#### SLA Breach Info (to Requester)

```json
{
  "type": "sla_breach_info",
  "entity_type": "kebutuhan_bmn",
  "entity_id": "uuid",
  "current_state": "SUBMITTED",
  "sla_deadline": "2026-02-15T10:00:00Z",
  "breach_duration_minutes": 120
}
```

### 4. Monitoring and Metrics

#### Prometheus Metrics

```promql
# SLA breach rate (last 5 minutes)
rate(workflow_sla_breaches_total[5m])

# Average breach duration by state
avg by (state) (workflow_sla_breach_duration_minutes)

# Escalation success rate
sum(workflow_escalations_total{status="success"}) / sum(workflow_escalations_total) * 100
```

#### Grafana Dashboard

The dashboard provides:
- **SLA Breach Rate Chart** - Real-time breach rate with alerting (threshold: 10%)
- **Breach Duration Heatmap** - Distribution of breach durations
- **Breaches by State Pie Chart** - Breakdown by workflow state
- **Escalation Success Rate** - Percentage of successful escalations
- **Total Escalations Counter** - Total number of escalations
- **Average Breach Duration Bar Chart** - Average duration by state
- **Breaches Timeline Table** - Detailed breach history

#### Alerts

**High SLA Breach Rate Alert:**
- Condition: Breach rate > 10% (0.1 breaches/second)
- Evaluation: Every 1 minute
- Action: Send alert notification

## Usage

### 1. Create SLA Monitor

```rust
use layanan_perlengkapan_api::workflow::{
    config::WorkflowConfig,
    sla::SlaMonitor,
    notifikasi_client::NotifikasiClient,
};

// Without notification service (logs only)
let monitor = SlaMonitor::new(config, db_pool);

// With notification service
let notifikasi_client = NotifikasiClient::new("http://localhost:50053").await?;
let monitor = SlaMonitor::with_notifikasi(config, db_pool, notifikasi_client);
```

### 2. Check SLA for Entity

```rust
// Check single entity
let breach = monitor.check_sla(entity_id).await?;

if let Some(breach_info) = breach {
    println!("SLA breached by {} minutes", breach_info.breach_duration_minutes);
}
```

### 3. Monitor All Entities

```rust
// Check all entities and escalate breaches
let breach_count = monitor.monitor_and_escalate().await?;
println!("Escalated {} SLA breaches", breach_count);
```

### 4. Get SLA Status

```rust
use layanan_perlengkapan_api::workflow::sla::SlaStatus;

let status = monitor.get_sla_status(entity_id).await?;

match status {
    SlaStatus::Normal { remaining_minutes } => {
        println!("SLA normal: {} minutes remaining", remaining_minutes);
    }
    SlaStatus::Warning { remaining_minutes } => {
        println!("SLA warning: {} minutes remaining", remaining_minutes);
    }
    SlaStatus::Critical { remaining_minutes } => {
        println!("SLA critical: {} minutes remaining", remaining_minutes);
    }
    SlaStatus::Breached { breach_duration_minutes } => {
        println!("SLA breached: {} minutes overdue", breach_duration_minutes);
    }
    SlaStatus::NoSla => {
        println!("No SLA defined for current state");
    }
}
```

## Scheduler Integration

The SLA monitor should be called periodically by a scheduler:

```rust
use tokio_cron_scheduler::{Job, JobScheduler};

// Create scheduler
let scheduler = JobScheduler::new().await?;

// Check SLA every hour
let sla_job = Job::new_async("0 0 * * * *", move |_uuid, _l| {
    Box::pin(async move {
        match monitor.monitor_and_escalate().await {
            Ok(count) => {
                tracing::info!("SLA monitoring completed: {} breaches escalated", count);
            }
            Err(e) => {
                tracing::error!("SLA monitoring failed: {}", e);
            }
        }
    })
})?;

scheduler.add(sla_job).await?;
scheduler.start().await?;
```

## Testing

### Unit Tests

```bash
# Run SLA monitoring tests
cargo test --test sla_notification_integration_tests

# Run specific test
cargo test --test sla_notification_integration_tests test_sla_breach_detection
```

### Integration Tests

The test suite includes:
- ✅ SLA breach detection
- ✅ SLA no breach (within limit)
- ✅ Check all SLAs
- ✅ Escalation with notification
- ✅ Monitor and escalate
- ✅ SLA status (normal, warning, critical, breached)
- ✅ Metrics recording

**Note:** Integration tests require:
- PostgreSQL database with test schema
- Notification service running (for full integration tests)

### Manual Testing

1. **Create test entity with SLA breach:**
   ```sql
   -- Insert kebutuhan 3 days ago (SLA is 2 days)
   INSERT INTO perlengkapan.kebutuhan_bmn (id, status, created_at, ...)
   VALUES (uuid_generate_v4(), 'SUBMITTED', NOW() - INTERVAL '3 days', ...);
   ```

2. **Check SLA:**
   ```bash
   curl http://localhost:3020/api/v1/workflow/sla/check/{entity_id}
   ```

3. **View metrics:**
   ```bash
   curl http://localhost:3020/metrics | grep workflow_sla
   ```

4. **View Grafana dashboard:**
   - Import `grafana/sla_breach_dashboard.json`
   - Navigate to dashboard
   - Observe real-time SLA metrics

## Configuration

### Workflow SLA Limits

SLA limits are configured in `workflow/config.rs`:

```rust
impl WorkflowConfig {
    pub fn default_kebutuhan_bmn() -> Self {
        let mut sla_minutes = HashMap::new();
        sla_minutes.insert("SUBMITTED".to_string(), 2880);  // 2 days
        sla_minutes.insert("REVIEWED".to_string(), 1440);   // 1 day
        sla_minutes.insert("APPROVED".to_string(), 4320);   // 3 days

        Self {
            sla_minutes,
            // ...
        }
    }
}
```

### Notification Service Endpoint

Configure in environment variables:

```bash
NOTIFIKASI_GRPC_URL=http://localhost:50053
```

## Troubleshooting

### SLA Breaches Not Detected

1. **Check workflow activity timestamps:**
   ```sql
   SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
   WHERE pengajuan_id = '{entity_id}'
   ORDER BY created_at DESC;
   ```

2. **Verify SLA configuration:**
   ```rust
   let sla_limit = config.get_sla_minutes("SUBMITTED");
   println!("SLA limit: {:?}", sla_limit);
   ```

### Notifications Not Sent

1. **Check notification service is running:**
   ```bash
   curl http://localhost:50053/health
   ```

2. **Verify notification client connection:**
   ```rust
   let client = NotifikasiClient::new("http://localhost:50053").await?;
   // Should not error
   ```

3. **Check notification service logs:**
   ```bash
   docker logs layanan-notifikasi
   ```

### Metrics Not Recorded

1. **Verify Prometheus is scraping:**
   ```bash
   curl http://localhost:3020/metrics
   ```

2. **Check metric registration:**
   ```rust
   let metric = metrics::workflow_sla_breaches_total();
   println!("Metric registered: {:?}", metric);
   ```

## Performance Considerations

1. **Database Queries:**
   - SLA checks use indexed queries (entity_id, status, created_at)
   - Batch checking uses LATERAL join for efficiency

2. **Notification Delivery:**
   - Notifications are sent asynchronously
   - Failed notifications don't block SLA monitoring
   - Retry logic handled by notification service

3. **Metrics Recording:**
   - Metrics use atomic counters (no locks)
   - Minimal overhead per escalation

4. **Scheduler Frequency:**
   - Recommended: Every hour
   - Can be increased for critical workflows
   - Balance between responsiveness and load

## Future Enhancements

1. **Configurable SLA Limits:**
   - Per-satker SLA configuration
   - Dynamic SLA adjustment based on workload

2. **Escalation Hierarchy:**
   - Multi-level escalation (supervisor → manager → director)
   - Automatic escalation after repeated breaches

3. **SLA Prediction:**
   - ML-based prediction of SLA breaches
   - Proactive notifications before breach

4. **Custom Notification Templates:**
   - Configurable notification content
   - Multi-language support

## References

- **Requirements:** `.kiro/specs/simpel-completion/requirements.md`
- **Design:** `.kiro/specs/simpel-completion/design.md`
- **Tasks:** `.kiro/specs/simpel-completion/tasks.md` (Task 28.5.8)
- **Workflow Engine:** `workflow/engine.rs`
- **Notification Service:** `crates/notifikasi/`

---

**Last Updated:** February 11, 2026
**Author:** SIMPEL Development Team
**Status:** Production Ready
