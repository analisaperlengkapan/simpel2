#!/bin/bash
# PostgreSQL HA Operator Installation Script
# Install Zalando Postgres Operator for High Availability

set -e

echo "📦 Installing Zalando Postgres Operator..."

# Add Helm repository
echo "Adding Helm repository..."
helm repo add postgres-operator https://opensource.zalando.com/postgres-operator/charts/postgres-operator
helm repo update

# Install operator
echo "Installing operator..."
helm install postgres-operator postgres-operator/postgres-operator \
  --namespace postgres-operator \
  --create-namespace \
  --set configKubernetes.enable_pod_antiaffinity=true \
  --set configKubernetes.storage_resize_mode=pvc \
  --wait

# Install UI (optional)
echo "Installing Postgres Operator UI..."
helm install postgres-operator-ui postgres-operator/postgres-operator-ui \
  --namespace postgres-operator \
  --wait

echo "✅ Postgres Operator installed successfully!"
echo ""
echo "Check operator status:"
echo "  kubectl get pods -n postgres-operator"
echo ""
echo "Access UI (after port-forward):"
echo "  kubectl port-forward svc/postgres-operator-ui -n postgres-operator 8081:80"
echo "  Open: http://localhost:8081"
