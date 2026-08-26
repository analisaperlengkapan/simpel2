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
#
# Env vars (install/upgrade):
#   SIMPEL_HELM_WAIT=false     jangan tunggu kesiapan. WAJIB untuk bootstrap:
#                              Secreton lahir sealed dan tak pernah Ready sampai
#                              di-unseal, jadi install yang menunggu pasti gagal.
#   SIMPEL_HELM_TIMEOUT=20m    batas tunggu saat SIMPEL_HELM_WAIT aktif.
#   SIMPEL_CONFIRM_DESTROY=yes izin eksplisit untuk `uninstall` di production.

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
    # --create-namespace is decided by ASKING THE CHART, not by a rule kept here.
    # `templates/namespace.yaml` renders a Namespace when `namespace.create` is
    # true; values-staging sets it false (the object predates this lineage and is
    # kept outside the release on purpose), while values-production inherits the
    # default true. Passing both makes Helm create the namespace WITHOUT ownership
    # metadata and the chart's own Namespace object then collides:
    #   Error: namespaces "simpelv2-production" already exists
    # That is the same defect as `backup-install` below, which hit for real on
    # 2026-08-18 (#794). Deriving the flag from the rendered output means it stays
    # correct if a values file flips `namespace.create` either way.
    #
    # Rendered into a variable rather than piped straight into `grep -q`: this
    # script runs under `pipefail`, and `grep -q` exits at the first match, which
    # SIGPIPEs helm and makes the PIPELINE fail exactly when the pattern IS found.
    # Piped that way the test reports the opposite of the truth.
    rendered="$(helm template "$RELEASE" "$CHART_DIR" -f "$VALUES_FILE" \
                  --namespace "$NAMESPACE" "${EXTRA_ARGS[@]}" 2>/dev/null || true)"
    ns_flag=()
    grep -q '^kind: Namespace$' <<<"$rendered" || ns_flag=(--create-namespace)

    # `--wait` blocks until every workload reports Ready, so it is WRONG for the
    # documented bootstrap: Secreton comes up sealed and never becomes Ready until
    # an operator unseals it, so a waiting install times out and rolls back the
    # very release the unseal needs. And 5m never fit this chart — a full bring-up
    # pulls nine images and runs the migration hooks. Both are knobs now; the
    # bootstrap invocation is SIMPEL_HELM_WAIT=false.
    wait_flag=()
    if [ "${SIMPEL_HELM_WAIT:-true}" = "true" ]; then
      wait_flag=(--wait --timeout "${SIMPEL_HELM_TIMEOUT:-20m}")
    else
      echo "NOTE: SIMPEL_HELM_WAIT=false — not waiting for readiness (bootstrap mode)."
      echo "      Secreton starts sealed; unseal it, then re-run without the flag."
    fi

    helm upgrade --install "$RELEASE" "$CHART_DIR" \
      -f "$VALUES_FILE" \
      --namespace "$NAMESPACE" \
      "${ns_flag[@]+"${ns_flag[@]}"}" \
      "${wait_flag[@]+"${wait_flag[@]}"}" \
      "${EXTRA_ARGS[@]}"

    # Only meaningful when we asked Helm to wait; in bootstrap mode the workloads
    # are legitimately not Ready and this would just burn ten minutes saying so.
    if [ "${SIMPEL_HELM_WAIT:-true}" = "true" ]; then
      echo
      echo "─── Rollout status ───"
      kubectl rollout status -n "$NAMESPACE" deployment --timeout=300s 2>/dev/null || true
      kubectl rollout status -n "$NAMESPACE" statefulset --timeout=300s 2>/dev/null || true
    fi
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
    #
    # TANPA `--create-namespace` — chart ini me-render Namespace-nya SENDIRI
    # (dengan `resource-policy: keep` + label istio-injection). Memakai keduanya
    # membuat `helm install` gagal "namespaces ... already exists": Helm membuat
    # namespace lebih dulu tanpa metadata kepemilikan, lalu objek Namespace milik
    # chart menabraknya. Terjadi sungguhan 2026-08-18.
    helm upgrade --install simpel-backup "$BACKUP_CHART_DIR" \
      --namespace simpelv2-backup \
      "${EXTRA_ARGS[@]}"
    ;;
  *)
    echo "Error: unknown action '$ACTION'" >&2
    echo "Valid: install | upgrade | template | diff | status | rollback | uninstall | metallb-install | storage-install | backup-install" >&2
    exit 2
    ;;
esac
