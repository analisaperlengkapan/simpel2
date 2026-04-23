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

- `RequestContext` dan `CorrelationId` hanya struct di sini. Method `from_headers()` ada di `lib-backend`.
- `Claims` hanya struct. `decode_jwt()` dan `AuthClaims` extractor ada di `lib-backend::jwt`.
- `AuditLogEntry` hanya struct. `AuditLogger` (DB persistence) ada di `lib-backend::audit`.
