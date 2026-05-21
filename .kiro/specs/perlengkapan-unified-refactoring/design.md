# Design Document: Unified Perlengkapan System

## Overview

This document provides the comprehensive technical design for the unified perlengkapan system, consolidating the previously planned multi-crate architecture into a single, cohesive backend service. The system manages Barang Milik Negara (BMN) for Kejaksaan RI, providing a production-ready, scalable, and maintainable solution from day one.

### Design Goals

1. **Unified Architecture**: Single crate backend (`layanan-perlengkapan`) with clear internal module boundaries
2. **Zero-Trust Security**: JWT validation via Authenc gRPC, secrets from Secreton vault
3. **Production-Ready**: No migration needed - built for production from the start
4. **Maintainability**: Clear module organization, comprehensive testing, excellent documentation
5. **Performance**: Sub-200ms API response times, efficient database access, Redis caching
6. **Scalability**: Stateless design supporting horizontal scaling

### Technology Stack

| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| **Language** | Rust Edition 2024 | MSRV 1.95+ | Type safety, performance, memory safety |
| **HTTP Framework** | Axum | 0.8.x | REST API endpoints |
| **Database** | PostgreSQL | 15+ | Primary data store |
| **DB Driver** | tokio-postgres + deadpool-postgres | Latest | Async connection pooling |
| **Migrations** | refinery | Latest | Database schema management |
| **Cache** | Redis | Latest | Performance optimization |
| **gRPC** | Tonic + Prost | 0.14.x | Inter-service communication |
| **Frontend** | Leptos | 0.8.x | WASM microfrontend |
| **Validation** | validator + garde | Latest | Input validation |
| **Error Handling** | thiserror | 2.0.x | Structured errors |
| **Logging** | tracing + tracing-subscriber | Latest | Structured logging |
| **Metrics** | Prometheus | Latest | Observability |
| **Tracing** | OpenTelemetry | Latest | Distributed tracing |
| **Scheduler** | tokio-cron-scheduler | Latest | Background jobs |
| **WebSocket** | Axum WS | 0.8.x | Real-time notifications |

## Architecture

### High-Level System Architecture

```mermaid
flowchart TB
    subgraph Browser["🌐 Browser (WASM CSR)"]
        Portal["Portal Microfrontend"]
        PerlengkapanUI["Perlengkapan Microfrontend<br/>(antarmuka/perlengkapan)"]
    end

    subgraph Backend["⚙️ Unified Backend Service"]
        direction TB
        Router["Axum Router<br/>(Single Entry Point)"]

        subgraph Modules["Internal Modules"]
            BankAset["bank_aset"]
            KebutuhanBMN["kebutuhan_bmn"]
            PemakaianBMN["pemakaian_bmn"]
            PenghapusanBMN["penghapusan_bmn"]
            PakaianDinas["pakaian_dinas"]
            Workflow["workflow"]
            Dashboard["dashboard"]
            Admin["admin"]
            Dokumen["dokumen"]
            Notifikasi["notifikasi"]
            Bantuan["bantuan"]
        end

        subgraph Infrastructure["Shared Infrastructure"]
            DB["Database Pool"]
            gRPCClients["gRPC Clients"]
            Middleware["Middleware"]
            Config["Configuration"]
            Errors["Error Handling"]
        end
    end

    subgraph Core["🔐 Core Services (gRPC)"]
        Authenc["Authenc<br/>(Authentication)"]
        Secreton["Secreton<br/>(Secrets Vault)"]
        Integrasi["Integrasi<br/>(External Systems)"]
    end

    subgraph Data["💾 Data Layer"]
        PostgreSQL[(PostgreSQL<br/>Database)]
        Redis[(Redis<br/>Cache)]
        S3["S3-Compatible<br/>Object Storage"]
    end

    Portal -->|"REST API<br/>(JSON/HTTP)"| Router
    PerlengkapanUI -->|"REST API<br/>(JSON/HTTP)"| Router
    PerlengkapanUI -.->|"WebSocket<br/>(Real-time)"| Router

    Router --> Modules
    Modules --> Infrastructure

    Infrastructure -->|"gRPC<br/>(mTLS)"| Core
    Infrastructure --> Data

    Authenc <-->|"gRPC"| Secreton

    style Browser fill:#e1f5fe
    style Backend fill:#fff3e0
    style Core fill:#fce4ec
    style Data fill:#e8f5e9
```

### Module Organization

The unified backend is organized as a single crate with clear internal module boundaries:

```
layanan/perlengkapan/
├── Cargo.toml                    # Single crate manifest (uses workspace dependencies)
├── Dockerfile                    # Multi-stage production build
├── README.md                     # Service documentation
├── AGENTS.md                     # Development guidelines
├── migrations/                   # Database migrations (refinery)
│   ├── V001__initial_schema.sql
│   ├── V002__bank_aset.sql
│   ├── V003__workflow.sql
│   └── ...
├── proto/                        # gRPC proto definitions (if needed)
│   └── perlengkapan.proto
├── src/
│   ├── main.rs                   # Application entry point
│   ├── lib.rs                    # Library exports for testing
│   ├── config.rs                 # Configuration management
│   │
│   ├── infrastructure/           # Shared infrastructure
│   │   ├── mod.rs
│   │   ├── database.rs           # Database pool setup
│   │   ├── grpc_clients.rs       # Authenc/Secreton clients
│   │   ├── middleware/           # HTTP middleware
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs           # JWT validation
│   │   │   ├── cors.rs           # CORS configuration
│   │   │   ├── rate_limit.rs     # Rate limiting
│   │   │   └── logging.rs        # Request logging
│   │   ├── errors.rs             # Error types
│   │   └── cache.rs              # Redis cache client
│   │
│   ├── models/                   # Shared domain models
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── bmn.rs
│   │   ├── workflow.rs
│   │   └── common.rs
│   │
│   ├── bank_aset/                # Asset management module
│   │   ├── mod.rs                # Public API
│   │   ├── handlers.rs           # HTTP handlers
│   │   ├── models.rs             # Module-specific models
│   │   ├── repository.rs         # Database operations
│   │   ├── services.rs           # Business logic
│   │   └── validation.rs         # Input validation
│   │
│   ├── kebutuhan_bmn/            # BMN needs planning
│   │   ├── mod.rs
│   │   ├── handlers.rs
│   │   ├── models.rs
│   │   ├── repository.rs
│   │   └── services.rs
│   │
│   ├── pemakaian_bmn/            # BMN usage tracking
│   │   └── ...
│   │
│   ├── penghapusan_bmn/          # Asset disposal
│   │   └── ...
│   │
│   ├── pakaian_dinas/            # Uniform management
│   │   └── ...
│   │
│   ├── workflow/                 # Workflow engine
│   │   ├── mod.rs
│   │   ├── handlers.rs
│   │   ├── models.rs
│   │   ├── repository.rs
│   │   ├── services.rs
│   │   ├── engine.rs             # Workflow execution engine
│   │   └── rules.rs              # Business rules
│   │
│   ├── dashboard/                # Dashboard & reporting
│   │   ├── mod.rs
│   │   ├── handlers.rs
│   │   ├── models.rs
│   │   ├── services.rs
│   │   └── aggregations.rs       # Data aggregation logic
│   │
│   ├── admin/                    # Admin functions
│   │   └── ...
│   │
│   ├── dokumen/                  # Document management
│   │   ├── mod.rs
│   │   ├── handlers.rs
│   │   ├── models.rs
│   │   ├── repository.rs
│   │   ├── services.rs
│   │   ├── storage.rs            # S3 integration
│   │   ├── templates.rs          # Document templates
│   │   └── generators/           # PDF/Excel generation
│   │       ├── mod.rs
│   │       ├── pdf.rs
│   │       └── excel.rs
│   │
│   ├── notifikasi/               # Notification system
│   │   ├── mod.rs
│   │   ├── handlers.rs
│   │   ├── models.rs
│   │   ├── repository.rs
│   │   ├── services.rs
│   │   ├── channels/             # Notification channels
│   │   │   ├── mod.rs
│   │   │   ├── email.rs
│   │   │   ├── sms.rs
│   │   │   ├── whatsapp.rs
│   │   │   └── push.rs
│   │   ├── templates.rs          # Notification templates
│   │   └── websocket.rs          # WebSocket handler
│   │
│   └── bantuan/                  # Help/ticket system
│       ├── mod.rs
│       ├── handlers.rs
│       ├── models.rs
│       ├── repository.rs
│       ├── services.rs
│       ├── workflow.rs           # Ticket workflow
│       └── faq.rs                # FAQ management
│
└── tests/                        # Integration tests
    ├── common/                   # Test utilities
    │   ├── mod.rs
    │   ├── fixtures.rs
    │   └── helpers.rs
    ├── api_tests.rs
    ├── database_tests.rs
    └── integration_tests.rs
```

### Communication Flow

```mermaid
sequenceDiagram
    participant FE as Frontend (Leptos WASM)
    participant Router as Axum Router
    participant MW as Auth Middleware
    participant Handler as Module Handler
    participant Service as Business Service
    participant Repo as Repository
    participant DB as PostgreSQL
    participant Cache as Redis
    participant Authenc as Authenc (gRPC)
    participant Secreton as Secreton (gRPC)

    FE->>Router: HTTP Request + JWT Token
    Router->>MW: Extract & Validate Token
    MW->>Authenc: ValidateToken(jwt)
    Authenc-->>MW: TokenInfo(user_id, roles)
    MW->>Handler: Request + UserContext
    Handler->>Service: Business Operation

    alt Cache Hit
        Service->>Cache: Get Cached Data
        Cache-->>Service: Cached Result
    else Cache Miss
        Service->>Repo: Query Data
        Repo->>DB: SQL Query
        DB-->>Repo: Result Set
        Repo-->>Service: Domain Models
        Service->>Cache: Store in Cache
    end

    Service-->>Handler: Result
    Handler-->>Router: JSON Response
    Router-->>FE: HTTP Response
```

## Components and Interfaces

### 1. Application Entry Point (main.rs)

**Responsibilities:**
- Initialize configuration from environment and Secreton
- Set up database connection pool
- Create gRPC clients (Authenc, Secreton, Integrasi)
- Initialize Redis cache
- Build Axum router with all module routes
- Configure middleware (auth, CORS, rate limiting, logging)
- Start HTTP server with graceful shutdown

**Key Functions:**

```rust
async fn main() -> Result<()>
async fn load_config() -> Result<Config>
async fn setup_database(config: &Config) -> Result<Pool>
async fn setup_grpc_clients(config: &Config) -> Result<GrpcClients>
async fn setup_cache(config: &Config) -> Result<RedisClient>
fn build_router(state: AppState) -> Router
async fn run_migrations(pool: &Pool) -> Result<()>
```

### 2. Configuration Management (config.rs)

**Configuration Sources:**
1. Environment variables (local development via dotenvy)
2. Secreton vault (production via Kubernetes auth)

**Configuration Structure:**

