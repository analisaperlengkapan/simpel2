# Login/Logout Synchronization Architecture

## Overview

Dokumentasi ini menjelaskan bagaimana Portal dan semua microfrontend (Badiklat, Datun, Intel, Pembinaan, dll) melakukan sinkronisasi login/logout menggunakan SSO cookie dan shared library.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Authenc Backend                          │
│  - /oidc/login (OAuth2/OIDC)                               │
│  - /oidc/logout                                            │
│  - Sets AUTHENC_SSO cookie (HttpOnly, Secure, SameSite)   │
│  - Domain: .simpel.kejaksaan.go.id                        │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   │ SSO Cookie
                   │ (shared across subdomains)
                   │
    ┌──────────────┴──────────────┐
    │                             │
    ▼                             ▼
┌─────────┐                  ┌──────────────┐
│ Portal  │                  │ Microfrontend│
│         │                  │ (Badiklat,   │
│ Login   │◄────────────────►│  Datun,      │
│ Page    │   Shared Auth    │  Intel, etc) │
└─────────┘                  └──────────────┘
    │                             │
    │                             │
    └──────────┬──────────────────┘
               │
               ▼
    ┌─────────────────────┐
    │  Shared Library     │
    │  - use_auth()       │
    │  - SsoCookieReader  │
    │  - AuthContext      │
    │  - UserSession      │
    └─────────────────────┘
```

## Components

### 1. Backend (Authenc)

**Responsibility**: Authentication, Authorization, Session Management

**Endpoints**:
- `POST /oidc/token` - Login (OAuth2 password grant)
- `GET /oidc/logout` - Logout with session termination
- `POST /oidc/refresh` - Token refresh
- `GET /.well-known/openid-configuration` - OIDC discovery

**SSO Cookie**:
```rust
Cookie {
    name: "AUTHENC_SSO",
    domain: ".simpel.kejaksaan.go.id",  // Shared across subdomains
    path: "/",
    max_age: 3600,  // 1 hour
    secure: true,   // HTTPS only
    http_only: true,  // Not accessible via JavaScript
    same_site: "Lax"  // CSRF protection
}
```

**Cookie Content** (Base64 encoded JSON):
```json
{
  "session_id": "uuid",
  "user_id": "uuid",
  "username": "string",
  "email": "string",
  "roles": ["admin", "user"],
  "created_at": "ISO8601",
  "expires_at": "ISO8601",
  "ip_address": "string",
  "user_agent": "string"
}
```

### 2. Portal (antarmuka/portal/)

**Responsibility**: Login UI, Session Initialization, Gateway

**Login Flow**:
```rust
// antarmuka/portal/src/features/auth.rs
pub async fn login(credentials: LoginCredentials) -> LoginResult {
    // 1. Validate credentials
    // 2. Call authenc /oidc/token
    // 3. Receive access_token + SSO cookie
    // 4. Decode JWT to UserSession
    // 5. Save to localStorage
    // 6. Redirect to dashboard
}
```

**Logout Flow**:
```rust
// antarmuka/portal/src/features/auth.rs
pub fn logout() {
    // 1. Clear localStorage (instant UI feedback)
    // 2. Call authenc /oidc/logout (async)
    // 3. Backend deletes SSO cookie
    // 4. Backend invalidates sessions
    // 5. Redirect to /logged-out
}
```

**Storage**:
- `localStorage.user_session` - UserSession object
- `localStorage.auth_token` - JWT access token
- `localStorage.refresh_token` - JWT refresh token

### 3. Shared Library (antarmuka/shared/)

**Responsibility**: Auth State Management, SSO Cookie Reading, Cross-App Sync

**Key Components**:

#### a. `use_auth()` Hook
```rust
// antarmuka/shared/src/hooks/use_auth.rs
pub fn use_auth() -> AuthContext {
    // Provides:
    // - session: RwSignal<Option<UserSession>>
    // - is_authenticated()
    // - has_permission(permission)
    // - logout()
    // - redirect_to_login()
}
```

#### b. SSO Cookie Reader
```rust
// antarmuka/shared/src/utils/sso_cookie.rs
pub struct SsoCookieReader {
    // Reads AUTHENC_SSO cookie
    // Decodes base64 JSON
    // Validates expiration
    // Converts to UserSession
}
```

#### c. Storage Event Listener
```rust
// Cross-tab synchronization
window.addEventListener("storage", (event) => {
    if (event.key === "user_session") {
        // Session changed in another tab
        // Update local session
    }
    if (event.key === "logout_event") {
        // Logout triggered in another tab
        // Clear local session
    }
});
```

### 4. Microfrontends (Badiklat, Datun, Intel, etc)

**Responsibility**: Feature-specific UI, Auth State Consumption

**Usage**:
```rust
// antarmuka/badiklat/src/app.rs
use shared_microfrontend::prelude::*;

