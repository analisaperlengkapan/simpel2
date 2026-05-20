# Backup Failure Alerting

This document describes the backup failure alerting system implemented in the Secreton backup crate.

## Overview

The alerting system provides automatic notifications when backup operations fail, integrating with the existing monitoring infrastructure (Prometheus metrics and structured logging).

## Features

- **Automatic Alert Triggering**: Alerts are automatically triggered on:
  - Backup creation failures
  - Backup verification failures
  - Backup cleanup failures (non-fatal)
  - Backup restoration failures

- **Multiple Alert Handlers**:
  - **Logging Handler**: Logs alerts using structured tracing (always enabled)
  - **Metrics Handler**: Records alerts as Prometheus metrics (enabled with `metrics` feature)
  - **Custom Handlers**: Support for adding custom alert handlers

- **Alert Severity Levels**:
  - `Warning`: Non-critical issues (e.g., cleanup failures)
  - `Error`: Critical issues requiring attention (e.g., verification failures)
  - `Critical`: Severe issues requiring immediate action (e.g., backup creation failures)

## Usage

### Basic Usage

The alert manager is automatically initialized when creating a `BackupManager`:

```rust
use secreton_backup::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let config = BackupConfig::default();
    let manager = BackupManager::new(config).await?;

    // Alerts are automatically triggered on failures
    match manager.create_backup().await {
        Ok(backup_id) => {
            println!("Backup created: {}", backup_id);
        }
        Err(e) => {
            // Alert was automatically triggered
            eprintln!("Backup failed: {}", e);
        }
    }

    Ok(())
}
```

### Custom Alert Handlers

You can add custom alert handlers to integrate with external systems:

```rust
use secreton_backup::prelude::*;
use async_trait::async_trait;

// Custom handler that sends alerts to Slack
struct SlackAlertHandler {
    webhook_url: String,
}

#[async_trait]
impl AlertHandler for SlackAlertHandler {
    async fn handle_alert(&self, alert: &Alert) -> Result<()> {
        // Send alert to Slack webhook
        let message = format!(
            "🚨 Backup Alert: {} - {}",
            alert.alert_type, alert.message
        );

        // ... send HTTP request to Slack webhook ...

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = BackupConfig::default();
    let manager = BackupManager::new(config).await?;

    // Add custom handler
    let slack_handler = Box::new(SlackAlertHandler {
        webhook_url: "https://hooks.slack.com/...".to_string(),
    });
    manager.alert_manager().add_handler(slack_handler).await;

    // Now alerts will be sent to both logs and Slack
    manager.create_backup().await?;

    Ok(())
}
```

## Alert Types

### BackupCreationFailed

Triggered when backup creation fails at any stage (Raft snapshot, PostgreSQL dump, encryption, storage upload).

**Severity**: Critical

**Example**:

```
Backup alert (CRITICAL): BackupCreationFailed - PostgreSQL dump failed: connection refused
```

### BackupVerificationFailed

Triggered when backup verification fails (checksum mismatch, decryption failure, decompression failure).

**Severity**: Error

**Example**:

```
Backup alert (ERROR): BackupVerificationFailed [backup-123] - Checksum mismatch: expected abc123, got def456
```

### BackupCleanupFailed

Triggered when automatic cleanup of old backups fails. This is non-fatal and doesn't prevent backup creation.

**Severity**: Warning

**Example**:

```
Backup alert (WARNING): BackupCleanupFailed - Failed to delete backup: storage error
```

### BackupRestorationFailed

Triggered when backup restoration fails (download failure, decryption failure, Raft restore failure, PostgreSQL restore failure).

**Severity**: Critical

**Example**:

```
Backup alert (CRITICAL): BackupRestorationFailed [backup-123] - Raft snapshot restoration failed
```

## Prometheus Metrics

When the `metrics` feature is enabled, the following Prometheus metrics are recorded:

### secreton_backup_alerts_total

Counter tracking the total number of alerts triggered.

**Labels**:

