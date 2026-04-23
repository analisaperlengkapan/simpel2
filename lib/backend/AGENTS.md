# ⚙️ AGENTS.md — lib-backend

> Crate ini berisi **backend infrastructure** untuk layanan SIMPEL (DB, cache, middleware, gRPC, storage).

## 📍 Peran dalam Arsitektur

```
lib-core (types) ◄──depends── lib-backend (infrastructure)
                              │
                              ├── Axum middleware
                              ├── DB pools (deadpool-postgres)
                              ├── Redis/LRU cache
                              ├── JWT decode + AuthClaims extractor
                              ├── gRPC TLS + interceptors
                              ├── S3 storage
                              └── Telemetry (tracing-subscriber)
```

## 📦 Modul & Feature Flags

| Feature | Modul yang aktif | Dependensi |
|---------|-----------------|------------|
| `axum` | `middleware/`, `error`, `correlation`, `context` | axum, tower, http, bytes |
| `db` | `db`, `validation`, `audit` | tokio-postgres, deadpool-postgres |
| `jwt` | `jwt` (decode + AuthClaims) | jsonwebtoken |
| `telemetry` | `telemetry` | tracing-subscriber |
| `redis-cache` | `cache` (Redis backend) | redis |
| `grpc` | `grpc/` | tonic, rustls |
| `storage` | `storage` | aws-sdk-s3 |
| `env-config` | `config` (load_base_config) | dotenvy |
| `full-backend` | Semua di atas | Semua |

## 📏 Aturan

1. Selalu `use lib_core::` untuk core types, JANGAN duplikasi.
2. **DILARANG** `impl` block untuk tipe dari `lib-core` (orphan rule). Gunakan:
   - Standalone functions: `decode_jwt()`, `request_context_from_headers()`, `load_base_config()`
   - Newtype wrapper: `AuthClaims(Claims)` untuk Axum extractor
3. Setiap modul harus di-gate dengan feature flag yang sesuai.
4. Re-export `CommonError` dan `Result` dari `lib-core` di `lib.rs`.

## 🔑 Pola Penting

### JWT Extraction
```rust
// Di handler:
use lib_backend::jwt::AuthClaims;
async fn handler(AuthClaims(claims): AuthClaims) { ... }

// Manual decode:
use lib_backend::jwt::decode_jwt;
let claims = decode_jwt(token, secret)?;
```

### Request Context
```rust
use lib_backend::context::request_context_from_headers;
let ctx = request_context_from_headers(&headers);
```

### Config Loading
```rust
use lib_backend::config::load_base_config;
let config = load_base_config(); // reads from env vars
```

## ⚠️ Pitfall

- `cache.rs` dan `memory.rs` selalu tersedia (tidak di-gate) karena menggunakan tokio sync primitives.
- `AuthClaims` adalah newtype wrapper, akses inner `Claims` via `claims.0` atau deref.
- Consumer yang menggunakan `load_base_config()` harus enable feature `env-config`.
