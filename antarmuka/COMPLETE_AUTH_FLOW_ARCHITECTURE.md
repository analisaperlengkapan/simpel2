# Complete Authentication Flow Architecture

## Overview

Dokumentasi lengkap tentang flow authentication yang sebenarnya di SIMPelv2, termasuk CAPTCHA, MFA, OAuth2 redirect, dan sinkronisasi antara Portal, Microfrontend, Authenc, dan Secreton.

## System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Browser                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │  Portal  │  │ Badiklat │  │  Datun   │  │  Intel   │  ...  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘       │
│       │             │              │              │              │
│       └─────────────┴──────────────┴──────────────┘              │
│                     │                                            │
│              ┌──────┴──────┐                                     │
│              │   Shared    │                                     │
│              │   Library   │                                     │
│              └──────┬──────┘                                     │
└─────────────────────┼────────────────────────────────────────────┘
                      │
                      │ HTTPS + Cookies
                      │
┌─────────────────────┼────────────────────────────────────────────┐
│                     ▼                                            │
│              ┌─────────────┐                                     │
│              │   Authenc   │◄──────────────┐                    │
│              │   Backend   │                │                    │
│              └──────┬──────┘                │                    │
│                     │                       │                    │
│                     │ MFA Secrets           │ Secrets            │
│                     ▼                       │                    │
│              ┌─────────────┐                │                    │
│              │  Secreton   │────────────────┘                    │
│              │  (Vault)    │                                     │
│              └─────────────┘                                     │
│                                                                  │
│              Backend Infrastructure                              │
└──────────────────────────────────────────────────────────────────┘
```

## Complete Login Flow

### Scenario 1: Login di Portal (First Time User - Belum Setup MFA)

```
┌──────┐                ┌────────┐              ┌─────────┐           ┌──────────┐
│ User │                │ Portal │              │ Authenc │           │ Secreton │
└──┬───┘                └───┬────┘              └────┬────┘           └────┬─────┘
   │                        │                        │                     │
   │ 1. Open /login         │                        │                     │
   ├───────────────────────>│                        │                     │
   │                        │                        │                     │
   │ 2. Show Login Form     │                        │                     │
   │    + CAPTCHA           │                        │                     │
   │<───────────────────────┤                        │                     │
   │                        │                        │                     │
   │ 3. Complete CAPTCHA    │                        │                     │
   ├───────────────────────>│                        │                     │
   │                        │                        │                     │
   │ 4. Enter credentials   │                        │                     │
   ├───────────────────────>│                        │                     │
   │                        │                        │                     │
   │                        │ 5. POST /oidc/token    │                     │
   │                        │    + username          │                     │
   │                        │    + password          │                     │
   │                        │    + captcha_token     │                     │
   │                        ├───────────────────────>│                     │
   │                        │                        │                     │
   │                        │                        │ 6. Validate CAPTCHA │
   │                        │                        │    Validate creds   │
   │                        │                        │    Check MFA status │
   │                        │                        │                     │
   │                        │ 7. Response:           │                     │
   │                        │    - temp_token        │                     │
   │                        │    - mfa_required=true │                     │
   │                        │    - mfa_setup=true    │                     │
   │                        │<───────────────────────┤                     │
   │                        │                        │                     │
   │ 8. Redirect to         │                        │                     │
   │    /mfa/setup          │                        │                     │
   │<───────────────────────┤                        │                     │
   │                        │                        │                     │
   │ 9. Show MFA Setup      │                        │                     │
   │    (QR Code + Secret)  │                        │                     │
   │<───────────────────────┤                        │                     │
   │                        │                        │                     │
   │                        │ 10. GET /mfa/setup     │                     │
   │                        │     + temp_token       │                     │
   │                        ├───────────────────────>│                     │
   │                        │                        │                     │
   │                        │                        │ 11. Generate secret │
   │                        │                        ├────────────────────>│
   │                        │                        │                     │
   │                        │                        │ 12. Store secret    │
   │                        │                        │<────────────────────┤
   │                        │                        │                     │
   │                        │ 13. QR + Secret        │                     │
   │                        │<───────────────────────┤                     │
   │                        │                        │                     │
   │ 14. Scan QR with       │                        │                     │
   │     Authenticator App  │                        │                     │
   │                        │                        │                     │
   │ 15. Enter OTP code     │                        │                     │
   ├───────────────────────>│                        │                     │
   │                        │                        │                     │
   │                        │ 16. POST /mfa/verify   │                     │
   │                        │     + temp_token       │                     │
   │                        │     + otp_code         │                     │
   │                        ├───────────────────────>│                     │
   │                        │                        │                     │
   │                        │                        │ 17. Verify OTP      │
   │                        │                        ├────────────────────>│
   │                        │                        │                     │
   │                        │                        │ 18. OTP valid       │
   │                        │                        │<────────────────────┤
   │                        │                        │                     │
   │                        │ 19. Response:          │                     │
   │                        │     - access_token     │                     │
   │                        │     - refresh_token    │                     │
   │                        │     - id_token         │                     │
   │                        │     + Set-Cookie:      │                     │
   │                        │       AUTHENC_SSO      │                     │
   │                        │<───────────────────────┤                     │
   │                        │                        │                     │
   │ 20. Save tokens +      │                        │                     │
   │     localStorage       │                        │                     │
   │                        │                        │                     │
   │ 21. Redirect to        │                        │                     │
   │     /dashboard         │                        │                     │
   │<───────────────────────┤                        │                     │
   │                        │                        │                     │
