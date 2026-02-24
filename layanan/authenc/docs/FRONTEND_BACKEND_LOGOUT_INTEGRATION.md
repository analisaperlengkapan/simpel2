# Frontend-Backend Logout Integration Guide

## Status Saat Ini

### ❌ Masalah yang Ditemukan

1. **Frontend tidak memanggil backend logout endpoint** - Portal dan microfrontend hanya membersihkan localStorage tanpa menginformasikan backend
2. **Tidak ada invalidasi session di server** - Session di backend tetap aktif meskipun user sudah logout di frontend
3. **Tidak ada audit trail untuk logout** - Logout event tidak tercatat di audit log
4. **SSO cookie tidak dihapus dengan benar** - Cookie AUTHENC_SSO masih ada setelah logout

### ✅ Yang Sudah Ada

**Backend (Authenc)**:
- ✅ Endpoint `/oidc/logout` sudah diimplementasi (Task 10.7)
- ✅ Session invalidation di server
- ✅ SSO cookie deletion
- ✅ Logout event publishing
- ✅ Federated logout propagation

**Frontend (Portal & Shared)**:
- ✅ `AuthService::logout()` untuk clear localStorage
- ✅ `AuthContext::logout()` di shared library
- ✅ Cross-tab logout synchronization
- ✅ Logout button component

## Solusi: Integrasi Frontend-Backend

### 1. Update Frontend Logout Flow

#### A. Update `antarmuka/portal/src/features/auth.rs`

```rust
/// Logout user - calls backend and clears local state
pub async fn logout() -> Result<(), String> {
    // 1. Call backend logout endpoint
    #[cfg(target_arch = "wasm32")]
    {
        let authenc_url = std::env::var("AUTHENC_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        // Get current URL for post_logout_redirect_uri
        let redirect_uri = if let Some(window) = web_sys::window() {
            window.location().origin().ok()
                .map(|origin| format!("{}/logged-out", origin))
                .unwrap_or_else(|| format!("{}/logged-out", authenc_url))
        } else {
            format!("{}/logged-out", authenc_url)
        };

        // Call backend logout endpoint
        let logout_url = format!(
            "{}/v1/oidc/logout?post_logout_redirect_uri={}",
            authenc_url,
            urlencoding::encode(&redirect_uri)
        );

        // Make request with credentials to include SSO cookie
        let request = gloo_net::http::Request::get(&logout_url)
            .credentials(gloo_net::http::RequestCredentials::Include)
            .build()
            .map_err(|e| format!("Failed to build logout request: {}", e))?;

        // Send request (don't wait for response, redirect will happen)
        let _ = request.send().await;
    }

    // 2. Clear localStorage
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.remove_item("user_session");
            let _ = storage.remove_item("auth_token");
            let _ = storage.remove_item("refresh_token");
        }
    }

    Ok(())
}
```

#### B. Update `antarmuka/shared/src/hooks/use_auth.rs`

```rust
/// Logout user and redirect to portal
pub fn logout(&self) {
    // Clear session immediately for responsive UI
    self.session.set(None);

    // Clear localStorage
    if let Some(storage) = window().and_then(|w| w.local_storage().ok()).flatten() {
        let _ = storage.remove_item("user_session");
        let _ = storage.remove_item("auth_token");
        let _ = storage.remove_item("refresh_token");
    }

    // Call backend logout endpoint
    spawn_local(async move {
        let authenc_url = std::env::var("AUTHENC_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let portal_url = get_portal_url();
        let redirect_uri = format!("{}/logged-out", portal_url);

        let logout_url = format!(
            "{}/v1/oidc/logout?post_logout_redirect_uri={}",
            authenc_url,
            urlencoding::encode(&redirect_uri)
        );

        // Make request with credentials to include SSO cookie
        if let Ok(request) = gloo_net::http::Request::get(&logout_url)
            .credentials(gloo_net::http::RequestCredentials::Include)
            .build()
        {
            let _ = request.send().await;
        }

        // Redirect will be handled by backend
    });
}
```

### 2. Tambahkan Logged Out Page

