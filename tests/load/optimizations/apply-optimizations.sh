#!/bin/bash

# Apply performance optimizations based on load test results
# Usage: ./tests/load/optimizations/apply-optimizations.sh [database_url]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
DATABASE_URL="${1:-${DATABASE_URL:-postgres://simpelv2:password@localhost:5432/perlengkapan}}"
OPTIMIZATIONS_DIR="tests/load/optimizations"

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Applying Performance Optimizations${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check if psql is available
if ! command -v psql &> /dev/null; then
    echo -e "${RED}Error: psql is not installed${NC}"
    echo "Install PostgreSQL client:"
    echo "  Ubuntu/Debian: sudo apt-get install postgresql-client"
    echo "  macOS: brew install postgresql"
    exit 1
fi

# Test database connection
echo -e "${BLUE}Testing database connection...${NC}"
if ! psql "$DATABASE_URL" -c "SELECT 1" &> /dev/null; then
    echo -e "${RED}Error: Cannot connect to database${NC}"
    echo "Database URL: $DATABASE_URL"
    exit 1
fi
echo -e "${GREEN}✓ Database connection successful${NC}"
echo ""

# Function to apply SQL file
apply_sql() {
    local sql_file=$1
    local description=$2

    echo -e "${BLUE}Applying: $description${NC}"
    echo "  File: $sql_file"

    if psql "$DATABASE_URL" -f "$sql_file" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Successfully applied${NC}"
        return 0
    else
        echo -e "${RED}✗ Failed to apply${NC}"
        echo "  Run manually: psql \"$DATABASE_URL\" -f \"$sql_file\""
        return 1
    fi
}

# Backup database (optional but recommended)
echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}Database Backup${NC}"
echo -e "${YELLOW}========================================${NC}"
read -p "Create database backup before optimization? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    BACKUP_FILE="perlengkapan_backup_$(date +%Y%m%d_%H%M%S).sql"
    echo -e "${BLUE}Creating backup: $BACKUP_FILE${NC}"
    pg_dump "$DATABASE_URL" > "$BACKUP_FILE"
    echo -e "${GREEN}✓ Backup created${NC}"
    echo ""
fi

# Apply optimizations
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Applying Optimizations${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

total_optimizations=0
applied_optimizations=0
failed_optimizations=0

# 1. Add performance indexes
total_optimizations=$((total_optimizations + 1))
if apply_sql "$OPTIMIZATIONS_DIR/001_add_performance_indexes.sql" "Performance Indexes"; then
    applied_optimizations=$((applied_optimizations + 1))
else
    failed_optimizations=$((failed_optimizations + 1))
fi
echo ""

# 2. Create materialized views
total_optimizations=$((total_optimizations + 1))
if apply_sql "$OPTIMIZATIONS_DIR/002_create_materialized_views.sql" "Materialized Views"; then
    applied_optimizations=$((applied_optimizations + 1))
else
    failed_optimizations=$((failed_optimizations + 1))
fi
echo ""

# Verify optimizations
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Verification${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

echo -e "${BLUE}Checking indexes...${NC}"
psql "$DATABASE_URL" -c "
SELECT
    schemaname,
    tablename,
    COUNT(*) as index_count
FROM pg_indexes
WHERE schemaname = 'perlengkapan'
GROUP BY schemaname, tablename
ORDER BY index_count DESC;
"
echo ""

echo -e "${BLUE}Checking materialized views...${NC}"
psql "$DATABASE_URL" -c "
SELECT
    schemaname,
    matviewname,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||matviewname)) as size
FROM pg_matviews
WHERE schemaname = 'perlengkapan';
"
echo ""

# Summary
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Summary${NC}"
echo -e "${GREEN}========================================${NC}"
echo "Total optimizations: $total_optimizations"
echo -e "${GREEN}Applied: $applied_optimizations${NC}"
if [ $failed_optimizations -gt 0 ]; then
    echo -e "${RED}Failed: $failed_optimizations${NC}"
fi
echo ""

# Recommendations
echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}Next Steps${NC}"
echo -e "${YELLOW}========================================${NC}"
echo ""
echo "1. Monitor query performance:"
echo "   psql \"$DATABASE_URL\" -c \"SELECT * FROM pg_stat_statements ORDER BY total_exec_time DESC LIMIT 10;\""
echo ""
echo "2. Check index usage:"
echo "   psql \"$DATABASE_URL\" -c \"SELECT * FROM pg_stat_user_indexes WHERE schemaname = 'perlengkapan' ORDER BY idx_scan DESC;\""
echo ""
echo "3. Refresh materialized views:"
echo "   psql \"$DATABASE_URL\" -c \"SELECT perlengkapan.refresh_all_dashboard_views();\""
echo ""
echo "4. Re-run load tests to verify improvements:"
echo "   ./tests/load/run-k8s-load-tests.sh simpelv2-staging all"
echo ""
echo "5. Compare results with baseline:"
echo "   ./tests/load/analyze-results.sh"
echo ""

# Exit with appropriate code
if [ $failed_optimizations -gt 0 ]; then
    exit 1
else
    exit 0
fi