```

### Scenario 2: Login di Portal (User Sudah Setup MFA)

```
┌──────┐                ┌────────┐              ┌─────────┐           ┌──────────┐
│ User │                │ Portal │              │ Authenc │           │ Secreton │
└──┬───┘                └───┬────┘              └────┬────┘           └────┬─────┘
   │                        │                        │                     │
   │ 1-4. Same as above     │                        │                     │
   │      (CAPTCHA + creds) │                        │                     │
   │                        │                        │                     │
   │                        │ 5. POST /oidc/token    │                     │
   │                        ├───────────────────────>│                     │
   │                        │                        │                     │
   │                        │ 6. Response:           │                     │
   │                        │    - temp_token        │                     │
   │                        │    - mfa_required=true │                     │
   │                        │    - mfa_setup=false   │                     │
   │                        │<───────────────────────┤                     │
   │                        │                        │                     │
   │ 7. Redirect to         │                        │                     │
   │    /mfa/verify         │                        │                     │
   │<───────────────────────┤                        │                     │
   │                        │                        │                     │
   │ 8. Show MFA Verify     │                        │                     │
   │    (Enter OTP)         │                        │                     │
   │<───────────────────────┤                        │                     │
   │                        │                        │                     │
   │ 9. Enter OTP code      │                        │                     │
   ├───────────────────────>│                        │                     │
   │                        │                        │                     │
   │                        │ 10. POST /mfa/verify   │                     │
   │                        ├───────────────────────>│                     │
   │                        │                        │                     │
   │                        │                        │ 11. Verify OTP      │
   │                        │                        ├────────────────────>│
   │                        │                        │                     │
   │                        │                        │ 12. OTP valid       │
   │                        │                        │<────────────────────┤
   │                        │                        │                     │
   │                        │ 13. Full tokens +      │                     │
   │                        │     SSO cookie         │                     │
   │                        │<───────────────────────┤                     │
   │                        │                        │                     │
   │ 14. Redirect to        │                        │                     │
   │     /dashboard         │                        │                     │
   │<───────────────────────┤                        │                     │
   │                        │                        │                     │
