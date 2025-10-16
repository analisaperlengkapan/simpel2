# Sinergi Komponen: Secreton-Authenc-Portal Integration

## 🎯 Peran & Tanggung Jawab Masing-Masing Komponen

### 1. **Secreton (Vault Service)** 🔐

**Peran Utama**: Secret Management & Storage

**Tanggung Jawab**:

- ✅ Menyimpan dan mengelola secrets (credentials, API keys, certificates)
- ✅ Versioning untuk setiap secret (audit trail lengkap)
- ✅ Enkripsi at-rest untuk semua data sensitif
- ✅ Access control via bearer token authentication
- ✅ Audit logging untuk compliance (SIEM integration ready)
- ✅ Performance metrics (Prometheus export)
- ✅ Health check endpoint untuk monitoring

**Yang TIDAK dilakukan**:

- ❌ User authentication (delegasi ke Authenc)
- ❌ UI rendering (delegasi ke Portal)
- ❌ Business logic aplikasi

**API Endpoints**:

```
POST   /v1/secret/data/{realm}/{key}    # Store secret
GET    /v1/secret/data/{realm}/{key}    # Retrieve secret
DELETE /v1/secret/data/{realm}/{key}    # Delete secret
GET    /v1/secret/metadata/{realm}      # List secrets
GET    /v1/sys/health                   # Health check
GET    /metrics                          # Prometheus metrics
```

---

### 2. **Authenc (IAM Service)** 👤

**Peran Utama**: Identity & Access Management

**Tanggung Jawab**:

- ✅ User authentication (login/logout)
- ✅ JWT token issuance dan validation
- ✅ OAuth2/OIDC protocol implementation
- ✅ RBAC (Role-Based Access Control)
- ✅ User/realm management
- ✅ Session management (SSO, timeouts)
- ✅ MFA (Multi-Factor Authentication)
- ✅ Social login integration
- ✅ **Menggunakan Secreton untuk store credentials** (client secrets, signing keys)

**Yang TIDAK dilakukan**:

- ❌ Secret storage implementation (delegasi ke Secreton)
- ❌ UI rendering (delegasi ke Portal)
- ❌ Direct database access untuk secrets

**API Endpoints**:

```
POST   /realms/{realm}/protocol/openid-connect/token       # Get token
GET    /realms/{realm}/protocol/openid-connect/userinfo    # User info
POST   /realms/{realm}/protocol/openid-connect/logout      # Logout
GET    /realms/{realm}/protocol/openid-connect/certs       # Public keys
POST   /admin/realms/{realm}/users                         # User management
GET    /health                                              # Health check
```

---

### 3. **Portal (Frontend UI)** 🖥️

**Peran Utama**: User Interface & Experience

**Tanggung Jawab**:

- ✅ User-facing web interface
- ✅ Login/logout UI
- ✅ Dashboard dan navigation
- ✅ **Consume Authenc OAuth2 APIs** (login flow)
- ✅ Store JWT token in localStorage
- ✅ Protected route guards (require authentication)
- ✅ Role-based UI rendering
- ✅ **Optional: Admin UI untuk secret management** (via Secreton API)

**Yang TIDAK dilakukan**:

- ❌ Password validation logic (delegasi ke Authenc)
- ❌ Secret storage (delegasi ke Secreton)
- ❌ Token generation (receive dari Authenc)

**Routes**:

```
/                   # Dashboard (public)
/login              # Login page
/dashboard          # User dashboard (protected)
/admin/secrets      # Secret management (admin only)
/admin/users        # User management (admin only)
```

---

## 🔄 Data Flow & Sinergi

### Scenario 1: User Login Flow

```
┌─────────┐                ┌───────────┐                ┌────────────┐
│ Portal  │                │  Authenc  │                │  Secreton  │
│ (WASM)  │                │   (IAM)   │                │  (Vault)   │
└────┬────┘                └─────┬─────┘                └──────┬─────┘
     │                           │                              │
     │ 1. User enters            │                              │
     │    username/password      │                              │
     │───────────────────────────>                              │
     │ POST /token               │                              │
     │ grant_type=password       │                              │
     │                           │                              │
     │                           │ 2. Validate credentials      │
     │                           │    (check database)          │
     │                           │                              │
     │                           │ 3. Need signing key?         │
     │                           │───────────────────────────────>
     │                           │    GET /secret/jwt-key       │
     │                           │                              │
     │                           │<───────────────────────────────
     │                           │    {key: "secret"}           │
     │                           │                              │
     │                           │ 4. Generate JWT token        │
     │                           │    (sign with secret key)    │
     │                           │                              │
     │<───────────────────────────                              │
     │ 5. Return token           │                              │
     │    {access_token: "..."}  │                              │
     │                           │                              │
     │ 6. Store token in         │                              │
     │    localStorage           │                              │
     │                           │                              │
```

