---
name: add-backend-feature
description: Add or extend a backend feature in an Axum service under layanan/ (most often layanan/perlengkapan) — the full vertical slice route → handler → service → repository → refinery migration → test, following the feature-module layout and migration conventions. Use when adding/changing an endpoint, a DB-backed capability, or a schema migration. Covers extractor ordering, the split-module layout, and idempotent/no-ALTER migration rules.
---

# Add a backend feature (SIMPel `layanan/`)

Read first: `layanan/AGENTS.md` + the service's `layanan/<svc>/AGENTS.md`
("Add New API Endpoint", layout). This Skill is the end-to-end procedure +
the gotchas; the per-service deltas live there.

## Vertical slice (perlengkapan-style feature module)

A feature lives in `src/<modul>/` with `handlers · models · services · repository`
(+ `mod.rs`). Large files split into directories with a re-exporting `mod.rs` (keep
`crate::<modul>::*` paths stable). Steps:

1. **Route** — register in the service's router (`routes.rs` / `app/router.rs`),
   pointing at `crate::<modul>::<handler>`.
2. **Handler** (`<modul>/handlers.rs` or `handlers/<group>.rs`) — keep it thin.
   **axum extractor order matters:** `State` first, then custom extractors
   (`Claims`), then `Path`/`Query`, with `Json(body)` **last**.
3. **Service** (`<modul>/services…`) — business rules; returns domain types.
4. **Repository** (`<modul>/repository…`) — SQL + row mapping. Row→struct mapping
   (`from_row`/`FromPgRow`) lives **here**, not in `lib-perlengkapan` (WASM-safe).
   Cross-module service calls go through `lib_perlengkapan` DTOs / port traits in
   `layanan/perlengkapan/src/contracts.rs`.
5. **Migration** — see conventions below.
6. **Test** — unit tests next to the service (mockall for repo traits); cross-module
   flows in `tests/`. Run `cargo test -p <crate>`.

Verify: `cargo check -p <crate> --all-targets` + `cargo fmt --check` + tests green.

## Refinery migration conventions

Schema lives in `<svc>/migrations/` as **`V###__name.sql`** (perlengkapan) or
`NNN_name.sql` (authenc/integrasi), embedded & run on boot / by the migrate runner.
- **NEVER `ALTER` a migration after it's merged** — append a new file.
- Numbering is sequential & unique per service (duplicate = refinery error).
- **Seed migrations must be idempotent** (UPSERT / `ON CONFLICT DO NOTHING`), since
  they may re-run across environments.

### 🔒 Expand/contract — additive first (WAJIB, F-REL)

Migrations must be **expand/contract** so that **`helm rollback` of the app never
requires a DB restore** (anti data-loss). The previous (rollback-target) code version
must keep working on the new schema.
- **This release = EXPAND only:** `ADD COLUMN` nullable/defaulted, new tables/indexes,
  `CREATE … IF NOT EXISTS`. No destructive change in the same release as the code that
  still needs the old shape.
- **CONTRACT (drop/NOT-NULL/rename) goes in a LATER release**, after no deployed/
  rollback-able version uses the old shape. Separate by ≥1 release.
- **Rename = 3 steps across releases:** add new col + dual-write → backfill + read new
  → drop old. Never a one-shot `ALTER … RENAME`.
- Operational rationale + DB-restore-last-resort: `infra/AGENTS.md` → Rollback.

### Topology & ordering (shared `dbsimpelv2`, schema-per-service)

- The trio **authenc + integrasi + perlengkapan** share DB `dbsimpelv2`,
  **schema-per-service**; reads cross-schema (e.g. perlengkapan → `integrasi.*`/
  `authenc.*`, FK `batch_operation_log.user_id → authenc.users`). secreton & simpelv1
  are isolated DBs. See `layanan/AGENTS.md` → Database Architecture.
- **Bring-up order: `integrasi-migrate` → `authenc-migrate` → perlengkapan** (cross-
  schema FK needs upstream schema first); wired in docker-compose + Helm hook Jobs.
- Extensions go in `public` (`CREATE EXTENSION … WITH SCHEMA public`); each service sets
  `search_path=<schema>,public`. Don't let an extension land in a service schema.
- `integrasi` migrations are embedded into the **`integrasi-migrate`** binary via
  `include_str!` (rebuild the binary after editing the SQL). authenc uses
  **`authenc-migrate`**; perlengkapan self-migrates on boot.
- Naming: `satker_id`/`satker_nama`/`satker_pusat_id`, `nama_barang`/`nama_pegawai`.
- Fresh-apply baselines exist (F5-B): authenc `001_baseline`+`002_seed`, perlengkapan
  `V001__baseline`+`V002__seed`. Pre-prod may squash; post-prod append only.

### ⛔ Single-transaction runner — fresh-apply anti-patterns (CI-guarded)

Each runner applies ONE migration file in ONE implicit transaction (authenc/integrasi
`batch_execute`, perlengkapan refinery). These constructs silently break fresh-apply and
are flagged by **`infra/lint/check-sql-migrations.sh`** (CI job `Lint SQL Migrations`):
- **No `CREATE/DROP INDEX … CONCURRENTLY`** — can't run inside a txn; pointless on a fresh DB.
- **No top-level `BEGIN`/`COMMIT`/`ROLLBACK`** — the runner owns the transaction (PL/pgSQL
  `DO $$ BEGIN … END $$` is fine; it has no trailing `;` on `BEGIN`).
- **No `SELECT … set_config('search_path','',…)`** — `pg_dump` emits this by DEFAULT; left in,
  it resets `search_path` so the runner's unqualified history INSERT fails → migration rolls
  back. **Strip it when squashing a baseline from `pg_dump`.**
- **No `ALTER DATABASE CURRENT …`** — invalid syntax (there is no `CURRENT` keyword for
  `ALTER DATABASE`); set `search_path` via the connection/role instead.
Waive a single line only with a trailing `-- guard:allow` + a reason. Check locally:
`./infra/lint/check-sql-migrations.sh`.

## Common pitfalls

- Forgetting a route registration → handler compiles but 404s.
- Wrong extractor order → opaque axum trait errors.
- Putting ORM/`Row` mapping or `tokio-postgres` types in `lib-perlengkapan` → breaks
  its WASM-safety. Keep persistence in the service repository layer.
