# Secreton Backup & Restore Guide

**Version:** 2.0
**Last Updated:** February 18, 2026
**Status:** Production Ready

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [S3-Compatible Storage Setup](#s3-compatible-storage-setup)
4. [Backup Schedule Configuration](#backup-schedule-configuration)
5. [Restoration Procedures](#restoration-procedures)
6. [Disaster Recovery Playbook](#disaster-recovery-playbook)
7. [CLI Reference](#cli-reference)
8. [Troubleshooting](#troubleshooting)
9. [Security Best Practices](#security-best-practices)
10. [Monitoring & Alerting](#monitoring--alerting)

---

## Overview

Secreton provides comprehensive automated backup and restore capabilities to protect your secrets, configuration, and audit logs from data loss. The backup system supports:

- **Automated Scheduled Backups**: Cron-based scheduling for hands-free operation
- **Multiple Storage Backends**: Local filesystem, S3-compatible storage (AWS S3, MinIO, Wasabi, etc.)
- **Encryption at Rest**: ChaCha20-Poly1305 encryption with separate backup keys
- **Compression**: Gzip compression to reduce storage costs (50-70% reduction)
- **Point-in-Time Recovery**: Restore to any previous backup
- **Automatic Verification**: Checksum validation ensures backup integrity
- **Retention Policies**: Automatic cleanup of old backups
- **Complete Data Protection**: Backs up Raft state, PostgreSQL data, and metadata

### What Gets Backed Up

Each backup includes:
1. **Raft Consensus State**: Complete cluster state for HA deployments
2. **PostgreSQL Database**: All secrets, policies, leases, audit logs
3. **Metadata**: Backup information, checksums, encryption details

### Backup Format

Backups are stored as encrypted, compressed archives with the following structure:

```
backup-YYYYMMDD-HHMMSS.enc
├── Metadata (JSON)
│   ├── backup_id (UUID)
│   ├── timestamp
│   ├── version info
│   ├── checksums
│   └── encryption details
├── Raft Snapshot (compressed)
└── PostgreSQL Dump (compressed)
```

---

## Quick Start

### Prerequisites

- Secreton installed and running
- `secreton-cli` installed
- S3-compatible storage (optional, for remote backups)
- Sufficient disk space (estimate 2-3x your database size)

### Create Your First Backup

```bash
# Manual backup to local filesystem
secreton-cli backup create \
  --output /var/backups/secreton/backup-$(date +%Y%m%d).enc \
  --password "your-secure-password" \
  --include-audit true \
  --compression 6

# Verify the backup
secreton-cli backup verify \
  --file /var/backups/secreton/backup-20260218.enc \
  --password "your-secure-password"
```

### Enable Automated Backups

Edit your Secreton configuration (`secreton.toml`):

```toml
[backup]
enabled = true
schedule = "0 2 * * *"  # Daily at 2 AM
retention_days = 30
compression_enabled = true
compression_level = 6
verify_after_backup = true

[backup.storage]
type = "local"
path = "/var/backups/secreton"

# Or use S3
# type = "s3"
# bucket = "secreton-backups"
# region = "us-east-1"
# prefix = "production/"
```

Restart Secreton to apply changes.

---
## S3-Compatible Storage Setup

### AWS S3

#### 1. Create S3 Bucket

```bash
aws s3 mb s3://secreton-backups --region us-east-1
```

#### 2. Create IAM Policy

Create `secreton-backup-policy.json`:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:PutObject",
        "s3:GetObject",
        "s3:DeleteObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::secreton-backups",
        "arn:aws:s3:::secreton-backups/*"
      ]
    }
  ]
}
```

Apply the policy:

```bash
aws iam create-policy \
  --policy-name SecretonBackupPolicy \
  --policy-document file://secreton-backup-policy.json
```

#### 3. Attach Policy to IAM Role/User

```bash
# For EC2 instance role
aws iam attach-role-policy \
  --role-name secreton-instance-role \
  --policy-arn arn:aws:iam::ACCOUNT_ID:policy/SecretonBackupPolicy

# Or for IAM user
aws iam attach-user-policy \
  --user-name secreton-backup-user \
  --policy-arn arn:aws:iam::ACCOUNT_ID:policy/SecretonBackupPolicy
```

#### 4. Configure Secreton

```toml
[backup.storage]
type = "s3"
bucket = "secreton-backups"
region = "us-east-1"
prefix = "production/"
# Use IAM role (recommended) or access keys
# access_key_id = "AKIAIOSFODNN7EXAMPLE"
# secret_access_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
```

### MinIO (Self-Hosted S3-Compatible)

#### 1. Install MinIO

```bash
# Docker
docker run -d \
  -p 9000:9000 \
  -p 9001:9001 \
  --name minio \
  -v /data/minio:/data \
  -e "MINIO_ROOT_USER=minioadmin" \
  -e "MINIO_ROOT_PASSWORD=minioadmin" \
  quay.io/minio/minio server /data --console-address ":9001"
```

#### 2. Create Bucket

```bash
# Using mc (MinIO Client)
mc alias set myminio http://localhost:9000 minioadmin minioadmin
mc mb myminio/secreton-backups
```

#### 3. Configure Secreton

```toml
[backup.storage]
type = "s3"
bucket = "secreton-backups"
region = "us-east-1"  # Required but not used by MinIO
endpoint = "http://localhost:9000"
access_key_id = "minioadmin"
secret_access_key = "minioadmin"
force_path_style = true  # Required for MinIO
prefix = "backups/"
```

### Wasabi Cloud Storage

```toml
[backup.storage]
type = "s3"
bucket = "secreton-backups"
region = "us-east-1"
endpoint = "https://s3.us-east-1.wasabisys.com"
access_key_id = "YOUR_WASABI_ACCESS_KEY"
secret_access_key = "YOUR_WASABI_SECRET_KEY"
prefix = "production/"
```

### DigitalOcean Spaces

```toml
[backup.storage]
type = "s3"
bucket = "secreton-backups"
region = "nyc3"
endpoint = "https://nyc3.digitaloceanspaces.com"
access_key_id = "YOUR_SPACES_ACCESS_KEY"
secret_access_key = "YOUR_SPACES_SECRET_KEY"
prefix = "production/"
```

---
## Backup Schedule Configuration

### Cron Expression Format

Secreton uses standard cron syntax for backup scheduling:

```
┌───────────── minute (0 - 59)
│ ┌───────────── hour (0 - 23)
│ │ ┌───────────── day of month (1 - 31)
│ │ │ ┌───────────── month (1 - 12)
│ │ │ │ ┌───────────── day of week (0 - 6) (Sunday to Saturday)
│ │ │ │ │
* * * * *
```

### Common Schedules

| Schedule | Cron Expression | Description |
|----------|----------------|-------------|
| Every hour | `0 * * * *` | Hourly backups at minute 0 |
| Every 6 hours | `0 */6 * * *` | 4 times per day (00:00, 06:00, 12:00, 18:00) |
| Daily at 2 AM | `0 2 * * *` | Once per day at 2:00 AM |
| Daily at midnight | `0 0 * * *` | Once per day at 00:00 |
| Twice daily | `0 2,14 * * *` | At 2:00 AM and 2:00 PM |
| Weekly (Sunday) | `0 0 * * 0` | Every Sunday at midnight |
| Monthly (1st) | `0 0 1 * *` | First day of month at midnight |
| Weekdays only | `0 2 * * 1-5` | Monday-Friday at 2:00 AM |

### Recommended Schedules by Environment

#### Production
```toml
[backup]
schedule = "0 */6 * * *"  # Every 6 hours
retention_days = 30        # Keep 30 days (120 backups)
```

#### Staging
```toml
[backup]
schedule = "0 2 * * *"     # Daily at 2 AM
retention_days = 14        # Keep 14 days
```

#### Development
```toml
[backup]
schedule = "0 0 * * 0"     # Weekly on Sunday
retention_days = 7         # Keep 7 days
```

### Configuration File

Edit `/etc/secreton/secreton.toml`:

```toml
[backup]
# Enable automated backups
enabled = true

# Cron schedule (daily at 2 AM)
schedule = "0 2 * * *"

# Retention policy (days)
retention_days = 30

# Storage configuration
storage_type = "s3"

[backup.storage]
type = "s3"
bucket = "secreton-backups"
region = "us-east-1"
prefix = "production/"

# Encryption (32-byte key in hex)
encryption_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

# Compression
compression_enabled = true
compression_level = 6  # 0-9, where 9 is maximum

# Verification
verify_after_backup = true
```

### Environment Variables

Override configuration with environment variables:

```bash
export SECRETON_BACKUP_ENABLED=true
export SECRETON_BACKUP_SCHEDULE="0 2 * * *"
export SECRETON_BACKUP_RETENTION_DAYS=30
export SECRETON_BACKUP_STORAGE_TYPE=s3
export SECRETON_BACKUP_S3_BUCKET=secreton-backups
export SECRETON_BACKUP_S3_REGION=us-east-1
```

---
## Restoration Procedures

### Pre-Restoration Checklist

Before restoring a backup, ensure:

- [ ] Secreton service is stopped (or in maintenance mode)
- [ ] You have the backup encryption password/key
- [ ] You have verified the backup integrity
- [ ] You have sufficient disk space
- [ ] You have a current backup (in case restore fails)
- [ ] All team members are notified
- [ ] You have tested the restore procedure in a non-production environment

### Basic Restore

#### 1. Stop Secreton Service

```bash
# Kubernetes
kubectl scale deployment secreton --replicas=0 -n secreton-system

# Systemd
sudo systemctl stop secreton

# Docker
docker stop secreton
```

#### 2. Verify Backup

```bash
secreton-cli backup verify \
  --file /path/to/backup-20260218.enc \
  --password "your-backup-password" \
  --verbose
```

#### 3. Restore from Backup

```bash
secreton-cli backup restore \
  --file /path/to/backup-20260218.enc \
  --password "your-backup-password"
```

#### 4. Start Secreton Service

```bash
# Kubernetes
kubectl scale deployment secreton --replicas=3 -n secreton-system

# Systemd
sudo systemctl start secreton

# Docker
docker start secreton
```

#### 5. Verify Restoration

```bash
# Check service health
secreton-cli status

# List secrets
secreton-cli kv list secret/

# Test secret retrieval
secreton-cli kv get secret/test
```

### Point-in-Time Restore

Restore to a specific point in time:

```bash
secreton-cli backup restore \
  --file /path/to/backup-20260218.enc \
  --password "your-backup-password" \
  --point-in-time "2026-02-18T10:00:00Z"
```

This will restore only secrets and audit logs created before the specified timestamp.

### Partial Restore

#### Restore Secrets Only

```bash
secreton-cli backup restore \
  --file /path/to/backup.enc \
  --password "your-password" \
  --secrets-only
```

#### Restore Audit Logs Only

```bash
secreton-cli backup restore \
  --file /path/to/backup.enc \
  --password "your-password" \
  --audit-only
```

### Restore to Different Namespace

```bash
secreton-cli backup restore \
  --file /path/to/backup.enc \
  --password "your-password" \
  --target-namespace "recovery"
```

### Dry Run (Test Restore)

Test the restore process without making changes:

```bash
secreton-cli backup restore \
  --file /path/to/backup.enc \
  --password "your-password" \
  --dry-run
```

### Force Overwrite

Overwrite existing secrets during restore:

```bash
secreton-cli backup restore \
  --file /path/to/backup.enc \
  --password "your-password" \
  --force
```

**⚠️ Warning**: This will overwrite existing secrets. Use with caution!

---
## Disaster Recovery Playbook

### Scenario 1: Complete Data Loss

**Situation**: Primary Secreton cluster is completely lost (hardware failure, data center outage, ransomware).

**Recovery Steps**:

1. **Assess the Situation** (5 minutes)
   ```bash
   # Check if any nodes are recoverable
   kubectl get pods -n secreton-system

   # Check backup availability
   aws s3 ls s3://secreton-backups/production/
   ```

2. **Deploy New Secreton Cluster** (15 minutes)
   ```bash
   # Deploy fresh Secreton instance via Helm
   ./infra/helm/deploy.sh production install

   # Wait for pods to be ready
   kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=secreton \
     -n simpelv2-production --timeout=300s
   ```

3. **Download Latest Backup** (5 minutes)
   ```bash
   # Find latest backup
   LATEST_BACKUP=$(aws s3 ls s3://secreton-backups/production/ | sort | tail -n 1 | awk '{print $4}')

   # Download backup
   aws s3 cp s3://secreton-backups/production/$LATEST_BACKUP /tmp/restore.enc
   ```

4. **Verify Backup Integrity** (2 minutes)
   ```bash
   secreton-cli backup verify \
     --file /tmp/restore.enc \
     --password "$BACKUP_PASSWORD"
   ```

5. **Restore Data** (10-30 minutes, depending on size)
   ```bash
   secreton-cli backup restore \
     --file /tmp/restore.enc \
     --password "$BACKUP_PASSWORD" \
     --force
   ```

6. **Verify Restoration** (5 minutes)
   ```bash
   # Check critical secrets
   secreton-cli kv get secret/database/password
   secreton-cli kv get secret/api/keys

   # Verify audit logs
   secreton-cli audit list --limit 10

   # Check service health
   secreton-cli status
   ```

7. **Notify Stakeholders** (5 minutes)
   - Send recovery completion notification
   - Document incident and recovery time
   - Schedule post-mortem meeting

**Total Recovery Time Objective (RTO)**: ~45-75 minutes

**Recovery Point Objective (RPO)**: Last backup (typically < 6 hours)

### Scenario 2: Corrupted Database

**Situation**: PostgreSQL database is corrupted but Raft state is intact.

**Recovery Steps**:

1. **Stop Secreton** (1 minute)
   ```bash
   kubectl scale deployment secreton --replicas=0 -n secreton-system
   ```

2. **Backup Current State** (5 minutes)
   ```bash
   # Create emergency backup of current state
   secreton-cli backup create \
     --output /tmp/emergency-backup-$(date +%Y%m%d-%H%M%S).enc \
     --password "$BACKUP_PASSWORD"
   ```

3. **Drop and Recreate Database** (2 minutes)
   ```bash
   psql -h postgres-host -U postgres -c "DROP DATABASE secreton;"
   psql -h postgres-host -U postgres -c "CREATE DATABASE secreton;"
   ```

4. **Restore from Backup** (10 minutes)
   ```bash
   secreton-cli backup restore \
     --file /path/to/latest-backup.enc \
     --password "$BACKUP_PASSWORD"
   ```

5. **Restart Secreton** (2 minutes)
   ```bash
   kubectl scale deployment secreton --replicas=3 -n secreton-system
   ```

**Total RTO**: ~20 minutes

### Scenario 3: Accidental Secret Deletion

**Situation**: Critical secrets were accidentally deleted.

**Recovery Steps**:

1. **Identify Deletion Time** (2 minutes)
   ```bash
   # Check audit logs
   secreton-cli audit list --operation delete --limit 50
   ```

2. **Find Appropriate Backup** (3 minutes)
   ```bash
   # List backups before deletion
   secreton-cli backup list --directory /var/backups/secreton
   ```

3. **Restore to Different Namespace** (5 minutes)
   ```bash
   secreton-cli backup restore \
     --file /path/to/backup-before-deletion.enc \
     --password "$BACKUP_PASSWORD" \
     --target-namespace "recovery" \
     --secrets-only
   ```

4. **Copy Secrets Back** (5 minutes)
   ```bash
   # Copy specific secrets from recovery namespace
   secreton-cli kv get recovery/secret/deleted-secret | \
     secreton-cli kv put secret/deleted-secret -
   ```

5. **Verify and Cleanup** (2 minutes)
   ```bash
   # Verify secret is restored
   secreton-cli kv get secret/deleted-secret

   # Delete recovery namespace
   secreton-cli namespace delete recovery
   ```

**Total RTO**: ~17 minutes

### Scenario 4: Multi-Region Failover

**Situation**: Primary region is unavailable, need to failover to DR region.

**Recovery Steps**:

1. **Activate DR Region** (5 minutes)
   ```bash
   # Scale up DR cluster
   kubectl scale deployment secreton --replicas=3 -n secreton-system --context=dr-region
   ```

2. **Restore Latest Backup** (15 minutes)
   ```bash
   # Download latest backup from S3
   aws s3 cp s3://secreton-backups/production/latest.enc /tmp/restore.enc

   # Restore to DR cluster
   secreton-cli backup restore \
     --file /tmp/restore.enc \
     --password "$BACKUP_PASSWORD" \
     --server-url https://secreton-dr.kejaksaan.go.id
   ```

3. **Update DNS** (5 minutes)
   ```bash
   # Point secreton.kejaksaan.go.id to DR region
   aws route53 change-resource-record-sets \
     --hosted-zone-id Z1234567890ABC \
     --change-batch file://dns-failover.json
   ```

4. **Verify Services** (5 minutes)
   ```bash
   # Test from multiple locations
   curl https://secreton.kejaksaan.go.id/v1/sys/health

   # Verify critical applications can access secrets
   kubectl logs -n production app-pod | grep "secret retrieved"
   ```

5. **Monitor and Document** (Ongoing)
   - Monitor DR cluster performance
   - Document failover time and issues
   - Plan failback procedure

**Total RTO**: ~30 minutes

---

## CLI Reference

### backup create

Create a new backup.

```bash
secreton-cli backup create [OPTIONS]
```

**Options**:
- `--output, -o <PATH>`: Output file path (required)
- `--password, -p <PASSWORD>`: Encryption password (prompts if not provided)
- `--include-audit`: Include audit logs (default: true)
- `--incremental, -i`: Create incremental backup
- `--base-backup <PATH>`: Base backup for incremental
- `--compression, -c <LEVEL>`: Compression level 0-9 (default: 6)

**Examples**:

```bash
# Basic backup
secreton-cli backup create --output backup.enc

# With custom compression
secreton-cli backup create --output backup.enc --compression 9

# Incremental backup
secreton-cli backup create \
  --output incremental.enc \
  --incremental \
  --base-backup full-backup.enc
```

### backup verify

Verify backup integrity.

```bash
secreton-cli backup verify [OPTIONS]
```

**Options**:
- `--file, -f <PATH>`: Backup file to verify (required)
- `--password, -p <PASSWORD>`: Decryption password
- `--verbose, -v`: Show detailed information

**Examples**:

```bash
# Basic verification
secreton-cli backup verify --file backup.enc

# Verbose output
secreton-cli backup verify --file backup.enc --verbose
```

### backup restore

Restore from backup.

```bash
secreton-cli backup restore [OPTIONS]
```

**Options**:
- `--file, -f <PATH>`: Backup file to restore (required)
- `--password, -p <PASSWORD>`: Decryption password
- `--point-in-time <TIMESTAMP>`: Restore to specific time (ISO 8601)
- `--secrets-only`: Restore only secrets
- `--audit-only`: Restore only audit logs
- `--dry-run`: Test restore without applying changes
- `--force`: Overwrite existing secrets
- `--target-namespace <NS>`: Restore to different namespace

**Examples**:

```bash
# Basic restore
secreton-cli backup restore --file backup.enc

# Point-in-time restore
secreton-cli backup restore \
  --file backup.enc \
  --point-in-time "2026-02-18T10:00:00Z"

# Dry run
secreton-cli backup restore --file backup.enc --dry-run

# Restore to different namespace
secreton-cli backup restore \
  --file backup.enc \
  --target-namespace recovery
```

### backup list

List available backups.

```bash
secreton-cli backup list [OPTIONS]
```

**Options**:
- `--directory, -d <PATH>`: Directory containing backups (default: current)
- `--detailed`: Show detailed information

**Examples**:

```bash
# List backups in current directory
secreton-cli backup list

# List with details
secreton-cli backup list --directory /var/backups/secreton --detailed
```

---
## Troubleshooting

### Backup Creation Fails

**Problem**: Backup creation fails with "Connection refused" error.

**Solution**:
```bash
# Check Secreton service status
kubectl get pods -n secreton-system

# Check logs
kubectl logs -n secreton-system deployment/secreton

# Verify network connectivity
curl https://secreton.kejaksaan.go.id/v1/sys/health
```

**Problem**: Backup fails with "Insufficient disk space".

**Solution**:
```bash
# Check disk space
df -h /var/backups/secreton

# Clean up old backups manually
find /var/backups/secreton -name "*.enc" -mtime +30 -delete

# Or increase retention policy
# Edit secreton.toml: retention_days = 14
```

### Backup Verification Fails

**Problem**: Verification fails with "Checksum mismatch".

**Solution**:
```bash
# Backup may be corrupted
# Try downloading again if from S3
aws s3 cp s3://secreton-backups/production/backup.enc /tmp/backup.enc --force

# If still fails, use previous backup
secreton-cli backup list --directory /var/backups/secreton
```

**Problem**: Verification fails with "Invalid password".

**Solution**:
```bash
# Ensure you're using the correct password
# Check if password was rotated
# Try with previous passwords

# If password is lost, backup cannot be recovered
# This is why password management is critical!
```

### Restore Fails

**Problem**: Restore fails with "Secret already exists".

**Solution**:
```bash
# Use --force to overwrite
secreton-cli backup restore --file backup.enc --force

# Or restore to different namespace first
secreton-cli backup restore \
  --file backup.enc \
  --target-namespace recovery
```

**Problem**: Restore fails with "Database connection error".

**Solution**:
```bash
# Check PostgreSQL status
kubectl get pods -n secreton-system -l app=postgres

# Check database connectivity
psql -h postgres-host -U secreton -d secreton -c "SELECT 1;"

# Restart PostgreSQL if needed
kubectl rollout restart statefulset/postgres -n secreton-system
```

### S3 Upload Fails

**Problem**: Backup upload to S3 fails with "Access Denied".

**Solution**:
```bash
# Check IAM permissions
aws iam get-role-policy \
  --role-name secreton-instance-role \
  --policy-name SecretonBackupPolicy

# Test S3 access manually
aws s3 ls s3://secreton-backups/

# Verify bucket policy
aws s3api get-bucket-policy --bucket secreton-backups
```

**Problem**: S3 upload fails with "Endpoint not found".

**Solution**:
```bash
# Check endpoint configuration
# For MinIO, ensure force_path_style = true
# For AWS S3, remove endpoint or use correct regional endpoint

# Test endpoint
curl -I https://s3.us-east-1.amazonaws.com
```

---
## Security Best Practices

### Encryption Key Management

**DO**:
- ✅ Generate strong 32-byte encryption keys using cryptographically secure random number generators
- ✅ Store encryption keys in a separate secure location (KMS, HSM, or another Secreton instance)
- ✅ Rotate encryption keys periodically (every 90 days)
- ✅ Use different keys for different environments (production, staging, development)
- ✅ Document key rotation procedures

**DON'T**:
- ❌ Store encryption keys in the same location as backups
- ❌ Use weak passwords (< 16 characters)
- ❌ Share encryption keys via email or chat
- ❌ Commit encryption keys to version control
- ❌ Reuse keys across environments

### Backup Storage Security

**DO**:
- ✅ Enable encryption at rest on S3 buckets (SSE-S3 or SSE-KMS)
- ✅ Enable versioning on S3 buckets to protect against accidental deletion
- ✅ Use IAM roles instead of access keys when possible
- ✅ Enable MFA delete on S3 buckets for production
- ✅ Restrict S3 bucket access to specific IP ranges or VPCs
- ✅ Enable S3 access logging for audit trail

**DON'T**:
- ❌ Make backup buckets publicly accessible
- ❌ Use overly permissive IAM policies
- ❌ Store backups on the same infrastructure as Secreton
- ❌ Disable S3 bucket versioning

### Access Control

**DO**:
- ✅ Limit backup creation to authorized personnel only
- ✅ Require MFA for backup restoration
- ✅ Log all backup and restore operations
- ✅ Review backup access logs regularly
- ✅ Use separate credentials for backup operations

**DON'T**:
- ❌ Allow unrestricted access to backup files
- ❌ Share backup passwords with unauthorized personnel
- ❌ Skip audit logging for backup operations

### Testing and Validation

**DO**:
- ✅ Test restore procedures quarterly
- ✅ Verify backups automatically after creation
- ✅ Perform disaster recovery drills annually
- ✅ Document restore procedures and keep them updated
- ✅ Test backups in a non-production environment first

**DON'T**:
- ❌ Assume backups work without testing
- ❌ Skip backup verification
- ❌ Test restores in production without proper planning

---
## Monitoring & Alerting

### Prometheus Metrics

Secreton exposes the following backup-related metrics:

```prometheus
# Backup operations
secreton_backup_total{status="success|failure"}
secreton_backup_duration_seconds
secreton_backup_size_bytes{type="uncompressed|compressed|encrypted"}
secreton_backup_last_success_timestamp

# Verification
secreton_backup_verification_total{status="success|failure"}
secreton_backup_verification_duration_seconds

# Retention
secreton_backup_retention_deleted_total
secreton_backup_retention_kept_total

# Storage
secreton_backup_storage_operations_total{operation="upload|download|delete",status="success|failure"}
secreton_backup_storage_latency_seconds{operation="upload|download"}
```

### Grafana Dashboard

Import the Secreton Backup Dashboard:

```bash
# Download dashboard JSON
curl -O https://raw.githubusercontent.com/kejaksaan/secreton/main/monitoring/grafana/backup-dashboard.json

# Import to Grafana
# Grafana UI > Dashboards > Import > Upload JSON file
```

Dashboard includes:
- Backup success rate (last 24h, 7d, 30d)
- Backup size trends
- Backup duration trends
- Failed backup alerts
- Storage usage
- Verification status

### Alerting Rules

#### Prometheus AlertManager Rules

Create `secreton-backup-alerts.yaml`:

```yaml
groups:
  - name: secreton_backup
    interval: 5m
    rules:
      # Backup failure alert
      - alert: SecretonBackupFailed
        expr: increase(secreton_backup_total{status="failure"}[1h]) > 0
        for: 5m
        labels:
          severity: critical
          component: backup
        annotations:
          summary: "Secreton backup failed"
          description: "Backup operation failed in the last hour. Check logs immediately."

      # No recent backup alert
      - alert: SecretonNoRecentBackup
        expr: (time() - secreton_backup_last_success_timestamp) > 28800  # 8 hours
        for: 10m
        labels:
          severity: warning
          component: backup
        annotations:
          summary: "No recent Secreton backup"
          description: "Last successful backup was more than 8 hours ago."

      # Backup verification failure
      - alert: SecretonBackupVerificationFailed
        expr: increase(secreton_backup_verification_total{status="failure"}[1h]) > 0
        for: 5m
        labels:
          severity: critical
          component: backup
        annotations:
          summary: "Secreton backup verification failed"
          description: "Backup verification failed. Backup may be corrupted."

      # Backup size anomaly
      - alert: SecretonBackupSizeAnomaly
        expr: |
          abs(secreton_backup_size_bytes{type="compressed"} -
              avg_over_time(secreton_backup_size_bytes{type="compressed"}[7d])) >
          (stddev_over_time(secreton_backup_size_bytes{type="compressed"}[7d]) * 3)
        for: 15m
        labels:
          severity: warning
          component: backup
        annotations:
          summary: "Secreton backup size anomaly detected"
          description: "Backup size deviates significantly from the 7-day average."

      # Storage operation failures
      - alert: SecretonBackupStorageErrors
        expr: increase(secreton_backup_storage_operations_total{status="failure"}[1h]) > 3
        for: 5m
        labels:
          severity: warning
          component: backup
        annotations:
          summary: "Secreton backup storage errors"
          description: "Multiple storage operation failures detected. Check S3 connectivity."
```

Apply the rules:

```bash
kubectl apply -f secreton-backup-alerts.yaml
```

### Slack Notifications

Configure Slack webhook for backup alerts:

```toml
[backup.alerting]
enabled = true
slack_webhook_url = "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
alert_on_failure = true
alert_on_success = false  # Set to true for production
```

### Email Notifications

Configure SMTP for email alerts:

```toml
[backup.alerting]
enabled = true
email_enabled = true
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = "alerts@kejaksaan.go.id"
smtp_password = "your-app-password"
alert_recipients = ["ops@kejaksaan.go.id", "security@kejaksaan.go.id"]
```

### Health Check Endpoint

Monitor backup health via HTTP endpoint:

```bash
# Check backup status
curl https://secreton.kejaksaan.go.id/v1/sys/backup/status

# Response
{
  "last_backup": "2026-02-18T02:00:00Z",
  "last_backup_status": "success",
  "last_backup_size_bytes": 1048576,
  "next_scheduled_backup": "2026-02-19T02:00:00Z",
  "retention_days": 30,
  "total_backups": 120,
  "storage_type": "s3",
  "storage_health": "healthy"
}
```

---

## Appendix A: Backup Encryption Key Generation

### Generate Secure Encryption Key

```bash
# Using OpenSSL (recommended)
openssl rand -hex 32

# Using Python
python3 -c "import secrets; print(secrets.token_hex(32))"

# Using /dev/urandom
head -c 32 /dev/urandom | xxd -p -c 32
```

### Store Key in Secreton

```bash
# Store backup encryption key in Secreton itself
secreton-cli kv put secret/backup/encryption-key \
  key="$(openssl rand -hex 32)"

# Retrieve when needed
BACKUP_KEY=$(secreton-cli kv get -field=key secret/backup/encryption-key)
```

### Store Key in AWS KMS

```bash
# Create KMS key
aws kms create-key \
  --description "Secreton Backup Encryption Key" \
  --key-usage ENCRYPT_DECRYPT

# Store key ID in configuration
# Use KMS to encrypt/decrypt backup encryption key
```

---
## Appendix B: Backup Retention Calculator

Calculate storage requirements based on retention policy:

```python
#!/usr/bin/env python3
"""
Backup Storage Calculator
Calculate storage requirements for Secreton backups
"""

def calculate_storage(
    backup_size_mb: float,
    backups_per_day: int,
    retention_days: int,
    compression_ratio: float = 0.3,  # 70% reduction
    growth_rate: float = 0.05  # 5% monthly growth
) -> dict:
    """Calculate storage requirements"""

    # Compressed backup size
    compressed_size_mb = backup_size_mb * compression_ratio

    # Total backups
    total_backups = backups_per_day * retention_days

    # Storage without growth
    base_storage_gb = (compressed_size_mb * total_backups) / 1024

    # Storage with growth (average over retention period)
    months = retention_days / 30
    avg_growth_factor = (1 + (growth_rate * months / 2))
    storage_with_growth_gb = base_storage_gb * avg_growth_factor

    # Add 20% buffer
    recommended_storage_gb = storage_with_growth_gb * 1.2

    return {
        "backup_size_mb": backup_size_mb,
        "compressed_size_mb": compressed_size_mb,
        "backups_per_day": backups_per_day,
        "retention_days": retention_days,
        "total_backups": total_backups,
        "base_storage_gb": round(base_storage_gb, 2),
        "storage_with_growth_gb": round(storage_with_growth_gb, 2),
        "recommended_storage_gb": round(recommended_storage_gb, 2),
    }

# Example calculations
print("Secreton Backup Storage Calculator")
print("=" * 50)

scenarios = [
    ("Small (100 MB DB, 4x/day, 30 days)", 100, 4, 30),
    ("Medium (500 MB DB, 4x/day, 30 days)", 500, 4, 30),
    ("Large (2 GB DB, 4x/day, 30 days)", 2048, 4, 30),
    ("Enterprise (10 GB DB, 4x/day, 30 days)", 10240, 4, 30),
]

for name, size, freq, retention in scenarios:
    result = calculate_storage(size, freq, retention)
    print(f"\n{name}")
    print(f"  Compressed backup size: {result['compressed_size_mb']:.1f} MB")
    print(f"  Total backups: {result['total_backups']}")
    print(f"  Recommended storage: {result['recommended_storage_gb']:.1f} GB")
```

**Example Output**:

```
Small (100 MB DB, 4x/day, 30 days)
  Compressed backup size: 30.0 MB
  Total backups: 120
  Recommended storage: 4.3 GB

Medium (500 MB DB, 4x/day, 30 days)
  Compressed backup size: 150.0 MB
  Total backups: 120
  Recommended storage: 21.6 GB

Large (2 GB DB, 4x/day, 30 days)
  Compressed backup size: 614.4 MB
  Total backups: 120
  Recommended storage: 88.5 GB

Enterprise (10 GB DB, 4x/day, 30 days)
  Compressed backup size: 3072.0 MB
  Total backups: 120
  Recommended storage: 442.4 GB
```

---
## Appendix C: Automated Backup Testing Script

```bash
#!/bin/bash
# backup-test.sh
# Automated backup testing script for Secreton
# Run this monthly to verify backup/restore procedures

set -euo pipefail

# Configuration
BACKUP_DIR="/var/backups/secreton"
TEST_NAMESPACE="backup-test-$(date +%s)"
BACKUP_PASSWORD="${BACKUP_PASSWORD:-}"
SECRETON_URL="${SECRETON_URL:-https://secreton.kejaksaan.go.id}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Step 1: Create test secrets
log_info "Step 1: Creating test secrets..."
secreton-cli kv put secret/test/backup-test-1 value="test-value-1"
secreton-cli kv put secret/test/backup-test-2 value="test-value-2"
secreton-cli kv put secret/test/backup-test-3 value="test-value-3"

# Step 2: Create backup
log_info "Step 2: Creating backup..."
BACKUP_FILE="${BACKUP_DIR}/test-backup-$(date +%Y%m%d-%H%M%S).enc"
secreton-cli backup create \
    --output "$BACKUP_FILE" \
    --password "$BACKUP_PASSWORD" \
    --compression 6

if [ ! -f "$BACKUP_FILE" ]; then
    log_error "Backup file not created!"
    exit 1
fi

log_info "Backup created: $BACKUP_FILE"

# Step 3: Verify backup
log_info "Step 3: Verifying backup integrity..."
if secreton-cli backup verify \
    --file "$BACKUP_FILE" \
    --password "$BACKUP_PASSWORD"; then
    log_info "✓ Backup verification passed"
else
    log_error "✗ Backup verification failed!"
    exit 1
fi

# Step 4: Delete test secrets
log_info "Step 4: Deleting test secrets..."
secreton-cli kv delete secret/test/backup-test-1
secreton-cli kv delete secret/test/backup-test-2
secreton-cli kv delete secret/test/backup-test-3

# Step 5: Restore to test namespace
log_info "Step 5: Restoring to test namespace..."
if secreton-cli backup restore \
    --file "$BACKUP_FILE" \
    --password "$BACKUP_PASSWORD" \
    --target-namespace "$TEST_NAMESPACE" \
    --secrets-only; then
    log_info "✓ Restore completed"
else
    log_error "✗ Restore failed!"
    exit 1
fi

# Step 6: Verify restored secrets
log_info "Step 6: Verifying restored secrets..."
ERRORS=0

for i in 1 2 3; do
    VALUE=$(secreton-cli kv get -field=value "$TEST_NAMESPACE/secret/test/backup-test-$i" 2>/dev/null || echo "")
    if [ "$VALUE" = "test-value-$i" ]; then
        log_info "✓ Secret $i verified"
    else
        log_error "✗ Secret $i verification failed! Expected: test-value-$i, Got: $VALUE"
        ERRORS=$((ERRORS + 1))
    fi
done

# Step 7: Cleanup
log_info "Step 7: Cleaning up..."
secreton-cli namespace delete "$TEST_NAMESPACE" 2>/dev/null || true
rm -f "$BACKUP_FILE"

# Summary
echo ""
echo "========================================="
echo "Backup Test Summary"
echo "========================================="
if [ $ERRORS -eq 0 ]; then
    log_info "✓ All tests passed!"
    echo "Backup and restore procedures are working correctly."
    exit 0
else
    log_error "✗ $ERRORS test(s) failed!"
    echo "Please investigate backup/restore issues immediately."
    exit 1
fi
```

**Usage**:

```bash
# Make executable
chmod +x backup-test.sh

# Run test
export BACKUP_PASSWORD="your-backup-password"
./backup-test.sh

# Schedule monthly test
crontab -e
# Add: 0 3 1 * * /path/to/backup-test.sh >> /var/log/backup-test.log 2>&1
```

---
## Appendix D: Compliance and Audit

### Regulatory Requirements

Secreton backup system helps meet various compliance requirements:

#### Indonesian Government Standards
- **Peraturan Pemerintah No. 71 Tahun 2019**: Data protection and backup requirements
- **Surat Edaran Menkominfo**: Disaster recovery planning for government systems
- **ISO 27001**: Information security management (backup and recovery controls)

#### International Standards
- **SOC 2 Type II**: Backup and disaster recovery controls
- **ISO 22301**: Business continuity management
- **NIST SP 800-53**: Contingency planning (CP family)

### Audit Trail

All backup operations are logged to the audit system:

```bash
# View backup-related audit logs
secreton-cli audit list --operation backup --limit 50

# Export audit logs for compliance
secreton-cli audit export \
    --start-date 2026-01-01 \
    --end-date 2026-12-31 \
    --format json \
    --output backup-audit-2026.json
```

### Compliance Checklist

- [ ] Automated backups enabled and running on schedule
- [ ] Backup encryption enabled with strong keys
- [ ] Backup verification enabled and passing
- [ ] Retention policy configured and enforced
- [ ] Backup storage secured with appropriate access controls
- [ ] Disaster recovery procedures documented and tested
- [ ] Backup restoration tested quarterly
- [ ] Backup monitoring and alerting configured
- [ ] Audit logging enabled for all backup operations
- [ ] Backup encryption keys stored securely
- [ ] Off-site backup storage configured
- [ ] Backup access restricted to authorized personnel
- [ ] Backup procedures reviewed annually
- [ ] Incident response plan includes backup scenarios

---

## Appendix E: FAQ

### General Questions

**Q: How long does a backup take?**
A: Backup duration depends on data size. Typical times:
- Small (< 1 GB): 1-2 minutes
- Medium (1-5 GB): 5-10 minutes
- Large (5-20 GB): 15-30 minutes
- Very Large (> 20 GB): 30+ minutes

**Q: Can I run backups while Secreton is serving requests?**
A: Yes! Backups are designed to run without downtime. They use consistent snapshots.

**Q: How much storage do I need?**
A: Use the formula: `(backup_size × backups_per_day × retention_days × 1.2)`
Example: 500 MB × 4 × 30 × 1.2 = 72 GB

**Q: Can I restore individual secrets?**
A: Yes, use `--target-namespace` to restore to a temporary namespace, then copy specific secrets.

**Q: What happens if a backup fails?**
A: Secreton will:
1. Log the error to audit trail
2. Trigger alerts (if configured)
3. Retry on next scheduled run
4. Keep previous successful backup

### Technical Questions

**Q: What encryption algorithm is used?**
A: ChaCha20-Poly1305 (AEAD cipher) with 256-bit keys.

**Q: Can I use my own encryption keys?**
A: Yes, provide a 32-byte key in the configuration or via `--encryption-key` flag.

**Q: Are backups compressed?**
A: Yes, using gzip. Compression level is configurable (0-9, default 6).

**Q: What's the difference between Raft snapshot and PostgreSQL dump?**
A:
- **Raft snapshot**: Consensus state for HA clusters
- **PostgreSQL dump**: All secrets, policies, leases, audit logs

**Q: Can I restore to a different Secreton version?**
A: Backups include version information. Restoring to a different major version may require migration.

**Q: How do I migrate backups between storage backends?**
A:
```bash
# Download from old storage
secreton-cli backup list --directory /old/storage
secreton-cli backup verify --file /old/storage/backup.enc

# Upload to new storage (configure new storage in secreton.toml)
# Backups will automatically use new storage on next run
```

### Disaster Recovery Questions

**Q: What's the RTO (Recovery Time Objective)?**
A: Typical RTO: 30-60 minutes for complete cluster recovery.

**Q: What's the RPO (Recovery Point Objective)?**
A: RPO equals your backup frequency. For 6-hour backups, RPO is 6 hours.

**Q: Can I restore to a different region?**
A: Yes, backups are portable. Download from S3 and restore to any Secreton instance.

**Q: What if I lose the backup encryption key?**
A: Backups cannot be recovered without the encryption key. This is by design for security.

**Q: How do I test disaster recovery without affecting production?**
A: Use `--target-namespace` to restore to a test namespace, or restore to a separate Secreton instance.

---

## Support and Resources

### Documentation
- [Secreton Main Documentation](../README.md)
- [Backup Manager README](../crates/backup/README.md)
- [CLI Reference](../crates/cli/README.md)
- [API Documentation](../docs/API.md)

### Community
- GitHub Issues: https://github.com/kejaksaan/secreton/issues
- Discussions: https://github.com/kejaksaan/secreton/discussions

### Professional Support
For enterprise support, contact: support@kejaksaan.go.id

---

**Document Version**: 2.0
**Last Updated**: February 18, 2026
**Maintained by**: SIMPEL DevOps Team
**Next Review**: May 18, 2026

---

## Change Log

### Version 2.0 (February 18, 2026)
- Complete rewrite with comprehensive disaster recovery procedures
- Added S3-compatible storage setup guides
- Added monitoring and alerting section
- Added compliance and audit section
- Added automated testing scripts
- Added storage calculator
- Added extensive troubleshooting guide

### Version 1.0 (February 12, 2026)
- Initial documentation
- Basic backup/restore procedures
- CLI reference

---

**✅ Validates Requirements 2.5.11**: Complete backup/restore documentation including setup guides, configuration, restoration procedures, and disaster recovery playbook.
