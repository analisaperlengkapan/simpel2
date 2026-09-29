# 🏗️ AGENTS.md — lib-core

> Crate ini adalah **core types & utilities WASM-safe** yang digunakan oleh frontend dan backend.

## 📍 Peran dalam Arsitektur

```
Browser (WASM) ──uses──► lib-core
Backend (Axum) ──uses──► lib-core
```

- **WASM-compatible**: Tidak ada dependensi `tokio`, `axum`, `deadpool`, atau OS-level.
- **Zero async runtime**: Semua fungsi synchronous.
- **Shared types**: `UserRole`, `Claims`, `SsoSession`, `ValidationError`, `AuditEvent`, dll.

## 📦 Modul

| Modul | Isi | Consumer |
|-------|-----|----------|
| `auth` | `UserRole`, `SsoSession` | Frontend + Backend |
| `jwt_claims` | `Claims`, `RealmAccess` (struct only, tanpa decode) | Frontend + Backend |
| `authz` | Kosakata otorisasi bersama: `ADMIN_ROLES`, `IAM_ADMIN_ROLES`, `CROSS_SATKER_ROLES`, `RoleSet`, `Capability`, `Authorization`, `ROLE_CATALOG` | Frontend + Backend |
| `validation` | Pure validators (email, NIP, password, satker) | Frontend + Backend |
| `error` | `CommonError`, `Result` | Semua |
| `config` | `ServerConfig`, `DatabaseConfig`, `BaseServiceConfig` (struct only) | Backend |
| `correlation` | `CorrelationId` (struct only, tanpa middleware) | Backend |
| `context` | `RequestContext` (struct only, tanpa `from_headers`) | Backend |
| `audit` | `AuditEvent`, `AuditLogEntry`, `AuditLogFilter` (tanpa DB logger) | Backend |
| `encoding` | Base64 encode/decode | Frontend + Backend |
| `sanitizer` | HTML/XSS sanitization | Frontend + Backend |
| `health` | Health check types | Backend |
| `models` | Shared domain models | Frontend + Backend |

## 📏 Aturan

1. **DILARANG** menambahkan dependensi async (`tokio`, `async-trait`) ke crate ini.
2. **DILARANG** menambahkan dependensi backend (`axum`, `deadpool-postgres`, `redis`).
3. Semua tipe harus `Serialize + Deserialize` untuk interop JSON.
4. Gunakan `#[cfg(feature = "validation")]` untuk dependensi `garde`.
5. Fungsi `default_*` di `config.rs` harus `pub` agar bisa diakses dari `lib-backend`.

## ⚠️ Pitfall

- **`authz` = SATU-SATUNYA definisi "siapa admin/pusat/…"**. Semua predikat **persis** (bukan
  prefix). Kapabilitas **aditif**: `admin` bukan superset peran bisnis (tak ada `is_admin_implied`);
  `Capability::View` = punya ≥ 1 role; `AdministerIam` = `admin` persis (`IAM_ADMIN_ROLES`,
  sengaja lebih sempit dari `ADMIN_ROLES`); `ValidateSatker`/`ApproveSatker` melayani
  `validator_satker`/`approver_satker`. Menambah role = ubah tabel `Capability::roles` +
  `ROLE_CATALOG`, bukan `has_role("admin")` di mana-mana. Daftar peran-literal di luar file ini
  dilarang (CI: `infra/scripts/check-authz-guards.py`, aturan R5).
- `Claims` membawa `assigned_roles` (dimiliki), `active_role` (peran sesi aktif, DSD) dan
  `groups` (hierarki satker) — semuanya `#[serde(default)]`: token lama tetap terurai.

- `RequestContext` dan `CorrelationId` hanya struct di sini. Method `from_headers()` ada di `lib-backend`.
- `Claims` hanya struct. `decode_jwt()` dan `AuthClaims` extractor ada di `lib-backend::jwt`.
- `AuditLogEntry` hanya struct. `AuditLogger` (DB persistence) ada di `lib-backend::audit`.
