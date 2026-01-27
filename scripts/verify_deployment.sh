#!/bin/bash
set -e
KUBECTL="microk8s kubectl"

echo "Verifying deployment health..."

echo "1. Checking Pod Status..."
$KUBECTL get pods -n simpelv2

echo "2. Checking Authenc Health..."
AUTHENC_POD=$($KUBECTL get pods -l app.kubernetes.io/name=authenc -n simpelv2 --no-headers | grep Running | head -n 1 | awk '{print $1}')
if [ -n "$AUTHENC_POD" ]; then
    $KUBECTL exec -n simpelv2 "$AUTHENC_POD" -- curl -s localhost:8088/health || echo "Authenc Health Check Failed"
else
    echo "Authenc Pod not found or not running"
fi

echo "3. Checking Secreton Health..."
SECRETON_POD=$($KUBECTL get pods -l app.kubernetes.io/name=secreton -n simpelv2 --no-headers | grep Running | head -n 1 | awk '{print $1}')
if [ -n "$SECRETON_POD" ]; then
    # Use crypto-specific health check for more detail
    $KUBECTL exec -n simpelv2 "$SECRETON_POD" -- curl -s localhost:9090/health || echo "Secreton Health Check Failed"
else
    echo "Secreton Pod not found or not running"
fi

echo "4. Checking Portal Backend Health..."
PORTAL_POD=$($KUBECTL get pods -l app.kubernetes.io/name=layanan-daskrimti-portal -n simpelv2 --no-headers | grep Running | head -n 1 | awk '{print $1}')
if [ -n "$PORTAL_POD" ]; then
    $KUBECTL exec -n simpelv2 "$PORTAL_POD" -- curl -s localhost:3010/health || echo "Portal Backend Health Check Failed"
else
    echo "Portal Backend Pod not found or not running"
fi

echo "5. Testing Authenc -> Secreton Connectivity..."
$KUBECTL exec -n simpelv2 $AUTHENC_POD -- curl -v http://secreton.simpelv2.svc.cluster.local:9090/health || echo "Connectivity Check Failed"

echo "Verification complete."