```

### Scenario 3: Login dari Microfrontend (OAuth2 Redirect Flow)

```
┌──────┐          ┌──────────┐        ┌────────┐        ┌─────────┐
│ User │          │ Badiklat │        │ Portal │        │ Authenc │
└──┬───┘          └────┬─────┘        └───┬────┘        └────┬────┘
   │                   │                   │                  │
   │ 1. Open           │                   │                  │
   │    badiklat.      │                   │                  │
   │    simpel.go.id   │                   │                  │
   ├──────────────────>│                   │                  │
   │                   │                   │                  │
   │ 2. Check auth     │                   │                  │
   │    - No session   │                   │                  │
   │    - No SSO cookie│                   │                  │
   │                   │                   │                  │
   │ 3. Show           │                   │                  │
   │    "Login" button │                   │                  │
   │<──────────────────┤                   │                  │
   │                   │                   │                  │
   │ 4. Click Login    │                   │                  │
   ├──────────────────>│                   │                  │
   │                   │                   │                  │
   │ 5. Redirect to    │                   │                  │
   │    Portal OAuth2  │                   │                  │
   │    /oidc/authorize│                   │                  │
   │    ?client_id=    │                   │                  │
   │     badiklat      │                   │                  │
   │    &redirect_uri= │                   │                  │
   │     badiklat.../  │                   │                  │
   │     callback      │                   │                  │
   │    &state=xyz     │                   │                  │
   │<──────────────────┤                   │                  │
   │                   │                   │                  │
   │ 6. GET /oidc/     │                   │                  │
   │    authorize      │                   │                  │
   ├───────────────────┼──────────────────>│                  │
   │                   │                   │                  │
   │                   │                   │ 7. Check session │
   │                   │                   │    - No session  │
   │                   │                   │                  │
   │ 8. Redirect to    │                   │                  │
   │    /login         │                   │                  │
   │    ?return_url=   │                   │                  │
   │     /oidc/        │                   │                  │
   │     authorize?... │                   │                  │
   │<──────────────────┼───────────────────┤                  │
   │                   │                   │                  │
   │ 9. Show Login     │                   │                  │
   │    + CAPTCHA      │                   │                  │
   │<──────────────────┼───────────────────┤                  │
   │                   │                   │                  │
   │ 10-15. Complete   │                   │                  │
   │        CAPTCHA +  │                   │                  │
   │        Login +    │                   │                  │
   │        MFA        │                   │                  │
   │        (as above) │                   │                  │
   │                   │                   │                  │
   │ 16. After MFA     │                   │                  │
   │     success,      │                   │                  │
   │     redirect to   │                   │                  │
   │     return_url    │                   │                  │
   │<──────────────────┼───────────────────┤                  │
   │                   │                   │                  │
   │ 17. GET /oidc/    │                   │                  │
   │     authorize     │                   │                  │
   │     (with session)│                   │                  │
   ├───────────────────┼──────────────────>│                  │
   │                   │                   │                  │
   │                   │                   │ 18. Generate     │
   │                   │                   │     auth code    │
   │                   │                   │                  │
   │ 19. Redirect to   │                   │                  │
   │     badiklat.../  │                   │                  │
   │     callback      │                   │                  │
   │     ?code=abc     │                   │                  │
   │     &state=xyz    │                   │                  │
   │<──────────────────┼───────────────────┤                  │
   │                   │                   │                  │
   │ 20. GET /callback │                   │                  │
   │     ?code=abc     │                   │                  │
   ├──────────────────>│                   │                  │
   │                   │                   │                  │
   │                   │ 21. POST /oidc/   │                  │
   │                   │     token         │                  │
   │                   │     + code        │                  │
   │                   │     + client_id   │                  │
   │                   ├───────────────────┼─────────────────>│
   │                   │                   │                  │
   │                   │                   │ 22. Validate code│
   │                   │                   │     Issue tokens │
   │                   │                   │     Set SSO      │
   │                   │                   │     cookie       │
   │                   │                   │                  │
   │                   │ 23. Tokens +      │                  │
   │                   │     SSO cookie    │                  │
   │                   │<──────────────────┼──────────────────┤
   │                   │                   │                  │
   │ 24. Save to       │                   │                  │
   │     localStorage  │                   │                  │
   │                   │                   │                  │
   │ 25. Redirect to   │                   │                  │
   │     Badiklat home │                   │                  │
   │<──────────────────┤                   │                  │
   │                   │                   │                  │
