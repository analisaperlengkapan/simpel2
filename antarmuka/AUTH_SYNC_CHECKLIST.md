# Auth Synchronization Checklist

## ✅ Verification Checklist

Gunakan checklist ini untuk memastikan login/logout sudah sinkron antara Portal dan semua microfrontend.

## 1. Backend (Authenc) ✅

- [x] `/oidc/token` endpoint untuk login
- [x] `/oidc/logout` endpoint untuk logout
- [x] `/oidc/refresh` endpoint untuk token refresh
- [x] SSO cookie dengan domain `.simpel.kejaksaan.go.id`
- [x] Cookie flags: HttpOnly, Secure, SameSite=Lax
- [x] Session invalidation on logout
- [x] Event publishing untuk audit
- [x] CORS configuration untuk semua microfrontend origins

## 2. Shared Library (antarmuka/shared/) ✅

- [x] `use_auth()` hook tersedia
- [x] `AuthContext` dengan session management
- [x] `UserSession` type definition
- [x] `SsoCookieReader` untuk read SSO cookie
- [x] Storage event listener untuk cross-tab sync
- [x] `logout()` method calls backend
- [x] Auth components (LoginRedirectPage, LogoutButton, etc)

## 3. Portal (antarmuka/portal/) ✅

- [x] Login page dengan form
- [x] `AuthService::login()` calls backend
- [x] `AuthService::logout()` calls backend
- [x] Save session to localStorage
- [x] Logged out confirmation page
- [x] Route `/logged-out` configured
- [x] Environment variables (AUTHENC_URL, PORTAL_URL)
- [x] Cross-tab logout synchronization

## 4. Microfrontends (Badiklat, Datun, Intel, etc)

### Badiklat
- [x] Uses `use_auth()` from shared
- [ ] Tested login flow
- [ ] Tested logout flow
- [ ] Tested cross-tab sync

### Datun
- [ ] Uses `use_auth()` from shared
- [ ] Tested login flow
- [ ] Tested logout flow
- [ ] Tested cross-tab sync

### Intel
- [ ] Uses `use_auth()` from shared
- [ ] Tested login flow
- [ ] Tested logout flow
- [ ] Tested cross-tab sync

### Pembinaan (Keuangan, Perencanaan, Perlengkapan)
- [ ] Uses `use_auth()` from shared
- [ ] Tested login flow
- [ ] Tested logout flow
- [ ] Tested cross-tab sync

### Pemulihan Aset
- [ ] Uses `use_auth()` from shared
- [ ] Tested login flow
- [ ] Tested logout flow
- [ ] Tested cross-tab sync

### Pengawasan
- [ ] Uses `use_auth()` from shared
- [ ] Tested login flow
- [ ] Tested logout flow
- [ ] Tested cross-tab sync

### Pidmil
- [ ] Uses `use_auth()` from shared
- [ ] Tested login flow
- [ ] Tested logout flow
- [ ] Tested cross-tab sync

### Pidsus
- [ ] Uses `use_auth()` from shared
- [ ] Tested login flow
- [ ] Tested logout flow
- [ ] Tested cross-tab sync

### Pidum
- [ ] Uses `use_auth()` from shared
- [ ] Tested login flow
- [ ] Tested logout flow
- [ ] Tested cross-tab sync

## 5. Integration Tests

### Login Flow
- [ ] Login in Portal
- [ ] Open Badiklat → Should be authenticated
- [ ] Open Datun → Should be authenticated
- [ ] Open Intel → Should be authenticated
- [ ] Check SSO cookie in DevTools
- [ ] Check localStorage in each app

### Logout Flow
- [ ] Logout in Portal
- [ ] Check Badiklat → Should redirect to login
- [ ] Check Datun → Should redirect to login
- [ ] Check Intel → Should redirect to login
- [ ] Verify SSO cookie deleted
- [ ] Verify localStorage cleared

### Cross-Tab Sync
- [ ] Open Portal in 2 tabs
- [ ] Login in tab 1
- [ ] Tab 2 should detect login
- [ ] Logout in tab 2
- [ ] Tab 1 should detect logout

### Token Refresh
- [ ] Wait for token expiration (or mock)
- [ ] App should refresh token automatically
- [ ] User should not be interrupted
- [ ] New tokens should be saved

### Session Expiration
- [ ] Wait for session expiration (or mock)
- [ ] App should redirect to login
- [ ] localStorage should be cleared
- [ ] Appropriate message should be shown

## 6. Security Checks

