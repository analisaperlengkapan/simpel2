# Secreton Backup Restore Guide

## Overview

The Secreton CLI provides comprehensive backup and restore functionality for disaster recovery and data migration scenarios. This guide covers the restore command and its various options.

## Restore Command

### Basic Restore

Restore all data from a backup file:

```bash
secreton-cli backup restore --file backup.bak
```

You will be prompted for the decryption password.

### Restore with Password

Provide the password via command line (not recommended for production):

```bash
secreton-cli backup restore --file backup.bak --password "your-password"
```

### Dry Run

Verify what would be restored without making any changes:

```bash
secreton-cli backup restore --file backup.bak --dry-run
```

This is useful for:
- Validating backup integrity
- Previewing restore operations
- Testing restore procedures

### Point-in-Time Restore

Restore data as it existed at a specific point in time:

```bash
secreton-cli backup restore --file backup.bak --point-in-time "2025-10-29T10:00:00Z"
```

Only secrets and audit logs created before the specified timestamp will be restored.

### Selective Restore

#### Restore Only Secrets

Skip audit logs and restore only secrets:

```bash
secreton-cli backup restore --file backup.bak --secrets-only
```

#### Restore Only Audit Logs

Skip secrets and restore only audit logs:

```bash
secreton-cli backup restore --file backup.bak --audit-only
```

### Force Overwrite

By default, existing secrets are skipped. Use `--force` to overwrite:

```bash
secreton-cli backup restore --file backup.bak --force
```

**Warning:** This will overwrite existing secrets with backup data.

### Namespace Migration

Restore to a different namespace:

```bash
secreton-cli backup restore --file backup.bak --target-namespace production
```

This is useful for:
- Migrating data between environments
- Testing with production data in staging
- Namespace reorganization

## Restore Process

The restore operation follows these steps:

1. **Load Backup File** - Read and decrypt the backup file
2. **Validate Integrity** - Verify checksum and backup structure
3. **Plan Restore** - Determine what will be restored
4. **Confirm Operation** - Prompt for user confirmation (unless `--force`)
5. **Restore Secrets** - Upload secrets to engine
6. **Restore Audit Logs** - Upload audit logs (if included)
7. **Verify** - Check engine accessibility and count

## Restore Options

| Option | Description | Default |
|--------|-------------|---------|
| `--file, -f` | Backup file path | Required |
| `--password, -p` | Decryption password | Prompt |
| `--point-in-time` | ISO 8601 timestamp | None |
| `--secrets-only` | Restore only secrets | false |
| `--audit-only` | Restore only audit logs | false |
| `--dry-run` | Verify without applying | false |
| `--force` | Overwrite existing secrets | false |
| `--target-namespace` | Restore to different namespace | None |

## Examples

### Example 1: Full Restore

```bash
# Create backup
secreton-cli backup create --output full-backup.bak

# Restore everything
secreton-cli backup restore --file full-backup.bak
```

### Example 2: Disaster Recovery

```bash
# Restore with force to overwrite corrupted data
secreton-cli backup restore \
  --file disaster-backup.bak \
  --force \
  --password "$BACKUP_PASSWORD"
```

### Example 3: Environment Migration

```bash
# Restore production backup to staging namespace
secreton-cli backup restore \
  --file prod-backup.bak \
  --target-namespace staging \
  --secrets-only
```

### Example 4: Point-in-Time Recovery

```bash
# Restore to state before incident
secreton-cli backup restore \
  --file backup.bak \
  --point-in-time "2025-10-29T09:00
  --force
```

### Example 5: Validation

```bash
# Verify backup without restoring
secreton-cli backup restore \
  --file backup.bak \
  --dry-run
```

## Best Practices

### Before Restore

1. **Verify Backup Integrity**
   ```bash
   secreton-cli backup verify --file backup.bak
   ```

2. **Test with Dry Run**
   ```bash
   secreton-cli backup restore --file backup.bak --dry-run
   ```

3. **Create Current Backup**
   ```bash
   secreton-cli backup create --output pre-restore-backup.bak
   ```

### During Restore

1. **Monitor Progress** - Watch the progress output
2. **Check for Errors** - Note any failed operations
3. **Verify Statistics** - Review restored/skipped/failed counts

### After Restore

1. **Verify Critical Secrets**
   ```bash
   secreton-cli secret get critical/path
   ```

2. **Test Application Connectivity**
   - Ensure applications can access restored secrets
   - Verify authentication works

3. **Review Audit Logs**
   - Check for restore operations in audit logs
   - Verify data integrity

4. **Create New Backup**
   ```bash
   secreton-cli backup create --output post-restore-backup.bak
   ```

## Troubleshooting

### Decryption Failed

**Problem:** "Decryption failed - check password"

**Solution:**
- Verify you're using the correct password
- Check if backup file is corrupted
- Ensure backup was created with same encryption algorithm

### Checksum Mismatch

**Problem:** "Backup integrity check failed! Checksum mismatch"

**Solution:**
- Backup file may be corrupted
- Try restoring from a different backup
- Verify backup file wasn't modified

### Secrets Already Exist

**Problem:** Many secrets skipped during restore

**Solution:**
- Use `--force` flag to overwrite existing secrets
- Or delete existing secrets before restore
- Or restore to different namespace with `--target-namespace`

### Connection Failed

**Problem:** Cannot connect to engine server

**Solution:**
- Verify server URL: `secreton-cli status`
- Check network connectivity
- Ensure engine is unsealed and running

### Partial Restore

**Problem:** Some secrets failed to restore

**Solution:**
- Check engine logs for errors
- Verify permissions and quotas
- Retry restore for failed secrets
- Contact administrator if issues persist

## Security Considerations

1. **Password Protection**
   - Never store passwords in scripts
   - Use environment variables or prompt
   - Rotate backup passwords regularly

2. **Backup Storage**
   - Store backups in secure location
   - Encrypt backup storage
   - Implement access controls

3. **Audit Trail**
   - All restore operations are logged
   - Review audit logs after restore
   - Monitor for unauthorized restores

4. **Namespace Isolation**
   - Respect namespace boundaries
   - Verify target namespace permissions
   - Test namespace migration in staging

## Related Commands

- `secreton-cli backup create` - Create a backup
- `secreton-cli backup verify` - Verify backup integrity
- `secreton-cli backup list` - List available backups
- `secreton-cli status` - Check engine status

## Support

For issues or questions:
- Check engine logs: `/var/log/secreton/`
- Review audit logs: `secreton-cli audit logs`
- Contact: Secreton Team

## Version

This guide is for Secreton CLI v1.0.0