#### `antarmuka/portal/src/pages/logged_out.rs`

```rust
use leptos::prelude::*;

/// Logged out confirmation page
#[component]
pub fn LoggedOutPage() -> impl IntoView {
    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
            <div class="max-w-md w-full space-y-8 p-8">
                <div class="text-center">
                    <i class="fas fa-check-circle text-6xl text-green-500 mb-4"></i>
                    <h2 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                        "Anda Telah Keluar"
                    </h2>
                    <p class="text-gray-600 dark:text-gray-400 mb-8">
                        "Sesi Anda telah berakhir dengan aman. Terima kasih telah menggunakan SIMPelv2."
                    </p>
                    <a
                        href="/login"
                        class="inline-flex items-center justify-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                    >
                        <i class="fas fa-sign-in-alt mr-2"></i>
                        "Masuk Kembali"
                    </a>
                </div>
            </div>
        </div>
    }
}
```

### 3. Update Environment Variables

#### `.env` (Development)

```bash
# Authenc backend URL
AUTHENC_URL=http://localhost:8080

# Portal URL
PORTAL_URL=http://localhost:3000
```

#### Production Configuration

```bash
# Authenc backend URL
AUTHENC_URL=https://authenc.simpel.kejaksaan.go.id

# Portal URL
PORTAL_URL=https://portal.simpel.kejaksaan.go.id
```

### 4. Update CORS Configuration di Backend

Pastikan backend authenc mengizinkan credentials dari Portal:

#### `layanan/authenc/src/middleware/cors_axum.rs`

```rust
// Allow credentials for SSO cookie
.allow_credentials(true)

// Allow Portal origin
.allow_origin([
    "http://localhost:3000".parse().unwrap(),
    "https://portal.simpel.kejaksaan.go.id".parse().unwrap(),
])
```

## Flow Diagram

### Current Flow (❌ Incomplete)

```
User clicks Logout
    ↓
Frontend clears localStorage
    ↓
Frontend redirects to /login
    ↓
❌ Backend session still active
❌ SSO cookie still valid
❌ No audit trail
```

### Recommended Flow (✅ Complete)

```
User clicks Logout
    ↓
Frontend clears localStorage (immediate UI feedback)
    ↓
Frontend calls /oidc/logout with credentials
    ↓
Backend receives logout request
    ↓
Backend invalidates all user sessions
    ↓
Backend clears SSO cookie (AUTHENC_SSO)
    ↓
Backend publishes logout event (audit trail)
    ↓
Backend propagates to federated providers (if applicable)
    ↓
Backend redirects to post_logout_redirect_uri
    ↓
User sees "Logged Out" confirmation page
    ↓
User can login again
```

## Security Considerations

### 1. CSRF Protection

Logout endpoint menggunakan GET method (sesuai OIDC spec) dan tidak memerlukan CSRF token karena:
- Tidak mengubah state yang sensitif (hanya menghapus session)
- Menggunakan SSO cookie untuk identifikasi
- Redirect URI divalidasi di backend

### 2. Cookie Security

SSO cookie harus memiliki flags:
- `HttpOnly=true` - Tidak bisa diakses JavaScript
- `Secure=true` - Hanya dikirim via HTTPS (production)
- `SameSite=Lax` - Proteksi CSRF
- `Domain=.simpel.kejaksaan.go.id` - Shared across subdomains

### 3. Redirect URI Validation

Backend memvalidasi `post_logout_redirect_uri` terhadap whitelist:
```rust
let allowed_patterns = vec![
    "http://localhost",
    "https://simpel.kejaksaan.go.id",
    "https://portal.simpel.kejaksaan.go.id",
    "https://authenc.simpel.kejaksaan.go.id",
];
```

## Testing

### Manual Testing

1. **Login ke Portal**
   ```bash
   # Navigate to http://localhost:3000/login
   # Login dengan credentials
   ```

2. **Verify SSO Cookie**
   ```bash
   # Check browser DevTools → Application → Cookies
   # Should see AUTHENC_SSO cookie
   ```

