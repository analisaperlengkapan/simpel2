# Implementasi Integrasi Logout Frontend-Backend

## File yang Perlu Diupdate

### 1. Portal Auth Service

**File**: `antarmuka/portal/src/features/auth.rs`

**Update method `logout()`**:

```rust
/// Logout user - calls backend and clears local state
pub fn logout() {
    // 1. Clear localStorage immediately for responsive UI
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

    // 2. Call backend logout endpoint asynchronously
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen_futures::spawn_local;

        spawn_local(async move {
            let authenc_url = std::env::var(_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string());

            // Get portal URL for redirect
            let portal_url = std::env::var("PORTAL_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string());
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
                // Send request (backend will handle redirect)
                let _ = request.send().await;
            }
        });
    }
}
```

### 2. Shared Auth Hook

**File**: `antarmuka/shared/src/hooks/use_auth.rs`

**Update method `logout()`**:

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
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen_futures::spawn_local;

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
        });
    }
}
```

### 3. Logged Out Page (New File)

**File**: `antarmuka/portal/src/pages/logged_out.rs`

```rust
use leptos::prelude::*;

/// Logged out confirmation page
///
/// Displayed after successful logout to confirm session termination
#[component]
pub fn LoggedOutPage() -> impl IntoView {
    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
            <div class="max-w-md w-full space-y-8 p-8">
                <div class="text-center">
                    // Success icon
                    <div class="mx-auto flex items-center justify-center h-16 w-16 rounded-full bg-green-100 dark:bg-green-900 mb-4">
                        <i class="fas fa-check-circle text-3xl text-green-600 dark:text-green-400"></i>
                    </div>

                    // Title
                    <h2 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                        "Anda Telah Keluar"
                    </h2>

                    // Description
                    <p class="text-gray-600 dark:text-gray-400 mb-8">
                        "Sesi Anda telah berakhir dengan aman. Terima kasih telah menggunakan SIMPelv2."
                    </p>

                    // Security notice
                    <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4 mb-8">
                        <div class="flex items-start">
                            <i class="fas fa-info-circle text-blue-500 mt-1 mr-3"></i>
                            <div class="text-left">
                                <p class="text-sm text-blue-800 dark:text-blue-300">
                                    "Untuk keamanan, pastikan Anda menutup browser jika menggunakan komputer bersama."
                                </p>
                            </div>
                        </div>
                    </div>

                    // Action buttons
                    <div class="space-y-3">
                        <a
                            href="/login"
                            class="w-full inline-flex items-center justify-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-bl0 transition-colors"
                        >
                            <i class="fas fa-sign-in-alt mr-2"></i>
                            "Masuk Kembali"
                        </a>

                        <a
                            href="/"
                            class="w-full inline-flex items-center justify-center px-6 py-3 border border-gray-300 dark:border-gray-600 text-base font-medium rounded-md text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
                        >
                            <i class="fas fa-home mr-2"></i>
                            "Kembali ke Beranda"
                        </a>
                    </div>
                </div>
            </div>
        </div>
    }
}
```

### 4. Update Router (Portal)

**File**: `antarmuka/portal/src/app.rs`

**Tambahkan route untuk logged out page**:

```rust
use crate::pages::logged_out::LoggedOutPage;

// Di dalam Router component
<Route path="/logged-out" view=LoggedOutPage />
```

### 5. Update mod.rs untuk export page

**File**: `antarmuka/portal/src/pages/mod.rs`

```rust
pub mod logged_out;
```

## Environment Variables

### Development (.env)

```bash
# Authenc Backend URL
AUTHENC_URL=http://localhost:8080

# Portal URL
PORTAL_URL=http://localhost:3000
```

### Staging (.env.staging)

```bash
# Authenc Backend URL
AUTHENC_URL=https://authenc-staging.simpel.kejaksaan.go.id

# Portal URL
PORTAL_URL=https://portal-staging.simpel.kejaksaan.go.id
```

### Production (.env.production)

```bash
# Authenc Backend URL
AUTHENC_URL=https://authenc.simpel.kejaksaan.go.id

