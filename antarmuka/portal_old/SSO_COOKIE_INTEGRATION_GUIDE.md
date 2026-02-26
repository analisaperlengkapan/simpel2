# Portal SSO Cookie Integration Guide

## Overview

Panduan ini menjelaskan cara mengintegrasikan SSO cookie management ke dalam Portal microfrontend untuk sinkronisasi dengan backend Authenc.

## Quick Start

### 1. Import SSO Cookie Utilities

```rust
use shared_microfrontend::utils::sso_cookie::{
    init_auth_from_sso_cookie,
    setup_sso_session_monitor,
    SsoCookieReader,
};
```

### 2. Initialize on App Mount

```rust
#[component]
pub fn App() -> impl IntoView {
    // Initialize auth from SSO cookie on mount
    create_effect(move |_| {
        // Try to restore session from SSO cookie
        init_auth_from_sso_cookie();

        // Setup periodic session validation (every 60 seconds)
        setup_sso_session_monitor(60000);
    });

    view! {
        <Router>
            <Routes>
                <Route path="/" view=HomePage/>
                <Route path="/login" view=LoginPage/>
                // ... other routes
            </Routes>
        </Router>
    }
}
```

## Login Flow Integration

### Update Login Handler

```rust
use shared_microfrontend::utils::sso_cookie::init_auth_from_sso_cookie;

#[component]
pub fn LoginPage() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            // Call backend login endpoint
            let response = Request::post("https://authenc.simpel.kejaksaan.go.id/v1/oidc/token")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(format!(
                    "grant_type=password&username={}&password={}",
                    urlencoding::encode(&username.get()),
                    urlencoding::encode(&password.get())
                ))
                .send()
                .await;

            match response {
                Ok(resp) if resp.ok() => {
                    // Backend sets AUTHENC_SSO cookie automatically
                    // Initialize auth from the cookie
                    init_auth_from_sso_cookie();

                    // Redirect to dashboard
                    let navigate = use_navigate();
                    navigate("/dashboard", Default::default());
                }
                Ok(resp) => {
                    set_error.set(Some("Login gagal. Periksa username dan password.".to_string()));
                }
                Err(e) => {
                    set_error.set(Some(format!("Error: {}", e)));
                }
            }

            set_loading.set(false);
        });
    };

    view! {
        <form on:submit=on_submit>
            <input
                type="text"
                placeholder="Username"
                prop:value=username
                on:input=move |ev| set_username.set(event_target_value(&ev))
            />
            <input
                type="password"
                placeholder="Password"
                prop:value=password
                on:input=move |ev| set_password.set(event_target_value(&ev))
            />
            <button type="submit" disabled=loading>
                {move || if loading.get() { "Loading..." } else { "Login" }}
            </button>
            <Show when=move || error.get().is_some()>
                <div class="error">{move || error.get()}</div>
            </Show>
        </form>
    }
}
```

## Logout Flow Integration

### Update Logout Handler

```rust
use shared_microfrontend::hooks::use_auth::use_auth;

#[component]
pub fn LogoutButton() -> impl IntoView {
    let auth = use_auth();

    let on_logout = move |_| {
        spawn_local(async move {
            // Call backend logout endpoint to clear SSO cookie
            let _ = Request::post("https://authenc.simpel.kejaksaan.go.id/v1/oidc/logout")
                .send()
                .await;

            // Clear local auth state
            auth.logout();

            // Redirect to login
            let navigate = use_navigate();
            navigate("/login", Default::default());
        });
    };

    view! {
        <button on:click=on_logout>
            "Logout"
        </button>
    }
}
```

## Protected Routes

### Check SSO Session

```rust
use shared_microfrontend::utils::sso_cookie::SsoCookieReader;
use shared_microfrontend::hooks::use_auth::use_auth;

#[component]
pub fn ProtectedRoute(children: ViewFn) -> impl IntoView {
    let auth = use_auth();
    let reader = SsoCookieReader::new();

    // Check both auth context and SSO cookie
    let is_authenticated = move || {
        auth.is_authenticated() || reader.has_valid_session()
    };

    view! {
        <Show
            when=is_authenticated
            fallback=|| view! { <Redirect path="/login"/> }
        >
            {children()}
        </Show>
    }
}
```

## Session Monitoring

### Custom Session Monitor

```rust
use shared_microfrontend::utils::sso_cookie::SsoCookieReader;
use shared_microfrontend::hooks::use_auth::use_auth;

#[component]
pub fn SessionMonitor() -> impl IntoView {
    let auth = use_auth();
    let reader = SsoCookieReader::new();

    // Check session every 30 seconds
    create_effect(move |_| {
        let interval = set_interval(
            move || {
                // Check if SSO session is still valid
                if !reader.has_valid_session() && auth.is_authenticated() {
                    // Session expired, logout
                    auth.logout();

                    // Show notification
                    show_notification("Sesi Anda telah berakhir. Silakan login kembali.");
                }
            },
            Duration::from_secs(30),
        );

        on_cleanup(move || {
            clear_interval(interval);
        });
    });

    view! {
        // This component doesn't render anything
        <></>
    }
}
```

## Cross-Tab Synchronization

### Listen to Storage Events

