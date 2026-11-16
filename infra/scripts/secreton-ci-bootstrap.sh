#!/usr/bin/env bash
# shellcheck shell=bash
#
# Secreton CI bootstrap — compose-native (no kubectl, no CLI binary).
#
# The runtime secreton image ships ONLY `api_server` (no `secreton` CLI), but it
# serves the REST API on :8200 (the Dockerfile healthcheck + infra/helm/
# bootstrap-secreton.sh both use it). This script drives that REST API with the
# root token init returns: REAL Shamir init/unseal and a KV write + read-back.
# Opt-in, it then drives the dynamic database secrets engine over gRPC (the path
# layanan-perlengkapan uses) down to a leased credential that logs in to Postgres.
#
# CI-ONLY. The DB is ephemeral every run, so the unseal keys printed here are
# throwaway — they never protect a real secret. Do NOT reuse this flow for
# staging/production (there the keys are captured OFFLINE; see secreton-ops).
set -euo pipefail

SECRETON_ADDR="${SECRETON_ADDR:-http://secreton:8200}"
SHARES="${SECRETON_SHARES:-5}"
THRESHOLD="${SECRETON_THRESHOLD:-3}"
SEED_PATH="${SECRETON_SEED_PATH:-ci-bootstrap/probe}"
SEED_VALUE="${SECRETON_SEED_VALUE:-ci-bootstrap-secret}"

log() { printf '>>> %s\n' "$*"; }
die() { printf '::error::secreton-ci-bootstrap: %s\n' "$*" >&2; exit 1; }

# ── 1. Wait for the secreton REST API ────────────────────────────────────────
log "[1/5] waiting for secreton REST at ${SECRETON_ADDR} ..."
ss=
for _ in $(seq 1 60); do
  if ss=$(curl -fsS "${SECRETON_ADDR}/v1/sys/seal-status" 2>/dev/null); then break; fi
  ss=
  sleep 2
done
[ -n "$ss" ] || die "secreton REST unreachable at ${SECRETON_ADDR}/v1/sys/seal-status"

# ── 2. Initialize (Shamir) if not already initialized ────────────────────────
initialized=$(printf '%s' "$ss" | jq -r '.initialized // false')
if [ "$initialized" = "true" ]; then
  # A fresh CI database should never be pre-initialized. If it is, we cannot
  # unseal (keys were generated on a run whose keys we no longer hold), so fail
  # loudly rather than hang.
  die "secreton already initialized on a supposedly fresh CI DB — cannot unseal without keys"
fi
log "[2/5] initializing engine (shares=${SHARES} threshold=${THRESHOLD}) ..."
init=$(curl -fsS -X POST "${SECRETON_ADDR}/v1/sys/init" \
  -H 'Content-Type: application/json' \
  -d "{\"secret_shares\":${SHARES},\"secret_threshold\":${THRESHOLD}}") \
  || die "init request failed"
mapfile -t KEYS < <(printf '%s' "$init" | jq -r '.keys[]')
[ "${#KEYS[@]}" -ge "$THRESHOLD" ] || die "init returned ${#KEYS[@]} keys, need >= ${THRESHOLD}"
ROOT_TOKEN=$(printf '%s' "$init" | jq -r '.root_token // empty')
[ -n "$ROOT_TOKEN" ] || die "init returned no root_token"

# ── 3. Unseal with THRESHOLD shares ──────────────────────────────────────────
log "[3/5] unsealing with ${THRESHOLD} of ${#KEYS[@]} shares ..."
for k in "${KEYS[@]:0:${THRESHOLD}}"; do
  curl -fsS -X POST "${SECRETON_ADDR}/v1/sys/unseal" \
    -H 'Content-Type: application/json' \
    -d "$(jq -nc --arg key "$k" '{key:$key}')" >/dev/null || die "unseal request failed"
done
# NOTE: no `// true` fallback here — jq's `//` treats `false` itself as empty,
# so `.sealed // true` returns "true" exactly when unseal SUCCEEDED (sealed=false).
# Plain `.sealed` keeps fail-closed semantics: a missing key prints "null" ≠ "false".
sealed=$(curl -fsS "${SECRETON_ADDR}/v1/sys/seal-status" | jq -r '.sealed')
[ "$sealed" = "false" ] || die "engine still sealed after ${THRESHOLD} shares (seal-status .sealed=${sealed})"
log "    unsealed ✓"

# Every call below is behind the REST auth middleware: bearer = the root token.
api() { curl -fsS -H "Authorization: Bearer ${ROOT_TOKEN}" -H 'Content-Type: application/json' "$@"; }

