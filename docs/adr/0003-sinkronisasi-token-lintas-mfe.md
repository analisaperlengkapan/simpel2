# ADR-0003: Cross-MFE Token Sync via localStorage Storage Events + Per-MFE Refresh Loop

- **Status**: Accepted
- **Date**: 2026-05-25
- **Deciders**: SIMPEL Frontend & Identity
- **Technical context**: Phase 1.1 stabilisasi (plan: `tolong-bantu-saya-saya-tender-flamingo`)

## Konteks

SIMPEL antarmuka dibagi jadi dua microfrontend (MFE) yang bersamaan
mengakses authenc:

- **Portal** (`antarmuka/portal/`) — entry point user, login + admin
  IAM + MFA setup. Mount path: `/portal/`.
- **Perlengkapan** (`antarmuka/perlengkapan/`) — modul BMN; user lompat
  ke sini setelah login dari Portal. Mount path: `/perlengkapan/simpel/v2/`.

Karena keduanya **same-origin** (`simpel.kejaksaan.go.id`), keduanya
berbagi `localStorage`. JWT access token disimpan di key kanonik
`auth_token` (lihat root `AGENTS.md` → Canonical localStorage Keys).

Tiga kelas masalah real yang terjadi sebelum ADR ini diputuskan:

1. **Stale claims di Portal saat Perlengkapan refresh token**: user
   login lewat Portal, lompat ke Perlengkapan, Perlengkapan refresh
   token setelah ~30 menit. Portal tab punya `UserSession` in-memory
   yang outdated — claims dipakai untuk render UI (mis. `is_admin()`)
   tetap memakai snapshot lama sampai user reload tab manual.

2. **Stale claims di Perlengkapan saat Portal refresh token**: kebalikan
   skenario di atas — Perlengkapan listener sudah dengar event
   `auth_token` (sudah benar), tapi Portal hanya menulis ke key legacy
   `user_session` (serialized blob); Portal **juga** menulis
   `auth_token` di `save_token()`, tapi listener Portal sendiri tidak
   mengamati `auth_token` event → cross-tab sync Portal ↔ Portal patah.

3. **Perlengkapan tab standalone, token expire diam-diam**: kalau
   Portal tab ditutup dan Perlengkapan dibuka sendirian, tidak ada
   yang memanggil refresh — Perlengkapan tidak punya scheduler
   sendiri. User mengalami 401 mid-task setelah ~30 menit, dengan
   redirect-ke-login yang tampak random dari sisi user.

## Keputusan

Tiga perubahan, semuanya WASM-side, no backend change:

### A. Portal storage listener juga mendengar `auth_token`

`antarmuka/portal/src/features/auth.rs::setup_storage_listener` —
tambahkan handler untuk key `auth_token`, bukan hanya `user_session`
legacy:

```rust
match key.as_str() {
    "auth_token" | "user_session" => {
        // Re-decode the JWT and update the in-memory session.
        let session = Self::load_session();
        on_session_change(session);
    }
    "logout_event" => on_session_change(None),
    _ => {}
}
```

`auth_token` adalah event yang Perlengkapan tulis saat refresh; dengan
listener ini, Portal tab langsung mengamati session baru tanpa reload
manual. Tetap dengar `user_session` untuk backward-compat dengan
flow lama yang belum di-migrate.

### B. Perlengkapan self-refresh loop (`features::session_monitor`)

`antarmuka/perlengkapan/src/features/session_monitor.rs`
`spawn_refresh_loop()` — tick tiap 30 detik:

- Load session dari JWT.
- Kalau `expires_within(120)` (2 menit sebelum exp), panggil
  `AuthService::try_refresh()`.
- Kalau gagal refresh (refresh token kadaluwarsa, jaringan mati),
  broadcast logout dan clear session — user diarahkan ke login,
  bukan stuck dengan 401 nasty.

Spawn-nya dari `App()` di `lib.rs` lewat `Effect::new` agar lifecycle
ikut komponen root.

### C. Pertahankan single source of truth: `auth_token`