**Sinergi**:

- Portal: Menyediakan UI dan mengirim credentials
- Authenc: Validasi user, generate JWT token
- Secreton: Menyimpan signing key untuk JWT (secure storage)

---

### Scenario 2: Portal Accessing Protected Resource

```
┌─────────┐                ┌───────────┐                ┌────────────┐
│ Portal  │                │  Authenc  │                │  Secreton  │
│ (WASM)  │                │   (IAM)   │                │  (Vault)   │
└────┬────┘                └─────┬─────┘                └──────┬─────┘
     │                           │                              │
     │ 1. User wants to view     │                              │
     │    API keys               │                              │
     │                           │                              │
     │ 2. Get token from         │                              │
     │    localStorage           │                              │
     │                           │                              │
     │ 3. Request API keys       │                              │
     │───────────────────────────────────────────────────────────>
     │    GET /v1/secret/metadata/prod                          │
     │    Authorization: Bearer <jwt_token>                     │
     │                           │                              │
     │                           │    4. Validate JWT token     │
     │                           │<──────────────────────────────
     │                           │       (optional: introspect) │
     │                           │                              │
     │                           │─────────────────────────────>│
     │                           │       token valid            │
     │                           │                              │
     │                           │                              │ 5. Check permissions
     │                           │                              │    (from JWT claims)
     │                           │                              │
     │                           │                              │ 6. Return secrets list
     │<─────────────────────────────────────────────────────────────
     │    {keys: ["api-key-1", "api-key-2"]}                   │
     │                           │                              │
     │ 7. Render in UI           │                              │
     │                           │                              │
```

**Sinergi**:

- Portal: Request data dengan JWT token
- Authenc: Issue JWT token (implicitly validates via Secreton)
- Secreton: Validate token, return secrets based on permissions

---

### Scenario 3: Service-to-Service Communication

```
┌───────────┐                                   ┌────────────┐
│  Authenc  │                                   │  Secreton  │
│   (IAM)   │                                   │  (Vault)   │
└─────┬─────┘                                   └──────┬─────┘
      │                                                │
      │ 1. Startup: Need to load client secrets       │
      │────────────────────────────────────────────────>
      │    GET /v1/secret/data/authenc/oauth2-clients │
      │    Authorization: Bearer <admin_token>        │
      │                                                │
      │<────────────────────────────────────────────────
      │    {secrets: [...]}                           │
      │                                                │
      │ 2. Store client secrets configuration         │
      │    (in-memory cache)                          │
      │                                                │
      │ 3. Runtime: OAuth2 flow needs client secret   │
      │    (fetch from in-memory cache)               │
      │                                                │
      │ 4. Periodic: Rotate secrets                   │
      │────────────────────────────────────────────────>
      │    POST /v1/secret/data/authenc/oauth2-clients│
      │    (new version)                              │
      │                                                │
      │<────────────────────────────────────────────────
      │    {version: 2}                               │
      │                                                │
```

**Sinergi**:

- Authenc: Consume secrets dari Secreton (client secrets, signing keys)
- Secreton: Provide secure storage dengan versioning

---

## 🏗️ Separation of Concerns

### Layer Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                        │
│                        (Portal)                              │
│  • UI Components (Leptos/WASM)                              │
│  • Client-side routing                                       │
│  • State management (signals)                                │
└────────────────────────┬────────────────────────────────────┘
                         │ HTTPS/WSS
                         │ (OAuth2, JWT)
┌────────────────────────┴────────────────────────────────────┐
│                   Application Layer                          │
│                      (Authenc IAM)                           │
│  • Authentication & Authorization                            │
│  • User management                                           │
│  • OAuth2/OIDC protocols                                     │
│  • JWT token issuance                                        │
│  • RBAC enforcement                                          │
└────────────────────────┬────────────────────────────────────┘
                         │ HTTP + Bearer Token
                         │ (REST API)
┌────────────────────────┴────────────────────────────────────┐
│                    Data Layer                                │
│                   (Secreton Vault)                           │
│  • Secret storage (KV engine)                               │
│  • Versioning & audit                                        │
│  • Encryption at rest                                        │
│  • Access control                                            │
│  • Metrics & monitoring                                      │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔐 Security Model

### Authentication Flow

