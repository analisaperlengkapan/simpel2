#!/usr/bin/env bash
# Memasang (atau menyegarkan) kredensial yang dipakai pipeline rilis untuk deploy.
#
# Dijalankan SEKALI oleh pemilik repo, dari mesin yang punya `kubectl` (ke klaster) dan `gh` yang login
# sebagai admin repo, lalu diulang saat tokennya mendekati kedaluwarsa (skrip mencetak tanggalnya).
# Token TIDAK pernah dicetak: ia mengalir dari `kubectl create token` ke `gh secret set` lewat variabel
# shell dan stdin, dan hanya tanggal kedaluwarsanya yang ditampilkan.
#
#   infra/scripts/buat-kredensial-deploy.sh            # staging dan production
#   infra/scripts/buat-kredensial-deploy.sh staging    # satu environment saja
#
# Yang dikerjakannya, berurutan:
#   1. memastikan chart `deploy-access` (ServiceAccount `release-deployer`) terpasang lewat Helm;
#   2. membuat GitHub Environment `staging` dan `production` (production butuh PERSETUJUAN Anda sebelum
#      job-nya berjalan) dan membatasinya ke tag rilis (staging juga ke `main`), jadi job pull request
#      tak bisa membaca rahasianya;
#   3. menyimpan variabel KUBE_SERVER, KUBE_CA (sertifikat publik, bukan rahasia) dan BASE_URL;
#   4. menyimpan rahasia KUBE_TOKEN per environment: token ber-umur (default 1 tahun, dibatasi API server).
#
# Environment variable: REPO, DURATION, STAGING_BASE_URL, PRODUCTION_BASE_URL, KUBE_SERVER (bawaan: ClusterIP
# Service kubernetes), SKIP_HELM=1
set -euo pipefail

REPO="${REPO:-analisaperlengkapan/simpel2}"
DURATION="${DURATION:-8760h}"
SA="release-deployer"
SA_NS="default"
# Alamat API server yang dipakai RUNNER. Pod runner ARC memakai `dnsPolicy: None` dengan resolver publik
# (infra/helm/arc/values-runner-set.yaml), jadi nama `kubernetes.default.svc` TIDAK ter-resolve di sana
# (Deploy Check pertama gagal tepat di situ). ClusterIP Service `kubernetes` terjangkau tanpa DNS dan
# tercantum di SAN sertifikat API server; ia diambil dari klaster, bukan ditulis tangan.
KUBE_SERVER="${KUBE_SERVER:-}"
STAGING_BASE_URL="${STAGING_BASE_URL:-http://10.1.7.121}"
PRODUCTION_BASE_URL="${PRODUCTION_BASE_URL:-https://simpel.kejaksaan.go.id}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ "$#" -gt 0 ]; then ENVS=("$@"); else ENVS=(staging production); fi

for tool in kubectl gh python3; do
  command -v "$tool" > /dev/null || { echo "perlu $tool di PATH" >&2; exit 127; }
done
gh auth status > /dev/null 2>&1 || { echo "gh belum login (gh auth login)" >&2; exit 1; }

if [ "${SKIP_HELM:-0}" != "1" ]; then
  command -v helm > /dev/null || { echo "perlu helm di PATH (atau SKIP_HELM=1 bila chart sudah terpasang)" >&2; exit 127; }
  echo "→ memasang chart deploy-access (ServiceAccount $SA di namespace $SA_NS)"
  helm upgrade --install deploy-access "$HERE/../helm/deploy-access" --namespace "$SA_NS"
fi

if [ -z "$KUBE_SERVER" ]; then
  API_IP="$(kubectl -n default get service kubernetes -o jsonpath='{.spec.clusterIP}')"
  [ -n "$API_IP" ] || { echo "ClusterIP Service kubernetes tak terbaca; isi KUBE_SERVER sendiri" >&2; exit 1; }
  KUBE_SERVER="https://${API_IP}:443"
fi
echo "→ alamat API server untuk runner: $KUBE_SERVER"

# Sertifikat CA API server: publik, diambil dari kubeconfig yang sedang aktif.
CA="$(kubectl config view --raw --minify -o jsonpath='{.clusters[0].cluster.certificate-authority-data}')"
[ -n "$CA" ] || { echo "kubeconfig aktif tak memuat certificate-authority-data" >&2; exit 1; }
USER_ID="$(gh api user --jq .id)"

for E in "${ENVS[@]}"; do
  case "$E" in
    staging)    BASE_URL="$STAGING_BASE_URL";    TAGS='v*-rc*'; REVIEWERS='[]' ;;
    production) BASE_URL="$PRODUCTION_BASE_URL"; TAGS='v*';     REVIEWERS="[{\"type\":\"User\",\"id\":${USER_ID}}]" ;;
    *) echo "environment tak dikenal: $E (staging | production)" >&2; exit 2 ;;
  esac
  echo "→ environment $E"
  printf '{"reviewers":%s,"deployment_branch_policy":{"protected_branches":false,"custom_branch_policies":true}}' "$REVIEWERS" \
    | gh api -X PUT "repos/$REPO/environments/$E" --input - > /dev/null
  # Tag yang boleh memakai rahasia environment ini; 422 bila kebijakannya sudah ada = tak apa.
  gh api -X POST "repos/$REPO/environments/$E/deployment-branch-policies" -f name="$TAGS" -f type=tag > /dev/null 2>&1 || true
  # Staging juga boleh dibaca job di `main` (kode yang sudah ter-merge), supaya workflow Deploy Check bisa
  # menguji sambungannya tanpa rilis. Production TIDAK: hanya tag final.
  if [ "$E" = staging ]; then
    gh api -X POST "repos/$REPO/environments/$E/deployment-branch-policies" -f name=main -f type=branch > /dev/null 2>&1 || true
  fi

  gh variable set KUBE_SERVER --env "$E" --repo "$REPO" --body "$KUBE_SERVER"
  gh variable set KUBE_CA     --env "$E" --repo "$REPO" --body "$CA"
  gh variable set BASE_URL    --env "$E" --repo "$REPO" --body "$BASE_URL"

  TOKEN="$(kubectl -n "$SA_NS" create token "$SA" --duration="$DURATION")"
  printf '%s' "$TOKEN" | gh secret set KUBE_TOKEN --env "$E" --repo "$REPO"
  EXP="$(TOKEN="$TOKEN" python3 -c 'import base64,json,os,time
p=os.environ["TOKEN"].split(".")[1]; p+="="*(-len(p)%4)
print(time.strftime("%Y-%m-%d", time.gmtime(json.loads(base64.urlsafe_b64decode(p))["exp"])))')"
  unset TOKEN
  echo "  KUBE_TOKEN tersimpan untuk $E, kedaluwarsa $EXP. Ulangi skrip ini sebelum tanggal itu."
done
echo "selesai."
