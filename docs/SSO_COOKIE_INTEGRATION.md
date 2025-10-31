# SSO Cookie Integration - Backend & Frontend

## Overview

Dokumentasi ini menjelaskan integrasi SSO cookie antara backend (Authenc) dan frontend (Portal + Shared Microfrontend) untuk memastikan sinergi dan menghindari duplikasi.

## Arsitektur

```
┌─────────────────────────────────────────────────────────────┐
│                    Browser (Client)                          │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Cookie: AUTHENC_SSO=<base64_encoded_session>        │  │
│  │  Domain: simpel.kejaksaan.go.id                      │  │
│  │  Secure: true, HttpOnly: true, SameSite: Lax         │  │
│  └──────────────────────────────────────────────────────┘  │
│                           ▲                                   │
│                           │                                   │
│         ┌─────────────────┴─────────────────┐                │
│         │                                   │                │
│Set-Cookie                          Read Cookie           │
│         │                                   │                │
└─────────┼───────────────────────────────────┼────────────────┘
          │                                   │
          │                                   │
┌─────────▼─────────────┐         ┌──────────▼──────────────┐
│   Backend (Authenc)   │         │  Frontend (Portal +     │
│                       │         │  Shared Microfrontend)  │
│  ┌─────────────────┐ │         │  ┌──────────────────┐   │
│  │ SsoCookieManager│ │         │  │ SsoCookieReader  │   │
│  │                 │ │         │  │                  │   │
│  │ - create_cookie │ │         │  │ - read_session   │   │
│  │ - delete_cookie │ │         │  │ - has_valid_     │   │
│  │ - extract_      │ │         │  │   session        │   │
│  │   session       │ │         │  │                  │   │
│  └─────────────────┘ │         │  └──────────────────┘   │
│                       │         │                          │
│  src/utils/          │         │  src/utils/              │
│  sso_cookie.rs       │         │  sso_cookie.rs           │
└───────────────────────┘         └──────────────────────────┘
```

## Backend Implementation

### Lokasi
- **File**: `infra/authenc/src/utils/sso_cookie.rs`
- **Config**: `infra/authenc/src/config/mod.rs`
- **Handlers**: `infra/authenc/src/handlers/oidc_sso.rs`

### Struktur Data

```rust
// Backend: SsoSession
pub struct SsoSession {
    pub session_id: String,      // UUID
    pub user_id: String,
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
```

### Operasi Backend

1. **Create Cookie** (`create_cookie`)
   - Serialize session to JSON
   - Base64 encode
   - Add security attributes (Secure, HttpOnly, SameSite=Lax)
   - Set domain to simpel.kejaksaan.go.id
   - Return Set-Cookie header

2. **Delete Cookie** (`delete_cookie`)
   - Create cookie with Max-Age=0
   - Same security attributes
   - Clear session data

3. **Extract Session** (`extract_session`)
   - Read cookie from request headers
   - Base64 decode
   - Deserialize JSON
   - Validate expiration
   - Return session or error

### Endpoints

- `POST /oidc/token` - Sets SSO cookie on token issuance
- `GET /oidc/authorize` - Sets SSO cookie on authorization
- `POST /oidc/logout` - Clears SSO cookie
- `GET /oidc/validate` - Validates SSO session

## Frontend Implementation

### Lokasi
- **File**: `antarmuka/shared/src/utils/sso_cookie.rs`
- **Integration**: `antarmuka/shared/src/hooks/use_auth.rs`

### Struktur Data

```rust
// Frontend: SsoSession (mirrors backend)
pub struct SsoSession {
    pub session_id: String,
    pub user_id: String,
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub created_at: String,      // ISO 8601 string
    pub expires_at: String,      // ISO 8601 string
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
```

### Operasi Frontend

1. **Read Session** (`read_session`)
   - Read cookie from document.cookie
   - Base64 decode
   - Deserialize JSON
   - Validate expiration
   - Return session or None