1. **Portal → Authenc**: User submits credentials
2. **Authenc**: Validates against database
3. **Authenc → Secreton**: Retrieve JWT signing key
4. **Authenc**: Sign JWT token with key
5. **Authenc → Portal**: Return JWT token
6. **Portal**: Store token in localStorage

### Authorization Flow

1. **Portal → Secreton**: Request with JWT token
2. **Secreton**: Extract claims from JWT (roles, permissions)
3. **Secreton**: Check if user has permission for operation
4. **Secreton**: Return data or 403 Forbidden

### Trust Model

```
Portal ──trusts──> Authenc ──trusts──> Secreton
   │                  │                    │
   │                  │                    │
   └─────────────────JWT Token────────────┘
```

- **Portal trusts Authenc**: untuk authentication
- **Authenc trusts Secreton**: untuk secret storage
- **Secreton trusts JWT**: issued by Authenc (via signature validation)

---

## 📊 Monitoring & Observability

### Metrics Collection

```
┌─────────┐         ┌───────────┐         ┌────────────┐
│ Portal  │         │  Authenc  │         │  Secreton  │
└────┬────┘         └─────┬─────┘         └──────┬─────┘
     │                    │                       │
     │ Browser metrics    │ Service metrics       │ Vault metrics
     │ (Sentry, GA)       │ (Prometheus)          │ (Prometheus)
     │                    │                       │
     ▼                    ▼                       ▼
┌────────────────────────────────────────────────────────┐
│              Monitoring Stack                          │
│  • Prometheus (metrics aggregation)                    │
│  • Grafana (visualization)                             │
│  • Loki (log aggregation)                              │
│  • Jaeger (distributed tracing)                        │
└────────────────────────────────────────────────────────┘
```

### Key Metrics by Component

**Portal**:

- Page load time
- Authentication success rate
- API call latency
- Error rate

**Authenc**:

- Login attempts
- Token issuance rate
- Failed authentication
- Session duration

**Secreton**:

- Secret operations (CRUD)
- Vault latency
- Audit events
- Storage usage

---

## 🧪 End-to-End Testing Strategy

### Integration Test Flow

```rust
#[tokio::test]
async fn test_full_integration_flow() {
    // 1. Start all services
    let secreton = start_secreton_server().await;
    let authenc = start_authenc_server().await;
    let portal = start_portal_server().await;

    // 2. User login via Portal
    let login_response = portal
        .post("/api/auth/login")
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await
        .unwrap();

    let token = login_response
        .json::<TokenResponse>()
        .await
        .unwrap()
        .access_token;

    // 3. Portal stores secret in Secreton (via token)
    let store_response = secreton
        .post("/v1/secret/data/prod/api-key")
        .bearer_auth(&token)
        .json(&json!({
            "data": "sk_live_test_key_123"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(store_response.status(), 200);

    // 4. Portal retrieves secret
    let get_response = secreton
        .get("/v1/secret/data/prod/api-key")
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    let secret_data = get_response
        .json::<SecretResponse>()
        .await
        .unwrap();

    assert_eq!(secret_data.data.data, "sk_live_test_key_123");

    // 5. Verify audit log
    let audit = secreton.get_audit_events().await;
    assert_eq!(audit.len(), 2); // put + get
    assert_eq!(audit[0].event_type, AuditEventType::SecretCreated);
    assert_eq!(audit[1].event_type, AuditEventType::SecretRead);
}
```

---

## 📝 Configuration Dependencies

### Environment Variables per Component

**Secreton**:

```bash
SECRETON_PORT=8200
SECRETON_ADMIN_TOKEN=<generated>
SECRETON_DATABASE_URL=postgresql://...
SECRETON_JWT_ISSUER=simpelv2-authenc  # Must match Authenc
SECRETON_JWT_AUDIENCE=secreton
```

**Authenc**:

```bash
AUTHENC_PORT=8088
AUTHENC_JWT_SECRET=<generated>
AUTHENC_JWT_ISSUER=simpelv2-authenc  # Must match Secreton
AUTHENC_DATABASE_URL=postgresql://...
SECRETON_API_URL=http://localhost:8200  # Secreton endpoint
SECRETON_TOKEN=<admin_token>            # For accessing Secreton
```

**Portal**:

```bash
PORTAL_PORT=8080
AUTHENC_API_URL=http://localhost:8088  # Authenc endpoint
SECRETON_API_URL=http://localhost:8200  # Optional: for admin UI
```

---

## 🚀 Deployment Checklist

### Pre-deployment

- [ ] Generate secure JWT secret (Authenc)
- [ ] Generate admin token (Secreton)
- [ ] Configure database connections
- [ ] Set up Redis for sessions
- [ ] Configure CORS origins
- [ ] Set up TLS certificates

