#!/usr/bin/env bash
# migrate-env-to-secreton.sh — Migrasi nilai dari .env file ke Secreton kv store.
#
# Usage:
#   ./scripts/migrate-env-to-secreton.sh \
#     --env-file layanan/integrasi/.env \
#     --kv-prefix integrasi/tokens \
#     [--secreton-addr http://127.0.0.1:8200] \
#     [--namespace simpelv2-staging] \
#     [--dry-run]
#
# Pre-requisites:
#   - SECRETON_TOKEN env wajib (root atau token dengan write permission ke kv/data/<prefix>/*).
#   - Secreton sudah unsealed.
#   - kv engine sudah enabled di path "kv" (default Secreton mount).
#   - jq installed.
#
# Behavior:
#   - Parse KEY=VALUE dari .env (skip komentar, empty line, quoted values).
#   - Map ke Secreton path: <prefix>/<key-lowercase> dengan struktur {"data": {"value": "..."}}.
#     Atau group multi-keys ke single object jika --bundle flag diberikan.
#   - Verify read-back setelah write.

set -euo pipefail

ENV_FILE=""
KV_PREFIX=""
SECRETON_ADDR="${SECRETON_ADDR:-http://127.0.0.1:8200}"
NAMESPACE=""
BUNDLE=0
DRY_RUN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --env-file)        ENV_FILE="$2"; shift 2 ;;
    --kv-prefix)       KV_PREFIX="$2"; shift 2 ;;
    --secreton-addr)   SECRETON_ADDR="$2"; shift 2 ;;
    --namespace)       NAMESPACE="$2"; shift 2 ;;
    --bundle)          BUNDLE=1; shift ;;
    --dry-run)         DRY_RUN=1; shift ;;
    -h|--help)
      head -25 "$0"; exit 0 ;;
    *) echo "Unknown arg: $1"; exit 1 ;;
  esac
done

[[ -f "$ENV_FILE" ]] || { echo "ERROR: --env-file tidak ditemukan: $ENV_FILE"; exit 1; }
[[ -n "$KV_PREFIX" ]] || { echo "ERROR: --kv-prefix wajib"; exit 1; }
: "${SECRETON_TOKEN:?SECRETON_TOKEN env wajib}"
command -v jq >/dev/null || { echo "ERROR: jq required"; exit 1; }

api() {
  local method=$1 path=$2
  shift 2
  curl -fsS -X "$method" \
    -H "X-Secreton-Token: $SECRETON_TOKEN" \
    -H "Content-Type: application/json" \
    "$@" \
    "$SECRETON_ADDR/v1$path"
}

# Parse .env → key=value pairs (sederhana — tidak support multiline / shell expansion).
parse_env() {
  awk -F= '
    /^[[:space:]]*#/ {next}
    /^[[:space:]]*$/ {next}
    /^[A-Za-z_][A-Za-z0-9_]*=/ {
      key=$1
      sub(/^[A-Za-z_][A-Za-z0-9_]*=/, "", $0)
      val=$0
      gsub(/^["'\'']|["'\'']$/, "", val)
      print key"\t"val
    }
  ' "$ENV_FILE"
}

if [[ $BUNDLE -eq 1 ]]; then
  # Mode bundle: 1 path, 1 object berisi semua key.
  echo ">>> Bundle mode: write semua key ke kv/data/$KV_PREFIX"
  PAYLOAD=$(parse_env | jq -Rn '
    [inputs | split("\t") | {(.[0]): .[1]}] | add | {data: .}')
  echo "$PAYLOAD" | jq 'keys[]' >&2 || true
  if [[ $DRY_RUN -eq 1 ]]; then
    echo "DRY-RUN: payload="; echo "$PAYLOAD" | jq '.data | keys'
  else
    echo "$PAYLOAD" | api PUT "/kv/data/$KV_PREFIX" --data @-
    api GET "/kv/data/$KV_PREFIX" | jq '.data.data | keys'
  fi
else
  # Mode per-key: 1 path per env var.
  while IFS=$'\t' read -r KEY VAL; do
    LKEY=$(echo "$KEY" | tr '[:upper:]' '[:lower:]')
    PATH_FULL="$KV_PREFIX/$LKEY"
    if [[ $DRY_RUN -eq 1 ]]; then
      printf "DRY-RUN: PUT kv/data/%s = (%d chars)\n" "$PATH_FULL" "${#VAL}"
    else
      printf "  - %s ... " "$PATH_FULL"
      jq -nc --arg v "$VAL" '{data: {value: $v}}' \
        | api PUT "/kv/data/$PATH_FULL" --data @- >/dev/null
      printf "OK\n"
    fi
  done < <(parse_env)
fi

echo "✓ Migrasi selesai dari $ENV_FILE → kv/$KV_PREFIX"
[[ $DRY_RUN -eq 0 ]] && echo "Verifikasi: SECRETON_TOKEN=... curl $SECRETON_ADDR/v1/kv/data/$KV_PREFIX"
