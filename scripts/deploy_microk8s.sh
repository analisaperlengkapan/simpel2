#!/bin/bash
set -e

REGISTRY="localhost:32000"
NAMESPACE_INFRA="simpelv2-infra"
NAMESPACE_BACKEND="simpelv2-backend"

echo "Checking for required images..."
IMAGES=(
    "simpelv2/secreton:latest"
    "simpelv2/authenc:latest"
    "simpelv2/layanan-daskrimti-portal:latest"
    "simpelv2/daskrimti-portal:latest"
    "simpelv2/layanan-pembinaan-perlengkapan:latest"
    "simpelv2/pembinaan-perlengkapan:latest"
)

for img in "${IMAGES[@]}"; do
    if [[ "$(docker images -q $REGISTRY/$img 2> /dev/null)" == "" ]]; then
        echo "Image $REGISTRY/$img not found locally!"
        # Optional: check if build is still running or warn user
    else
        echo "Pushing $REGISTRY/$img..."
        docker push $REGISTRY/$img
    fi
done

echo "Applying Kubernetes manifests..."
kubectl apply -f infra/k8s/00-namespace.yaml
# Secrets should be applied manually or pre-existing
# kubectl apply -f infra/k8s/02-secrets.yaml
kubectl apply -f infra/k8s/01-configmap.yaml
kubectl apply -f infra/k8s/03-persistent-volumes.yaml
kubectl apply -f infra/k8s/04-infrastructure-deployments.yaml
kubectl apply -f infra/k8s/04-authenc-deployment.yaml
kubectl apply -f infra/k8s/04-secreton-deployment.yaml
kubectl apply -f infra/k8s/05-backend-deployments.yaml
kubectl apply -f infra/k8s/06-frontend-deployments.yaml
kubectl apply -f infra/k8s/07-services.yaml

echo "Restarting deployments to ensure fresh pull..."
kubectl rollout restart deployment/authenc -n $NAMESPACE_INFRA
kubectl rollout restart deployment/secreton -n $NAMESPACE_INFRA
kubectl rollout restart deployment/layanan-daskrimti-portal -n $NAMESPACE_BACKEND
kubectl rollout restart deployment/layanan-pembinaan-perlengkapan -n $NAMESPACE_BACKEND
# Frontends likely use same name
kubectl rollout restart deployment/daskrimti-portal -n $NAMESPACE_BACKEND
kubectl rollout restart deployment/pembinaan-perlengkapan -n $NAMESPACE_BACKEND

echo "Deployment update triggered."
