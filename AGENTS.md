# 🤖 AGENTS.md - AI Developer Guide for SIMPEL

> **Notice to Agents**: This file serves as the **primary source of truth** for AI agents working on this repository. Read this before planning or executing tasks to understand the architecture, conventions, and workflows.

## 🌍 Project Context

**SIMPEL** (Sistem Informasi Perlengkapan) is a mission-critical **Rust Monorepo** for the Indonesian Attorney General's Office (Kejaksaan RI). It uses a modular crate-based architecture for backend services (`layanan/`) and a microfrontend architecture for the frontend (`antarmuka/`).

- **Production URL:** https://simpel.kejaksaan.go.id/
- **Compliance:** Zero-trust security, government standards

## 📐 Governance Rule Hierarchy

Use this precedence order for all guidance in this repository:

1. `AGENTS.md` (root) - global mandatory rules
2. `<domain>/AGENTS.md` - domain-specific overrides and local constraints
3. Local README/docs - implementation details and operational notes

If there is a conflict, root `AGENTS.md` wins unless a domain file explicitly documents an allowed override for that domain.

### 🔑 Key Tech Stack

| Component | Technology | Version |
|-----------|------------|---------|
| **Language** | Rust (Edition 2024, MSRV 1.90+) | 1.93+ |
| **Backend HTTP** | Axum (REST API) | 0.8.7 |
| **Backend gRPC** | Tonic + Prost | 0.14.x |
| **Frontend** | Leptos (WASM CSR) | 0.8.14 |
| **Database** | PostgreSQL (tokio-postgres / deadpool) | 15+ |
| **Caching** | Redis | — |
| **Infrastructure** | Docker, Kubernetes | — |
| **Identity** | Authenc (custom OAuth2/OIDC via gRPC) | — |
| **Secrets** | Secreton (custom vault via gRPC) | — |

---

## 🏛️ System Architecture

### Target Architecture (Canonical)

- Keep **modular monolith per bounded context** as the default deployment strategy.
- Use **internal platform services** (`authenc`, `secreton`, `layanan-integrasi`) through explicit service contracts.
- Prefer internal modularization first (crate/module boundaries) before splitting into new deployables.
- Keep communication patterns strict:
  - microfrontend -> backend via REST/JSON only
  - backend -> backend/core services via gRPC only
  - no direct microfrontend access to auth/secrets infrastructure

### Runtime Contract Rules

- Internal endpoint naming must be consistent across:
  - runtime config/env parsing
  - Kubernetes manifests
  - deployment overlays
- Use canonical env var names for internal gRPC targets:
  - `AUTHENC_GRPC_URL`
  - `SECRETON_GRPC_URL`
  - `INTEGRASI_GRPC_URL`
- Keep a compatibility alias only for local migration windows, then remove it.

```mermaid
flowchart TB
    subgraph Browser["🌐 Browser (WASM)"]
        MF1["Portal<br/>Microfrontend"]
        MF2["Perlengkapan<br/>Microfrontend"]
    end

    subgraph Backend["⚙️ Backend Services (Axum)"]
        LP["layanan-perlengkapan<br/>(api, dokumen,<br/>notifikasi, bantuan)"]
        LI["layanan-integrasi<br/>(MySIMKARI, SIMAN)"]
    end

    subgraph CoreServices["🔐 Core Infrastructure Services (gRPC)"]
        AUTH["Authenc<br/>(Identity Provider)"]
        SEC["Secreton<br/>(Secrets Vault)"]
    end

    subgraph Data["💾 Data Layer"]
        PG[(PostgreSQL)]
        RD[(Redis)]
    end

    %% Browser to Backend: REST API only
    MF1 -->|"REST API<br/>(JSON/HTTP)"| LP
    MF2 -->|"REST API<br/>(JSON/HTTP)"| LP

    %% Backend to Infrastructure: gRPC only
    LP -->|"gRPC<br/>(mTLS)"| AUTH
    LP -->|"gRPC<br/>(mTLS)"| SEC
    LI -->|"gRPC<br/>(mTLS)"| AUTH

    %% Infrastructure internal
    AUTH <-->|"gRPC<br/>(mTLS)"| SEC

    %% Data access
    LP --> PG
    LP --> RD
    LI --> PG
    AUTH --> PG
    AUTH --> RD
    SEC --> PG

    style Browser fill:#e1f5fe
    style Backend fill:#fff3e0
    style CoreServices fill:#fce4ec
    style Data fill:#e8f5e9
```