# Portal URL
PORTAL_URL=https://portal.simpel.kejaksaan.go.id
```

## Dependencies yang Diperlukan

Pastikan dependencies berikut ada di `Cargo.toml`:

### Portal (`antarmuka/portal/Cargo.toml`)

```toml
[pendencies]
gloo-net = { version = "0.6", features = ["http"] }
wasm-bindgen-futures = "0.4"
urlencoding = "2.1"
```

### Shared (`antarmuka/shared/Cargo.toml`)

```toml
[dependencies]
gloo-net = { version = "0.6", features = ["http"] }
wasm-bindgen-futures = "0.4"
urlencoding = "2.1"
```

## Testing

### Manual Testing Steps

1. **Build dan Run**
   ```bash
   # Terminal 1: Start backend
   cd infra/authenc
   cargo run

   # Terminal 2: Start portal
   cd antarmuka/portal
   trunk serve
   ```

2. **Test Logout Flow**
   - Login ke portal (http://localhost:3000/login)
   - Verify SSO cookie di DevTools → Application → Cookies
   - Click logout button
   - Verify redirect ke /logged-out
   - Verify SSO cookie terhapus
   - Verify localStorage terhapus

3. **Test Cross-Tab Logout**
   - Open portal di 2 tabs
   - Login di kedua tabs
   - Logout di tab 1
   - Verify tab 2 juga logout (vtorage event)

### Automated Test

**File**: `antarmuka/portal/tests/logout_test.rs`

```rust
#[cfg(test)]
mod logout_tests {
    use super::*;

    #[wasm_bindgen_test]
    async fn test_logout_clears_storage() {
        // Setup
        let storage = web_sys::window()
            .unwrap()
            .local_storage()
            .unwrap()
            .unwrap();

        storage.set_item("user_session", "test_session").unwrap();
        storage.set_item("auth_token", "test_token").unwrap();

        // Execute
        AuthService::logout();

        // Wait for async operations
        gloo_timers::future::TimeoutFuture::new(100).await;

        // Verify
        assert!(storage.get_item("user_session").unwrap().is_none());
        assert!(storage.get_item("auth_token").unwrap().is_none());
    }
}
```

## Rollback Plan

Jika ada masalah setelah deployment:

1. **Revert frontend changes**
   ```bash
   git revert <commit-hash>
   git push
   ```

2. **Backend tetap kompatibel** - Endpoint `/oidc/logout` bersifat optional, frontend lama masih bisa berfungsi (meskipun session tidak ter-invalidate di server)

3. **Gradual rollout** - Deploy ke staging dulu, test thoroughly, baru deploy ke production

## Monitoring

### Metrics to Track

1. **Logout Success Rate**
   - Track berapa % logout yang berhasil call backend
   - Alert jika < 95%

2. **Logout Latency**
   - Track waktu dari click logout sampai redirect
   - Target: < 500ms

3. **Session Cleanup**
   - Track berapa session yang ter-invalidate
   - Alert jika ada session orphan

### Logs to Monitor

```bash
# Backend logs
grep "User logout completed" /var/log/authenc/app.log

# Check logout events in Kafka
kafka-console-consumer --topic user-events --from-beginning | grep USER_LOGOUT
```

## Kesimpulan

Implementasi ini akan:
1. ✅ Memanggil backend logout endpoint untuk invalidate session
2. ✅ Menghapus SSO cookie dengan benar
3. ✅ Mencatat logout event untuk audit trail
4. ✅ Memberikan user feedback yang jelas (logged out page)
5. ✅ Maintain backward compatibility

**Tidak ada duplikasi** - Frontend dan backend punya tanggung jawab yang jelas:
- **Frontend**: UI/UX, localStorage management, user feedback
- **Backend**: Session invalidation, audit logging, security enforcement

**Sinergi yang baik** - Frontend call backend untuk security operations, backend redirect kembali ke frontend untuk user experience.

