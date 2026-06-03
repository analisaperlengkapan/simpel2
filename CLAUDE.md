# CLAUDE.md

> Panduan untuk Claude Code / agen AI di repo **SIMPel v2** (sistem informasi
> manajemen aset Kejaksaan RI). File ini **sengaja ringkas** dan **tidak
> menduplikasi** dokumentasi — sumber kebenaran adalah berkas `AGENTS.md`.

## Baca AGENTS.md (Single Source of Truth)

Mulai dari **[`AGENTS.md`](AGENTS.md)** (root) untuk makro-arsitektur, lalu ikuti
AGENTS.md spesifik sesuai direktori yang sedang dikerjakan. `AGENTS.md` root
memuat **AI Routing Guide** ke semua AGENTS.md per-domain. Pintasan:

| Area | Panduan |
|------|---------|
| Backend services | [`layanan/AGENTS.md`](layanan/AGENTS.md) (+ `layanan/<svc>/AGENTS.md`) |
| Frontend (Leptos WASM) | [`antarmuka/AGENTS.md`](antarmuka/AGENTS.md) |
| Shared libraries | [`lib/AGENTS.md`](lib/AGENTS.md) (+ `lib/<crate>/AGENTS.md`) |
| Legacy (Laravel) | [`monolith/AGENTS.md`](monolith/AGENTS.md), [`monolith/simpelv1/AGENTS.md`](monolith/simpelv1/AGENTS.md) |
| Infra / DevOps / Deploy | [`infra/AGENTS.md`](infra/AGENTS.md) |
| Testing (E2E/Integration) | [`tests/AGENTS.md`](tests/AGENTS.md) |
| Docs / ADR | [`docs/AGENTS.md`](docs/AGENTS.md) |

**Precedence:** `AGENTS.md` (root) > `<domain>/AGENTS.md` > `README.md`.

## Aturan yang sering terlewat (rujukan cepat)

- **Deploy WAJIB staging → promote → production.** DILARANG prod-only ke
  `simpel.kejaksaan.go.id`. Detail: [`infra/AGENTS.md`](infra/AGENTS.md) →
  "Pemisahan Lingkungan" → "Alur deploy WAJIB".
- **Semua perubahan cluster lewat Helm** (`infra/helm/`), bukan `kubectl apply`/
  `kubectl edit`. Image tag **semver immutable** (dilarang `latest`/`stag`/`prod`).
- **CI runner: ARC ephemeral** (`infra/helm/arc/`). Autofix push pakai
  `AUTOFIX_PAT` agar status check PR ter-refresh otomatis.
- **Uji**: backend `cargo test`; frontend Playwright e2e (`antarmuka/*/tests/e2e`).

## Catatan

- Berkas `AGENTS.md` adalah sumber utama; jangan pindahkan kontennya ke sini.
  Bila perlu aturan baru, tulis di `AGENTS.md` yang relevan, bukan menduplikasi.