# ── 4. Write a secret ────────────────────────────────────────────────────────
log "[4/5] writing secret '${SEED_PATH}' ..."
api -X POST "${SECRETON_ADDR}/v1/secret/data/${SEED_PATH}" \
  -d "$(jq -nc --arg v "$SEED_VALUE" '{data:{value:$v}}')" >/dev/null \
  || die "secret write failed"

# ── 5. Read it back (asserts real state, not presence) ───────────────────────
log "[5/5] verifying read-back ..."
got=$(api "${SECRETON_ADDR}/v1/secret/data/${SEED_PATH}" | jq -r '.data.data.value // empty') \
  || die "secret read-back failed"
[ "$got" = "$SEED_VALUE" ] || die "read-back mismatch: wrote '${SEED_VALUE}', read '${got}'"
log "    secret round-trip ✓"

# ── 6. (opt-in) provision the dynamic DB secrets engine + verify a lease ──────
# Enabled with SECRETON_DB_PROVISION=1. Configures the admin connection Secreton
# uses to mint roles, creates the role perlengkapan leases, issues ONE
# short-lived credential, and proves it actually authenticates against Postgres.
# Exit code is the assertion, same as the secret round-trip above.
#
# These three calls go over gRPC, not REST: gRPC is the path layanan-perlengkapan
# leases its database credential on (`SecretonClient` in its main.rs). The REST
# route for the same lease (`GET /v1/dynamic/database/creds/{role}`) answers 500,
# "relation \"leases\" does not exist", so it is not the path to prove here.
# The compose listener is plaintext (GRPC_ALLOW_INSECURE=true), and the proto
# files are mounted at /proto.
if [ "${SECRETON_DB_PROVISION:-}" = "1" ]; then
  log "[6/6] provisioning dynamic DB engine over gRPC + verifying a leased credential ..."
  apk add --no-cache postgresql-client >/dev/null 2>&1 \
    || die "apk add postgresql-client failed"
  command -v grpcurl >/dev/null || die "grpcurl missing (the bootstrap image must be fullstorydev/grpcurl:*-alpine)"
  ADMIN_URL="${SECRETON_DB_ADMIN_URL:?SECRETON_DB_ADMIN_URL required when SECRETON_DB_PROVISION=1}"
  GRPC_ADDR="${SECRETON_GRPC_ADDR:-secreton:50053}"
  ROLE="${SECRETON_DB_ROLE:-perlengkapan-dyn}"
  CONN_NAME="${SECRETON_DB_CONN:-perlengkapan}"
  rpc() { grpcurl -plaintext -import-path /proto -proto secreton.proto -d "$2" "$GRPC_ADDR" "secreton.v1.SecretonService/$1"; }

  log "    configuring admin connection '${CONN_NAME}' ..."
  rpc ConfigureDatabaseConnection "$(jq -nc --arg n "$CONN_NAME" --arg url "$ADMIN_URL" \
        '{name:$n, db_type:"POSTGRESQL", connection_url:$url, max_open_connections:4,
          max_idle_connections:2, max_connection_lifetime:3600, verify_connection:true}')" >/dev/null \
    || die "configure database connection failed"

  # SUPERUSER only because the ephemeral e2e database is throwaway and the leased
  # user must run every consumer query + migration; production uses a scoped
  # GRANT instead (see secreton-ops).
  create_sql='CREATE ROLE "{{username}}" WITH LOGIN SUPERUSER PASSWORD '\''{{password}}'\'';'
  revoke_sql='DROP ROLE IF EXISTS "{{username}}";'
  log "    creating role '${ROLE}' ..."
  rpc CreateDatabaseRole "$(jq -nc --arg r "$ROLE" --arg db "$CONN_NAME" --arg c "$create_sql" --arg v "$revoke_sql" \
        '{role_name:$r, db_name:$db, default_ttl:60, max_ttl:3600, creation_statements:[$c], revocation_statements:[$v]}')" >/dev/null \
    || die "create database role failed"

  log "    leasing a credential + authenticating it against Postgres ..."
  creds=$(rpc GenerateDatabaseCredentials "$(jq -nc --arg r "$ROLE" '{role_name:$r}')") \
    || die "generate database credentials failed"
  dsn=$(printf '%s' "$creds" | jq -r '.credentials.connectionUrl // empty')
  [ -n "$dsn" ] || die "leased credential missing connection_url: $(printf '%s' "$creds" | jq -c 'del(.credentials.password)')"
  # The leased user must be able to connect AND query — proves CREATE ROLE ran
  # for real, not that the RPC merely returned OK.
  [ "$(psql "$dsn" -tAc 'SELECT 1' 2>/dev/null | tr -d '[:space:]')" = "1" ] \
    || die "leased dynamic credential could not authenticate/query Postgres"
  log "    dynamic DB lease issued + verified ✓"
fi

log "✓ secreton bootstrap complete — Shamir unsealed, secret round-trips over REST"
