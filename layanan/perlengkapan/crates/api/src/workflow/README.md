# Workflow Engine Module

This module provides a comprehensive workflow engine for managing approval processes in the SIMPEL system.

## Components

### Core Engine (`engine.rs`)
- **WorkflowEngine**: Core state machine for workflow transitions
- Validates state transitions based on workflow configuration
- Integrates with Authenc for role-based access control
- Integrates with Dokumen service for document generation
- Integrates with Notifikasi service for notifications
- Records metrics for monitoring

### Configuration (`config.rs`)
- **WorkflowConfig**: Defines workflow states, transitions, and SLA limits
- Pre-configured workflows for:
  - Kebutuhan BMN (BMN Requirements)
  - Pemakaian BMN (BMN Usage Permits)
  - Penghapusan BMN (BMN Deletion)

### SLA Monitoring (`sla.rs`)
- **SlaMonitor**: Monitors Service Level Agreement compliance
- Detects SLA breaches based on configured time limits
- Sends escalation notifications when SLA is breached
- Records SLA metrics for monitoring

### SLA Escalation Scheduler (`sla_scheduler.rs`)
- **SlaEscalationScheduler**: Automated periodic SLA breach detection and escalation
- Runs on a configurable cron schedule (default: every 15 minutes)
- Checks all workflow types for SLA breaches
- Automatically escalates breached workflows
- Records metrics for monitoring

### Delegation (`delegation.rs`)
- Workflow delegation support
- Allows users to delegate approval authority

### Parallel Approval (`parallel.rs`)
- Support for parallel approval workflows
- Multiple approvers can approve simultaneously

### Monitoring (`monitoring.rs`)
- Workflow metrics and monitoring
- Integration with Prometheus

### Handlers (`handlers.rs`)
- REST API endpoints for workflow operations
- Transition endpoints
- Status query endpoints

## Usage

### Starting the SLA Escalation Scheduler

The scheduler is automatically started in `main.rs`:

```rust
use layanan_perlengkapan_api::workflow::SlaEscalationScheduler;

// Create scheduler
let sla_scheduler = SlaEscalationScheduler::new(db_pool.clone());

// Optional: Add notifikasi client for sending notifications
// let sla_scheduler = sla_scheduler.with_notifikasi_client(notifikasi_client);

// Start the scheduler
sla_scheduler.start()?;
```

### Configuration

The scheduler can be configured via environment variables:

- `SLA_CHECK_INTERVAL_CRON`: Cron expression for check interval (default: `"0 */15 * * * *"` - every 15 minutes)
- `SLA_SCHEDULER_ENABLED`: Enable/disable the scheduler (default: `true`)

Example `.env`:
```bash
# Run SLA checks every 30 minutes
SLA_CHECK_INTERVAL_CRON="0 */30 * * * *"

# Enable the scheduler
SLA_SCHEDULER_ENABLED=true
```

### Cron Expression Format

The cron expression uses 6 fields:
```
┌───────────── second (0-59)
│ ┌───────────── minute (0-59)
│ │ ┌───────────── hour (0-23)
│ │ │ ┌───────────── day of month (1-31)
│ │ │ │ ┌───────────── month (1-12)
│ │ │ │ │ ┌───────────── day of week (0-6) (Sunday to Saturday)
│ │ │ │ │ │
│ │ │ │ │ │
* * * * * *
```

Examples:
- `"0 */15 * * * *"` - Every 15 minutes
- `"0 0 * * * *"` - Every hour
- `"0 0 */6 * * *"` - Every 6 hours
- `"0 0 0 * * *"` - Daily at midnight

### Manual SLA Check

You can also manually check SLA for a specific entity:

```rust
use layanan_perlengkapan_api::workflow::{SlaMonitor, WorkflowConfig};

let config = WorkflowConfig::default_kebutuhan_bmn();
let monitor = SlaMonitor::new(config, db_pool.clone());

// Check SLA for a specific entity
if let Some(breach) = monitor.check_sla(entity_id).await? {
    println!("SLA breach detected: {} minutes overdue", breach.breach_duration_minutes);

    // Escalate the breach
    monitor.escalate_sla_breach(&breach).await?;
}
```

## Workflow Types

