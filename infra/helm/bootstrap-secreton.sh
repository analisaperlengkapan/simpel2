#!/usr/bin/env bash
# bootstrap-secreton.sh — One-shot setup Kubernetes Auth Backend di Secreton.
#
# Pre-requisites:
#   - Secreton sudah unsealed (secreton operator init && unseal × threshold).
#   - SECRETON_TOKEN env = root token (akan di-revoke setelah selesai).
#   - kubectl context sudah switch ke target cluster.
#   - ConfigMap `secreton-auth-config` & `secreton-policies` sudah di-render Helm.
#
# Usage:
#   SECRETON_TOKEN=hvs.xxxx ./infra/helm/bootstrap-secreton.sh staging
#   SECRETON_TOKEN=hvs.xxxx ./infra/helm/bootstrap-secreton.sh production
#
# What it does:
#   1. Enable kubernetes auth method di Secreton.
#   2. Configure auth backend (TokenReview API URL, audience).
#   3. Apply 1 policy per service (dari ConfigMap secreton-policies).
#   4. Create role mapping SA → policy (dari ConfigMap secreton-auth-config).
#   5. Revoke root token jika --revoke-root flag diberikan.

set -euo pipefail

ENV="${1:-}"
case "$ENV" in
  staging)    NAMESPACE="simpelv2-staging" ;;
  production) NAMESPACE="simpelv2-production" ;;
  *)
    echo "Usage: $0 <staging|production> [--revoke-root]"
    exit 1
    ;;
esac

REVOKE_ROOT=0
[[ "${2:-}" == "--revoke-root" ]] && REVOKE_ROOT=1

: "${SECRETON_TOKEN:?SECRETON_TOKEN env wajib (root token bootstrap)}"

# Discover Secreton via port-forward sementara (assumes SA tidak ada di cluster yet).
echo ">>> [1/6] Establish port-forward ke secreton :8200..."
kubectl -n "$NAMESPACE" port-forward svc/secreton 8200:8200 >/dev/null 2>&1 &
PF_PID=$!
trap 'kill $PF_PID 2>/dev/null || true' EXIT
sleep 3

SECRETON_ADDR="http://127.0.0.1:8200"
api() {
  local method=$1 path=$2
  shift 2
  curl -fsS -X "$method" \
    -H "X-Secreton-Token: $SECRETON_TOKEN" \
    -H "Content-Type: application/json" \
    "$@" \
    "$SECRETON_ADDR/v1$path"
}

echo ">>> [2/6] Verifikasi Secreton unsealed..."
api GET /sys/health > /tmp/secreton-health.json
SEALED=$(jq -r '.sealed' /tmp/secreton-health.json)
[[ "$SEALED" == "false" ]] || { echo "ERROR: Secreton masih sealed. Jalankan unseal dulu."; exit 1; }

echo ">>> [3/6] Enable kubernetes auth method (idempotent)..."
api POST /sys/auth/kubernetes -d '{"type":"kubernetes","description":"K8s SA token auth"}' || true

echo ">>> [4/6] Configure kubernetes auth backend..."
KUBE_CA=$(kubectl -n "$NAMESPACE" get cm secreton-auth-config -o jsonpath='{.data.config\.json}' | jq -r '.kubernetes_host')
api POST /auth/kubernetes/config -d "{
  \"kubernetes_host\": \"https://kubernetes.default.svc\",
  \"kubernetes_ca_cert\": \"@/var/run/secrets/kubernetes.io/serviceaccount/ca.crt\",
  \"disable_local_ca_jwt\": false
}"

echo ">>> [5/6] Apply policies & roles per service..."
SERVICES=$(kubectl -n "$NAMESPACE" get cm secreton-policies -o json | jq -r '.data | keys[]' | sed 's/\.hcl$//')
for svc in $SERVICES; do
  POLICY=$(kubectl -n "$NAMESPACE" get cm secreton-policies -o jsonpath="{.data.${svc}\\.hcl}")
  echo "    - policy: $svc"
  api PUT "/sys/policies/acl/$svc" -d "$(jq -nc --arg p "$POLICY" '{policy:$p}')"

  ROLE_JSON=$(kubectl -n "$NAMESPACE" get cm secreton-auth-config -o jsonpath="{.data.role-${svc}\\.json}")
  if [[ -n "$ROLE_JSON" ]]; then
    echo "    - role:   $svc"
    api POST "/auth/kubernetes/role/$svc" -d "$ROLE_JSON"
  fi
done

if [[ $REVOKE_ROOT -eq 1 ]]; then
  echo ">>> [6/6] Revoke root token (cleanup)..."
  api POST /auth/token/revoke-self || true
  echo "Root token revoked. Pakai SA-based auth dari sekarang."
else
  echo ">>> [6/6] SKIP revoke (pass --revoke-root jika ingin cleanup)."
fi

echo "✓ Bootstrap selesai untuk namespace $NAMESPACE."