#[component]
fn BadiklatApp() -> impl IntoView {
    let auth = use_auth();

    view! {
        <Show when=move || auth.is_authenticated()>
            // Protected content
        </Show>
        <Show when=move || !auth.is_authenticated()>
            <LoginRedirectPage />
        </Show>
    }
}
```

## Login Flow (Complete)

### Step-by-Step

1. **User Opens Portal**
   ```
   https://portal.simpel.kejaksaan.go.id/login
   ```

2. **User Enters Credentials**
   - Username: `user@kejaksaan.go.id`
   - Password: `********`
   - CAPTCHA: `token123`

3. **Portal Calls Backend**
   ```http
   POST https://authenc.simpel.kejaksaan.go.id/oidc/token
   Content-Type: application/x-www-form-urlencoded

   grant_type=password
   &client_id=portal
   &username=user@kejaksaan.go.id
   &password=********
   &captcha_token=token123
   ```

4. **Backend Validates & Responds**
   ```http
   HTTP/1.1 200 OK
   Set-Cookie: AUTHENC_SSO=base64_encoded_session; Domain=.simpel.kejaksaan.go.id; ...
   Content-Type: application/json

   {
     "access_token": "eyJhbGc...",
     "refresh_token": "eyJhbGc...",
     "token_type": "Bearer",
     "expires_in": 3600,
     "id_token": "eyJhbGc..."
   }
   ```

5. **Portal Processes Response**
   - Decode JWT to extract user info
   - Create UserSession object
   - Save to localStorage
   - SSO cookie automatically saved by browser

6. **Portal Redirects**
   ```
   https://portal.simpel.kejaksaan.go.id/dashboard
   ```

7. **User Opens Microfrontend**
   ```
   https://badiklat.simpel.kejaksaan.go.id/
   ```

8. **Microfrontend Checks Auth**
   ```rust
   let auth = use_auth();

   // Option 1: Read from localStorage
   if let Some(session) = load_from_storage("user_session") {
       auth.session.set(Some(session));
   }

   // Option 2: Read from SSO cookie (if localStorage empty)
   let reader = SsoCookieReader::new();
   if let Some(sso_session) = reader.read_session() {
       let user_session = UserSession::from(sso_session);
       auth.session.set(Some(user_session));
   }
   ```

9. **Microfrontend Shows Content**
   - User is authenticated
   - No need to login again
   - SSO cookie shared across subdomains

## Logout Flow (Complete)

### Step-by-Step

1. **User Clicks Logout** (in any app)
   - Portal, Badiklat, Datun, etc

2. **Frontend Clears localStorage**
   ```rust
   localStorage.remove_item("user_session");
   localStorage.remove_item("auth_token");
   localStorage.remove_item("refresh_token");
   ```

3. **Frontend Calls Backend**
   ```http
   GET https://authenc.simpel.kejaksaan.go.id/v1/oidc/logout
       ?post_logout_redirect_uri=https://portal.simpel.kejaksaan.go.id/logged-out
   Cookie: AUTHENC_SSO=base64_encoded_session
   ```

4. **Backend Processes Logout**
   - Validate redirect URI
   - Invalidate all user sessions in database
   - Delete SSO cookie
   - Publish logout event to Kafka
   - Propagate to federated providers (if any)

5. **Backend Responds**
   ```http
   HTTP/1.1 302 Found
   Location: https://portal.simpel.kejaksaan.go.id/logged-out
   Set-Cookie: AUTHENC_SSO=; Max-Age=0; Domain=.simpel.kejaksaan.go.id; ...
   ```

6. **Browser Redirects**
   ```
   https://portal.simpel.kejaksaan.go.id/logged-out
   ```

7. **Cross-Tab Logout** (automatic)
   ```rust
   // In other tabs/windows
   window.addEventListener("storage", (event) => {
       if (event.key === "user_session" && event.newValue === null) {
           // Session cleared in another tab
           auth.session.set(None);
       }
   });
   ```

8. **All Apps Logged Out**
   - Portal: Shows logged out page
   - Badiklat: Redirects to login
   - Datun: Redirects to login
   - All other microfrontends: Redirects to login

## Session Synchronization

### Scenario 1: Login in Portal, Access Microfrontend

```
1. Login in Portal
   ├─ localStorage: user_session ✓
   ├─ Cookie: AUTHENC_SSO ✓
   └─ Backend: session active ✓

