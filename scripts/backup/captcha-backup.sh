#!/bin/bash
# CAPTCHA System Backup Script
# Backs up CAPTCHA analytics data and configuration

set -euo pipefail

# Configuration
BACKUP_DIR="/backups"
DB_HOST="${CAPTCHA_DB_HOST:-captcha-analytics-db}"
DB_PORT="${CAPTCHA_DB_PORT:-5432}"
DB_NAME="${CAPTCHA_DB_NAME:-captcha_analytics}"
DB_USER="${CAPTCHA_DB_USER:-captcha_user}"
RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-30}"

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Generate timestamp
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_FILE="$BACKUP_DIR/captcha_backup_$TIMESTAMP.sql.gz"

echo "Starting CAPTCHA database backup at $(date)"

# Create database backup
pg_dump \
    --host="$DB_HOST" \
    --port="$DB_PORT" \
    --username="$DB_USER" \
    --dbname="$DB_NAME" \
    --no-password \
    --verbose \
    --format=custom \
    --compress=9 \
    --exclude-table-data="captcha_challenges" \
    --exclude-table-data="captcha_validation_attempts" \
    | gzip > "$BACKUP_FILE"

if [ $? -eq 0 ]; then
    echo "Database backup completed successfully: $BACKUP_FILE"

    # Verify backup file
    if [ -f "$BACKUP_FILE" ] && [ -s "$BACKUP_FILE" ]; then
        echo "Backup file verified: $(du -h "$BACKUP_FILE" | cut -f1)"
    else
        echo "ERROR: Backup file is empty or missing"
        exit 1
    fi
else
    echo "ERROR: Database backup failed"
    exit 1
fi

# Backup configuration files
CONFIG_BACKUP_FILE="$BACKUP_DIR/captcha_config_$TIMESTAMP.tar.gz"
tar -czf "$CONFIG_BACKUP_FILE" \
    /etc/captcha/ \
    /etc/prometheus/captcha.yml \
    /etc/grafana/provisioning/ \
    /etc/alertmanager/captcha.yml \
    2>/dev/null || echo "Some config files may not exist, continuing..."

echo "Configuration backup completed: $CONFIG_BACKUP_FILE"

# Clean up old backups
echo "Cleaning up backups older than $RETENTION_DAYS days"
find "$BACKUP_DIR" -name "captcha_backup_*.sql.gz" -mtime +$RETENTION_DAYS -delete
find "$BACKUP_DIR" -name "captcha_config_*.tar.gz" -mtime +$RETENTION_DAYS -delete

# Create backup summary
SUMMARY_FILE="$BACKUP_DIR/backup_summary_$TIMESTAMP.txt"
cat > "$SUMMARY_FILE" << EOF
CAPTCHA System Backup Summary
=============================
Timestamp: $(date)
Database Host: $DB_HOST
Database Name: $DB_NAME
Backup File: $BACKUP_FILE
Config Backup: $CONFIG_BACKUP_FILE
Backup Size: $(du -h "$BACKUP_FILE" | cut -f1)
Config Size: $(du -h "$CONFIG_BACKUP_FILE" | cut -f1)

Database Statistics:
EOF

# Add database statistics to summary
psql \
    --host="$DB_HOST" \
    --port="$DB_PORT" \
    --username="$DB_USER" \
    --dbname="$DB_NAME" \
    --no-password \
    --tuples-only \
    --command="
        SELECT
            'Total Challenges: ' || COUNT(*)
        FROM captcha_challenges;

        SELECT
            'Solved Challenges: ' || COUNT(*)
        FROM captcha_challenges
        WHERE solved = true;

        SELECT
            'Bot Detections: ' || COUNT(*)
        FROM captcha_behavioral_metrics
        WHERE classification = 'Bot';

        SELECT
            'Validation Attempts: ' || COUNT(*)
        FROM captcha_validation_attempts;
    " >> "$SUMMARY_FILE" 2>/dev/null || echo "Could not retrieve database statistics" >> "$SUMMARY_FILE"

echo "Backup summary created: $SUMMARY_FILE"

# Send backup notification (if configured)
if [ -n "${BACKUP_NOTIFICATION_WEBHOOK:-}" ]; then
    curl -X POST "$BACKUP_NOTIFICATION_WEBHOOK" \
        -H "Content-Type: application/json" \
        -d "{
            \"text\": \"CAPTCHA backup completed successfully\",
            \"timestamp\": \"$(date -Iseconds)\",
            \"backup_file\": \"$BACKUP_FILE\",
            \"backup_size\": \"$(du -h "$BACKUP_FILE" | cut -f1)\"
        }" \
        2>/dev/null || echo "Failed to send backup notification"
fi

echo "CAPTCHA backup process completed at $(date)"