- [ ] SSO cookie has HttpOnly flag
- [ ] SSO cookie has Secure flag (production)
- [ ] SSO cookie has SameSite=Lax
- [ ] SSO cookie domain is `.simpel.kejaksaan.go.id`
- [ ] CORS allows credentials
- [ ] CORS allows all microfrontend origins
- [ ] Redirect URI validation in backend
- [ ] No sensitive data in localStorage
- [ ] Tokens are short-lived
- [ ] Refresh token rotation implemented

## 7. Performance Checks

- [ ] Login completes in < 2 seconds
- [ ] Logout completes in < 500ms
- [ ] localStorage access is fast (< 10ms)
- [ ] SSO cookie read is fast (< 50ms)
- [ ] Token refresh is transparent
- [ ] No unnecessary API calls

## 8. User Experience Checks

- [ ] Login form is user-friendly
- [ ] Error messages are clear
- [ ] Logout confirmation is clear
- [ ] Loading states are shown
- [ ] No flickering during auth check
- [ ] Smooth transitions between states
- [ ] Mobile-friendly

## 9. Documentation

- [x] Architecture documentation created
- [x] Login/logout flow documented
- [x] SSO cookie format documented
- [x] Integration guide created
- [x] Testing guide created
- [ ] API documentation updated
- [ ] User guide created

## 10. Deployment

### Development
- [x] Backend running on localhost:8080
- [x] Portal running on localhost:3000
- [ ] Microfrontends running on different ports
- [x] Environment variables configured

### Staging
- [ ] Backend deployed
- [ ] Portal deployed
- [ ] Microfrontends deployed
- [ ] DNS configured (*.simpel.kejaksaan.go.id)
- [ ] SSL certificates installed
- [ ] Environment variables configured
- [ ] Integration tests passed

### Production
- [ ] Backend deployed
- [ ] Portal deployed
- [ ] Microfrontends deployed
- [ ] DNS configured
- [ ] SSL certificates installed
- [ ] Environment variables configured
- [ ] Load testing completed
- [ ] Security audit completed
- [ ] Monitoring configured
- [ ] Rollback plan ready

## Testing Commands

### Start Backend
```bash
cd infra/authenc
cargo run
```

### Start Portal
```bash
cd antarmuka/portal
trunk serve
```

### Start Microfrontend (Example: Badiklat)
```bash
cd antarmuka/badiklat
trunk serve --port 3001
```

### Check SSO C
javascript
// In browser console
document.cookie.split(';').find(c => c.includes('AUTHENC_SSO'))
```

### Check localStorage
```javascript
// In browser console
localStorage.getItem('user_session')
```

### Trigger Storage Event
```javascript
// In browser console (tab 1)
localStorage.setItem('logout_event', Date.now())
localStorage.removeItem('logout_event')
// Tab 2 should detect and logout
```

## Common Issues & Solutions

### Issue: Microfrontend not detecting login

**Check**:
1. SSO cookie domain: `.simpel.kejaksaan.go.id`
2. CORS credentials: `include`
3. SsoCookieReader implementation

**Fix**:
```rust
// In microfrontend app.rs
let auth = use_auth();

// Try SSO cookie if localStorage empty
if !auth.is_authenticated() {
    let reader = SsoCookieReader::new();
    if let Some(sso_session) = reader.read_session() {
        let user_session = UserSession::from(sso_session);
        auth.session.set(Some(user_session));
    }
}
```

### Issue: Logout not propagating

**Check**:
1. Backend logout endpoint called
2. Storage event listener registered
3. Cookie deletion verified

**Fix**:
```rust
// Ensure storage event listener is set up
setup_storage_listener(session);
```

### Issue: CORS error

**Check**:
1. Backend CORS configuration
2. Frontend credentials setting
3. Origin matching

**Fix**:
```rust
// Backend
.allow_origin(["https://portal.simpel.kejaksaan.go.id"])
.allow_credentials(true)

// Frontend
Request::get(url)
    .credentials(RequestCredentials::Include)
```

## Next Steps

1. **Complete Testing** (Priority: HIGH)
   - Test all microfrontends
   - Verify cross-tab sync
   - Test token refresh
   - Test session expiration

2. **Fix Any Issues** (Priority: HIGH)
   - Address failing tests
   - Fix integration issues
   - Improve error handling

3. **Deploy to Staging** (Priority: MEDIUM)
   - Deploy backend
   - Deploy portal
   - Deploy microfrontends
   - Run integration tests

4. **Production Deployment** (Priority: LOW)
   - After staging validation
   - With rollback plan
   - With monitoring

## Sign-off

- [ ] Backend Developer: _______________
- [ ] Frontend Developer: _______________
- [ ] QA Engineer: _______________
- [ ] Security Reviewer: _______________
- [ ] Product Owner: _______________

## Notes

Add any additional notes or observations here:

---

**Last Updated**: 2025-01-XX
**Status**: In Progress
**Next Review**: After testing completion