2. Open Badiklat
   ├─ Check localStorage: user_session ✓
   ├─ OR check cookie: AUTHENC_SSO ✓
   └─ Result: Authenticated ✓
```

### Scenario 2: Logout in Microfrontend, Portal Updates

```
1. Logout in Badiklat
   ├─ Clear localStorage
   ├─ Call backend /oidc/logout
   └─ Trigger storage event

2. Portal Detects
   ├─ Storage event listener fires
   ├─ Detects user_session removed
   └─ Updates auth state to logged out
```

### Scenario 3: Session Expires

```
1. Session Timeout (1 hour)
   ├─ Backend: session expired
   ├─ Cookie: still exists but invalid
   └─ localStorage: still exists but expired

2. User Makes Request
   ├─ Backend returns 401 Unauthorized
   ├─ Frontend detects expired session
   ├─ Clear localStorage
   └─ Redirect to login
```

### Scenario 4: Token Refresh

```
1. Access Token Expires (1 hour)
   ├─ Frontend detects expiration
   └─ Calls /oidc/refresh with refresh_token

2. Backend Validates
   ├─ Check refresh token validity
   ├─ Issue new access token
   └─ Update SSO cookie

3. Frontend Updates
   ├─ Save new access_token
   ├─ Update localStorage
   └─ Continue operation
```

## Storage Strategy

### localStorage (Per-Origin)

**Scope**: `https://portal.simpel.kejaksaan.go.id`

**Data**:
```javascript
{
  "user_session": {
    "id": "uuid",
    "username": "user@kejaksaan.go.id",
    "role": "Admin",
    "name": "User Name",
    "email": "user@kejaksaan.go.id",
    "access_token": "eyJhbGc...",
    "refresh_token": "eyJhbGc...",
    "expires_at": 1234567890
  }
}
```

**Purpose**:
- Fast access (no network call)
- Survives page refresh
- Per-origin isolation

### SSO Cookie (Cross-Subdomain)

**Scope**: `.simpel.kejaksaan.go.id` (all subdomains)

**Data**: Base64(JSON)

**Purpose**:
- Cross-subdomain authentication
- HttpOnly (secure from XSS)
- Backend-controlled
- Single source of truth

### Synchronization Priority

1. **localStorage** (fastest, per-origin)
2. **SSO Cookie** (fallback, cross-domain)
3. **Backend API** (validation, refresh)

## Security Considerations

### 1. Cookie Security

```rust
Cookie {
    http_only: true,    // ✓ Prevents XSS access
    secure: true,       // ✓ HTTPS only
    same_site: "Lax",   // ✓ CSRF protection
    domain: ".simpel.kejaksaan.go.id",  // ✓ Subdomain sharing
    path: "/",          // ✓ All paths
    max_age: 3600       // ✓ 1 hour expiration
}
```

### 2. Token Security

- **Access Token**: Short-lived (1 hour)
- **Refresh Token**: Long-lived (30 days), rotated on use
- **ID Token**: Contains user claims, signed with Ed25519