### 🚨 Critical Communication Rules

| From | To | Protocol | Allowed? |
|------|-----|----------|----------|
| Microfrontend | Backend Service | **REST API** (JSON/HTTP) | ✅ YES |
| Microfrontend | Authenc | ❌ FORBIDDEN (except explicit gateway proxy route) | ⛔ NO |
| Microfrontend | Secreton | ❌ FORBIDDEN | ⛔ NO |
| Backend Service | Backend Service | **gRPC** (Protobuf) | ✅ YES |
| Backend Service | Authenc | **gRPC** (mTLS) | ✅ YES |
| Backend Service | Secreton | **gRPC** (mTLS) | ✅ YES |
| Authenc | Secreton | **gRPC** (mTLS) | ✅ YES |

---

## 🏗️ Workspace Structure

```mermaid
flowchart LR
    subgraph Root["📦 Root Workspace (Single Cargo.lock)"]
        direction TB
        CT["Cargo.toml<br/>(Single Source of Truth)"]

        subgraph Lib["lib/ — Shared Libraries"]
            LU["ui/"]
            LC["common/"]
            LPR["perlengkapan/"]
        end

        subgraph Antarmuka["antarmuka/ — Microfrontends (Leptos WASM)"]
            AP["portal/"]
            APP["perlengkapan/"]
        end

        subgraph Layanan["layanan/ — Backend & Core Services"]
            subgraph L_Perlengkapan["perlengkapan/crates/ (4 crates)"]
                LPI["api"]
                LPD["dokumen"]
                LPNot["notifikasi"]
                LPB["bantuan"]
            end
            LPInt["integrasi/<br/>(standalone)"]
            subgraph L_Authenc["authenc/crates/ (10 crates)"]
                LA["types, core, crypto, storage,<br/>api, iam-api, grpc, mfa,<br/>federation, webauthn"]
            end
            subgraph L_Secreton["secreton/crates/ (14 crates)"]
                LS["core, api, storage, crypto,<br/>types, agent, cli, grpc,<br/>hsm, k8s-operator, auto-unseal,<br/>backup, health, replication"]
            end
        end

        subgraph Extra["Other"]
            TESTS["tests/"]
            DOCS["docs/"]
            INFRA["infra/"]
        end
    end

    CT --> Lib
    CT --> Antarmuka
    CT --> Layanan

    style Root fill:#e3f2fd
    style Layanan fill:#fff3e0
    style Lib fill:#e8f5e9
```

### Directory Layout

