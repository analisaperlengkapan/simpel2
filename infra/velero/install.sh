#!/usr/bin/env bash
# Pasang / mutakhirkan Velero untuk SIMPel.
#
# Velero = komponen platform (binernya tidak di-vendor ke repo ini), tapi
# konfigurasinya ada di `infra/velero/values.yaml` dan ter-versi. Skrip ini
# adalah SATU-SATUNYA cara yang benar memasangnya, supaya klaster tak pelan-pelan
# menyimpang dari repo lewat perintah helm yang diketik ulang.
#
# Prasyarat: chart `infra/helm/backup` sudah ter-install (MinIO + bucket).
#
# Pakai:
#   MINIO_ROOT_USER=... MINIO_ROOT_PASSWORD=... ./infra/velero/install.sh
#
# Kredensial diambil dari ENV, bukan dari file di repo dan bukan dengan membaca
# balik Secret di klaster.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VALUES_FILE="${SCRIPT_DIR}/values.yaml"

VELERO_NS="${VELERO_NS:-velero}"
CHART_VERSION="${CHART_VERSION:-12.1.0}"
HELM_REPO_NAME="${HELM_REPO_NAME:-vmware-tanzu}"
HELM_REPO_URL="https://vmware-tanzu.github.io/helm-charts"

if [ -z "${MINIO_ROOT_USER:-}" ] || [ -z "${MINIO_ROOT_PASSWORD:-}" ]; then
  echo "ERROR: MINIO_ROOT_USER dan MINIO_ROOT_PASSWORD wajib di-set." >&2
  echo "       Nilainya sama dengan yang dipakai chart infra/helm/backup." >&2
  exit 2
fi

echo "==> namespace ${VELERO_NS}"
kubectl create namespace "${VELERO_NS}" --dry-run=client -o yaml | kubectl apply -f -

echo "==> Secret velero-minio"
# Server-side apply, BUKAN client-side. `kubectl apply` biasa menuliskan SELURUH
# objek — termasuk `data` — ke anotasi `last-applied-configuration`, sehingga
# kredensialnya tersimpan dua kali: sekali terenkripsi-at-rest sebagai Secret,
# sekali sebagai anotasi teks pada objek yang sama. Server-side apply menyimpan
# kepemilikan field di `managedFields` dan tidak menyalin nilainya.
kubectl create secret generic velero-minio \
  --namespace "${VELERO_NS}" \
  --from-literal=cloud="[default]
aws_access_key_id=${MINIO_ROOT_USER}
aws_secret_access_key=${MINIO_ROOT_PASSWORD}
" \
  --dry-run=client -o yaml \
  | kubectl apply --server-side --force-conflicts -f -

# Bersihkan jejak anotasi kalau Secret ini pernah dibuat dengan apply client-side.
kubectl annotate secret velero-minio --namespace "${VELERO_NS}" \
  kubectl.kubernetes.io/last-applied-configuration- >/dev/null 2>&1 || true

echo "==> helm repo ${HELM_REPO_NAME}"
helm repo add "${HELM_REPO_NAME}" "${HELM_REPO_URL}" >/dev/null 2>&1 || true
helm repo update "${HELM_REPO_NAME}" >/dev/null

echo "==> helm upgrade --install velero (chart ${CHART_VERSION})"
helm upgrade --install velero "${HELM_REPO_NAME}/velero" \
  --version "${CHART_VERSION}" \
  --namespace "${VELERO_NS}" \
  -f "${VALUES_FILE}" \
  "$@"

echo "==> menunggu BackupStorageLocation 'default' jadi Available"
# BSL yang `Unavailable` = setiap backup akan gagal. Gagal di sini, saat masih
# ada orang yang menonton, jauh lebih baik daripada baru ketahuan saat restore.
for _ in $(seq 1 30); do
  phase="$(kubectl -n "${VELERO_NS}" get backupstoragelocation default \
            -o jsonpath='{.status.phase}' 2>/dev/null || true)"
  if [ "${phase}" = "Available" ]; then
    echo "    BSL default: Available"
    exit 0
  fi
  sleep 5
done

echo "ERROR: BSL 'default' tidak Available setelah ~150 detik." >&2
echo "       Periksa: kubectl -n ${VELERO_NS} describe backupstoragelocation default" >&2
echo "       Penyebab tersering: bucket belum ada (Job minio-bucket-init gagal)," >&2
echo "       atau MinIO belum Ready." >&2
exit 1
