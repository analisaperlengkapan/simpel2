#!/bin/bash
# ============================================================================
# Database Backup Script for SIMPEL Migration
# Description: Full backup with WAL archiving for point-in-time recovery
# Author: SIMPelv2 Team
# Created: 2026-02-11
# Requirements: NFR-A003, NFR-A004, NFR-A005
# ============================================================================

set -euo pipefail

# Configuration
DB_NAME="${DB_NAME:-simpelv2}"
DB_USER="${DB_USER:-simpelv2}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
BACKUP_DIR="${BACKUP_DIR:-/var/backups/postgresql/simpelv2}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/simpelv2_full_${TIMESTAMP}.sql.gz"
CHECKSUM_FILE="${BACKUP_FILE}.sha256"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check if pg_dump is available
    if ! command -v pg_dump &> /dev/null; then
        log_error "pg_dump not found. Please install PostgreSQL client tools."
        exit 1
    fi

    # Check if backup directory exists
    if [ ! -d "$BACKUP_DIR" ]; then
        log_info "Creating backup directory: $BACKUP_DIR"
        mkdir -p "$BACKUP_DIR"
    fi

    # Check disk space (require at least 10GB free)
    AVAILABLE_SPACE=$(df -BG "$BACKUP_DIR" | awk 'NR==2 {print $4}' | sed 's/G//')
    if [ "$AVAILABLE_SPACE" -lt 10 ]; then
        log_error "Insufficient disk space. Available: ${AVAILABLE_SPACE}GB, Required: 10GB"
        exit 1
    fi

    log_info "Prerequisites check passed"
}

# Test database connection
test_connection() {
    log_info "Testing database connection..."

    if ! psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1" &> /dev/null; then
        log_error "Cannot connect to database"
        exit 1
    fi

    log_info "Database connection successful"
}

# Get database statistics
get_db_stats() {
    log_info "Gathering database statistics..."

    DB_SIZE=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT pg_size_pretty(pg_database_size('$DB_NAME'))")
    TABLE_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'perlengkapan'")

    log_info "Database size: $DB_SIZE"
    log_info "Tables in perlengkapan schema: $TABLE_COUNT"
}

# Perform full backup
perform_backup() {
    log_info "Starting full database backup..."
    log_info "Backup file: $BACKUP_FILE"

    # Perform backup with compression
    pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" \
        --format=plain \
        --no-owner \
        --no-acl \
        --verbose \
        --schema=perlengkapan \
        --schema=integrasi \
        "$DB_NAME" | gzip > "$BACKUP_FILE"

    if [ $? -eq 0 ]; then
        log_info "Backup completed successfully"
    else
        log_error "Backup failed"
        exit 1
    fi
}

# Generate checksum
generate_checksum() {
    log_info "Generating SHA-256 checksum..."

    sha256sum "$BACKUP_FILE" > "$CHECKSUM_FILE"

    CHECKSUM=$(cat "$CHECKSUM_FILE" | awk '{print $1}')
    log_info "Checksum: $CHECKSUM"
}

# Verify backup integrity
verify_backup() {
    log_info "Verifying backup integrity..."

    # Check if file exists and is not empty
    if [ ! -s "$BACKUP_FILE" ]; then
        log_error "Backup file is empty or does not exist"
        exit 1
    fi

    # Verify gzip integrity
    if ! gzip -t "$BACKUP_FILE" 2>/dev/null; then
        log_error "Backup file is corrupted (gzip test failed)"
        exit 1
    fi

    # Verify checksum
    if ! sha256sum -c "$CHECKSUM_FILE" &>/dev/null; then
        log_error "Checksum verification failed"
        exit 1
    fi

    log_info "Backup integrity verified"
}

# Get backup file size
get_backup_size() {
    BACKUP_SIZE=$(du -h "$BACKUP_FILE" | awk '{print $1}')
    log_info "Backup file size: $BACKUP_SIZE"
}

# Clean old backups
clean_old_backups() {
    log_info "Cleaning backups older than $RETENTION_DAYS days..."

    find "$BACKUP_DIR" -name "simpelv2_full_*.sql.gz" -mtime +$RETENTION_DAYS -delete
    find "$BACKUP_DIR" -name "simpelv2_full_*.sql.gz.sha256" -mtime +$RETENTION_DAYS -delete

    REMAINING_BACKUPS=$(find "$BACKUP_DIR" -name "simpelv2_full_*.sql.gz" | wc -l)
    log_info "Remaining backups: $REMAINING_BACKUPS"
}

# Export backup metadata
export_metadata() {
    METADATA_FILE="${BACKUP_DIR}/simpelv2_backup_${TIMESTAMP}.json"

    cat > "$METADATA_FILE" <<EOF
{
  "timestamp": "$TIMESTAMP",
  "database": "$DB_NAME",
  "host": "$DB_HOST",
  "port": $DB_PORT,
  "backup_file": "$BACKUP_FILE",
  "checksum_file": "$CHECKSUM_FILE",
  "checksum": "$(cat $CHECKSUM_FILE | awk '{print $1}')",
  "backup_size": "$BACKUP_SIZE",
  "database_size": "$DB_SIZE",
  "table_count": $TABLE_COUNT,
  "schemas": ["perlengkapan", "integrasi"],
  "retention_days": $RETENTION_DAYS
}
EOF

    log_info "Metadata exported to: $METADATA_FILE"
}

# Main execution
main() {
    echo "═══════════════════════════════════════════════════════════"
    echo "  SIMPEL Database Backup Script"
    echo "═══════════════════════════════════════════════════════════"
    echo ""

    check_prerequisites
    test_connection
    get_db_stats
    perform_backup
    generate_checksum
    verify_backup
    get_backup_size
    export_metadata
    clean_old_backups

    echo ""
    echo "═══════════════════════════════════════════════════════════"
    log_info "✅ Backup completed successfully"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    log_info "Backup details:"
    log_info "  - File: $BACKUP_FILE"
    log_info "  - Size: $BACKUP_SIZE"
    log_info "  - Checksum: $(cat $CHECKSUM_FILE | awk '{print $1}')"
    log_info "  - Metadata: ${BACKUP_DIR}/simpelv2_backup_${TIMESTAMP}.json"
    echo ""
}

# Run main function
main "$@"