```bash
simpel2/
├── Cargo.toml               # ROOT WORKSPACE MANIFEST (Single source of truth)
├── Cargo.lock               # Unified lockfile for all workspace members
├── lib/                     # SHARED LIBRARIES
│   ├── ui/                  # UI components (lib-ui) — Leptos shared components
│   ├── common/              # Common utilities (lib-common) — types, config, crypto
│   └── perlengkapan/        # Perlengkapan domain types (lib-perlengkapan)
├── antarmuka/               # FRONTEND MICROFRONTENDS (Leptos WASM CSR)
│   ├── portal/              # Portal microfrontend — SSO gateway
│   └── perlengkapan/        # Perlengkapan microfrontend — BMN management
├── layanan/                 # BACKEND SERVICES & CORE INFRASTRUCTURE
│   ├── perlengkapan/        # Perlengkapan domain backend
│   │   └── crates/          # → api, dokumen, notifikasi, bantuan
│   ├── integrasi/           # Integrasi layanan eksternal (MySIMKARI, SIMAN)
│   ├── authenc/             # Identity Provider (OAuth2, OIDC, MFA, RBAC, SAML)
│   │   └── crates/          # → types, core, crypto, storage, api, iam-api,
│   │                        #   grpc, mfa, federation, webauthn
│   └── secreton/            # Secrets Vault (Transit, PKI, HSM, Auto-Unseal)
│       └── crates/          # → core, api, storage, crypto, types, agent,
│                            #   cli, grpc, hsm, k8s-operator, auto-unseal,
│                            #   backup, health, replication
├── tests/                   # INTEGRATION & E2E TESTS
│   ├── e2e/                 # End-to-end tests
│   ├── load/                # Load/performance tests
│   └── verification/        # Verification scripts
├── docs/                    # ENGINEERING DOCUMENTATION
├── infra/                   # OPERATIONS & INFRASTRUCTURE
│   ├── k8s/                 # Kubernetes manifests (Kustomize)
│   ├── monitoring/          # Prometheus, Grafana configs
│   └── nginx/               # Nginx configs
├── .github/                 # GitHub Actions & templates
└── .gitlab-ci.yml           # GitLab CI/CD pipeline
```

### Naming Conventions

| Type | Directory Location | Package Name Schema | Example |
|------|-------------------|---------------------|---------|
| **Microfrontend** | `antarmuka/[name]/` | (per Cargo.toml) | `antarmuka/portal` |
| **Domain Service** | `layanan/[domain]/crates/[name]/` | `[domain]-[name]` | `layanan-perlengkapan-api` |
| **Core Service** | `layanan/[core]/crates/[name]/` | `[core]-[name]` | `authenc-core`, `secreton-grpc` |
| **Shared Library** | `lib/[name]/` | `lib-[name]` | `lib-ui`, `lib-common` |

---

## 📏 Critical Conventions (DO NOT VIOLATE)

### 1. 📦 Dependency Management

- **Root Cargo.toml is King**: ALL external dependencies MUST be defined in `[workspace.dependencies]`
- **Inheritance**: Member crates MUST use `dependency_name = { workspace = true }`
- **NEVER specify versions** in member `Cargo.toml` files
- **lib-ui**: Use for shared Leptos UI components
- **lib-common (FREEZE)**: Do NOT add new modules to `lib/common/`. It is a legacy "God Crate". Instead, create specific shared crates like `lib-telemetry` or `lib-auth-client`.
- **Database Rules**: ⛔ **DO NOT USE SQLx**. Use `tokio-postgres`, `deadpool-postgres`, and `refinery` for all database interactions.
- **Unified Workspace**: ALL services including `authenc` and `secreton` crates are part of the main workspace

### 2. 🧹 Code Quality Commands

```bash
# Verification & Building
cargo check --workspace          # Quick compilation check
cargo build --workspace          # Full workspace build
cargo build -p authenc-core      # Build specific crate
cargo build -p secreton-api      # Build specific crate

# Code Quality
cargo fmt --all                  # Format all code
cargo clippy --workspace         # Lint with warnings as errors

# Testing
cargo test --workspace           # Run all tests
cargo test -p authenc-core       # Test specific crate

# Security
cargo audit                      # Check for vulnerabilities
cargo deny check                 # Check licenses and advisories

# Frontend (Leptos WASM)
cd antarmuka/portal && trunk serve --open    # Dev server with hot reload
cd antarmuka/portal && trunk build --release # Production build
```

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
use tokio::net::TcpListener;

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
        // ⛔ DO NOT duplicate middleware. Use shared middleware from lib (e.g. lib-axum-middleware)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        // .layer(lib_auth_client::middleware::AuthLayer::new()) // Example centralized auth validation
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

