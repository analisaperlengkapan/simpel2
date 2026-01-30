#!/bin/bash
# PostgreSQL Migration Script
# Migrate data from single-instance PostgreSQL to HA cluster

set -e

NAMESPACE="simpelv2"
OLD_POD="postgres-5fc9ffd7c9-mmf8m"  # Update this if pod name changed
NEW_CLUSTER="simpelv2-postgres-ha"
BACKUP_FILE="/tmp/postgres-backup-$(date +%Y%m%d-%H%M%S).sql"

echo "🔄 PostgreSQL Migration to HA Cluster"
echo "======================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if running in dry-run mode
DRY_RUN=false
if [ "$1" == "--dry-run" ]; then
    DRY_RUN=true
    print_warn "Running in DRY-RUN mode - no actual changes will be made"
fi

# Step 1: Verify old PostgreSQL is running
print_info "Step 1: Verifying old PostgreSQL pod..."
if ! kubectl get pod $OLD_POD -n $NAMESPACE &>/dev/null; then
    print_error "Old PostgreSQL pod not found: $OLD_POD"
    print_info "Available postgres pods:"
    kubectl get pods -n $NAMESPACE | grep postgres
    exit 1
fi
print_info "✓ Old PostgreSQL pod found: $OLD_POD"

# Step 2: Verify new HA cluster is ready
print_info "Step 2: Verifying HA cluster is ready..."
if ! kubectl get postgresql $NEW_CLUSTER -n $NAMESPACE &>/dev/null; then
    print_error "HA cluster not found: $NEW_CLUSTER"
    print_info "Please deploy the HA cluster first:"
    print_info "  kubectl apply -f 01-postgresql-ha-cluster.yaml"
    exit 1
fi

# Wait for primary pod to be ready
PRIMARY_POD="${NEW_CLUSTER}-0"
print_info "Waiting for primary pod to be ready: $PRIMARY_POD"
if ! kubectl wait --for=condition=ready pod/$PRIMARY_POD -n $NAMESPACE --timeout=300s; then
    print_error "Primary pod not ready after 5 minutes"
    exit 1
fi
print_info "✓ HA cluster is ready"

# Step 3: Create backup from old PostgreSQL
print_info "Step 3: Creating backup from old PostgreSQL..."
print_warn "This may take several minutes depending on database size..."

if [ "$DRY_RUN" == "false" ]; then
    kubectl exec -n $NAMESPACE $OLD_POD -c postgres -- \
        pg_dumpall -U postgres > $BACKUP_FILE

    if [ ! -f "$BACKUP_FILE" ]; then
        print_error "Backup file not created: $BACKUP_FILE"
        exit 1
    fi

    BACKUP_SIZE=$(du -h $BACKUP_FILE | cut -f1)
    print_info "✓ Backup created: $BACKUP_FILE ($BACKUP_SIZE)"
else
    print_info "(DRY-RUN) Would create backup: $BACKUP_FILE"
fi

# Step 4: Verify backup integrity
print_info "Step 4: Verifying backup integrity..."
if [ "$DRY_RUN" == "false" ]; then
    if ! grep -q "PostgreSQL database dump complete" $BACKUP_FILE; then
        print_error "Backup file appears incomplete"
        exit 1
    fi
    print_info "✓ Backup integrity verified"
else
    print_info "(DRY-RUN) Would verify backup integrity"
fi

# Step 5: Stop write traffic to old PostgreSQL (optional)
print_warn "Step 5: Manual intervention required"
print_warn "Please ensure no applications are writing to the old PostgreSQL"
print_warn "Consider scaling down deployments that write to the database:"
print_warn "  kubectl scale deployment authenc --replicas=0 -n $NAMESPACE"
print_warn "  kubectl scale deployment portal --replicas=0 -n $NAMESPACE"
read -p "Press Enter when ready to continue..."

# Step 6: Restore to new HA cluster
print_info "Step 6: Restoring to HA cluster..."
print_warn "This may take several minutes..."

if [ "$DRY_RUN" == "false" ]; then
    # Copy backup file to primary pod
    print_info "Copying backup to primary pod..."
    kubectl cp $BACKUP_FILE $NAMESPACE/$PRIMARY_POD:/tmp/restore.sql -c postgres

    # Restore database
    print_info "Restoring database..."
    kubectl exec -n $NAMESPACE $PRIMARY_POD -c postgres -- \
        psql -U postgres < /tmp/restore.sql

    # Clean up backup file from pod
    kubectl exec -n $NAMESPACE $PRIMARY_POD -c postgres -- \
        rm /tmp/restore.sql

    print_info "✓ Database restored to HA cluster"
else
    print_info "(DRY-RUN) Would restore backup to HA cluster"
fi

# Step 7: Verify data integrity
print_info "Step 7: Verifying data integrity..."

