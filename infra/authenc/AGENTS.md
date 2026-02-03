# 🤖 AGENTS.md - Authenc

> **AI Agent Guide** for working with the Authentication & Authorization Service

## 🌍 Service Context

**Authenc** adalah Enterprise-grade Identity Provider (IdP) dan Authorization Server untuk SIMPelv2, dikembangkan oleh Cipherce. Menyediakan:

### Core Authentication
- **Multi-Protocol Auth**: OIDC, SAML, JWT, OAuth2
- **MFA Support**: TOTP, SMS, Email, WebAuthn/FIDO2
- **Brute Force Protection**: Automatic lockout & anomaly detection
- **AI-Resistant CAPTCHA**: Dynamic difficulty based on risk scoring

### Authorization
- **OAuth2/OIDC**: Full RFC compliance with PAR, Token Exchange (RFC 8693)
- **UMA 2.0**: User-Managed Access with policy engine
- **RBAC/ABAC**: Role and attribute-based access control
- **Client Policies**: Dynamic Client Registration (RFC 7591/7592)

### Enterprise Features
- **SSO/Federation**: External IdP integration, identity brokering
- **OID4VC**: Verifiable Credentials support
- **Realm Management**: Multi-tenant isolation
- **Satker Authorization**: Government hierarchy (Kejaksaan RI struktur)
- **Admin Console**: Leptos-based web admin interface

**Compliance:**
- Zero-trust security architecture
- FIPS 140-2 cryptography
- GDPR-aligned data handling (consent management)
- Government security standards
- Quantum-resistant cryptography (ML-DSA, ML-KEM, Falcon) [optional]

## 🔑 Tech Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Language | Rust | Edition 2024, MSRV 1.90+ |
| Web Framework | Axum | 0.8.x |
| Database | PostgreSQL | tokio-postgres + deadpool + refinery |
| Cryptography | Ed25519, X25519, ChaCha20-Poly1305 | jsonwebtoken, argon2 |
| gRPC | Tonic + Prost | 0.14.x |
| Metrics | OpenTelemetry, Prometheus | 0.31+ |
| Events | Kafka (rdkafka) | Optional |
| Admin UI | Leptos | 0.8.x (feature: admin_console) |
| Post-Quantum | pqcrypto-mldsa, mlkem, falcon | Optional |

### Feature Flags
```toml
[features]
default = ["axum", "grpc", "auth", "oidc", "db", "metrics"]
axum = ["dep:axum", "dep:tower", "dep:tower-http"]
grpc = ["dep:tonic", "dep:tonic-prost", "dep:prost", "dep:prost-types"]
auth = ["dep:jsonwebtoken", "dep:argon2"]
oidc = ["auth"]
db = ["dep:tokio-postgres", "dep:deadpool-postgres"]
metrics = ["dep:opentelemetry", "dep:opentelemetry-otlp", "dep:tracing-opentelemetry"]
rdkafka = ["dep:rdkafka"]
admin_console = ["dep:leptos"]
quantum = ["dep:pqcrypto-mldsa", "dep:pqcrypto-mlkem", "dep:pqcrypto-falcon"]
tpm = ["dep:tss-esapi"]
```

## 🏗️ Architecture