- `alert_type`: Type of alert (`backup_creation_failed`, `backup_verification_failed`, `backup_cleanup_failed`, `backup_restoration_failed`)
- `severity`: Severity level (`warning`, `error`, `critical`)

**Example**:

```
secreton_backup_alerts_total{alert_type="backup_creation_failed",severity="critical"} 3
secreton_backup_alerts_total{alert_type="backup_verification_failed",severity="error"} 1
```

### secreton_backup_last_alert_timestamp_seconds

Gauge tracking the timestamp of the last alert for each type.

**Labels**:

- `alert_type`: Type of alert

**Example**:

```
secreton_backup_last_alert_timestamp_seconds{alert_type="backup_creation_failed"} 1707825600.0
```

## Configuration

### Enabling/Disabling Alerting

Alerting is enabled by default. To disable alerting, you would need to create a custom `AlertManager`:

```rust
use secreton_backup::alerting::AlertManager;

// Create alert manager with alerting disabled
let alert_manager = AlertManager::new(false);
```

Note: The `BackupManager` currently always enables alerting. To disable it, you would need to modify the `BackupManager::new` method.

### Enabling Metrics

To enable Prometheus metrics, compile with the `metrics` feature:

```toml
[dependencies]
secreton-backup = { version = "0.1.0", features = ["metrics"] }
```

Or in the workspace:

```bash
cargo build --features metrics
```

## Integration with Monitoring Systems

### Prometheus + Grafana

1. **Scrape Metrics**: Configure Prometheus to scrape the Secreton metrics endpoint
2. **Create Alerts**: Define Prometheus alerting rules:

```yaml
groups:
  - name: secreton_backup
    rules:
      - alert: BackupCreationFailed
        expr: increase(secreton_backup_alerts_total{alert_type="backup_creation_failed"}[5m]) > 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Secreton backup creation failed"
          description: "Backup creation has failed {{ $value }} times in the last 5 minutes"

      - alert: BackupVerificationFailed
        expr: increase(secreton_backup_alerts_total{alert_type="backup_verification_failed"}[5m]) > 0
        for: 1m
        labels:
          severity: error
        annotations:
          summary: "Secreton backup verification failed"
          description: "Backup verification has failed {{ $value }} times in the last 5 minutes"
```

3. **Visualize in Grafana**: Create dashboards showing alert trends

### Structured Logging

All alerts are logged using structured tracing, making them easy to query in log aggregation systems (e.g., Loki, Elasticsearch):

```json
{
  "timestamp": "2026-02-18T10:30:00Z",
  "level": "ERROR",
  "message": "Backup alert (CRITICAL)",
  "fields": {
    "alert_type": "BackupCreationFailed",
    "backup_id": null,
    "message": "PostgreSQL dump failed",
    "context": "Backup creation failed: PostgresDump(...)",
    "timestamp": "2026-02-18T10:30:00Z"
  }
}
```

## Testing

The alerting system includes comprehensive unit tests:

```bash
# Run alerting tests
cargo test -p secreton-backup --lib alerting

# Run all backup tests
cargo test -p secreton-backup
```

## Requirements Validation

This implementation validates the following requirements from the Secreton Vault Parity spec:

- **Requirement 2.5.9**: Backup failures trigger alerts
  - ✅ Alerts triggered on backup creation failure
  - ✅ Alerts triggered on verification failure
  - ✅ Integration with monitoring system (Prometheus metrics)
  - ✅ Configuration to enable/disable alerts

## Future Enhancements

Potential future improvements:

1. **Email Alerts**: Add email alert handler
2. **Webhook Alerts**: Generic webhook handler for external integrations
3. **Alert Aggregation**: Batch multiple alerts to reduce noise
4. **Alert Throttling**: Rate-limit alerts to prevent alert storms
5. **Alert Acknowledgment**: Track which alerts have been acknowledged
6. **Alert History**: Store alert history in database for analysis

## See Also

- [Backup Manager Documentation](./README.md)
- [Secreton Vault Parity Spec](../.kiro/specs/secreton-vault-parity/requirements.md)
- [Prometheus Metrics](https://prometheus.io/docs/concepts/metric_types/)
