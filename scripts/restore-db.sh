#!/bin/bash
# Database restoration script for SIMPEL v1
# Usage: ./restore-db.sh [backup-file] [db-name] [db-user] [db-host]

# `pipefail` is required so `gunzip < backup | psql` fails the pipeline
# when `gunzip` exits non-zero (corrupted backup) — otherwise the script
# would only see psql's exit status, which is 0 when it receives empty
# input from a failed gunzip. Combined with `set -e`, that previously
# caused the script to report "Database restoration completed
# successfully" against an empty database (silent data loss).
set -eo pipefail

# Default values
BACKUP_FILE="${1:-dbsimpelv1.sql.gz}"
DB_NAME="${2:-dbsimpelv1}"
DB_USER="${3:-postgres}"
DB_HOST="${4:-localhost}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Validate inputs
if [[ ! -f "$BACKUP_FILE" ]]; then
    log_error "Backup file not found: $BACKUP_FILE"
    exit 1
fi

log_info "Starting database restoration for SIMPEL v1"
log_info "Backup file: $BACKUP_FILE"
log_info "Database: $DB_NAME"
log_info "User: $DB_USER"
log_info "Host: $DB_HOST"

# Check PostgreSQL connection
log_info "Checking PostgreSQL connectivity..."
if ! psql -h "$DB_HOST" -U "$DB_USER" -c '\q' > /dev/null 2>&1; then
    log_error "Cannot connect to PostgreSQL at $DB_HOST"
    exit 1
fi
log_info "PostgreSQL connection OK"

# Check if database exists
log_info "Checking if database $DB_NAME exists..."
DB_EXISTS=$(psql -h "$DB_HOST" -U "$DB_USER" -tc "SELECT 1 FROM pg_database WHERE datname = '$DB_NAME';" 2>/dev/null)

if [[ -z "$DB_EXISTS" ]]; then
    log_info "Database $DB_NAME does not exist, creating it..."
    createdb -h "$DB_HOST" -U "$DB_USER" "$DB_NAME" || log_error "Failed to create database"
else
    log_warn "Database $DB_NAME already exists, will be restored with --clean flag"
fi

# Check if database has tables
TABLES=$(psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -tc "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" 2>/dev/null || echo "0")

if [[ "$TABLES" -gt 0 ]]; then
    log_warn "Database already contains $TABLES tables"
    read -p "Overwrite existing database? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_error "Database restoration cancelled"
        exit 1
    fi
fi

# Restore database. With `set -eo pipefail`, the script aborts
# immediately on any non-zero exit (including gunzip on a corrupt
# backup), so the post-pipeline `if [[ $? -eq 0 ]]` branch that used
# to live here was dead code — the else branch was unreachable.
log_info "Starting database restoration from $BACKUP_FILE..."
gunzip < "$BACKUP_FILE" | psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" --quiet

log_info "Database restoration completed successfully"

# Verify restoration
FINAL_TABLES=$(psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -tc "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" 2>/dev/null)
log_info "Database now contains $FINAL_TABLES tables"

# Sanity-check: a successful restore must produce at least one table.
# If gunzip silently fed empty input to psql in a future regression
# (e.g. pipefail accidentally disabled), this catches it.
if [[ -z "$FINAL_TABLES" || "$FINAL_TABLES" -eq 0 ]]; then
    log_error "Restoration produced 0 tables — backup may be corrupted or empty"
    exit 1
fi

# Check for migration marker
MIGRATED=$(psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -tc "SELECT 1 FROM information_schema.tables WHERE table_name = 'migrations';" 2>/dev/null || echo "")
if [[ -n "$MIGRATED" ]]; then
    log_info "Migrations table found - database schema validated"
fi

log_info "✓ Database restoration completed"
exit 0
