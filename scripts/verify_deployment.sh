#!/bin/bash
set -e
KUBECTL="microk8s kubectl"

echo "Verifying deployment health..."

echo "1. Checking Pod Status..."
$KUBECTL get pods -n simpelv2-infra
$KUBECTL get pods -n simpelv2

echo "2. Checking Authenc Health..."
AUTHENC_POD=$($KUBECTL get pod -l app.kubernetes.io/name=authenc -n simpelv2 -o jsonpath="{.items[0].metadata.name}")
$KUBECTL exec -n simpelv2 $AUTHENC_POD -- curl -s localhost:8088/health || echo "Authenc Health Check Failed"

echo "3. Checking Secreton Health..."
SECRETON_POD=$($KUBECTL get pod -l app.kubernetes.io/name=secreton -n simpelv2 -o jsonpath="{.items[0].metadata.name}")
$KUBECTL exec -n simpelv2 $SECRETON_POD -- curl -s localhost:8200/v1/health || echo "Secreton Health Check Failed"

echo "4. Checking Portal Backend Health..."
PORTAL_POD=$($KUBECTL get pod -l app.kubernetes.io/name=layanan-daskrimti-portal -n simpelv2 -o jsonpath="{.items[0].metadata.name}")
$KUBECTL exec -n simpelv2 $PORTAL_POD -- curl -s localhost:3010/health || echo "Portal Backend Health Check Failed"

echo "5. Testing Authenc -> Secreton Connectivity..."
$KUBECTL exec -n simpelv2 $AUTHENC_POD -- curl -v http://secreton.simpelv2.svc.cluster.local:8200/v1/health || echo "Connectivity Check Failed"

echo "Verification complete."