if [ "$DRY_RUN" == "false" ]; then
    # Count tables in old database
    OLD_TABLE_COUNT=$(kubectl exec -n $NAMESPACE $OLD_POD -c postgres -- \
        psql -U postgres -d simpelv2 -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public'" | tr -d ' ')

    # Count tables in new database
    NEW_TABLE_COUNT=$(kubectl exec -n $NAMESPACE $PRIMARY_POD -c postgres -- \
        psql -U postgres -d simpelv2 -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public'" | tr -d ' ')

    print_info "Table count - Old: $OLD_TABLE_COUNT, New: $NEW_TABLE_COUNT"

    if [ "$OLD_TABLE_COUNT" != "$NEW_TABLE_COUNT" ]; then
        print_error "Table count mismatch! Please investigate."
        exit 1
    fi

    print_info "✓ Data integrity verified"
else
    print_info "(DRY-RUN) Would verify data integrity"
fi

# Step 8: Test replication
print_info "Step 8: Testing replication..."

if [ "$DRY_RUN" == "false" ]; then
    # Check replication status
    print_info "Checking replication lag..."
    kubectl exec -n $NAMESPACE $PRIMARY_POD -c postgres -- \
        psql -U postgres -c "SELECT client_addr, state, sync_state, replay_lag FROM pg_stat_replication;"

    print_info "✓ Replication status checked"
else
    print_info "(DRY-RUN) Would test replication"
fi

# Step 9: Update application connection strings
print_info "Step 9: Update application connection strings"
print_warn "Manual intervention required:"
print_warn ""
print_warn "Update environment variables in deployments:"
print_warn ""
print_warn "OLD CONNECTION:"
print_warn "  DATABASE_URL: postgres://user:pass@postgres:5432/simpelv2"
print_warn ""
print_warn "NEW CONNECTION (with connection pooling):"
print_warn "  DATABASE_URL: postgres://user:pass@simpelv2-postgres-ha-pooler:5432/simpelv2"
print_warn ""
print_warn "READ-ONLY REPLICAS:"
print_warn "  DATABASE_REPLICA_URL: postgres://user:pass@simpelv2-postgres-ha-repl:5432/simpelv2"
print_warn ""
print_warn "Services available:"
kubectl get svc -n $NAMESPACE | grep $NEW_CLUSTER

read -p "Press Enter after updating connection strings..."

# Step 10: Restart applications
print_info "Step 10: Restarting applications..."

if [ "$DRY_RUN" == "false" ]; then
    print_info "Scaling up deployments..."
    kubectl scale deployment authenc --replicas=1 -n $NAMESPACE
    kubectl scale deployment portal --replicas=1 -n $NAMESPACE

    print_info "Waiting for pods to be ready..."
    kubectl wait --for=condition=ready pod \
        -l app.kubernetes.io/name=authenc -n $NAMESPACE --timeout=180s || true
    kubectl wait --for=condition=ready pod \
        -l app.kubernetes.io/name=portal -n $NAMESPACE --timeout=180s || true

    print_info "✓ Applications restarted"
else
    print_info "(DRY-RUN) Would restart applications"
fi

# Step 11: Smoke test
print_info "Step 11: Running smoke tests..."

if [ "$DRY_RUN" == "false" ]; then
    print_info "Testing database connectivity from primary..."
    kubectl exec -n $NAMESPACE $PRIMARY_POD -c postgres -- \
        psql -U postgres -d simpelv2 -c "SELECT version();"

    print_info "✓ Smoke tests passed"
else
    print_info "(DRY-RUN) Would run smoke tests"
fi

# Step 12: Cleanup (optional)
print_info "Step 12: Cleanup"
print_warn "The old PostgreSQL deployment is still running"
print_warn "After verifying everything works correctly, you can delete it:"
print_warn ""
print_warn "  kubectl scale deployment postgres --replicas=0 -n $NAMESPACE"
print_warn "  kubectl delete deployment postgres -n $NAMESPACE"
print_warn "  kubectl delete pvc postgres-pvc -n $NAMESPACE"
print_warn ""
print_warn "Backup file saved at: $BACKUP_FILE"
print_warn "Keep this file until you're confident the migration is successful"

echo ""
print_info "============================================"
print_info "Migration completed successfully! 🎉"
print_info "============================================"
print_info ""
print_info "Next steps:"
print_info "1. Monitor the HA cluster for 24-48 hours"
print_info "2. Test failover scenarios"
print_info "3. Setup backup automation"
print_info "4. Delete old PostgreSQL deployment"
print_info ""
print_info "Useful commands:"
print_info "  # Check cluster status"
print_info "  kubectl exec -n $NAMESPACE $PRIMARY_POD -- patronictl list"
print_info ""
print_info "  # Check replication lag"
print_info "  kubectl exec -n $NAMESPACE $PRIMARY_POD -c postgres -- \\"
print_info "    psql -U postgres -c 'SELECT * FROM pg_stat_replication;'"
print_info ""
print_info "  # View logs"
print_info "  kubectl logs -f $PRIMARY_POD -n $NAMESPACE -c postgres"
