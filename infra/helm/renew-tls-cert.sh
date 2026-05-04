#!/usr/bin/env bash
# Regenerasi self-signed TLS cert untuk staging Istio Gateway (default 10.1.7.121).
# Override via env var:
#   STAGING_IP=10.1.7.121
#   STAGING_DNS=simpelv2-staging.kejaksaan.go.id
#   TLS_SECRET_NAME=simpelv2-tls-secret
#   TLS_NAMESPACE=istio-system
#   CERT_DAYS=730

set -euo pipefail

STAGING_IP="${STAGING_IP:-10.1.7.121}"
STAGING_DNS="${STAGING_DNS:-simpelv2-staging.kejaksaan.go.id}"
TLS_SECRET_NAME="${TLS_SECRET_NAME:-simpelv2-tls-secret}"
TLS_NAMESPACE="${TLS_NAMESPACE:-istio-system}"
CERT_DAYS="${CERT_DAYS:-730}"

CERT_FILE="$(mktemp /tmp/staging-XXXXXX.crt)"
KEY_FILE="$(mktemp /tmp/staging-XXXXXX.key)"
trap 'rm -f "$CERT_FILE" "$KEY_FILE"' EXIT

echo "Generating self-signed TLS cert for ${STAGING_IP} (CN=simpelv2-staging)…"
openssl req -x509 -newkey rsa:4096 \
  -keyout "$KEY_FILE" \
  -out "$CERT_FILE" \
  -days "$CERT_DAYS" \
  -nodes \
  -subj "/C=ID/ST=DKI Jakarta/L=Jakarta Selatan/O=KEJAKSAAN AGUNG REPUBLIK INDONESIA/CN=simpelv2-staging" \
  -addext "subjectAltName=IP:${STAGING_IP},DNS:${STAGING_DNS}"

echo "Cert generated:"
openssl x509 -noout -subject -enddate -in "$CERT_FILE"

echo
echo "Updating Kubernetes TLS secret in ${TLS_NAMESPACE}…"
kubectl create secret tls "$TLS_SECRET_NAME" \
  --cert="$CERT_FILE" \
  --key="$KEY_FILE" \
  -n "$TLS_NAMESPACE" \
  --dry-run=client -o yaml | kubectl apply -f -

echo "Restarting istio-ingressgateway to reload cert…"
kubectl rollout restart deployment/istio-ingressgateway -n "$TLS_NAMESPACE"
kubectl rollout status deployment/istio-ingressgateway -n "$TLS_NAMESPACE" --timeout=60s

sleep 2
echo
echo "Verifying HTTPS:"
curl -sk --max-time 10 -o /dev/null -w "HTTPS /: %{http_code}\n" "https://${STAGING_IP}/" || true
