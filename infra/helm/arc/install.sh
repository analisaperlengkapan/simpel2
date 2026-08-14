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

echo "==> PriorityClass arc-ci-runner (cluster-scoped)"
# Runner CI berbagi node dengan produksi, jadi butuh urutan mengalah yang tegas.
#
# NILAI NEGATIF, dan itu disengaja: pod tanpa priorityClassName bernilai 0.
# Kelas positif berapa pun akan membuat runner MENGALAHKAN pod produksi yang
# tak punya kelas — kebalikan dari yang kita mau. -100 menaruh runner di bawah
# segalanya tanpa perlu menyentuh satu pun manifest produksi.
#
# preemptionPolicy: Never → runner tak pernah menggusur pod lain; ia hanya bisa
# digusur. Beban lain (default PreemptLowerPriority) bebas mengambil kembali
# resource dengan menggusur runner saat butuh menjadwal.
#
# Chart gha-runner-scale-set upstream tak men-template PriorityClass, dan objek
# ini cluster-scoped — jadi tempatnya di sini, sederet dgn namespace & label PSA
# yang juga prasyarat. Idempotent (apply).
kubectl apply -f - <<'EOF'
apiVersion: scheduling.k8s.io/v1
kind: PriorityClass
metadata:
  name: arc-ci-runner
value: -100
globalDefault: false
preemptionPolicy: Never
description: >-
  Runner CI ephemeral (ARC). Nilai negatif = di bawah setiap beban lain,
  termasuk pod produksi tanpa priorityClassName (yang bernilai 0). Runner
  mengalah agar produksi bisa memakai kapasitas kedua node kapan pun perlu.
EOF

echo "==> Pemanen runner zombie (CronJob + RBAC) di ${RUNNER_NS}"
# ARC bisa meninggalkan EphemeralRunner yang memegang penugasan job yang sudah
# mati di sisi GitHub; runner itu ikut dihitung sebagai kapasitas aktif sehingga
# listener berhenti menaikkan jumlah runner dan CI menggantung. Alasan lengkap
# + penurunan angka ambangnya ada di reaper.yaml. Idempotent (apply).
kubectl apply -f "${SCRIPT_DIR}/reaper.yaml"

echo "==> Secret PAT (arc-github-token) di ${RUNNER_NS}"
# `kubectl apply` client-side menyimpan SELURUH objek yang di-apply ke annotation
# `kubectl.kubernetes.io/last-applied-configuration` — termasuk `data`, jadi PAT
# berakhir DUA KALI di objek yang sama: sekali di `data`, sekali lagi terbaca
# jelas di metadata.
#
# Yang punya `get secret` memang bisa membaca `data` juga, jadi ini bukan jalur
# akses baru. Yang berubah adalah SEBARANNYA: metadata ikut terbawa ke tempat
# orang tak menyangka ada kredensial di sana — backup Velero, `kubectl get -o yaml`
# yang ditempel ke tiket, diff manifest. Kredensial sebaiknya hanya berada di
# satu tempat yang memang diperlakukan sebagai rahasia.
#
# Server-side apply tidak menulis annotation itu sama sekali (kepemilikan field
# dilacak di `managedFields`), dan tetap idempotent seperti apply biasa.
kubectl create secret generic arc-github-token \
  --namespace "${RUNNER_NS}" \
  --from-literal=github_token="${GITHUB_PAT}" \
  --dry-run=client -o yaml \
  | kubectl apply --server-side --force-conflicts -f -

# Bersihkan sisa dari instalasi client-side sebelumnya. Tanpa ini, salinan lama
# PAT tetap tertinggal di metadata meski instalasi berikutnya sudah server-side.
# No-op bila annotation-nya memang tak ada.
kubectl annotate secret arc-github-token \
  --namespace "${RUNNER_NS}" \
  kubectl.kubernetes.io/last-applied-configuration- >/dev/null 2>&1 || true

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