### Leptos 0.8.x Pattern (Microfrontend WASM)

```rust
use leptos::prelude::*;
use lib_ui::prelude::*;  // Shared UI components

// ✅ Signal creation (NOT create_signal!)
#[component]
pub fn Counter(initial: i32) -> impl IntoView {
    let (count, set_count) = signal(initial);

    let increment = move |_| *set_count.write() += 1;
    let decrement = move |_| *set_count.write() -= 1;

    view! {
        <div class="counter">
            <button on:click=decrement>"-1"</button>
            <span>{count}</span>
            <button on:click=increment>"+1"</button>
        </div>
    }
}

// ✅ Global State Management (NO prop-drilling!)
#[component]
pub fn AppRoot() -> impl IntoView {
    // Provide state globally at the root
    let (user_session, set_user_session) = signal(None::<UserSession>);
    provide_context((user_session, set_user_session));
    
    view! { <MainRouter /> }
}

// ✅ Resource for async data (calls REST API, NOT gRPC!)
#[component]
pub fn UserList() -> impl IntoView {
    let users = Resource::new(
        || (),
        |_| async move {
            gloo_net::http::Request::get("/api/v1/users")
                .send().await?.json::<Vec<User>>().await
        }
    );

    view! {
        <Suspense fallback=|| view! { <Loading /> }>
            {move || users.get().map(|result| match result {
                Ok(users) => view! { <UserTable users=users /> }.into_any(),
                Err(e) => view! { <Error message=e.to_string() /> }.into_any(),
            })}
        </Suspense>
    }
}

// ✅ Main mount (CSR)
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
```

---

## 🔐 Authentication Flow

```mermaid
sequenceDiagram
    participant User as 👤 User
    participant MF as 🌐 Microfrontend<br/>(WASM)
    participant API as ⚙️ layanan API<br/>(Axum REST)
    participant Auth as 🔐 Authenc<br/>(gRPC)
    participant Sec as 🔒 Secreton<br/>(gRPC)

    User->>MF: 1. Access app
    MF->>MF: 2. Check localStorage for JWT

    alt No JWT / Expired
        MF->>API: 3. GET /api/auth/status
        API->>Auth: 4. gRPC: ValidateSession()
        Auth-->>API: 5. Session invalid
        API-->>MF: 6. 401 Unauthorized
        MF->>User: 7. Redirect to login
    end

    MF->>API: API call with JWT header
    API->>Auth: gRPC: ValidateToken()
    Auth-->>API: Token valid + claims
    API-->>MF: API response

    Note over MF,Auth: ⛔ Microfrontend NEVER calls Authenc/Secreton directly!
```

### Auth Patterns in Code

```rust
// In microfrontend — use hooks from lib-ui
use lib_ui::hooks::use_auth;

// Session stored in localStorage (NOT sessionStorage) for cross-tab sync
// JWT validation: microfrontend → REST → gRPC Authenc
```

### Secrets Management

```rust
// In layanan backend — call Secreton via gRPC
let secret = state.secreton_client
    .get_secret(GetSecretRequest { key: "database_password".into() })
    .await?
    .into_inner()
    .value;

// NEVER use environment variables as long-term production secrets!
// Env vars are allowed only for bootstrap/local development.
```

---

## 🔍 Observability & Monitoring Patterns

### Structured Logging
- Gunakan `tracing` dengan structured fields untuk semua services
- Log level: ERROR, WARN, INFO, DEBUG, TRACE
- Include fields: `request_id`, `user_id`, `service_name`, `operation`, `duration_ms`
- Use JSON format di production untuk parsing otomatis

```rust
use tracing::{info, error, instrument};

#[instrument(skip(db))]
async fn get_item(
    db: &Database,
    item_id: Uuid,
    user_id: Uuid,
) -> Result<Item> {
    info!(item_id = %item_id, user_id = %user_id, "Fetching item");
    // ...
}
```

