#!/usr/bin/env bash
# SIMPEL Helm deployment wrapper.
#
# Usage:
#   ./deploy.sh <env> <action> [extra helm args...]
#
# Env:    staging | production | review-<slug>
# Action: install | upgrade | template | diff | status | rollback | uninstall
#         metallb-install | storage-install | backup-install
#                                             (cluster-scoped, env diabaikan)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHART_DIR="${SCRIPT_DIR}/simpel"
METALLB_CHART_DIR="${SCRIPT_DIR}/metallb"
STORAGE_CHART_DIR="${SCRIPT_DIR}/storage"
BACKUP_CHART_DIR="${SCRIPT_DIR}/backup"

ENV="${1:-staging}"
ACTION="${2:-template}"
shift 2 || true
EXTRA_ARGS=("$@")

RELEASE="simpel"
case "$ENV" in
  staging)
    NAMESPACE="simpelv2-staging"
    VALUES_FILE="${CHART_DIR}/values-staging.yaml"
    ;;
  production)
    NAMESPACE="simpelv2-production"
    VALUES_FILE="${CHART_DIR}/values-production.yaml"
    ;;
  review-*)
    SLUG="${ENV#review-}"
    NAMESPACE="simpelv2-review-${SLUG}"
    VALUES_FILE="${CHART_DIR}/values-staging.yaml"
    RELEASE="simpel-review-${SLUG}"
    ;;
  *)
    echo "Error: unknown env '$ENV' (expected: staging | production | review-<slug>)" >&2
    exit 2
    ;;
esac

if ! command -v helm >/dev/null 2>&1; then
  echo "Error: helm not found in PATH" >&2
  exit 127
fi

case "$ACTION" in
  install|upgrade)
    helm upgrade --install "$RELEASE" "$CHART_DIR" \
      -f "$VALUES_FILE" \
      --namespace "$NAMESPACE" \
      --create-namespace \
      --wait --timeout 5m \
      "${EXTRA_ARGS[@]}"
    echo
    echo "─── Rollout status ───"
    kubectl rollout status -n "$NAMESPACE" deployment --timeout=300s 2>/dev/null || true
    kubectl rollout status -n "$NAMESPACE" statefulset --timeout=300s 2>/dev/null || true
    ;;
  template)
    helm template "$RELEASE" "$CHART_DIR" \
      -f "$VALUES_FILE" \
      --namespace "$NAMESPACE" \
      "${EXTRA_ARGS[@]}"
    ;;
  diff)
    if helm plugin list 2>/dev/null | grep -q '^diff'; then
      helm diff upgrade "$RELEASE" "$CHART_DIR" \
        -f "$VALUES_FILE" \
        --namespace "$NAMESPACE" \
        "${EXTRA_ARGS[@]}"
    else
      helm template "$RELEASE" "$CHART_DIR" \
        -f "$VALUES_FILE" \
        --namespace "$NAMESPACE" \
        "${EXTRA_ARGS[@]}" |
        kubectl diff -f -
    fi
    ;;
  status)
    helm status "$RELEASE" -n "$NAMESPACE" "${EXTRA_ARGS[@]}"
    ;;
  rollback)
    helm rollback "$RELEASE" -n "$NAMESPACE" "${EXTRA_ARGS[@]}"
    ;;
  uninstall)
    # SAFETY (incident 2026-06-17): `helm uninstall` tears down ALL workloads.
    # The namespace + data PVCs are retained (chart `resource-policy: keep` +
    # PVC retention policy), but the environment goes DOWN. Routine lifecycle is
    # upgrade-only; uninstall is exceptional. Refuse on production unless the
    # operator explicitly confirms (and has a fresh Velero backup).
    if [ "$ENV" = "production" ] && [ "${SIMPEL_CONFIRM_DESTROY:-}" != "yes" ]; then
      echo "REFUSED: uninstall on PRODUCTION requires SIMPEL_CONFIRM_DESTROY=yes" >&2
      echo "  Namespace + PVCs are retained, but ALL workloads will be torn down." >&2
      echo "  Take a Velero backup first, then re-run with SIMPEL_CONFIRM_DESTROY=yes." >&2
      exit 3
    fi
    helm uninstall "$RELEASE" -n "$NAMESPACE" "${EXTRA_ARGS[@]}"
    ;;
  metallb-install)
    helm upgrade --install simpel-metallb "$METALLB_CHART_DIR" \
      --namespace metallb-system \
      "${EXTRA_ARGS[@]}"
    ;;
  storage-install)
    # Cluster-scoped: StorageClass `longhorn-retain` dipakai BERSAMA oleh staging
    # dan production, jadi satu rilis untuk seluruh klaster (bukan per-env).
    helm upgrade --install simpel-storage "$STORAGE_CHART_DIR" \
      --namespace longhorn-system \
      "${EXTRA_ARGS[@]}"
    ;;
  backup-install)
    # Cluster-scoped: satu object store MinIO melayani backup SEMUA environment.
    # Namespace-nya sengaja di luar namespace aplikasi — backup yang tinggal di
    # namespace yang sama akan ikut mati bersama insiden yang seharusnya ia
    # selamatkan (2026-06-17).
    #
    # First install butuh kredensial:
    #   ./deploy.sh <env> backup-install \
    #     -f infra/helm/backup/values-secrets.yaml --set secrets.bootstrap=true
    helm upgrade --install simpel-backup "$BACKUP_CHART_DIR" \
      --namespace simpelv2-backup --create-namespace \
      "${EXTRA_ARGS[@]}"
    ;;
  *)
    echo "Error: unknown action '$ACTION'" >&2
    echo "Valid: install | upgrade | template | diff | status | rollback | uninstall | metallb-install | storage-install | backup-install" >&2
    exit 2
    ;;
esac