Token kanonik tetap `auth_token` (kontrak global di root `AGENTS.md`).
ADR ini **tidak** memperkenalkan key baru, hanya memperluas siapa
yang mendengarkan key existing.

## Alternatif yang dipertimbangkan

1. **`BroadcastChannel` API** (sebagai pengganti `storage` event) —
   lebih real-time (~puluhan mikrodetik vs ~milidetik untuk storage
   event), tidak ter-trigger oleh tab sendiri. Ditolak untuk Phase 1
   karena dukungan Safari ITP historis tidak konsisten, dan `storage`
   event sudah cukup cepat untuk auth use-case. Dapat ditambahkan
   sebagai second layer di Phase 2 kalau perlu.

2. **Service Worker shared session** — Service Worker bisa jadi
   single auth controller untuk semua tab same-origin. Ditolak karena
   menambah surface area (SW lifecycle, registration, scope) untuk
   masalah yang bisa diselesaikan dengan storage event.

3. **Cookie session (HttpOnly Secure SameSite=Strict)** — secara teori
   lebih aman dari XSS karena tidak bisa dibaca JS. Ditolak: arsitektur
   SIMPEL adalah SPA + REST API, JWT di `Authorization: Bearer`
   header yang konsisten dengan gRPC backend (Tonic interceptor). Cookie
   tidak cocok untuk WASM yang fetch dari MFE berbeda
   sub-path.

4. **Polling backend `/auth/status` periodic** — kerja, tapi membuang
   network round-trip dan memberi beban tetap ke authenc. Storage
   event + local JWT decode lebih murah dan cukup akurat.

## Konsekuensi

**Positif**:

- Cross-tab session sync deterministik antar MFE tanpa user action.
- Perlengkapan tidak lagi bergantung pada Portal untuk refresh —
  bisa dipakai sebagai aplikasi standalone (deep-link langsung ke
  perlengkapan dari email notifikasi, dll).
- Logout dari salah satu MFE langsung terlihat di MFE lain (<2 detik).

**Negatif / trade-off**:

- Dua MFE yang sama-sama jalan akan refresh token paralel saat sama
  expire — risiko double refresh kecil (authenc idempotent untuk
  refresh token yang sama). Dimitigasi dengan `expires_within(120)`
  threshold yang sama di kedua MFE: keduanya trigger di window yang
  sama, tapi *satu* request menang race dan yang lain dapat new token
  via storage event.
- Storage event tidak fire di tab pengirim sendiri (web standard).
  Tab yang melakukan refresh harus update state-nya sendiri via
  return value `try_refresh()` (sudah handled di kedua MFE).
- Tabs yang tidur lama (background, throttled) bisa skip beberapa
  tick. Browser modern (Chrome/Edge/Firefox) throttle setTimeout di
  background ke ~1 detik minimum; 30-detik tick masih jalan, sekedar
  rate-limited. Refresh request tetap masuk pre-expiry karena
  threshold 120 detik > tick interval.

## Referensi & Catatan Implementasi

- File yang dimodifikasi (commit `76f72fc`):
  - `antarmuka/portal/src/features/auth.rs`
  - `antarmuka/perlengkapan/src/features/auth.rs`
  - `antarmuka/perlengkapan/src/features/session_monitor.rs` (baru)
  - `antarmuka/perlengkapan/src/features/mod.rs`
  - `antarmuka/perlengkapan/src/lib.rs`

- Manual verification (lihat antarmuka/AGENTS.md → Common Tasks #3):

  1. Login di Portal, buka Perlengkapan di tab baru, tunggu 5 menit
     → operasi di Perlengkapan tetap jalan (token tersinkron).
  2. Logout di salah satu MFE → MFE lain logout dalam <2 detik.
  3. Tutup Portal tab, biarkan Perlengkapan terbuka sendirian, tunggu
     selama 35 menit (>30 menit standard JWT expiry) → operasi tetap
     berhasil; log browser console menunjukkan `try_refresh` succeeded.