### Metrics Collection
- Prometheus metrics untuk semua services (counter, histogram, gauge)
- Counter: request counts, error counts, active sessions
- Histogram: request latency (P50, P95, P99), response sizes
- Gauge: active connections, queue sizes, memory usage

```rust
use prometheus::{IntCounter, Histogram};

lazy_static! {
    static ref REQUESTS_TOTAL: IntCounter = register_int_counter!(
        "api_requests_total",
        "Total number of API requests"
    ).unwrap();
    static ref REQUEST_DURATION: Histogram = register_histogram!(
        "api_request_duration_seconds",
        "API request duration in seconds"
    ).unwrap();
}
```

### Distributed Tracing
- OpenTelemetry integration untuk distributed tracing
- Trace context propagation via gRPC metadata
- Span naming convention: `service.operation` (e.g., `perlengkapan-api.get_items`)
- Include baggage untuk cross-service context

```rust
use opentelemetry::trace::TraceContextExt;

// Propagate trace context via gRPC metadata
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

### Error Types Hierarchy
- `AppError`: Base error type dengan context dan source chain
- `DomainError`: Business logic errors (validation, authorization)
- `InfrastructureError`: Database, network, external service errors
- `ValidationError`: Input validation errors dengan field-level details

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
        (status, self.to_string()).into_response()
    }
}
```

### Error Response Format
Standardized error response untuk semua API endpoints:

```json
{
  "error": {
    "code": "PERLENGKAPAN_NOT_FOUND",
    "message": "Barang dengan ID xxx tidak ditemukan",
    "details": {
      "resource": "perlengkapan",
      "id": "uuid"
    },
    "request_id": "uuid",
    "timestamp": "2025-01-15T10:30:00Z"
  }
}
```

### Error Propagation
- **Backend**: Use `thiserror` untuk error enums dengan `#[from]` conversion
- **Frontend**: Error boundaries dengan user-friendly messages
- **gRPC**: Status codes mapping (NotFound, InvalidArgument, Internal, Unauthenticated)

```rust
// gRPC status mapping
impl From<AppError> for tonic::Status {
    fn from(err: AppError) -> Self {
        match err {
            AppError::NotFound { .. } => Status::not_found(err.to_string()),
            AppError::Validation(_) => Status::invalid_argument(err.to_string()),
            AppError::Unauthorized(_) => Status::unauthenticated(err.to_string()),
            _ => Status::internal(err.to_string()),
        }
    }
}
```

---

## 🧪 Testing Strategy

### Test Pyramid
- **Unit Tests (70%)**: Test functions, modules, business logic
- **Integration Tests (20%)**: Test API endpoints, database interactions
- **E2E Tests (10%)**: Test user flows across services

### Testing Tools
- **Unit**: `cargo test` dengan built-in test framework
- **Integration**: `axum-test` untuk HTTP handlers, `tokio-test` untuk async
- **E2E**: Playwright untuk microfrontends
- **Mocking**: `mockall` untuk external dependencies

### Test Organization
```
tests/
├── unit/           # Unit tests per crate
│   ├── lib/
│   ├── layanan/
│   └── antarmuka/
├── integration/     # Integration tests
│   ├── api/
│   └── database/
└── e2e/            # End-to-end tests
    ├── flows/
    └── scenarios/
```