```
infra/authenc/
├── Cargo.toml                  # Part of main simpelv2 workspace
├── src/
│   ├── main.rs                 # Entry point - ApplicationBuilder
│   ├── lib.rs                  # Module exports
│   ├── app.rs                  # AppState with all services (989 lines!)
│   ├── app_init/               # Initialization helpers
│   ├── app_logging.rs          # Logging setup
│   ├── config/                 # Configuration management
│   ├── error.rs                # AuthencError types
│   │
│   ├── database/               # Database layer (refinery migrations)
│   │   ├── mod.rs              # Connection pool
│   │   ├── migrations.rs       # Migration runner
│   │   ├── operations/         # CRUD operations
│   │   └── models/             # Data models
│   │
│   ├── services/               # Business logic (70+ modules!)
│   │   ├── mod.rs              # Service registry
│   │   ├── stores/             # Entity stores (user, role, realm, etc.)
│   │   ├── session_store.rs    # Session management
│   │   ├── totp_store.rs       # TOTP secret management
│   │   ├── brute_force_protector.rs  # Attack prevention
│   │   ├── anomaly_detector.rs       # Threat detection
│   │   ├── captcha/                  # AI-resistant CAPTCHA
│   │   ├── risk_engine.rs            # Dynamic risk scoring
│   │   ├── mfa_service.rs            # MFA orchestration
│   │   ├── mfa_admin_service.rs      # MFA administration
│   │   ├── federation/               # Federation protocols
│   │   ├── federation_manager.rs     # Federation orchestration
│   │   ├── broker/                   # Identity brokering
│   │   ├── sso/                      # Single Sign-On
│   │   ├── uma/                      # UMA 2.0 policy engine
│   │   ├── token/                    # Token management
│   │   ├── token_exchange.rs         # RFC 8693 Token Exchange
│   │   ├── client_registration.rs    # Dynamic Client Registration
│   │   ├── par/                      # Pushed Authorization Requests
│   │   ├── oid4vc.rs                 # Verifiable Credentials
│   │   ├── cache/                    # Redis cache
│   │   ├── clustering/               # High availability
│   │   ├── observability/            # Metrics & tracing
│   │   ├── key_rotation.rs           # Automatic key rotation
│   │   ├── compliance/               # Compliance reporting
│   │   ├── fips/                     # FIPS mode
│   │   ├── zero_trust/               # Zero-trust policies
│   │   └── webauthn.rs               # WebAuthn/FIDO2
│   │
│   ├── handlers/               # HTTP request handlers (40+ handlers!)
│   │   ├── mod.rs
│   │   ├── api/                # API handlers
│   │   ├── oauth2.rs           # OAuth2 endpoints
│   │   ├── oauth2_authz_code.rs
│   │   ├── oidc_*.rs           # OIDC endpoints (provider, keys, sso)
│   │   ├── saml.rs             # SAML endpoints
│   │   ├── totp.rs             # TOTP endpoints
│   │   ├── webauthn.rs         # WebAuthn endpoints
│   │   ├── uma.rs              # UMA 2.0 endpoints
│   │   ├── federated_*.rs      # Federation handlers
│   │   ├── admin.rs            # Admin endpoints
│   │   ├── metrics.rs          # Prometheus metrics
│   │   └── health.rs           # Health checks
│   │
│   ├── middleware/             # Axum middlewares
│   │   └── ...                 # JWT validation, rate limiting, audit
│   │
│   ├── grpc/                   # gRPC services
│   │   ├── mod.rs
│   │   ├── authenc_service.rs  # Main gRPC service
│   │   ├── captcha_service.rs  # CAPTCHA gRPC
│   │   ├── health.rs           # gRPC health check
│   │   ├── interceptors.rs     # Auth interceptors
│   │   └── batch_operations.rs # Batch gRPC ops
│   │
│   ├── protocol/               # Protocol mappers (OIDC/SAML claims)
│   ├── authenticator/          # Custom authenticator SPI
│   ├── spi/                    # Service Provider Interface
│   ├── events/                 # Event-driven architecture
│   ├── health/                 # Shared health checks
│   ├── models/                 # Data models
│   ├── crypto/                 # Cryptographic utilities
│   ├── utils/                  # Common utilities
│   │   └── jwt_key_manager.rs  # Dynamic JWT key management
│   ├── secreton_client/        # Secreton gRPC client
│   │   └── secreton_client.rs
│   ├── axum_app/               # Axum-specific code
│   └── admin_console/          # Leptos admin UI (feature-gated)
│
├── migrations/                 # 37+ SQL migrations
│   ├── 001_initial_schema.sql
│   ├── 021_mfa_fields.sql
│   ├── 025_captcha_system.sql
│   ├── 031_satker_hierarchy.sql
│   ├── 035_token_exchange.sql
│   └── ...
│
└── proto/                      # gRPC proto files
    ├── authenc.proto           # Main auth service
    ├── common.proto            # Shared types
    ├── secreton.proto          # Secreton client proto
    └── raft.proto              # Raft consensus
```

## 📏 Critical Conventions

### 1. Configuration

**File:** `src/config/mod.rs`

