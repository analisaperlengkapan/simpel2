#!/bin/bash
# Script untuk regenerasi self-signed TLS cert untuk staging (10.1.7.121)
# Cert saat ini: valid sampai Feb 18, 2028
# Jalankan script ini jika cert expired atau perlu diperbarui

set -e

CERT_FILE=$(mktemp /tmp/staging-XXXXXX.crt)
KEY_FILE=$(mktemp /tmp/staging-XXXXXX.key)

echo "Generating self-signed TLS cert for staging (IP: 10.1.7.121)..."

openssl req -x509 -newkey rsa:4096 \
  -keyout "$KEY_FILE" \
  -out "$CERT_FILE" \
  -days 730 \
  -nodes \
  -subj "/C=ID/ST=DKI Jakarta/L=Jakarta Selatan/O=KEJAKSAAN AGUNG REPUBLIK INDONESIA/CN=simpelv2-staging" \
  -addext "subjectAltName=IP:10.1.7.121,DNS:simpelv2-staging.kejaksaan.go.id"

echo "Cert generated:"
openssl x509 -noout -subject -enddate -in "$CERT_FILE"

echo ""
echo "Updating Kubernetes TLS secret in istio-system..."
kubectl create secret tls simpelv2-tls-secret \
  --cert="$CERT_FILE" \
  --key="$KEY_FILE" \
  -n istio-system \
  --dry-run=client -o yaml | kubectl replace -f -

echo "Restarting istio-ingressgateway to reload cert..."
kubectl rollout restart deployment/istio-ingressgateway -n istio-system
kubectl rollout status deployment/istio-ingressgateway -n istio-system --timeout=60s

echo "Done! Testing HTTPS..."
sleep 3
curl -sk --max-time 10 -o /dev/null -w "HTTPS /: %{http_code}\n" https://10.1.7.121/

rm -f "$CERT_FILE" "$KEY_FILE"
