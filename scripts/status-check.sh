#!/bin/bash
# Quick Status Check - Authenc & Secreton
# Usage: ./scripts/status-check.sh

echo "╔════════════════════════════════════════════════════╗"
echo "║  SIMPelv2 - Quick Status Check                     ║"
echo "╚════════════════════════════════════════════════════╝"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if containers are running
echo "🐳 Container Status:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
docker ps --filter 'name=authenc\|secreton' --format "{{.Names}}\t{{.Status}}" 2>/dev/null || echo "No containers running"
echo ""

# Health Checks
echo "🏥 Health Checks:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Authenc Health
if authenc_health=$(curl -s -m 3 http://localhost:8088/health 2>/dev/null); then
    status=$(echo "$authenc_health" | jq -r '.status' 2>/dev/null)
    if [ "$status" = "healthy" ]; then
        echo -e "Authenc:   ${GREEN}✓ Healthy${NC}"
    else
        echo -e "Authenc:   ${YELLOW}⚠ Unknown${NC}"
    fi
else
    echo -e "Authenc:   ${RED}✗ Unreachable${NC}"
fi

# Secreton Health
if secreton_health=$(curl -s -m 3 http://localhost:8200/health 2>/dev/null); then
    status=$(echo "$secreton_health" | jq -r '.status' 2>/dev/null)
    if [ "$status" = "healthy" ]; then
        echo -e "Secreton:  ${GREEN}✓ Healthy${NC}"
    else
        echo -e "Secreton:  ${YELLOW}⚠ Unknown${NC}"
    fi
else
    echo -e "Secreton:  ${RED}✗ Unreachable${NC}"
fi
echo ""

# Database Connectivity
echo "🗄️  Database Connectivity:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if authenc_ready=$(curl -s -m 3 http://localhost:8088/ready 2>/dev/null); then
    db_status=$(echo "$authenc_ready" | jq -r '.database' 2>/dev/null)
    if [ "$db_status" = "connected" ]; then
        echo -e "Authenc DB:   ${GREEN}✓ Connected${NC}"
    else
        echo -e "Authenc DB:   ${RED}✗ Disconnected${NC}"
    fi
else
    echo -e "Authenc DB:   ${RED}✗ Cannot check${NC}"
fi

if docker ps | grep -q secreton-postgres; then
    echo -e "Secreton DB:  ${GREEN}✓ Running${NC}"
else
    echo -e "Secreton DB:  ${RED}✗ Not running${NC}"
fi
echo ""

# Vault Status
echo "🔐 Vault Status:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if seal_status=$(curl -s -m 3 http://localhost:8200/v1/sys/seal-status 2>/dev/null); then
    sealed=$(echo "$seal_status" | jq -r '.sealed' 2>/dev/null)
    initialized=$(echo "$seal_status" | jq -r '.initialized' 2>/dev/null)

    if [ "$initialized" = "true" ]; then
        echo -e "Initialized:  ${GREEN}✓ Yes${NC}"
    else
        echo -e "Initialized:  ${YELLOW}⚠ Not yet${NC}"
    fi

    if [ "$sealed" = "false" ]; then
        echo -e "Sealed:       ${GREEN}✓ Unsealed${NC}"
    else
        echo -e "Sealed:       ${YELLOW}⚠ Sealed${NC}"
    fi
else
    echo -e "Vault:        ${RED}✗ Cannot check${NC}"
fi
echo ""

# Quick Access URLs
echo "🌐 Quick Access:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Authenc Health:    http://localhost:8088/health"
echo "Authenc Ready:     http://localhost:8088/ready"
echo "Authenc Metrics:   http://localhost:8088/metrics"
echo ""
echo "Secreton Health:   http://localhost:8200/health"
echo "Secreton Version:  http://localhost:8200/version"
echo "Secreton Seal:     http://localhost:8200/v1/sys/seal-status"
echo ""

# Useful Commands
echo "🔧 Useful Commands:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "View Authenc logs:   docker logs authenc -f"
echo "View Secreton logs:  docker logs secreton-server -f"
echo "Restart Authenc:     cd infra/authenc && docker compose restart"
echo "Restart Secreton:    cd infra/secreton && docker compose restart"
echo "Full test:           ./scripts/comprehensive-test.sh"
echo ""
echo "═══════════════════════════════════════════════════"