### 3. CORS Configuration

```rust
// Backend CORS
.allow_origin([
    "https://portal.simpel.kejaksaan.go.id",
    "https://badiklat.simpel.kejaksaan.go.id",
    "https://datun.simpel.kejaksaan.go.id",
    // ... all microfrontends
])
.allow_credentials(true)  // Required for cookies
.allow_methods([Method::GET, Method::POST])
```

### 4. Redirect URI Validation

```rust
// Backend validates redirect URIs
let allowed_patterns = vec![
    "https://portal.simpel.kejaksaan.go.id",
    "https://*.simpel.kejaksaan.go.id",  // All subdomains
];
```

## No Duplication

### Clear Separation

| Component | Responsibility | Storage |
|-----------|---------------|---------|
| **Backend** | Authentication, Session Management | Database, Redis |
| **Portal** | Login UI, Inession | localStorage |
| **Shared** | Auth State, SSO Reading | Context API |
| **Microfrontends** | Feature UI, Auth Consumption | Context API |

### Shared Code

**What's Shared** (in `antarmuka/shared/`):
- ✓ `use_auth()` hook
- ✓ `AuthContext` type
- ✓ `UserSession` type
- ✓ `SsoCookieReader`
- ✓ Storage event listeners
- ✓ Auth components (LoginRedirectPage, LogoutButton, etc)

**What's NOT Shared** (app-specific):
- ✗ Login form UI (Portal only)
- ✗ Login API call (Portal only)
- ✗ Feature-specific auth checks (per microfrontend)

## Testing

### Test Scenarios

1. **Login in Portal, Access Microfrontend**
   - ✓ Should be authenticated
   - ✓ Should see user info
   - ✓ Should not need to login again

2. **Logout in Microfrontend, Check Portal**
   - ✓ Portal should detect logout
   - ✓ All tabs should logout
   - ✓ SSO cookie should be deleted

3. **Session Expiration**
   - ✓ Should redirect to login
   - ✓ Should clear localStorage
   - ✓ Should show appropriate message

4. **Token Refresh**
   - ✓ Should refresh automatically
   - ✓ Should not interrupt user
   - ✓ Should update tokens

5. **Cross-Tab Sync**
   - ✓ Login in tab 1, tab 2 updates
   - ✓ Logout in tab 2, tab 1 updates
   - ✓ Session change propagates

## Troubleshooting

### Issue: Microfrontend Not Detecting Login

**Symptoms**: User logged in Portal but microfrontend shows login page

**Causes**:
1. localStorage not shared (different origin)
2. SSO cookie not readable (domain mismatch)
3. Cookie not sent (CORS credentials)

**Solutions**:
1. Check cookie domain: `.simpel.kejaksaan.go.id`
2. Check CORS: `credentials: "include"`
3. Check SsoCookieReader implementation

### Issue: Logout Not Propagating

**Symptoms**: Logout in one app, others still authenticated

**Causes**:
1. Storage event not firing
2. Backend not called
3. Cookie not deleted

**Solutions**:
1. Check storage event listener
2. Verify backend logout call
3. Check cookie deletion in DevTools

### Issue: Session Expires Too Quickly

**Symptoms**: User logged out after short time

**Causes**:
1. Token expiration too short
2. No token refresh
3. Backend session timeout

**Solutions**:
1. Increase token expiration
2. Implement token refresh
3. Adjust backend session timeout

## Conclusion

**Perfect Sinergi** ✓:
- Portal handles login UI
- Backend handles authentication
- Shared library provides auth state
- Microfrontends consume auth state
- SSO cookie enables cross-domain auth
- localStorage provides fast access
- Storage events enable cross-tab sync

**No Duplication** ✓:
- Each component has clear responsibility
- Shared code in shared library
- App-specific code in apps
- Backend is single source of truth

**Secure & Performant** ✓:
- HttpOnly cookies prevent XSS
- CORS properly configured
- Tokens short-lived
- Session properly managed
- Fast localStorage access
- Automatic token refresh

