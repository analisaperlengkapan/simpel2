#!/usr/bin/env bash
#
# integrasi-migrate applies EVERY migration on EVERY invocation — it keeps no
# `schema_migrations` table and skips nothing (see layanan/integrasi/src/bin/
# migrate.rs). Its own doc comment states the migrations "are idempotent … so
# re-running is safe". Nothing tested that claim, and it stopped being true:
#
#   003 does `DROP VIEW IF EXISTS integrasi.v_satker_code_map_auto`
#   004 does `CREATE MATERIALIZED VIEW mv_satker_code_map_auto AS SELECT * FROM` it
#
# so once 004 had run anywhere, 003's DROP failed with "other objects depend on
# it" and integrasi-migrate could never succeed again. It blocked the rc28
# staging upgrade outright.
#
# CI never caught it because CI only ever migrates a FRESH database — every e2e
# stack starts empty, and a first run is exactly the case that works. Staging
# and production are never fresh. So this applies the migrations TWICE to the
# same database: pass 1 is what CI already proved, pass 2 is what every real
# upgrade actually does.
#
# authenc and perlengkapan are deliberately out of scope: both TRACK applied
# migrations (authenc `schema_migrations`, perlengkapan refinery
# `refinery_schema_history`) and skip what has run, so replay is a no-op by
# construction. integrasi is the only one that replays.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MIGRATE_RS="$ROOT/layanan/integrasi/src/bin/migrate.rs"
MIG_DIR="$ROOT/layanan/integrasi/migrations"
IMAGE="${POSTGRES_IMAGE:-postgres:16-alpine}"

# The order applied here is DERIVED from the binary's own MIGRATIONS array, not
# from a glob and not from a list kept in this file — so what we test is exactly
# what production runs, in exactly its order.
mapfile -t FILES < <(grep -oE 'include_str!\("\.\./\.\./migrations/[^"]+"\)' "$MIGRATE_RS" \
                     | sed -E 's|.*/migrations/([^"]+)"\)|\1|')

if [ "${#FILES[@]}" -eq 0 ]; then
  echo "GAGAL: tak satu pun migrasi terbaca dari $(basename "$MIGRATE_RS")."
  echo "Itu bukan 'tak ada migrasi' — itu bentuk berkasnya berubah dan"
  echo "pemeriksaan ini berhenti mengukur apa pun."
  exit 1
fi

# A migration file that exists on disk but is not in the array would never be
# applied by anything — written, committed, and silently inert.
missing_from_array=()
for f in "$MIG_DIR"/*.sql; do
  n="$(basename "$f")"
  case "$n" in *_rollback.sql) continue ;; esac   # down-migration, applied by hand
  found=0
  for m in "${FILES[@]}"; do [ "$m" = "$n" ] && found=1 && break; done
  [ "$found" -eq 0 ] && missing_from_array+=("$n")
done
if [ "${#missing_from_array[@]}" -gt 0 ]; then
  echo "GAGAL: berkas migrasi ada di disk tapi tak terdaftar di MIGRATIONS:"
  printf '  - %s\n' "${missing_from_array[@]}"
  echo "Berkas seperti itu tak pernah dijalankan oleh apa pun."
  exit 1
fi

for m in "${FILES[@]}"; do
  [ -f "$MIG_DIR/$m" ] || { echo "GAGAL: MIGRATIONS menyebut $m, yang tak ada di disk."; exit 1; }
done

CONTAINER="migration-replay-$$"
cleanup() { docker rm -f "$CONTAINER" >/dev/null 2>&1 || true; }
trap cleanup EXIT

echo "Menjalankan ${#FILES[@]} migrasi integrasi dua kali di $IMAGE…"
docker run -d --rm --name "$CONTAINER" \
  -e POSTGRES_PASSWORD=replay -e POSTGRES_DB=replaytest "$IMAGE" >/dev/null

# `pg_isready` is NOT proof of readiness for this image, and trusting it made
# this job flaky: the official postgres entrypoint starts a TEMPORARY server on
# a unix socket to run initdb, answers pg_isready from it, then shuts it down
# and starts the real one. A client that connects inside that window is dropped
# mid-statement — which is what happened on PR #856 while the identical script
# passed on #850.
#
# So wait for what we actually need: a real query, against the real target
# database, succeeding three times in a row. The repetition is the point — one
# success can be the doomed temporary server; three across ~2s cannot.
ready=0
for _ in $(seq 90); do
  if docker exec "$CONTAINER" psql -U postgres -d replaytest -tAc 'SELECT 1' >/dev/null 2>&1; then
    ready=$((ready + 1))
    [ "$ready" -ge 3 ] && break
  else
    ready=0
  fi
  sleep 1
done
if [ "$ready" -lt 3 ]; then
  echo "GAGAL: postgres tak pernah siap menerima kueri. Log kontainer:"
  docker logs "$CONTAINER" 2>&1 | tail -20 | sed 's/^/    /'
  exit 1
fi

rc=0
for pass in 1 2; do
  echo "--- pass $pass ---"
  for m in "${FILES[@]}"; do
    # --single-transaction is not tidiness: `batch_execute` hands Postgres the
    # whole file as ONE implicit transaction, so a migration is all-or-nothing
    # and statements Postgres forbids inside a transaction block (CREATE INDEX
    # CONCURRENTLY, VACUUM, ALTER SYSTEM) fail there. psql's default — each
    # statement in its own transaction — accepts all of them, so without this
    # flag the guard would certify a migration the deploy hook cannot run.
    # See layanan/integrasi/AGENTS.md §0 for the rule this enforces.
    if out=$(docker exec -i "$CONTAINER" \
               psql -U postgres -d replaytest --single-transaction \
                    -v ON_ERROR_STOP=1 -q < "$MIG_DIR/$m" 2>&1); then
      echo "  ok    $m"
    else
      echo "  GAGAL $m"
      # Print the REASON, not just the filename. Filtering to ^ERROR/DETAIL/HINT
      # silently produced nothing whenever psql failed for a reason that is not
      # a server-side rejection — a dropped connection, a missing database, psql
      # itself erroring — and the job then reported `GAGAL 001_init_schema.sql`
      # with no explanation at all. That is the same defect #849 fixed in the
      # migrate binary: naming the file and withholding everything needed to act.
      reason=$(printf '%s\n' "$out" | grep -E '^(ERROR|DETAIL|HINT|FATAL|psql:)' || true)
      [ -z "$reason" ] && reason=$(printf '%s\n' "$out" | tail -20)
      printf '%s\n' "$reason" | sed 's/^/        /'
      rc=1
    fi
  done
done

if [ "$rc" -ne 0 ]; then
  echo
  echo "Migrasi integrasi TIDAK bisa dijalankan ulang. Karena integrasi-migrate"
  echo "menerapkan semuanya di setiap invokasi, ini berarti setiap upgrade"
  echo "setelah yang pertama akan gagal — hook pre-upgrade membatalkan deploy."
  exit 1
fi

echo
echo "OK: ${#FILES[@]} migrasi, dua kali berturut-turut, bersih."
