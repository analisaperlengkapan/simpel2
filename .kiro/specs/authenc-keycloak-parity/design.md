# Authenc Enterprise Features - Design Document

## 1. Overview

### 1.1 Purpose
This design document specifies the architecture and implementation details for achieving full enterprise readiness of Authenc, SIMPEL's Identity and Access Management (IAM) service. The design addresses critical gaps identified in the requirements document, focusing on production management capabilities, user self-service, compliance features, and government-specific integrations for Indonesian government deployment.

**Document Status**: REFRESHED - Updated to ensure complete alignment with all 31 functional requirements (FR-1 through FR-31), 66 non-functional requirements (NFR-1 through NFR-66), and 56 correctness properties.

### 1.2 Scope
This design covers the following enterprise features:

**Critical Priority (Phase 1 - Q1 2026)**:
- Admin Console UI (Leptos 0.8.x WASM-based web interface) - **Addresses FR-1 through FR-6**
- User Account Console (self-service portal) - **Addresses FR-7 through FR-12**
- Enhanced Event System (separate admin/user/system events) - **Addresses FR-13 through FR-15**
- Required Actions framework - **Addresses FR-17**

**High Priority (Phase 2 - Q2 2026)**:
- Authentication Flows customization - **Addresses FR-16**
- Composite Roles system - **Addresses FR-18, FR-19**
- User Storage Federation (MySIMKARI integration) - **Addresses FR-20, FR-21**
- User Self-Registration with NIP validation - **Addresses FR-24, FR-25**
- Email Template system - **Addresses FR-26**

**Medium Priority (Phase 3 - Q3 2026)**:
- Session Management enhancements - **Addresses FR-22, FR-23**
- Import/Export capabilities - **Addresses FR-27, FR-28**
- Client Scopes & Protocol Mappers enhancement

**Government-Specific Features (All Phases)**:
- NIP (Nomor Induk Pegawai) validation - **Addresses FR-29**
- Satker (Satuan Kerja) hierarchy enforcement - **Addresses FR-30**
- Audit reports - **Addresses FR-31**
- MySIMKARI integration - **Addresses FR-21**
- Indonesian language support - **Addresses NFR-25 through NFR-29**

### 1.3 Goals
1. **Production Readiness**: Enable full management of Authenc without manual database operations
2. **User Self-Service**: Reduce administrative burden through user account console
3. **Compliance**: Meet Indonesian government requirements (audit compliance, ISO 27001, SPBE)
4. **Integration**: Seamless integration with MySIMKARI, SIMAN, MonSAKTI
5. **Flexibility**: Support customizable authentication flows for different security contexts
6. **Performance**: Maintain Authenc's performance advantages (Rust vs Java, <100ms p95) - **Addresses NFR-1 through NFR-7**
7. **Security**: Maintain superior security features (Ed25519, Post-Quantum Crypto) - **Addresses NFR-8 through NFR-18**

### 1.4 Non-Goals
- Kerberos authentication (low priority, specialized use case)
- X.509 client certificates (alternative methods available)
- JavaScript/Drools policy engine (current policy engine sufficient)
- CIBA (Client Initiated Backchannel Authentication) - specialized use case
- Migration from Keycloak (Authenc is primary IAM)

### 1.5 Success Criteria
- Admin Console loads in <2 seconds - **NFR-1**
- API response time <100ms at p95 - **NFR-2**
- Database queries <10ms at p95 - **NFR-3**
- Support 1000+ concurrent users - **NFR-4**
- Memory usage <500MB per instance - **NFR-5**
- 99.9% uptime - **NFR-47**
- Audit compliance achieved - **FR-31**
- Zero critical security vulnerabilities - **NFR-8 through NFR-18**
- Test coverage >80% - **NFR-37, NFR-38**
- WCAG 2.1 Level AA compliance - **NFR-19 through NFR-24**

## 2. Architecture

### 2.0 Requirements Coverage Summary

This design addresses all functional and non-functional requirements as follows:

**Functional Requirements Coverage**:
- **FR-1 to FR-6** (Admin Console): Section 3.1 - Admin Console UI
- **FR-7 to FR-12** (User Account Console): Section 3.2 - User Account Console
- **FR-13 to FR-15** (Event System): Section 3.3.1 - Event Service
- **FR-16** (Authentication Flows): Section 3.3.2 - Authentication Flow Service
- **FR-17** (Required Actions): Section 3.3.2 - Authentication Flow Service
- **FR-18, FR-19** (Role Management): Section 4.1.1 - Composite Role Model
- **FR-20, FR-21** (User Storage Federation): Section 4.1.2 - User Storage Provider Model
- **FR-22, FR-23** (Session Management): Section 3.3.2 - Session Service
- **FR-24, FR-25** (User Registration): Section 4.1.3 - Registration Configuration Model
- **FR-26** (Email Templates): Section 3.3.2 - Email Template Service
- **FR-27, FR-28** (Import/Export): Section 3.3.2 - Import/Export Service
- **FR-29** (NIP Validation): Section 6.1 - NIP Validation Service
- **FR-30** (Satker Hierarchy): Section 6.2 - Satker Hierarchy Service
- **FR-31** (Audit Reports): Section 6.3 - Audit Report Service

**Non-Functional Requirements Coverage**:
- **NFR-1 to NFR-7** (Performance): Section 10 - Performance Optimization
- **NFR-8 to NFR-18** (Security): Section 9 - Security Considerations
- **NFR-19 to NFR-24** (Accessibility): Section 3.1, 3.2 - UI Components
- **NFR-25 to NFR-29** (Internationalization): Section 3.1, 3.2 - UI Components
- **NFR-30 to NFR-34** (Browser Compatibility): Section 2.2.1 - Frontend Layer
- **NFR-35 to NFR-41** (Maintainability): Section 7 - Testing Strategy
- **NFR-42 to NFR-46** (Scalability): Section 2.4 - Deployment Architecture
- **NFR-47 to NFR-52** (Reliability): Section 6.2 - Error Handling Patterns
- **NFR-53 to NFR-58** (Observability): Section 11 - Monitoring and Observability
- **NFR-59 to NFR-66** (Deployment): Section 12 - Deployment Strategy

**Correctness Properties**: All 56 properties defined in Section 5 with full traceability to requirements.

### 2.1 System Architecture

