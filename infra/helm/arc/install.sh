#!/usr/bin/env bash
# install.sh — Pasang Actions Runner Controller (ARC) + satu runner scale set
# untuk repo simpel2 di microk8s. Idempotent (helm upgrade --install).
#
# PRASYARAT:
#   - kubectl & helm bisa menjangkau cluster (cek: `kubectl get nodes`).
#   - PAT GitHub di env var GITHUB_PAT (classic: scope `repo`; atau
#     fine-grained: Administration RW + Actions RW + Metadata R untuk repo ini).
#     PAT TIDAK disimpan di repo — hanya jadi Secret di cluster.
#
# PEMAKAIAN:
#   GITHUB_PAT=ghp_xxx ./infra/helm/arc/install.sh
#
# Variabel opsional:
#   ARC_VERSION         (default: kosong = chart terbaru)
#   CONTROLLER_NS       (default: arc-systems)
#   RUNNER_NS           (default: arc-runners)
#   RELEASE_CONTROLLER  (default: arc)
#   RELEASE_RUNNERSET   (default: arc-simpel)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTROLLER_NS="${CONTROLLER_NS:-arc-systems}"
RUNNER_NS="${RUNNER_NS:-arc-runners}"
RELEASE_CONTROLLER="${RELEASE_CONTROLLER:-arc}"
RELEASE_RUNNERSET="${RELEASE_RUNNERSET:-arc-simpel}"
OCI_BASE="oci://ghcr.io/actions/actions-runner-controller-charts"
VER_FLAG=()
[ -n "${ARC_VERSION:-}" ] && VER_FLAG=(--version "${ARC_VERSION}")

if [ -z "${GITHUB_PAT:-}" ]; then
  echo "ERROR: set GITHUB_PAT (lihat header skrip)." >&2
  exit 1
fi

echo "==> Namespaces (+ PSA privileged untuk dind)"
kubectl create namespace "${CONTROLLER_NS}" --dry-run=client -o yaml | kubectl apply -f -
kubectl create namespace "${RUNNER_NS}" --dry-run=client -o yaml | kubectl apply -f -
# dind butuh pod privileged → pastikan namespace runner tidak menolaknya.
kubectl label namespace "${RUNNER_NS}" \
  pod-security.kubernetes.io/enforce=privileged --overwrite

echo "==> Secret PAT (arc-github-token) di ${RUNNER_NS}"
kubectl create secret generic arc-github-token \
  --namespace "${RUNNER_NS}" \
  --from-literal=github_token="${GITHUB_PAT}" \
  --dry-run=client -o yaml | kubectl apply -f -

echo "==> Controller (${RELEASE_CONTROLLER}) di ${CONTROLLER_NS}"
helm upgrade --install "${RELEASE_CONTROLLER}" \
  "${OCI_BASE}/gha-runner-scale-set-controller" \
  "${VER_FLAG[@]}" \
  --namespace "${CONTROLLER_NS}" \
  -f "${SCRIPT_DIR}/values-controller.yaml" \
  --wait

echo "==> Runner scale set (${RELEASE_RUNNERSET}) di ${RUNNER_NS}"
helm upgrade --install "${RELEASE_RUNNERSET}" \
  "${OCI_BASE}/gha-runner-scale-set" \
  "${VER_FLAG[@]}" \
  --namespace "${RUNNER_NS}" \
  -f "${SCRIPT_DIR}/values-runner-set.yaml" \
  --wait

echo "==> Selesai. Verifikasi:"
echo "    kubectl get pods -n ${CONTROLLER_NS}"
echo "    kubectl get autoscalingrunnerset,ephemeralrunner -n ${RUNNER_NS}"
echo "    (di GitHub: Settings → Actions → Runners → runner scale set 'arc-simpel')"