### Service Startup Order

1. **PostgreSQL** (database layer)
2. **Redis** (session store)
3. **Secreton** (vault service) - Port 8200
4. **Authenc** (IAM service) - Port 8088
5. **Portal** (frontend) - Port 8080
6. **Nginx** (reverse proxy) - Port 80/443

### Health Check Sequence

```bash
# 1. Secreton
curl http://localhost:8200/v1/sys/health
# Expected: 200 OK

# 2. Authenc
curl http://localhost:8088/health
# Expected: 200 OK

# 3. Portal
curl http://localhost:8080/
# Expected: 200 OK (HTML page)

# 4. Integration check
curl -X POST http://localhost:8088/realms/simpel/protocol/openid-connect/token \
  -d "grant_type=password&client_id=portal&username=admin&password=admin123"
# Expected: {"access_token": "..."}
```

---

## 🎯 Best Practices per Component

### Secreton Best Practices

✅ **DO**:

- Always require bearer token authentication
- Log all operations to audit trail
- Mask secret keys in logs
- Version all secrets
- Export metrics for monitoring
- Implement rate limiting
- Use TLS for all connections

❌ **DON'T**:

- Store secrets in plaintext
- Return secrets in error messages
- Allow unauthenticated access
- Skip audit logging
- Expose admin endpoints publicly

### Authenc Best Practices

✅ **DO**:

- Use strong JWT secrets (32+ bytes)
- Set appropriate token expiration
- Implement refresh token rotation
- Store sensitive config in Secreton
- Use HTTPS for all endpoints
- Implement rate limiting on login
- Log failed authentication attempts

❌ **DON'T**:

- Store passwords in plaintext
- Use weak JWT algorithms (HS256 minimum)
- Allow unlimited login attempts
- Expose admin endpoints without auth
- Return detailed error messages to clients

### Portal Best Practices

✅ **DO**:

- Store JWT token securely (httpOnly cookies preferred, localStorage acceptable)
- Implement token refresh logic
- Clear token on logout
- Validate token expiration client-side
- Use HTTPS in production
- Implement CSRF protection
- Sanitize all user inputs

❌ **DON'T**:

- Store passwords client-side
- Log sensitive data to console
- Trust client-side validation alone
- Expose API keys in source code
- Skip CORS validation

---

## 📊 Performance Targets

### Response Time SLA

| Service  | Operation        | Target | Max   |
| -------- | ---------------- | ------ | ----- |
| Secreton | Get Secret       | 10ms   | 50ms  |
| Secreton | Put Secret       | 20ms   | 100ms |
| Authenc  | Login            | 100ms  | 500ms |
| Authenc  | Token Validation | 5ms    | 20ms  |
| Portal   | Page Load        | 1s     | 3s    |

### Throughput Targets

| Service  | Operations/sec | Notes               |
| -------- | -------------- | ------------------- |
| Secreton | 1,000          | Per instance        |
| Authenc  | 500            | Login operations    |
| Authenc  | 5,000          | Token validations   |
| Portal   | 10,000         | Page views (cached) |

---

## 🎓 Training & Onboarding

### Developer Onboarding Path

1. **Week 1**: Understand architecture & roles
2. **Week 2**: Set up local development environment
3. **Week 3**: Run integration tests, modify code
4. **Week 4**: Deploy to staging, monitor metrics

### Required Knowledge

**For Secreton Development**:

- Rust async programming
- Cryptography basics
- Vault concepts (versioning, audit)
- HTTP/REST APIs

**For Authenc Development**:

- OAuth2/OIDC protocols
- JWT tokens
- RBAC concepts
- Database design

**For Portal Development**:

- Leptos framework
- WASM/WebAssembly
- OAuth2 client flows
- React-like patterns (signals, effects)

---

## 📞 Support & Escalation

### Issue Resolution Matrix

| Issue Type           | First Contact | Escalation | SLA     |
| -------------------- | ------------- | ---------- | ------- |
| Secreton down        | DevOps        | Backend    | 15 min  |
| Authenc down         | DevOps        | Backend    | 15 min  |
| Portal down          | Frontend      | DevOps     | 30 min  |
| Login failures       | Backend       | Authenc    | 1 hour  |
| Secret access denied | Backend       | Secreton   | 1 hour  |
| Performance issues   | DevOps        | Backend    | 4 hours |

---

**Document Version**: 1.0.0
**Last Updated**: 2025-10-07
**Authors**: SIMPelv2 Integration Team