```rust
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub authenc: AuthencConfig,
    pub secreton: SecretonConfig,
    pub integrasi: IntegrasiConfig,
    pub cors: CorsConfig,
    pub rate_limit: RateLimitConfig,
    pub features: FeatureFlags,
}

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub shutdown_timeout: Duration,
}

pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
}
```

**Validation:**
- All required fields validated at startup
- Descriptive errors for missing/invalid configuration
- Secrets never logged

### 3. Infrastructure Layer

#### Database (infrastructure/database.rs)

**Connection Pooling:**

```rust
pub struct DatabasePool {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl DatabasePool {
    pub async fn new(config: &DatabaseConfig) -> Result<Self>
    pub async fn get_connection(&self) -> Result<PooledConnection>
    pub async fn health_check(&self) -> Result<()>
}
```

**Migration Management:**
- Use refinery for schema migrations
- Migrations run as init container in Kubernetes
- Versioned migration files in `migrations/` directory
- Rollback support for failed migrations

#### gRPC Clients (infrastructure/grpc_clients.rs)

**Client Management:**

```rust
pub struct GrpcClients {
    pub authenc: AuthencClient,
    pub secreton: SecretonClient,
    pub integrasi: IntegrasiClient,
}

pub struct AuthencClient {
    client: AuthServiceClient<Channel>,
}

impl AuthencClient {
    pub async fn new(endpoint: String) -> Result<Self>
    pub async fn validate_token(&self, token: &str) -> Result<TokenInfo>
    pub async fn check_permission(&self, user_id: Uuid, permission: &str) -> Result<bool>
    pub async fn health_check(&self) -> Result<()>
}
```

**Features:**
- mTLS for secure communication
- Connection pooling
- Retry logic with exponential backoff
- Circuit breaker pattern
- Health checks

#### Middleware (infrastructure/middleware/)

**Authentication Middleware (auth.rs):**

```rust
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError>
```

**Functionality:**
- Extract JWT from Authorization header
- Validate token via Authenc gRPC
- Inject UserContext into request extensions
- Return 401 for invalid/expired tokens

**CORS Middleware (cors.rs):**
- Configure allowed origins from config
- Support preflight requests
- Secure headers

**Rate Limiting (rate_limit.rs):**
- Per-user and per-IP rate limiting
- Configurable limits
- Redis-backed for distributed rate limiting

**Logging Middleware (logging.rs):**
- Request/response logging with correlation IDs
- Performance metrics
- Sanitize sensitive data

#### Error Handling (infrastructure/errors.rs)

**Error Type Hierarchy:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),

    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Authentication failed: {0}")]
    Unauthorized(String),

    #[error("Permission denied: {0}")]
    Forbidden(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "AUTH_FAILED", self.to_string()),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, "PERMISSION_DENIED", self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND", self.to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", self.to_string()),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT", self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}
```

### 4. Module Pattern

Each business module follows a consistent pattern:

#### Module Structure

```rust
// mod.rs - Public API
pub mod handlers;
pub mod models;
pub mod repository;
pub mod services;
pub mod validation;

