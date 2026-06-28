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

### 2. Integration Testing

- **Lokasi**: `tests/integration/`
- **Framework**: `axum-test`, `tokio-test`

### 3. Load & Performance Testing

- **Lokasi**: `tests/load/`
- **Standar**: Hanya ke namespace `simpelv2-staging`.

## ⚠️ Aturan AI untuk Pengujian

❌ **DON'T:**

- Jangan masukkan logika *mocking* internal ke E2E tests.
- Jangan bergantung pada status database produksi.

✅ **DO:**

- Gunakan *seeding* database mandiri sebelum *integration test*.
- Pastikan semua pengujian dapat dijalankan via CI/CD (`cargo test --workspace` atau `npx playwright test`).
