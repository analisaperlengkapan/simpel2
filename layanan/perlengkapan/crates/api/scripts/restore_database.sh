#!/bin/bash
# ============================================================================
# Database Restore Script for SIMPEL Migration
# Description: Restore database from backup with verification
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
BACKUP_FILE="${1:-}"

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

# Usage information
usage() {
    echo "Usage: $0 <backup_file>"
    echo ""
    echo "Example:"
    echo "  $0 /var/backups/postgresql/simpelv2/simpelv2_full_20260211_120000.sql.gz"
    echo ""
    exit 1
}

# Check if backup file is provided
if [ -z "$BACKUP_FILE" ]; then
    log_error "Backup file not specified"
    usage
fi

# Check if backup file exists
if [ ! -f "$BACKUP_FILE" ]; then
    log_error "Backup file not found: $BACKUP_FILE"
    exit 1
fi

# Verify checksum
verify_checksum() {
    CHECKSUM_FILE="${BACKUP_FILE}.sha256"

    if [ ! -f "$CHECKSUM_FILE" ]; then
        log_warn "Checksum file not found: $CHECKSUM_FILE"
        log_warn "Skipping checksum verification"
        return 0
    fi

    log_info "Verifying backup checksum..."

    if sha256sum -c "$CHECKSUM_FILE" &>/dev/null; then
        log_info "Checksum verification passed"
    else
        log_error "Checksum verification failed"
        exit 1
    fi
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

# Confirm restore operation
confirm_restore() {
    echo ""
    log_warn "⚠️  WARNING: This will restore the database from backup"
    log_warn "⚠️  All current data in perlengkapan and integrasi schemas will be replaced"
    echo ""
    read -p "Are you sure you want to continue? (yes/no): " CONFIRM

    if [ "$CONFIRM" != "yes" ]; then
        log_info "Restore cancelled by user"
        exit 0
    fi
}

# Drop existing schemas
drop_schemas() {
    log_info "Dropping existing schemas..."

    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
DROP SCHEMA IF EXISTS perlengkapan CASCADE;
DROP SCHEMA IF EXISTS integrasi CASCADE;
EOF

    log_info "Schemas dropped"
}

# Restore database
restore_database() {
    log_info "Restoring database from backup..."
    log_info "Backup file: $BACKUP_FILE"

    # Decompress and restore
    gunzip -c "$BACKUP_FILE" | psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME"

    if [ $? -eq 0 ]; then
        log_info "Database restored successfully"
    else
        log_error "Database restore failed"
        exit 1
    fi
}

# Verify restore
verify_restore() {
    log_info "Verifying restored data..."

    # Check if schemas exist
    SCHEMA_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM information_schema.schemata WHERE schema_name IN ('perlengkapan', 'integrasi')")

    if [ "$SCHEMA_COUNT" -ne 2 ]; then
        log_error "Schemas not found after restore"
        exit 1
    fi

    # Check table count
    TABLE_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema IN ('perlengkapan', 'integrasi')")

    log_info "Tables restored: $TABLE_COUNT"

    # Check record counts
    KEBUTUHAN_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM perlengkapan.pengajuan_kebutuhan_bmn" 2>/dev/null || echo "0")
    PAKAIAN_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas" 2>/dev/null || echo "0")

    log_info "Record counts:"
    log_info "  - Kebutuhan BMN: $KEBUTUHAN_COUNT"
    log_info "  - Pakaian Dinas: $PAKAIAN_COUNT"

    log_info "Restore verification completed"
}

# Analyze tables
analyze_tables() {
    log_info "Analyzing tables for query planner..."

    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "ANALYZE;"

    log_info "Table analysis completed"
}

# Main execution
main() {
    echo "═══════════════════════════════════════════════════════════"
    echo "  SIMPEL Database Restore Script"
    echo "═══════════════════════════════════════════════════════════"
    echo ""

    verify_checksum
    test_connection
    confirm_restore
    drop_schemas
    restore_database
    verify_restore
    analyze_tables

    echo ""
    echo "═══════════════════════════════════════════════════════════"
    log_info "✅ Restore completed successfully"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
}

# Run main function
main "$@"