pub use handlers::*;
pub use models::*;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/:id", get(get_by_id).put(update).delete(delete))
}
```

#### Handlers (handlers.rs)

- HTTP request/response handling
- Input validation
- Call service layer
- Error handling

```rust
pub async fn create_asset(
    State(state): State<AppState>,
    user: UserContext,
    Json(request): Json<CreateAssetRequest>,
) -> Result<Json<AssetResponse>, AppError> {
    request.validate()?;

    let asset = state.bank_aset_service
        .create_asset(user.user_id, request)
        .await?;

    Ok(Json(asset.into()))
}
```

#### Models (models.rs)

- Request/response DTOs
- Domain models
- Validation rules

```rust
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAssetRequest {
    #[validate(length(min = 1, max = 50))]
    pub kode_barang: String,

    #[validate(length(min = 1, max = 255))]
    pub nama: String,

    pub kategori_id: Uuid,
    pub satker_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetResponse {
    pub id: Uuid,
    pub kode_barang: String,
    pub nama: String,
    pub kategori_id: Uuid,
    pub satker_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### Repository (repository.rs)

- Database operations
- Query building
- Transaction management

```rust
pub struct AssetRepository {
    pool: DatabasePool,
}

impl AssetRepository {
    pub async fn create(&self, asset: &CreateAssetRequest, created_by: Uuid) -> Result<Asset> {
        let client = self.pool.get_connection().await?;
        let row = client.query_one(
            "INSERT INTO assets (kode_barang, nama, kategori_id, satker_id, created_by)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *",
            &[&asset.kode_barang, &asset.nama, &asset.kategori_id, &asset.satker_id, &created_by]
        ).await?;

        Ok(Asset::from_row(&row)?)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Asset>>
    pub async fn list(&self, filters: AssetFilters, pagination: Pagination) -> Result<Vec<Asset>>
    pub async fn update(&self, id: Uuid, updates: &UpdateAssetRequest) -> Result<Asset>
    pub async fn delete(&self, id: Uuid) -> Result<()>
}
```

#### Services (services.rs)

- Business logic
- Authorization checks
- Cache management
- Event publishing

```rust
pub struct AssetService {
    repository: AssetRepository,
    cache: RedisClient,
    authenc: AuthencClient,
}

impl AssetService {
    pub async fn create_asset(
        &self,
        user_id: Uuid,
        request: CreateAssetRequest,
    ) -> Result<Asset> {
        // Check permissions
        if !self.authenc.check_permission(user_id, "asset:create").await? {
            return Err(AppError::Forbidden("Cannot create asset".into()));
        }

        // Create asset
        let asset = self.repository.create(&request, user_id).await?;

        // Invalidate cache
        self.cache.delete(&format!("assets:list")).await?;

        Ok(asset)
    }
}
```

## Data Models

### Core Domain Models

#### Asset (BMN)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub kode_barang: String,
    pub nama: String,
    pub kategori_id: Uuid,
    pub satker_id: Uuid,
    pub kondisi: AssetCondition,
    pub nilai_perolehan: BigDecimal,
    pub tanggal_perolehan: NaiveDate,
    pub lokasi: String,
    pub status: AssetStatus,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetCondition {
    Baik,
    RusakRingan,
    RusakBerat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetStatus {
    Aktif,
    Dipinjam,
    Dalam Perbaikan,
    Dihapus,
}
```

#### Workflow Instance

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: Uuid,
    pub workflow_definition_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub current_step: String,
    pub status: WorkflowStatus,
    pub initiated_by: Uuid,
    pub initiated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Pending,
    InProgress,
    Approved,
    Rejected,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub instance_id: Uuid,
    pub step_name: String,
    pub approver_id: Uuid,
    pub action: Option<ApprovalAction>,
    pub comments: Option<String>,
    pub acted_at: Option<DateTime<Utc>>,
    pub sla_deadline: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalAction {
    Approve,
    Reject,
    Delegate(Uuid),
    RequestChanges,
}
```

#### Document

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_path: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub classification: DocumentClassification,
    pub uploaded_by: Uuid,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentClassification {
    Public,
    Internal,
    Confidential,
    Secret,
}
```

#### Notification

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    WorkflowApproval,
    WorkflowCompleted,
    AssetUpdate,
    SystemAlert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Medium,
    High,
    Critical,
}
```

#### Ticket

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub category: TicketCategory,
    pub priority: TicketPriority,
    pub status: TicketStatus,
    pub created_by: Uuid,
    pub assigned_to: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TicketCategory {
    Technical,
    DataEntry,
    Workflow,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TicketPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TicketStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}
```

### Database Schema

```mermaid
erDiagram
    USERS ||--o{ ASSETS : creates
    USERS ||--o{ WORKFLOW_INSTANCES : initiates
    USERS ||--o{ NOTIFICATIONS : receives
    USERS ||--o{ TICKETS : creates

    ASSETS ||--|| ASSET_CATEGORIES : belongs_to
    ASSETS ||--|| SATKERS : belongs_to
    ASSETS ||--o{ ASSET_USAGE : has
    ASSETS ||--o{ ASSET_DISPOSAL : has
    ASSETS ||--o{ DOCUMENTS : has

    WORKFLOW_DEFINITIONS ||--o{ WORKFLOW_INSTANCES : defines
    WORKFLOW_INSTANCES ||--o{ WORKFLOW_STEPS : contains
    WORKFLOW_STEPS ||--|| USERS : assigned_to

    TICKETS ||--o{ TICKET_COMMENTS : has
    TICKETS ||--|| USERS : assigned_to

    DOCUMENTS ||--|| USERS : uploaded_by

    USERS {
        uuid id PK
        string email UK
        string name
        jsonb roles
        timestamp created_at
    }

    ASSETS {
        uuid id PK
        string kode_barang UK
        string nama
        uuid kategori_id FK
        uuid satker_id FK
        string kondisi
        decimal nilai_perolehan
        date tanggal_perolehan
        string lokasi
        string status
        uuid created_by FK
        timestamp created_at
        timestamp updated_at
    }

    ASSET_CATEGORIES {
        uuid id PK
        string kode UK
        string nama
        uuid parent_id FK
    }

    SATKERS {
        uuid id PK
        string kode UK
        string nama
        string alamat
    }

    WORKFLOW_DEFINITIONS {
        uuid id PK
        string name UK
        string entity_type
        jsonb steps
        boolean active
    }

    WORKFLOW_INSTANCES {
        uuid id PK
        uuid workflow_definition_id FK
        string entity_type
        uuid entity_id
        string current_step
        string status
        uuid initiated_by FK
        timestamp initiated_at
        timestamp completed_at
    }

    WORKFLOW_STEPS {
        uuid id PK
        uuid instance_id FK
        string step_name
        uuid approver_id FK
        string action
        text comments
        timestamp acted_at
        timestamp sla_deadline
    }

    DOCUMENTS {
        uuid id PK
        string filename
        string content_type
        bigint size_bytes
        string storage_path
        string entity_type
        uuid entity_id
        string classification
        uuid uploaded_by FK
        timestamp uploaded_at
    }

    NOTIFICATIONS {
        uuid id PK
        uuid user_id FK
        string title
        text message
        string notification_type
        string priority
        boolean read
        timestamp read_at
        timestamp created_at
        jsonb metadata
    }

    TICKETS {
        uuid id PK
        string title
        text description
        string category
        string priority
        string status
        uuid created_by FK
        uuid assigned_to FK
        timestamp created_at
        timestamp updated_at
        timestamp resolved_at
    }

    TICKET_COMMENTS {
        uuid id PK
        uuid ticket_id FK
        uuid user_id FK
        text comment
        timestamp created_at
    }
```

### Database Indexes

**Performance-Critical Indexes:**

```sql
-- Assets
CREATE INDEX idx_assets_satker_id ON assets(satker_id);
CREATE INDEX idx_assets_kategori_id ON assets(kategori_id);
CREATE INDEX idx_assets_status ON assets(status);
CREATE INDEX idx_assets_created_at ON assets(created_at DESC);

-- Workflow
CREATE INDEX idx_workflow_instances_status ON workflow_instances(status);
CREATE INDEX idx_workflow_instances_entity ON workflow_instances(entity_type, entity_id);
CREATE INDEX idx_workflow_steps_approver ON workflow_steps(approver_id) WHERE action IS NULL;
CREATE INDEX idx_workflow_steps_sla ON workflow_steps(sla_deadline) WHERE action IS NULL;

-- Notifications
CREATE INDEX idx_notifications_user_unread ON notifications(user_id, read, created_at DESC);

-- Documents
CREATE INDEX idx_documents_entity ON documents(entity_type, entity_id);

-- Tickets
CREATE INDEX idx_tickets_assigned ON tickets(assigned_to, status);
CREATE INDEX idx_tickets_created_by ON tickets(created_by);
```

### Database Migrations Strategy

**Migration Files:**
- `V001__initial_schema.sql` - Core tables (users, satkers, categories)
- `V002__bank_aset.sql` - Asset management tables
- `V003__kebutuhan_bmn.sql` - BMN needs planning
- `V004__pemakaian_bmn.sql` - Usage tracking
- `V005__penghapusan_bmn.sql` - Asset disposal
- `V006__pakaian_dinas.sql` - Uniform management
- `V007__workflow.sql` - Workflow engine
- `V008__dokumen.sql` - Document management
- `V009__notifikasi.sql` - Notifications
- `V010__bantuan.sql` - Help/ticket system
- `V011__indexes.sql` - Performance indexes
- `V012__audit_triggers.sql` - Audit logging

**Migration Execution:**
- Run as Kubernetes init container before main container starts
- Fail fast if migration fails
- Log migration progress
- Support rollback for failed migrations

## API Endpoint Structure

### REST API Design

**Base Path:** `/api/pembinaan/perlengkapan`

**Authentication:** All endpoints require JWT token in `Authorization: Bearer <token>` header

### Endpoint Groups

#### 1. Bank Aset (Asset Management)

```
GET    /api/pembinaan/perlengkapan/assets              # List assets (paginated, filtered)
POST   /api/pembinaan/perlengkapan/assets              # Create asset
GET    /api/pembinaan/perlengkapan/assets/:id          # Get asset details
PUT    /api/pembinaan/perlengkapan/assets/:id          # Update asset
DELETE /api/pembinaan/perlengkapan/assets/:id          # Delete asset
GET    /api/pembinaan/perlengkapan/assets/:id/history  # Asset history
POST   /api/pembinaan/perlengkapan/assets/bulk-import  # Bulk import from Excel
GET    /api/pembinaan/perlengkapan/assets/export       # Export to Excel
```

#### 2. Kebutuhan BMN (Needs Planning)

```
GET    /api/pembinaan/perlengkapan/kebutuhan           # List needs
POST   /api/pembinaan/perlengkapan/kebutuhan           # Create need
GET    /api/pembinaan/perlengkapan/kebutuhan/:id       # Get need details
PUT    /api/pembinaan/perlengkapan/kebutuhan/:id       # Update need
DELETE /api/pembinaan/perlengkapan/kebutuhan/:id       # Delete need
POST   /api/pembinaan/perlengkapan/kebutuhan/:id/submit # Submit for approval
```

#### 3. Pemakaian BMN (Usage Tracking)

```
GET    /api/pembinaan/perlengkapan/pemakaian           # List usage records
POST   /api/pembinaan/perlengkapan/pemakaian           # Record usage
GET    /api/pembinaan/perlengkapan/pemakaian/:id       # Get usage details
PUT    /api/pembinaan/perlengkapan/pemakaian/:id       # Update usage
POST   /api/pembinaan/perlengkapan/pemakaian/:id/return # Return asset
```

#### 4. Penghapusan BMN (Asset Disposal)

```
GET    /api/pembinaan/perlengkapan/penghapusan         # List disposal requests
POST   /api/pembinaan/perlengkapan/penghapusan         # Create disposal request
GET    /api/pembinaan/perlengkapan/penghapusan/:id     # Get disposal details
PUT    /api/pembinaan/perlengkapan/penghapusan/:id     # Update disposal
POST   /api/pembinaan/perlengkapan/penghapusan/:id/submit # Submit for approval
```

#### 5. Pakaian Dinas (Uniform Management)

```
GET    /api/pembinaan/perlengkapan/pakaian-dinas       # List uniforms
POST   /api/pembinaan/perlengkapan/pakaian-dinas       # Create uniform record
GET    /api/pembinaan/perlengkapan/pakaian-dinas/:id   # Get uniform details
PUT    /api/pembinaan/perlengkapan/pakaian-dinas/:id   # Update uniform
POST   /api/pembinaan/perlengkapan/pakaian-dinas/distribute # Distribute uniforms
```

#### 6. Workflow & Approval

```
GET    /api/pembinaan/perlengkapan/workflows           # List workflow definitions
GET    /api/pembinaan/perlengkapan/workflows/:id       # Get workflow definition
POST   /api/pembinaan/perlengkapan/workflows           # Create workflow definition
PUT    /api/pembinaan/perlengkapan/workflows/:id       # Update workflow definition

GET    /api/pembinaan/perlengkapan/workflow-instances  # List instances
GET    /api/pembinaan/perlengkapan/workflow-instances/:id # Get instance details
POST   /api/pembinaan/perlengkapan/workflow-instances/:id/approve # Approve step
POST   /api/pembinaan/perlengkapan/workflow-instances/:id/reject  # Reject step
POST   /api/pembinaan/perlengkapan/workflow-instances/:id/delegate # Delegate step

GET    /api/pembinaan/perlengkapan/approvals/pending   # My pending approvals
```

#### 7. Dashboard & Reporting

```
GET    /api/pembinaan/perlengkapan/dashboard           # Dashboard statistics
GET    /api/pembinaan/perlengkapan/dashboard/kpis      # Key performance indicators
GET    /api/pembinaan/perlengkapan/reports             # List available reports
POST   /api/pembinaan/perlengkapan/reports/generate    # Generate report
GET    /api/pembinaan/perlengkapan/reports/:id         # Download report
GET    /api/pembinaan/perlengkapan/charts/:type        # Chart data
```

#### 8. Admin & User Management

```
GET    /api/pembinaan/perlengkapan/admin/users         # List users
POST   /api/pembinaan/perlengkapan/admin/users         # Create user
GET    /api/pembinaan/perlengkapan/admin/users/:id     # Get user details
PUT    /api/pembinaan/perlengkapan/admin/users/:id     # Update user
DELETE /api/pembinaan/perlengkapan/admin/users/:id     # Delete user

GET    /api/pembinaan/perlengkapan/admin/roles         # List roles
POST   /api/pembinaan/perlengkapan/admin/roles         # Create role
PUT    /api/pembinaan/perlengkapan/admin/roles/:id     # Update role
```

#### 9. Document Management

```
GET    /api/pembinaan/perlengkapan/documents           # List documents
POST   /api/pembinaan/perlengkapan/documents           # Upload document
GET    /api/pembinaan/perlengkapan/documents/:id       # Get document metadata
GET    /api/pembinaan/perlengkapan/documents/:id/download # Download document
DELETE /api/pembinaan/perlengkapan/documents/:id       # Delete document
GET    /api/pembinaan/perlengkapan/documents/search    # Search documents

POST   /api/pembinaan/perlengkapan/documents/generate-pdf # Generate PDF from template
POST   /api/pembinaan/perlengkapan/documents/generate-excel # Generate Excel report
```

#### 10. Notifications

```
GET    /api/pembinaan/perlengkapan/notifications       # List notifications
GET    /api/pembinaan/perlengkapan/notifications/unread-count # Unread count
PUT    /api/pembinaan/perlengkapan/notifications/:id/read # Mark as read
PUT    /api/pembinaan/perlengkapan/notifications/read-all # Mark all as read
GET    /api/pembinaan/perlengkapan/notifications/preferences # Get preferences
PUT    /api/pembinaan/perlengkapan/notifications/preferences # Update preferences

WS     /api/pembinaan/perlengkapan/notifications/ws    # WebSocket for real-time
```

#### 11. Help/Ticket System

```
GET    /api/pembinaan/perlengkapan/tickets             # List tickets
POST   /api/pembinaan/perlengkapan/tickets             # Create ticket
GET    /api/pembinaan/perlengkapan/tickets/:id         # Get ticket details
PUT    /api/pembinaan/perlengkapan/tickets/:id         # Update ticket
POST   /api/pembinaan/perlengkapan/tickets/:id/comments # Add comment
POST   /api/pembinaan/perlengkapan/tickets/:id/close   # Close ticket

GET    /api/pembinaan/perlengkapan/faq                 # List FAQs
GET    /api/pembinaan/perlengkapan/faq/:id             # Get FAQ details
GET    /api/pembinaan/perlengkapan/faq/search          # Search FAQs

GET    /api/pembinaan/perlengkapan/kb                  # List knowledge base articles
GET    /api/pembinaan/perlengkapan/kb/:id              # Get article
```

#### 12. Health & Monitoring

```
GET    /health                                          # Health check
GET    /health/ready                                    # Readiness probe
GET    /health/live                                     # Liveness probe
GET    /metrics                                         # Prometheus metrics
```

### API Response Format

**Success Response:**

```json
{
  "data": {
    "id": "uuid",
    "field1": "value1",
    "field2": "value2"
  }
}
```

**List Response (Paginated):**

```json
{
  "data": [
    { "id": "uuid1", "..." },
    { "id": "uuid2", "..." }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 100,
    "total_pages": 5
  }
}
```

**Error Response:**

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input data",
    "details": {
      "field": "kode_barang",
      "reason": "must be between 1 and 50 characters"
    }
  }
}
```

### Request/Response Examples

**Create Asset:**

```http
POST /api/pembinaan/perlengkapan/assets
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "kode_barang": "BMN-2024-001",
  "nama": "Laptop Dell Latitude 5420",
  "kategori_id": "uuid",
  "satker_id": "uuid",
  "kondisi": "Baik",
  "nilai_perolehan": "15000000.00",
  "tanggal_perolehan": "2024-01-15",
  "lokasi": "Ruang IT Lantai 2"
}
```

**Response:**

```http
HTTP/1.1 201 Created
Content-Type: application/json

{
  "data": {
    "id": "uuid",
    "kode_barang": "BMN-2024-001",
    "nama": "Laptop Dell Latitude 5420",
    "kategori_id": "uuid",
    "satker_id": "uuid",
    "kondisi": "Baik",
    "nilai_perolehan": "15000000.00",
    "tanggal_perolehan": "2024-01-15",
    "lokasi": "Ruang IT Lantai 2",
    "status": "Aktif",
    "created_by": "uuid",
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

## Integration Points

### 1. Authenc Integration (gRPC)

**Purpose:** Authentication and authorization

**gRPC Service Definition:**

```protobuf
service AuthService {
  rpc ValidateToken(ValidateTokenRequest) returns (ValidateTokenResponse);
  rpc CheckPermission(CheckPermissionRequest) returns (CheckPermissionResponse);
  rpc GetUserInfo(GetUserInfoRequest) returns (GetUserInfoResponse);
}

message ValidateTokenRequest {
  string token = 1;
}

message ValidateTokenResponse {
  bool valid = 1;
  string user_id = 2;
  repeated string roles = 3;
  int64 expires_at = 4;
}

message CheckPermissionRequest {
  string user_id = 1;
  string permission = 2;
}

message CheckPermissionResponse {
  bool allowed = 1;
}
```

**Usage Pattern:**

```rust
// In auth middleware
let token_info = state.authenc_client
    .validate_token(&token)
    .await?;

// In service layer
let allowed = state.authenc_client
    .check_permission(user_id, "asset:create")
    .await?;
```

### 2. Secreton Integration (gRPC)

**Purpose:** Secrets management

**Authentication Flow:**
1. Pod starts with projected service account token (audience: `secreton`)
2. Exchange SA token for Secreton client token via Kubernetes auth backend
3. Use client token to fetch secrets from vault

**Secrets Path Structure:**

```
kv/data/postgres/perlengkapan     # Database credentials
kv/data/redis/perlengkapan        # Redis credentials
kv/data/s3/perlengkapan           # S3 credentials
kv/data/smtp/perlengkapan         # Email credentials
kv/data/sms/perlengkapan          # SMS gateway credentials
```

**Usage Pattern:**

```rust
// At startup
let secreton_client = SecretonClient::new_with_k8s_auth(
    &config.secreton_endpoint,
    "/var/run/secrets/tokens/secreton-token",
    "layanan-perlengkapan"
).await?;

let db_password = secreton_client
    .get_secret("kv/data/postgres/perlengkapan", "password")
    .await?;
```

### 3. Integrasi Service Integration (gRPC)

**Purpose:** Sync with external systems (MySIMKARI, SIMAN)

**gRPC Service Definition:**

```protobuf
service IntegrasiService {
  rpc SyncAssetToMySIMKARI(SyncAssetRequest) returns (SyncAssetResponse);
  rpc SyncAssetToSIMAN(SyncAssetRequest) returns (SyncAssetResponse);
  rpc GetSyncStatus(GetSyncStatusRequest) returns (GetSyncStatusResponse);
}

message SyncAssetRequest {
  string asset_id = 1;
  string target_system = 2;
}

message SyncAssetResponse {
  bool success = 1;
  string external_id = 2;
  string message = 3;
}
```

**Usage Pattern:**

```rust
// After asset creation/update
let sync_result = state.integrasi_client
    .sync_asset_to_mysimkari(&asset.id)
    .await?;

if !sync_result.success {
    tracing::warn!("Failed to sync asset to MySIMKARI: {}", sync_result.message);
}
```

### 4. Frontend Integration (REST API)

**Communication Pattern:**
- Frontend: Leptos WASM microfrontend
- Protocol: REST API (JSON/HTTP)
- Authentication: JWT in Authorization header
- Real-time: WebSocket for notifications

**Frontend API Client:**

```rust
// In Leptos frontend
use gloo_net::http::Request;

pub async fn create_asset(token: &str, request: CreateAssetRequest) -> Result<AssetResponse> {
    let response = Request::post("/api/pembinaan/perlengkapan/assets")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if response.ok() {
        Ok(response.json().await?)
    } else {
        Err(parse_error_response(response).await)
    }
}
```

**WebSocket Integration:**

```rust
// Frontend WebSocket connection
let ws = WebSocket::new("/api/pembinaan/perlengkapan/notifications/ws")?;

ws.set_onmessage(Some(Closure::wrap(Box::new(move |e: MessageEvent| {
    if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
        let notification: Notification = serde_json::from_str(&txt.as_string().unwrap()).unwrap();
        // Update UI with new notification
    }
}))));
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Properties Reflection

After analyzing all acceptance criteria, the following properties were identified as suitable for property-based testing. Properties have been consolidated to eliminate redundancy:

**Consolidated Properties:**
- JSON serialization round-trip (21.4) subsumes individual parse (21.1) and serialize (21.2) tests
- Excel serialization round-trip (21.10) subsumes individual parse (21.8) and serialize (21.9) tests
- RBAC permission checking (6.6) covers the general case; specific 403 response (6.7) is an example test
- Workflow state transitions (14.3) covers the general invariant; specific actions (14.4) are examples
- Transaction handling (3.6) is the comprehensive property; individual query patterns are implementation details

**Properties Excluded from PBT:**
- Infrastructure setup (database pool, gRPC clients) - SMOKE tests
- External service behavior (Authenc validation) - INTEGRATION tests with mocks
- Structural requirements (module organization, crate structure) - SMOKE tests
- One-time checks (compilation, configuration parsing) - SMOKE tests

### Property 1: JSON Serialization Round-Trip

*For any* valid data model (Asset, WorkflowInstance, Document, Notification, Ticket), serializing to JSON then deserializing SHALL produce an equivalent object.

**Validates: Requirements 21.1, 21.2, 21.3, 21.4**

**Rationale:** This property ensures data integrity across the API boundary. If serialization is not bijective, data corruption can occur when frontend and backend exchange information.

**Test Implementation:**

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn asset_json_round_trip(asset in arb_asset()) {
            let json = serde_json::to_string(&asset).unwrap();
            let deserialized: Asset = serde_json::from_str(&json).unwrap();
            assert_eq!(asset, deserialized);
        }
    }
}
```

### Property 2: Excel Data Round-Trip

*For any* valid tabular data (asset list, usage records, disposal requests), exporting to Excel then importing SHALL produce equivalent data.

**Validates: Requirements 21.8, 21.9, 21.10**

**Rationale:** Excel import/export is critical for bulk operations. Data loss or corruption during round-trip would cause serious operational issues.

**Test Implementation:**

```rust
proptest! {
    #[test]
    fn excel_round_trip(assets in prop::collection::vec(arb_asset(), 1..100)) {
        let excel_bytes = export_to_excel(&assets).unwrap();
        let imported = import_from_excel(&excel_bytes).unwrap();
        assert_eq!(assets, imported);
    }
}
```

### Property 3: Configuration Loading Preserves Values

*For any* valid configuration structure, loading from environment then serializing SHALL preserve all configuration values.

**Validates: Requirements 1.7, 7.1, 7.2, 7.4**

**Rationale:** Configuration errors can cause runtime failures or security issues. This property ensures configuration is correctly loaded and accessible.

**Test Implementation:**

```rust
proptest! {
    #[test]
    fn config_round_trip(config in arb_config()) {
        let env_vars = config.to_env_vars();
        set_env_vars(&env_vars);
        let loaded = Config::load().unwrap();
        assert_eq!(config, loaded);
    }
}
```

### Property 4: JWT Token Extraction

*For any* valid Authorization header format (`Bearer <token>`), extracting the token SHALL succeed and return the correct token value.

**Validates: Requirements 6.2, 6.5**

**Rationale:** Token extraction is the first step in authentication. Incorrect extraction would break all authenticated requests.

**Test Implementation:**

```rust
proptest! {
    #[test]
    fn jwt_extraction(token in "[a-zA-Z0-9._-]{20,500}") {
        let header = format!("Bearer {}", token);
        let extracted = extract_jwt_from_header(&header).unwrap();
        assert_eq!(extracted, token);
    }
}
```

### Property 5: RBAC Permission Consistency

*For any* user with a given set of roles, permission checks SHALL be consistent—the same user/permission combination SHALL always return the same result within a single request context.

**Validates: Requirements 6.6, 6.9**

**Rationale:** Inconsistent permission checks would create security vulnerabilities. Authorization decisions must be deterministic.

**Test Implementation:**

```rust
proptest! {
    #[test]
    fn rbac_consistency(
        user_id in arb_uuid(),
        roles in prop::collection::vec(arb_role(), 1..5),
        permission in arb_permission()
    ) {
        let result1 = check_permission(user_id, &roles, &permission);
        let result2 = check_permission(user_id, &roles, &permission);
        assert_eq!(result1, result2);
    }
}
```

### Property 6: Authentication Failure Logging

*For any* authentication or authorization failure, a log entry SHALL be created with appropriate context (user_id, reason, timestamp).

**Validates: Requirements 6.10, 8.6, 8.7**

**Rationale:** Security audit trails are critical for compliance and incident response. Missing logs would prevent security analysis.

**Test Implementation:**

```rust
proptest! {
    #[test]
    fn auth_failure_logged(
        user_id in arb_uuid(),
        failure_reason in arb_auth_failure_reason()
    ) {
        let log_capture = capture_logs();
        simulate_auth_failure(user_id, failure_reason);
        let logs = log_capture.finish();

        assert!(logs.iter().any(|log|
            log.contains(&user_id.to_string()) &&
            log.contains(&failure_reason.to_string())
        ));
    }
}
```

### Property 7: Workflow State Invariants

*For any* workflow instance, state transitions SHALL preserve invariants: (1) current_step must be valid for the workflow definition, (2) status must match step completion state, (3) timestamps must be monotonically increasing.

**Validates: Requirements 14.1, 14.2, 14.3, 14.7**

**Rationale:** Workflow state corruption would cause approval processes to fail or skip required steps, violating business rules.

**Test Implementation:**

```rust
proptest! {
    #[test]
    fn workflow_state_invariants(
        definition in arb_workflow_definition(),
        actions in prop::collection::vec(arb_approval_action(), 1..20)
    ) {
        let mut instance = WorkflowInstance::new(definition);

        for action in actions {
            if let Ok(_) = instance.apply_action(action) {
                // Invariant 1: current step is valid
                assert!(definition.has_step(&instance.current_step));

                // Invariant 2: status matches completion
                if instance.all_steps_completed() {
                    assert!(matches!(instance.status, WorkflowStatus::Approved | WorkflowStatus::Rejected));
                }

                // Invariant 3: timestamps are monotonic
                if let Some(completed_at) = instance.completed_at {
                    assert!(completed_at >= instance.initiated_at);
                }
            }
        }
    }
}
```

### Property 8: Workflow Conditional Routing

*For any* workflow definition with conditional routing rules and any input data, the routing decision SHALL be deterministic—the same input SHALL always route to the same next step.

**Validates: Requirements 14.8**

**Rationale:** Non-deterministic routing would cause unpredictable approval flows, violating business process requirements.

**Test Implementation:**

```rust
proptest! {
    #[test]
    fn workflow_routing_deterministic(
        definition in arb_workflow_definition_with_conditionals(),
        input_data in arb_workflow_input()
    ) {
        let next_step1 = definition.evaluate_routing(&input_data);
        let next_step2 = definition.evaluate_routing(&input_data);
        assert_eq!(next_step1, next_step2);
    }
}
```

### Property 9: Workflow Notification Triggering

*For any* workflow step requiring approval, a notification SHALL be sent to the assigned approver when the step becomes active.

**Validates: Requirements 14.6**

**Rationale:** Missing notifications would cause approval delays and SLA breaches. Every approval requirement must trigger notification.

**Test Implementation:**

```rust
proptest! {
    #[test]
    fn workflow_approval_notification(
        workflow in arb_workflow_with_approval_steps(),
        approver_id in arb_uuid()
    ) {
        let notification_capture = capture_notifications();

        workflow.activate_approval_step(approver_id);

        let notifications = notification_capture.finish();
        assert!(notifications.iter().any(|n|
            n.user_id == approver_id &&
            n.notification_type == NotificationType::WorkflowApproval
        ));
    }
}
```

### Property 10: Database Transaction Atomicity

*For any* multi-step database operation, if any step fails, ALL changes SHALL be rolled back—the database state SHALL be unchanged from before the transaction started.

**Validates: Requirements 3.6**

**Rationale:** Partial updates would corrupt data integrity. Transactions must be atomic (all-or-nothing).

**Test Implementation:**

```rust
proptest! {
    #[test]
    fn transaction_atomicity(
        operations in prop::collection::vec(arb_db_operation(), 2..10),
        failure_index in 0..10usize
    ) {
        let initial_state = capture_db_state();

        let result = execute_transaction(|tx| {
            for (i, op) in operations.iter().enumerate() {
                if i == failure_index {
                    return Err(AppError::Internal("Simulated failure".into()));
                }
                op.execute(tx)?;
            }
            Ok(())
        });

        if result.is_err() {
            let final_state = capture_db_state();
            assert_eq!(initial_state, final_state);
        }
    }
}
```

### Property 11: Redis Cache Consistency

*For any* cacheable data, a cache hit SHALL return the same data as a cache miss (database query)—caching SHALL be transparent to the caller.

**Validates: Requirements 16.2**

**Rationale:** Cache inconsistency would cause users to see stale or incorrect data. Cache must be transparent.

**Test Implementation:**

```rust
proptest! {
    #[test]
    fn cache_consistency(
        cache_key in "[a-z:]{5,50}",
        data in arb_cacheable_data()
    ) {
        // Cache miss - fetch from DB
        cache.delete(&cache_key);
        let from_db = fetch_data(&cache_key).unwrap();

        // Cache hit - fetch from cache
        cache.set(&cache_key, &data);
        let from_cache = fetch_data(&cache_key).unwrap();

        assert_eq!(from_db, from_cache);
    }
}
```

### Property 12: Pagination Completeness

*For any* result set and pagination parameters, combining all pages SHALL produce the complete result set in the correct order.

**Validates: Requirements 16.4**

**Rationale:** Pagination bugs would cause data loss or duplication in list views. All data must be accessible through pagination.

**Test Implementation:**

```rust
proptest! {
    #[test]
    fn pagination_completeness(
        total_items in 1..1000usize,
        page_size in 1..100usize
    ) {
        let all_items = generate_test_items(total_items);

        let mut paginated_items = Vec::new();
        let mut page = 1;

        loop {
            let page_result = fetch_page(page, page_size);
            if page_result.data.is_empty() {
                break;
            }
            paginated_items.extend(page_result.data);
            page += 1;
        }

        assert_eq!(all_items, paginated_items);
    }
}
```

### Property 13: Stateless Request Handling

*For any* API request, the response SHALL be the same regardless of which backend instance handles the request (assuming no concurrent modifications).

**Validates: Requirements 16.8**

**Rationale:** Stateful behavior would break horizontal scaling and cause inconsistent user experience.

**Test Implementation:**

```rust
proptest! {
    #[test]
    fn stateless_request_handling(
        request in arb_api_request(),
        instance_count in 2..5usize
    ) {
        let instances = create_test_instances(instance_count);

        let responses: Vec<_> = instances.iter()
            .map(|instance| instance.handle_request(&request))
            .collect();

        // All responses should be identical
        for response in &responses[1..] {
            assert_eq!(responses[0], *response);
        }
    }
}
```

### Property 14: Input Validation Consistency

*For any* input data violating validation rules, validation SHALL fail with a descriptive error message identifying the violated constraint.

**Validates: Requirements 21.5, 21.6, 21.7**

**Rationale:** Inconsistent validation would allow invalid data into the system, causing data corruption or business rule violations.

**Test Implementation:**

```rust
proptest! {
    #[test]
    fn validation_rejects_invalid_input(
        invalid_input in arb_invalid_asset_request()
    ) {
        let result = invalid_input.validate();

        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(!error.to_string().is_empty());
        assert!(error.to_string().len() > 10); // Descriptive, not just "error"
    }
}
```

## Error Handling

### Error Handling Strategy

**Principles:**
1. **Fail Fast**: Detect errors early and return immediately
2. **Descriptive Errors**: Provide context for debugging
3. **Type Safety**: Use Result types, never panic in production
4. **Structured Errors**: Use thiserror for error definitions
5. **Logging**: Log all errors with context
6. **User-Friendly**: Return appropriate HTTP status codes and messages

### Error Type Hierarchy

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),

    #[error("Database pool error: {0}")]
    DatabasePool(#[from] deadpool_postgres::PoolError),

    // gRPC errors
    #[error("gRPC communication error: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("Authenc service error: {0}")]
    AuthencError(String),

    #[error("Secreton service error: {0}")]
    SecretonError(String),

    // Validation errors
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Business rule violation: {0}")]
    BusinessRule(String),

    // Authentication/Authorization errors
    #[error("Authentication failed: {0}")]
    Unauthorized(String),

    #[error("Permission denied: {0}")]
    Forbidden(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    // Resource errors
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Resource conflict: {0}")]
    Conflict(String),

    // External service errors
    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Integration error: {0}")]
    Integration(String),

    // File/Storage errors
    #[error("File operation error: {0}")]
    FileOperation(String),

    #[error("Storage error: {0}")]
    Storage(String),

    // Cache errors
    #[error("Cache error: {0}")]
    Cache(String),

    // Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    // Internal errors
    #[error("Internal server error: {0}")]
    Internal(String),
}
```

### HTTP Status Code Mapping

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message, log_level) = match &self {
            AppError::Unauthorized(_) => (
                StatusCode::UNAUTHORIZED,
                "AUTH_FAILED",
                self.to_string(),
                Level::WARN,
            ),
            AppError::Forbidden(_) => (
                StatusCode::FORBIDDEN,
                "PERMISSION_DENIED",
                self.to_string(),
                Level::WARN,
            ),
            AppError::InvalidToken(_) => (
                StatusCode::UNAUTHORIZED,
                "INVALID_TOKEN",
                self.to_string(),
                Level::WARN,
            ),
            AppError::NotFound(_) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                self.to_string(),
                Level::INFO,
            ),
            AppError::Validation(_) | AppError::BusinessRule(_) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                self.to_string(),
                Level::INFO,
            ),
            AppError::Conflict(_) => (
                StatusCode::CONFLICT,
                "CONFLICT",
                self.to_string(),
                Level::INFO,
            ),
            AppError::ExternalService(_) | AppError::Integration(_) => (
                StatusCode::BAD_GATEWAY,
                "EXTERNAL_SERVICE_ERROR",
                "External service unavailable".to_string(),
                Level::ERROR,
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error".to_string(),
                Level::ERROR,
            ),
        };

        // Log error with appropriate level
        match log_level {
            Level::ERROR => tracing::error!("{}", self),
            Level::WARN => tracing::warn!("{}", self),
            Level::INFO => tracing::info!("{}", self),
            _ => {}
        }

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}
```

### Error Context Pattern

```rust
use anyhow::Context;

pub async fn get_asset(id: Uuid) -> Result<Asset, AppError> {
    let asset = repository
        .find_by_id(id)
        .await
        .context(format!("Failed to fetch asset {}", id))?
        .ok_or_else(|| AppError::NotFound(format!("Asset {} not found", id)))?;

    Ok(asset)
}
```

### Retry Logic for Transient Errors

```rust
use tokio::time::{sleep, Duration};

pub async fn call_with_retry<F, T, E>(
    mut f: F,
    max_retries: u32,
) -> Result<T, E>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>>>>,
    E: std::fmt::Display,
{
    let mut attempts = 0;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                let backoff = Duration::from_millis(100 * 2u64.pow(attempts));
                tracing::warn!("Attempt {} failed: {}. Retrying in {:?}", attempts, e, backoff);
                sleep(backoff).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Circuit Breaker Pattern

```rust
pub struct CircuitBreaker {
    failure_threshold: u32,
    timeout: Duration,
    state: Arc<Mutex<CircuitState>>,
}

enum CircuitState {
    Closed { failures: u32 },
    Open { opened_at: Instant },
    HalfOpen,
}

impl CircuitBreaker {
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>>,
    {
        let mut state = self.state.lock().await;

        match *state {
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() > self.timeout {
                    *state = CircuitState::HalfOpen;
                } else {
                    return Err(/* circuit open error */);
                }
            }
            _ => {}
        }

        drop(state);

        match f.await {
            Ok(result) => {
                let mut state = self.state.lock().await;
                *state = CircuitState::Closed { failures: 0 };
                Ok(result)
            }
            Err(e) => {
                let mut state = self.state.lock().await;
                match *state {
                    CircuitState::Closed { failures } => {
                        if failures + 1 >= self.failure_threshold {
                            *state = CircuitState::Open { opened_at: Instant::now() };
                        } else {
                            *state = CircuitState::Closed { failures: failures + 1 };
                        }
                    }
                    CircuitState::HalfOpen => {
                        *state = CircuitState::Open { opened_at: Instant::now() };
                    }
                    _ => {}
                }
                Err(e)
            }
        }
    }
}
```

## Testing Strategy

### Testing Pyramid

```
                    /\
                   /  \
                  / E2E \
                 /--------\
                /          \
               / Integration \
              /--------------\
             /                \
            /   Unit + Property \
           /--------------------\
```

### 1. Unit Tests

**Scope:** Individual functions and methods

**Coverage Target:** 80% minimum

**Tools:**
- Standard Rust `#[test]` attribute
- `mockall` for mocking dependencies
- `tokio-test` for async tests

**Example:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_validation() {
        let request = CreateAssetRequest {
            kode_barang: "".to_string(), // Invalid: empty
            nama: "Test Asset".to_string(),
            kategori_id: Uuid::new_v4(),
            satker_id: Uuid::new_v4(),
        };

        assert!(request.validate().is_err());
    }

    #[tokio::test]
    async fn test_asset_repository_create() {
        let pool = create_test_pool().await;
        let repo = AssetRepository::new(pool);

        let request = create_valid_asset_request();
        let asset = repo.create(&request, Uuid::new_v4()).await.unwrap();

        assert_eq!(asset.kode_barang, request.kode_barang);
    }
}
```

### 2. Property-Based Tests

**Scope:** Universal properties that should hold for all inputs

**Minimum Iterations:** 100 per property

**Tools:**
- `proptest` for property-based testing
- Custom generators for domain models

**Tag Format:** `Feature: perlengkapan-unified-refactoring, Property {number}: {property_text}`

**Example:**

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    // Feature: perlengkapan-unified-refactoring, Property 1: JSON Serialization Round-Trip
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn asset_json_round_trip(asset in arb_asset()) {
            let json = serde_json::to_string(&asset).unwrap();
            let deserialized: Asset = serde_json::from_str(&json).unwrap();
            assert_eq!(asset, deserialized);
        }
    }

    // Arbitrary generator for Asset
    fn arb_asset() -> impl Strategy<Value = Asset> {
        (
            any::<Uuid>(),
            "[A-Z]{3}-[0-9]{4}-[0-9]{3}",
            "[a-zA-Z ]{5,100}",
            any::<Uuid>(),
            any::<Uuid>(),
            prop_oneof![
                Just(AssetCondition::Baik),
                Just(AssetCondition::RusakRingan),
                Just(AssetCondition::RusakBerat),
            ],
        ).prop_map(|(id, kode, nama, kategori_id, satker_id, kondisi)| {
            Asset {
                id,
                kode_barang: kode,
                nama,
                kategori_id,
                satker_id,
                kondisi,
                // ... other fields
            }
        })
    }
}
```

### 3. Integration Tests

**Scope:** API endpoints, database operations, gRPC clients

**Tools:**
- `axum-test` for HTTP testing
- Test database with fixtures
- Mock gRPC servers

**Example:**

```rust
#[cfg(test)]
mod integration_tests {
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_create_asset_endpoint() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let response = server
            .post("/api/pembinaan/perlengkapan/assets")
            .add_header("Authorization", "Bearer test_token")
            .json(&json!({
                "kode_barang": "BMN-2024-001",
                "nama": "Test Asset",
                "kategori_id": "uuid",
                "satker_id": "uuid",
            }))
            .await;

        response.assert_status_created();
        response.assert_json_contains(&json!({
            "data": {
                "kode_barang": "BMN-2024-001"
            }
        }));
    }
}
```

### 4. Contract Tests

**Scope:** API stability, backward compatibility

**Tools:**
- JSON schema validation
- API versioning tests

**Example:**

```rust
#[test]
fn test_asset_response_schema() {
    let response = AssetResponse {
        id: Uuid::new_v4(),
        kode_barang: "BMN-2024-001".to_string(),
        // ... other fields
    };

    let json = serde_json::to_value(&response).unwrap();

    // Verify required fields exist
    assert!(json.get("id").is_some());
    assert!(json.get("kode_barang").is_some());
    assert!(json.get("nama").is_some());

    // Verify field types
    assert!(json["id"].is_string());
    assert!(json["kode_barang"].is_string());
}
```

### 5. Load Tests

**Scope:** Performance validation, scalability testing

**Tools:**
- `criterion` for benchmarking
- Custom load testing scripts

**Targets:**
- 95th percentile response time < 200ms
- Support 1000 concurrent users
- Handle 10,000 requests per minute

**Example:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_asset_list(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = rt.block_on(create_test_app());

    c.bench_function("list_assets", |b| {
        b.to_async(&rt).iter(|| async {
            let response = app
                .get("/api/pembinaan/perlengkapan/assets")
                .await;
            black_box(response)
        });
    });
}

criterion_group!(benches, benchmark_asset_list);
criterion_main!(benches);
```

### 6. End-to-End Tests

**Scope:** Complete user workflows

**Tools:**
- Playwright (for frontend E2E)
- API E2E tests

**Example Scenarios:**
- User creates asset → submits for approval → approver approves → asset becomes active
- User uploads document → document is validated and stored
- User creates ticket → support staff responds → ticket is resolved

### Test Data Management

**Fixtures:**

```rust
pub fn create_test_asset() -> Asset {
    Asset {
        id: Uuid::new_v4(),
        kode_barang: "TEST-001".to_string(),
        nama: "Test Asset".to_string(),
        kategori_id: Uuid::new_v4(),
        satker_id: Uuid::new_v4(),
        kondisi: AssetCondition::Baik,
        nilai_perolehan: BigDecimal::from(1000000),
        tanggal_perolehan: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        lokasi: "Test Location".to_string(),
        status: AssetStatus::Aktif,
        created_by: Uuid::new_v4(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
```

**Database Fixtures:**

```rust
pub async fn setup_test_database() -> Pool {
    let pool = create_test_pool().await;

    // Run migrations
    run_migrations(&pool).await.unwrap();

    // Insert test data
    insert_test_categories(&pool).await;
    insert_test_satkers(&pool).await;
    insert_test_users(&pool).await;

    pool
}

pub async fn cleanup_test_database(pool: &Pool) {
    // Truncate all tables
    let client = pool.get().await.unwrap();
    client.execute("TRUNCATE TABLE assets CASCADE", &[]).await.unwrap();
    // ... truncate other tables
}
```

### CI/CD Integration

**GitHub Actions Workflow:**

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: self-hosted

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run unit tests
        run: cargo test -p layanan-perlengkapan --lib

      - name: Run integration tests
        run: cargo test -p layanan-perlengkapan --test '*'
        env:
          DATABASE_URL: postgres://postgres:test@localhost/test

      - name: Run property-based tests
        run: cargo test -p layanan-perlengkapan property_tests

      - name: Generate coverage report
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --output-dir coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### Test Coverage Requirements

**Minimum Coverage by Component:**
- Handlers: 90%
- Services: 85%
- Repositories: 80%
- Models: 70%
- Overall: 80%

**Coverage Exclusions:**
- Generated code (proto files)
- Test utilities
- Main entry point

## Deployment Architecture

### Container Strategy

**Multi-Stage Dockerfile:**

```dockerfile
# Stage 1: Build
FROM rust:1.95-slim as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY layanan/perlengkapan ./layanan/perlengkapan
COPY lib ./lib

# Build release binary
RUN cargo build --release -p layanan-perlengkapan

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 perlengkapan

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/layanan-perlengkapan /app/

# Copy migrations
COPY layanan/perlengkapan/migrations /app/migrations

USER perlengkapan

EXPOSE 8080

CMD ["/app/layanan-perlengkapan"]
```

### Kubernetes Deployment

**Deployment Manifest:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: layanan-perlengkapan
  namespace: simpel
spec:
  replicas: 3
  selector:
    matchLabels:
      app: layanan-perlengkapan
  template:
    metadata:
      labels:
        app: layanan-perlengkapan
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: layanan-perlengkapan

      # Init container for migrations
      initContainers:
        - name: migrations
          image: ghcr.io/analisaperlengkapan/simpel2/layanan-perlengkapan:v0.1.0
          command: ["/app/layanan-perlengkapan", "migrate"]
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: perlengkapan-secrets
                  key: database-url

      containers:
        - name: perlengkapan
          image: ghcr.io/analisaperlengkapan/simpel2/layanan-perlengkapan:v0.1.0
          ports:
            - containerPort: 8080
              name: http

          env:
            - name: RUST_LOG
              value: "info,layanan_perlengkapan=debug"
            - name: SERVER_HOST
              value: "0.0.0.0"
            - name: SERVER_PORT
              value: "8080"
            - name: AUTHENC_ENDPOINT
              value: "https://authenc.simpel.svc.cluster.local:50051"
            - name: SECRETON_ENDPOINT
              value: "https://secreton.simpel.svc.cluster.local:8200"
            - name: SECRETON_AUTH_METHOD
              value: "kubernetes"
            - name: SECRETON_AUTH_ROLE
              value: "layanan-perlengkapan"
            - name: SECRETON_K8S_TOKEN_PATH
              value: "/var/run/secrets/tokens/secreton-token"

          volumeMounts:
            - name: secreton-token
              mountPath: /var/run/secrets/tokens
              readOnly: true

          resources:
            requests:
              cpu: 500m
              memory: 512Mi
            limits:
              cpu: 2000m
              memory: 2Gi

          livenessProbe:
            httpGet:
              path: /health/live
              port: 8080
            initialDelaySeconds: 30
            periodSeconds: 10
            timeoutSeconds: 5
            failureThreshold: 3

          readinessProbe:
            httpGet:
              path: /health/ready
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 5
            timeoutSeconds: 3
            failureThreshold: 3

      volumes:
        - name: secreton-token
          projected:
            sources:
              - serviceAccountToken:
                  path: secreton-token
                  expirationSeconds: 3600
                  audience: secreton
```

**Service Manifest:**

```yaml
apiVersion: v1
kind: Service
metadata:
  name: layanan-perlengkapan
  namespace: simpel
spec:
  selector:
    app: layanan-perlengkapan
  ports:
    - name: http
      port: 80
      targetPort: 8080
  type: ClusterIP
```

**HorizontalPodAutoscaler:**

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: layanan-perlengkapan
  namespace: simpel
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: layanan-perlengkapan
  minReplicas: 3
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
```

### Helm Chart Structure

```
infra/helm/simpel/
├── Chart.yaml
├── values.yaml
└── templates/
    ├── layanan-perlengkapan/
    │   ├── deployment.yaml
    │   ├── service.yaml
    │   ├── hpa.yaml
    │   ├── serviceaccount.yaml
    │   └── configmap.yaml
    └── ...
```

**Helm Values (values.yaml):**

```yaml
layananPerlengkapan:
  enabled: true
  replicaCount: 3

  image:
    repository: ghcr.io/analisaperlengkapan/simpel2/layanan-perlengkapan
    tag: v0.1.0
    pullPolicy: IfNotPresent

  resources:
    requests:
      cpu: 500m
      memory: 512Mi
    limits:
      cpu: 2000m
      memory: 2Gi

  autoscaling:
    enabled: true
    minReplicas: 3
    maxReplicas: 10
    targetCPUUtilizationPercentage: 70
    targetMemoryUtilizationPercentage: 80

  secretonAuth:
    enabled: true
    role: layanan-perlengkapan
    policies:
      - kv/data/postgres/perlengkapan
      - kv/data/redis/perlengkapan
      - kv/data/s3/perlengkapan
      - kv/data/smtp/perlengkapan

  ingress:
    enabled: true
    className: nginx
    annotations:
      cert-manager.io/cluster-issuer: letsencrypt-prod
    hosts:
      - host: simpel.kejaksaan.go.id
        paths:
          - path: /api/pembinaan/perlengkapan
            pathType: Prefix
    tls:
      - secretName: simpel-tls
        hosts:
          - simpel.kejaksaan.go.id
```

### Health Checks

**Liveness Probe:**

```rust
pub async fn health_live() -> impl IntoResponse {
    // Basic health check - is the service running?
    StatusCode::OK
}
```

**Readiness Probe:**

```rust
pub async fn health_ready(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Check database connection
    state.db_pool.health_check().await?;

    // Check gRPC clients
    state.authenc_client.health_check().await?;
    state.secreton_client.health_check().await?;

    // Check Redis
    state.cache.ping().await?;

    Ok(StatusCode::OK)
}
```

### Graceful Shutdown

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // ... initialization ...

    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal());

    tracing::info!("Server listening on {}", addr);

    server.await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down gracefully");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, shutting down gracefully");
        },
    }

    // Give in-flight requests time to complete
    tokio::time::sleep(Duration::from_secs(10)).await;
}
```

## Security Considerations

### Zero-Trust Security Model

**Principles:**
1. **Never Trust, Always Verify**: Validate every request
2. **Least Privilege**: Grant minimum necessary permissions
3. **Defense in Depth**: Multiple layers of security
4. **Audit Everything**: Log all security-relevant events

### Authentication Flow

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant Perlengkapan
    participant Authenc
    participant Secreton

    User->>Frontend: Login
    Frontend->>Authenc: Authenticate (REST)
    Authenc-->>Frontend: JWT Token
    Frontend->>Frontend: Store in localStorage

    Frontend->>Perlengkapan: API Request + JWT
    Perlengkapan->>Authenc: ValidateToken (gRPC)
    Authenc-->>Perlengkapan: TokenInfo (user_id, roles)
    Perlengkapan->>Authenc: CheckPermission (gRPC)
    Authenc-->>Perlengkapan: Allowed/Denied

    alt Authorized
        Perlengkapan->>Perlengkapan: Process Request
        Perlengkapan-->>Frontend: Response
    else Unauthorized
        Perlengkapan-->>Frontend: 401/403 Error
    end
```

### Secret Management

**Secreton Integration:**

```rust
pub async fn fetch_secrets_from_secreton() -> Result<Secrets> {
    // Authenticate using Kubernetes service account token
    let sa_token = tokio::fs::read_to_string("/var/run/secrets/tokens/secreton-token").await?;

    let secreton_client = SecretonClient::new(&config.secreton_endpoint);

    // Exchange SA token for Secreton client token
    let client_token = secreton_client
        .kubernetes_login(&sa_token, "layanan-perlengkapan")
        .await?;

    // Fetch secrets
    let db_secrets = secreton_client
        .get_secret(&client_token, "kv/data/postgres/perlengkapan")
        .await?;

    let redis_secrets = secreton_client
        .get_secret(&client_token, "kv/data/redis/perlengkapan")
        .await?;

    Ok(Secrets {
        database_url: db_secrets.get("url")?,
        redis_url: redis_secrets.get("url")?,
        // ... other secrets
    })
}
```

**Secret Rotation:**
- Secrets fetched at startup
- Support hot-reload for non-critical secrets
- Database credentials rotated via Secreton
- JWT signing keys rotated periodically

### Input Validation

**Validation Layers:**
1. **Schema Validation**: JSON structure validation
2. **Type Validation**: Rust type system
3. **Business Validation**: Custom validation rules
4. **SQL Injection Prevention**: Prepared statements only

**Example:**

```rust
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAssetRequest {
    #[validate(length(min = 1, max = 50))]
    #[validate(regex = "^[A-Z]{3}-[0-9]{4}-[0-9]{3}$")]
    pub kode_barang: String,

    #[validate(length(min = 1, max = 255))]
    pub nama: String,

    #[validate(custom = "validate_kategori_exists")]
    pub kategori_id: Uuid,

    #[validate(custom = "validate_satker_exists")]
    pub satker_id: Uuid,
}

fn validate_kategori_exists(kategori_id: &Uuid) -> Result<(), ValidationError> {
    // Check if kategori exists in database
    // Return error if not found
}
```

### SQL Injection Prevention

**Always Use Prepared Statements:**

```rust
// ✅ GOOD: Prepared statement
let row = client.query_one(
    "SELECT * FROM assets WHERE kode_barang = $1",
    &[&kode_barang]
).await?;

// ❌ BAD: String interpolation (NEVER DO THIS)
let query = format!("SELECT * FROM assets WHERE kode_barang = '{}'", kode_barang);
let row = client.query_one(&query, &[]).await?;
```

### CSRF Protection

**Token-Based CSRF Protection:**

```rust
pub async fn csrf_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    if matches!(req.method(), &Method::POST | &Method::PUT | &Method::DELETE) {
        let csrf_token = req.headers()
            .get("X-CSRF-Token")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing CSRF token".into()))?;

        // Validate CSRF token
        state.csrf_validator.validate(csrf_token)?;
    }

    Ok(next.run(req).await)
}
```

### Rate Limiting

**Redis-Backed Rate Limiter:**

```rust
pub struct RateLimiter {
    redis: RedisClient,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub async fn check_rate_limit(&self, key: &str) -> Result<bool, AppError> {
        let count: u32 = self.redis
            .incr(key)
            .await?;

        if count == 1 {
            self.redis.expire(key, self.window.as_secs() as usize).await?;
        }

        Ok(count <= self.max_requests)
    }
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    user: UserContext,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let key = format!("rate_limit:user:{}", user.user_id);

    if !state.rate_limiter.check_rate_limit(&key).await? {
        return Err(AppError::TooManyRequests("Rate limit exceeded".into()));
    }

    Ok(next.run(req).await)
}
```

### Data Encryption

**Encryption at Rest:**
- PostgreSQL: Transparent Data Encryption (TDE)
- S3: Server-side encryption (SSE-S3)
- Sensitive fields: Application-level encryption

**Encryption in Transit:**
- TLS 1.3 for all HTTP traffic
- mTLS for gRPC communication
- Certificate management via cert-manager

**Application-Level Encryption:**

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct FieldEncryption {
    cipher: Aes256Gcm,
}

impl FieldEncryption {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, AppError> {
        let nonce = Nonce::from_slice(b"unique nonce"); // Use proper nonce generation
        self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<String, AppError> {
        let nonce = Nonce::from_slice(b"unique nonce");
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| AppError::Internal(format!("Invalid UTF-8: {}", e)))
    }
}
```

### Audit Logging

**Security Event Logging:**

```rust
pub async fn log_security_event(
    event_type: SecurityEventType,
    user_id: Option<Uuid>,
    details: serde_json::Value,
) {
    tracing::warn!(
        event_type = ?event_type,
        user_id = ?user_id,
        details = ?details,
        "Security event"
    );

    // Also store in audit log table
    // INSERT INTO audit_log (event_type, user_id, details, timestamp) VALUES (...)
}

pub enum SecurityEventType {
    AuthenticationFailed,
    AuthorizationDenied,
    InvalidToken,
    RateLimitExceeded,
    SuspiciousActivity,
}
```

### Security Headers

**HTTP Security Headers:**

```rust
pub async fn security_headers_middleware(
    req: Request,
    next: Next,
) -> Response {
    let mut response = next.run(req).await;

    let headers = response.headers_mut();
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
    headers.insert("Content-Security-Policy", "default-src 'self'".parse().unwrap());

    response
}
```

## Performance Optimization

### Caching Strategy

**Cache Layers:**
1. **Application Cache**: In-memory LRU cache for hot data
2. **Redis Cache**: Distributed cache for shared data
3. **Database Query Cache**: PostgreSQL query result cache

**Cache Invalidation:**

```rust
pub struct CacheManager {
    redis: RedisClient,
    local_cache: Arc<Mutex<LruCache<String, Vec<u8>>>>,
}

impl CacheManager {
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        // Try local cache first
        if let Some(data) = self.local_cache.lock().await.get(key) {
            return Ok(Some(serde_json::from_slice(data)?));
        }

        // Try Redis
        if let Some(data) = self.redis.get(key).await? {
            // Populate local cache
            self.local_cache.lock().await.put(key.to_string(), data.clone());
            return Ok(Some(serde_json::from_slice(&data)?));
        }

        Ok(None)
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<()> {
        let data = serde_json::to_vec(value)?;

        // Set in Redis
        self.redis.set_ex(key, &data, ttl.as_secs() as usize).await?;

        // Set in local cache
        self.local_cache.lock().await.put(key.to_string(), data);

        Ok(())
    }

    pub async fn invalidate(&self, pattern: &str) -> Result<()> {
        // Clear from Redis
        let keys = self.redis.keys(pattern).await?;
        for key in keys {
            self.redis.del(&key).await?;
        }

        // Clear from local cache
        self.local_cache.lock().await.clear();

        Ok(())
    }
}
```

**Cache Keys:**

```
assets:list:{satker_id}:{page}:{per_page}
assets:detail:{asset_id}
workflow:definition:{workflow_id}
workflow:pending:{user_id}
dashboard:stats:{satker_id}:{date}
```

**TTL Strategy:**
- List endpoints: 5 minutes
- Detail endpoints: 15 minutes
- Dashboard stats: 1 minute
- Workflow definitions: 1 hour
- User permissions: 5 minutes

### Database Optimization

**Connection Pooling:**

```rust
pub struct DatabaseConfig {
    pub max_connections: u32,      // 20
    pub min_connections: u32,       // 5
    pub connection_timeout: Duration, // 30s
    pub idle_timeout: Duration,     // 10 minutes
    pub max_lifetime: Duration,     // 30 minutes
}
```

**Query Optimization:**

```rust
// Use indexes for frequently queried columns
CREATE INDEX idx_assets_satker_status ON assets(satker_id, status);
CREATE INDEX idx_workflow_instances_user_status ON workflow_instances(initiated_by, status);

// Use EXPLAIN ANALYZE to optimize queries
EXPLAIN ANALYZE
SELECT a.*, c.nama as kategori_nama
FROM assets a
JOIN asset_categories c ON a.kategori_id = c.id
WHERE a.satker_id = $1 AND a.status = 'Aktif'
ORDER BY a.created_at DESC
LIMIT 20 OFFSET 0;
```

**Prepared Statements:**

```rust
pub struct AssetRepository {
    pool: DatabasePool,
    // Cache prepared statements
    stmt_cache: Arc<Mutex<HashMap<String, Statement>>>,
}

impl AssetRepository {
    pub async fn find_by_satker(&self, satker_id: Uuid) -> Result<Vec<Asset>> {
        let stmt = self.get_or_prepare_statement(
            "find_by_satker",
            "SELECT * FROM assets WHERE satker_id = $1"
        ).await?;

        let client = self.pool.get_connection().await?;
        let rows = client.query(&stmt, &[&satker_id]).await?;

        rows.iter().map(Asset::from_row).collect()
    }
}
```

**Batch Operations:**

```rust
pub async fn bulk_insert_assets(&self, assets: Vec<CreateAssetRequest>) -> Result<Vec<Asset>> {
    let client = self.pool.get_connection().await?;
    let tx = client.transaction().await?;

    // Use COPY for bulk inserts
    let sink = tx.copy_in("COPY assets (kode_barang, nama, kategori_id, satker_id) FROM STDIN").await?;

    let writer = BinaryCopyInWriter::new(sink, &[Type::TEXT, Type::TEXT, Type::UUID, Type::UUID]);

    for asset in assets {
        writer.write(&[
            &asset.kode_barang,
            &asset.nama,
            &asset.kategori_id,
            &asset.satker_id,
        ]).await?;
    }

    writer.finish().await?;
    tx.commit().await?;

    Ok(vec![]) // Return inserted assets
}
```

### API Response Optimization

**Pagination:**

```rust
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,

    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }

impl PaginationParams {
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }

    pub fn limit(&self) -> u32 {
        self.per_page.min(100) // Max 100 items per page
    }
}
```

**Field Selection:**

```rust
#[derive(Debug, Deserialize)]
pub struct FieldSelection {
    #[serde(default)]
    pub fields: Option<String>, // Comma-separated field names
}

pub async fn list_assets(
    Query(pagination): Query<PaginationParams>,
    Query(fields): Query<FieldSelection>,
) -> Result<Json<PaginatedResponse<Asset>>> {
    let selected_fields = fields.fields
        .map(|f| f.split(',').map(String::from).collect())
        .unwrap_or_else(|| vec!["*".to_string()]);

    // Build query with selected fields only
    let query = format!("SELECT {} FROM assets LIMIT $1 OFFSET $2", selected_fields.join(", "));

    // ... execute query
}
```

**Response Compression:**

```rust
use tower_http::compression::CompressionLayer;

let app = Router::new()
    .route("/api/pembinaan/perlengkapan/assets", get(list_assets))
    .layer(CompressionLayer::new());
```

### Async Processing

**Background Jobs:**

```rust
use tokio_cron_scheduler::{JobScheduler, Job};

pub async fn setup_background_jobs() -> Result<JobScheduler> {
    let scheduler = JobScheduler::new().await?;

    // Daily report generation
    scheduler.add(Job::new_async("0 0 2 * * *", |_uuid, _l| {
        Box::pin(async move {
            generate_daily_reports().await.ok();
        })
    })?).await?;

    // Hourly cache cleanup
    scheduler.add(Job::new_async("0 0 * * * *", |_uuid, _l| {
        Box::pin(async move {
            cleanup_expired_cache().await.ok();
        })
    })?).await?;

    // SLA monitoring every 5 minutes
    scheduler.add(Job::new_async("0 */5 * * * *", |_uuid, _l| {
        Box::pin(async move {
            check_workflow_sla().await.ok();
        })
    })?).await?;

    scheduler.start().await?;

    Ok(scheduler)
}
```

**Async Notifications:**

```rust
pub async fn send_notification_async(notification: Notification) {
    tokio::spawn(async move {
        if let Err(e) = send_notification_internal(notification).await {
            tracing::error!("Failed to send notification: {}", e);
        }
    });
}
```

### Monitoring and Metrics

**Prometheus Metrics:**

```rust
use prometheus::{Registry, Counter, Histogram, Gauge};

pub struct Metrics {
    pub http_requests_total: Counter,
    pub http_request_duration: Histogram,
    pub db_connections_active: Gauge,
    pub cache_hits: Counter,
    pub cache_misses: Counter,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Result<Self> {
        let http_requests_total = Counter::new(
            "http_requests_total",
            "Total number of HTTP requests"
        )?;
        registry.register(Box::new(http_requests_total.clone()))?;

        let http_request_duration = Histogram::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds"
        )?;
        registry.register(Box::new(http_request_duration.clone()))?;

        // ... register other metrics

        Ok(Self {
            http_requests_total,
            http_request_duration,
            // ...
        })
    }
}

pub async fn metrics_middleware(
    State(metrics): State<Arc<Metrics>>,
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();

    metrics.http_requests_total.inc();

    let response = next.run(req).await;

    let duration = start.elapsed().as_secs_f64();
    metrics.http_request_duration.observe(duration);

    response
}
```

**OpenTelemetry Tracing:**

```rust
use opentelemetry::trace::{Tracer, SpanKind};
use tracing_opentelemetry::OpenTelemetryLayer;

pub fn setup_tracing() -> Result<()> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(opentelemetry::runtime::Tokio)?;

    let telemetry = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}
```

### Performance Targets

**Response Time Targets:**
- List endpoints: < 100ms (95th percentile)
- Detail endpoints: < 50ms (95th percentile)
- Create/Update: < 200ms (95th percentile)
- Complex reports: < 2s (95th percentile)

**Throughput Targets:**
- 10,000 requests per minute
- 1,000 concurrent users
- 100 concurrent database connections

**Resource Limits:**
- CPU: 2 cores per pod
- Memory: 2GB per pod
- Database connections: 20 per pod

## Migration Strategy

### From Planned Multi-Crate to Unified Structure

**Context:**
The original plan was to build separate crates (api, dokumen, notifikasi, bantuan). This design consolidates everything into a single crate from the start, avoiding the need for future migration.

**Benefits of Unified Approach:**
1. **Simpler Deployment**: Single binary, single container
2. **Easier Development**: No inter-crate dependency management
3. **Better Performance**: No serialization overhead between crates
4. **Clearer Boundaries**: Module system enforces encapsulation
5. **Faster Builds**: Single compilation unit

### Workspace Integration

**Update Root Cargo.toml:**

```toml
[workspace]
members = [
    # ... existing members ...
    "layanan/perlengkapan",  # Add unified crate
    # Remove old entries:
    # "layanan/perlengkapan/crates/api",
    # "layanan/perlengkapan/crates/dokumen",
    # "layanan/perlengkapan/crates/notifikasi",
    # "layanan/perlengkapan/crates/bantuan",
]
```

**Unified Crate Cargo.toml:**

```toml
[package]
name = "layanan-perlengkapan"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
# All dependencies use workspace = true
axum = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
# ... etc

[dev-dependencies]
proptest = { workspace = true }
axum-test = { workspace = true }
# ... etc
```

### CI/CD Updates

**GitHub Actions Workflow:**

```yaml
# .github/workflows/ci.yml
jobs:
  test-perlengkapan:
    runs-on: self-hosted
    steps:
      - uses: actions/checkout@v3

      - name: Build layanan-perlengkapan
        run: cargo build -p layanan-perlengkapan

      - name: Test layanan-perlengkapan
        run: cargo test -p layanan-perlengkapan

      - name: Clippy
        run: cargo clippy -p layanan-perlengkapan -- -D warnings
```

**Docker Build:**

```yaml
# .github/workflows/docker.yml
jobs:
  build-perlengkapan:
    runs-on: self-hosted
    steps:
      - uses: actions/checkout@v3

      - name: Build Docker image
        run: |
          docker build \
            -f layanan/perlengkapan/Dockerfile \
            -t ghcr.io/analisaperlengkapan/simpel2/layanan-perlengkapan:${{ github.sha }} \
            .

      - name: Push Docker image
        run: |
          docker push ghcr.io/analisaperlengkapan/simpel2/layanan-perlengkapan:${{ github.sha }}
```

### Documentation Updates

**Files to Update:**
1. `README.md` - Update architecture overview
2. `AGENTS.md` - Update routing guide
3. `layanan/AGENTS.md` - Document unified pattern
4. `layanan/perlengkapan/AGENTS.md` - Update module structure
5. `CONTRIBUTING.md` - Update development guidelines

**Architecture Diagram Updates:**
- Remove separate crate boxes
- Show single unified service
- Update communication flows

### Cleanup Tasks

**Remove Old Structure:**

```bash
# After migration is complete
rm -rf layanan/perlengkapan/crates/
```

**Update References:**
- Search for references to old crate names
- Update import paths
- Update documentation links

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)

**Tasks:**
1. Create unified crate structure
2. Set up infrastructure layer (database, gRPC, middleware)
3. Implement configuration management
4. Set up error handling
5. Create shared models
6. Write infrastructure tests

**Deliverables:**
- Compiling crate with infrastructure
- Database connection working
- gRPC clients functional
- Basic health checks

### Phase 2: Core Modules (Week 3-4)

**Tasks:**
1. Implement Bank Aset module
2. Implement Kebutuhan BMN module
3. Implement Pemakaian BMN module
4. Implement Penghapusan BMN module
5. Implement Pakaian Dinas module
6. Write module tests

**Deliverables:**
- CRUD operations for all core modules
- API endpoints functional
- Unit tests passing
- Integration tests passing

### Phase 3: Workflow Engine (Week 5)

**Tasks:**
1. Implement workflow definitions
2. Implement workflow instances
3. Implement approval logic
4. Implement SLA monitoring
5. Implement escalation
6. Write workflow tests

**Deliverables:**
- Workflow engine functional
- Approval flows working
- SLA monitoring active
- Property-based tests passing

### Phase 4: Supporting Modules (Week 6-7)

**Tasks:**
1. Implement Dokumen module
2. Implement Notifikasi module
3. Implement Bantuan module
4. Implement Dashboard module
5. Implement Admin module
6. Write module tests

**Deliverables:**
- Document management working
- Notifications sending
- Ticket system functional
- Dashboard displaying data
- Admin functions working

### Phase 5: Integration & Testing (Week 8)

**Tasks:**
1. Integrate with Authenc
2. Integrate with Secreton
3. Integrate with Integrasi service
4. Write integration tests
5. Write E2E tests
6. Performance testing

**Deliverables:**
- All integrations working
- Test coverage > 80%
- Performance targets met
- Load tests passing

### Phase 6: Deployment & Documentation (Week 9-10)

**Tasks:**
1. Create Docker image
2. Create Kubernetes manifests
3. Create Helm charts
4. Write deployment documentation
5. Write API documentation
6. Write developer documentation

**Deliverables:**
- Production-ready deployment
- Complete documentation
- Runbooks for operations
- Developer onboarding guide

### Phase 7: Production Readiness (Week 11-12)

**Tasks:**
1. Security audit
2. Performance optimization
3. Monitoring setup
4. Alerting configuration
5. Disaster recovery planning
6. Production deployment

**Deliverables:**
- Security audit passed
- Monitoring dashboards
- Alerts configured
- DR plan documented
- Production deployment successful

## Success Criteria

### Technical Success Criteria

1. **Compilation**: Crate compiles without errors or warnings
2. **Tests**: All tests passing with > 80% coverage
3. **Performance**: API response times < 200ms (95th percentile)
4. **Security**: Zero critical vulnerabilities
5. **Documentation**: Complete API and developer documentation

### Business Success Criteria

1. **Functionality**: All business modules operational
2. **Reliability**: 99.9% uptime
3. **Scalability**: Supports 1000 concurrent users
4. **Maintainability**: Clear code structure, good documentation
5. **User Satisfaction**: Positive feedback from users

### Operational Success Criteria

1. **Deployment**: Automated CI/CD pipeline
2. **Monitoring**: Comprehensive metrics and logging
3. **Alerting**: Timely alerts for issues
4. **Recovery**: Disaster recovery plan tested
5. **Support**: Runbooks for common issues

## Conclusion

This design document provides a comprehensive blueprint for building the unified perlengkapan system. The design emphasizes:

- **Production-Ready Architecture**: Built for production from day one
- **Zero-Trust Security**: Comprehensive security at all layers
- **High Performance**: Optimized for speed and scalability
- **Maintainability**: Clear structure and excellent documentation
- **Testability**: Comprehensive testing strategy with property-based tests

The unified crate approach simplifies development, deployment, and maintenance while providing all the benefits of modular architecture through Rust's module system.
