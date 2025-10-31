#!/bin/bash
# Script untuk verifikasi SSL certificate yang sudah di-deploy

set -e

echo "=================================================="
echo "SSL Certificate Verification"
echo "=================================================="
echo ""

DOMAIN="simpel.kejaksaan.go.id"
SECRET_NAME="simpelv2-tls-secret"
NAMESPACE="istio-system"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Detect kubectl command (support both kubectl and microk8s)
if command -v microk8s &> /dev/null; then
    KUBECTL="microk8s kubectl"
elif command -v kubectl &> /dev/null; then
    KUBECTL="kubectl"
else
    KUBECTL=""
fi

echo -e "${BLUE}1. Checking Kubernetes Secret${NC}"
echo "--------------------------------"
if [ -n "$KUBECTL" ]; then
    if $KUBECTL get secret "$SECRET_NAME" -n "$NAMESPACE" &> /dev/null; then
        echo -e "${GREEN}✓ Secret exists in $NAMESPACE namespace${NC}"
        echo ""
        echo "Secret details:"
        $KUBECTL get secret "$SECRET_NAME" -n "$NAMESPACE" -o jsonpath='{.metadata.creationTimestamp}' | xargs -I {} echo "Created: {}"
        $KUBECTL get secret "$SECRET_NAME" -n "$NAMESPACE" -o jsonpath='{.type}' | xargs -I {} echo "Type: {}"
        echo ""
    else
        echo -e "${RED}✗ Secret not found in $NAMESPACE namespace${NC}"
    fi
else
    echo -e "${YELLOW}⚠ kubectl/microk8s not available, skipping Kubernetes checks${NC}"
fi
echo ""

echo -e "${BLUE}2. Testing HTTPS Connection${NC}"
echo "--------------------------------"
if command -v curl &> /dev/null; then
    echo "Testing connection to https://$DOMAIN..."
    if curl -sI --max-time 10 https://$DOMAIN > /dev/null 2>&1; then
        echo -e "${GREEN}✓ HTTPS connection successful${NC}"
    else
        echo -e "${RED}✗ HTTPS connection failed${NC}"
    fi
else
    echo -e "${YELLOW}⚠ curl not available${NC}"
fi
echo ""

echo -e "${BLUE}3. Certificate Details from Server${NC}"
echo "--------------------------------"
if command -v openssl &> /dev/null; then
    echo "Retrieving certificate from $DOMAIN:443..."
    CERT_INFO=$(echo | openssl s_client -connect $DOMAIN:443 -servername $DOMAIN 2>/dev/null | openssl x509 -noout -subject -issuer -dates 2>/dev/null)
    
    if [ -n "$CERT_INFO" ]; then
        echo "$CERT_INFO"
        echo ""
        
        # Check if it's the fake certificate
        if echo "$CERT_INFO" | grep -q "Kubernetes Ingress Controller Fake Certificate"; then
            echo -e "${RED}✗ WARNING: Still using FAKE certificate!${NC}"
            echo ""
            echo "Possible causes:"
            echo "1. Secret not properly deployed to istio-system namespace"
            echo "2. Istio Ingress Gateway not restarted"
            echo "3. Certificate not yet propagated (wait 1-2 minutes)"
            echo ""
            echo "Run deployment script again:"
            echo "  sudo ./deploy-real-certificate.sh"
        elif echo "$CERT_INFO" | grep -q "DigiCert"; then
            echo -e "${GREEN}✓ Using REAL DigiCert certificate!${NC}"
            
            # Check expiry
            EXPIRY_DATE=$(echo | openssl s_client -connect $DOMAIN:443 -servername $DOMAIN 2>/dev/null | openssl x509 -noout -enddate 2>/dev/null | cut -d= -f2)
            if [ -n "$EXPIRY_DATE" ]; then
                EXPIRY_EPOCH=$(date -d "$EXPIRY_DATE" +%s 2>/dev/null || echo "0")
                CURRENT_EPOCH=$(date +%s)
                if [ "$EXPIRY_EPOCH" != "0" ]; then
                    DAYS_UNTIL_EXPIRY=$(( ($EXPIRY_EPOCH - $CURRENT_EPOCH) / 86400 ))
                    
                    if [ $DAYS_UNTIL_EXPIRY -lt 0 ]; then
                        echo -e "${RED}✗ Certificate EXPIRED!${NC}"
                    elif [ $DAYS_UNTIL_EXPIRY -lt 30 ]; then
                        echo -e "${YELLOW}⚠ Certificate expires in $DAYS_UNTIL_EXPIRY days${NC}"
                    else
                        echo -e "${GREEN}✓ Certificate valid for $DAYS_UNTIL_EXPIRY days${NC}"
                    fi
                fi
            fi
        else
            echo -e "${YELLOW}⚠ Unknown certificate issuer${NC}"
        fi
    else
        echo -e "${RED}✗ Could not retrieve certificate from server${NC}"
        echo "Server may not be reachable or TLS handshake failed"
    fi
else
    echo -e "${YELLOW}⚠ openssl not available${NC}"
fi
echo ""

echo -e "${BLUE}4. Full Certificate Chain${NC}"
echo "--------------------------------"
if command -v openssl &> /dev/null; then
    echo | openssl s_client -connect $DOMAIN:443 -servername $DOMAIN -showcerts 2>/dev/null | grep -E "subject=|issuer=" | head -10
fi
echo ""

echo -e "${BLUE}5. Istio Gateway Status${NC}"
echo "--------------------------------"
if [ -n "$KUBECTL" ]; then
    if $KUBECTL get gateway simpelv2-gateway -n istio-system &> /dev/null; then
        echo -e "${GREEN}✓ Gateway exists${NC}"
        echo ""
        echo "Gateway configuration:"
        $KUBECTL get gateway simpelv2-gateway -n istio-system -o jsonpath='{.spec.servers[0].tls.credentialName}' | xargs -I {} echo "TLS Secret: {}"
    else
        echo -e "${YELLOW}⚠ Gateway not found${NC}"
    fi
    echo ""
    
    echo "Istio Ingress Gateway Pods:"
    $KUBECTL get pods -n istio-system -l istio=ingressgateway
else
    echo -e "${YELLOW}⚠ kubectl/microk8s not available${NC}"
fi
echo ""

echo "=================================================="
echo "Verification Complete"
echo "=================================================="
echo ""
echo "Browser Testing:"
echo "1. Open: https://$DOMAIN"
echo "2. Check certificate details (click padlock icon)"
echo "3. Should show: CN=*.kejaksaan.go.id, Issuer=DigiCert"
echo ""
echo "If still showing fake certificate:"
echo "- Clear browser cache: Ctrl+Shift+Delete"
echo "- Clear HSTS: chrome://net-internals/#hsts"
echo "- Wait 2-3 minutes for propagation"
echo "- Try incognito/private mode"
