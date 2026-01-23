#!/bin/bash
set -e

echo "Verifying deployment health..."

echo "1. Checking Pod Status..."
kubectl get pods -n simpelv2-infra
kubectl get pods -n simpelv2-backend

echo "2. Checking Authenc Health..."
AUTHENC_POD=$(kubectl get pod -l app=authenc -n simpelv2-infra -o jsonpath="{.items[0].metadata.name}")
kubectl exec -n simpelv2-infra $AUTHENC_POD -- curl -s localhost:8088/health || echo "Authenc Health Check Failed"

echo "3. Checking Secreton Health..."
SECRETON_POD=$(kubectl get pod -l app=secreton -n simpelv2-infra -o jsonpath="{.items[0].metadata.name}")
kubectl exec -n simpelv2-infra $SECRETON_POD -- curl -s localhost:8200/v1/health || echo "Secreton Health Check Failed"

echo "4. Checking Portal Backend Health..."
PORTAL_POD=$(kubectl get pod -l app=layanan-daskrimti-portal -n simpelv2-backend -o jsonpath="{.items[0].metadata.name}")
kubectl exec -n simpelv2-backend $PORTAL_POD -- curl -s localhost:3010/health || echo "Portal Backend Health Check Failed"

echo "5. Testing Authenc -> Secreton Connectivity..."
kubectl exec -n simpelv2-infra $AUTHENC_POD -- curl -v http://secreton.simpelv2-infra.svc.cluster.local:8200/v1/health || echo "Connectivity Check Failed"

echo "Verification complete."