```rust
pub struct AppConfig {
    // Server configuration
    pub server: ServerConfig,       // host, port, grpc_port

    // Database configuration
    pub database: DatabaseConfig,   // from lib-common

    // Rate limiting
    pub rate_limit: RateLimitConfig,
    pub adaptive_rate_limit: AdaptiveRateLimitConfig,
    pub mfa_rate_limit: MfaRateLimitConfig,

    // Security
    pub security: BasicSecurityConfig,

    // Observability (logging, metrics)
    pub observability: ObservabilityConfig,

    // Feature flags
    pub features: FeatureConfig,

    // Authentication protocols
    pub oidc: Option<OidcConfig>,
    pub saml: Option<SamlConfig>,
    pub sso_cookie: SsoCookieConfig,

    // UI configuration (admin console)
    pub ui: Option<UiConfig>,

    // External services
    pub secreton: Option<SecretonConfig>,  // Secret management
    pub redis: Option<RedisConfig>,        // Caching
    pub kafka: Option<KafkaConfig>,        // Event streaming

    // Enterprise features
    pub events: EventsConfig,       // Event retention
    pub spi: SpiConfig,             // Service Provider Interface
    pub clustering: ClusterConfig,   // High availability
    pub key_rotation: Option<KeyRotationConfig>,
    pub federation: Option<FederationConfig>,
}

// Alias for backward compatibility
pub type AuthencConfig = AppConfig;
```

**Configuration Files:**
```
config/
├── default.toml       # Default values
├── production.toml    # Production overrides
├── development.toml   # Dev overrides
└── testing.toml       # Test overrides
```

Configuration hierarchy: `default.toml` → `production.toml` → Environment Variables
    pub grpc_port: u16,

    // Database
    pub database_url: String,
    pub database_pool_size: usize,

    // JWT Signing
    pub jwt_issuer: String,
    pub jwt_access_token_ttl: Duration,   // Default: 15 minutes
    pub jwt_refresh_token_ttl: Duration,  // Default: 7 days
    pub jwt_signing_key_path: String,     // Ed25519 private key

    // MFA
    pub mfa_totp_enabled: bool,
    pub mfa_sms_enabled: bool,
    pub mfa_email_enabled: bool,
    pub mfa_issuer: String,               // For TOTP (e.g., "Kejaksaan RI")

    // Rate Limiting
    pub rate_limit_enabled: bool,
    pub rate_limit_requests_per_minute: u32,

    // Integration with Secreton
    pub secreton_grpc_url: String,
}
```

**Environment Variables:**
```bash
# Server
AUTHENC_HOST=0.0.0.0
AUTHENC_PORT=8080
AUTHENC_GRPC_PORT=50051

# Database
DATABASE_URL=postgres://authenc:password@localhost:5432/authenc
DATABASE_POOL_SIZE=20

# JWT
JWT_ISSUER=https://authenc.kejaksaan.go.id
JWT_ACCESS_TOKEN_TTL=900      # 15 minutes
JWT_REFRESH_TOKEN_TTL=604800  # 7 days
JWT_SIGNING_KEY_PATH=/secrets/authenc_ed25519.key

# MFA
MFA_TOTP_ENABLED=true
MFA_ISSUER=Kejaksaan RI
MFA_SMS_ENABLED=false
MFA_EMAIL_ENABLED=true

# Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_REQUESTS_PER_MINUTE=60

# Secreton Integration
SECRETON_GRPC_URL=https://secreton.internal:50052
```

### 2. Database Layer

**File:** `src/database/mod.rs`

```rust
pub struct Database {
    pool: deadpool_postgres::Pool,
    prepared_cache: PreparedStatementCache,
}

