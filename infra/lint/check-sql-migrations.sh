#!/usr/bin/env bash
#
# check-sql-migrations.sh — static guard for SQL migration anti-patterns.
#
# WHY THIS EXISTS (and why it is NOT a generic SQL linter):
#   SIMPel's migration runners apply ONE migration file inside ONE implicit
#   transaction:
#     - authenc   → `authenc-migrate` binary, `client.batch_execute(file)`
#     - integrasi → `integrasi-migrate` binary, `client.batch_execute(file)`
#     - perlengkapan → refinery `embed_migrations!` (per-migration txn)
#     - secreton  → migration files not yet wired to a runner; same rules apply
#                   the moment one is adopted.
#   Under that model a handful of constructs SILENTLY break fresh-apply in ways a
#   generic SQL linter (sqlfluff/style) does not model — these exact traps cost
#   real time in F5-B (authenc-040 `ALTER DATABASE CURRENT`, perlengkapan
#   V007/V008 explicit COMMIT, the `pg_dump`-default search_path reset that broke
#   refinery's history INSERT). This guard encodes only those high-signal,
#   runner-specific traps. Correctness beyond this is covered by the real-Postgres
#   fresh-apply gate (`test-services` in ci.yml), which this complements (it is
#   faster and covers migration dirs that gate does not fresh-apply).
#
# SCOPE: layanan/*/migrations/**/*.sql only — the runner-applied sources.
#   NOT scripts/ or tests/load/optimizations/ (run by psql directly, different
#   transaction semantics) and NOT a style check on the squashed pg_dump baselines.
#
# ESCAPE HATCH: append `-- guard:allow` to a line to waive it (sparingly, with a
#   reason in an adjacent comment).
#
# Deps: bash + grep only. Runtime: <1s. Exit 1 on any finding.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

shopt -s nullglob
mapfile -t FILES < <(find layanan/*/migrations -type f -name '*.sql' 2>/dev/null | sort)

if [[ ${#FILES[@]} -eq 0 ]]; then
  echo "check-sql-migrations: no migration SQL found under layanan/*/migrations/ — nothing to check."
  exit 0
fi

findings=0

# check <description> <ERE pattern>
# Matches the pattern (case-insensitive) against each line's CODE portion (text
# before the first `--`), so comments never produce false positives. Truncating at
# `--` can only hide a match (false negative on contrived string literals), never
# invent one. Lines bearing `-- guard:allow` are skipped.
check() {
  local desc="$1" pat="$2"
  local f line code lineno
  for f in "${FILES[@]}"; do
    lineno=0
    while IFS= read -r line || [[ -n "$line" ]]; do
      lineno=$((lineno + 1))
      [[ "$line" == *"-- guard:allow"* ]] && continue
      code="${line%%--*}"
      [[ -z "${code//[[:space:]]/}" ]] && continue
      if grep -qiE "$pat" <<<"$code"; then
        printf '::error file=%s,line=%s::[%s] %s\n' "$f" "$lineno" "$desc" "${line#"${line%%[![:space:]]*}"}"
        printf '  %s:%s\n      %s\n' "$f" "$lineno" "${line#"${line%%[![:space:]]*}"}" >&2
        findings=$((findings + 1))
      fi
    done <"$f"
  done
}

echo "check-sql-migrations: scanning ${#FILES[@]} file(s) under layanan/*/migrations/ ..."

# 1) CREATE/DROP INDEX ... CONCURRENTLY — cannot run inside a transaction, so it
#    aborts the whole migration. (REFRESH MATERIALIZED VIEW CONCURRENTLY inside a
#    function body is fine and is intentionally NOT matched.) On an empty/fresh DB
#    the lock CONCURRENTLY avoids is irrelevant — drop it.
check "CONCURRENTLY index DDL cannot run inside a single-transaction migration" \
  '(create|drop)[[:space:]]+(unique[[:space:]]+)?index\b.*concurrently'

# 2) Explicit transaction control — the runner already wraps the file in ONE
#    implicit transaction; a top-level BEGIN/COMMIT/ROLLBACK/START TRANSACTION
#    breaks that framing (PL/pgSQL block `BEGIN` has no trailing `;` and is not
#    matched).
check "explicit transaction control (runner already wraps the file in one txn)" \
  '^[[:space:]]*(begin|commit|rollback|start[[:space:]]+transaction)[[:space:]]*;'

# 3) search_path reset — `set_config('search_path','',...)` is what `pg_dump`
#    emits by DEFAULT at the top of a dump; left in a migration it resets the
#    search_path so the runner's UNQUALIFIED history INSERT (refinery
#    `refinery_schema_history`, authenc history table) fails and the migration
#    rolls back. Strip it when squashing a baseline from pg_dump output.
check "search_path reset breaks the migration-history INSERT (pg_dump default — strip it)" \
  "set_config\([[:space:]]*'search_path'[[:space:]]*,[[:space:]]*''"

# 4) ALTER DATABASE CURRENT — invalid syntax: PostgreSQL has no CURRENT keyword
#    for ALTER DATABASE (verified: ERROR `database "current" does not exist`). Set
#    search_path via the connection options / role, not by altering the database.
check "ALTER DATABASE CURRENT is invalid PostgreSQL syntax" \
  'alter[[:space:]]+database[[:space:]]+current\b'

if ((findings > 0)); then
  {
    echo ""
    echo "✖ ${findings} SQL migration anti-pattern(s) found."
    echo "  These break fresh-apply under the single-transaction migration runners."
    echo "  See add-backend-feature SKILL → migration conventions, and infra/AGENTS.md."
    echo "  To intentionally waive one line, append '-- guard:allow' (with a reason)."
  } >&2
  exit 1
fi

echo "✓ no SQL migration anti-patterns found."
