#!/bin/bash
# SIMPelv2 MicroK8s Deployment Script
# 2-namespace architecture: simpelv2-infra + simpelv2
set -e

REGISTRY="localhost:32000"
NAMESPACE_INFRA="simpelv2-infra"
NAMESPACE_APPS="simpelv2"
KUBECTL="microk8s kubectl"

echo "=== SIMPelv2 MicroK8s Deployment ==="
echo "Namespaces: $NAMESPACE_INFRA (infra), $NAMESPACE_APPS (apps)"
echo ""

echo "Checking for required images..."
IMAGES=(
    "simpelv2/secreton:latest"
    "simpelv2/authenc:latest"
    "simpelv2/layanan-daskrimti-portal:latest"
    "simpelv2/daskrimti-portal:latest"
    "simpelv2/layanan-pembinaan-perlengkapan:latest"
    "simpelv2/pembinaan-perlengkapan:latest"
    "simpelv2/portal:latest"
)

for img in "${IMAGES[@]}"; do
    if [[ "$(docker images -q $REGISTRY/$img 2> /dev/null)" == "" ]]; then
        echo "Image $REGISTRY/$img not found locally!"
    else
        echo "Pushing $REGISTRY/$img..."
        docker push $REGISTRY/$img
    fi
done

echo ""
echo "Applying Kubernetes configurations..."
# Apply namespaces first
$KUBECTL apply -f infra/k8s/00-namespace.yaml

# Apply configs and secrets
$KUBECTL apply -f infra/k8s/01-configmap.yaml
$KUBECTL apply -f infra/k8s/02-secrets.yaml || true
$KUBECTL apply -f infra/k8s/03-persistent-volumes.yaml || true

# Apply infrastructure deployments
$KUBECTL apply -f infra/k8s/04-infrastructure-deployments.yaml || true

# Apply application deployments
$KUBECTL apply -f infra/k8s/05-backend-deployments.yaml
$KUBECTL apply -f infra/k8s/06-frontend-deployments.yaml

# Apply services
$KUBECTL apply -f infra/k8s/07-services.yaml

echo ""
echo "Restarting deployments to ensure fresh pull..."

# Infrastructure services
echo "Restarting infrastructure services in $NAMESPACE_INFRA..."
$KUBECTL rollout restart deployment/authenc -n $NAMESPACE_INFRA || true
$KUBECTL rollout restart deployment/secreton -n $NAMESPACE_INFRA || true

# Application services (all in simpelv2 namespace)
echo "Restarting application services in $NAMESPACE_APPS..."
$KUBECTL rollout restart deployment/layanan-daskrimti-portal -n $NAMESPACE_APPS || true
$KUBECTL rollout restart deployment/layanan-pembinaan-perlengkapan -n $NAMESPACE_APPS || true
$KUBECTL rollout restart deployment/portal -n $NAMESPACE_APPS || true
$KUBECTL rollout restart deployment/daskrimti-portal -n $NAMESPACE_APPS || true
$KUBECTL rollout restart deployment/pembinaan-perlengkapan -n $NAMESPACE_APPS || true

echo ""
echo "Waiting for deployments to stabilize..."
$KUBECTL rollout status deployment/authenc -n $NAMESPACE_INFRA --timeout=120s || true
$KUBECTL rollout status deployment/secreton -n $NAMESPACE_INFRA --timeout=120s || true

echo ""
echo "=== Current Pod Status ==="
echo "--- Infrastructure ($NAMESPACE_INFRA) ---"
$KUBECTL get pods -n $NAMESPACE_INFRA -o wide

echo ""
echo "--- Applications ($NAMESPACE_APPS) ---"
$KUBECTL get pods -n $NAMESPACE_APPS -o wide

echo ""
echo "=== Services ==="
echo "--- Infrastructure ---"
$KUBECTL get svc -n $NAMESPACE_INFRA

echo ""
echo "--- Applications ---"
$KUBECTL get svc -n $NAMESPACE_APPS

echo ""
echo "Deployment update triggered successfully."
echo ""
echo "To check logs:"
echo "  $KUBECTL logs -f deployment/authenc -n $NAMESPACE_INFRA"
echo "  $KUBECTL logs -f deployment/secreton -n $NAMESPACE_INFRA"
echo "  $KUBECTL logs -f deployment/layanan-daskrimti-portal -n $NAMESPACE_APPS"