3. **Logout**
   ```bash
   # Click logout button
   # Should redirect to /logged-out
   ```

4. **Verify Cleanup**
   ```bash
   # Check browser DevTools:
   # - localStorage should be empty
   # - AUTHENC_SSO cookie should be deleted
   # - Session should be invalid in backend
   ```

### Automated Testing

####ntend Test (Leptos)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logout_clears_storage() {
        // Setup
        let auth = AuthContext::new();

        // Login
        auth.session.set(Some(UserSession::default()));

        // Logout
        auth.logout();

        // Verify
        assert!(auth.session.get().is_none());
    }
}
```

#### Backend Test (Rust)

```rust
#[tokio::test]
async fn test_logout_invalidates_session() {
    let state = create_test_state();
    let mut params = std::collections::HashMap::new();
    params.insert(
        "post_logout_redirect_uri".to_string(),
        "https://portal.simpel.kejaksaan.go.id/logged-out".to_string(),
    );

    // Create session
    let session = create_test_session();
    let mut headers = HeaderMap::new();
    state.cookie_manager.add_cookie_header(&mut headers, &session).unwrap();

    // Logout
    let result = oidc_logout_with_sso(
        State(state.clone()),
        headers,
        Query(params),
    ).await;

    assert!(result.is_ok());

    // Verify session invalidated
    let sessions = state.session_store.unwrap()
        .get_user_sessions(session.user_id).await.unwrap();
    assert_eq!(sessions.len(), 0);
}
```

## Migration Plan

### Phase 1: Backend Ready (✅ Completed)
- ✅ Implement `/oidc/logout` endpoint
- ✅ Session invalidation
- ✅ Event publishing
- ✅ Cookie deletion

### Phase 2: Frontend Integration (🔨 To Do)
1. Update `AuthService::logout()` di Portal
2. Update `AuthContext::logout()` di shared library
3. Tambahkan logged out page
4. Update environment variables
5. Test integration

### Phase 3: Testing & Deployment
1. Manual testing di development
2. Automated tests
3. Staging deployment
4. Production deployment

## Checklist

### Backend (Authenc)
- [x] Logout endpoint implemented
- [x] Session invalidation
- [x] SSO cookie deletion
- [x] Event publishing
- [x] Federated logout support
- [ ] CORS configuration updated

### Frontend (Portal)
- [ ] Update `AuthService::logout()` to call backend
- [ ] Add logged out confirmation page
- [ ] Update environment variables
- [ ] Add error handling for logout failures
- [ ] Test cross-tab logout synchronization

### Frontend (Shared)
- [ ] Update `AuthContext::logout()` to call backend
- [ ] Update `LogoutButton` component
- [ ] Add loading state during logout
- [ ] Test in all microfrontends

### Documentation
- [x] Integration guide created
- [ ] API documentation updated
- [ ] User guide updated
- [ ] Deployment guide updated

## Kesimpulan

Implementasi logout backend sudah lengkap dan aman, tetapi **frontend belum terintegrasi dengan backend**. Diperlukan update di:

1. **Portal** (`antarmuka/portal/src/features/auth.rs`) - Panggil `/oidc/logout` sebelum clear localStorage
2. **Shared Library** (`antarmuka/shared/src/hooks/use_auth.rs`) - Sama seperti Portal
3. **Logged Out Page** - Tambahkan halaman konfirmasi logout
4. **Environment Config** - Set AUTHENC_URL di semua environment

Tanpa integrasi ini, logout hanya terjadi di frontend (localStorage) tetapi session di backend tetap aktif, yang merupakan **security risk**.

## Rekomendasi Prioritas

**HIGH PRIORITY** (Harus segera):
1. Update frontend logout untuk panggil backend endpoint
2. Test integrasi end-to-end
3. Deploy ke staging untuk testing

**MEDIUM PRIORITY** (Dalam 1-2 minggu):
1. Tambahkan logged out confirmation page
2. Improve error handling
3. Add loading states

**LOW PRIORITY** (Nice to have):
1. Add logout analytics
2. Add logout reason tracking
3. Add "logout from all devices" feature

