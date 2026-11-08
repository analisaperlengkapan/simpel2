#!/usr/bin/env bash
# cutover-production.sh — kosongkan sisa era pra-Helm di namespace produksi SEBELUM
# `helm install` pertama.
#
# KENAPA ADA: `simpelv2-production` dibuat dengan `kubectl apply` sebelum chart ini
# ada. `helm install` menolak mengambil objek yang bukan miliknya ("invalid
# ownership metadata"), dan 34 dari 89 objek chart bentrok nama dengan sisa itu
# (Deployment/Service authenc & portal, StatefulSet postgres & secreton, 6
# NetworkPolicy, ...). Menambal satu per satu berarti menyalin nama dari pandangan;
# skrip ini MENURUNKAN daftarnya dari cluster: semua objek namespaced yang
#   - tidak dimiliki Helm (label managed-by=Helm / anotasi meta.helm.sh/*), dan
#   - tidak dimiliki objek lain (ownerReferences kosong — sisanya ikut terhapus
#     oleh garbage collector), dan
#   - bukan prasyarat yang dibuat operator (Secret `ghcr-pull`).
# Plus VirtualService relik di istio-system.
#
# Ini SATU-SATUNYA tempat kubectl delete diizinkan menyentuh produksi: objeknya
# bukan milik rilis Helm mana pun, jadi tak ada cara Helm untuk menghapusnya.
#
# AMAN: default DRY-RUN (hanya mencetak). Menolak jalan bila
#   * sudah ada rilis Helm `simpel` di namespace itu (jangan pernah ke produksi hidup),
#   * ada PVC berstatus Bound (itu DATA, bukan relik),
#   * SIMPEL_CONFIRM_CUTOVER=yes tidak diset saat --apply.
#
# Pakai:
#   infra/helm/cutover-production.sh                       # dry-run, cetak daftar
#   SIMPEL_CONFIRM_CUTOVER=yes infra/helm/cutover-production.sh --apply
set -euo pipefail

NS="${NS:-simpelv2-production}"
ISTIO_NS="${ISTIO_NS:-istio-system}"
RELEASE="${RELEASE:-simpel}"
RELIC_VS="${RELIC_VS:-simpelv2-prod-routes}"
KEEP_SECRETS_RE='^(ghcr-pull)$'

APPLY=0
case "${1:-}" in
  "") ;;
  --apply) APPLY=1 ;;
  *) echo "usage: $0 [--apply]" >&2; exit 2 ;;
esac

die() { echo "ABORT: $*" >&2; exit 1; }
command -v kubectl >/dev/null || die "kubectl tidak ditemukan"
command -v jq >/dev/null || die "jq tidak ditemukan"
command -v helm >/dev/null || die "helm tidak ditemukan"

kubectl get ns "$NS" >/dev/null 2>&1 || die "namespace $NS tidak ada"

if helm status "$RELEASE" -n "$NS" >/dev/null 2>&1; then
  die "rilis Helm '$RELEASE' sudah ada di $NS — cutover hanya untuk install PERTAMA. Pakai helm upgrade."
fi

bound="$(kubectl -n "$NS" get pvc -o json | jq -r '.items[] | select(.status.phase=="Bound") | .metadata.name')"
if [[ -n "$bound" ]]; then
  echo "$bound" | sed 's/^/  PVC Bound: /' >&2
  die "ada PVC Bound di $NS — itu data, bukan sisa. Selidiki dulu; skrip tidak menyentuhnya."
fi

# Jenis objek namespaced yang bisa di-list & di-delete. Dikecualikan hanya yang
# dibuat/dihapus otomatis oleh controller (events, endpoints, ...).
mapfile -t KINDS < <(kubectl api-resources --namespaced=true --verbs=list,delete -o name \
  | grep -v -E '^(events|events\.events\.k8s\.io|endpoints|endpointslices\.discovery\.k8s\.io|controllerrevisions\.apps|leases\.coordination\.k8s\.io|bindings)$' \
  | sort)
[[ ${#KINDS[@]} -gt 0 ]] || die "api-resources kosong — tidak bisa menurunkan daftar"

declare -a TARGETS=()   # "kind<TAB>namespace<TAB>name"
for kind in "${KINDS[@]}"; do
  while IFS=$'\t' read -r name; do
    [[ -n "$name" ]] && TARGETS+=("$kind"$'\t'"$NS"$'\t'"$name")
  done < <(kubectl -n "$NS" get "$kind" -o json 2>/dev/null | jq -r --arg keep "$KEEP_SECRETS_RE" '
      .items[]
      | select((.metadata.ownerReferences // []) | length == 0)
      | select((.metadata.labels["app.kubernetes.io/managed-by"] // "") != "Helm")
      | select(.metadata.annotations["meta.helm.sh/release-name"] == null)
      | select(((.type // "") != "kubernetes.io/service-account-token"))
      | select(.metadata.name | test($keep) | not)
      | select(.metadata.name != "kube-root-ca.crt" and .metadata.name != "istio-ca-root-cert" and .metadata.name != "default")
      | .metadata.name')
done

# VirtualService relik di namespace gateway (bukan milik Helm → chart akan menolak).
if kubectl -n "$ISTIO_NS" get virtualservice "$RELIC_VS" >/dev/null 2>&1; then
  owner="$(kubectl -n "$ISTIO_NS" get virtualservice "$RELIC_VS" -o jsonpath='{.metadata.annotations.meta\.helm\.sh/release-name}')"
  [[ -z "$owner" ]] || die "VirtualService $ISTIO_NS/$RELIC_VS dimiliki rilis Helm '$owner' — bukan relik."
  TARGETS+=("virtualservices.networking.istio.io"$'\t'"$ISTIO_NS"$'\t'"$RELIC_VS")
fi

echo "== Sisa pra-Helm di $NS (+ $ISTIO_NS/$RELIC_VS): ${#TARGETS[@]} objek"
printf '%s\n' "${TARGETS[@]}" | awk -F'\t' '{printf "  %-52s %-22s %s\n",$1,$2,$3}'

if [[ ${#TARGETS[@]} -eq 0 ]]; then
  echo "Sudah bersih — tidak ada yang perlu dihapus."
  exit 0
fi
if [[ $APPLY -eq 0 ]]; then
  echo
  echo "DRY-RUN: tidak ada yang dihapus. Jalankan dengan --apply (dan SIMPEL_CONFIRM_CUTOVER=yes) untuk menghapus."
  exit 0
fi
[[ "${SIMPEL_CONFIRM_CUTOVER:-}" == "yes" ]] || die "--apply butuh SIMPEL_CONFIRM_CUTOVER=yes"

echo
echo "== Menghapus"
for t in "${TARGETS[@]}"; do
  IFS=$'\t' read -r kind ns name <<<"$t"
  kubectl -n "$ns" delete "$kind" "$name" --wait=false --ignore-not-found
done

echo "== Menunggu objek benar-benar hilang (maks 3 menit)"
deadline=$((SECONDS + 180))
while :; do
  left=0
  for t in "${TARGETS[@]}"; do
    IFS=$'\t' read -r kind ns name <<<"$t"
    if kubectl -n "$ns" get "$kind" "$name" >/dev/null 2>&1; then left=$((left + 1)); fi
  done
  [[ $left -eq 0 ]] && break
  [[ $SECONDS -lt $deadline ]] || die "$left objek belum hilang setelah 3 menit — periksa finalizer."
  sleep 5
done
echo "Selesai: namespace $NS siap untuk helm install (Namespace itu sendiri dan Secret ghcr-pull tidak disentuh)."
