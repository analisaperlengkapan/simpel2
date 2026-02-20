# Authenc & Portal IAM Migration Guide

> **Comprehensive guide for migrating from monolithic Authenc to multi-crate architecture**

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Why We're Migrating](#why-were-migrating)
3. [Architecture Overview](#architecture-overview)
4. [Migration Phases](#migration-phases)
5. [Step-by-Step Migration Guide](#step-by-step-migration-guide)
6. [Rollback Procedures](#rollback-procedures)
7. [Testing Strategy](#testing-strategy)
8. [Troubleshooting](#troubleshooting)
9. [FAQ](#faq)

---

## Executive Summary

This migration transforms the monolithic Authenc service (989-line app.rs, 70+ modules) into a modern, enterprise-grade IAM platform with:

- **9 modular crates**: Clear separation of concerns, maintainable architecture
- **WebAuthn/Passkeys**: PRIMARY authentication method (MANDATORY)
- **Direct REST API**: Portal communicates directly with Authenc (eliminates layanan-portal)
- **OAuth 2.1 compliance**: Modern security standards
- **FAPI support**: Optional financial-grade security (FAPI-1, FAPI-2)

**Timeline**: 14 weeks (6 phases)
**Risk Level**: Medium (comprehensive testing, rollback plan)
**Downtime**: Zero (blue-green deployment)

---

## Why We're Migrating

### Current Problems

1. **Monolithic Complexity**
   - 989-line app.rs with 50+ Arc fields
   - 70+ service modules in single crate
   - 40+ handlers tightly coupled
   - Difficult to test, maintain, and extend

2. **Unnecessary Layering**
   - layanan-portal adds complexity without value
   - Duplicate authentication logic
   - Extra network hop (microfrontend → portal → authenc)
   - Two services to maintain for authentication

3. **Limited IAM Features**
   - Portal lacks full IAM capabilities
   - No comprehensive user management
   - No realm/client management UI
   - No audit log viewer

4. **Security Gaps**
   - Password-only authentication (phishing vulnerable)
   - No passwordless authentication
   - Limited MFA options
   - No FAPI compliance for financial integrations

### Benefits of New Architecture

1. **Maintainability**
   - Clear crate boundaries
   - <500 lines per file
   - Easy to understand and modify
   - Faster onboarding for new developers

2. **Performance**
   - Eliminated unnecessary portal layer
   - Direct API communication (one hop)
   - Optimized database access
   - Better caching strategies

3. **Security**
   - Passkeys as PRIMARY authentication (phishing-resistant)
   - Multi-device passkey support
   - OAuth 2.1 compliance
   - Optional FAPI compliance

4. **Developer Experience**
   - Type-safe APIs
   - Comprehensive documentation
   - Clear error messages
   - Better testing infrastructure

---

## Architecture Overview

### Current Architecture (Monolithic)

```
┌─────────────────────────────────────────────────────────────┐
│                  Monolithic Authenc                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ app.rs (989 lines, 50+ Arc fields)                   │   │
│  │  ├─ services/ (70+ modules)                          │   │
│  │  ├─ handlers/ (40+ handlers)                         │   │
│  │  ├─ database/ (operations)                           │   │
│  │  └─ grpc/ (services)                                 │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           ▲
                           │ gRPC
                           │
┌─────────────────────────────────────────────────────────────┐
│              layanan-portal (TO BE ELIMINATED)              │
│  ├─ handlers/ (REST API)                                    │
│  ├─ middleware/ (auth, logging)                             │
│  └─ services/ (business logic)                              │
└─────────────────────────────────────────────────────────────┘
                           ▲
                           │ REST API
                           │
┌─────────────────────────────────────────────────────────────┐
│              Portal Microfrontend (Limited)                 │
│  ├─ pages/ (login, profile)                                 │
│  ├─ components/ (UI components)                             │
│  └─ features/ (limited functionality)                       │
└─────────────────────────────────────────────────────────────┘
```

**Problems**:
- Unnecessary layanan-portal service
- Duplicate authentication logic
- Extra network hop
- Limited IAM features


### Target Architecture (Multi-Crate)

```
┌─────────────────────────────────────────────────────────────┐
│           Authenc Multi-Crate Workspace                     │
│                                                              │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────┐  │
│  │ authenc-types  │  │ authenc-core   │  │ authenc-     │  │
│  │ (Shared types, │  │ (Business      │  │ crypto       │  │
│  │  traits)       │  │  logic)        │  │ (JWT, Argon2)│  │
│  └────────────────┘  └────────────────┘  └──────────────┘  │
│                                                              │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────┐  │
│  │ authenc-       │  │ authenc-api    │  │ authenc-     │  │
│  │ storage        │  │ (Public REST)  │  │ iam-api      │  │
│  │ (PostgreSQL)   │  │                │  │ (Admin REST) │  │
│  └────────────────┘  └────────────────┘  └──────────────┘  │
│                                                              │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────┐  │
│  │ authenc-grpc   │  │ authenc-mfa    │  │ authenc-     │  │
│  │ (Service-to-   │  │ (TOTP, Backup  │  │ federation   │  │
│  │  service)      │  │  codes)        │  │ (SSO/SAML)   │  │
│  └────────────────┘  └────────────────┘  └──────────────┘  │
│                                                              │
│  ┌────────────────┐                                          │
│  │ authenc-       │                                          │
│  │ webauthn       │  ← MANDATORY (PRIMARY authentication)   │
│  │ (Passkeys)     │                                          │
│  └────────────────┘                                          │
└─────────────────────────────────────────────────────────────┘
                           ▲
                           │ REST API (Direct)
                           │
┌─────────────────────────────────────────────────────────────┐
│         Portal IAM Microfrontend (Rebuilt)                  │
│  ├─ Authentication (Login, MFA, Passkeys)                   │
│  ├─ User Self-Service (Profile, Password, Passkeys)         │
│  ├─ IAM Administration (Users, Realms, Clients, Roles)      │
│  ├─ Federation Management (External IdPs)                   │
│  └─ Audit Logs (View, Export)                               │
└─────────────────────────────────────────────────────────────┘
```

**Benefits**:
- Direct communication (one hop)
- Clear crate boundaries
- Full IAM capabilities
- Passkeys as PRIMARY authentication


### Crate Responsibilities

| Crate | Responsibility | Dependencies | Lines of Code (Est.) |
|-------|---------------|--------------|---------------------|
| **authenc-types** | Shared types, traits, interfaces | None | ~500 |
| **authenc-core** | Business logic, service implementations | types, crypto, storage, mfa, federation | ~2000 |
| **authenc-crypto** | JWT, password hashing, encryption | types | ~800 |
| **authenc-storage** | Database layer, PostgreSQL stores | types | ~1500 |
| **authenc-api** | Public REST API (Axum) | core, types | ~1200 |
| **authenc-iam-api** | Admin REST API (Axum) | core, types | ~1500 |
| **authenc-grpc** | gRPC service (Tonic) | core, types | ~800 |
| **authenc-mfa** | MFA logic (TOTP, backup codes) | types, crypto | ~600 |
| **authenc-federation** | SSO/Federation (OIDC, SAML) | types, core | ~1000 |
| **authenc-webauthn** | Passkeys/WebAuthn (MANDATORY) | types, storage | ~1200 |

**Total**: ~11,100 lines (vs. current ~15,000 lines in monolith)

### Dependency Graph

```mermaid
graph TB
    TYPES[authenc-types<br/>Shared types & traits]

    CRYPTO[authenc-crypto<br/>Cryptography]
    STORAGE[authenc-storage<br/>Database]
    MFA[authenc-mfa<br/>Multi-Factor Auth]
    FED[authenc-federation<br/>SSO/Federation]
    WEBAUTHN[authenc-webauthn<br/>Passkeys]

    CORE[authenc-core<br/>Business Logic]

    API[authenc-api<br/>Public REST API]
    IAM_API[authenc-iam-api<br/>Admin REST API]
    GRPC[authenc-grpc<br/>gRPC Service]

    TYPES --> CRYPTO
    TYPES --> STORAGE
    TYPES --> MFA
    TYPES --> FED
    TYPES --> WEBAUTHN

    CRYPTO --> CORE
    STORAGE --> CORE
    MFA --> CORE
    FED --> CORE
    WEBAUTHN --> CORE

    CORE --> API
    CORE --> IAM_API
    CORE --> GRPC

    style TYPES fill:#99ff99
    style CORE fill:#99ccff
    style API fill:#ffcc99
    style IAM_API fill:#ffaa66
    style WEBAUTHN fill:#ff99cc
```

**Key Principles**:
- No circular dependencies
- Clear layering (types → services → APIs)
- Dependency injection via traits
- Minimal public API surface


---

## Migration Phases

### Overview

The migration follows a **6-phase approach** over **14 weeks**:

| Phase | Duration | Focus | Risk Level |
|-------|----------|-------|------------|
| **Phase 1: Preparation** | Week 1-2 | Workspace setup, CI/CD | Low |
| **Phase 2: Core Migration** | Week 3-6 | Storage, crypto, core logic, **WebAuthn** | Medium |
| **Phase 3: API Migration** | Week 7-8 | REST APIs, gRPC | Medium |
| **Phase 4: Feature Migration** | Week 9-10 | MFA, Federation | Low |
| **Phase 5: Portal Refactoring** | Week 11-12 | Portal rebuild, eliminate layanan-portal | High |
| **Phase 6: Cleanup** | Week 13-14 | Remove old code, optimization, production | Medium |

**Optional Phases** (if FAPI compliance required):
- **Phase 7: FAPI-1** (4-6 weeks) - mTLS, JAR, JARM
- **Phase 8: FAPI-2** (6-8 weeks) - PAR, DPoP, Grant Management

### Phase 1: Preparation (Week 1-2)

**Goal**: Set up multi-crate workspace structure

**Tasks**:
1. Create authenc-types crate with core traits
2. Update root Cargo.toml with workspace members
3. Create directory structure for all 9 crates
4. Set up CI/CD pipeline for workspace
5. **Document migration plan** (this document)

**Deliverables**:
- ✅ All crates compile with `cargo check --workspace`
- ✅ No circular dependencies
- ✅ CI/CD pipeline passes
- ✅ MIGRATION.md complete

**Risk**: Low (no production code changes)

### Phase 2: Core Migration (Week 3-6)

**Goal**: Implement core business logic and **WebAuthn/Passkeys**

**Tasks**:
1. Implement authenc-storage (PostgreSQL stores)
2. Implement authenc-crypto (JWT, Argon2, encryption)
3. Implement authenc-core (authentication, user management, OAuth2)
4. **Implement authenc-webauthn (MANDATORY - PRIMARY authentication)**
   - Passkey registration flow
   - Passkey authentication flow
   - Credential management
   - Replay attack prevention
   - Origin binding enforcement
5. Write unit tests (>80% coverage)
6. Write property-based tests for crypto and WebAuthn

**Deliverables**:
- ✅ All core services implemented
- ✅ WebAuthn integration complete
- ✅ Unit tests pass (>80% coverage)
- ✅ Property-based tests pass

**Risk**: Medium (core logic changes, WebAuthn integration)

**Rollback**: Keep old code in parallel (not deployed)


### Phase 3: API Migration (Week 7-8)

**Goal**: Implement REST and gRPC APIs

**Tasks**:
1. Implement authenc-api (public REST endpoints)
   - Authentication endpoints (login, logout, refresh)
   - **WebAuthn endpoints (register, authenticate, manage)**
   - OAuth2/OIDC endpoints
   - Token validation endpoint
2. Implement authenc-iam-api (admin REST endpoints)
   - User management
   - Realm management
   - Client management
   - Role management
   - Audit logs
3. Implement authenc-grpc (service-to-service)
4. Add CORS, rate limiting, security headers
5. Write integration tests for all endpoints

**Deliverables**:
- ✅ All REST endpoints functional
- ✅ WebAuthn endpoints tested with browser API
- ✅ gRPC service functional
- ✅ Integration tests pass
- ✅ OpenAPI documentation generated

**Risk**: Medium (API contract changes)

**Rollback**: Keep old API endpoints active (dual-run)

### Phase 4: Feature Migration (Week 9-10)

**Goal**: Migrate MFA and Federation features

**Tasks**:
1. Implement authenc-mfa (TOTP, backup codes)
2. Implement authenc-federation (OIDC, SAML providers)
3. Write integration tests for MFA and federation
4. Test with external IdPs (Google, Microsoft)

**Deliverables**:
- ✅ MFA fully functional
- ✅ Federation with external IdPs working
- ✅ Integration tests pass

**Risk**: Low (isolated features)

**Rollback**: Disable new features via feature flags

### Phase 5: Portal Refactoring (Week 11-12)

**Goal**: Rebuild Portal IAM and eliminate layanan-portal

**Tasks**:
1. Rebuild Portal with Leptos 0.8.x
   - Authentication pages (login with passkey/password)
   - **Passkey management page (MANDATORY)**
   - User self-service pages
   - IAM administration pages
   - Audit log viewer
2. Implement AuthencApiClient (direct REST API calls)
3. Update other microfrontends to use Authenc API directly
4. Decommission layanan-portal service
5. Write integration tests for Portal

**Deliverables**:
- ✅ Portal fully functional with all IAM features
- ✅ Passkey authentication works across browsers
- ✅ layanan-portal eliminated
- ✅ Other microfrontends updated
- ✅ Integration tests pass

**Risk**: High (user-facing changes, service elimination)

**Rollback**: Revert to old Portal, re-enable layanan-portal


### Phase 6: Cleanup and Optimization (Week 13-14)

**Goal**: Remove old code, optimize, and deploy to production

**Tasks**:
1. Remove old monolithic code (app.rs, old services/)
2. Update documentation (AGENTS.md, README.md)
3. Performance testing and optimization
   - Load testing (1000 req/s authentication)
   - Database query optimization
   - WASM bundle optimization
4. Security audit
   - Penetration testing
   - Vulnerability scanning
   - WebAuthn security validation
5. Production deployment
   - Update Kubernetes manifests
   - Configure monitoring and alerting
   - Blue-green deployment
   - Smoke tests

**Deliverables**:
- ✅ Old code removed
- ✅ Documentation updated
- ✅ Performance targets met (<100ms p99 auth)
- ✅ Security audit passed
- ✅ Production deployment successful
- ✅ Zero downtime achieved

**Risk**: Medium (production deployment)

**Rollback**: Blue-green deployment allows instant rollback

---

## Step-by-Step Migration Guide

### Prerequisites

Before starting the migration, ensure:

1. **Development Environment**
   - Rust 1.90+ (Edition 2024)
   - PostgreSQL 16.x running
   - Redis 7.x running (optional)
   - Secreton service accessible
   - Kubernetes cluster available

2. **Access and Permissions**
   - Write access to repository
   - Database admin credentials
   - Kubernetes cluster admin access
   - CI/CD pipeline access

3. **Backups**
   - Database backup created
   - Current code tagged in Git
   - Kubernetes manifests backed up

### Phase 1: Workspace Setup

#### Step 1.1: Create authenc-types Crate

```bash
cd infra/authenc
mkdir -p crates/types/src
cd crates/types

# Create Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "authenc-types"
version = "0.1.0"
edition = "2024"

[dependencies]
uuid = { workspace = true, features = ["v4", "serde"] }
serde = { workspace = true, features = ["derive"] }
chrono = { workspace = true, features = ["serde"] }
async-trait = { workspace = true }
EOF

# Create lib.rs with core types
cat > src/lib.rs << 'EOF'
//! Shared types and traits for Authenc

pub mod types;
pub mod traits;
pub mod error;

pub use types::*;
pub use traits::*;
pub use error::*;
EOF
```


#### Step 1.2: Update Root Cargo.toml

```bash
cd /var/www/simpelv2

# Add workspace members
# Edit Cargo.toml and add:
```

```toml
[workspace]
members = [
    # ... existing members ...
    "infra/authenc/crates/types",
    "infra/authenc/crates/core",
    "infra/authenc/crates/crypto",
    "infra/authenc/crates/storage",
    "infra/authenc/crates/api",
    "infra/authenc/crates/iam-api",
    "infra/authenc/crates/grpc",
    "infra/authenc/crates/mfa",
    "infra/authenc/crates/federation",
    "infra/authenc/crates/webauthn",
]

[workspace.dependencies]
# Authenc crates
authenc-types = { path = "infra/authenc/crates/types" }
authenc-core = { path = "infra/authenc/crates/core" }
authenc-crypto = { path = "infra/authenc/crates/crypto" }
authenc-storage = { path = "infra/authenc/crates/storage" }
authenc-api = { path = "infra/authenc/crates/api" }
authenc-iam-api = { path = "infra/authenc/crates/iam-api" }
authenc-grpc = { path = "infra/authenc/crates/grpc" }
authenc-mfa = { path = "infra/authenc/crates/mfa" }
authenc-federation = { path = "infra/authenc/crates/federation" }
authenc-webauthn = { path = "infra/authenc/crates/webauthn" }

# WebAuthn (MANDATORY)
webauthn-rs = { version = "0.5", features = ["danger-allow-state-serialisation"] }
webauthn-rs-proto = "0.5"

# ... existing dependencies ...
```

#### Step 1.3: Verify Workspace

```bash
# Check all crates compile
cargo check --workspace

# Run tests
cargo test --workspace

# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace
```

**Expected Output**:
```
   Compiling authenc-types v0.1.0
   Compiling authenc-crypto v0.1.0
   Compiling authenc-storage v0.1.0
   ...
    Finished dev [unoptimized + debuginfo] target(s) in 45.23s
```


### Phase 2: Core Implementation

#### Step 2.1: Implement authenc-storage

```bash
cd infra/authenc/crates/storage

# Create PostgreSQL store implementations
mkdir -p src/postgres
touch src/postgres/user_store.rs
touch src/postgres/session_store.rs
touch src/postgres/realm_store.rs
touch src/postgres/client_store.rs
```

**Example: User Store Implementation**

```rust
// src/postgres/user_store.rs
use authenc_types::{User, UserId, UserStore, CreateUserRequest, Result};
use deadpool_postgres::Pool;

pub struct PostgresUserStore {
    pool: Pool,
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn get_user(&self, id: UserId) -> Result<User> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "SELECT * FROM users WHERE id = $1",
            &[&id.0]
        ).await?;

        Ok(User::from_row(row)?)
    }

    async fn create_user(&self, req: CreateUserRequest) -> Result<User> {
        let id = UserId(Uuid::new_v4());
        let client = self.pool.get().await?;

        client.execute(
            "INSERT INTO users (id, username, email, password_hash, created_at)
             VALUES ($1, $2, $3, $4, NOW())",
            &[&id.0, &req.username, &req.email, &req.password_hash]
        ).await?;

        self.get_user(id).await
    }

    // ... other methods
}
```

#### Step 2.2: Implement authenc-webauthn (MANDATORY)

```bash
cd infra/authenc/crates/webauthn

# Create WebAuthn service
touch src/service.rs
touch src/credential_store.rs
touch src/models.rs
```

**Example: WebAuthn Service**

```rust
// src/service.rs
use webauthn_rs::prelude::*;
use authenc_types::{UserId, Result};

pub struct WebAuthnService {
    webauthn: Arc<Webauthn>,
    credential_store: Arc<dyn CredentialStore>,
}

impl WebAuthnService {
    pub fn new(rp_id: String, rp_origin: Url) -> Result<Self> {
        let builder = WebauthnBuilder::new(&rp_id, &rp_origin)?;
        let webauthn = Arc::new(builder.build()?);

        Ok(Self { webauthn, credential_store })
    }

    pub async fn start_registration(
        &self,
        user_id: UserId,
        username: &str,
        display_name: &str,
    ) -> Result<(CreationChallengeResponse, PasskeyRegistration)> {
        // Get existing credentials for exclusion
        let existing = self.credential_store
            .get_credentials_for_user(user_id)
            .await?;

        let excluded: Vec<CredentialID> = existing
            .iter()
            .map(|c| c.cred_id.clone())
            .collect();

        // Start registration ceremony
        let user_unique_id = Uuid::new_v4();
        let (ccr, reg_state) = self.webauthn.start_passkey_registration(
            user_unique_id,
            username,
            display_name,
            Some(excluded),
        )?;

        Ok((ccr, PasskeyRegistration { user_id, state: reg_state }))
    }

    pub async fn finish_registration(
        &self,
        user_id: UserId,
        reg: &RegisterPublicKeyCredential,
        state: &PasskeyRegistration,
    ) -> Result<StoredCredential> {
        // Finish registration ceremony
        let passkey = self.webauthn.finish_passkey_registration(reg, &state.state)?;

        // Store credential
        let credential = StoredCredential {
            id: Uuid::new_v4(),
            user_id,
            cred_id: passkey.cred_id().clone(),
            cred: passkey,
            nickname: None,
            created_at: Utc::now(),
            last_used: None,
            counter: 0,
        };

        self.credential_store.store_credential(&credential).await?;

        Ok(credential)
    }

    // ... authentication methods
}
```


#### Step 2.3: Write Tests

```bash
cd infra/authenc/crates/storage

# Create test file
mkdir -p tests
touch tests/user_store_tests.rs
```

**Example: Unit Test**

```rust
// tests/user_store_tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_get_user() {
        let store = setup_test_store().await;

        let req = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
        };

        let user = store.create_user(req).await.unwrap();
        assert_eq!(user.username, "testuser");

        let retrieved = store.get_user(user.id).await.unwrap();
        assert_eq!(retrieved.id, user.id);
    }
}
```

**Run Tests**:
```bash
cargo test --workspace
```

### Phase 3: API Implementation

#### Step 3.1: Implement authenc-api

```bash
cd infra/authenc/crates/api

# Create API structure
mkdir -p src/handlers
touch src/handlers/auth.rs
touch src/handlers/webauthn.rs
touch src/handlers/oauth2.rs
touch src/state.rs
touch src/router.rs
```

**Example: WebAuthn Endpoints**

```rust
// src/handlers/webauthn.rs
use axum::{extract::State, Json, Extension};
use authenc_types::{AuthenticatedUser, Result};

pub async fn start_registration_handler(
    State(state): State<Arc<ApiState>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<WebAuthnRegistrationResponse>> {
    let (ccr, reg_state) = state.webauthn_service
        .start_registration(user.id, &user.username, &user.display_name)
        .await?;

    // Store state temporarily (5 minutes TTL)
    let state_id = Uuid::new_v4();
    state.temp_storage.store(state_id, reg_state, Duration::minutes(5)).await?;

    Ok(Json(WebAuthnRegistrationResponse {
        challenge: ccr,
        state_id,
    }))
}

pub async fn finish_registration_handler(
    State(state): State<Arc<ApiState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<FinishRegistrationRequest>,
) -> Result<Json<StoredCredential>> {
    let reg_state = state.temp_storage.get(req.state_id).await?;

    let credential = state.webauthn_service
        .finish_registration(user.id, &req.credential, &reg_state)
        .await?;

    state.temp_storage.delete(req.state_id).await?;

    Ok(Json(credential))
}

// ... authentication endpoints
```


#### Step 3.2: Create Router

```rust
// src/router.rs
use axum::{Router, routing::{get, post, put, delete}};

pub fn create_router(state: Arc<ApiState>) -> Router {
    Router::new()
        // Authentication endpoints
        .route("/api/v1/auth/login", post(handlers::auth::login_handler))
        .route("/api/v1/auth/logout", post(handlers::auth::logout_handler))
        .route("/api/v1/auth/refresh", post(handlers::auth::refresh_handler))

        // WebAuthn endpoints (MANDATORY)
        .route("/api/v1/auth/webauthn/register/start",
               post(handlers::webauthn::start_registration_handler))
        .route("/api/v1/auth/webauthn/register/finish",
               post(handlers::webauthn::finish_registration_handler))
        .route("/api/v1/auth/webauthn/authenticate/start",
               post(handlers::webauthn::start_authentication_handler))
        .route("/api/v1/auth/webauthn/authenticate/finish",
               post(handlers::webauthn::finish_authentication_handler))
        .route("/api/v1/auth/webauthn/credentials",
               get(handlers::webauthn::list_credentials_handler))
        .route("/api/v1/auth/webauthn/credentials/:id",
               delete(handlers::webauthn::delete_credential_handler))

        // OAuth2 endpoints
        .route("/api/v1/oauth2/authorize", get(handlers::oauth2::authorize_handler))
        .route("/api/v1/oauth2/token", post(handlers::oauth2::token_handler))

        // Middleware
        .layer(cors_layer())
        .layer(rate_limiting_layer())
        .layer(tracing_layer())
        .with_state(state)
}
```

#### Step 3.3: Test API Endpoints

```bash
# Start API server
cargo run --bin authenc-api

# Test WebAuthn registration (in another terminal)
curl -X POST http://localhost:8080/api/v1/auth/webauthn/register/start \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json"
```

### Phase 5: Portal Refactoring

#### Step 5.1: Create Portal Structure

```bash
cd antarmuka/portal

# Create new structure
mkdir -p src/{pages,components,features,state,api,utils}
touch src/pages/login.rs
touch src/pages/profile/passkeys.rs
touch src/api/client.rs
touch src/state.rs
```

#### Step 5.2: Implement Passkey Management Page

```rust
// src/pages/profile/passkeys.rs
use leptos::prelude::*;
use crate::api::AuthencApiClient;

#[component]
pub fn PasskeysManagementPage() -> impl IntoView {
    let passkeys = Resource::new(
        || (),
        |_| async move {
            let client = AuthencApiClient::new();
            client.webauthn().list_credentials().await
        }
    );

    let add_passkey_action = Action::new(move |_: &()| async move {
        let client = AuthencApiClient::new();

        // Start registration
        let start_response = client.webauthn().start_registration().await?;

        // Call browser WebAuthn API
        let credential = create_credential(start_response.options).await?;

        // Finish registration
        client.webauthn().finish_registration(credential, start_response.state).await
    });

    view! {
        <div class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold">"Passkeys"</h1>

            <button
                on:click=move |_| add_passkey_action.dispatch(())
                class="px-4 py-2 bg-indigo-600 text-white rounded-md"
            >
                "Add Passkey"
            </button>

            <Suspense fallback=move || view! { <LoadingSpinner /> }>
                {move || passkeys.get().map(|result| match result {
                    Ok(credentials) => view! {
                        <ul class="divide-y divide-gray-200">
                            <For
                                each=move || credentials.clone()
                                key=|cred| cred.id
                                children=move |cred| view! {
                                    <li class="px-6 py-4">
                                        <div class="flex justify-between">
                                            <div>
                                                <div class="font-medium">
                                                    {cred.nickname.unwrap_or("Unnamed".to_string())}
                                                </div>
                                                <div class="text-sm text-gray-500">
                                                    "Added " {format_date(cred.created_at)}
                                                </div>
                                            </div>
                                            <button
                                                on:click=move |_| delete_passkey(cred.id)
                                                class="text-red-600"
                                            >
                                                "Delete"
                                            </button>
                                        </div>
                                    </li>
                                }
                            />
                        </ul>
                    }.into_any(),
                    Err(e) => view! { <ErrorDisplay error=e /> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

// Browser WebAuthn API wrapper
async fn create_credential(options: PublicKeyCredentialCreationOptions)
    -> Result<PublicKeyCredential>
{
    use web_sys::{window, CredentialCreationOptions};

    let window = window().ok_or(ApiError::BrowserApiUnavailable)?;
    let navigator = window.navigator();
    let credentials = navigator.credentials();

    let js_options: JsValue = serde_wasm_bindgen::to_value(&options)?;
    let credential_options = CredentialCreationOptions::from(js_options);

    let promise = credentials.create_with_options(&credential_options)?;
    let js_credential = wasm_bindgen_futures::JsFuture::from(promise).await?;

    serde_wasm_bindgen::from_value(js_credential).map_err(Into::into)
}
```


#### Step 5.3: Build and Test Portal

```bash
cd antarmuka/portal

# Build for development
trunk serve --open

# Build for production
trunk build --release

# Test in browsers
# - Chrome 93+ (passkey sync)
# - Safari 16+ (iCloud Keychain)
# - Firefox 88+
# - Edge 90+
```

#### Step 5.4: Eliminate layanan-portal

```bash
# 1. Verify all Portal functionality uses Authenc API directly
grep -r "layanan-portal" antarmuka/portal/src/
# Should return no results

# 2. Update other microfrontends
cd antarmuka/perlengkapan
# Update API calls to use Authenc directly

# 3. Remove layanan-portal from Kubernetes
kubectl delete deployment layanan-portal -n simpelv2-production
kubectl delete service layanan-portal -n simpelv2-production

# 4. Remove from CI/CD
# Edit .github/workflows/deploy.yml
# Remove layanan-portal build and deploy steps

# 5. Archive codebase
git mv layanan/portal layanan/portal.archived
git commit -m "Archive layanan-portal (eliminated)"
```

---

## Rollback Procedures

### Immediate Rollback (Critical Issues)

If critical issues are discovered during migration:

#### Step 1: Identify Issue Severity

**Critical Issues** (require immediate rollback):
- Authentication completely broken
- Data loss or corruption
- Security vulnerability
- >50% error rate

**Non-Critical Issues** (can be fixed forward):
- Minor bugs
- Performance degradation <20%
- UI issues
- <5% error rate

#### Step 2: Execute Rollback

**For Phase 2-4 (Core/API/Features)**:

```bash
# 1. Revert to old monolithic code
cd infra/authenc
git checkout <previous-tag>

# 2. Rebuild old code
cargo build --release --bin authenc

# 3. Redeploy old version
kubectl set image deployment/authenc authenc=localhost:32000/simpelv2/authenc:<old-tag>

# 4. Verify rollback
kubectl rollout status deployment/authenc
curl http://authenc.internal/health
```

**For Phase 5 (Portal)**:

```bash
# 1. Revert Portal to old version
cd antarmuka/portal
git checkout <previous-tag>

# 2. Rebuild old Portal
trunk build --release

# 3. Redeploy old Portal
kubectl set image deployment/portal portal=localhost:32000/simpelv2/portal:<old-tag>

# 4. Re-enable layanan-portal
kubectl apply -f infra/k8s/overlays/production/layanan-portal.yaml

# 5. Verify rollback
curl http://portal.simpel.kejaksaan.go.id/
```


### Gradual Rollback (Feature Flags)

For non-critical issues, use feature flags to disable new features:

```rust
// In authenc-api/src/config.rs
pub struct FeatureFlags {
    pub webauthn_enabled: bool,
    pub new_oauth2_flow: bool,
    pub new_mfa_flow: bool,
}

// In handlers
if !state.feature_flags.webauthn_enabled {
    return Err(ApiError::FeatureDisabled("WebAuthn"));
}
```

**Disable Feature**:
```bash
# Update ConfigMap
kubectl edit configmap authenc-config -n simpelv2-production

# Set feature flag to false
data:
  FEATURE_WEBAUTHN_ENABLED: "false"

# Restart pods to pick up new config
kubectl rollout restart deployment/authenc -n simpelv2-production
```

### Data Rollback

If database schema changes cause issues:

```bash
# 1. Stop all Authenc pods
kubectl scale deployment/authenc --replicas=0

# 2. Restore database from backup
pg_restore -d authenc < authenc_backup_$(date +%Y%m%d).sql

# 3. Run down migrations
cd infra/authenc/migrations
refinery migrate -e DATABASE_URL -p . -t <previous-version>

# 4. Restart pods with old code
kubectl scale deployment/authenc --replicas=3
```

### Rollback Checklist

- [ ] Identify issue severity (critical vs. non-critical)
- [ ] Notify stakeholders (Slack, email)
- [ ] Execute rollback procedure
- [ ] Verify rollback successful
- [ ] Monitor error rates and performance
- [ ] Document issue and root cause
- [ ] Create fix plan
- [ ] Schedule re-deployment

---

## Testing Strategy

### Unit Testing

**Goal**: >80% line coverage, >90% branch coverage

**Tools**: `cargo test`, `cargo-tarpaulin`

**Example**:
```bash
# Run all unit tests
cargo test --workspace

# Run with coverage
cargo tarpaulin --workspace --out Html --output-dir coverage/

# View coverage report
open coverage/index.html
```

**Critical Areas**:
- Authentication logic
- Password hashing
- JWT generation/validation
- WebAuthn registration/authentication
- OAuth2 flows
- Database operations


### Integration Testing

**Goal**: Test all API endpoints end-to-end

**Tools**: `reqwest`, `tokio-test`

**Example**:
```rust
#[tokio::test]
async fn test_webauthn_registration_flow() {
    let client = TestClient::new();

    // 1. Start registration
    let start_response = client
        .post("/api/v1/auth/webauthn/register/start")
        .bearer_auth(&user_token)
        .send()
        .await
        .unwrap();

    assert_eq!(start_response.status(), 200);
    let start_data: WebAuthnRegistrationResponse = start_response.json().await.unwrap();

    // 2. Simulate browser WebAuthn API
    let credential = simulate_webauthn_create(&start_data.challenge);

    // 3. Finish registration
    let finish_response = client
        .post("/api/v1/auth/webauthn/register/finish")
        .bearer_auth(&user_token)
        .json(&FinishRegistrationRequest {
            state_id: start_data.state_id,
            credential,
        })
        .send()
        .await
        .unwrap();

    assert_eq!(finish_response.status(), 200);
    let credential: StoredCredential = finish_response.json().await.unwrap();
    assert_eq!(credential.user_id, user.id);
}
```

### Property-Based Testing

**Goal**: Verify cryptographic invariants and security properties

**Tools**: `proptest`, `quickcheck`

**Example**:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_password_hash_verify_roundtrip(password in "[a-zA-Z0-9]{8,50}") {
        let hasher = Argon2PasswordHasher::new();
        let hash = hasher.hash(&password).unwrap();
        assert!(hasher.verify(&password, &hash).unwrap());
    }

    #[test]
    fn test_jwt_encode_decode_roundtrip(user_id in any::<Uuid>()) {
        let jwt_service = JwtService::new();
        let claims = TokenClaims {
            sub: user_id.to_string(),
            exp: (Utc::now() + Duration::minutes(15)).timestamp(),
            iss: "authenc".to_string(),
        };

        let token = jwt_service.generate_access_token(&claims).unwrap();
        let decoded = jwt_service.verify_token(&token).unwrap();

        assert_eq!(decoded.sub, claims.sub);
    }

    #[test]
    fn test_webauthn_counter_monotonicity(counter in 0u32..1000u32) {
        // Property: Credential counter always increases
        let mut credential = create_test_credential();
        credential.counter = counter;

        // Simulate authentication
        let new_counter = counter + 1;
        assert!(new_counter > credential.counter);
    }
}
```

### Performance Testing

**Goal**: <100ms p99 authentication latency, 1000 req/s throughput

**Tools**: `wrk`, `k6`

**Example**:
```bash
# Load test authentication endpoint
wrk -t12 -c400 -d30s --latency \
  -s scripts/auth_load_test.lua \
  http://authenc.internal/api/v1/auth/login

# Expected output:
# Latency Distribution
#   50%    45ms
#   75%    68ms
#   90%    82ms
#   99%    95ms  ← Target: <100ms
# Requests/sec: 1250  ← Target: >1000
```


### Security Testing

**Goal**: No high/critical vulnerabilities, OWASP Top 10 compliance

**Tools**: `cargo audit`, `cargo deny`, penetration testing

**Example**:
```bash
# Vulnerability scanning
cargo audit

# License and advisory checks
cargo deny check

# WebAuthn security validation
# - Test replay attack prevention
# - Test origin binding enforcement
# - Test credential counter validation
# - Test phishing resistance

# Penetration testing checklist:
# - [ ] SQL injection attempts
# - [ ] XSS attempts
# - [ ] CSRF attempts
# - [ ] Brute force attacks
# - [ ] Session hijacking
# - [ ] Token manipulation
# - [ ] WebAuthn replay attacks
# - [ ] Origin spoofing
```

---

## Troubleshooting

### Common Issues

#### Issue 1: WebAuthn Registration Fails

**Symptoms**:
- Browser shows "NotAllowedError"
- Registration challenge fails

**Causes**:
- HTTPS not enabled (WebAuthn requires HTTPS)
- Origin mismatch (RP ID doesn't match domain)
- User cancelled prompt

**Solutions**:
```bash
# 1. Verify HTTPS is enabled
curl -I https://portal.simpel.kejaksaan.go.id/

# 2. Check RP ID configuration
# In authenc-webauthn config:
WEBAUTHN_RP_ID=simpel.kejaksaan.go.id
WEBAUTHN_RP_ORIGIN=https://portal.simpel.kejaksaan.go.id

# 3. Check browser console for errors
# Open DevTools → Console
# Look for WebAuthn API errors

# 4. Test with different authenticator
# Try platform authenticator (Touch ID) vs security key
```

#### Issue 2: Database Connection Pool Exhausted

**Symptoms**:
- "Connection pool timeout" errors
- Slow API responses

**Causes**:
- Too many concurrent requests
- Long-running queries
- Connection leaks

**Solutions**:
```bash
# 1. Increase pool size
# In authenc-storage config:
DATABASE_POOL_SIZE=50  # Increase from 20

# 2. Check for connection leaks
# Look for queries without proper cleanup

# 3. Optimize slow queries
# Run EXPLAIN ANALYZE on slow queries
psql -d authenc -c "EXPLAIN ANALYZE SELECT * FROM users WHERE username = 'test';"

# 4. Add missing indexes
psql -d authenc -c "CREATE INDEX idx_users_username ON users(username);"
```


#### Issue 3: Portal WASM Bundle Too Large

**Symptoms**:
- Slow page load times (>5 seconds)
- Large WASM file (>2MB)

**Causes**:
- Debug build
- Unused dependencies
- No code splitting

**Solutions**:
```bash
# 1. Build with release mode
trunk build --release

# 2. Enable wasm-opt
# In Trunk.toml:
[build]
release = true

[[hooks]]
stage = "post_build"
command = "wasm-opt"
command_arguments = ["-Oz", "--output", "dist/portal.wasm", "dist/portal.wasm"]

# 3. Check bundle size
ls -lh dist/*.wasm

# Target: <500KB compressed

# 4. Analyze bundle
wasm-pack build --target web
twiggy top dist/portal.wasm
```

#### Issue 4: JWT Token Validation Fails

**Symptoms**:
- "Invalid signature" errors
- "Token expired" errors

**Causes**:
- Key mismatch (signing key != verification key)
- Clock skew
- Token expired

**Solutions**:
```bash
# 1. Verify signing key matches
# Check Secreton for JWT signing key
secreton-cli get jwt_signing_key

# 2. Check token expiry
# Decode JWT (without verification)
echo $TOKEN | jwt decode -

# 3. Check clock synchronization
timedatectl status

# 4. Verify issuer matches
# In JWT claims:
{
  "iss": "https://authenc.kejaksaan.go.id",  # Must match config
  "sub": "user-id",
  "exp": 1234567890
}
```

### Debugging Tips

1. **Enable Debug Logging**
   ```bash
   RUST_LOG=debug cargo run --bin authenc-api
   ```

2. **Use Tracing**
   ```rust
   use tracing::{info, debug, error};

   #[tracing::instrument]
   async fn authenticate_user(username: &str) -> Result<User> {
       debug!("Authenticating user: {}", username);
       // ...
       info!("User authenticated successfully");
       Ok(user)
   }
   ```

3. **Check Kubernetes Logs**
   ```bash
   kubectl logs -f deployment/authenc -n simpelv2-production
   kubectl logs -f deployment/portal -n simpelv2-production
   ```

4. **Monitor Metrics**
   ```bash
   # Prometheus queries
   rate(authenc_auth_requests_total[5m])
   histogram_quantile(0.99, authenc_auth_duration_seconds)
   ```

---

## FAQ

### General Questions

**Q: How long will the migration take?**
A: 14 weeks for core migration (6 phases). Optional FAPI compliance adds 4-14 weeks.

**Q: Will there be downtime?**
A: No. We use blue-green deployment for zero-downtime migration.

**Q: Can we rollback if issues occur?**
A: Yes. We maintain old code in parallel during phases 1-5 and have comprehensive rollback procedures.

**Q: What happens to existing users?**
A: All existing users are preserved. No data loss. Existing passwords remain valid.


### WebAuthn/Passkeys Questions

**Q: Why are passkeys MANDATORY?**
A: Passkeys provide phishing-resistant authentication, better security than passwords, and improved user experience. They are the future of authentication.

**Q: What browsers support passkeys?**
A: Chrome 67+, Firefox 60+, Safari 13+, Edge 18+. For passkey sync: Chrome 93+, Safari 16+.

**Q: Can users still use passwords?**
A: Yes. Passwords remain as a SECONDARY authentication method. Passkeys are PRIMARY (default).

**Q: What if a user loses their device?**
A: Passkeys sync across devices via platform providers (iCloud Keychain, Google Password Manager). Users can also register multiple passkeys.

**Q: Do we need special hardware?**
A: No. Platform authenticators (Touch ID, Face ID, Windows Hello) work out of the box. Security keys (YubiKey) are optional.

**Q: How do we handle replay attacks?**
A: WebAuthn includes a credential counter that increments on each use. We verify the counter increases to prevent replay attacks.

### Architecture Questions

**Q: Why eliminate layanan-portal?**
A: It adds unnecessary complexity without value. Direct API communication is simpler, faster, and easier to maintain.

**Q: Why 9 crates instead of 1?**
A: Clear separation of concerns, better testability, easier maintenance, and faster compilation (only changed crates rebuild).

**Q: Can we add more crates later?**
A: Yes. The architecture is designed to be extensible. New crates can be added as needed.

**Q: What about backward compatibility?**
A: We maintain API versioning (/api/v1/, /api/v2/) and provide migration guides for breaking changes.

### FAPI Questions

**Q: Do we need FAPI compliance?**
A: Only if required for specific integrations (financial institutions, government agencies requiring FAPI certification). It's OPTIONAL.

**Q: What's the difference between FAPI-1 and FAPI-2?**
A: FAPI-1 (Advanced Profile) adds mTLS, signed request objects, and JARM. FAPI-2 (Security Profile) adds PAR, DPoP, and Grant Management.

**Q: How long does FAPI implementation take?**
A: FAPI-1: 4-6 weeks. FAPI-2: 6-8 weeks (requires FAPI-1 first).

**Q: Can we get FAPI certified?**
A: Yes. After implementation, run OpenID Foundation conformance tests and apply for certification.

### Performance Questions

**Q: What are the performance targets?**
A: Authentication <100ms p99, token validation <50ms p99, 1000 req/s throughput.

**Q: How do we monitor performance?**
A: Prometheus metrics, distributed tracing (OpenTelemetry), and structured logging.

**Q: What if performance degrades?**
A: We have comprehensive performance testing and optimization procedures. Rollback is available if needed.

### Security Questions

**Q: How do we ensure security?**
A: Comprehensive security testing (penetration testing, vulnerability scanning, OWASP Top 10 compliance), security audit, and code review.

**Q: What about password security?**
A: Passwords are hashed with Argon2id (64 MB memory, 3 iterations, 4 threads). JWTs are signed with Ed25519.

**Q: How do we handle secrets?**
A: All secrets are stored in Secreton (secrets vault). No secrets in environment variables or code.

**Q: What about GDPR compliance?**
A: We support right to access, right to be forgotten, data portability, and consent management.

---

## Appendix

### A. Crate Dependency Matrix

| Crate | Depends On |
|-------|-----------|
| authenc-types | None |
| authenc-crypto | types |
| authenc-storage | types |
| authenc-mfa | types, crypto |
| authenc-federation | types, core |
| authenc-webauthn | types, storage |
| authenc-core | types, crypto, storage, mfa, federation, webauthn |
| authenc-api | core, types |
| authenc-iam-api | core, types |
| authenc-grpc | core, types |

### B. Database Schema Changes

**New Tables**:
- `webauthn_credentials` - Passkey storage
- `webauthn_registration_state` - Temporary registration state
- `webauthn_authentication_state` - Temporary authentication state

**Modified Tables**:
- `users` - Add `webauthn_enabled` column
- `audit_logs` - Add `webauthn_event` type

**Migrations**:
```sql
-- Migration: 038_webauthn_support.sql
CREATE TABLE webauthn_credentials (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    cred_id BYTEA NOT NULL UNIQUE,
    public_key BYTEA NOT NULL,
    counter BIGINT NOT NULL DEFAULT 0,
    nickname TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used TIMESTAMPTZ,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_webauthn_credentials_user_id ON webauthn_credentials(user_id);
CREATE INDEX idx_webauthn_credentials_cred_id ON webauthn_credentials(cred_id);

ALTER TABLE users ADD COLUMN webauthn_enabled BOOLEAN NOT NULL DEFAULT FALSE;
```


### C. API Endpoint Mapping

**Old (layanan-portal) → New (authenc-api)**:

| Old Endpoint | New Endpoint | Notes |
|-------------|--------------|-------|
| POST /auth/login | POST /api/v1/auth/login | Direct to Authenc |
| POST /auth/logout | POST /api/v1/auth/logout | Direct to Authenc |
| GET /auth/me | GET /api/v1/auth/me | Direct to Authenc |
| N/A | POST /api/v1/auth/webauthn/register/start | New (WebAuthn) |
| N/A | POST /api/v1/auth/webauthn/register/finish | New (WebAuthn) |
| N/A | POST /api/v1/auth/webauthn/authenticate/start | New (WebAuthn) |
| N/A | POST /api/v1/auth/webauthn/authenticate/finish | New (WebAuthn) |
| N/A | GET /api/v1/auth/webauthn/credentials | New (WebAuthn) |
| N/A | DELETE /api/v1/auth/webauthn/credentials/:id | New (WebAuthn) |

**New (authenc-iam-api) - Admin Endpoints**:

| Endpoint | Purpose |
|----------|---------|
| GET /api/v1/iam/users | List users |
| POST /api/v1/iam/users | Create user |
| PUT /api/v1/iam/users/:id | Update user |
| DELETE /api/v1/iam/users/:id | Delete user |
| GET /api/v1/iam/realms | List realms |
| POST /api/v1/iam/realms | Create realm |
| GET /api/v1/iam/clients | List OAuth2 clients |
| POST /api/v1/iam/clients | Create OAuth2 client |
| GET /api/v1/iam/audit-logs | List audit logs |

### D. Environment Variables

**Required**:
```bash
# Database
DATABASE_URL=postgres://authenc:password@localhost:5432/authenc
DATABASE_POOL_SIZE=20

# JWT
JWT_ISSUER=https://authenc.kejaksaan.go.id
JWT_ACCESS_TOKEN_TTL=900  # 15 minutes
JWT_REFRESH_TOKEN_TTL=604800  # 7 days

# WebAuthn (MANDATORY)
WEBAUTHN_RP_ID=simpel.kejaksaan.go.id
WEBAUTHN_RP_ORIGIN=https://portal.simpel.kejaksaan.go.id
WEBAUTHN_RP_NAME="SIMPelv2 IAM"

# Secreton
SECRETON_GRPC_URL=https://secreton.internal:50052

# Server
AUTHENC_API_HOST=0.0.0.0
AUTHENC_API_PORT=8080
AUTHENC_IAM_API_PORT=8081
AUTHENC_GRPC_PORT=50051
```

**Optional**:
```bash
# Redis (caching)
REDIS_URL=redis://localhost:6379

# MFA
MFA_TOTP_ENABLED=true
MFA_ISSUER="Kejaksaan RI"

# Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_REQUESTS_PER_MINUTE=60

# Feature Flags
FEATURE_WEBAUTHN_ENABLED=true
FEATURE_FAPI_ENABLED=false

# Logging
RUST_LOG=info
LOG_FORMAT=json
```

### E. Monitoring Metrics

**Prometheus Metrics**:

```
# Authentication metrics
authenc_auth_requests_total{method="password|passkey|oauth2"}
authenc_auth_success_total{method="password|passkey|oauth2"}
authenc_auth_failure_total{method="password|passkey|oauth2",reason="invalid_credentials|mfa_required|account_locked"}
authenc_auth_duration_seconds{method="password|passkey|oauth2"}

# WebAuthn metrics
authenc_webauthn_registration_total{status="success|failure"}
authenc_webauthn_authentication_total{status="success|failure"}
authenc_webauthn_replay_attack_detected_total

# Token metrics
authenc_token_generated_total{type="access|refresh"}
authenc_token_validated_total{status="valid|expired|invalid"}
authenc_token_revoked_total

# Database metrics
authenc_db_connections_active
authenc_db_connections_idle
authenc_db_query_duration_seconds{operation="select|insert|update|delete"}

# API metrics
authenc_api_requests_total{endpoint="/api/v1/auth/login",method="POST",status="200"}
authenc_api_duration_seconds{endpoint="/api/v1/auth/login",method="POST"}
```

**Grafana Dashboards**:
- Authentication Overview
- WebAuthn Usage
- API Performance
- Database Performance
- Error Rates

### F. References

**External Documentation**:
- [WebAuthn Level 3 Specification](https://www.w3.org/TR/webauthn-3/)
- [OAuth 2.1 Authorization Framework](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-07)
- [FAPI 1.0 Advanced Profile](https://openid.net/specs/openid-financial-api-part-2-1_0.html)
- [FAPI 2.0 Security Profile](https://openid.net/specs/fapi-2_0-security-profile.html)
- [webauthn-rs Documentation](https://docs.rs/webauthn-rs/)

**Internal Documentation**:
- [Design Document](infra/authenc/.kiro/specs/authenc-portal-comprehensive-refactoring/design.md)
- [Requirements Document](infra/authenc/.kiro/specs/authenc-portal-comprehensive-refactoring/requirements.md)
- [Tasks Document](infra/authenc/.kiro/specs/authenc-portal-comprehensive-refactoring/tasks.md)
- [AGENTS.md - Authenc](infra/authenc/AGENTS.md)
- [AGENTS.md - Root](AGENTS.md)

---

**Document Version**: 1.0
**Last Updated**: 2026-02-19
**Authors**: SIMPelv2 Architecture Team
**Status**: Complete
**Maintained By**: DevOps Team

**For Questions or Issues**:
- Slack: #simpelv2-authenc-migration
- Email: devops@kejaksaan.go.id
- GitHub Issues: simpelv2/issues