impl Database {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let pool = create_pool(config).await?;
        let prepared_cache = PreparedStatementCache::new();
        Ok(Self { pool, prepared_cache })
    }

    // Transaction support
    pub async fn transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: for<'a> FnOnce(DatabaseTransaction<'a>)
            -> Pin<Box<dyn Future<Output = Result<R>> + Send + 'a>>,
    {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;
        let db_tx = DatabaseTransaction::new(tx);

        let result = f(db_tx).await?;
        tx.commit().await?;
        Ok(result)
    }

    // Prepared statements with caching
    pub async fn query_one_prepared(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<tokio_postgres::Row> {
        let client = self.get_connection().await?;
        let stmt = self.prepared_cache
            .get_or_prepare(&client, statement)
            .await?;

        client.query_one(stmt.as_ref(), params).await
            .map_err(|e| AuthencError::database(format!("Query failed: {}", e)))
    }
}
```

### 3. User Operations

**File:** `src/database/operations/users_ops.rs`

```rust
pub async fn create_user(
    db: &Database,
    request: CreateUserRequest,
    realm_id: Uuid,
) -> Result<User> {
    let user_id = Uuid::new_v4();
    let password_hash = hash_password(&request.password)?;

    let query = r#"
        INSERT INTO users (
            id, username, email, password_hash, enabled,
            email_verified, realm_id, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        RETURNING *
    "#;

    let row: tokio_postgres::Row = db
        .query_one_prepared(
            query,
            &[
                &user_id,
                &request.username,
                &request.email,
                &password_hash,
                &true,  // enabled
                &false, // email_verified
                &realm_id,
            ],
        )
        .await?;

    row.try_into()
}

pub async fn authenticate_user(
    db: &Database,
    username: &str,
    password: &str,
) -> Result<User> {
    let user = get_user_by_username(db, username).await?;

    if !verify_password(password, &user.password_hash)? {
        return Err(AuthencError::InvalidCredentials);
    }

    if !user.enabled {
        return Err(AuthencError::UserDisabled);
    }

    // Check MFA requirement
    if user.mfa_enabled {
        // MFA verification required - handled separately
    }

    Ok(user)
}
```

### 4. OAuth2 Flow

**File:** `src/oauth2/flows/authorization_code.rs`

```rust
pub async fn handle_authorization_request(
    db: &Database,
    request: AuthorizationRequest,
) -> Result<AuthorizationResponse> {
    // 1. Validate client
    let client = get_client_by_id(db, &request.client_id).await?;
    validate_redirect_uri(&client, &request.redirect_uri)?;

    // 2. Validate scope
    validate_scopes(&client.allowed_scopes, &request.scope)?;

    // 3. Generate authorization code
    let auth_code = generate_authorization_code();
    let expires_at = Utc::now() + Duration::minutes(10);

    // 4. Store authorization code
    store_authorization_code(db, AuthorizationCode {
        code: auth_code.clone(),
        client_id: request.client_id,
        user_id: request.user_id,
        redirect_uri: request.redirect_uri.clone(),
        scope: request.scope.clone(),
        expires_at,
    }).await?;

    // 5. Return redirect URI with code
    Ok(AuthorizationResponse {
        redirect_uri: format!(
            "{}?code={}&state={}",
            request.redirect_uri, auth_code, request.state
        ),
    })
}

pub async fn handle_token_request(
    db: &Database,
    request: TokenRequest,
) -> Result<TokenResponse> {
    // 1. Validate authorization code
    let auth_code = validate_authorization_code(
        db,
        &request.code,
        &request.client_id,
        &request.redirect_uri,
    ).await?;

    // 2. Generate tokens
    let access_token = generate_jwt_access_token(
        &auth_code.user_id,
        &auth_code.scope,
    )?;

    let refresh_token = generate_refresh_token();

    // 3. Store tokens
    store_tokens(db, &access_token, &refresh_token, &auth_code).await?;

    // 4. Invalidate authorization code
    invalidate_authorization_code(db, &request.code).await?;

    Ok(TokenResponse {
        access_token,
        refresh_token: Some(refresh_token),
        token_type: "Bearer".to_string(),
        expires_in: 900, // 15 minutes
    })
}
```

### 5. JWT Token Generation

**File:** `src/oauth2/tokens.rs`

```rust
use lib_crypto::jwt::{create_jwt, JwtClaims};
use lib_crypto::keys::Ed25519KeyPair;

pub fn generate_jwt_access_token(
    user_id: &Uuid,
    scope: &str,
) -> Result<String> {
    // Load signing key from Secreton
    let signing_key = load_signing_key_from_secreton()?;

    let claims = JwtClaims {
        iss: config.jwt_issuer.clone(),
        sub: user_id.to_string(),
        aud: vec!["simpelv2".to_string()],
        exp: (Utc::now() + Duration::minutes(15)).timestamp(),
        iat: Utc::now().timestamp(),
        scope: scope.to_string(),
        // Custom claims
        realm: "kejaksaan-ri".to_string(),
    };

    create_jwt(&claims, &signing_key)
}

pub async fn validate_jwt_token(
    token: &str,
) -> Result<JwtClaims> {
    // Load verification key
    let verify_key = load_verification_key_from_secreton()?;

    // Verify signature and expiry
    let claims = verify_jwt(token, &verify_key)?;

    // Additional validation
    if claims.exp < Utc::now().timestamp() {
        return Err(AuthencError::TokenExpired);
    }

    if claims.iss != config.jwt_issuer {
        return Err(AuthencError::InvalidIssuer);
    }

    Ok(claims)
}
```

### 6. gRPC Service

**File:** `src/grpc/auth_service.rs`

```rust
use tonic::{Request, Response, Status};

pub mod auth_proto {
    tonic::include_proto!("auth");
}

use auth_proto::auth_service_server::{AuthService, AuthServiceServer};
use auth_proto::{AuthenticateRequest, AuthenticateResponse};

#[derive(Debug)]
pub struct AuthServiceImpl {
    db: Arc<Database>,
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn authenticate(
        &self,
        request: Request<AuthenticateRequest>,
    ) -> Result<Response<AuthenticateResponse>, Status> {
        let req = request.into_inner();

        // Authenticate user
        let user = authenticate_user(
            &self.db,
            &req.username,
            &req.password,
        ).await.map_err(|e| Status::unauthenticated(e.to_string()))?;

        // Generate access token
        let access_token = generate_jwt_access_token(&user.id, "openid profile")
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AuthenticateResponse {
            access_token,
            user_id: user.id.to_string(),
            username: user.username,
        }))
    }

    async fn validate_token(
        &self,
        request: Request<TokenRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        let token = request.into_inner().token;

        let claims = validate_jwt_token(&token).await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        Ok(Response::new(TokenResponse {
            valid: true,
            user_id: claims.sub,
            scope: claims.scope,
        }))
    }
}
```

### 7. MFA Implementation

**File:** `src/auth/mfa/totp.rs`

```rust
use totp_rs::{TOTP, Algorithm};