```

## Complete Logout Flow

### Scenario 1: Logout dari Portal

```
┌──────┐          ┌────────┐          ┌─────────┐
│ User │          │ Portal │          │ Authenc │
└──┬───┘          └───┬────┘          └────┬────┘
   │                  │                     │
   │ 1. Click Logout  │                     │
   ├─────────────────>│                     │
   │                  │                     │
   │                  │ 2. Clear            │
   │                  │    localStorage     │
   │                  │                     │
   │                  │ 3. Broadcast        │
   │                  │    logout_event     │
   │                  │    (storage event)  │
   │                  │                     │
   │                  │ 4. GET /oidc/logout │
   │                  │    ?post_logout_    │
   │                  │     redirect_uri=   │
   │                  │     portal.../      │
   │                  │     logged-out      │
   │                  ├────────────────────>│
   │                  │                     │
   │                  │                     │ 5. Invalidate
   │                  │                     │    all sessions
   │                  │                     │    Delete SSO
   │                  │                     │    cookie
   │                  │                     │    Publish event
   │                  │                     │
   │                  │ 6. 302 Redirect +   │
   │                  │    Set-Cookie:      │
   │                  │    AUTHENC_SSO=;    │
   │                  │    Max-Age=0        │
   │                  │<────────────────────┤
   │                  │                     │
   │ 7. Show          │                     │
   │    /logged-out   │                     │
   │<─────────────────┤                     │
   │                  │                     │
```

### Scenario 2: Logout Propagation ke Microfrontend

```
┌──────────┐        ┌──────────┐        ┌────────┐
│ Badiklat │        │  Datun   │        │ Portal │
│  (Tab 1) │        │  (Tab 2) │        │ (Tab 3)│
└────┬─────┘        └────┬─────┘        └───┬────┘
     │                   │                   │
     │                   │                   │ User clicks
     │                   │                   │ logout
     │                   │                   │
     │                   │                   │ Storage event:
     │                   │                   │ logout_event
     │                   │                   │ triggered
     │                   │                   │
     │ Detect storage    │ Detect storage    │
     │ event             │ event             │
     │<──────────────────┼───────────────────┤
     │                   │<──────────────────┤
     │                   │                   │
     │ Clear session     │ Clear session     │
     │ Redirect to       │ Redirect to       │
     │ login             │ login             │
     │                   │                   │
```

## Session Synchronization

### SSO Cookie Sharing

```
Domain: .simpel.kejaksaan.go.id

Accessible by:
✓ portal.simpel.kejaksaan.go.id
✓ badiklat.simpel.kejaksaan.go.id
✓ datun.simpel.kejaksaan.go.id
✓ intel.simpel.kejaksaan.go.id
✓ All *.simpel.kejaksaan.go.id subdomains
```

### localStorage Isolation

```
portal.simpel.kejaksaan.go.id
├─ localStorage.user_session
├─ localStorage.auth_token
└─ localStorage.refresh_token

badiklat.simpel.kejaksaan.go.id
├─ localStorage.user_session (separate)
├─ localStorage.auth_token (separate)
└─ localStorage.refresh_token (separate)

Note: localStorage is per-origin, NOT shared
```

### Session Priority

```
1. Check localStorage (fastest, per-origin)
   ↓ If not found
2. Check SSO Cookie (cross-domain)
   ↓ If found
3. Convert to UserSession
   ↓
4. Save to localStorage
   ↓
5. User authenticated
```

## Integration with Secreton

### MFA Secret Storage

```
┌─────────┐                    ┌──────────┐
│ Authenc │                    │ Secreton │
└────┬────┘                    └────┬─────┘
     │                              │
     │ 1. User setup MFA            │
     │                              │
     │ 2. Generate TOTP secret      │
     │                              │
     │ 3. POST /v1/secret/kv/       │
     │    mfa/{user_id}             │
     │    {                         │
     │      "secret": "base32...",  │
     │      "algorithm": "SHA1",    │
     │      "digits": 6,            │
     │      "period": 30            │
     │    }                         │
     ├─────────────────────────────>│
     │                              │
     │                              │ 4. Encrypt secret
     │                              │    Store in vault
     │                              │
     │ 5. 200 OK                    │
     │<─────────────────────────────┤
     │                              │
     │ 6. User verify OTP           │
     │                              │
     │ 7. GET /v1/secret/kv/        │
     │    mfa/{user_id}             │
     ├─────────────────────────────>│
     │                              │
     │                              │ 8. Decrypt secret
     │                              │
     │ 9. Return secret             │
     │<─────────────────────────────┤
     │                              │
     │ 10. Verify OTP with secret   │
     │                              │
```

### Backup Codes Storage

```
Authenc → Secreton
Path: /v1/secret/kv/mfa/{user_id}/backup_codes

