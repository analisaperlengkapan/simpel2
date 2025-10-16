# 🔧 Troubleshooting Guide - SIMPelv2 Frontend

Panduan lengkap untuk mengatasi masalah umum pada portal dan microfrontends SIMPelv2.

## 📖 Table of Contents

1. [Build Issues](#build-issues)
2. [Runtime Issues](#runtime-issues)
3. [Authentication Issues](#authentication-issues)
4. [API Issues](#api-issues)
5. [Styling Issues](#styling-issues)
6. [Performance Issues](#performance-issues)
7. [Deployment Issues](#deployment-issues)
8. [Browser Compatibility](#browser-compatibility)

---

## Build Issues

### Issue: Trunk Build Fails

**Error:**
```
error: failed to compile `trunk` v0.21.14
```

**Solutions:**

1. Update Rust toolchain:
```bash
rustup update stable
rustup default stable
```

2. Clean and rebuild:
```bash
cargo clean
trunk clean
trunk build
```

3. Check Trunk version:
```bash
trunk --version
cargo install trunk --force
```

---

### Issue: WASM Target Not Found

**Error:**
```
error: can't find crate for `std`
target 'wasm32-unknown-unknown' may not be installed
```

**Solution:**
```bash
rustup target add wasm32-unknown-unknown
```

---

### Issue: Dependency Resolution Errors

**Error:**
```
error: failed to select a version for `leptos`
```

**Solutions:**

1. Update Cargo.lock:
```bash
cargo update
```

2. Check workspace dependencies:
```toml
[dependencies]
leptos = { workspace = true }  # Ensure workspace = true
```

3. Clean cargo cache:
```bash
rm -rf ~/.cargo/registry
cargo build
```

---

### Issue: Out of Memory During Build

**Error:**
```
error: could not compile `my-module` due to previous error
SIGKILL: killed
```

**Solutions:**

1. Increase swap space:
```bash
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

2. Build with fewer parallel jobs:
```bash
cargo build -j 2
```

3. Use release profile for smaller builds:
```bash
trunk build --release
```

---

## Runtime Issues

### Issue: Blank Page / White Screen

**Symptoms:**
- Page loads but shows nothing
- No errors in console

**Solutions:**

1. Check browser console for errors:
```javascript
// Open DevTools (F12)
// Look for JavaScript errors
```

2. Verify WASM loaded:
```javascript
// In console
console.log(WebAssembly);
```

3. Check network tab:
- Verify .wasm file downloaded (200 status)
- Check file size (should be > 0 bytes)

4. Clear browser cache:
```
Ctrl+Shift+Delete (Chrome/Firefox)
```

5. Check index.html:
```html
<!-- Ensure div#app exists -->
<div id="app"></div>
```

---

### Issue: Component Not Rendering

**Symptoms:**
- Component doesn't appear
- No errors in console

**Solutions:**

1. Check component import:
```rust
use shared_microfrontend::components::*;
```

2. Verify props are correct:
```rust
// Check prop types match
<Button variant=ButtonVariant::Primary>  // ✅
<Button variant="primary">  // ❌ Wrong type
```

3. Check conditional rendering:
```rust
<Show when=move || condition.get()>
    <MyComponent />
</Show>
```

4. Verify signal updates:
```rust
// Use move || to access signal
{move || my_signal.get()}  // ✅
{my_signal.get()}  // ❌ May not update
```

---

### Issue: Signal Not Updating

**Symptoms:**
- UI doesn't re-render when data changes
- Signal value changes but component doesn't update

**Solutions:**

1. Use `move ||` closure:
```rust
// ✅ Correct
<div>{move || count.get()}</div>

// ❌ Wrong
<div>{count.get()}</div>
```

2. Check signal creation:
```rust
// ✅ Correct
let (count, set_count) = signal(0);

// ❌ Wrong (old Leptos API)
let (count, set_count) = create_signal(0);
```

3. Verify signal updates:
```rust
// ✅ Correct
set_count.set(new_value);

// ❌ Wrong
set_count.update(|c| *c = new_value);  // Use set() instead
```

---

### Issue: Memory Leak

**Symptoms:**
- Browser tab uses increasing memory
- Page becomes slow over time

**Solutions:**

1. Clean up effects:
```rust
create_effect(move |_| {
    // Your effect code

    // Return cleanup function
    move || {
        // Cleanup code
    }
});
```

2. Remove event listeners:
```rust
let closure = Closure::wrap(Box::new(move |_| {
    // Handler
}) as Box<dyn FnMut(_)>);

window().add_event_listener_with_callback("event", closure.as_ref().unchecked_ref());

// Don't forget to drop closure when done
drop(closure);
```

3. Use weak references for circular dependencies

---

## Authentication Issues

### Issue: Infinite Redirect Loop

**Symptoms:**
- Page keeps redirecting between login and app
- Browser shows "Too many redirects"

**Solutions:**

1. Check route configuration:
```rust
// ✅ Correct - LoginRedirectPage on different route
<Route path="/" view=LoginRedirectPage />
<Route path="/dashboard" view=DashboardPage />

// ❌ Wrong - Both on same route
<Route path="/" view=LoginRedirectPage />
<Route path="/" view=DashboardPage />
```

2. Verify ProtectedRoute logic:
```rust
// Ensure ProtectedRoute doesn't redirect to itself
<ProtectedRoute>
    <DashboardPage />  // Should not contain LoginRedirectPage
</ProtectedRoute>
```

3. Check localStorage:
```javascript
// In browser console
localStorage.getItem('user_session')
// Should return session JSON or null
```

---

### Issue: Session Not Persisting

**Symptoms:**
- User logged out after page refresh
- Session lost when opening new tab

**Solutions:**

1. Verify localStorage is enabled:
```javascript
// In browser console
try {
    localStorage.setItem('test', 'test');
    localStorage.removeItem('test');
    console.log('localStorage works');
} catch (e) {
    console.error('localStorage disabled', e);
}
```

2. Check session storage:
```rust
// Ensure using localStorage, not sessionStorage
save_to_storage("user_session", &session);  // ✅
```

3. Verify session structure:
```javascript
// In browser console
let session = JSON.parse(localStorage.getItem('user_session'));
console.log(session);
// Should have: id, username, access_token, etc.
```

---

### Issue: Cross-Tab Sync Not Working

**Symptoms:**
- Login in one tab doesn't update other tabs
- Logout in one tab doesn't affect other tabs

**Solutions:**

1. Verify storage event listener:
```rust
// Ensure use_auth hook is called in root component
let auth = use_auth();  // Sets up storage listener
```

2. Check browser support:
```javascript
// Storage events only fire in OTHER tabs, not same tab
// Test by opening two tabs
```

3. Manual sync:
```rust
// Force sync across tabs
window().dispatch_event(&StorageEvent::new("storage").unwrap());
```

---

## API Issues

### Issue: 401 Unauthorized

**Symptoms:**
- API calls return 401 error
- "Unauthorized" message

**Solutions:**

1. Check JWT token:
```rust
let auth = use_auth();
if let Some(session) = auth.get_session() {
    logging::log!("Token: {}", session.access_token.unwrap_or_default());
} else {
    logging::log!("No session");
}
```

2. Verify Authorization header:
```rust
.header("Authorization", &format!("Bearer {}", token))
```

3. Check token expiration:
```javascript
// In browser console
let session = JSON.parse(localStorage.getItem('user_session'));
let expires = new Date(session.expires_at * 1000);
console.log('Expires:', expires);
console.log('Now:', new Date());
```

4. Refresh token:
```rust
// Implemerefresh logic
if token_expired() {
    refresh_token().await?;
}
```

---

### Issue: CORS Errors

**Symptoms:**
- Console error: "CORS policy blocked"
- API calls fail with network error

**Solutions:**

1. Check API URL:
```rust
// Use relative URL (goes through same origin)
let url = "/api/users";  // ✅

// Avoid absolute URL (triggers CORS)
let url = "https://api.example.com/users";  // ❌
```

2. Verify Nginx proxy:
```nginx
location /api/ {
    proxy_pass https://api.simpelv2.kejaksaan.go.id/;
    proxy_set_header Host $host;
}
```

3. Check CORS headers (if using absolute URLs):
```nginx
add_header Access-Control-Allow-Origin "*";
add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS";
add_header Access-Control-Allow-Headers "Authorization, Content-Type";
```

---

### Issue: API Timeout

**Symptoms:**
- Requests take too long
- Timeout errors

**Solutions:**

1. Increase timeout:
```rust
let response = Request::get(url)
    .timeout(Duration::from_secs(30))  // 30 second timeout
    .send()
    .await?;
```

2. Add loading indicator:
```rust
let (loading, set_loading) = signal(false);

<Show when=move || loading.get()>
    <Loading message="Loading..." />
</Show>
```

3. Implement retry logic:
```rust
async fn api_get_with_retry<T>(url: &str, max_retries: u32) -> Result<T, String> {
    for attempt in 0..max_retries {
        match api_get(url).await {
            Ok(data) => return Ok(data),
            Err(e) if attempt < max_retries - 1 => {
                gloo_timers::future::TimeoutFuture::new(1000 * (attempt + 1)).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

---

## Styling Issues

### Issue: Styles Not Applied

**Symptoms:**
- Components look unstyled
- Missing colors/spacing

**Solutions:**

1. Check CSS import in index.html:
```html
<link rel="stylesheet" href="/styles/main.css">
```

2. Verify CSS file exists:
```bash
ls -la antarmuka/shared/styles/main.css
```

3. Check browser DevTools:
- Open DevTools (F12)
- Go to Network tab
- Check if main.css loaded (200 status)

4. Clear browser cache:
```
Ctrl+Shift+R (hard refresh)
```

---

### Issue: Tailwind Classes Not Working

**Symptoms:**
- Tailwind utility classes have no effect
- Custom classes work but Tailwind doesn't

**Solutions:**

1. Verify Tailwind is included:
```css
/* In main.css */
@tailwind base;
@tailwind components;
@tailwind utilities;
```

2. Check class names:
```rust
// ✅ Correct
<div class="bg-blue-500 text-white p-4">

// ❌ Wrong (typo)
<div class="bg-blue-500 text-white padding-4">
```

3. Purge configuration:
```javascript
// tailwind.config.js
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
  ],
  // ...
}
```

---

### Issue: Dark Mode Not Working

**Symptoms:**
- Dark mode toggle doesn't change theme
- Colors don't update

**Solutions:**

1. Check theme hook:
```rust
let theme = use_theme();
theme.toggle();  // Should toggle theme
```

2. Verify CSS variables:
```css
:root {
    --color-bg: #ffffff;
}

[data-theme="dark"] {
    --color-bg: #1a1a1a;
}
```

3. Check HTML attribute:
```javascript
// In browser console
document.documentElement.getAttribute('data-theme')
// Should be 'light' or 'dark'
```

---

## Performance Issues

### Issue: Slow Page Load

**Symptoms:**
- Long Time to Interactive (TTI)
- Large bundle size

**Solutions:**

1. Check bundle size:
```bash
ls -lh dist/*.wasm
# Should be < 500KB
```

2. Optimize WASM:
```bash
wasm-opt -Oz input.wasm -o output.wasm
```

3. Enable code splitting:
```rust
// Lazy load routes
<Route path="/admin" view=|| {
    lazy(|| import("./pages/admin.rs"))
} />
```

4. Optimize images:
```bash
# Convert to WebP
cwebp -q 80 image.png -o image.webp
```

---

### Issue: High Memory Usage

**Symptoms:**
- Browser tab uses lots of RAM
- Page becomes unresponsive

**Solutions:**

1. Use virtual scrolling for large lists:
```rust
// Instead of rendering all items
{items.iter().map(|item| view! { <Item data=item /> }).collect_view()}

// Use pagination or virtual scroll
<Pagination /* ... */ />
```

2. Implement lazy loading:
```rust
<OptimizedImage src="/large-image.jpg" lazy=true />
```

3. Clean up resources:
```rust
on_cleanup(move || {
    // Clean up event listeners, timers, etc.
});
```

---

## Deployment Issues

### Issue: 404 Not Found After Deployment

**Symptoms:**
- Direct URL access returns 404
- Refresh on route returns 404

**Solutions:**

1. Configure SPA routing in Nginx:
```nginx
location / {
    try_files $uri $uri/ /index.html;
}
```

2. Check base path:
```toml
# Trunk.toml
[build]
public-url = "/my-module"
```

3. Verify file permissions:
```bash
chmod 644 /var/www/portal/*.html
chmod 644 /var/www/portal/*.wasm
```

---

### Issue: WASM Not Loading in Production

**Symptoms:**
- Works locally but not in production
- Console error: "Failed to fetch WASM"

**Solutions:**

1. Check WASM mime type:
```nginx
types {
    application/wasm wasm;
}
```

2. Verify CORS headers:
```nginx
add_header Access-Control-Allow-Origin "*";
```

3. Check file path:
```bash
# Ensure WASM file exists
ls -la /var/www/portal/*.wasm
```

4. Test WASM download:
```bash
curl -I https://portal.simpelv2.kejaksaan.go.id/portal-*.wasm
```

---

## Browser Compatibility

### Issue: Not Working in Internet Explorer

**Solution:**
Internet Explorer is not supported. Use modern browsers:
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

---

### Issue: Safari-Specific Issues

**Symptoms:**
- Works in Chrome but not Safari
- WebAssembly errors

**Solutions:**

1. Check Safari version (14+)

2. Enable WebAssembly:
- Safari > Preferences > Advanced
- Check "Show Develop menu"
- Develop > Experimental Features > WebAssembly

3. Clear Safari cache:
- Safari > Preferences > Privacy
- Manage Website Data > Remove All

---

## Getting Help

If you can't resolve the issue:

1. **Check logs:**
```bash
# Browser console (F12)
# Server logs
tail -f /var/log/nginx/error.log
kubectl logs -f deployment/portal -n simpelv2
```

2. **Gather information:**
- Browser and version
- Error messages (full text)
- Steps to reproduce
- Screenshots

3. **Contact support:**
- **Email**: dev@kejaksaan.go.id
- **Slack**: #simpelv2-support
- **Issue Tracker**: https://gitlab.kejaksaan.go.id/simpelv2/issues

---

**Built with ❤️ by Tim Pengembang SIMPelv2**
**Kejaksaan Agung Republik Indonesia**