pub fn generate_totp_secret() -> String {
    // Generate 32-byte random secret
    let secret = totp_rs::Secret::generate_secret();
    secret.to_encoded().to_string()
}

pub fn generate_totp_qr_code(
    username: &str,
    secret: &str,
) -> Result<String> {
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,  // 6 digits
        1,  // 1 step (30 seconds)
        30, // period
        secret.as_bytes().to_vec(),
        Some(config.mfa_issuer.clone()),
        username.to_string(),
    )?;

    // Generate QR code URL
    let qr_url = totp.get_qr_base64()?;
    Ok(qr_url)
}

pub fn verify_totp_code(
    secret: &str,
    code: &str,
) -> Result<bool> {
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.as_bytes().to_vec(),
        None,
        "".to_string(),
    )?;

    Ok(totp.check_current(code)?)
}
```

## 🚀 Common Tasks

### Add New OAuth2 Scope

```rust
// 1. Add to database
INSERT INTO scopes (name, description, realm_id)
VALUES ('read:documents', 'Read document permissions', realm_id);

// 2. Add to client allowed scopes
UPDATE clients
SET allowed_scopes = array_append(allowed_scopes, 'read:documents')
WHERE client_id = 'portal-client';

// 3. Include in JWT claims
let claims = JwtClaims {
    scope: "openid profile read:documents".to_string(),
    // ...
};
```

### Implement Custom Authorization Policy

```rust
// src/uma/policy.rs
pub async fn evaluate_policy(
    db: &Database,
    resource_id: &Uuid,
    user_id: &Uuid,
    action: &str,
) -> Result<bool> {
    // 1. Get resource permissions
    let permissions = get_resource_permissions(db, resource_id).await?;

    // 2. Check user roles
    let user_roles = get_user_roles(db, user_id).await?;

    // 3. Evaluate policy
    for permission in permissions {
        if permission.action == action {
            if user_roles.iter().any(|r| permission.roles.contains(&r.name)) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
```

### Add External IdP (Federation)

```rust
// src/federation/external_idp.rs
pub async fn configure_external_idp(
    db: &Database,
    config: ExternalIdpConfig,
) -> Result<()> {
    // 1. Store IdP configuration
    let idp = create_identity_provider(db, CreateIdentityProviderRequest {
        name: config.name,
        provider_type: "oidc".to_string(),
        config: serde_json::json!({
            "authorization_endpoint": config.authorization_endpoint,
            "token_endpoint": config.token_endpoint,
            "client_id": config.client_id,
            "client_secret": config.client_secret,
        }),
        enabled: true,
    }).await?;

    // 2. Set up attribute mapping
    create_attribute_mapper(db, &idp.id, vec![
        ("email", "email"),
        ("name", "name"),
        ("nip", "employee_id"),
    ]).await?;

    Ok(())
}
```

## ⚠️ Common Pitfalls

### ❌ DON'T

1. **Store passwords in plaintext**
   ```rust
   // ❌ BAD
   user.password = request.password;

   // ✅ GOOD
   user.password_hash = hash_password(&request.password)?;
   ```

2. **Use short-lived refresh tokens**
   ```rust
   // ❌ BAD
   JWT_REFRESH_TOKEN_TTL=900  // 15 minutes

   // ✅ GOOD
   JWT_REFRESH_TOKEN_TTL=604800  // 7 days
   ```

3. **Skip token validation**
   ```rust
   // ❌ BAD - Trust token without verification
   let user_id = extract_user_id_from_token(token);

   // ✅ GOOD - Always validate
   let claims = validate_jwt_token(token).await?;
   let user_id = Uuid::parse_str(&claims.sub)?;
   ```

4. **Hard-code signing keys**
   ```rust
   // ❌ BAD
   const SIGNING_KEY: &str = "my-secret-key-123";

   // ✅ GOOD - Load from Secreton
   let signing_key = load_signing_key_from_secreton().await?;
   ```

5. **Allow unlimited login attempts**
   ```rust
   // ❌ BAD - No rate limiting
   authenticate_user(username, password).await?;

   // ✅ GOOD - Use rate limiting middleware
   .layer(rate_limiting_layer())
   ```

### ✅ DO

1. **Use Argon2 for password hashing**
2. **Validate all JWT tokens** (signature, expiry, issuer)
3. **Implement rate limiting** on login endpoints
4. **Store keys in Secreton**, not environment variables
5. **Use prepared statements** to prevent SQL injection
6. **Log all authentication events** to audit log
7. **Require MFA** for admin accounts

## 🔍 Troubleshooting

### JWT Validation Fails

**Problem:** "Invalid signature" error

**Solution:**
```bash
# 1. Check signing key matches
secreton-cli get jwt_signing_key

# 2. Verify token issuer
jwt decode $TOKEN | jq .iss

# 3. Check key rotation hasn't happened
SELECT * FROM signing_keys WHERE is_active = true;
```

### Database Connection Pool Exhausted

**Problem:** "Connection pool timeout"

**Solution:**
```rust
// In config
DATABASE_POOL_SIZE=50  // Increase from 20

// Or use transactions properly
db.transaction(|tx| async move {
    // All queries here
    tx.commit().await?;
}).await?;
```

### MFA TOTP Not Working

**Problem:** Code always invalid

**Solution:**
```bash
# Check server time synchronization
timedatectl status

# Verify secret encoding
echo $TOTP_SECRET | base32 -d | xxd

# Test with known-good authenticator
# Google Authenticator, Microsoft Authenticator
```

## 📚 Key Files Reference

| File | Purpose |
|------|---------|
| `src/database/operations/users_ops.rs` | User CRUD |
| `src/oauth2/flows/authorization_code.rs` | OAuth2 auth code flow |
| `src/oauth2/tokens.rs` | JWT generation/validation |
| `src/grpc/auth_service.rs` | gRPC authentication service |
| `src/auth/mfa/totp.rs` | TOTP implementation |
| `migrations/001_initial_schema.sql` | Complete DB schema |

## 🎓 Best Practices

1. **Always hash passwords** with Argon2
2. **Validate tokens** on every request (via gRPC from layanan)
3. **Use Ed25519** for JWT signing (not RSA)
4. **Store keys in Secreton**, rotate regularly
5. **Implement MFA** for sensitive operations
6. **Log authentication events** to audit log
7. **Use prepared statements** for database queries
8. **Rate limit** login endpoints
9. **Validate redirect URIs** strictly in OAuth2
10. **Use HTTPS/mTLS** for all communication

---

**Last Updated:** February 2, 2026
**Maintainer:** SIMPelv2 Team
**Related:** `/.github/copilot-instructions.md`, `/AGENTS.md`
