#!/bin/bash
# ============================================================================
# Migration Verification Script for SIMPEL
# Description: Comprehensive data integrity and migration verification
# Author: SIMPelv2 Team
# Created: 2026-02-11
# ============================================================================

set -euo pipefail

DB_NAME="${DB_NAME:-simpelv2}"
DB_USER="${DB_USER:-simpelv2}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
REPORT_FILE="${REPORT_FILE:-/tmp/simpelv2_migration_report_$(date +%Y%m%d_%H%M%S).txt}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1" | tee -a "$REPORT_FILE"
    ((PASSED_CHECKS++))
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$REPORT_FILE"
    ((FAILED_CHECKS++))
}

exec_sql() {
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "$1" 2>/dev/null || echo "0"
}

# Run checks
echo "Starting migration verification..." | tee "$REPORT_FILE"

((TOTAL_CHECKS++))
SCHEMAS=$(exec_sql "SELECT COUNT(*) FROM information_schema.schemata WHERE schema_name IN ('perlengkapan', 'integrasi')")
[ "$SCHEMAS" -eq 2 ] && log_pass "Schemas exist" || log_error "Missing schemas"

((TOTAL_CHECKS++))
TABLES=$(exec_sql "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'perlengkapan'")
[ "$TABLES" -ge 30 ] && log_pass "Tables exist ($TABLES)" || log_error "Insufficient tables ($TABLES)"

((TOTAL_CHECKS++))
VIEWS=$(exec_sql "SELECT COUNT(*) FROM information_schema.views WHERE table_schema = 'perlengkapan'")
[ "$VIEWS" -ge 5 ] && log_pass "Views exist ($VIEWS)" || log_error "Missing views ($VIEWS)"

((TOTAL_CHECKS++))
INDEXES=$(exec_sql "SELECT COUNT(*) FROM pg_indexes WHERE schemaname = 'perlengkapan'")
[ "$INDEXES" -ge 50 ] && log_pass "Indexes exist ($INDEXES)" || log_error "Insufficient indexes ($INDEXES)"

echo ""
echo "Total: $TOTAL_CHECKS | Passed: $PASSED_CHECKS | Failed: $FAILED_CHECKS" | tee -a "$REPORT_FILE"
[ "$FAILED_CHECKS" -eq 0 ] && echo -e "${GREEN}✅ PASSED${NC}" || echo -e "${RED}❌ FAILED${NC}"