Data:
{
  "codes": [
    "ABCD-1234-EFGH",
    "IJKL-5678-MNOP",
    ...
  ],
  "created_at": "2025-01-XX",
  "used": []
}
```

## Security Considerations

### 1. CAPTCHA Protection

- **Always required** on login
- Prevents brute force attacks
- Behavioral analysis enabled
- Accessibility compliant

### 2. MFA Enforcement

- **Required for all users**
- TOTP (Time-based One-Time Password)
- Backup codes available
- Secrets stored in Secreton (encrypted)

### 3. OAuth2 Security

- **Authorization Code Flow** (not implicit)
- State parameter (CSRF protection)
- PKCE support (optional)
- Redirect URI validation

### 4. Cookie Security

```rust
Cookie {
    http_only: true,    // XSS protection
    secure: true,       // HTTPS only
    same_site: "Lax",   // CSRF protection
    domain: ".simpel.kejaksaan.go.id",
    max_age: 3600       // 1 hour
}
```

### 5. Token Security

- **Access Token**: 1 hour (short-lived)
- **Refresh Token**: 30 days (rotated)
- **ID Token**: Contains user claims
- **Temp Token**: 10 minutes (MFA flow)

## Component Responsibilities

| Component | Login | CAPTCHA | MFA | OAuth2 | Logout | Secrets |
|-----------|-------|---------|-----|--------|--------|---------|
| **Portal** | ✅ UI | ✅ UI | ✅ UI | ✅ Authorize | ✅ UI | ❌ |
| **Microfrontend** | ❌ Redirect | ❌ | ❌ | ✅ Callback | ✅ Trigger | ❌ |
| **Shared** | ❌ | ✅ Component | ❌ | ❌ | ✅ Logic | ❌ |
| **Authenc** | ✅ Validate | ✅ Validate | ✅ Validate | ✅ Issue codes | ✅ Invalidate | ✅ Store/Retrieve |
| **Secreton** | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Encrypt/Decrypt |

## No Duplication

### What's Shared

✅ **Shared Library** (`antarmuka/shared/`):
- CAPTCHA component
- Auth hooks (`use_auth`)
- Auth types (`UserSession`, `AuthContext`)
- SSO cookie reader
- Storage event listeners
- Login redirect page
- Logout button

### What's NOT Shared

❌ **Portal Only**:
- Login form UI
- MFA setup UI
- MFA verification UI
- OAuth2 authorization UI
- Backend API calls for login

❌ **Microfrontend Specific**:
- OAuth2 callback handling
- Feature-specific auth checks
- App-specific redirects

❌ **Backend Only**:
- Authentication logic
- Session management
- Token issuance
- MFA validation
- Secret management

## Testing Checklist

### Login Flow
- [ ] Login with CAPTCHA works
- [ ] MFA setup required for new users
- [ ] MFA verification works for existing users
- [ ] Backup codes work
- [ ] OAuth2 redirect from microfrontend works
- [ ] SSO cookie set correctly
- [ ] localStorage saved correctly

### Logout Flow
- [ ] Logout from Portal works
- [ ] Logout from microfrontend works
- [ ] Cross-tab logout works
- [ ] SSO cookie deleted
- [ ] localStorage cleared
- [ ] All sessions invalidated

### Integration
- [ ] Portal ↔ Authenc
- [ ] Microfrontend ↔ Portal ↔ Authenc
- [ ] Authenc ↔ Secreton (MFA secrets)
- [ ] Cross-subdomain cookie sharing
- [ ] Storage event propagation

## Conclusion

**Complete Flow** ✅:
1. CAPTCHA always required
2. MFA setup for new users
3. MFA verification for existing users
4. OAuth2 redirect for microfrontends
5. SSO cookie for cross-domain auth
6. Logout propagation to all apps
7. Secreton for MFA secret storage

**Perfect Sinergi** ✅:
- Portal = Login gateway + OAuth2 authorization
- Microfrontend = OAuth2 client + Auth consumer
- Shared = Reusable components + Auth state
- Authenc = Authentication + Authorization + Session
- Secreton = Secret storage + Encryption

**No Duplication** ✅:
- Each component has clear responsibility
- Shared code properly organized
- No redundant implementations
- Clean separation of concerns