Following SIMPEL architecture patterns, Authenc uses a clear separation between frontend (WASM microfrontends), backend (REST API), and infrastructure services (gRPC):

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Browser (WASM CSR)                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌──────────────────────┐  ┌──────────────────────┐  ┌──────────────┐  │
│  │   Admin Console      │  │  User Account        │  │  Auth Pages  │  │
│  │   (Leptos 0.8.x)     │  │  Console             │  │  (Login/Reg) │  │
│  │   /admin/*           │  │  (Leptos 0.8.x)      │  │  /auth/*     │  │
│  │                      │  │  /account/*          │  │              │  │
│  └──────────┬───────────┘  └──────────┬───────────┘  └──────┬───────┘  │
│             │                          │                     │          │
│             └──────────────────────────┼─────────────────────┘          │
│                                        │ REST API (JSON/HTTP)           │
└────────────────────────────────────────┼────────────────────────────────┘
                                │
┌───────────────────────────────┼─────────────────────────────────────────┐
│                    Authenc Backend (Axum 0.8.x)                          │
├───────────────────────────────┼─────────────────────────────────────────┤
│                               │                                          │
│  ┌────────────────────────────▼──────────────────────────────────────┐  │
│  │                      REST API Layer                               │  │
│  │  /api/admin/*  - Admin operations (realm, user, client mgmt)     │  │
│  │  /api/account/* - User self-service (profile, MFA, sessions)     │  │
│  │  /api/auth/*   - Authentication endpoints (login, logout, etc.)  │  │
│  └────────────────────────────┬──────────────────────────────────────┘  │
│                               │                                          │
│  ┌────────────────────────────▼──────────────────────────────────────┐  │
│  │                    Service Layer                                  │  │
│  │  • RealmService       • UserService        • ClientService       │  │
│  │  • RoleService        • EventService       • SessionService      │  │
│  │  • FlowService        • ActionService      • TemplateService     │  │
│  │  • FederationService  • RegistrationService • SatkerService      │  │
│  └────────────────────────────┬──────────────────────────────────────┘  │
│                               │                                          │
│  ┌────────────────────────────▼──────────────────────────────────────┐  │
│  │                    Data Access Layer (Stores)                     │  │
│  │  • RealmStore         • UserStore          • ClientStore         │  │
│  │  • RoleStore          • EventStore         • SessionStore        │  │
│  │  • FlowStore          • ActionStore        • TemplateStore       │  │
│  │  • SatkerStore        • NipValidationCache                       │  │
│  └────────────────────────────┬──────────────────────────────────────┘  │
│                               │                                          │
└───────────────────────────────┼──────────────────────────────────────────┘
                                │
         ┌──────────────────────┼──────────────────────┐
         │                      │                      │
    ┌────▼─────┐         ┌─────▼──────┐       ┌──────▼──────┐
    │PostgreSQL│         │   Redis    │       │  Secreton   │
    │ (Primary)│         │  (Cache &  │       │   (gRPC)    │
    │          │         │  Sessions) │       │             │
    └──────────┘         └────────────┘       └──────┬──────┘
         │                                            │
         │                                            │
    ┌────▼─────┐                              ┌──────▼──────┐
    │  Kafka   │                              │ MySIMKARI   │
    │(Optional)│                              │   (gRPC)    │
    │ Events   │                              │  User Data  │
    └──────────┘                              └─────────────┘
```

### 2.2 Component Architecture

#### 2.2.1 Frontend Layer (Leptos 0.8.x WASM CSR)

**Technology Stack**:
- **Framework**: Leptos 0.8.x (Client-Side Rendering)
- **Build Tool**: Trunk
- **Styling**: Tailwind CSS (via lib-ui)
- **Icons**: Heroicons
- **HTTP Client**: gloo-net
- **State Management**: Leptos `signal()` and `RwSignal`
- **Routing**: Leptos Router
- **Shared Components**: lib-ui (from SIMPEL workspace)

**UI Folder Structure**:
```
layanan/authenc/src/ui/
├── mod.rs                  # UI module exports
├── admin/                  # Admin Console (/admin/*)
│   ├── mod.rs
│   ├── dashboard.rs        # System dashboard
│   ├── realms.rs           # Realm management
│   ├── users.rs            # User management
│   ├── clients.rs          # Client management
│   ├── roles.rs            # Role management
│   ├── flows.rs            # Authentication flows
│   ├── events.rs           # Audit log viewer
│   └── ...
├── account/                # User Account Console (/account/*)
│   ├── mod.rs
│   ├── profile.rs          # Profile management
│   ├── password.rs         # Password change
│   ├── mfa.rs              # MFA device management
│   ├── sessions.rs         # Session management
│   ├── activity.rs         # Activity log
│   └── ...
├── auth/                   # Authentication Pages (/auth/*)
│   ├── mod.rs
│   ├── login.rs            # Login page
│   ├── register.rs         # Registration page
│   ├── forgot.rs           # Forgot password
│   ├── mfa_challenge.rs    # MFA challenge pages
│   ├── verify_email.rs     # Email verification
│   └── ...
└── components/             # Shared UI components
    ├── mod.rs
    ├── layout.rs           # Base layout
    ├── forms.rs            # Form components
    ├── tables.rs           # Table components
    ├── modals.rs           # Modal dialogs
    └── ...
```

**Design Rationale**:
- **Separation of Concerns**: Each UI module (admin, account, auth) has distinct access controls and user audiences
- **Reusability**: Shared components in `components/` reduce code duplication
- **Maintainability**: Clear folder structure makes it easy to locate and modify specific features
- **Scalability**: New features can be added to appropriate modules without affecting others
- **Security**: Different authentication requirements for admin vs account vs auth pages

**Key Patterns**:
```rust
// ✅ Leptos 0.8.x signal creation (NOT create_signal!)
let (count, set_count) = signal(0);

// ✅ Resource for async data fetching
let users = Resource::new(
    || (),
    |_| async move {
        gloo_net::http::Request::get("/api/admin/users")
            .send()
            .await?
            .json::<Vec<User>>()
            .await
    }
);

// ✅ Action for mutations
let save_user = Action::new(move |user: &User| {
    let user = user.clone();
    async move {
        gloo_net::http::Request::post("/api/admin/users")
            .json(&user)?
            .send()
            .await
    }
});
```

**Communication Rules** (CRITICAL):
- ✅ Microfrontends call Authenc REST API (JSON/HTTP)
- ⛔ Microfrontends NEVER call Secreton or MySIMKARI directly
- ⛔ Microfrontends NEVER call gRPC services
- ✅ All external service calls go through Authenc backend

#### 2.2.2 Backend Layer (Axum 0.8.x + Tonic 0.14.x)

**Technology Stack**:
- **HTTP Framework**: Axum 0.8.x
- **gRPC Framework**: Tonic 0.14.x + Prost 0.14.x
- **Database**: tokio-postgres + deadpool-postgres
- **Cache**: redis-rs
- **Async Runtime**: Tokio
- **Serialization**: serde + serde_json
- **Validation**: validator
- **Logging**: tracing + tracing-subscriber
- **Metrics**: prometheus + opentelemetry

**Key Patterns**:
```rust
// ✅ Axum 0.8.x router with State
pub fn create_admin_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/admin/users", get(list_users).post(create_user))
        .route("/api/admin/users/{id}", get(get_user).put(update_user).delete(delete_user))
        .layer(jwt_validation_layer())
        .layer(rate_limiting_layer())
        .with_state(state)
}

// ✅ Handler with extractors
async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service.get_user(id).await?;
    Ok(Json(user))
}

// ✅ gRPC client usage (backend to Secreton)
let jwt_key = state.secreton_client
    .get_secret(GetSecretRequest {
        path: "authenc/jwt-signing-key".to_string(),
        version: None,
    })
    .await?
    .into_inner()
    .data;
```

**Service Layer Architecture**:
- Services contain business logic
- Services orchestrate multiple stores
- Services handle external integrations (gRPC to Secreton, MySIMKARI)
- Services enforce authorization rules
- Services emit events for audit logging

**Data Access Layer (Stores)**:
- Stores handle database operations
- Stores use connection pooling (deadpool-postgres)
- Stores use prepared statements for performance
- Stores handle caching (Redis)
- Stores are transaction-aware

#### 2.2.3 Data Layer

**PostgreSQL** (Primary Data Store):
- Users, roles, clients, realms
- Authentication flows, required actions
- Admin events, user events
- Sessions (with Redis fallback)
- Satker hierarchy
- Email templates

**Redis** (Cache & Sessions):
- Active sessions (primary)
- NIP validation cache (5 minutes TTL)
- User attribute cache
- Rate limiting counters
- Distributed locks

**Kafka** (Optional - Event Streaming):
- Admin event stream
- User event stream
- Security event stream
- Integration with SIEM systems

**Elasticsearch** (Optional - Audit Search):
- Full-text search on audit logs
- Compliance reporting
- Security analytics

### 2.3 Integration Architecture

#### 2.3.1 Secreton Integration (gRPC)
```rust
// Authenc backend calls Secreton for:
// - JWT signing keys
// - Client secrets
// - Encryption keys
// - Sensitive configuration

let secreton_client = SecretonGrpcClient::new(
    Channel::from_static("https://secreton.internal:50052")
        .tls_config(ClientTlsConfig::new())?
        .connect()
        .await?
);
```

#### 2.3.2 MySIMKARI Integration (gRPC)
```rust
// Authenc backend calls MySIMKARI for:
// - NIP validation
// - Employee data retrieval
// - Satker information
// - Organizational hierarchy

let mysimkari_client = IntegrasiClient::new(
    Channel::from_static("https://integrasi.internal:50053")
        .tls_config(ClientTlsConfig::new())?
        .connect()
        .await?
);
```

### 2.4 Deployment Architecture

**Kubernetes Deployment** (MicroK8s with Istio):
```yaml
# Namespace: simpelv2-production
# Replicas: 3 (HA)
# Resources: 500m CPU, 512Mi memory per pod
# Storage: Longhorn PVC for PostgreSQL
# LoadBalancer: MetalLB (172.15.10.200-230)
# Service Mesh: Istio (mTLS STRICT mode)
```

**Service Communication**:
- Frontend → Authenc REST API: HTTPS (Istio mTLS)
- Authenc → Secreton: gRPC (mTLS)
- Authenc → MySIMKARI: gRPC (mTLS)
- Authenc → PostgreSQL: TCP (internal network)
- Authenc → Redis: TCP (internal network)

## 3. Components and Interfaces

### 3.1 Admin Console UI

#### 3.1.1 Technology Stack
- **Framework**: Leptos 0.8.x (WASM CSR)
- **Styling**: Tailwind CSS
- **Icons**: Heroicons
- **Charts**: Chart.js (via WASM bindings)
- **State Management**: Leptos RwSignal
- **Routing**: Leptos Router
- **HTTP Client**: gloo-net

#### 3.1.2 Page Structure
```
/admin
├── /dashboard              # System overview and metrics
├── /realms
│   ├── /list              # List all realms
│   ├── /create            # Create new realm
│   └── /{realm_id}
│       ├── /settings      # Realm configuration
│       ├── /users         # User management
│       ├── /clients       # Client management
│       ├── /roles         # Role management
│       ├── /groups        # Group management
│       ├── /authentication # Authentication flows
│       ├── /identity-providers # Federation config
│       ├── /events        # Event logs
│       └── /sessions      # Active sessions
├── /users
│   ├── /list              # All users across realms
│   ├── /create            # Create user
│   └── /{user_id}
│       ├── /profile       # User details
│       ├── /credentials   # Password, MFA
│       ├── /role-mappings # Assigned roles
│       ├── /sessions      # User sessions
│       └── /events        # User activity log
├── /clients
│   ├── /list              # All clients
│   ├── /create            # Register client
│   └── /{client_id}
│       ├── /settings      # Client configuration
│       ├── /credentials   # Client secrets
│       ├── /roles         # Client roles
│       ├── /scopes        # Client scopes
│       └── /mappers       # Protocol mappers
├── /roles
│   ├── /realm-roles       # Realm-level roles
│   └── /client-roles      # Client-specific roles
├── /authentication
│   ├── /flows             # Authentication flows
│   ├── /required-actions  # Required actions config
│   └── /policies          # Authentication policies
├── /identity-providers
│   ├── /list              # External IdPs
│   └── /create            # Add IdP
├── /user-federation
│   ├── /ldap              # LDAP configuration
│   └── /custom            # Custom providers
├── /events
│   ├── /admin-events      # Admin operations
│   ├── /user-events       # User activities
│   └── /config            # Event configuration
├── /sessions
│   └── /active            # Active sessions
├── /import-export
│   ├── /export            # Export realm
│   └── /import            # Import realm
└── /settings
    ├── /email             # Email configuration
    ├── /themes            # Theme customization
    └── /security          # Security settings
```

#### 3.1.3 Key Components

**Dashboard Component**
```rust
#[component]
pub fn AdminDashboard() -> impl IntoView {
    let metrics = Resource::new(
        || (),
        |_| async move {
            gloo_net::http::Request::get("/api/admin/metrics")
                .send()
                .await?
                .json::<SystemMetrics>()
                .await
        }
    );

    view! {
        <div class="dashboard">
            <h1>"System Dashboard"</h1>
            <Suspense fallback=move || view! { <Loading /> }>
                {move || metrics.get().map(|result| match result {
                    Ok(metrics) => view! {
                        <MetricsGrid metrics=metrics />
                        <ActiveUsersChart data=metrics.active_users />
                        <RecentEvents events=metrics.recent_events />
                    }.into_any(),
                    Err(e) => view! { <ErrorDisplay error=e.to_string() /> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
```

**User Management Component**
```rust
#[component]
pub fn UserManagement() -> impl IntoView {
    let (search_query, set_search_query) = signal(String::new());
    let (selected_realm, set_selected_realm) = signal(None::<Uuid>);

    let users = Resource::new(
        move || (search_query.get(), selected_realm.get()),
        |(query, realm_id)| async move {
            let mut url = "/api/admin/users?".to_string();
            if !query.is_empty() {
                url.push_str(&format!("q={}&", query));
            }
            if let Some(realm) = realm_id {
                url.push_str(&format!("realm_id={}", realm));
            }

            gloo_net::http::Request::get(&url)
                .send()
                .await?
                .json::<Vec<User>>()
                .await
        }
    );

    view! {
        <div class="user-management">
            <div class="toolbar">
                <SearchInput value=search_query on_change=set_search_query />
                <RealmSelector value=selected_realm on_change=set_selected_realm />
                <Button on:click=move |_| navigate("/admin/users/create")>
                    "Create User"
                </Button>
            </div>

            <Suspense fallback=move || view! { <Loading /> }>
                {move || users.get().map(|result| match result {
                    Ok(users) => view! {
                        <UserTable users=users />
                    }.into_any(),
                    Err(e) => view! { <ErrorDisplay error=e.to_string() /> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
```

### 3.2 User Account Console

#### 3.2.1 Page Structure
```
/account
├── /                       # Profile overview
├── /profile                # Edit profile
├── /password               # Change password
├── /mfa                    # MFA device management
│   ├── /totp              # TOTP setup
│   ├── /webauthn          # WebAuthn setup
│   └── /backup-codes      # Backup codes
├── /sessions               # Active sessions
├── /activity               # Activity log
├── /applications           # Authorized applications
├── /consents               # Consent management
├── /linked-accounts        # Social login accounts
└── /data
    ├── /export            # Export personal data
    └── /delete            # Request account deletion
```

#### 3.2.2 Key Components

**Profile Management**
```rust
#[component]
pub fn ProfileManagement() -> impl IntoView {
    let profile = Resource::new(
        || (),
        |_| async move {
            gloo_net::http::Request::get("/api/account/profile")
                .send()
                .await?
                .json::<UserProfile>()
                .await
        }
    );

    let (editing, set_editing) = signal(false);
    let (form_data, set_form_data) = signal(UserProfileForm::default());

    let save_profile = Action::new(move |form: &UserProfileForm| {
        let form = form.clone();
        async move {
            gloo_net::http::Request::put("/api/account/profile")
                .json(&form)?
                .send()
                .await?
                .json::<UserProfile>()
                .await
        }
    });

    view! {
        <div class="profile-management">
            <h1>"My Profile"</h1>
            <Suspense fallback=move || view! { <Loading /> }>
                {move || profile.get().map(|result| match result {
                    Ok(profile) => view! {
                        <Show when=move || !editing.get()>
                            <ProfileDisplay profile=profile.clone() />
                            <Button on:click=move |_| set_editing.set(true)>
                                "Edit Profile"
                            </Button>
                        </Show>
                        <Show when=move || editing.get()>
                            <ProfileForm
                                initial_data=profile
                                on_save=move |data| {
                                    save_profile.dispatch(data);
                                    set_editing.set(false);
                                }
                                on_cancel=move |_| set_editing.set(false)
                            />
                        </Show>
                    }.into_any(),
                    Err(e) => view! { <ErrorDisplay error=e.to_string() /> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
```

**MFA Device Management**
```rust
#[component]
pub fn MfaDeviceManagement() -> impl IntoView {
    let devices = Resource::new(
        || (),
        |_| async move {
            gloo_net::http::Request::get("/api/account/mfa/devices")
                .send()
                .await?
                .json::<Vec<MfaDevice>>()
                .await
        }
    );

    let (show_add_dialog, set_show_add_dialog) = signal(false);
    let (device_type, set_device_type) = signal(MfaDeviceType::Totp);

    view! {
        <div class="mfa-management">
            <h1>"Multi-Factor Authentication"</h1>

            <div class="device-list">
                <Suspense fallback=move || view! { <Loading /> }>
                    {move || devices.get().map(|result| match result {
                        Ok(devices) => view! {
                            <For
                                each=move || devices.clone()
                                key=|device| device.id
                                children=|device| view! {
                                    <MfaDeviceCard device=device />
                                }
                            />
                        }.into_any(),
                        Err(e) => view! { <ErrorDisplay error=e.to_string() /> }.into_any(),
                    })}
                </Suspense>
            </div>

            <Button on:click=move |_| set_show_add_dialog.set(true)>
                "Add Device"
            </Button>

            <Show when=move || show_add_dialog.get()>
                <AddMfaDeviceDialog
                    device_type=device_type
                    on_close=move |_| set_show_add_dialog.set(false)
                />
            </Show>
        </div>
    }
}
```

### 3.3 Backend Services

#### 3.3.1 Event Service

**Event Storage Schema**
```sql
-- Admin Events Table
CREATE TABLE admin_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    realm_id UUID NOT NULL REFERENCES realms(id),
    operation_type VARCHAR(50) NOT NULL, -- CREATE, UPDATE, DELETE, ACTION
    resource_type VARCHAR(100) NOT NULL, -- USER, ROLE, CLIENT, REALM, etc.
    resource_path TEXT NOT NULL,         -- /realms/master/users/123
    representation JSONB,                -- Full JSON of changed object
    admin_user_id UUID NOT NULL REFERENCES users(id),
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    INDEX idx_admin_events_realm (realm_id),
    INDEX idx_admin_events_admin (admin_user_id),
    INDEX idx_admin_events_created (created_at),
    INDEX idx_admin_events_resource (resource_type, resource_path)
);

-- User Events Table
CREATE TABLE user_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    realm_id UUID NOT NULL REFERENCES realms(id),
    event_type VARCHAR(100) NOT NULL,   -- LOGIN, LOGOUT, REGISTER, etc.
    user_id UUID REFERENCES users(id),  -- NULL for failed login attempts
    client_id UUID REFERENCES clients(id),
    ip_address INET,
    error TEXT,                          -- Error message if failed
    details JSONB,                       -- Additional event details
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    INDEX idx_user_events_realm (realm_id),
    INDEX idx_user_events_user (user_id),
    INDEX idx_user_events_type (event_type),
    INDEX idx_user_events_created (created_at)
);

-- Event Configuration Table
CREATE TABLE event_config (
    realm_id UUID PRIMARY KEY REFERENCES realms(id),
    admin_events_enabled BOOLEAN NOT NULL DEFAULT true,
    user_events_enabled BOOLEAN NOT NULL DEFAULT true,
    admin_events_retention_days INTEGER NOT NULL DEFAULT 365,
    user_events_retention_days INTEGER NOT NULL DEFAULT 90,
    export_to_kafka BOOLEAN NOT NULL DEFAULT false,
    export_to_elasticsearch BOOLEAN NOT NULL DEFAULT false,
    kafka_topic VARCHAR(255),
    elasticsearch_index VARCHAR(255),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Event Service Implementation**
```rust
pub struct EventService {
    admin_event_store: Arc<AdminEventStore>,
    user_event_store: Arc<UserEventStore>,
    kafka_producer: Option<Arc<KafkaProducer>>,
    elasticsearch_client: Option<Arc<ElasticsearchClient>>,
}

impl EventService {
    pub async fn log_admin_event(&self, event: AdminEvent) -> Result<()> {
        // 1. Store in database
        self.admin_event_store.create(event.clone()).await?;

        // 2. Export to Kafka if enabled
        if let Some(producer) = &self.kafka_producer {
            let config = self.get_event_config(event.realm_id).await?;
            if config.export_to_kafka {
                producer.send_admin_event(&event, &config.kafka_topic).await?;
            }
        }

        // 3. Export to Elasticsearch if enabled
        if let Some(es_client) = &self.elasticsearch_client {
            let config = self.get_event_config(event.realm_id).await?;
            if config.export_to_elasticsearch {
                es_client.index_admin_event(&event, &config.elasticsearch_index).await?;
            }
        }

        Ok(())
    }

    pub async fn log_user_event(&self, event: UserEvent) -> Result<()> {
        // Similar to admin event logging
        self.user_event_store.create(event.clone()).await?;

        if let Some(producer) = &self.kafka_producer {
            let config = self.get_event_config(event.realm_id).await?;
            if config.export_to_kafka {
                producer.send_user_event(&event, &config.kafka_topic).await?;
            }
        }

        Ok(())
    }

    pub async fn cleanup_expired_events(&self) -> Result<CleanupStats> {
        let mut stats = CleanupStats::default();

        // Get all realm configurations
        let configs = self.get_all_event_configs().await?;

        for config in configs {
            // Cleanup admin events
            let admin_cutoff = Utc::now() - Duration::days(config.admin_events_retention_days as i64);
            let admin_deleted = self.admin_event_store
                .delete_before(config.realm_id, admin_cutoff)
                .await?;
            stats.admin_events_deleted += admin_deleted;

            // Cleanup user events
            let user_cutoff = Utc::now() - Duration::days(config.user_events_retention_days as i64);
            let user_deleted = self.user_event_store
                .delete_before(config.realm_id, user_cutoff)
                .await?;
            stats.user_events_deleted += user_deleted;
        }

        Ok(stats)
    }

    pub async fn search_admin_events(
        &self,
        filter: AdminEventFilter,
    ) -> Result<PaginatedResult<AdminEvent>> {
        self.admin_event_store.search(filter).await
    }

    pub async fn search_user_events(
        &self,
        filter: UserEventFilter,
    ) -> Result<PaginatedResult<UserEvent>> {
        self.user_event_store.search(filter).await
    }
}

#[derive(Debug, Clone)]
pub struct AdminEvent {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub operation_type: AdminOperationType,
    pub resource_type: String,
    pub resource_path: String,
    pub representation: Option<serde_json::Value>,
    pub admin_user_id: Uuid,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdminOperationType {
    Create,
    Update,
    Delete,
    Action,
}

#[derive(Debug, Clone)]
pub struct UserEvent {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub event_type: UserEventType,
    pub user_id: Option<Uuid>,
    pub client_id: Option<Uuid>,
    pub ip_address: Option<IpAddr>,
    pub error: Option<String>,
    pub details: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEventType {
    Login,
    LoginError,
    Logout,
    Register,
    RegisterError,
    UpdateProfile,
    UpdatePassword,
    UpdateEmail,
    VerifyEmail,
    ResetPassword,
    ResetPasswordError,
    MfaEnabled,
    MfaDisabled,
    MfaVerified,
    MfaVerifyError,
    ConsentGranted,
    ConsentRevoked,
    AccountDeleted,
}
```

#### 3.3.2 Authentication Flow Service

**Authentication Flow Schema**
```sql
-- Authentication Flows Table
CREATE TABLE authentication_flows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    realm_id UUID NOT NULL REFERENCES realms(id),
    alias VARCHAR(255) NOT NULL,
    description TEXT,
    provider_id VARCHAR(100) NOT NULL, -- "basic-flow", "client-flow"
    top_level BOOLEAN NOT NULL DEFAULT false,
    built_in BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(realm_id, alias)
);

-- Authentication Executions Table
CREATE TABLE authentication_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    flow_id UUID NOT NULL REFERENCES authentication_flows(id) ON DELETE CASCADE,
    authenticator VARCHAR(255) NOT NULL, -- "username-password-form", "otp-form"
    requirement VARCHAR(50) NOT NULL,    -- REQUIRED, ALTERNATIVE, DISABLED, CONDITIONAL
    priority INTEGER NOT NULL,
    authenticator_flow BOOLEAN NOT NULL DEFAULT false,
    flow_alias VARCHAR(255),
    config JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    INDEX idx_auth_exec_flow (flow_id),
    INDEX idx_auth_exec_priority (flow_id, priority)
);

-- Required Actions Table
CREATE TABLE required_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    realm_id UUID NOT NULL REFERENCES realms(id),
    alias VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    provider_id VARCHAR(255) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    default_action BOOLEAN NOT NULL DEFAULT false,
    priority INTEGER NOT NULL,
    config JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(realm_id, alias)
);

-- User Required Actions Table
CREATE TABLE user_required_actions (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    required_action VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, required_action)
);
```

**Authentication Flow Service Implementation**
```rust
pub struct AuthenticationFlowService {
    flow_store: Arc<AuthenticationFlowStore>,
    execution_store: Arc<AuthenticationExecutionStore>,
    action_store: Arc<RequiredActionStore>,
}

impl AuthenticationFlowService {
    pub async fn create_flow(
        &self,
        realm_id: Uuid,
        request: CreateFlowRequest,
    ) -> Result<AuthenticationFlow> {
        // Validate flow doesn't exist
        if self.flow_store.get_by_alias(realm_id, &request.alias).await?.is_some() {
            return Err(AuthencError::FlowAlreadyExists);
        }

        let flow = AuthenticationFlow {
            id: Uuid::new_v4(),
            realm_id,
            alias: request.alias,
            description: request.description,
            provider_id: request.provider_id,
            top_level: request.top_level,
            built_in: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.flow_store.create(flow.clone()).await?;
        Ok(flow)
    }

    pub async fn add_execution(
        &self,
        flow_id: Uuid,
        request: AddExecutionRequest,
    ) -> Result<AuthenticationExecution> {
        // Get flow
        let flow = self.flow_store.get(flow_id).await?
            .ok_or(AuthencError::FlowNotFound)?;

        // Check if built-in flow
        if flow.built_in {
            return Err(AuthencError::CannotModifyBuiltInFlow);
        }

        // Get next priority
        let executions = self.execution_store.get_by_flow(flow_id).await?;
        let next_priority = executions.iter()
            .map(|e| e.priority)
            .max()
            .unwrap_or(0) + 1;

        let execution = AuthenticationExecution {
            id: Uuid::new_v4(),
            flow_id,
            authenticator: request.authenticator,
            requirement: request.requirement,
            priority: next_priority,
            authenticator_flow: request.authenticator_flow,
            flow_alias: request.flow_alias,
            config: request.config,
            created_at: Utc::now(),
        };

        self.execution_store.create(execution.clone()).await?;
        Ok(execution)
    }

    pub async fn reorder_executions(
        &self,
        flow_id: Uuid,
        execution_ids: Vec<Uuid>,
    ) -> Result<()> {
        // Get flow
        let flow = self.flow_store.get(flow_id).await?
            .ok_or(AuthencError::FlowNotFound)?;

        if flow.built_in {
            return Err(AuthencError::CannotModifyBuiltInFlow);
        }

        // Update priorities
        for (index, execution_id) in execution_ids.iter().enumerate() {
            self.execution_store
                .update_priority(*execution_id, index as i32)
                .await?;
        }

        Ok(())
    }

    pub async fn copy_flow(
        &self,
        flow_id: Uuid,
        new_alias: String,
    ) -> Result<AuthenticationFlow> {
        // Get source flow
        let source_flow = self.flow_store.get(flow_id).await?
            .ok_or(AuthencError::FlowNotFound)?;

        // Create new flow
        let new_flow = self.create_flow(
            source_flow.realm_id,
            CreateFlowRequest {
                alias: new_alias,
                description: source_flow.description.clone(),
                provider_id: source_flow.provider_id.clone(),
                top_level: source_flow.top_level,
            },
        ).await?;

        // Copy executions
        let executions = self.execution_store.get_by_flow(flow_id).await?;
        for execution in executions {
            self.add_execution(
                new_flow.id,
                AddExecutionRequest {
                    authenticator: execution.authenticator,
                    requirement: execution.requirement,
                    authenticator_flow: execution.authenticator_flow,
                    flow_alias: execution.flow_alias,
                    config: execution.config,
                },
            ).await?;
        }

        Ok(new_flow)
    }

    pub async fn execute_flow(
        &self,
        flow_id: Uuid,
        context: &mut AuthenticationContext,
    ) -> Result<AuthenticationResult> {
        let executions = self.execution_store.get_by_flow(flow_id).await?;

        for execution in executions {
            match execution.requirement {
                ExecutionRequirement::Required => {
                    // Must execute and succeed
                    let result = self.execute_authenticator(&execution, context).await?;
                    if !result.success {
                        return Ok(AuthenticationResult::Failed(result.error));
                    }
                }
                ExecutionRequirement::Alternative => {
                    // One of alternatives must succeed
                    let result = self.execute_authenticator(&execution, context).await?;
                    if result.success {
                        break; // Alternative succeeded, skip remaining
                    }
                }
                ExecutionRequirement::Conditional => {
                    // Execute based on condition
                    if self.evaluate_condition(&execution, context).await? {
                        let result = self.execute_authenticator(&execution, context).await?;
                        if !result.success {
                            return Ok(AuthenticationResult::Failed(result.error));
                        }
                    }
                }
                ExecutionRequirement::Disabled => {
                    // Skip
                    continue;
                }
            }
        }

        Ok(AuthenticationResult::Success)
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticationFlow {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub alias: String,
    pub description: Option<String>,
    pub provider_id: String,
    pub top_level: bool,
    pub built_in: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AuthenticationExecution {
    pub id: Uuid,
    pub flow_id: Uuid,
    pub authenticator: String,
    pub requirement: ExecutionRequirement,
    pub priority: i32,
    pub authenticator_flow: bool,
    pub flow_alias: Option<String>,
    pub config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionRequirement {
    Required,
    Alternative,
    Disabled,
    Conditional,
}
```

## 4. Data Models

### 4.1 Core Entities

#### 4.1.1 Composite Role Model
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub role_type: RoleType,
    pub client_id: Option<Uuid>,
    pub realm_id: Uuid,
    pub composite: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoleType {
    RealmRole,
    ClientRole,
}

#[derive(Debug, Clone)]
pub struct CompositeRole {
    pub parent_role_id: Uuid,
    pub child_role_id: Uuid,
    pub realm_id: Uuid,
}

// Database schema
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    role_type VARCHAR(50) NOT NULL, -- 'realm' or 'client'
    client_id UUID REFERENCES clients(id),
    realm_id UUID NOT NULL REFERENCES realms(id),
    composite BOOLEAN NOT NULL DEFAULT false,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(realm_id, name, client_id)
);

CREATE TABLE composite_roles (
    parent_role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    child_role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    realm_id UUID NOT NULL REFERENCES realms(id),
    PRIMARY KEY (parent_role_id, child_role_id),
    CHECK (parent_role_id != child_role_id)
);

CREATE INDEX idx_composite_roles_parent ON composite_roles(parent_role_id);
CREATE INDEX idx_composite_roles_child ON composite_roles(child_role_id);
```

#### 4.1.2 User Storage Provider Model
```rust
#[async_trait]
pub trait UserStorageProvider: Send + Sync {
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn search_users(&self, query: &str, max_results: usize) -> Result<Vec<User>>;
    async fn validate_credentials(&self, username: &str, password: &str) -> Result<bool>;
    async fn update_user(&self, user: &User) -> Result<()>;
    async fn is_read_only(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct UserStorageProviderConfig {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub name: String,
    pub provider_type: String, // "ldap", "mysimkari", "custom"
    pub priority: i32,
    pub enabled: bool,
    pub config: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// MySIMKARI Provider Implementation
pub struct MysimkariUserStorageProvider {
    grpc_client: Arc<IntegrasiClient>,
    cache: Arc<RedisCache>,
    config: UserStorageProviderConfig,
}

impl UserStorageProvider for MysimkariUserStorageProvider {
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        // 1. Check cache
        let cache_key = format!("mysimkari:user:{}", username);
        if let Some(user) = self.cache.get::<User>(&cache_key).await? {
            return Ok(Some(user));
        }

        // 2. Fetch from MySIMKARI via gRPC
        let mysimkari_user = match self.grpc_client
            .get_pegawai_by_nip(username)
            .await
        {
            Ok(user) => user,
            Err(e) if e.is_not_found() => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        // 3. Map to Authenc User
        let user = User {
            id: Uuid::new_v4(),
            username: mysimkari_user.nip.clone(),
            email: mysimkari_user.email.clone(),
            email_verified: true,
            enabled: true,
            first_name: Some(mysimkari_user.nama.clone()),
            last_name: None,
            attributes: hashmap! {
                "nip".to_string() => mysimkari_user.nip,
                "satker_id".to_string() => mysimkari_user.satker_id,
                "satker_name".to_string() => mysimkari_user.satker_name,
                "jabatan".to_string() => mysimkari_user.jabatan,
                "golongan".to_string() => mysimkari_user.golongan,
            },
            federated: true,
            federation_link: Some("mysimkari".to_string()),
            ..Default::default()
        };

        // 4. Cache for 5 minutes
        self.cache.set(&cache_key, &user, Duration::from_secs(300)).await?;

        Ok(Some(user))
    }

    async fn is_read_only(&self) -> bool {
        true // MySIMKARI is read-only
    }

    async fn validate_credentials(&self, username: &str, password: &str) -> Result<bool> {
        // Delegate credential validation to MySIMKARI
        self.grpc_client
            .validate_pegawai_credentials(username, password)
            .await
            .map_err(Into::into)
    }
}
```

#### 4.1.3 Registration Configuration Model
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationConfig {
    pub realm_id: Uuid,
    pub enabled: bool,
    pub email_as_username: bool,
    pub verify_email: bool,
    pub captcha_enabled: bool,
    pub terms_and_conditions: bool,
    pub terms_url: Option<String>,
    pub custom_fields: Vec<RegistrationField>,
    pub default_roles: Vec<String>,
    pub default_groups: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub validation: Option<String>, // Regex pattern
    pub options: Option<Vec<String>>, // For SELECT type
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Text,
    Email,
    Tel,
    Select,
    Checkbox,
    Textarea,
}

// Government-specific registration fields
pub fn create_gov_registration_fields() -> Vec<RegistrationField> {
    vec![
        RegistrationField {
            name: "nip".to_string(),
            label: "NIP (Nomor Induk Pegawai)".to_string(),
            field_type: FieldType::Text,
            required: true,
            validation: Some(r"^\d{18}$".to_string()),
            options: None,
            placeholder: Some("18 digit NIP".to_string()),
            help_text: Some("Masukkan NIP 18 digit Anda".to_string()),
        },
        RegistrationField {
            name: "satker".to_string(),
            label: "Satuan Kerja".to_string(),
            field_type: FieldType::Select,
            required: true,
            validation: None,
            options: Some(vec![
                "Kejaksaan Agung".to_string(),
                "Kejaksaan Tinggi DKI Jakarta".to_string(),
                "Kejaksaan Negeri Jakarta Pusat".to_string(),
                // ... loaded from database
            ]),
            placeholder: None,
            help_text: Some("Pilih satuan kerja Anda".to_string()),
        },
        RegistrationField {
            name: "phone".to_string(),
            label: "Nomor Telepon".to_string(),
            field_type: FieldType::Tel,
            required: true,
            validation: Some(r"^(\+62|0)[0-9]{9,12}$".to_string()),
            options: None,
            placeholder: Some("+62812345678".to_string()),
            help_text: Some("Format: +62 atau 0".to_string()),
        },
    ]
}
```

## 5. Correctness Properties

**What are Correctness Properties?**

A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.

In this design, we define 56 correctness properties that validate the functional requirements. Each property:
- Is universally quantified (applies to all valid inputs)
- Is testable through property-based testing
- Maps directly to one or more functional requirements
- Provides a formal specification of expected behavior

**Property Testing Approach**:
- Each property will be implemented as a property-based test using **proptest**
- Minimum 100 iterations per test (due to randomization)
- Each test tagged with: `// Feature: authenc-keycloak-parity, Property {number}: {property_text}`
- Properties complement unit tests (properties verify universal correctness, unit tests verify specific examples)

### 5.1 Realm Management Properties (FR-1)

Property 1: Realm enable/disable state persistence
*For any* realm, toggling its enabled state and then retrieving it should reflect the new state
**Validates: Requirements FR-1.2**

Property 2: Realm name validation
*For any* string, realm name validation should accept only alphanumeric characters without spaces, and reject all other formats
**Validates: Requirements FR-1.3**

Property 3: Realm deletion protection
*For any* realm with active users, deletion attempts should fail with an appropriate error, while realms without users should be deletable
**Validates: Requirements FR-1.4**

### 5.2 Event System Properties

Property 4: Event category separation
*For any* event, it should be stored in the correct table (admin_events or user_events) based on its category
**Validates: Requirements FR-13.1**

Property 5: Admin event completeness
*For any* admin event, all required fields (operation_type, resource_type, resource_path, admin_user_id) should be present and non-null
**Validates: Requirements FR-13.2**

Property 6: User event completeness
*For any* user event, all required fields (event_type, realm_id, created_at) should be present, and optional fields should be null only when appropriate
**Validates: Requirements FR-13.3**

Property 7: Event retention enforcement
*For any* realm with a retention policy, events older than the retention period should be automatically deleted by the cleanup process
**Validates: Requirements FR-14.1, FR-14.2**

### 5.3 Authentication Flow Properties

Property 8: Flow execution ordering
*For any* authentication flow with multiple executions, executions should be processed in priority order (lowest to highest)
**Validates: Requirements FR-16.2**

Property 9: Required execution enforcement
*For any* authentication flow with REQUIRED executions, all required executions must succeed for authentication to succeed
**Validates: Requirements FR-16.1**

Property 10: Alternative execution logic
*For any* authentication flow with ALTERNATIVE executions, authentication should succeed if at least one alternative succeeds
**Validates: Requirements FR-16.1**

### 5.4 Role Management Properties

Property 11: Circular dependency detection
*For any* role hierarchy, attempting to create a circular dependency (A → B → C → A) should be detected and rejected
**Validates: Requirements FR-16.3**

Property 12: Composite role inheritance
*For any* user assigned a composite role, the user should automatically receive all child roles (transitively)
**Validates: Requirements FR-18.1, FR-18.2**

Property 13: Role type scoping
*For any* client role, it should only appear in tokens for that specific client, while realm roles should appear in tokens for all clients
**Validates: Requirements FR-18.1**

### 5.5 User Storage Federation Properties

Property 14: Credential delegation
*For any* user from a federated storage provider, credential validation should be delegated to that provider, not checked locally
**Validates: Requirements FR-20.2**

Property 15: Provider priority ordering
*For any* user lookup, storage providers should be queried in priority order (lowest to highest) until a match is found
**Validates: Requirements FR-20.1**

Property 16: NIP validation with MySIMKARI
*For any* NIP string, validation should call MySIMKARI gRPC service and return the correct validation result (valid/invalid/not found)
**Validates: Requirements FR-21.1, FR-29.2**

### 5.6 Session Management Properties

Property 17: Session limit enforcement
*For any* user with a maximum session limit, when the limit is reached, creating a new session should revoke the oldest existing session
**Validates: Requirements FR-22.1, FR-22.2**

Property 18: Session idle timeout
*For any* session that has been idle longer than the configured timeout, it should be automatically deleted by the cleanup process
**Validates: Requirements FR-22.1**

Property 19: Session max lifespan
*For any* session older than the configured maximum lifespan, it should be automatically deleted regardless of activity
**Validates: Requirements FR-22.1**

### 5.7 User Registration Properties

Property 20: Email verification requirement
*For any* newly registered user (when email verification is enabled), login attempts should fail until the email is verified
**Validates: Requirements FR-24.3**

Property 21: NIP validation during registration
*For any* registration attempt with a NIP, the NIP should be validated against MySIMKARI before account creation
**Validates: Requirements FR-24.2**

Property 22: Default role assignment
*For any* newly registered user, all configured default roles should be automatically assigned upon successful registration
**Validates: Requirements FR-24.1**

### 5.8 Email Template Properties

Property 23: Template variable substitution
*For any* email template with variables, all variables should be correctly substituted with actual values when the email is generated
**Validates: Requirements FR-26.2**

Property 24: Template language selection
*For any* email sent to a user, the template should be selected based on the user's preferred language (if available)
**Validates: Requirements FR-26.1**

### 5.9 Import/Export Properties

Property 25: Realm export completeness
*For any* realm, exporting it should produce a JSON document containing all realm configuration (clients, roles, flows, etc.)
**Validates: Requirements FR-27.1**

Property 26: Export inclusion options
*For any* realm export, the include_users option should control whether users are included in the export
**Validates: Requirements FR-27.2**

Property 27: Import/Export round-trip
*For any* valid realm, exporting then importing should produce an equivalent realm configuration
**Validates: Requirements FR-28.1**

### 5.10 Government-Specific Properties

Property 28: NIP format validation
*For any* string, NIP format validation should accept only 18-digit numeric strings and reject all other formats
**Validates: Requirements FR-29.1**

Property 29: Satker hierarchy access control
*For any* user and resource, access should be granted if the user's Satker is the same as or a parent of the resource's Satker
**Validates: Requirements FR-30.2**

Property 30: Satker assignment persistence
*For any* user, their Satker assignment should be stored and retrievable from user attributes
**Validates: Requirements FR-30.1**

### 5.11 User Search Properties

Property 31: Multi-field search
*For any* search query, searching by username, email, or NIP should return users matching any of these fields
**Validates: Requirements FR-2.2**

Property 32: Search result accuracy
*For any* search query, all returned results should match the query, and all matching users should be returned (up to the limit)
**Validates: Requirements FR-2.2**

### 5.12 User Account Console Properties

Property 33: Profile update validation
*For any* profile update request, system fields (username, NIP) should be rejected while editable fields (name, phone) should be accepted
**Validates: Requirements FR-7.2, FR-7.3**

Property 34: Password change session termination
*For any* successful password change, all other user sessions should be terminated except the current session
**Validates: Requirements FR-8.3**

Property 35: MFA device removal protection
*For any* user with MFA required, attempting to remove all MFA devices should be rejected
**Validates: Requirements FR-9.5**

Property 36: Session termination notification
*For any* remotely terminated session, the user should receive a notification email
**Validates: Requirements FR-10.4**

### 5.13 Data Export and Deletion Properties

Property 37: Personal data export completeness
*For any* user data export request, the exported JSON should contain all user data (profile, attributes, activity log, consents)
**Validates: Requirements FR-12.1**

Property 38: Account deletion grace period
*For any* account deletion request, the account should remain active for 30 days before permanent deletion
**Validates: Requirements FR-12.5**

Property 39: Deletion cancellation
*For any* account deletion request within the grace period, the user should be able to cancel the deletion
**Validates: Requirements FR-12.6**

### 5.14 Client Management Properties

Property 40: Client secret generation security
*For any* newly generated client secret, it should be cryptographically random with at least 256 bits of entropy
**Validates: Requirements FR-3.2**

Property 41: Redirect URI validation
*For any* OAuth2 authorization request, the redirect URI should exactly match one of the client's registered URIs
**Validates: Requirements FR-3.3**

Property 42: Client secret rotation
*For any* client secret rotation, the old secret should remain valid for a configurable grace period
**Validates: Requirements FR-3.5**

### 5.15 Audit Log Properties

Property 43: Admin event immutability
*For any* admin event, once stored, it should not be modifiable or deletable (except by retention policy)
**Validates: Requirements FR-5.1**

Property 44: Event filtering accuracy
*For any* event filter criteria, all returned events should match the filter, and all matching events should be returned
**Validates: Requirements FR-5.2**

Property 45: Event export format
*For any* event export, the exported data should be in valid CSV/JSON format and contain all requested fields
**Validates: Requirements FR-5.3**

### 5.16 System Monitoring Properties

Property 46: Metrics accuracy
*For any* point in time, displayed metrics (active users, sessions, requests/sec) should match actual system state within 5 seconds
**Validates: Requirements FR-6.1**

Property 47: Auto-refresh consistency
*For any* dashboard with auto-refresh enabled, metrics should update every 30 seconds without user interaction
**Validates: Requirements FR-6.6**

### 5.17 Backup Code Properties

Property 48: Backup code uniqueness
*For any* set of generated backup codes, all codes should be unique within the set
**Validates: Requirements FR-9.6**

Property 49: Backup code single-use
*For any* backup code, after successful use, it should be invalidated and not usable again
**Validates: Requirements FR-9.6**

### 5.18 Activity Log Properties

Property 50: Activity log completeness
*For any* security-relevant user action (login, password change, MFA change), an entry should be created in the activity log
**Validates: Requirements FR-11.1, FR-11.2, FR-11.3, FR-11.4**

Property 51: Activity log pagination
*For any* activity log query, results should be paginated with exactly 50 events per page (or fewer on the last page)
**Validates: Requirements FR-11.6**

### 5.19 Government-Specific Properties

Property 52: MySIMKARI cache validity
*For any* NIP validation result cached from MySIMKARI, the cache should expire after exactly 5 minutes
**Validates: Requirements FR-21.4, FR-29.4**

Property 53: Satker hierarchy transitivity
*For any* three Satkers A, B, C where A is parent of B and B is parent of C, then A should be parent of C (transitive closure)
**Validates: Requirements FR-30.2**

Property 54: Satker access control inheritance
*For any* user with access to parent Satker, they should automatically have access to all child Satkers
**Validates: Requirements FR-30.4**

Property 55: Audit report completeness
*For any* date range, the generated audit report should include all required metrics (total users, logins, failed logins, MFA adoption, admin actions, security incidents, compliance status)
**Validates: Requirements FR-31.1**

Property 56: Audit report PDF generation
*For any* audit report, the PDF export should include government branding and all report data in readable format
**Validates: Requirements FR-31.3, FR-31.4**

## 6. Government-Specific Features Design

**Overview**: This section addresses FR-29 (NIP Validation), FR-30 (Satker Hierarchy), and FR-31 (Audit Reports) - critical requirements for Indonesian government deployment.

**Integration Points**:
- MySIMKARI gRPC service for NIP validation and employee data
- SIMAN integration for organizational hierarchy
- MonSAKTI integration for financial data (future)

**Compliance Requirements**:
- Audit compliance
- SPBE (Sistem Pemerintahan Berbasis Elektronik) standards
- Data residency (all data must remain in Indonesia)
- Government security standards

### 6.1 NIP Validation Service (FR-29)

#### 6.1.1 Architecture
```rust
pub struct NipValidationService {
    mysimkari_client: Arc<IntegrasiClient>,
    cache: Arc<RedisCache>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl NipValidationService {
    pub async fn validate_nip(&self, nip: &str) -> Result<NipValidationResult> {
        // 1. Format validation (18 digits, numeric only)
        if !Self::is_valid_format(nip) {
            return Ok(NipValidationResult::InvalidFormat {
                nip: nip.to_string(),
                error: "NIP must be exactly 18 numeric digits".to_string(),
            });
        }

        // 2. Check cache (5 minute TTL)
        let cache_key = format!("nip:validation:{}", nip);
        if let Some(cached) = self.cache.get::<NipValidationResult>(&cache_key).await? {
            return Ok(cached);
        }

        // 3. Call MySIMKARI via circuit breaker
        let result = self.circuit_breaker.call(async {
            self.mysimkari_client
                .verify_nip(nip)
                .await
                .map(|pegawai| NipValidationResult::Valid {
                    nip: nip.to_string(),
                    pegawai: PegawaiInfo {
                        nama: pegawai.nama,
                        email: pegawai.email,
                        satker_id: pegawai.satker_id,
                        satker_name: pegawai.satker_name,
                        jabatan: pegawai.jabatan,
                        golongan: pegawai.golongan,
                    },
                })
                .or_else(|e| {
                    if e.is_not_found() {
                        Ok(NipValidationResult::NotFound {
                            nip: nip.to_string(),
                        })
                    } else {
                        Err(e)
                    }
                })
        }).await?;

        // 4. Cache result for 5 minutes
        self.cache.set(&cache_key, &result, Duration::from_secs(300)).await?;

        Ok(result)
    }

    fn is_valid_format(nip: &str) -> bool {
        nip.len() == 18 && nip.chars().all(|c| c.is_ascii_digit())
    }

    pub async fn auto_populate_user_from_nip(
        &self,
        nip: &str,
    ) -> Result<UserCreationData> {
        let validation = self.validate_nip(nip).await?;

        match validation {
            NipValidationResult::Valid { pegawai, .. } => {
                Ok(UserCreationData {
                    username: nip.to_string(),
                    email: pegawai.email,
                    first_name: Some(pegawai.nama.clone()),
                    last_name: None,
                    attributes: hashmap! {
                        "nip".to_string() => nip.to_string(),
                        "satker_id".to_string() => pegawai.satker_id,
                        "satker_name".to_string() => pegawai.satker_name,
                        "jabatan".to_string() => pegawai.jabatan,
                        "golongan".to_string() => pegawai.golongan,
                    },
                    email_verified: true, // MySIMKARI emails are pre-verified
                    enabled: true,
                })
            }
            NipValidationResult::NotFound { nip } => {
                Err(AuthencError::NipNotFoundInMysimkari(nip))
            }
            NipValidationResult::InvalidFormat { nip, error } => {
                Err(AuthencError::InvalidNipFormat(format!("{}: {}", nip, error)))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NipValidationResult {
    Valid {
        nip: String,
        pegawai: PegawaiInfo,
    },
    NotFound {
        nip: String,
    },
    InvalidFormat {
        nip: String,
        error: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PegawaiInfo {
    pub nama: String,
    pub email: String,
    pub satker_id: String,
    pub satker_name: String,
    pub jabatan: String,
    pub golongan: String,
}
```

#### 6.1.2 Database Schema
```sql
-- NIP validation cache (optional, Redis is primary cache)
CREATE TABLE nip_validation_cache (
    nip VARCHAR(18) PRIMARY KEY,
    validation_result JSONB NOT NULL,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    INDEX idx_nip_cache_expires (expires_at)
);

-- NIP validation audit log
CREATE TABLE nip_validation_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nip VARCHAR(18) NOT NULL,
    validation_result VARCHAR(50) NOT NULL, -- 'valid', 'not_found', 'invalid_format'
    requested_by UUID REFERENCES users(id),
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    INDEX idx_nip_log_nip (nip),
    INDEX idx_nip_log_created (created_at)
);
```

### 6.2 Satker Hierarchy Service

#### 6.2.1 Architecture
```rust
pub struct SatkerHierarchyService {
    db: Arc<Database>,
    cache: Arc<RedisCache>,
}

impl SatkerHierarchyService {
    pub async fn get_satker_path(&self, satker_id: &str) -> Result<Vec<Satker>> {
        // Check cache
        let cache_key = format!("satker:path:{}", satker_id);
        if let Some(cached) = self.cache.get::<Vec<Satker>>(&cache_key).await? {
            return Ok(cached);
        }

        // Query database with recursive CTE
        let query = r#"
            WITH RECURSIVE satker_hierarchy AS (
                -- Base case: start with the given satker
                SELECT id, name, parent_id, level, 0 as depth
                FROM satkers
                WHERE id = $1

                UNION ALL

                -- Recursive case: get parent satkers
                SELECT s.id, s.name, s.parent_id, s.level, sh.depth + 1
                FROM satkers s
                INNER JOIN satker_hierarchy sh ON s.id = sh.parent_id
            )
            SELECT id, name, parent_id, level
            FROM satker_hierarchy
            ORDER BY depth DESC
        "#;

        let rows = self.db.query(query, &[&satker_id]).await?;
        let path: Vec<Satker> = rows.iter()
            .map(|row| Satker {
                id: row.get("id"),
                name: row.get("name"),
                parent_id: row.get("parent_id"),
                level: row.get("level"),
            })
            .collect();

        // Cache for 1 hour (hierarchy changes infrequently)
        self.cache.set(&cache_key, &path, Duration::from_secs(3600)).await?;

        Ok(path)
    }

    pub async fn check_satker_access(
        &self,
        user_satker: &str,
        resource_satker: &str,
    ) -> Result<bool> {
        // Same satker = access granted
        if user_satker == resource_satker {
            return Ok(true);
        }

        // Get resource satker path
        let resource_path = self.get_satker_path(resource_satker).await?;

        // Check if user's satker is in the resource's hierarchy (parent)
        let has_access = resource_path.iter().any(|s| s.id == user_satker);

        Ok(has_access)
    }

    pub async fn get_accessible_satkers(&self, user_satker: &str) -> Result<Vec<String>> {
        // Get all satkers where user's satker is in the hierarchy
        let query = r#"
            WITH RECURSIVE satker_tree AS (
                -- Base case: user's satker
                SELECT id, name, parent_id, level
                FROM satkers
                WHERE id = $1

                UNION ALL

                -- Recursive case: all child satkers
                SELECT s.id, s.name, s.parent_id, s.level
                FROM satkers s
                INNER JOIN satker_tree st ON s.parent_id = st.id
            )
            SELECT id FROM satker_tree
        "#;

        let rows = self.db.query(query, &[&user_satker]).await?;
        let satker_ids: Vec<String> = rows.iter()
            .map(|row| row.get("id"))
            .collect();

        Ok(satker_ids)
    }

    pub async fn load_satker_hierarchy(&self) -> Result<SatkerTree> {
        // Load entire hierarchy for visualization
        let query = r#"
            SELECT id, name, parent_id, level
            FROM satkers
            ORDER BY level, name
        "#;

        let rows = self.db.query(query, &[]).await?;
        let satkers: Vec<Satker> = rows.iter()
            .map(|row| Satker {
                id: row.get("id"),
                name: row.get("name"),
                parent_id: row.get("parent_id"),
                level: row.get("level"),
            })
            .collect();

        // Build tree structure
        let tree = Self::build_tree(satkers);
        Ok(tree)
    }

    fn build_tree(satkers: Vec<Satker>) -> SatkerTree {
        let mut nodes: HashMap<String, SatkerNode> = HashMap::new();
        let mut roots: Vec<String> = Vec::new();

        // Create nodes
        for satker in satkers {
            nodes.insert(satker.id.clone(), SatkerNode {
                satker: satker.clone(),
                children: Vec::new(),
            });

            if satker.parent_id.is_none() {
                roots.push(satker.id.clone());
            }
        }

        // Build parent-child relationships
        let node_ids: Vec<String> = nodes.keys().cloned().collect();
        for id in node_ids {
            if let Some(node) = nodes.get(&id) {
                if let Some(parent_id) = &node.satker.parent_id {
                    if let Some(parent) = nodes.get_mut(parent_id) {
                        parent.children.push(id.clone());
                    }
                }
            }
        }

        SatkerTree { nodes, roots }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Satker {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub level: i32, // 1=Kejaksaan Agung, 2=Kejati, 3=Kejari, etc.
}

#[derive(Debug, Clone)]
pub struct SatkerNode {
    pub satker: Satker,
    pub children: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SatkerTree {
    pub nodes: HashMap<String, SatkerNode>,
    pub roots: Vec<String>,
}
```

#### 6.2.2 Database Schema
```sql
-- Satker hierarchy table
CREATE TABLE satkers (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    parent_id VARCHAR(50) REFERENCES satkers(id),
    level INTEGER NOT NULL, -- 1=Kejaksaan Agung, 2=Kejati, 3=Kejari
    code VARCHAR(20) UNIQUE,
    address TEXT,
    phone VARCHAR(20),
    email VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    INDEX idx_satkers_parent (parent_id),
    INDEX idx_satkers_level (level)
);

-- User satker assignment (stored in user attributes, but indexed here)
CREATE TABLE user_satker_assignments (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    satker_id VARCHAR(50) NOT NULL REFERENCES satkers(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by UUID REFERENCES users(id),
    INDEX idx_user_satker_satker (satker_id)
);

-- Example data
INSERT INTO satkers (id, name, parent_id, level, code) VALUES
('KEJAGUNG', 'Kejaksaan Agung Republik Indonesia', NULL, 1, 'KA'),
('KEJATI_DKI', 'Kejaksaan Tinggi DKI Jakarta', 'KEJAGUNG', 2, 'KT-DKI'),
('KEJARI_JAKPUS', 'Kejaksaan Negeri Jakarta Pusat', 'KEJATI_DKI', 3, 'KN-JAKPUS'),
('KEJARI_JAKSEL', 'Kejaksaan Negeri Jakarta Selatan', 'KEJATI_DKI', 3, 'KN-JAKSEL'),
('KEJATI_JABAR', 'Kejaksaan Tinggi Jawa Barat', 'KEJAGUNG', 2, 'KT-JABAR'),
('KEJARI_BANDUNG', 'Kejaksaan Negeri Bandung', 'KEJATI_JABAR', 3, 'KN-BANDUNG');
```

### 6.3 Audit Report Service

#### 6.3.1 Architecture
```rust
pub struct AuditReportService {
    db: Arc<Database>,
    event_store: Arc<EventStore>,
    user_store: Arc<UserStore>,
    pdf_generator: Arc<PdfGenerator>,
}

impl AuditReportService {
    pub async fn generate_report(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        realm_id: Option<Uuid>,
    ) -> Result<BpkAuditReport> {
        // 1. Collect metrics
        let total_users = self.count_total_users(realm_id).await?;
        let active_users = self.count_active_users(start_date, end_date, realm_id).await?;
        let total_logins = self.count_logins(start_date, end_date, realm_id).await?;
        let failed_logins = self.count_failed_logins(start_date, end_date, realm_id).await?;
        let mfa_adoption = self.calculate_mfa_adoption(realm_id).await?;
        let admin_actions = self.get_admin_actions_summary(start_date, end_date, realm_id).await?;
        let security_incidents = self.get_security_incidents(start_date, end_date, realm_id).await?;
        let compliance_status = self.check_compliance_status(realm_id).await?;

        // 2. Generate report
        Ok(BpkAuditReport {
            report_id: Uuid::new_v4(),
            generated_at: Utc::now(),
            period_start: start_date,
            period_end: end_date,
            realm_id,
            metrics: BpkMetrics {
                total_users,
                active_users,
                total_logins,
                failed_logins,
                failed_login_rate: (failed_logins as f64 / total_logins as f64) * 100.0,
                mfa_adoption_rate: mfa_adoption,
            },
            admin_actions,
            security_incidents,
            compliance_status,
        })
    }

    pub async fn export_to_pdf(&self, report: &BpkAuditReport) -> Result<Vec<u8>> {
        // Generate PDF with government branding
        self.pdf_generator.generate(PdfTemplate {
            title: "Laporan Audit Keamanan Sistem".to_string(),
            logo: include_bytes!("../assets/kejaksaan_logo.png").to_vec(),
            header: format!(
                "Periode: {} s/d {}",
                report.period_start.format("%d %B %Y"),
                report.period_end.format("%d %B %Y")
            ),
            sections: vec![
                PdfSection {
                    title: "Ringkasan Eksekutif".to_string(),
                    content: self.generate_executive_summary(report),
                },
                PdfSection {
                    title: "Metrik Pengguna".to_string(),
                    content: self.generate_user_metrics_section(report),
                },
                PdfSection {
                    title: "Aktivitas Administrator".to_string(),
                    content: self.generate_admin_actions_section(report),
                },
                PdfSection {
                    title: "Insiden Keamanan".to_string(),
                    content: self.generate_security_incidents_section(report),
                },
                PdfSection {
                    title: "Status Kepatuhan".to_string(),
                    content: self.generate_compliance_section(report),
                },
            ],
            footer: format!(
                "Dokumen ini dihasilkan secara otomatis oleh SIMPEL v2\n\
                 Kejaksaan Republik Indonesia\n\
                 Tanggal: {}",
                Utc::now().format("%d %B %Y %H:%M WIB")
            ),
        }).await
    }

    async fn count_total_users(&self, realm_id: Option<Uuid>) -> Result<u64> {
        let query = if let Some(realm) = realm_id {
            "SELECT COUNT(*) FROM users WHERE realm_id = $1"
        } else {
            "SELECT COUNT(*) FROM users"
        };

        let row = if let Some(realm) = realm_id {
            self.db.query_one(query, &[&realm]).await?
        } else {
            self.db.query_one(query, &[]).await?
        };

        Ok(row.get::<_, i64>(0) as u64)
    }

    async fn count_active_users(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        realm_id: Option<Uuid>,
    ) -> Result<u64> {
        let query = if let Some(realm) = realm_id {
            r#"
            SELECT COUNT(DISTINCT user_id)
            FROM user_events
            WHERE event_type = 'LOGIN'
              AND created_at BETWEEN $1 AND $2
              AND realm_id = $3
            "#
        } else {
            r#"
            SELECT COUNT(DISTINCT user_id)
            FROM user_events
            WHERE event_type = 'LOGIN'
              AND created_at BETWEEN $1 AND $2
            "#
        };

        let row = if let Some(realm) = realm_id {
            self.db.query_one(query, &[&start, &end, &realm]).await?
        } else {
            self.db.query_one(query, &[&start, &end]).await?
        };

        Ok(row.get::<_, i64>(0) as u64)
    }

    async fn calculate_mfa_adoption(&self, realm_id: Option<Uuid>) -> Result<f64> {
        let query = if let Some(realm) = realm_id {
            r#"
            SELECT
                COUNT(*) FILTER (WHERE mfa_enabled = true) as mfa_users,
                COUNT(*) as total_users
            FROM users
            WHERE realm_id = $1
            "#
        } else {
            r#"
            SELECT
                COUNT(*) FILTER (WHERE mfa_enabled = true) as mfa_users,
                COUNT(*) as total_users
            FROM users
            "#
        };

        let row = if let Some(realm) = realm_id {
            self.db.query_one(query, &[&realm]).await?
        } else {
            self.db.query_one(query, &[]).await?
        };

        let mfa_users: i64 = row.get("mfa_users");
        let total_users: i64 = row.get("total_users");

        if total_users == 0 {
            Ok(0.0)
        } else {
            Ok((mfa_users as f64 / total_users as f64) * 100.0)
        }
    }

    async fn check_compliance_status(&self, realm_id: Option<Uuid>) -> Result<ComplianceStatus> {
        // Check various compliance requirements
        let password_policy_compliant = self.check_password_policy_compliance(realm_id).await?;
        let mfa_policy_compliant = self.check_mfa_policy_compliance(realm_id).await?;
        let audit_log_compliant = self.check_audit_log_compliance(realm_id).await?;
        let session_policy_compliant = self.check_session_policy_compliance(realm_id).await?;

        let all_compliant = password_policy_compliant
            && mfa_policy_compliant
            && audit_log_compliant
            && session_policy_compliant;

        Ok(ComplianceStatus {
            overall_status: if all_compliant { "COMPLIANT" } else { "NON_COMPLIANT" }.to_string(),
            password_policy: password_policy_compliant,
            mfa_policy: mfa_policy_compliant,
            audit_logging: audit_log_compliant,
            session_management: session_policy_compliant,
            last_audit_date: Utc::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpkAuditReport {
    pub report_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub realm_id: Option<Uuid>,
    pub metrics: BpkMetrics,
    pub admin_actions: AdminActionsSummary,
    pub security_incidents: Vec<SecurityIncident>,
    pub compliance_status: ComplianceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpkMetrics {
    pub total_users: u64,
    pub active_users: u64,
    pub total_logins: u64,
    pub failed_logins: u64,
    pub failed_login_rate: f64,
    pub mfa_adoption_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminActionsSummary {
    pub total_actions: u64,
    pub user_created: u64,
    pub user_updated: u64,
    pub user_deleted: u64,
    pub role_changes: u64,
    pub config_changes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncident {
    pub incident_id: Uuid,
    pub incident_type: String,
    pub severity: String,
    pub description: String,
    pub occurred_at: DateTime<Utc>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub overall_status: String,
    pub password_policy: bool,
    pub mfa_policy: bool,
    pub audit_logging: bool,
    pub session_management: bool,
    pub last_audit_date: DateTime<Utc>,
}
```

#### 6.3.2 REST API Endpoints
```rust
// GET /api/admin/reports/bpk/audit
pub async fn generate_bpk_audit_report(
    State(state): State<Arc<AppState>>,
    Query(params): Query<BpkReportParams>,
) -> Result<Json<BpkAuditReport>, AuthencError> {
    let report = state.bpk_report_service
        .generate_report(params.start_date, params.end_date, params.realm_id)
        .await?;

    Ok(Json(report))
}

// GET /api/admin/reports/bpk/audit/pdf
pub async fn export_bpk_audit_report_pdf(
    State(state): State<Arc<AppState>>,
    Query(params): Query<BpkReportParams>,
) -> Result<impl IntoResponse, AuthencError> {
    let report = state.bpk_report_service
        .generate_report(params.start_date, params.end_date, params.realm_id)
        .await?;

    let pdf_bytes = state.bpk_report_service
        .export_to_pdf(&report)
        .await?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            &format!(
                "attachment; filename=\"Audit_Report_{}.pdf\"",
                Utc::now().format("%Y%m%d")
            ),
        ),
    ];

    Ok((headers, pdf_bytes))
}
```

## 7. Error Handling

**Requirements Addressed**: This section implements error handling patterns to support NFR-47 through NFR-52 (Reliability) and NFR-8 through NFR-18 (Security).

**Error Handling Principles**:
1. **Graceful Degradation**: System continues operating when dependencies fail (NFR-48)
2. **Circuit Breaker Pattern**: Prevents cascading failures in external service calls (NFR-49)
3. **Retry Logic**: Exponential backoff for transient failures (NFR-50)
4. **Clear Error Messages**: User-friendly error messages without exposing sensitive information (NFR-13)
5. **Audit Trail**: All errors logged for troubleshooting (NFR-56)

### 7.1 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthencError {
    // Realm errors
    #[error("Realm not found: {0}")]
    RealmNotFound(Uuid),

    #[error("Realm already exists: {0}")]
    RealmAlreadyExists(String),

    #[error("Cannot delete realm with active users")]
    RealmHasActiveUsers,

    #[error("Invalid realm name: {0}")]
    InvalidRealmName(String),

    // User errors
    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    #[error("Username already exists: {0}")]
    UsernameAlreadyExists(String),

    #[error("Email already exists: {0}")]
    EmailAlreadyExists(String),

    #[error("Invalid NIP format: {0}")]
    InvalidNipFormat(String),

    #[error("NIP not found in MySIMKARI: {0}")]
    NipNotFoundInMysimkari(String),

    // Role errors
    #[error("Role not found: {0}")]
    RoleNotFound(Uuid),

    #[error("Circular role dependency detected")]
    CircularRoleDependency,

    #[error("Cannot add role to itself")]
    SelfReferentialRole,

    // Flow errors
    #[error("Authentication flow not found: {0}")]
    FlowNotFound(Uuid),

    #[error("Cannot modify built-in flow")]
    CannotModifyBuiltInFlow,

    #[error("Flow already exists: {0}")]
    FlowAlreadyExists(String),

    // Session errors
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session limit reached for user: {0}")]
    SessionLimitReached(Uuid),

    #[error("Session expired")]
    SessionExpired,

    // Registration errors
    #[error("Registration disabled for realm")]
    RegistrationDisabled,

    #[error("Email verification required")]
    EmailVerificationRequired,

    #[error("CAPTCHA verification failed")]
    CaptchaVerificationFailed,

    // Import/Export errors
    #[error("Invalid export format")]
    InvalidExportFormat,

    #[error("Import validation failed: {0}")]
    ImportValidationFailed(String),

    // External service errors
    #[error("MySIMKARI service unavailable")]
    MysimkariUnavailable,

    #[error("Secreton service unavailable")]
    SecretonUnavailable,

    // Government-specific errors
    #[error("Satker not found: {0}")]
    SatkerNotFound(String),

    #[error("Satker access denied: user satker {0} cannot access resource satker {1}")]
    SatkerAccessDenied(String, String),

    #[error("Invalid satker hierarchy: circular reference detected")]
    CircularSatkerHierarchy,

    #[error("Audit report generation failed: {0}")]
    AuditReportGenerationFailed(String),

    // Database errors
    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),

    // Generic errors
    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,
}

impl IntoResponse for AuthencError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthencError::RealmNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AuthencError::UserNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AuthencError::RoleNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AuthencError::FlowNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AuthencError::SessionNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),

            AuthencError::RealmAlreadyExists(_) => (StatusCode::CONFLICT, self.to_string()),
            AuthencError::UsernameAlreadyExists(_) => (StatusCode::CONFLICT, self.to_string()),
            AuthencError::EmailAlreadyExists(_) => (StatusCode::CONFLICT, self.to_string()),
            AuthencError::FlowAlreadyExists(_) => (StatusCode::CONFLICT, self.to_string()),

            AuthencError::InvalidRealmName(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AuthencError::InvalidNipFormat(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AuthencError::CircularRoleDependency => (StatusCode::BAD_REQUEST, self.to_string()),
            AuthencError::CannotModifyBuiltInFlow => (StatusCode::BAD_REQUEST, self.to_string()),
            AuthencError::InvalidExportFormat => (StatusCode::BAD_REQUEST, self.to_string()),
            AuthencError::ImportValidationFailed(_) => (StatusCode::BAD_REQUEST, self.to_string()),

            AuthencError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthencError::EmailVerificationRequired => (StatusCode::UNAUTHORIZED, self.to_string()),

            AuthencError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AuthencError::RealmHasActiveUsers => (StatusCode::FORBIDDEN, self.to_string()),

            AuthencError::MysimkariUnavailable => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AuthencError::SecretonUnavailable => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),

            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
```

### 6.2 Error Handling Patterns

#### 6.2.1 Circuit Breaker for External Services
```rust
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_threshold: u32,
    timeout: Duration,
    half_open_timeout: Duration,
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let state = self.state.read().await;

        match *state {
            CircuitState::Open { opened_at } => {
                if Utc::now() - opened_at > self.half_open_timeout {
                    drop(state);
                    self.transition_to_half_open().await;
                    self.try_call(f).await
                } else {
                    Err(AuthencError::InternalError("Circuit breaker open".to_string()))
                }
            }
            CircuitState::HalfOpen => {
                drop(state);
                self.try_call(f).await
            }
            CircuitState::Closed => {
                drop(state);
                self.try_call(f).await
            }
        }
    }

    async fn try_call<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match timeout(self.timeout, f).await {
            Ok(Ok(result)) => {
                self.on_success().await;
                Ok(result)
            }
            Ok(Err(e)) => {
                self.on_failure().await;
                Err(e)
            }
            Err(_) => {
                self.on_failure().await;
                Err(AuthencError::InternalError("Request timeout".to_string()))
            }
        }
    }
}
```

#### 6.2.2 Retry Logic with Exponential Backoff
```rust
pub async fn retry_with_backoff<F, T>(
    mut f: F,
    max_retries: u32,
    initial_delay: Duration,
) -> Result<T>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T>> + Send>>,
{
    let mut delay = initial_delay;
    let mut last_error = None;

    for attempt in 0..=max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if is_retryable(&e) && attempt < max_retries => {
                last_error = Some(e);
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        AuthencError::InternalError("Max retries exceeded".to_string())
    }))
}

fn is_retryable(error: &AuthencError) -> bool {
    matches!(
        error,
        AuthencError::MysimkariUnavailable
            | AuthencError::SecretonUnavailable
            | AuthencError::DatabaseError(_)
    )
}
```

### 6.3 Graceful Degradation

When external services are unavailable, the system should degrade gracefully:

- **MySIMKARI unavailable**: Allow login with cached user data, skip NIP validation
- **Secreton unavailable**: Use fallback encryption keys from environment
- **Redis unavailable**: Fall back to in-memory session storage (single instance only)
- **Kafka unavailable**: Buffer events in memory, retry later
- **Elasticsearch unavailable**: Continue logging to database only

## 8. Testing Strategy

**Requirements Addressed**: This section implements testing approaches to meet NFR-37 through NFR-41 (Maintainability) and ensure all 56 correctness properties are verified.

**Testing Goals**:
- Unit test coverage >80% (NFR-37)
- Integration test coverage >70% (NFR-38)
- E2E test coverage for critical user flows (NFR-39)
- All code passes `cargo clippy` without warnings (NFR-40)
- All code formatted with `cargo fmt` (NFR-41)
- 100% of correctness properties tested via property-based testing

### 8.1 Testing Approach

The testing strategy follows a dual approach combining unit tests and property-based tests:

- **Unit Tests**: Verify specific examples, edge cases, and error conditions
- **Property Tests**: Verify universal properties across all inputs
- Together, they provide comprehensive coverage (unit tests catch concrete bugs, property tests verify general correctness)

### 7.2 Property-Based Testing

#### 7.2.1 Testing Library
Use **proptest** for Rust property-based testing:
```toml
[dev-dependencies]
proptest = "1.4"
```

#### 7.2.2 Test Configuration
- Minimum 100 iterations per property test (due to randomization)
- Each property test must reference its design document property
- Tag format: `// Feature: authenc-keycloak-parity, Property {number}: {property_text}`

#### 7.2.3 Example Property Tests

**Property 1: Realm enable/disable state persistence**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: authenc-keycloak-parity, Property 1: Realm enable/disable state persistence
    proptest! {
        #[test]
        fn test_realm_enable_disable_persistence(
            realm_name in "[a-z]{5,20}",
            initial_enabled in any::<bool>(),
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let service = setup_test_service().await;

                // Create realm with initial state
                let realm = service.create_realm(CreateRealmRequest {
                    name: realm_name.clone(),
                    enabled: initial_enabled,
                    ..Default::default()
                }).await.unwrap();

                // Toggle state
                let new_enabled = !initial_enabled;
                service.update_realm(realm.id, UpdateRealmRequest {
                    enabled: Some(new_enabled),
                    ..Default::default()
                }).await.unwrap();

                // Retrieve and verify
                let retrieved = service.get_realm(realm.id).await.unwrap();
                prop_assert_eq!(retrieved.enabled, new_enabled);

                Ok(())
            }).unwrap();
        }
    }
}
```

**Property 11: Circular dependency detection**
```rust
// Feature: authenc-keycloak-parity, Property 11: Circular dependency detection
proptest! {
    #[test]
    fn test_circular_role_dependency_detection(
        role_count in 3usize..10,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = setup_test_service().await;
            let realm_id = create_test_realm(&service).await;

            // Create chain of roles
            let mut roles = Vec::new();
            for i in 0..role_count {
                let role = service.create_role(realm_id, CreateRoleRequest {
                    name: format!("role_{}", i),
                    ..Default::default()
                }).await.unwrap();
                roles.push(role);
            }

            // Create chain: role_0 -> role_1 -> role_2 -> ... -> role_n
            for i in 0..role_count - 1 {
                service.add_composite_role(
                    roles[i].id,
                    roles[i + 1].id,
                ).await.unwrap();
            }

            // Attempt to close the circle: role_n -> role_0
            let result = service.add_composite_role(
                roles[role_count - 1].id,
                roles[0].id,
            ).await;

            // Should detect circular dependency
            prop_assert!(matches!(result, Err(AuthencError::CircularRoleDependency)));

            Ok(())
        }).unwrap();
    }
}
```

**Property 17: Session limit enforcement**
```rust
// Feature: authenc-keycloak-parity, Property 17: Session limit enforcement
proptest! {
    #[test]
    fn test_session_limit_enforcement(
        max_sessions in 1u32..5,
        extra_sessions in 1usize..5,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = setup_test_service().await;
            let user = create_test_user(&service).await;

            // Configure session limit
            service.set_session_policy(SessionPolicy {
                max_sessions_per_user: Some(max_sessions),
                ..Default::default()
            }).await.unwrap();

            // Create max_sessions sessions
            let mut sessions = Vec::new();
            for _ in 0..max_sessions {
                let session = service.create_session(user.id).await.unwrap();
                sessions.push(session);
            }

            // Create extra sessions
            for _ in 0..extra_sessions {
                let new_session = service.create_session(user.id).await.unwrap();
                sessions.push(new_session);
            }

            // Verify only max_sessions are active
            let active_sessions = service.get_user_sessions(user.id).await.unwrap();
            prop_assert_eq!(active_sessions.len(), max_sessions as usize);

            // Verify oldest sessions were revoked
            for i in 0..extra_sessions {
                let session_exists = service.get_session(sessions[i].id).await.is_ok();
                prop_assert!(!session_exists);
            }

            Ok(())
        }).unwrap();
    }
}
```

**Property 27: Import/Export round-trip**
```rust
// Feature: authenc-keycloak-parity, Property 27: Import/Export round-trip
proptest! {
    #[test]
    fn test_realm_export_import_roundtrip(
        realm_name in "[a-z]{5,20}",
        user_count in 0usize..10,
        client_count in 0usize..5,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = setup_test_service().await;

            // Create realm with random configuration
            let realm = create_test_realm_with_config(&service, realm_name).await;

            // Add users
            for i in 0..user_count {
                create_test_user_in_realm(&service, realm.id, format!("user_{}", i)).await;
            }

            // Add clients
            for i in 0..client_count {
                create_test_client_in_realm(&service, realm.id, format!("client_{}", i)).await;
            }

            // Export realm
            let export = service.export_realm(realm.id, ExportOptions {
                include_users: true,
                include_credentials: false,
                ..Default::default()
            }).await.unwrap();

            // Delete original realm
            service.delete_realm(realm.id).await.unwrap();

            // Import realm
            let imported_realm = service.import_realm(export, false).await.unwrap();

            // Verify equivalence
            let original_users = user_count;
            let imported_users = service.count_users(imported_realm).await.unwrap();
            prop_assert_eq!(original_users, imported_users);

            let original_clients = client_count;
            let imported_clients = service.count_clients(imported_realm).await.unwrap();
            prop_assert_eq!(original_clients, imported_clients);

            Ok(())
        }).unwrap();
    }
}
```

### 7.3 Unit Testing

Unit tests focus on specific examples, edge cases, and error conditions:

```rust
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_realm_name_validation_rejects_spaces() {
        let service = setup_test_service().await;

        let result = service.create_realm(CreateRealmRequest {
            name: "invalid name".to_string(), // Contains space
            ..Default::default()
        }).await;

        assert!(matches!(result, Err(AuthencError::InvalidRealmName(_))));
    }

    #[tokio::test]
    async fn test_realm_name_validation_rejects_special_chars() {
        let service = setup_test_service().await;

        let result = service.create_realm(CreateRealmRequest {
            name: "invalid@name".to_string(), // Contains @
            ..Default::default()
        }).await;

        assert!(matches!(result, Err(AuthencError::InvalidRealmName(_))));
    }

    #[tokio::test]
    async fn test_nip_format_validation_accepts_18_digits() {
        let validator = NipValidator::new();

        let result = validator.validate_format("123456789012345678");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_nip_format_validation_rejects_17_digits() {
        let validator = NipValidator::new();

        let result = validator.validate_format("12345678901234567");
        assert!(matches!(result, Err(AuthencError::InvalidNipFormat(_))));
    }

    #[tokio::test]
    async fn test_nip_format_validation_rejects_non_numeric() {
        let validator = NipValidator::new();

        let result = validator.validate_format("12345678901234567A");
        assert!(matches!(result, Err(AuthencError::InvalidNipFormat(_))));
    }

    #[tokio::test]
    async fn test_email_verification_blocks_login() {
        let service = setup_test_service().await;

        // Create user without email verification
        let user = service.create_user(CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            email_verified: false,
            ..Default::default()
        }).await.unwrap();

        // Attempt login
        let result = service.authenticate(AuthenticateRequest {
            username: "testuser".to_string(),
            password: "password".to_string(),
        }).await;

        assert!(matches!(result, Err(AuthencError::EmailVerificationRequired)));
    }
}
```

### 7.4 Integration Testing

Integration tests verify end-to-end flows:

```rust
#[tokio::test]
async fn test_complete_registration_flow() {
    let service = setup_test_service().await;
    let realm = create_test_realm(&service).await;

    // Enable registration
    service.update_registration_config(realm.id, RegistrationConfig {
        enabled: true,
        verify_email: true,
        ..Default::default()
    }).await.unwrap();

    // Register user
    let registration = service.register_user(RegisterRequest {
        realm_id: realm.id,
        username: "newuser".to_string(),
        email: "newuser@example.com".to_string(),
        password: "SecurePass123!".to_string(),
        custom_fields: HashMap::new(),
    }).await.unwrap();

    // Verify email sent
    let emails = get_sent_emails().await;
    assert_eq!(emails.len(), 1);
    assert_eq!(emails[0].to, "newuser@example.com");

    // Attempt login before verification (should fail)
    let login_result = service.authenticate(AuthenticateRequest {
        username: "newuser".to_string(),
        password: "SecurePass123!".to_string(),
    }).await;
    assert!(matches!(login_result, Err(AuthencError::EmailVerificationRequired)));

    // Verify email
    service.verify_email(registration.verification_token).await.unwrap();

    // Login after verification (should succeed)
    let login_result = service.authenticate(AuthenticateRequest {
        username: "newuser".to_string(),
        password: "SecurePass123!".to_string(),
    }).await;
    assert!(login_result.is_ok());
}
```

### 7.5 Test Coverage Goals

- Unit test coverage: >80%
- Integration test coverage: >70%
- Property test coverage: 100% of correctness properties
- E2E test coverage: All critical user flows

### 7.6 Performance Testing

Performance tests verify non-functional requirements:

```rust
#[tokio::test]
async fn test_api_response_time_p95() {
    let service = setup_test_service().await;
    let mut response_times = Vec::new();

    // Make 1000 requests
    for _ in 0..1000 {
        let start = Instant::now();
        service.get_realm(test_realm_id).await.unwrap();
        let duration = start.elapsed();
        response_times.push(duration);
    }

    // Calculate p95
    response_times.sort();
    let p95_index = (response_times.len() as f64 * 0.95) as usize;
    let p95 = response_times[p95_index];

    // Verify p95 < 100ms
    assert!(p95 < Duration::from_millis(100), "p95 response time: {:?}", p95);
}
```

## 8. Implementation Phases

**Overview**: Implementation is organized into 5 phases over 12 months, prioritizing critical features first. Each phase includes specific functional requirements and deliverables.

**Phase Summary**:
- **Phase 1 (Q1 2026)**: Critical features - Admin Console, User Console, Event System, Required Actions
- **Phase 2 (Q2 2026)**: High-priority features - Authentication Flows, Composite Roles, Federation, Registration
- **Phase 3 (Q3 2026)**: Medium-priority features - Session Management, Import/Export, Client Scopes
- **Phase 4 (Q3-Q4 2026)**: Government-specific features - NIP Validation, Satker Hierarchy, Audit Reports
- **Phase 5 (Q4 2026)**: Low-priority features - Client Adapters, Device Flow, Polish

### Phase 1: Critical Features (Q1 2026 - 3 months)

**Goal**: Make Authenc production-ready with full management capabilities

**Requirements Addressed**: FR-1 through FR-17, NFR-1 through NFR-24

**Priority 1: Admin Console UI (8 weeks) - Addresses FR-1 through FR-6**
- Week 1-2: Architecture, design, and Leptos setup
  - Set up Leptos 0.8.x project structure
  - Create routing system and layout components
  - Implement authentication and authorization middleware
  - Design component library and styling system
- Week 3-4: Realm and user management
  - Realm CRUD operations UI
  - User management interface (list, search, create, edit, delete)
  - Bulk user operations (CSV import, bulk enable/disable)
  - User session management UI
- Week 5-6: Client and role management
  - Client CRUD operations UI
  - Client secret generation and rotation
  - Role management UI (realm roles and client roles)
  - Composite role creation and visualization
- Week 7-8: Security settings, audit logs, and monitoring dashboard
  - Security policy configuration UI
  - Audit log viewer with filtering and export
  - System monitoring dashboard with real-time metrics
  - Health check and diagnostics UI

**Priority 2: Event System (3 weeks)**
- Week 1: Database schema for separate admin/user events
  - Create admin_events and user_events tables
  - Create event_config table
  - Implement database migrations
  - Add indexes for performance
- Week 2: Event storage and retrieval implementation
  - Implement EventService with admin/user event separation
  - Add event filtering and search
  - Implement event retention policies
  - Add automatic cleanup job
- Week 3: Event export to Kafka/Elasticsearch
  - Implement Kafka producer for event streaming
  - Implement Elasticsearch indexing
  - Add export configuration UI
  - Test event export pipeline

**Priority 3: Required Actions System (2 weeks)**
- Week 1: Required actions framework and database schema
  - Create required_actions and user_required_actions tables
  - Implement RequiredActionService
  - Create built-in actions (verify email, update password, configure MFA, accept terms)
  - Implement action execution engine
- Week 2: UI integration and testing
  - Create required actions UI flow
  - Integrate with authentication flow
  - Add admin UI for action configuration
  - Test all built-in actions

**Priority 4: User Account Console (4 weeks)**
- Week 1-2: Profile and password management
  - Create user account console layout
  - Implement profile view and edit UI
  - Implement password change UI with validation
  - Add email change with verification
- Week 3-4: MFA device management and session management
  - Implement MFA device list and registration UI
  - Add TOTP and WebAuthn device setup flows
  - Implement backup code generation and display
  - Create session management UI (view active sessions, logout)
  - Add activity log viewer

### Phase 2: High-Priority Features (Q2 2026 - 3 months)

**Goal**: Improve usability and integration

**Priority 5: Authentication Flows (4 weeks)**
- Week 1-2: Flow configuration framework
  - Create authentication_flows and authentication_executions tables
  - Implement AuthenticationFlowService
  - Create flow execution engine
  - Implement built-in flows (browser, direct grant, registration, reset credentials)
- Week 3-4: UI for flow management and testing
  - Create flow management UI (list, create, edit, copy)
  - Implement execution management UI (add, remove, reorder)
  - Add flow testing interface
  - Create flow visualization

**Priority 6: Composite Roles (2 weeks)**
- Week 1: Database schema and core logic
  - Create composite_roles table
  - Implement composite role service
  - Add circular dependency detection
  - Implement effective roles calculation
- Week 2: UI integration and hierarchy visualization
  - Add composite role creation UI
  - Implement role hierarchy tree visualization
  - Add child role management UI
  - Test composite role inheritance

**Priority 7: User Storage Federation (3 weeks)**
- Week 1: Storage provider SPI
  - Design UserStorageProvider trait
  - Implement provider registry
  - Add provider priority system
  - Create provider configuration framework
- Week 2: MySIMKARI integration
  - Implement MysimkariUserStorageProvider
  - Add NIP validation integration
  - Implement user attribute mapping
  - Add provider caching
- Week 3: Testing and caching
  - Test MySIMKARI provider with real data
  - Optimize caching strategy
  - Add fallback mechanisms
  - Document provider integration

**Priority 8: User Self-Registration (3 weeks)**
- Week 1: Registration page and NIP validation
  - Create registration page UI
  - Implement NIP validation with MySIMKARI
  - Add custom field support
  - Implement registration form validation
- Week 2: Email verification and CAPTCHA
  - Implement email verification flow
  - Add CAPTCHA integration
  - Create verification email template
  - Test verification flow
- Week 3: Custom fields and testing
  - Add custom field configuration UI
  - Implement field validation rules
  - Add default role assignment
  - Test complete registration flow

**Priority 9: Email Templates (2 weeks)**
- Week 1: Template engine and default templates
  - Implement email template system
  - Create default templates (Indonesian/English)
  - Add variable substitution
  - Implement HTML and plain text versions
- Week 2: Admin UI for template customization
  - Create template management UI
  - Add template editor with preview
  - Implement test email functionality
  - Add government branding to templates

### Phase 3: Medium-Priority Features (Q3 2026 - 3 months)

**Goal**: Operational excellence and flexibility

**Priority 10: Client Scopes & Protocol Mappers Enhancement (3 weeks)**
- Week 1: Client scopes implementation
  - Create client_scopes table
  - Implement ClientScopeService
  - Add default/optional scope assignment
  - Implement scope consent UI
- Week 2: Protocol mappers enhancement
  - Extend protocol mapper types
  - Add custom mapper support
  - Implement Satker hierarchy mapper
  - Add mapper testing UI
- Week 3: UI integration and testing
  - Create client scope management UI
  - Add protocol mapper configuration UI
  - Test token generation with scopes
  - Document mapper configuration

**Priority 11: Session Management Enhancement (2 weeks)**
- Week 1: Session policies implementation
  - Implement session limit enforcement
  - Add idle timeout and max lifespan
  - Implement Remember Me functionality
  - Add offline session support
- Week 2: Session cleanup and UI
  - Implement automatic session cleanup
  - Add session policy configuration UI
  - Create session monitoring dashboard
  - Test session policies

**Priority 12: Import/Export System (3 weeks)**
- Week 1: Export implementation
  - Implement realm export to JSON
  - Add export options (users, credentials, etc.)
  - Implement large export splitting
  - Test export completeness
- Week 2: Import implementation
  - Implement realm import from JSON
  - Add import validation
  - Implement overwrite/create modes
  - Add import rollback on failure
- Week 3: UI and automation
  - Create import/export UI
  - Add import progress display
  - Implement backup scheduling
  - Document import/export procedures

**Priority 13: Theme System (2 weeks)**
- Week 1: Theme engine implementation
  - Design theme system architecture
  - Implement theme loading and rendering
  - Create default themes (light, dark)
  - Add theme inheritance
- Week 2: Theme customization UI
  - Create theme management UI
  - Add theme editor (CSS, templates)
  - Implement theme preview mode
  - Add logo and branding support

**Priority 14: Admin CLI (3 weeks)**
- Week 1: CLI framework
  - Set up CLI project structure
  - Implement authentication
  - Add configuration management
  - Create command framework
- Week 2: Admin operations
  - Implement realm management commands
  - Add user management commands
  - Implement client management commands
  - Add role management commands
- Week 3: Import/export and scripting
  - Implement import/export commands
  - Add batch operation support
  - Create scripting examples
  - Document CLI usage

### Phase 4: Government-Specific Features (Q3-Q4 2026 - 3 months)

**Goal**: Complete government integration and compliance

**Priority 15: NIP Validation Enhancement (2 weeks)**
- Week 1: NIP validation service
  - Implement NipValidationService
  - Add MySIMKARI integration
  - Implement caching strategy
  - Add validation audit logging
- Week 2: UI integration and testing
  - Add NIP validation to registration
  - Implement NIP validation in user management
  - Add validation status display
  - Test with MySIMKARI

**Priority 16: Satker Hierarchy Integration (3 weeks)**
- Week 1: Satker hierarchy service
  - Implement SatkerHierarchyService
  - Create satkers table and load data
  - Implement hierarchy queries
  - Add access control logic
- Week 2: Access control integration
  - Integrate Satker access control with authorization
  - Add Satker filtering to queries
  - Implement Satker-based permissions
  - Test access control
- Week 3: UI and visualization
  - Create Satker hierarchy visualization
  - Add Satker assignment UI
  - Implement Satker selector component
  - Document Satker hierarchy

**Priority 17: Audit Reports (2 weeks)**
- Week 1: Report generation
  - Implement BpkAuditReportService
  - Add metrics collection
  - Implement compliance checking
  - Create report data structure
- Week 2: PDF export and UI
  - Implement PDF generation with branding
  - Create report generation UI
  - Add report scheduling
  - Test report generation

**Priority 18: SPBE Compliance Features (2 weeks)**
- Week 1: Compliance framework
  - Implement compliance checking framework
  - Add SPBE-specific checks
  - Create compliance dashboard
  - Implement compliance reporting
- Week 2: Documentation and testing
  - Document compliance requirements
  - Create compliance checklist
  - Test compliance checks
  - Generate compliance reports

### Phase 5: Low-Priority Features (Q4 2026 - 2 months)

**Goal**: Complete feature parity and polish

**Priority 19: Client Adapters (4 weeks)**
- Week 1: Rust adapter
- Week 2: JavaScript/TypeScript adapter
- Week 3: Python adapter
- Week 4: Documentation and examples

**Priority 20: Device Flow (2 weeks)**
- Week 1: Device flow implementation
- Week 2: UI and testing

**Priority 21: User Impersonation (1 week)**
- Implementation and audit trail

**Priority 22: Additional Features (3 weeks)**
- Theme system enhancements
- Additional protocol mappers
- Performance optimizations
- Bug fixes and polish

## 9. Security Considerations

**Requirements Addressed**: This section implements security controls to meet NFR-8 through NFR-18 (Security) and ensure compliance with government security standards.

**Security Standards**:
- OWASP Top 10 compliance (NFR-8)
- XSS protection (NFR-9)
- CSRF protection (NFR-10)
- Content Security Policy (NFR-11)
- Secure headers (NFR-12)
- Input validation (NFR-13)
- Rate limiting (NFR-14)
- Argon2id password hashing (NFR-15)
- Ed25519 JWT signing (NFR-16)
- TLS 1.3 for all communication (NFR-17)
- Tamper-proof audit logs (NFR-18)

### 9.1 Authentication Security
- All passwords hashed with Argon2id (work factor: 2, memory: 64MB, parallelism: 4)
- JWT tokens signed with Ed25519 (superior to RSA)
- Token expiry: Access tokens 15 minutes, refresh tokens 7 days
- MFA required for admin accounts
- Brute force protection with adaptive rate limiting

### 9.2 Authorization Security
- Role-based access control (RBAC) for all admin operations
- Attribute-based access control (ABAC) for fine-grained permissions
- Satker hierarchy enforcement for government data
- Principle of least privilege

### 9.3 Data Security
- All sensitive data encrypted at rest (ChaCha20-Poly1305)
- All communication encrypted in transit (TLS 1.3, mTLS for gRPC)
- Secrets stored in Secreton, never in environment variables
- Audit logs tamper-proof with cryptographic signatures

### 9.4 Input Validation
- All user inputs validated and sanitized
- SQL injection prevention via prepared statements
- XSS prevention via Content Security Policy
- CSRF protection on all state-changing operations

### 9.5 Compliance
- OWASP Top 10 compliance
- WCAG 2.1 Level AA accessibility
- GDPR data protection (consent management, right to erasure)
- ISO 27001 security controls
- Audit requirements

## 10. Performance Optimization

**Requirements Addressed**: This section implements optimizations to meet NFR-1 through NFR-7 (Performance) and NFR-42 through NFR-46 (Scalability).

**Performance Targets**:
- Admin Console load time <2 seconds (NFR-1)
- API response time p95 <100ms (NFR-2)
- Database query time p95 <10ms (NFR-3)
- Support 1000+ concurrent users (NFR-4)
- Memory usage <500MB per instance (NFR-5)
- JWT token generation <5ms (NFR-6)
- Session lookup <1ms (NFR-7)

**Scalability Requirements**:
- Horizontal scaling support (NFR-42)
- Stateless design with sessions in Redis (NFR-43)
- Database connection pooling (NFR-44)
- Caching strategy for frequently accessed data (NFR-45)
- Load balancer ready (NFR-46)

### 10.1 Caching Strategy
- Redis cache for sessions (TTL: session timeout)
- Redis cache for frequently accessed data (users, roles, clients)
- Cache invalidation on updates
- Cache warming on startup

### 10.2 Database Optimization
- Connection pooling (min: 10, max: 50)
- Prepared statement caching
- Indexes on frequently queried columns
- Partitioning for large tables (events)

### 10.3 Query Optimization
- Pagination for large result sets (50 items per page)
- Lazy loading for related entities
- Batch operations for bulk updates
- Async/await for non-blocking I/O

### 10.4 Frontend Optimization
- WASM bundle size optimization (<2MB)
- Code splitting for lazy loading
- Resource caching (service worker)
- Debouncing for search inputs

## 11. Monitoring and Observability

**Requirements Addressed**: This section implements monitoring and observability to meet NFR-53 through NFR-58 (Observability).

**Observability Requirements**:
- Prometheus metrics for all operations (NFR-53)
- Structured logging in JSON format (NFR-54)
- Distributed tracing with OpenTelemetry (NFR-55)
- Audit trail for all admin operations (NFR-56)
- Performance monitoring (NFR-57)
- Error tracking and alerting (NFR-58)

### 11.1 Metrics (Prometheus)
- Request rate, error rate, duration (RED metrics)
- Active users, active sessions
- Database connection pool utilization
- Cache hit/miss ratio
- Event processing rate
- External service latency (MySIMKARI, Secreton)

### 11.2 Logging (Structured JSON)
- All requests logged with correlation ID
- Error logs with stack traces
- Audit logs for all admin operations
- Performance logs for slow queries (>100ms)

### 11.3 Tracing (OpenTelemetry)
- Distributed tracing across services
- Span annotations for key operations
- Trace sampling (10% in production)

### 11.4 Alerting
- High error rate (>5%)
- High latency (p95 >100ms)
- Database connection pool exhaustion
- External service unavailability
- Disk space low (<20%)

## 12. Deployment Strategy

**Requirements Addressed**: This section implements deployment practices to meet NFR-59 through NFR-66 (Deployment).

**Deployment Requirements**:
- Docker images <300MB (NFR-59)
- Kubernetes manifests provided (NFR-60)
- Helm charts available (NFR-61)
- CI/CD pipeline configured (NFR-62)
- Blue-green deployment support (NFR-63)
- Canary deployment support (NFR-64)
- Automated database migrations (NFR-65)
- Startup time <5 seconds (NFR-66)

### 12.1 Container Images
- Multi-stage Docker builds
- Image size <300MB
- Security scanning with Trivy
- Signed images with Cosign

### 12.2 Kubernetes Deployment
- Deployment with rolling updates
- HorizontalPodAutoscaler (min: 3, max: 10)
- PodDisruptionBudget (minAvailable: 2)
- Resource requests/limits defined
- Health checks (liveness, readiness)

### 12.3 Database Migrations
- Automated with refinery
- Backward compatible migrations
- Rollback plan for each migration
- Migration testing in staging

### 12.4 Blue-Green Deployment
- Zero-downtime deployments
- Traffic switching via Istio
- Automated rollback on errors
- Canary deployments for risky changes

## 13. Documentation Requirements

### 13.1 User Documentation
- Admin Console user guide
- User Account Console guide
- Registration guide
- MFA setup guide
- Troubleshooting guide

### 13.2 Developer Documentation
- API reference (OpenAPI/Swagger)
- Architecture documentation
- Database schema documentation
- Integration guide (MySIMKARI, Secreton)
- Custom authenticator development guide

### 13.3 Operations Documentation
- Deployment guide
- Configuration guide
- Backup and restore procedures
- Disaster recovery plan
- Monitoring and alerting setup

## 14. Success Criteria

### 14.1 Feature Completeness
- ✅ 100% of critical features implemented (Admin Console, User Console, Event System, Required Actions) by Q1 2026
- ✅ 90%+ of high-priority features implemented (Authentication Flows, Composite Roles, User Storage Federation, Self-Registration, Email Templates) by Q2 2026
- ✅ 70%+ of medium-priority features implemented (Client Scopes, Session Management, Import/Export, Theme System, Admin CLI) by Q3 2026
- ✅ 100% of government-specific features implemented (NIP Validation, Satker Hierarchy, Audit Reports) by Q4 2026

### 14.2 Quality Metrics
- ✅ Test coverage >80% (unit tests)
- ✅ Integration test coverage >70%
- ✅ Property test coverage: 100% of correctness properties (56 properties)
- ✅ Zero critical security vulnerabilities
- ✅ <5 high-priority bugs in production
- ✅ Performance targets met (p95 <100ms for API, <2s for UI load)
- ✅ All accessibility requirements met (WCAG 2.1 AA)
- ✅ All code passes `cargo clippy` without warnings
- ✅ All code formatted with `cargo fmt`

### 14.3 User Satisfaction
- ✅ Admin console usability score >4/5
- ✅ User account console usability score >4/5
- ✅ Documentation completeness score >4/5
- ✅ Support ticket reduction by 30%
- ✅ User onboarding time reduced by 50% (with self-registration)

### 14.4 Adoption Metrics
- ✅ 100% of SIMPEL services using Authenc by Q2 2026
- ✅ Zero Keycloak dependencies by Q3 2026
- ✅ Admin CLI usage >50% of admin operations by Q3 2026
- ✅ Production deployment in 5+ Satker by Q2 2026
- ✅ 1000+ active users by Q4 2026
- ✅ MFA adoption rate >80% for admin accounts
- ✅ Self-registration accounts for >60% of new users

### 14.5 Operational Metrics
- ✅ 99.9% uptime achieved
- ✅ Mean time to recovery (MTTR) <15 minutes
- ✅ Zero data loss incidents
- ✅ Audit compliance achieved by Q3 2026
- ✅ ISO 27001 compliance maintained
- ✅ All security audits passed
- ✅ Disaster recovery tested quarterly

### 14.6 Performance Metrics
- ✅ API response time p95 <100ms
- ✅ Database query time p95 <10ms
- ✅ Admin Console load time <2 seconds
- ✅ User Account Console load time <2 seconds
- ✅ Support 1000+ concurrent users
- ✅ Memory usage <500MB per instance
- ✅ JWT token generation <5ms
- ✅ Session lookup <1ms (Redis cache)

### 14.7 Security Metrics
- ✅ All passwords hashed with Argon2id
- ✅ All JWT tokens signed with Ed25519
- ✅ All communication uses TLS 1.3
- ✅ All audit logs cryptographically signed
- ✅ Zero XSS vulnerabilities
- ✅ Zero SQL injection vulnerabilities
- ✅ Zero CSRF vulnerabilities
- ✅ All OWASP Top 10 mitigated

### 14.8 Compliance Metrics
- ✅ WCAG 2.1 Level AA compliance
- ✅ GDPR compliance (consent management, right to erasure)
- ✅ ISO 27001 compliance
- ✅ Audit requirements met
- ✅ SPBE (Sistem Pemerintahan Berbasis Elektronik) compliance
- ✅ Data residency requirements met (data in Indonesia)
- ✅ Government security standards met

## 15. Risks and Mitigation

### 15.1 Technical Risks
- **Leptos UI complexity**: Mitigate with early prototyping, proven patterns, experienced developers
- **Performance degradation**: Mitigate with load testing, profiling, optimization, horizontal scaling
- **Security vulnerabilities**: Mitigate with security audits, penetration testing, bug bounty program
- **Integration issues**: Mitigate with comprehensive integration tests, fallback mechanisms

### 15.2 Project Risks
- **Timeline delays**: Mitigate with phased approach, MVP first, parallel development
- **Resource constraints**: Mitigate by prioritizing critical features, hiring contractors if needed
- **Scope creep**: Mitigate with strict change control process, prioritization framework

### 15.3 Operational Risks
- **Production downtime**: Mitigate with blue-green deployment, rollback procedures, monitoring
- **Data loss**: Mitigate with automated backups, replication, disaster recovery plan
- **Performance issues**: Mitigate with monitoring, auto-scaling, performance testing
- **Security breach**: Mitigate with security hardening, monitoring, incident response plan

---

**Document Version**: 2.0
**Last Updated**: 2026-02-12
**Status**: READY FOR IMPLEMENTATION
**Next Step**: Create tasks.md based on this design

## Appendix A: Requirements Traceability Matrix

This section provides complete traceability from requirements to design components and correctness properties.

### A.1 Functional Requirements Traceability

| Requirement ID | Requirement Name | Design Section | Correctness Properties | Implementation Phase |
|----------------|------------------|----------------|------------------------|----------------------|
| FR-1 | Realm Management | 3.1 Admin Console UI | Properties 1-3 | Phase 1 |
| FR-2 | User Management | 3.1 Admin Console UI | Properties 31-32 | Phase 1 |
| FR-3 | Client Management | 3.1 Admin Console UI | Properties 40-42 | Phase 1 |
| FR-4 | Role Management | 3.1 Admin Console UI, 4.1.1 | Properties 11-13 | Phase 1 |
| FR-5 | Audit Log Viewer | 3.1 Admin Console UI | Properties 43-45 | Phase 1 |
| FR-6 | System Monitoring | 3.1 Admin Console UI | Properties 46-47 | Phase 1 |
| FR-7 | Profile Management | 3.2 User Account Console | Property 33 | Phase 1 |
| FR-8 | Password Management | 3.2 User Account Console | Property 34 | Phase 1 |
| FR-9 | MFA Device Management | 3.2 User Account Console | Properties 35, 48-49 | Phase 1 |
| FR-10 | Session Management | 3.2 User Account Console | Property 36 | Phase 1 |
| FR-11 | Activity Log | 3.2 User Account Console | Properties 50-51 | Phase 1 |
| FR-12 | Data Export & Deletion | 3.2 User Account Console | Properties 37-39 | Phase 1 |
| FR-13 | Event Separation | 3.3.1 Event Service | Properties 4-6 | Phase 1 |
| FR-14 | Event Retention | 3.3.1 Event Service | Property 7 | Phase 1 |
| FR-15 | Event Export | 3.3.1 Event Service | Properties 43-45 | Phase 1 |
| FR-16 | Flow Configuration | 3.3.2 Authentication Flow Service | Properties 8-10 | Phase 2 |
| FR-17 | Required Actions | 3.3.2 Authentication Flow Service | N/A (framework) | Phase 1 |
| FR-18 | Composite Roles | 4.1.1 Composite Role Model | Properties 11-12 | Phase 2 |
| FR-19 | Role Scoping | 4.1.1 Composite Role Model | Property 13 | Phase 2 |
| FR-20 | Storage Provider Interface | 4.1.2 User Storage Provider Model | Properties 14-15 | Phase 2 |
| FR-21 | MySIMKARI Integration | 4.1.2 User Storage Provider Model | Property 16 | Phase 2 |
| FR-22 | Session Policies | 3.3.2 Session Service | Properties 17-19 | Phase 3 |
| FR-23 | Session Cleanup | 3.3.2 Session Service | Properties 18-19 | Phase 3 |
| FR-24 | Self-Registration | 4.1.3 Registration Configuration Model | Properties 20-22 | Phase 2 |
| FR-25 | Registration Customization | 4.1.3 Registration Configuration Model | N/A (configuration) | Phase 2 |
| FR-26 | Email Templates | 3.3.2 Email Template Service | Properties 23-24 | Phase 2 |
| FR-27 | Realm Export | 3.3.2 Import/Export Service | Properties 25-26 | Phase 3 |
| FR-28 | Realm Import | 3.3.2 Import/Export Service | Property 27 | Phase 3 |
| FR-29 | NIP Validation | 6.1 NIP Validation Service | Properties 28, 52 | Phase 4 |
| FR-30 | Satker Hierarchy | 6.2 Satker Hierarchy Service | Properties 29-30, 53-54 | Phase 4 |
| FR-31 | Audit Reports | 6.3 Audit Report Service | Properties 55-56 | Phase 4 |

### A.2 Non-Functional Requirements Traceability

| NFR Category | NFR IDs | Design Section | Verification Method |
|--------------|---------|----------------|---------------------|
| Performance | NFR-1 to NFR-7 | Section 10 - Performance Optimization | Load testing, profiling |
| Security | NFR-8 to NFR-18 | Section 9 - Security Considerations | Security audit, penetration testing |
| Accessibility | NFR-19 to NFR-24 | Section 3.1, 3.2 - UI Components | WCAG 2.1 AA compliance testing |
| Internationalization | NFR-25 to NFR-29 | Section 3.1, 3.2 - UI Components | Language testing, locale testing |
| Browser Compatibility | NFR-30 to NFR-34 | Section 2.2.1 - Frontend Layer | Cross-browser testing |
| Maintainability | NFR-35 to NFR-41 | Section 8 - Testing Strategy | Code review, test coverage analysis |
| Scalability | NFR-42 to NFR-46 | Section 2.4 - Deployment Architecture | Load testing, horizontal scaling tests |
| Reliability | NFR-47 to NFR-52 | Section 7.2 - Error Handling Patterns | Chaos engineering, failover testing |
| Observability | NFR-53 to NFR-58 | Section 11 - Monitoring and Observability | Metrics validation, log analysis |
| Deployment | NFR-59 to NFR-66 | Section 12 - Deployment Strategy | CI/CD pipeline testing, deployment verification |

### A.3 Correctness Properties Summary

**Total Properties**: 56
**Properties with Requirements Mapping**: 56 (100%)
**Property Categories**:
- Realm Management: Properties 1-3 (FR-1)
- Event System: Properties 4-7 (FR-13, FR-14)
- Authentication Flows: Properties 8-10 (FR-16)
- Role Management: Properties 11-13 (FR-18, FR-19)
- User Storage Federation: Properties 14-16 (FR-20, FR-21)
- Session Management: Properties 17-19 (FR-22, FR-23)
- User Registration: Properties 20-22 (FR-24)
- Email Templates: Properties 23-24 (FR-26)
- Import/Export: Properties 25-27 (FR-27, FR-28)
- Government-Specific: Properties 28-30, 52-56 (FR-29, FR-30, FR-31)
- User Search: Properties 31-32 (FR-2)
- User Account Console: Properties 33-39 (FR-7 to FR-12)
- Client Management: Properties 40-42 (FR-3)
- Audit Log: Properties 43-45 (FR-5)
- System Monitoring: Properties 46-47 (FR-6)
- Backup Codes: Properties 48-49 (FR-9)
- Activity Log: Properties 50-51 (FR-11)

### A.4 Implementation Phase Coverage

| Phase | Duration | Requirements Covered | Deliverables |
|-------|----------|---------------------|--------------|
| Phase 1 | Q1 2026 (3 months) | FR-1 to FR-15, FR-17 | Admin Console, User Console, Event System, Required Actions |
| Phase 2 | Q2 2026 (3 months) | FR-16, FR-18 to FR-21, FR-24 to FR-26 | Authentication Flows, Composite Roles, Federation, Registration, Email Templates |
| Phase 3 | Q3 2026 (3 months) | FR-22, FR-23, FR-27, FR-28 | Session Management, Import/Export, Client Scopes |
| Phase 4 | Q3-Q4 2026 (3 months) | FR-29 to FR-31 | NIP Validation, Satker Hierarchy, BPK Reports |
| Phase 5 | Q4 2026 (2 months) | N/A (enhancements) | Client Adapters, Device Flow, Polish |

**Total Duration**: 12 months
**Total Requirements**: 31 functional requirements
**Coverage**: 100% of functional requirements addressed

## Appendix B: Technology Stack Details

### B.1 Frontend Stack
- **Framework**: Leptos 0.8.x (WASM CSR)
- **Styling**: Tailwind CSS 3.x
- **Icons**: Heroicons
- **Charts**: Chart.js via WASM bindings
- **HTTP Client**: gloo-net
- **State Management**: Leptos RwSignal
- **Routing**: Leptos Router
- **Build Tool**: Trunk

### B.2 Backend Stack
- **Language**: Rust (Edition 2024, MSRV 1.90+)
- **HTTP Framework**: Axum 0.8.x
- **gRPC Framework**: Tonic 0.14.x + Prost 0.14.x
- **Database**: PostgreSQL 13+ (tokio-postgres + deadpool)
- **Cache**: Redis 6+ (redis-rs)
- **Async Runtime**: Tokio
- **Serialization**: serde + serde_json
- **Validation**: validator
- **Logging**: tracing + tracing-subscriber
- **Metrics**: prometheus + opentelemetry

### B.3 Infrastructure Stack
- **Container Platform**: Kubernetes (MicroK8s)
- **Service Mesh**: Istio (mTLS STRICT mode)
- **Load Balancer**: MetalLB (172.15.10.200-230)
- **Storage**: Longhorn (distributed block storage)
- **Monitoring**: Prometheus + Grafana
- **Tracing**: OpenTelemetry + Jaeger
- **Event Streaming**: Kafka (optional)
- **Search**: Elasticsearch (optional)

### B.4 Security Stack
- **Password Hashing**: Argon2id (work factor: 2, memory: 64MB)
- **JWT Signing**: Ed25519 (superior to RSA)
- **Symmetric Encryption**: ChaCha20-Poly1305, AES-256-GCM
- **Hashing**: Blake3, SHA3-256
- **Key Derivation**: HKDF, PBKDF2
- **Post-Quantum**: ML-KEM, ML-DSA (optional)
- **TLS**: TLS 1.3 for all communication
- **mTLS**: Istio service mesh for gRPC

---

## Document Summary

**Document Version**: 2.0 (REFRESHED)
**Last Updated**: 2026-02-12
**Status**: READY FOR IMPLEMENTATION
**Next Step**: Create tasks.md based on this design

### Coverage Summary

**Functional Requirements**: 31/31 (100%)
- Critical: FR-1 to FR-17 (Admin Console, User Console, Event System, Required Actions)
- High Priority: FR-18 to FR-26 (Authentication Flows, Roles, Federation, Registration)
- Medium Priority: FR-27, FR-28 (Import/Export)
- Government-Specific: FR-29 to FR-31 (NIP, Satker, BPK Reports)

**Non-Functional Requirements**: 66/66 (100%)
- Performance: NFR-1 to NFR-7
- Security: NFR-8 to NFR-18
- Accessibility: NFR-19 to NFR-24
- Internationalization: NFR-25 to NFR-29
- Browser Compatibility: NFR-30 to NFR-34
- Maintainability: NFR-35 to NFR-41
- Scalability: NFR-42 to NFR-46
- Reliability: NFR-47 to NFR-52
- Observability: NFR-53 to NFR-58
- Deployment: NFR-59 to NFR-66

**Correctness Properties**: 56/56 (100%)
- All properties mapped to functional requirements
- All properties testable via property-based testing
- Complete traceability from requirements to properties

**Implementation Phases**: 5 phases over 12 months
- Phase 1 (Q1 2026): Critical features
- Phase 2 (Q2 2026): High-priority features
- Phase 3 (Q3 2026): Medium-priority features
- Phase 4 (Q3-Q4 2026): Government-specific features
- Phase 5 (Q4 2026): Low-priority features and polish

**Architecture Compliance**:
- ✅ Follows SIMPEL architecture patterns
- ✅ Leptos 0.8.x for frontend (WASM CSR)
- ✅ Axum 0.8.x for backend REST API
- ✅ Tonic 0.14.x for gRPC services
- ✅ PostgreSQL for primary data storage
- ✅ Redis for caching and sessions
- ✅ Istio service mesh for mTLS
- ✅ Kubernetes deployment with MicroK8s

**Quality Assurance**:
- ✅ Unit test coverage target: >80%
- ✅ Integration test coverage target: >70%
- ✅ Property-based testing for all 56 properties
- ✅ E2E testing for critical user flows
- ✅ Security audits and penetration testing
- ✅ Performance testing and optimization
- ✅ Accessibility testing (WCAG 2.1 AA)
- ✅ Cross-browser compatibility testing

**Government Compliance**:
- ✅ BPK audit requirements addressed
- ✅ SPBE compliance designed
- ✅ Data residency requirements met
- ✅ MySIMKARI integration designed
- ✅ NIP validation system designed
- ✅ Satker hierarchy enforcement designed
- ✅ Indonesian language support planned

**Production Readiness**:
- ✅ High availability (3 replicas minimum)
- ✅ Horizontal scaling support
- ✅ Blue-green deployment strategy
- ✅ Automated database migrations
- ✅ Comprehensive monitoring and alerting
- ✅ Disaster recovery procedures
- ✅ Security hardening
- ✅ Performance optimization

This design document provides a complete, production-ready architecture for Authenc enterprise features with full traceability to all requirements and comprehensive implementation guidance.s Router

### B.2 Backend Stack
- **Language**: Rust Edition 2024, MSRV 1.90+
- **HTTP Framework**: Axum 0.8.x
- **gRPC Framework**: Tonic 0.14.x + Prost 0.14.x
- **Database**: PostgreSQL 13+ with tokio-postgres + deadpool
- **Cache**: Redis 6+ with redis-rs
- **Cryptography**: Ed25519, ChaCha20-Poly1305, Argon2id
- **JWT**: jsonwebtoken
- **MFA**: totp-rs, webauthn-rs
- **Metrics**: Prometheus, OpenTelemetry
- **Tracing**: OpenTelemetry, tracing
- **Testing**: proptest (property-based), tokio-test

### B.3 Infrastructure Stack
- **Container**: Docker
- **Orchestration**: Kubernetes (MicroK8s)
- **Service Mesh**: Istio
- **Load Balancer**: MetalLB
- **Storage**: Longhorn
- **Monitoring**: Prometheus + Grafana
- **Logging**: Elasticsearch + Kibana (optional)
- **Event Streaming**: Kafka (optional)

### B.4 External Integrations
- **Secreton**: gRPC client for secret management
- **MySIMKARI**: gRPC client for NIP validation and employee data
- **SMTP**: Email sending
- **Kafka**: Event streaming (optional)
- **Elasticsearch**: Audit log search (optional)

## Appendix C: Database Schema Summary

### C.1 Core Tables
- `realms` - Realm configuration
- `users` - User accounts
- `roles` - Roles (realm and client)
- `composite_roles` - Role hierarchy
- `clients` - OAuth2/OIDC clients
- `sessions` - User sessions
- `user_attributes` - Custom user attributes

### C.2 Event Tables
- `admin_events` - Administrator actions
- `user_events` - User activities
- `event_config` - Event configuration per realm

### C.3 Authentication Tables
- `authentication_flows` - Authentication flows
- `authentication_executions` - Flow executions
- `required_actions` - Required action definitions
- `user_required_actions` - User-specific required actions

### C.4 Federation Tables
- `user_storage_providers` - External user storage configuration
- `identity_providers` - External IdP configuration
- `federated_identities` - User identity links

### C.5 Government Tables
- `satkers` - Satker hierarchy
- `user_satker_assignments` - User Satker assignments
- `nip_validation_cache` - NIP validation cache
- `nip_validation_log` - NIP validation audit log

### C.6 Configuration Tables
- `registration_config` - Registration configuration
- `email_templates` - Email templates
- `session_policies` - Session policies
- `client_scopes` - Client scopes
- `protocol_mappers` - Protocol mappers

## Appendix D: API Endpoints Summary

### D.1 Admin API
- `GET /api/admin/realms` - List realms
- `POST /api/admin/realms` - Create realm
- `GET /api/admin/realms/{id}` - Get realm
- `PUT /api/admin/realms/{id}` - Update realm
- `DELETE /api/admin/realms/{id}` - Delete realm
- `GET /api/admin/users` - List users
- `POST /api/admin/users` - Create user
- `GET /api/admin/users/{id}` - Get user
- `PUT /api/admin/users/{id}` - Update user
- `DELETE /api/admin/users/{id}` - Delete user
- `GET /api/admin/clients` - List clients
- `POST /api/admin/clients` - Create client
- `GET /api/admin/roles` - List roles
- `POST /api/admin/roles` - Create role
- `GET /api/admin/events/admin` - List admin events
- `GET /api/admin/events/user` - List user events
- `GET /api/admin/metrics` - Get system metrics
- `GET /api/admin/reports/bpk/audit` - Generate BPK audit report
- `GET /api/admin/reports/bpk/audit/pdf` - Export BPK report as PDF

### D.2 User Account API
- `GET /api/account/profile` - Get user profile
- `PUT /api/account/profile` - Update user profile
- `POST /api/account/password` - Change password
- `GET /api/account/mfa/devices` - List MFA devices
- `POST /api/account/mfa/totp` - Register TOTP device
- `POST /api/account/mfa/webauthn` - Register WebAuthn device
- `DELETE /api/account/mfa/devices/{id}` - Remove MFA device
- `GET /api/account/sessions` - List active sessions
- `DELETE /api/account/sessions/{id}` - Terminate session
- `GET /api/account/activity` - Get activity log
- `POST /api/account/export` - Request data export
- `POST /api/account/delete` - Request account deletion

### D.3 Authentication API
- `POST /api/auth/login` - User login
- `POST /api/auth/logout` - User logout
- `POST /api/auth/register` - User registration
- `POST /api/auth/verify-email` - Verify email
- `POST /api/auth/reset-password` - Request password reset
- `POST /api/auth/mfa/verify` - Verify MFA code

### D.4 OAuth2/OIDC API
- `GET /oauth2/authorize` - Authorization endpoint
- `POST /oauth2/token` - Token endpoint
- `GET /oauth2/userinfo` - UserInfo endpoint
- `POST /oauth2/revoke` - Token revocation
- `POST /oauth2/introspect` - Token introspection
- `GET /.well-known/openid-configuration` - OIDC discovery

### D.5 Government API
- `POST /api/gov/nip/validate` - Validate NIP
- `GET /api/gov/satker/hierarchy` - Get Satker hierarchy
- `GET /api/gov/satker/{id}/path` - Get Satker path
- `GET /api/gov/satker/accessible` - Get accessible Satkers

## Appendix E: Glossary

- **Authenc**: SIMPEL's custom Identity and Access Management service
- **BPK**: Badan Pemeriksa Keuangan (Indonesian Supreme Audit Agency)
- **MySIMKARI**: Government employee management system
- **NIP**: Nomor Induk Pegawai (Employee Identification Number) - 18 digits
- **Satker**: Satuan Kerja (Organizational Unit) in government hierarchy
- **SIMAN**: Sistem Informasi Manajemen (Management Information System)
- **MonSAKTI**: Monitoring SAKTI (Government financial system monitoring)
- **SPBE**: Sistem Pemerintahan Berbasis Elektronik (Electronic-Based Government System)
- **Kejaksaan Agung**: Supreme Prosecutor's Office of Indonesia
- **Kejati**: Kejaksaan Tinggi (High Prosecutor's Office - provincial level)
- **Kejari**: Kejaksaan Negeri (District Prosecutor's Office)
- **WCAG**: Web Content Accessibility Guidelines
- **OWASP**: Open Web Application Security Project
- **GDPR**: General Data Protection Regulation
- **ISO 27001**: Information Security Management System standard
- **FIPS 140-3**: Federal Information Processing Standard for cryptographic modules
