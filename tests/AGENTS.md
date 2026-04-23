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