2. **Initialize Auth** (`init_auth_from_sso_cookie`)
   - Called on app initialization
   - Read SSO cookie
   - Convert to UserSession
   - Set in auth context
   - Save to localStorage

3. **Monitor Session** (`setup_sso_session_monitor`)
   - Periodic validation (e.g., every 60 seconds)
   - Auto-logout on expiration
   - Cross-tab synchronization

### Integrasi dengan use_auth

```rust
// Conversion from SsoSession to UserSession
impl From<SsoSession> for UserSession {
    fn from(sso: SsoSession) -> Self {
        // Map SSO session to UserSession
        // Determine role from roles array
        // Parse timestamps
        // Set authentication state
    }
}
```

## Sinkronisasi Data

### Backend → Frontend

1. **Login Flow**
   ```
   User Login → Authenc validates → Generate JWT
   → Create SsoSession → Set AUTHENC_SSO cookie
   → Frontend reads cookie → Initialize auth context
   ```

2. **Cookie Attributes**
   - **Name**: `AUTHENC_SSO` (configurable)
   - **Domain**: `simpel.kejaksaan.go.id`
   - **Path**: `/`
   - **Max-Age**: `3600` (1 hour, configurable)
   - **Secure**: `true` (HTTPS only)
   - **HttpOnly**: `true` (no JavaScript access from backend)
   - **SameSite**: `Lax` (CSRF protection)

3. **Session Data**
   - Backend serializes to JSON
   - Base64 encodes for cookie storage
   - Frontend decodes and deserializes
   - **Same structure** ensures compatibility

### Frontend → Backend

1. **Validation Flow**
   ```
   Frontend reads cookie → Validates expiration locally
   → If valid, use session → If expired, redirect to login
   → Backend validates on API calls → Returns 401 if invalid
   ```

2. **Logout Flow**
   ```
   User clicks logout → Frontend calls backend /oidc/logout
   → Backend sets Max-Age=0 → Frontend clears auth context
   → Redirect to login page
   ```

## Keamanan

### Backend Security

1. **Cookie Generation**
   - Secure random session IDs (UUID v4)
   - Signed timestamps (ISO 8601)
   - HttpOnly prevents XSS attacks
   - Secure flag enforces HTTPS
   - SameSite=Lax prevents CSRF

2. **Session Validation**
   - Expiration checking
   - Tamper detection (future: HMAC signature)
   - IP address tracking (optional)
   - User agent validation (optional)

### Frontend Security

1. **Cookie Reading**
   - Read-only access (HttpOnly from backend)
   - Expiration validation
   - Automatic logout on expiration
   - No sensitive data in localStorage

2. **Session Monitoring**
   - Periodic validation
   - Cross-tab synchronization
   - Automatic cleanup

## Konfigurasi

### Backend Configuration

```toml
# config/authenc.toml
[sso_cookie]
name = "AUTHENC_SSO"
domain = "simpel.kejaksaan.go.id"
path = "/"
max_age = 3600
secure = true
http_only = true
same_site = "Lax"
```

### Frontend Configuration

```rust
// Frontend uses same cookie name
const COOKIE_NAME: &str = "AUTHENC_SSO";

// Initialize on app mount
#[component]
pub fn App() -> impl IntoView {
    create_effect(move |_| {
        init_auth_from_sso_cookie();
        setup_sso_session_monitor(60000); // 60 seconds
    });

    view! { /* ... */ }
}
```

## Perbedaan dengan Implementasi Sebelumnya

### Sebelumnya (localStorage only)

```rust
// Frontend only - no backend integration
pub fn use_auth() -> AuthContext {
    let session = load_from_storage("user_session");
    // ...
}
```

**Masalah**:
- Tidak ada sinkronisasi dengan backend
- Session tidak ter-validasi server-side
- Tidak ada SSO antar microfrontend
- Rentan terhadap manipulasi client-side

### Sekarang (SSO Cookie)

```rust
// Backend sets cookie
cookie_manager.add_cookie_header(&mut headers, &sso_session)?;

// Frontend reads cookie
let reader = SsoCookieReader::new();
if let Some(sso_session) = reader.read_session() {
    // Convert and use
}
```

