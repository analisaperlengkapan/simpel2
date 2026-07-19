# 🤖 AGENTS.md - Pengujian (Testing)

> **Notice to Agents**: File ini adalah pedoman (Level 2 Archetype) untuk semua operasi terkait Pengujian E2E, Integrasi, dan *Load Testing* di direktori `tests/`.

## 📑 Daftar Isi (Table of Contents)

1. 🗺️ Domain Routing (Unit Tests)
2. 🧪 Strategi Pengujian Global
3. ⚠️ Aturan AI untuk Pengujian

## 🗺️ Domain Routing (Unit Tests)

Unit Tests dan Integration Tests berada di dalam *crate* masing-masing, bukan di sini.

- ⚙️ **Unit Test Backend**: Baca `layanan/AGENTS.md`.
- 🌐 **Unit Test Frontend**: Baca `antarmuka/AGENTS.md`.

## 🧪 Strategi Pengujian Global (`tests/`)

### 1. End-to-End (E2E) Testing

- **Lokasi**: `tests/e2e/`
- **Framework**: **Playwright**
- **Aturan**: Mencakup CAPTCHA handling, integrasi Authenc ↔ layanan-integrasi.
- **Tagging suite untuk integrasi data**:
  - Tag `@requires-integrasi-data` untuk suite yang butuh hasil sync API eksternal (SIMAN/MySIMKARI/MonSAKTI).
  - Di **staging**, CronJob integrasi `suspend: true` (cegah rate-limit token API). Suite tagged tsb butuh manual trigger sync sebelum dijalankan, atau di-skip via env:

    ```bash
    PLAYWRIGHT_SKIP_INTEGRASI=1 npx playwright test --grep-invert "@requires-integrasi-data"
    ```

  - Di **production**, CronJob aktif → suite read-only boleh dijalankan tanpa trigger manual.
- **RBAC data-scoping (WAJIB di-assert server-side, bukan via UI):** penegakan
  scoping berjenjang (`operator_satker`/`validator_wilayah`/`validator_pusat`/
  `admin`) **dibuktikan di BACKEND**, bukan sekadar "tombol disembunyikan di FE".
  Pola acuan = `antarmuka/perlengkapan/tests/e2e/rbac-scoping.spec.ts` (project
  `perlengkapan-rbac`, pure-API via fixture `request`): login per-peran nyata →
  panggil API backend → assert (a) JWT memuat `role`+`satker_code` benar, (b)
  jumlah baris ter-scope **persis**, (c) isolasi per-baris (tak ada kebocoran
  lintas-satker), (d) object-level fail-closed (404), (e) tanpa token → 401.
  - Data deterministik dari `tests/fixtures/e2e/seed-multisatker.sql` (FRESH
    dbsimpelv2). Hitungan persis (operator=2/2, wilayah=4, pusat=5) hanya berlaku
    di stack CI (DB kosong + seed ini); di staging (snapshot nyata) pakai
    invarian isolasi per-baris, bukan hitungan absolut.
  - Fixtures/expected-scope per-peran di-pusat-kan di
    `helpers/real-auth.ts` (`TEST_USERS`) — perbarui seiring perubahan seed/role.