### Test Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_item_success() {
        let mock_db = MockDatabase::new();
        let result = create_item(&mock_db, test_data()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_item_validation_error() {
        let mock_db = MockDatabase::new();
        let invalid_data = CreateItemRequest { name: "".to_string() };
        let result = create_item(&mock_db, invalid_data).await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
```

---

## ⚡ Performance Optimization Patterns

### Database Optimization
- Use connection pooling (deadpool-postgres) dengan optimal pool size
- Index strategy untuk frequently queried columns
- Query optimization dengan `EXPLAIN ANALYZE`
- Read replicas untuk read-heavy operations
- Prepared statements untuk query yang sering dieksekusi

```rust
// Connection pool configuration
let pool = deadpool_postgres::Config::new(
    "postgres://user:pass@localhost/db"
)
.pool_size(20)
.max_lifetime(Some(Duration::from_secs(1800)))
.build()?;
```

### Caching Strategy
- Redis untuk session data dan hot data
- Cache invalidation: write-through atau cache-aside
- TTL configuration per data type (session: 30min, reference data: 1h)
- Cache warming untuk critical data pada startup

```rust
// Cache-aside pattern
async fn get_item_cached(
    cache: &RedisClient,
    db: &Database,
    id: Uuid,
) -> Result<Item> {
    let cache_key = format!("item:{}", id);

    // Try cache first
    if let Some(cached) = cache.get(&cache_key).await? {
        return Ok(serde_json::from_str(&cached)?);
    }

    // Cache miss - fetch from DB
    let item = db.get_item(id).await?;

    // Write to cache
    cache.set_ex(&cache_key, serde_json::to_string(&item)?, 3600).await?;

    Ok(item)
}
```

### WASM Optimization
- Code splitting untuk microfrontends (lazy loading routes)
- Compression: gzip/brotli untuk production builds
- Tree shaking untuk unused code
- Minimize dependency size di `lib-ui`

```toml
# Trunk.toml
[tools]
wasm-bindgen = "0.2"
wasm-opt = ['-O3', '--enable-bulk-memory']

[build]
release = true
```

---

## 🛡️ Resilience Patterns

### Circuit Breaker
- Implement circuit breaker untuk gRPC calls ke external services
- Thresholds: failure rate (50%), timeout (5s), consecutive errors (5)
- States: Closed, Open, Half-Open
- Fallback mechanisms untuk degraded service

```rust
use governor::{Quota, RateLimiter};

// Rate limiter with circuit breaker
let limiter = RateLimiter::direct(Quota::per_second(10));
if limiter.check().is_err() {
    return Err(AppError::RateLimited);
}
```

### Retry Strategy
- Exponential backoff untuk transient failures
- Max retry limits per operation (3-5 retries)
- Idempotent operations untuk safe retries
- Dead letter queue untuk failed messages

```rust
use backoff::ExponentialBackoff;

async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let backoff = ExponentialBackoff::default();
    backoff::retry(backoff, operation).await
}
```

### Rate Limiting
- Token bucket algorithm untuk rate limiting
- Per-user and per-service limits
- Distributed rate limiting dengan Redis
- Backpressure handling untuk high load

```rust
use governor::{Quota, RateLimiter};

// Per-user rate limiting
let user_limiter = RateLimiter::direct(
    Quota::per_minute(std::num::NonZeroU32::new(60).unwrap())
);
```

---

## 🚀 Deployment Patterns

### Blue-Green Deployment
- Zero-downtime deployments dengan blue-green strategy
- Health checks sebelum traffic switch
- Rollback capability dengan instant switchback
- Database migration strategy dengan backward compatibility

```yaml
# Kubernetes example
apiVersion: apps/v1
kind: Deployment
metadata:
  name: perlengkapan-api-blue
spec:
  replicas: 3
  # ... blue deployment
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: perlengkapan-api-green
spec:
  replicas: 3
  # ... green deployment
```

### Canary Deployment
- Gradual traffic shift (5% → 50% → 100%)
- Metrics monitoring during canary (error rate, latency)
- Automated rollback on error rate increase (>1%)
- Feature flags untuk gradual rollout

```yaml
# Istio VirtualService for canary
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: perlengkapan-api
spec:
  http:
  - match:
    - headers:
        canary:
          exact: "true"
    route:
    - destination:
        host: perlengkapan-api-canary
      weight: 10
    - destination:
        host: perlengkapan-api-stable
      weight: 90
```

### Configuration Management
- Environment-specific configs (development, staging, production)
- Secrets via Secreton (bukan environment variables)
- Config validation at startup dengan clear error messages
- Hot reload untuk non-critical configs

```rust
use config::{Config, Environment};

let config = Config::builder()
    .add_source(config::File::with_name("config/default"))
    .add_source(config::Environment::with_prefix("APP"))
    .build()?;
```

---

## ⚠️ Common Pitfalls

❌ **DON'T:**
- Call Authenc/Secreton directly from microfrontend
- Use `create_signal` in Leptos 0.8.x (use `signal()`)
- Duplicate auth logic in microfrontends (redirect to Portal)
- Specify dependency versions in member `Cargo.toml` files
- Use RSA for new signing implementations (use Ed25519)
- Use environment variables for production secrets (use Secreton)
- Store JWT tokens in cookies (use `localStorage`)
- **Use SQLx** (strict restriction; use `tokio-postgres` and `refinery`)
- **Add to `lib/common/`** (it is frozen; create specific crates instead)
- Use `.unwrap()` or `expect()` in production (use `?` and custom `AppError`)
- Do prop-drilling in Leptos for global state (use `provide_context`)

✅ **DO:**
- Import from `lib_ui` for shared UI components
- Add new dependencies to root `Cargo.toml` `workspace.dependencies` first
- Run `cargo fmt --all && cargo clippy --workspace` before commits
- Use REST API from microfrontend → layanan
- Use gRPC from layanan → authenc/secreton
- Use `trunk build --release` for production WASM builds

---

## 🔍 Pre-Implementation Checklist

Before implementing a feature, check:

1. **Auth-related?** → Use Portal + `use_auth()` hook, flow: microfrontend → REST → gRPC
2. **Needs secrets?** → Use Secreton gRPC client in layanan (NOT env vars)
3. **Reusable UI?** → Put in `lib/ui/`, import via `lib_ui`
4. **New dependency?** → Add to root `Cargo.toml` `[workspace.dependencies]` first
5. **Database?** → Each service has its own database for isolation

---

## 📚 Key Documentation References

| Topic | Location |
|-------|----------|
| Authenc details | `layanan/authenc/AGENTS.md` |
| Secreton details | `layanan/secreton/AGENTS.md` |
| Perlengkapan details | `layanan/perlengkapan/AGENTS.md` |
| Integrasi details | `layanan/integrasi/AGENTS.md` |
| Shared UI lib | `lib/ui/README.md` |
| Shared common lib | `lib/common/README.md` |
| Shared perlengkapan lib | `lib/perlengkapan/README.md` |
| Kubernetes deploy | `infra/k8s/` |
| Engineering docs | `docs/README.md` |
| Contributing guide | `CONTRIBUTING.md` |

---

## 🗄️ Database Architecture

Each service uses a separate database for isolation:

| Database | Service | Description |
|----------|---------|-------------|
| `perlengkapan` | layanan/perlengkapan/* | Core BMN & equipment management |
| `authenc` | layanan/authenc/* | Authentication, SSO, OIDC & Identity |
| `secreton` | layanan/secreton/* | Secrets, HSM & transit vault |

---

## 🛡️ Security Patterns

- **Zero-trust:** No implicit trust between services, all via mTLS
- **JWT validation:** On every request via middleware → Authenc gRPC
- **Audit logs:** Immutable PostgreSQL logging
- **Cryptography:** Ed25519 (signing), X25519 (key exchange), ChaCha20-Poly1305 (encryption)
- **Password hashing:** Argon2id with recommended parameters
- **Post-quantum:** ML-DSA, ML-KEM (experimental, feature-gated in authenc-crypto)

> **Catatan Akhir**: Arsitektur modular *crates/* ditujukan untuk memisahkan domain boundary dengan ketat. Harap pastikan setiap crate hanya bertanggung jawab pada satu lingkup (misalnya `api`, `core`, `storage`).
