#!/bin/bash
# Script untuk deploy sertifikat SSL asli ke Kubernetes
# Sertifikat: /etc/ssl/simpel.kejaksaan.go.id/

set -e

echo "=================================================="
echo "Deploy Real SSL Certificate to Kubernetes"
echo "=================================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Certificate paths
CERT_DIR="/etc/ssl/simpel.kejaksaan.go.id"
FULLCHAIN="${CERT_DIR}/fullchain.pem"
PRIVKEY="${CERT_DIR}/privkey.pem"

# Kubernetes settings
SECRET_NAME="simpelv2-tls-secret"
NAMESPACE="istio-system"
OLD_NAMESPACE="istio-ingress"

# Check if running as root or with sudo
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}Error: This script must be run as root or with sudo${NC}"
    echo "Usage: sudo $0"
    exit 1
fi

# Verify certificate files exist
echo "Step 1: Verifying certificate files..."
if [ ! -f "$FULLCHAIN" ]; then
    echo -e "${RED}Error: Certificate file not found: $FULLCHAIN${NC}"
    exit 1
fi

if [ ! -f "$PRIVKEY" ]; then
    echo -e "${RED}Error: Private key file not found: $PRIVKEY${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Certificate files found${NC}"
echo ""

# Display certificate information
echo "Step 2: Certificate Information"
echo "--------------------------------"
openssl x509 -in "$FULLCHAIN" -noout -subject -issuer -dates
echo ""

# Check certificate expiry
EXPIRY_DATE=$(openssl x509 -in "$FULLCHAIN" -noout -enddate | cut -d= -f2)
EXPIRY_EPOCH=$(date -d "$EXPIRY_DATE" +%s)
CURRENT_EPOCH=$(date +%s)
DAYS_UNTIL_EXPIRY=$(( ($EXPIRY_EPOCH - $CURRENT_EPOCH) / 86400 ))

if [ $DAYS_UNTIL_EXPIRY -lt 0 ]; then
    echo -e "${RED}Warning: Certificate has EXPIRED!${NC}"
    exit 1
elif [ $DAYS_UNTIL_EXPIRY -lt 30 ]; then
    echo -e "${YELLOW}Warning: Certificate will expire in $DAYS_UNTIL_EXPIRY days${NC}"
else
    echo -e "${GREEN}✓ Certificate valid for $DAYS_UNTIL_EXPIRY days${NC}"
fi
echo ""

# Detect kubectl command (support both kubectl and microk8s)
if command -v microk8s &> /dev/null; then
    KUBECTL="microk8s kubectl"
    echo "Detected: MicroK8s"
elif command -v kubectl &> /dev/null; then
    KUBECTL="kubectl"
    echo "Detected: kubectl"
else
    echo -e "${RED}Error: Neither kubectl nor microk8s found.${NC}"
    echo "Please install one of them first."
    exit 1
fi

# Check cluster connection
echo "Step 3: Checking Kubernetes cluster connection..."
if ! $KUBECTL cluster-info &> /dev/null; then
    echo -e "${RED}Error: Cannot connect to Kubernetes cluster${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Connected to Kubernetes cluster${NC}"
echo ""

# Delete old secret from wrong namespace if exists
echo "Step 4: Cleaning up old secrets..."
$KUBECTL delete secret "$SECRET_NAME" -n "$OLD_NAMESPACE" --ignore-not-found
echo -e "${GREEN}✓ Old secret cleaned up (if existed)${NC}"
echo ""

# Delete existing secret in correct namespace if exists
echo "Step 5: Removing existing secret in $NAMESPACE namespace..."
$KUBECTL delete secret "$SECRET_NAME" -n "$NAMESPACE" --ignore-not-found
echo -e "${GREEN}✓ Existing secret removed (if existed)${NC}"
echo ""

# Create new TLS secret
echo "Step 6: Creating new TLS secret..."
$KUBECTL create secret tls "$SECRET_NAME" \
    --cert="$FULLCHAIN" \
    --key="$PRIVKEY" \
    -n "$NAMESPACE"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ TLS secret created successfully${NC}"
else
    echo -e "${RED}Error: Failed to create TLS secret${NC}"
    exit 1
fi
echo ""

# Verify secret was created
echo "Step 7: Verifying secret..."
$KUBECTL get secret "$SECRET_NAME" -n "$NAMESPACE" -o yaml | head -20
echo ""

# Restart Istio Ingress Gateway
echo "Step 8: Restarting Istio Ingress Gateway..."
if $KUBECTL get deployment istio-ingressgateway -n istio-system &> /dev/null; then
    $KUBECTL rollout restart deployment istio-ingressgateway -n istio-system
    echo "Waiting for rollout to complete..."
    $KUBECTL rollout status deployment istio-ingressgateway -n istio-system --timeout=120s
    echo -e "${GREEN}✓ Istio Ingress Gateway restarted${NC}"
else
    echo -e "${YELLOW}Warning: istio-ingressgateway deployment not found in istio-system namespace${NC}"
    echo "You may need to manually restart your ingress controller"
fi
echo ""

# Final verification
echo "Step 9: Final Verification"
echo "-------------------------"
echo "Secret details:"
$KUBECTL describe secret "$SECRET_NAME" -n "$NAMESPACE"
echo ""

echo -e "${GREEN}=================================================="
echo "Deployment Complete!"
echo "==================================================${NC}"
echo ""
echo "Next steps:"
echo "1. Wait 30-60 seconds for the certificate to propagate"
echo "2. Test the connection:"
echo "   curl -vI https://simpel.kejaksaan.go.id"
echo ""
echo "3. Verify certificate in browser:"
echo "   https://simpel.kejaksaan.go.id"
echo ""
echo "4. Check certificate details:"
echo "   echo | openssl s_client -connect simpel.kejaksaan.go.id:443 -servername simpel.kejaksaan.go.id 2>/dev/null | openssl x509 -noout -text"
echo ""
echo "If you still see the fake certificate:"
echo "- Clear browser cache and cookies"
echo "- Chrome: chrome://net-internals/#hsts → Delete domain"
echo "- Wait a few more minutes for DNS/certificate propagation"