- **Guard (RoleGate/SatkerGate) per-peran:** `auth.setup.ts` login tiap peran
  nyata → tulis **storageState per-peran** (`results/.auth/<key>.json`, helper
  `storageStatePath`); spec ber-browser pakai `test.use({ storageState })`. Pola
  acuan `guards-rbac.spec.ts` (#482): peran non-admin DITOLAK di `/admin/*`
  (ForbiddenPage), admin lolos. Guard = **client-side JWT-claim** (`is_admin`),
  jadi jalan di stack compose CI **tanpa** jalur FE→BE.
- **UI-layer data-scoping (Layer-3):** `rbac-scoping-ui.spec.ts` (project
  `perlengkapan-ui-rbac`) membuktikan FE me-RENDER hanya baris in-scope —
  navigasi `/bank-aset/daftar` per-peran (storageState), assert NUP seed yang
  boleh tampil + yang TIDAK (E2E-A/B/C). Pelengkap bukti server-side
  (`perlengkapan-rbac`).
- **Route-coverage GATE (mekanikal, `tests/e2e/route-coverage.mjs`, WAJIB):**
  penegak "semua halaman diuji, tanpa terkecuali". Skrip Node tanpa-dependency
  (parse statis — tak butuh build/browser/stack) mem-parse tiap `routes.rs`
  (kedua FE) → daftar rute navigable → **GAGAL** bila ada rute bertipe yang tak
  dikunjungi ≥1 spec (`goto`), atau `allowUncovered` basi, atau (saat
  `blocking:true`) masih ada utang. Job CI `route-coverage` masuk
  `ci-summary.needs` (blocking). Debt e2e yang belum ditulis dicatat eksplisit di
  `tests/e2e/route-coverage.config.json` `allowUncovered` (menyusut ke `[]`
  per-FE seiring domain E-1..E-7 mendarat; lalu flip `blocking:true`). **Rute
  BARU tanpa e2e = CI merah sejak hari-1**, walau FE masih "advisory". Menambah
  halaman ⇒ WAJIB menambah e2e yang membukanya (atau catat sbagai utang sadar).
- **Reachability:** `nav-access.spec.ts` (project `perlengkapan-nav`) — tiap rute
  modul fitur WAJIB mount shell ter-autentikasi (`app_chrome` header "SIMPEL")
  per-peran; signal data-independent. Spec MOCK lama (`ui-*.spec.ts`,
  `helpers/session.ts`) DIHAPUS — JANGAN hidupkan pola mock; pakai real-auth
  storageState.
- **Workflow bisnis PENUH = di CI (F-E2E, mandat user 2026-07-06), BUKAN ditunda
  ke staging.** Tiap workflow (kebutuhan/pakaian-dinas/pemakaian/penghapusan/
  bank-aset/analitik/dashboard/notifikasi/admin) di-drive end-to-end lewat UI
  (isi form → klik tombol nyata "Submit ke Validator Wilayah"/"Teruskan ke
  Validator Pusat"/"Generate Konsep SK"/"Download PDF" → handoff multi-peran via
  banyak storageState) dgn asersi **state UI + verifikasi state backend via API**
  (transisi status ter-persist) — **bukan** presence/`if-visible`/screenshot.
  Prasyarat non-UI (master data, periode terbuka, campaign target satker) di-seed
  di `tests/fixtures/e2e/seed-multisatker.sql` atau dibuat via-UI. Suite
  komprehensif ini **blocking tiap PR**. @staging = superset destruktif.
- **Catatan jalur FE→BE @compose:** WASM memanggil API **origin-relative**
  (`api/client.rs` API_BASE=`/api/v1/perlengkapan`) → di-proxy nginx FE ke
  backend. Upstream host:port nginx **env-overridable** (entrypoint substitusi;
  default = port K8s `8093`/`8091`; docker-compose set `:3020`/`:8088`). Tanpa
  override ini jalur data FE→BE putus di compose (hanya AuthGate jalan karena
  decode JWT client-side). Dengan fix, UI-data e2e jalan **di CI** — tetap
  utamakan bukti scoping **server-side via API** (deterministik, tak rapuh
  selector); UI-layer = pelengkap render.
- **Portal e2e (real-auth):** job `e2e-portal` bawa stack compose (portal FE +
  authenc captcha-debug) seperti `e2e-perlengkapan`; spec mem-proxy `/api/**` ke
  authenc via Playwright `page.route()` (TIDAK butuh proxy nginx portal). **Gate
  = project `portal-chromium` = HANYA `portal-auth-e2e`** (alur login FE portal
  nyata: captcha+login+token; cukup portal+authenc). Spec lain di-pisah ke
  project NON-gate karena butuh layanan lain — JANGAN masukkan ke gate tanpa
  layanannya: `portal-cross-app` (auth-login-flow, butuh FE perlengkapan → SSO
  cross-app @staging), `portal-integration` (integrasi-authenc, browser-free
  grpc-js+REST), `portal-screenshots` (artefak visual). Aktifkan gate
  via `E2E_PORTAL_ENABLED=true` atau dispatch `run_e2e`.
- **Integrasi↔Authenc e2e (backend):** job `e2e-integrasi-authenc` (project
  `portal-integration`) — stack `authenc`+`secreton`+`layanan-integrasi` (BUKAN
  FE); uji gRPC `IntegrasiService` (`layanan-integrasi:50052`) + authenc REST.
  Proto integrasi ADA DI LUAR konteks image e2e → di-copy ke build context +
  `INTEGRASI_PROTO_PATH` (env-override di spec). Gate `E2E_INTEGRATION_ENABLED`
  / dispatch `run_e2e`.
- **Cross-app SSO e2e (portal↔perlengkapan):** job `e2e-portal-cross-app`
  (project `portal-cross-app`, spec `auth-login-flow`) — SSO NYATA lewat
  **single-origin `cross-app-ingress`** (`infra/nginx/e2e-cross-app-ingress.conf`,
  meniru Istio VS prod: `/portal`+`/perlengkapan`+`/api/*` satu origin). Alur =
  **dua-hop**: `/perlengkapan/` → login-page perlengkapan sendiri → link "Masuk
  via Portal" (`/portal/login?redirect_uri=…dashboard`) → login authenc nyata →
  portal honor redirect_uri → dashboard perlengkapan mount dari JWT same-origin
  (`localStorage["auth_token"]`). **JANGAN** `page.addInitScript(localStorage.
  clear())` (re-run tiap navigasi → hapus token saat hard-redirect ke dashboard;
  Playwright sudah isolasi context per-test). Butuh KEDUA FE build → **tidak** di
  `ci-summary.needs`. Gate `E2E_INTEGRATION_ENABLED` / `run_e2e`.
- **Secreton bootstrap↔Gateway e2e:** job `e2e-secreton-gateway` — init+unseal
  (Shamir)+seed secret via gateway REST (`infra/scripts/secreton-ci-bootstrap.sh`).
  Opsi `SECRETON_DB_PROVISION=1`: provision DB-secrets-engine + lease + buktikan
  dynamic credential PG login (psql SELECT 1) lewat gateway (`GATEWAY_ENABLE_DB_ADMIN=1`).
  Provisioning + issuance WAJIB ke proses secreton yang SAMA. Gate `E2E_INTEGRATION_ENABLED`
  / `run_e2e`; skill `secreton-ops`.
- **simpelv1↔Gateway live e2e:** job `e2e-simpelv1-integration` — PHPUnit suite
  `IntegrationLive` lawan **gateway sidecar NYATA** (BUKAN `Http::fake`) →
  authenc+secreton+layanan-integrasi. Membuktikan jalur internal v1 (verifyToken,
  fetch-secret) nyata. Gate `E2E_INTEGRATION_ENABLED` / `run_e2e`.

#### Matriks coverage E2E lintas-service (kanonik — F-GW)

Batas-kepercayaan (pair service/MFE) → di mana diuji. **Real** = stack nyata
(bukan mock/fake). Jobs opt-in jalan pada `E2E_INTEGRATION_ENABLED` / dispatch
`run_e2e`; hanya `e2e-portal`+`e2e-perlengkapan` yang **blocking** (`ci-summary.needs`,
via `E2E_PORTAL_ENABLED`/`E2E_PERLENGKAPAN_ENABLED`).

| Pair (batas-kepercayaan) | Job CI | Real? | Status |
|---|---|---|---|
| portal → authenc (login/captcha/token) | `e2e-portal` (`portal-chromium`) | ✅ | **blocking** |
| perlengkapan → authenc (RBAC/guards/nav) | `e2e-perlengkapan` | ✅ | **blocking** |
| perlengkapan → integrasi (bank_aset cross-schema + gRPC pegawai) | `e2e-perlengkapan` + `e2e-integrasi-authenc` | ✅ | opt-in |
| **portal → perlengkapan (SSO same-origin)** | `e2e-portal-cross-app` | ✅ | opt-in |
| authenc → integrasi (resolve-satker gRPC) + integrasi gRPC/REST | `e2e-integrasi-authenc` | ✅ | opt-in |
| secreton ↔ gateway (bootstrap + dynamic-lease) | `e2e-secreton-gateway` | ✅ | opt-in |
| simpelv1 → gateway → {authenc, secreton, integrasi} | `e2e-simpelv1-integration` | ✅ | opt-in |
| portal → simpelv1 (SSO + gateway) | — | — | **staging-only** (F5-E / #35) |
| simpelv1 → integrasi (data-plane pegawai/BMN) | — | — | **belum ada** (#41 FDW+REST, pasca-F5) |
| authenc ↔ secreton (MFA-opt fetch) | — | — | **staging-only** (tak ter-wire di compose) |
| integrasi ↔ secreton (app gRPC) | — | — | **N/A** (integrasi 0 secreton gRPC ref → `SECRETON_GRPC_URL` vestigial, dibuang. Token API eksternal disuntik via SA-token @prod = concern platform, bukan integrasi app, lihat `layanan/integrasi/AGENTS.md`) |

Pair "staging-only"/"belum ada"/"N/A" **didokumentasikan, TIDAK di-fake**. Yang
staging-only masuk checklist sertifikasi **F5-E (#35)** — komprehensif @staging.

### 2. Integration Testing

- **Lokasi**: `tests/integration/`
- **Framework**: `axum-test`, `tokio-test`

### 3. Load & Performance Testing

- **Lokasi**: `tests/load/`
- **Standar**: Hanya ke namespace `simpelv2-staging`.

## 🚫 Tes yatim (orphaned tests) — gate `lint-orphan-tests`

Cargo hanya menjadikan **`tests/*.rs`** sebagai test target. Berkas di
**sub-direktori** `tests/` hanya ikut terkompilasi bila ada target yang
mendeklarasikannya (`mod integration;`, `#[path = "..."] mod x;`, atau
`[[test]] path =` di `Cargo.toml`). Sub-direktori yang tak dideklarasikan
**tak pernah dikompilasi dan tak pernah dijalankan** — dan tak ada gate yang
menyadarinya membusuk.

Ini bukan hipotetis. `layanan/secreton/tests/{integration,unit,security,
performance,common}` berisi 8.640 baris dalam kondisi persis itu (#103/#104),
membusuk tiga arah sekaligus tanpa terdeteksi: galat sintaks (sebuah penyuntingan
otomatis mencopot `}` dari literal `json!`/`format!`), pemanggilan API yang sudah
lama direfaktor (`create_lease`: 11 argumen → 1 struct request, 19 lokasi), dan
pembacaan field yang kini privat. Satu berkasnya berjudul *"verifies all 16
secrets engines are functional"* padahal asersinya
`assert!(r.is_ok() || r.is_err())` — tautologi, di berkas yang belum pernah
sekalipun dibangun.

**Aturan:** berkas `.rs` di sub-direktori `tests/` **wajib** terjangkau dari
suatu target. Gate `lint-orphan-tests` (`infra/lint/check-orphan-tests.sh`,
**BLOCKING**) menggagalkan CI bila tidak. Perbaiki dengan menghapusnya, atau
memindahkannya ke `tests/<nama>.rs`, atau mendeklarasikannya — lalu **pastikan
ia benar-benar kompilasi dan lulus** sebelum diandalkan.

**Pelajaran yang lebih luas:** "ada berkas tes" ≠ "ada cakupan tes". Sebelum
mempercayai sebuah suite, pastikan ia benar-benar dijalankan dan asersinya bisa
gagal. Bandingkan dengan asersi lemah di E2E (`ada h1` lolos di halaman 404).

## ⚠️ Aturan AI untuk Pengujian

❌ **DON'T:**

- Jangan masukkan logika *mocking* internal ke E2E tests.
- Jangan bergantung pada status database produksi.

✅ **DO:**

- Gunakan *seeding* database mandiri sebelum *integration test*.
- Pastikan semua pengujian dapat dijalankan via CI/CD (`cargo test --workspace` atau `npx playwright test`).
