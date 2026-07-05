#!/usr/bin/env bash
# shellcheck shell=bash
#
# Secreton CI bootstrap — compose-native (no kubectl, no CLI binary).
#
# The runtime secreton image ships ONLY `api_server` (no `secreton` CLI), but it
# serves the REST API on :8200 (the Dockerfile healthcheck + infra/helm/
# bootstrap-secreton.sh both use it). This script drives that REST API to run the
# REAL Shamir init/unseal, then seeds a demo secret THROUGH THE GATEWAY so the
# write path (gateway → secreton gRPC StoreSecret) matches the gRPC read path
# simpelv1 uses at runtime — REST put_secret (state.engine) and gRPC get_secret
# (self.storage, HashMap<String,String> envelope) are different code paths, so
# seeding via the gateway is the only way to guarantee round-trip parity.
#
# CI-ONLY. The DB is ephemeral every run, so the unseal keys printed here are
# throwaway — they never protect a real secret. Do NOT reuse this flow for
# staging/production (there the keys are captured OFFLINE; see secreton-ops).
set -euo pipefail

SECRETON_ADDR="${SECRETON_ADDR:-http://secreton:8200}"
GATEWAY_ADDR="${GATEWAY_ADDR:-http://gateway:8090}"
SHARES="${SECRETON_SHARES:-5}"
THRESHOLD="${SECRETON_THRESHOLD:-3}"
# Flat name (no '/') to avoid %2F path-segment ambiguity at the gateway route.
SEED_PATH="${SECRETON_SEED_PATH:-simpelv1-app-key}"
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

# ── 4. Seed a secret THROUGH THE GATEWAY (gRPC write == gRPC read) ────────────
log "[4/5] seeding secret '${SEED_PATH}' via gateway ${GATEWAY_ADDR} ..."
for _ in $(seq 1 30); do
  curl -fsS "${GATEWAY_ADDR}/healthz" >/dev/null 2>&1 && break
  sleep 2
done
curl -fsS -X PUT "${GATEWAY_ADDR}/v1/secrets/${SEED_PATH}" \
  -H 'Content-Type: application/json' \
  -d "$(jq -nc --arg v "$SEED_VALUE" '{value:$v}')" >/dev/null \
  || die "seed PUT via gateway failed"

# ── 5. Verify readback via the gateway (asserts real state, not presence) ─────
log "[5/5] verifying readback via gateway ..."
got=$(curl -fsS "${GATEWAY_ADDR}/v1/secrets/${SEED_PATH}" | jq -r '.value // empty') \
  || die "readback GET via gateway failed"
[ "$got" = "$SEED_VALUE" ] || die "readback mismatch (secreton⇄gateway round-trip broke)"

# ── 6. (opt-in) provision the dynamic DB secrets engine + verify a lease ──────
# Enabled with SECRETON_DB_PROVISION=1 (needs the gateway started with
# GATEWAY_ENABLE_DB_ADMIN=1). Configures the admin connection Secreton uses to
# mint roles, creates the role perlengkapan leases, issues ONE short-lived
# credential, and proves it actually authenticates against Postgres — the full
# gateway → secreton gRPC → database-engine → real CREATE ROLE path. Exit code
# is the assertion, same as the secret round-trip above.
if [ "${SECRETON_DB_PROVISION:-}" = "1" ]; then
  log "[6/6] provisioning dynamic DB engine + verifying a leased credential ..."
  apk add --no-cache postgresql-client >/dev/null 2>&1 \
    || die "apk add postgresql-client failed"
  ADMIN_URL="${SECRETON_DB_ADMIN_URL:?SECRETON_DB_ADMIN_URL required when SECRETON_DB_PROVISION=1}"
  ROLE="${SECRETON_DB_ROLE:-perlengkapan-dyn}"
  CONN_NAME="${SECRETON_DB_CONN:-perlengkapan}"

  log "    configuring admin connection '${CONN_NAME}' ..."
  curl -fsS -X POST "${GATEWAY_ADDR}/v1/database/config/${CONN_NAME}" \
    -H 'Content-Type: application/json' \
    -d "$(jq -nc --arg url "$ADMIN_URL" '{connection_url:$url}')" >/dev/null \
    || die "configure database connection failed (is GATEWAY_ENABLE_DB_ADMIN=1 on the gateway?)"

  # SUPERUSER only because the ephemeral e2e database is throwaway and the leased
  # user must run every consumer query + migration; production uses a scoped
  # GRANT instead (see secreton-ops).
  create_sql='CREATE ROLE "{{username}}" WITH LOGIN SUPERUSER PASSWORD '\''{{password}}'\'';'
  revoke_sql='DROP ROLE IF EXISTS "{{username}}";'
  log "    creating role '${ROLE}' ..."
  curl -fsS -X POST "${GATEWAY_ADDR}/v1/database/roles/${ROLE}" \
    -H 'Content-Type: application/json' \
    -d "$(jq -nc --arg db "$CONN_NAME" --arg c "$create_sql" --arg r "$revoke_sql" \
          '{db_name:$db, default_ttl:60, max_ttl:3600, creation_statements:[$c], revocation_statements:[$r]}')" \
    >/dev/null \
    || die "create database role failed"

  log "    leasing a credential + authenticating it against Postgres ..."
  creds=$(curl -fsS "${GATEWAY_ADDR}/v1/database-credentials/${ROLE}") \
    || die "generate database credentials failed"
  dsn=$(printf '%s' "$creds" | jq -r '.credentials.connection_url // empty')
  [ -n "$dsn" ] || die "leased credential missing connection_url: ${creds}"
  # The leased user must be able to connect AND query — proves CREATE ROLE ran
  # for real, not that the RPC merely returned 200.
  [ "$(psql "$dsn" -tAc 'SELECT 1' 2>/dev/null | tr -d '[:space:]')" = "1" ] \
    || die "leased dynamic credential could not authenticate/query Postgres"
  log "    dynamic DB lease issued + verified ✓"
fi

log "✓ secreton bootstrap complete — Shamir unsealed + secret round-trips secreton⇄gateway"