```rust
use shared_microfrontend::hooks::use_auth::use_auth;

#[component]
pub fn CrossTabSync() -> impl IntoView {
    let auth = use_auth();

    create_effect(move |_| {
        let closure = Closure::wrap(Box::new(move |event: web_sys::StorageEvent| {
            if let Some(key) = event.key() {
                match key.as_str() {
                    "logout_event" => {
                        // Logout triggered in another tab
                        auth.session.set(None);

                        // Redirect to login
                        if let Some(window) = window() {
                            let _ = window.location().set_href("/login");
                        }
                    }
                    "user_session" => {
                        // Session updated in another tab
                        // Re-initialize from SSO cookie
                        init_auth_from_sso_cookie();
                    }
                    _ => {}
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(window) = window() {
            let _ = window.add_event_listener_with_callback(
                "storage",
                closure.as_ref().unchecked_ref()
            );
        }

        closure.forget();
    });

    view! {
        <></>
    }
}
```

## Complete Example

### Main App Component

```rust
use leptos::prelude::*;
use leptos_router::*;
use shared_microfrontend::utils::sso_cookie::{
    init_auth_from_sso_cookie,
    setup_sso_session_monitor,
};

#[component]
pub fn App() -> impl IntoView {
    // Initialize SSO cookie integration
    create_effect(move |_| {
        // Restore session from SSO cookie
        init_auth_from_sso_cookie();

        // Setup session monitoring (check every 60 seconds)
        setup_sso_session_monitor(60000);
    });

    view! {
        <Router>
            // Session monitor component
            <SessionMonitor/>

            // Cross-tab synchronization
            <CrossTabSync/>

            <Routes>
                // Public routes
                <Route path="/login" view=LoginPage/>
                <Route path="/register" view=RegisterPage/>

                // Protected routes
                <Route path="/" view=move || view! {
                    <ProtectedRoute>
                        <HomePage/>
                    </ProtectedRoute>
                }/>

                <Route path="/dashboard" view=move || view! {
                    <ProtectedRoute>
                        <DashboardPage/>
                    </ProtectedRoute>
                }/>

                // ... other routes
            </Routes>
        </Router>
    }
}
```

## Configuration

### Environment Variables

```bash
# .env
AUTHENC_URL=https://authenc.simpel.kejaksaan.go.id
PORTAL_URL=https://simpel.kejaksaan.go.id
SSO_COOKIE_NAME=AUTHENC_SSO
```

### Build Configuration

```toml
# Trunk.toml
[build]
target = "index.html"

[[proxy]]
backend = "https://authenc.simpel.kejaksaan.go.id"
rewrite = "/api/auth"
```

## Testing

### Test SSO Cookie Reading

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_sso_cookie_reader() {
        let reader = SsoCookieReader::new();

        // This will return None in test environment
        // as there's no actual cookie set
        let session = reader.read_session();
        assert!(session.is_none());
    }
}
```

## Troubleshooting

### Cookie Not Found

**Problem**: `SsoCookieReader::read_session()` returns `None`

**Solutions**:
1. Check if user is logged in via backend
2. Verify cookie domain matches (simpel.kejaksaan.go.id)
3. Ensure HTTPS is enabled
4. Check browser cookie settings
5. Verify backend sets cookie correctly

### Session Expires Too Quickly

**Problem**: User logged out unexpectedly

**Solutions**:
1. Increase `max_age` in backend config
2. Implement token refresh mechanism
3. Adjust session monitoring interval
4. Check server time synchronization

### Cross-Origin Issues

**Problem**: Cookie not accessible from microfrontend

**Solutions**:
1. Verify domain is set to parent domain
2. Ensure all apps use HTTPS
3. Check SameSite attribute (should be Lax)
4. Verify CORS configuration

## Best Practices

1. **Always initialize on mount** - Call `init_auth_from_sso_cookie()` early
2. **Monitor session validity** - Use `setup_sso_session_monitor()`
3. **Handle expiration gracefully** - Show notification before logout
4. **Sync across tabs** - Listen to storage events
5. **Clear on logout** - Call backend logout endpoint
6. **Test thoroughly** - Test login, logout, and expiration flows
7. **Log errors** - Track SSO cookie issues for debugging

## Migration Checklist

- [ ] Import SSO cookie utilities
- [ ] Initialize
- [ ] Update login flow to use SSO cookie
- [ ] Update logout flow to clear SSO cookie
- [ ] Add session monitoring
- [ ] Add cross-tab synchronization
- [ ] Update protected routes
- [ ] Test login flow
- [ ] Test logout flow
- [ ] Test session expiration
- [ ] Test cross-tab sync
- [ ] Deploy to staging
- [ ] Test in production

## Support

For issues or questions:
- Check backend logs: `layanan/authenc/logs/`
- Review integration docs: `docs/SSO_COOKIE_INTEGRATION.md`
- Contact: DevOps team

## References

- Backend Implementation: `layanan/authenc/TASK_10.4_SSO_COOKIE_IMPLEMENTATION.md`
- Integration Guide: `docs/SSO_COOKIE_INTEGRATION.md`
- Shared Utils: `antarmuka/shared/src/utils/sso_cookie.rs`

