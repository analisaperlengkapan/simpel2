# 🤖 AGENTS.md - Layanan (Backend & Core Services)

> **Notice to Agents**: File ini adalah pedoman (Level 2 Archetype) untuk SELURUH Backend Services (`layanan/`) di monorepo SIMPEL. Baca file ini sebelum memodifikasi kode backend.

## 🗺️ Domain Routing
Jika Anda bekerja di subdirektori spesifik, baca aturan detailnya di sini:
- 🛠️ **Perlengkapan**: Baca `layanan/perlengkapan/AGENTS.md` (Backend BMN, 4 crates).
- 🔗 **Integrasi**: Baca `layanan/integrasi/AGENTS.md` (MonSAKTI, MySIMKARI, SIMAN).
- 🔐 **Authenc**: Baca `layanan/authenc/AGENTS.md` (OAuth2, OIDC, MFA, Identity).
- 🔒 **Secreton**: Baca `layanan/secreton/AGENTS.md` (Secrets Vault, Transit, PKI).

## 📑 Daftar Isi (Table of Contents)
1. 🏛️ Backend Architecture
2. 🗄️ Database Architecture
--- *Batas Truncation* ---
3. 🦀 Code Patterns (Axum 0.8.x, Tonic gRPC)
4. 🔐 Authentication & Secrets
5. 🔍 Observability & Monitoring
6. ⚠️ Error Handling Best Practices
7. 🛡️ Resilience Patterns

## 🏛️ Backend Architecture

Backend SIMPEL terbagi menjadi dua jenis layanan:
1. **Domain Services** (e.g., `layanan-perlengkapan`, `layanan-integrasi`): Menangani logika bisnis spesifik dan menerima *traffic* dari Microfrontends (via REST API).
2. **Core Infrastructure Services** (e.g., `authenc`, `secreton`): Layanan dasar untuk autentikasi dan manajemen secret, diakses HANYA oleh layanan backend lain (via gRPC mTLS).

### 🚨 Critical Backend Communication Rules
- Backend → Backend/Core: **WAJIB gRPC (mTLS)**
- Frontend → Backend: **REST API (JSON/HTTP)**
- DILARANG mengekspos Authenc atau Secreton langsung ke Microfrontend.

---

## 🗄️ Database Architecture

Setiap layanan menggunakan database terpisah untuk isolasi:

| Database | Service | Description |
|----------|---------|-------------|
| `perlengkapan` | `layanan/perlengkapan/*` | Core BMN & equipment management |
| `authenc` | `layanan/authenc/*` | Authentication, SSO, OIDC & Identity |
| `secreton` | `layanan/secreton/*` | Secrets, HSM & transit vault |

⛔ **DO NOT USE SQLx**. Gunakan `tokio-postgres`, `deadpool-postgres`, dan `refinery` untuk semua interaksi database.

---

## 🦀 Code Patterns

### Axum 0.8.x Pattern (Backend REST API)

```rust
use axum::{
    extract::{Path, Query, State, Json},
    routing::{get, post, put, delete},
    Router,
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

// ✅ Application State with gRPC clients
#[derive(Clone)]
pub struct AppState {
    pub db_pool: deadpool_postgres::Pool,
    pub redis: redis::aio::ConnectionManager,
    pub authenc_client: AuthencGrpcClient,   // gRPC to Authenc
    pub secreton_client: SecretonGrpcClient, // gRPC to Secreton
}

// ✅ Router setup with State and Centralized Middleware
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/items", get(list_items).post(create_item))
        .route("/api/v1/items/{id}", get(get_item).put(update_item))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

// ✅ Handler with extractors
async fn get_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<Item>, AppError> {
    let client = state.db_pool.get().await?;
    let row = client.query_one("SELECT * FROM items WHERE id = $1", &[&id]).await?;
    Ok(Json(Item::try_from(row)?))
}
```

### Tonic + Prost Pattern (gRPC)

```rust
// build.rs — Generate gRPC code from proto
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::Config::new()
        .out_dir("src/generated")
        .compile_protos(&["proto/auth.proto"], &["proto/"])?;
    Ok(())
}

// gRPC Client (layanan → authenc)
use tonic::transport::Channel;

pub async fn create_authenc_client() -> Result<AuthServiceClient<Channel>, tonic::transport::Error> {
    let channel = Channel::from_static("https://authenc.internal:50051")
        .tls_config(tonic::transport::ClientTlsConfig::new())?
        .connect().await?;
    Ok(AuthServiceClient::new(channel))
}
```

---

## 🔐 Authentication & Secrets

```rust
// Secrets Management: ALWAYS use Secreton via gRPC in Backend
let secret = state.secreton_client
    .get_secret(GetSecretRequest { key: "database_password".into() })
    .await?
    .into_inner()
    .value;
// NEVER use environment variables for production secrets!
```

---

## 🔍 Observability & Monitoring Patterns

### Structured Logging
Gunakan `tracing` dengan structured fields untuk semua layanan backend.
- Include fields: `request_id`, `user_id`, `service_name`, `operation`, `duration_ms`

```rust
use tracing::{info, error, instrument};

#[instrument(skip(db))]
async fn get_item(db: &Database, item_id: Uuid, user_id: Uuid) -> Result<Item> {
    info!(item_id = %item_id, user_id = %user_id, "Fetching item");
    // ...
}
```

### Distributed Tracing
Gunakan OpenTelemetry integration untuk distributed tracing. Propagasi *trace context* via gRPC metadata:

```rust
use opentelemetry::trace::TraceContextExt;

let mut request = tonic::Request::new(request);
let cx = opentelemetry::Context::current();
let carrier = opentelemetry::global::get_text_map_propagator(|propagator| {
    let mut carrier = MetadataMap::new();
    propagator.inject_context(&cx, &mut carrier);
    carrier
});
request.metadata_mut().extend(carrier);
```

---

## ⚠️ Error Handling Best Practices

Gunakan hirarki tipe error standar:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] deadpool_postgres::PoolError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {resource} with id {id}")]
    NotFound { resource: String, id: Uuid },

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

// ✅ ALWAYS map domain errors to HTTP responses (Axum IntoResponse)
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::NotFound { .. } => axum::http::StatusCode::NOT_FOUND,
            Self::Validation(_) => axum::http::StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => axum::http::StatusCode::UNAUTHORIZED,
            Self::Database(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        // Standarisasi response error format
        (status, axum::Json(serde_json::json!({"error": self.to_string()}))).into_response()
    }
}
```

---

## 🛡️ Resilience Patterns

### Circuit Breaker & Rate Limiting
- Implementasi circuit breaker untuk gRPC calls ke layanan eksternal.
- Thresholds: failure rate (50%), timeout (5s), consecutive errors (5).

```rust
use governor::{Quota, RateLimiter};

// Rate limiter
let limiter = RateLimiter::direct(Quota::per_second(10));
if limiter.check().is_err() {
    return Err(AppError::RateLimited);
}
```

### Retry Strategy
- Gunakan *Exponential backoff* untuk transient failures.
- Max retry limits per operation (3-5 retries).

```rust
use backoff::ExponentialBackoff;

async fn retry_with_backoff<F, Fut, T, E>(mut operation: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let backoff = ExponentialBackoff::default();
    backoff::retry(backoff, operation).await
}
```