### Kebutuhan BMN (BMN Requirements)
States: DRAFT → INPUT_BARANG → SUBMITTED → REVIEWED → APPROVED/REJECTED

SLA Limits:
- SUBMITTED: 2 days (2880 minutes)
- REVIEWED: 3 days (4320 minutes)

### Pemakaian BMN (BMN Usage Permits)
States: DRAFT → SUBMITTED → APPROVED/REJECTED

SLA Limits:
- SUBMITTED: 1 day (1440 minutes)

### Penghapusan BMN (BMN Deletion)
States: DRAFT → SUBMITTED → REVIEWED → APPROVED/REJECTED

SLA Limits:
- SUBMITTED: 3 days (4320 minutes)
- REVIEWED: 5 days (7200 minutes)

## Metrics

The workflow engine and SLA scheduler expose the following Prometheus metrics:

### Workflow Metrics
- `workflow_transitions_total{entity_type, from_state, to_state, status}` - Total workflow transitions
- `workflow_transition_duration_seconds{entity_type, from_state, to_state}` - Transition duration
- `workflow_items_by_state{entity_type, state}` - Current items in each state

### SLA Metrics
- `workflow_sla_breaches_total{entity_type, state}` - Total SLA breaches
- `workflow_sla_breach_duration_minutes{entity_type, state}` - Breach duration
- `workflow_escalations_total{entity_type, state, status}` - Total escalations
- `workflow_sla_check_duration_seconds{workflow_type}` - SLA check duration
- `workflow_sla_check_total{status}` - Total SLA checks performed

## Integration

### Authenc Integration
The workflow engine integrates with Authenc for:
- User role validation
- Approver identification
- Permission checks

### Dokumen Integration
The workflow engine integrates with Dokumen service for:
- Automatic document generation on approval
- SK (Surat Keputusan) generation
- Permit document generation

### Notifikasi Integration
The workflow engine integrates with Notifikasi service for:
- Approval request notifications
- Approval completion notifications
- Rejection notifications
- SLA breach escalation notifications
- Revision request notifications

## Requirements Mapping

- **REQ-W001**: Workflow state machine implementation
- **REQ-W002**: Role-based approval validation
- **REQ-W003**: SLA monitoring and escalation
- **REQ-W004**: State transition validation
- **REQ-W005**: Workflow history tracking
- **REQ-W011**: Document generation integration
- **REQ-N001**: Notification integration
- **REQ-N003**: Multi-channel notifications
- **REQ-N005**: Workflow event notifications
- **REQ-N008**: SLA breach notifications
- **NFR-M004**: Metrics and monitoring

## Testing

Run the workflow tests:

```bash
# Unit tests
cargo test -p layanan-perlengkapan-api --lib workflow

# Integration tests
cargo test -p layanan-perlengkapan-api --test sla_scheduler_test
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    SLA Escalation Scheduler                  │
│                  (Runs every 15 minutes)                     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
         ┌───────────────────────────────────┐
         │      Check SLA for all workflows   │
         │  - Kebutuhan BMN                   │
         │  - Pemakaian BMN                   │
         │  - Penghapusan BMN                 │
         └───────────────┬───────────────────┘
                         │
                         ▼
         ┌───────────────────────────────────┐
         │     SLA Monitor (per workflow)     │
         │  - Query database for entities     │
         │  - Calculate elapsed time          │
         │  - Compare with SLA limits         │
         └───────────────┬───────────────────┘
                         │
                         ▼
         ┌───────────────────────────────────┐
         │    Detect SLA Breaches             │
         │  - Identify overdue entities       │
         │  - Calculate breach duration       │
         └───────────────┬───────────────────┘
                         │
                         ▼
         ┌───────────────────────────────────┐
         │    Escalate Breaches               │
         │  - Send notifications to approver  │
         │  - Send info to requester          │
         │  - Log escalation activity         │
         │  - Record metrics                  │
         └───────────────────────────────────┘
```

## Future Enhancements

- [ ] Configurable escalation levels (L1, L2, L3)
- [ ] Automatic workflow reassignment on escalation
- [ ] SLA warning notifications (before breach)
- [ ] Custom SLA limits per satker
- [ ] Workflow analytics dashboard
- [ ] Workflow definition versioning
- [ ] Conditional branching support