**Keuntungan**:
- Server-side validation
- SSO antar microfrontend (shared domain)
- HttpOnly security
- Automatic expiration
- Cross-tab synchronization

## Tidak Ada Duplikasi

### Backend Responsibilities

✅ **Cookie Creation** - Only backend creates cookies
✅ **Session Management** - Backend manages session lifecycle
✅ **Security Attributes** - Backend sets HttpOnly, Secure, SameSite
✅ **Expiration Control** - Backend controls max-age
✅ **Validation** - Backend validates on API calls

### Frontend Responsibilities

✅ **Cookie Reading** - Frontend reads existing cookies
✅ **Local Validation** - Frontend validates expiration locally
✅ **UI State** - Frontend manages auth UI state
✅ **Monitoring** - Frontend monitors session validity
✅ **Logout Trigger** - Frontend triggers logout flow

### Shared Responsibilities

🔄 **Session Structure** - Same data structure (SsoSession)
🔄 **Expiration Logic** - Both validate expiration
🔄 **Cookie Name** - Same cookie name (AUTHENC_SSO)

## Testing

### Backend Tests

```bash
cd infra/authenc
cargo test --lib utils::sso_cookie::tests
```

### Frontend Tests

```bash
cd antarmuka/shared
cargo test --lib utils::sso_cookie::tests
```

## Migration Path

### Phase 1: Backend Implementation ✅
- Implement SsoCookieManager
- Add OIDC SSO handlers
- Configure cookie settings

### Phase 2: Frontend Implementation ✅
- Implement SsoCookieReader
- Integrate with use_auth
- Add session monitoring

### Phase 3: Portal Integration (Next)
- Update login flow to use SSO cookie
- Update logout flow to clear SSO cookie
- Add session monitoring to portal

### Phase 4: Microfrontend Integration (Next)
- Update each microfrontend to read SSO cookie
- Remove localStorage-only auth
- Test cross-microfrontend SSO

## Troubleshooting

### Cookie Not Set

**Symptom**: Frontend cannot read AUTHENC_SSO cookie

**Checklist**:
- [ ] Backend sets Set-Cookie header
- [ ] Domain matches (simpel.kejaksaan.go.id)
- [ ] HTTPS enabled (Secure flag)
- [ ] Path is correct (/)
- [ ] Browser allows cookies

### Session Expired

**Symptom**: User logged out unexpectedly

**Checklist**:
- [ ] Check max_age configuration (default: 3600s)
- [ ] Verify server time synchronization
- [ ] Check frontend validation logic
- [ ] Review session monitoring interval

### Cross-Origin Issues

**Symptom**: Cookie not shared across subdomains

**Checklist**:
- [ ] Domain set to parent domain (.simpel.kejaksaan.go.id)
- [ ] SameSite=Lax allows cross-subdomain
- [ ] HTTPS on all subdomains
- [ ] Cookie path is /

## Best Practices

1. **Always use HTTPS** - Secure flag requires HTTPS
2. **Short expiration** - 1 hour default, refresh on activity
3. **Monitor sessions** - Periodic validation on frontend
4. **Validate server-side** - Don't trust client-side validation
5. **Log security events** - Track cookie creation/deletion
6. **Test cross-browser** - Verify cookie behavior
7. **Document changes** - Keep this doc updated

## References

- Backend Implementation: `infra/authenc/TASK_10.4_SSO_COOKIE_IMPLEMENTATION.md`
- OWASP Cookie Security: https://owasp.org/www-community/controls/SecureCookieAttribute
- RFC 6265 (HTTP Cookies): https://tools.ietf.org/html/rfc6265
- SameSite Cookies: https://web.dev/samesite-cookies-explained/

## Status

✅ **Backend Implementation** - Complete
✅ **Frontend Implementation** - Complete
⏳ **Portal Integration** - Pending
⏳ **Microfrontend Integration** - Pending
⏳ **Production Testing** - Pending

