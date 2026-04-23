#!/bin/bash
# SIMPEL Kubernetes Deployment Script
# Usage: ./deploy.sh [staging|production] [apply|delete|diff]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
K8S_DIR="${SCRIPT_DIR}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
ENVIRONMENT="${1:-staging}"
ACTION="${2:-apply}"
DRY_RUN="${3:-}"

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(staging|production)$ ]]; then
    echo -e "${RED}Error: Invalid environment '$ENVIRONMENT'${NC}"
    echo "Usage: $0 [staging|production] [apply|delete|diff]"
    exit 1
fi

# Validate action
if [[ ! "$ACTION" =~ ^(apply|delete|diff)$ ]]; then
    echo -e "${RED}Error: Invalid action '$ACTION'${NC}"
    echo "Usage: $0 [staging|production] [apply|delete|diff]"
    exit 1
fi

OVERLAY_DIR="${K8S_DIR}/overlays/${ENVIRONMENT}"
NAMESPACE="simpelv2-${ENVIRONMENT}"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  SIMPEL Kubernetes Deployment                            ║${NC}"
echo -e "${BLUE}║  Environment: ${GREEN}${ENVIRONMENT}${BLUE}                                      ║${NC}"
echo -e "${BLUE}║  Action: ${YELLOW}${ACTION}${BLUE}                                             ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"

# Check prerequisites
echo -e "\n${YELLOW}Checking prerequisites...${NC}"

if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}Error: kubectl not found${NC}"
    exit 1
fi

if ! kubectl cluster-info &> /dev/null; then
    echo -e "${RED}Error: Cannot connect to Kubernetes cluster${NC}"
    exit 1
fi

echo -e "${GREEN}✓ kubectl configured${NC}"

# Validate kustomization
echo -e "\n${YELLOW}Validating kustomization...${NC}"
if ! kubectl kustomize "${OVERLAY_DIR}" > /dev/null 2>&1; then
    echo -e "${RED}Error: Kustomization validation failed${NC}"
    kubectl kustomize "${OVERLAY_DIR}" 2>&1
    exit 1
fi
echo -e "${GREEN}✓ Kustomization valid${NC}"

# Execute action
case "$ACTION" in
    apply)
        echo -e "\n${YELLOW}Applying ${ENVIRONMENT} environment...${NC}"

        # Create namespace first if it doesn't exist
        kubectl apply -f "${OVERLAY_DIR}/namespace.yaml" 2>/dev/null || true

        # Apply with prune for GitOps-style management
        if [[ -n "$DRY_RUN" && "$DRY_RUN" == "--dry-run" ]]; then
            kubectl apply -k "${OVERLAY_DIR}" --dry-run=client
        else
            kubectl apply -k "${OVERLAY_DIR}"
        fi

        echo -e "\n${GREEN}✓ Deployment applied successfully${NC}"

        # Warn about CHANGEME secrets
        SECRETS_FILE="${OVERLAY_DIR}/${ENVIRONMENT}-secrets.yaml"
        echo ""
        echo -e "${RED}========================================================================${NC}"
        echo -e "${RED}  WARNING: kustomize deployed CHANGEME placeholder secrets.${NC}"
        echo -e "${RED}  Pods WILL CrashLoopBackOff until you apply real secrets:${NC}"
        echo ""
        echo -e "${YELLOW}    kubectl apply -f ${SECRETS_FILE} -n ${NAMESPACE}${NC}"
        echo ""
        echo -e "${YELLOW}  If you haven't created the secrets file yet:${NC}"
        echo -e "${YELLOW}    cp ${OVERLAY_DIR}/staging-secrets.example.yaml ${SECRETS_FILE}${NC}"
        echo -e "${YELLOW}    \$EDITOR ${SECRETS_FILE}${NC}"
        echo -e "${RED}========================================================================${NC}"

        # Wait for rollout
        echo -e "\n${YELLOW}Waiting for rollouts to complete...${NC}"
        kubectl rollout status deployment --timeout=300s -n "${NAMESPACE}" 2>/dev/null || true

        # Show status
        echo -e "\n${BLUE}Deployment Status:${NC}"
        kubectl get deployments,statefulsets,pods -n "${NAMESPACE}" --no-headers 2>/dev/null | head -20
        ;;

    delete)
        echo -e "\n${RED}⚠️  WARNING: This will delete all resources in ${NAMESPACE}${NC}"
        read -p "Are you sure? (yes/no): " confirm

        if [[ "$confirm" == "yes" ]]; then
            echo -e "\n${YELLOW}Deleting ${ENVIRONMENT} environment...${NC}"
            kubectl delete -k "${OVERLAY_DIR}" --ignore-not-found
            echo -e "\n${GREEN}✓ Deletion completed${NC}"
        else
            echo -e "${YELLOW}Deletion cancelled${NC}"
        fi
        ;;

    diff)
        echo -e "\n${YELLOW}Showing diff for ${ENVIRONMENT} environment...${NC}"
        kubectl diff -k "${OVERLAY_DIR}" 2>/dev/null || echo -e "${GREEN}No changes detected${NC}"
        ;;
esac

echo -e "\n${BLUE}Done!${NC}"
