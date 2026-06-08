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

Schema lives in `<svc>/migrations/` as **`V###__name.sql`**, embedded & run by
refinery from `main.rs` (`migrations::run`).
- **NEVER `ALTER` a migration after it's merged** — append a new `V###__` file.
- Numbering is sequential & unique per service (duplicate `V###` = refinery error).
- **Seed migrations must be idempotent** (UPSERT / `ON CONFLICT DO NOTHING`), since
  they may re-run across environments.
- Cross-service DB ordering matters when a schema is shared (e.g. authenc baseline
  before perlengkapan) — document it.
- `integrasi` migrations are manual SQL (see its `migrations/README.md`).
- NB: full migration fresh-apply is currently NOT clean (akreasi) — squash/repair is
  deferred to F6 using staging as reference (memory `project-migration-health-f6`).
  Don't attempt a squash here; just append.

## Common pitfalls

- Forgetting a route registration → handler compiles but 404s.
- Wrong extractor order → opaque axum trait errors.
- Putting ORM/`Row` mapping or `tokio-postgres` types in `lib-perlengkapan` → breaks
  its WASM-safety. Keep persistence in the service repository layer.
