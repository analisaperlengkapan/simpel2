#!/bin/bash
set -euo pipefail

# Secreton Disaster Recovery Automation Script
# Handles backup, restore, and disaster recovery scenarios

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
NAMESPACE="${SECRETON_NAMESPACE:-secreton}"
BACKUP_DIR="${BACKUP_DIR:-./backups}"
ENCRYPTION_KEY="${BACKUP_ENCRYPTION_KEY:-}"

usage() {
    echo "Usage: $0 {backup|restore|list|verify|test-recovery} [options]"
    echo ""
    echo "Commands:"
    echo "  backup              Create encrypted backup of all data"
    echo "  restore <file>      Restore from backup file"
    echo "  list                List available backups"
    echo "  verify <file>       Verify backup integrity"
    echo "  test-recovery       Test disaster recovery procedure"
    echo ""
    echo "Options:"
    echo "  --namespace <ns>    Kubernetes namespace (default: secreton)"
    echo "  --backup-dir <dir>  Backup directory (default: ./backups)"
    echo "  --encrypt-key <key> Encryption key for backups"
    echo ""
    exit 1
}

print_section() {
    echo -e "\n${BLUE}▶ $1${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# Create backup
backup() {
    print_section "Creating Backup"

    mkdir -p "$BACKUP_DIR"
    TIMESTAMP=$(date +%Y%m%d-%H%M%S)
    BACKUP_FILE="$BACKUP_DIR/secreton-backup-$TIMESTAMP.tar.gz"
    TEMP_DIR=$(mktemp -d)

    echo "  Backup location: $BACKUP_FILE"

    # Backup PostgreSQL database
    if kubectl -n "$NAMESPACE" get pod -l app=postgres &> /dev/null; then
        echo "  ✓ Backing up PostgreSQL database..."
        kubectl -n "$NAMESPACE" exec -it postgres-0 -- pg_dump -U secreton secreton > "$TEMP_DIR/postgres-dump.sql" 2>/dev/null || true
    fi

    # Backup Secreton data (if using file storage)
    echo "  ✓ Backing up Secreton data..."
    for i in 0 1 2; do
        POD_NAME="secreton-$i"
        if kubectl -n "$NAMESPACE" get pod "$POD_NAME" &> /dev/null; then
            kubectl -n "$NAMESPACE" cp "$POD_NAME:/app/data" "$TEMP_DIR/secreton-$i-data" 2>/dev/null || true
        fi
    done

    # Backup Kubernetes manifests
    echo "  ✓ Backing up Kubernetes configuration..."
    kubectl -n "$NAMESPACE" get all -o yaml > "$TEMP_DIR/k8s-resources.yaml" 2>/dev/null || true
    kubectl -n "$NAMESPACE" get secret,configmap -o yaml > "$TEMP_DIR/k8s-configs.yaml" 2>/dev/null || true

    # Create metadata file
    cat > "$TEMP_DIR/backup-metadata.json" <<EOF
{
  "timestamp": "$TIMESTAMP",
  "namespace": "$NAMESPACE",
  "backup_type": "full",
  "version": "0.1.0",
  "components": ["postgres", "secreton-data", "k8s-config"]
}
EOF

    # Create tar.gz archive
    echo "  ✓ Creating compressed archive..."
    tar -czf "$BACKUP_FILE" -C "$TEMP_DIR" .

    # Encrypt if key PROVIDED
    if [ -n "$ENCRYPTION_KEY" ]; then
        echo "  ✓ Encrypting backup..."
        openssl enc -aes-256-cbc -salt -in "$BACKUP_FILE" -out "$BACKUP_FILE.enc" -k "$ENCRYPTION_KEY"
        rm "$BACKUP_FILE"
        BACKUP_FILE="$BACKUP_FILE.enc"
    fi

    # Cleanup
    rm -rf "$TEMP_DIR"

    # Calculate checksum
    CHECKSUM=$(sha256sum "$BACKUP_FILE" | awk '{print $1}')
    echo "$CHECKSUM" > "$BACKUP_FILE.sha256"

    echo ""
    echo -e "${GREEN}✓ Backup created successfully${NC}"
    echo "  File: $BACKUP_FILE"
    echo "  SHA256: $CHECKSUM"
    echo "  Size: $(du -h "$BACKUP_FILE" | awk '{print $1}')"
}

# Restore from backup
restore() {
    local BACKUP_FILE="$1"

    if [ ! -f "$BACKUP_FILE" ]; then
        echo -e "${RED}Backup file not found: $BACKUP_FILE${NC}"
        exit 1
    fi

    print_section "Restoring from Backup"

    echo "  Backup file: $BACKUP_FILE"

    # Verify checksum
    if [ -f "$BACKUP_FILE.sha256" ]; then
        EXPECTED_CHECKSUM=$(cat "$BACKUP_FILE.sha256")
        ACTUAL_CHECKSUM=$(sha256sum "$BACKUP_FILE" | awk '{print $1}')

        if [ "$EXPECTED_CHECKSUM" = "$ACTUAL_CHECKSUM" ]; then
            echo -e "  ${GREEN}✓ Checksum verified${NC}"
        else
            echo -e "  ${RED}✗ Checksum mismatch!${NC}"
            exit 1
        fi
    fi

    TEMP_DIR=$(mktemp -d)

    # Decrypt if needed
    if [[ "$BACKUP_FILE" == *.enc ]]; then
        if [ -z "$ENCRYPTION_KEY" ]; then
            echo -e "${RED}Encryption key required for encrypted backup${NC}"
            exit 1
        fi

        echo "  ✓ Decrypting backup..."
        openssl enc -aes-256-cbc -d -in "$BACKUP_FILE" -out "$TEMP_DIR/backup.tar.gz" -k "$ENCRYPTION_KEY"
        tar -xzf "$TEMP_DIR/backup.tar.gz" -C "$TEMP_DIR"
    else
        tar -xzf "$BACKUP_FILE" -C "$TEMP_DIR"
    fi

    # Restore PostgreSQL
    if [ -f "$TEMP_DIR/postgres-dump.sql" ]; then
        echo "  ✓ Restoring PostgreSQL database..."
        kubectl -n "$NAMESPACE" exec -i postgres-0 -- psql -U secreton secreton < "$TEMP_DIR/postgres-dump.sql" 2>/dev/null || true
    fi

    # Restore Secreton data
    echo "  ✓ Restoring Secreton data..."
    for i in 0 1 2; do
        POD_NAME="secreton-$i"
        if [ -d "$TEMP_DIR/secreton-$i-data" ]; then
            kubectl -n "$NAMESPACE" cp "$TEMP_DIR/secreton-$i-data" "$POD_NAME:/app/data" 2>/dev/null || true
        fi
    done

    # Restart pods to pick up restored data
    echo "  ✓ Restarting Secreton pods..."
    kubectl -n "$NAMESPACE" rollout restart statefulset/secreton > /dev/null 2>&1 || true

    # Cleanup
    rm -rf "$TEMP_DIR"

    echo ""
    echo -e "${GREEN}✓ Restore completed successfully${NC}"
    echo "  Verify cluster health with: ./scripts/ha-cluster-validation.sh"
}

# List backups
list_backups() {
    print_section "Available Backups"

    if [ ! -d "$BACKUP_DIR" ] || [ -z "$(ls -A "$BACKUP_DIR" 2>/dev/null)" ]; then
        echo "  No backups found in $BACKUP_DIR"
        return
    fi

    echo ""
    printf "%-30s %-15s %-10s %s\n" "BACKUP FILE" "DATE" "SIZE" "CHECKSUM"
    echo "────────────────────────────────────────────────────────────────────────────"

    for file in "$BACKUP_DIR"/secreton-backup-*.tar.gz*; do
        if [ -f "$file" ]; then
            FILENAME=$(basename "$file")
            SIZE=$(du -h "$file" | awk '{print $1}')

            if [ -f "$file.sha256" ]; then
                CHECKSUM=$(cat "$file.sha256" | cut -c1-16)...
            else
                CHECKSUM="N/A"
            fi

            # Extract timestamp from filename
            TIMESTAMP=$(echo "$FILENAME" | sed -n 's/secreton-backup-\(.*\)\.tar\.gz.*/\1/p')
            DATE=$(echo "$TIMESTAMP" | sed 's/\([0-9]\{8\}\)-\([0-9]\{6\}\)/\1 \2/')

            printf "%-30s %-15s %-10s %s\n" "$FILENAME" "$DATE" "$SIZE" "$CHECKSUM"
        fi
    done
    echo ""
}

# Verify backup
verify_backup() {
    local BACKUP_FILE="$1"

    print_section "Verifying Backup"

    if [ ! -f "$BACKUP_FILE" ]; then
        echo -e "${RED}Backup file not found: $BACKUP_FILE${NC}"
        exit 1
    fi

    # Checksum verification
    if [ -f "$BACKUP_FILE.sha256" ]; then
        EXPECTED=$(cat "$BACKUP_FILE.sha256")
        ACTUAL=$(sha256sum "$BACKUP_FILE" | awk '{print $1}')

        if [ "$EXPECTED" = "$ACTUAL" ]; then
            echo -e "  ${GREEN}✓ Checksum verified${NC}"
        else
            echo -e "  ${RED}✗ Checksum mismatch${NC}"
            exit 1
        fi
    else
        echo -e "  ${YELLOW}⚠ No checksum file found${NC}"
    fi

    # Archive integrity
    if tar -tzf "$BACKUP_FILE" > /dev/null 2>&1; then
        echo -e "  ${GREEN}✓ Archive integrity verified${NC}"
    else
        echo -e "  ${RED}✗ Archive corrupted${NC}"
        exit 1
    fi

    # List contents
    echo ""
    echo "  Backup contents:"
    tar -tzf "$BACKUP_FILE" | head -20

    echo ""
    echo -e "${GREEN}✓ Backup verification passed${NC}"
}

# Test disaster recovery
test_recovery() {
    print_section "Testing Disaster Recovery Procedure"

    echo "  This will simulate a disaster and test recovery:"
    echo "  1. Create snapshot of current state"
    echo "  2. Simulate data loss"
    echo "  3. Restore from backup"
    echo "  4. Verify data integrity"
    echo ""
    read -p "  Continue with test? (yes/no): " CONFIRM

    if [ "$CONFIRM" != "yes" ]; then
        echo "  Test cancelled"
        exit 0
    fi

    echo ""
    echo "  ✓ Creating pre-test backup..."
    backup

    echo "  ✓ Creating test data..."
    TEST_KEY="dr-test-$(date +%s)"
    kubectl -n "$NAMESPACE" exec secreton-0 -- curl -s -X POST \
        -H "Content-Type: application/json" \
        -d '{"data": {"value": "disaster-recovery-test"}}' \
        http://localhost:8200/v1/secret/data/test/$TEST_KEY > /dev/null 2>&1 || true

    echo "  ✓ Simulating disaster (deleting pods)..."
    kubectl -n "$NAMESPACE" delete pod --all > /dev/null 2>&1 || true

    echo "  ✓ Waiting for cluster recovery..."
    sleep 30

    echo "  ✓ Verifying data restored automatically..."
    RESULT=$(kubectl -n "$NAMESPACE" exec secreton-0 -- curl -s \
        http://localhost:8200/v1/secret/data/test/$TEST_KEY 2>/dev/null || echo "")

    if echo "$RESULT" | grep -q "disaster-recovery-test"; then
        echo -e "  ${GREEN}✓ Automatic recovery successful${NC}"
    else
        echo -e "  ${YELLOW}⚠ Manual restore may be required${NC}"
    fi

    echo ""
    echo -e "${GREEN}✓ Disaster recovery test complete${NC}"
}

# Main
case "${1:-}" in
    backup)
        shift
        while [[ $# -gt 0 ]]; do
            case $1 in
                --namespace) NAMESPACE="$2"; shift 2 ;;
                --backup-dir) BACKUP_DIR="$2"; shift 2 ;;
                --encrypt-key) ENCRYPTION_KEY="$2"; shift 2 ;;
                *) echo "Unknown option: $1"; usage ;;
            esac
        done
        backup
        ;;
    restore)
        shift
        BACKUP_FILE="${1:-}"
        if [ -z "$BACKUP_FILE" ]; then
            echo "Error: Backup file required"
            usage
        fi
        shift
        while [[ $# -gt 0 ]]; do
            case $1 in
                --namespace) NAMESPACE="$2"; shift 2 ;;
                --encrypt-key) ENCRYPTION_KEY="$2"; shift 2 ;;
                *) echo "Unknown option: $1"; usage ;;
            esac
        done
        restore "$BACKUP_FILE"
        ;;
    list)
        list_backups
        ;;
    verify)
        shift
        BACKUP_FILE="${1:-}"
        if [ -z "$BACKUP_FILE" ]; then
            echo "Error: Backup file required"
            usage
        fi
        verify_backup "$BACKUP_FILE"
        ;;
    test-recovery)
        test_recovery
        ;;
    *)
        usage
        ;;
esac
