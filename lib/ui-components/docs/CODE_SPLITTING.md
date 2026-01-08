# Code Splitting Guide for SIMPelv2

## Overview

This guide explains the code splitting strategy for SIMPelv2's unified frontend system. Due to WASM limitations in Leptos 0.8, we use **build-time code splitting** through microfrontend architecture rather than runtime lazy loading.

## Architecture-Based Code Splitting

### Microfrontend Separation

Each microfrontend is a separate WASM bundle, providing natural code splitting:

```
Portal Bundle:
├── portal.wasm           (~300KB gzipped)
├── portal.js             (~50KB gzipped)
└── Total: ~350KB

Microfrontend Bundles (loaded on demand):
├── badiklat.wasm         (~250KB gzipped)
├── datun.wasm            (~200KB gzipped)
├── intel.wasm            (~220KB gzipped)
├── pidum.wasm            (~210KB gzipped)
├── pidsus.wasm           (~215KB gzipped)
├── pidmil.wasm           (~205KB gzipped)
├── pengawasan.wasm       (~195KB gzipped)
├── pemulihan_aset.wasm   (~200KB gzipped)
└── pembinaan/*.wasm      (~180KB each)
```

**Benefits:**
- Users only download what they need
- Independent deployment per microfrontend
- Parallel development without conflicts
- Natural cache boundaries

## Build Optimization

### Trunk Configuration

Optimize `Trunk.toml` for maximum size reduction:

```toml
[build]
target = "index.html"
dist = "dist"
release = true

# WASM optimization - aggressive size reduction
[[build.wasm_opt]]
args = [
  "-Oz",                # Maximum size optimization
  "--enable-all",       # Enable all features
  "--strip-debug",      # Remove debug information
  "--strip-producers",  # Remove producer metadata
  "--strip-dwarf",      # Remove DWARF debug info
  "--vacuum",           # Remove unused code
]

# Advanced wasm-bindgen configuration
[build.wasm-bindgen]
target = "web"
typescript = false      # Disable TS generation
debug = false
demangle = false        # No name demangling
keep-debug = false
reference-types = true  # Enable reference types
weak-refs = true        # Enable weak references

# Rust optimization
[build.rust]
optimization-level = "z"  # Maximum size optimization
debug = false
lto = true                # Link-time optimization
codegen-units = 1         # Single codegen unit
```

### Cargo Profile

Add to root `Cargo.toml`:

```toml
[profile.release-wasm]
inherits = "release"
opt-level = "z"           # Optimize for size
lto = "fat"               # Full LTO
codegen-units = 1         # Better optimization
strip = true              # Strip symbols
panic = "abort"           # Smaller panic handler
```

## Performance Utilities

### Bundle Size Analysis

Analyze bundle size on application load:

```rust
use shared_microfrontend::utils::code_splitting::analyze_bundle_size;

#[component]
pub fn App() -> impl IntoView {
    // Analyze bundle size on mount (development only)
    #[cfg(debug_assertions)]
    create_effect(move |_| {
        analyze_bundle_size();
    });

    view! {
        <Router>
            <Routes>
                // Your routes
            </Routes>
        </Router>
    }
}
```

Output in browser console:
```
📦 Bundle Size Analysis
  WASM: 287.45 KB
  JavaScript: 48.23 KB
  Total: 335.68 KB
  ✅ Bundle size is optimal!
```

### Component Render Time Measurement

Track performance of expensive components:

```rust
use shared_microfrontend::utils::code_splitting::measure_render_time;

#[component]
pub fn ExpensiveDataTable() -> impl IntoView {
    measure_render_time("ExpensiveDataTable", || {
        view! {
            <Table data=large_dataset />
        }
    })
}
```

Output in console:
```
⏱️ ExpensiveDataTable rendered in 45.23ms
```

### Route Preloading

Improve perceived performance by preloading routes on hover:

```rust
use shared_microfrontend::utils::code_splitting::preload_route;

#[component]
pub fn NavigationMenu() -> impl IntoView {
    view! {
        <nav>
            <a
                href="/dashboard"
                on:mouseenter=move |_| {
                    preload_route("dashboard");
                }
            >
                "Dashboard"
            </a>
            <a
                href="/apps"
                on:mouseenter=move |_| {
                    preload_route("apps");
                }
            >
                "Applications"
            </a>
        </nav>
    }
}
```

## Route Organization Strategy

### Critical vs Feature Routes

Organize routes by loading priority:

```rust
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                // CRITICAL ROUTES - Always loaded (authentication flow)
                <Route path="/" view=HomePage />
                <Route path="/login" view=LoginPage />
                <Route path="/callback" view=CallbackPage />

                // FEATURE ROUTES - Loaded on navigation
                <Route path="/dashboard" view=DashboardPage />
                <Route path="/apps" view=AppsPage />
                <Route path="/notifications" view=NotificationsPage />

                // ADMIN ROUTES - Rarely accessed
                <Route path="/admin/*" view=AdminRoutes />
            </Routes>
        </Router>
    }
}
```

### Component Splitting

Split large components into smaller modules:

```rust
// ❌ BAD: Everything in one file
// dashboard.rs (5000 lines)

// ✅ GOOD: Split by feature
// dashboard/
// ├── mod.rs           (main component)
// ├── widgets.rs       (dashboard widgets)
// ├── charts.rs        (chart components)
// └── statistics.rs    (stats components)
```

## Nginx Configuration

Configure Nginx for optimal delivery:

```nginx
server {
    listen 443 ssl http2;
    server_name simpelv2.kejaksaan.go.id;

    # Enable gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types
        text/css
        text/javascript
        application/javascript
        application/wasm;

    # Enable brotli (if available)
    brotli on;
    brotli_types
        text/css
        text/javascript
        application/javascript
        application/wasm;

    # WASM mime type
    types {
        application/wasm wasm;
    }

    # Cache static assets
    location ~* \.(wasm|js|css)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Portal
    location / {
        root /usr/share/nginx/html/portal;
        try_files $uri $uri/ /index.html;
    }

    # Microfrontends
    location /badiklat {
        alias /usr/share/nginx/html/badiklat;
        try_files $uri $uri/ /badiklat/index.html;
    }

    # ... other microfrontends
}
```

## Performance Targets

### Bundle Size Targets

| Bundle Type | Target Size (gzipped) | Status |
|-------------|----------------------|--------|
| Portal | < 400KB | ✅ ~350KB |
| Microfrontend | < 300KB | ✅ ~250KB avg |
| Shared Library | Included in bundles | ✅ |

### Loading Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| First Contentful Paint (FCP) | < 1.5s | Lighthouse |
| Time to Interactive (TTI) | < 3.0s | Lighthouse |
| Largest Contentful Paint (LCP) | < 2.5s | Core Web Vitals |
| First Input Delay (FID) | < 100ms | Core Web Vitals |
| Cumulative Layout Shift (CLS) | < 0.1 | Core Web Vitals |

## Optimization Checklist

### Build Time
- [ ] Use `--release` flag for production builds
- [ ] Enable `-Oz` optimization in wasm-opt
- [ ] Configure LTO in Cargo.toml
- [ ] Strip debug symbols
- [ ] Use single codegen unit

### Runtime
- [ ] Implement route preloading on hover
- [ ] Use pagination instead of virtual scrolling
- [ ] Lazy load images with `loading="lazy"`
- [ ] Minimize initial render complexity
- [ ] Use CSS instead of JS animations

### Deployment
- [ ] Enable gzip/brotli compression
- [ ] Configure proper cache headers
- [ ] Use CDN for static assets (optional)
- [ ] Implement HTTP/2 server push (optional)
- [ ] Monitor bundle sizes in CI/CD

## Measuring Performance

### Browser DevTools

1. **Network Tab**
   - Check WASM bundle sizes
   - Verify compression is working
   - Monitor load times

2. **Performance Tab**
   - Record page load
   - Analyze render timeline
   - Identify bottlenecks

3. **Lighthouse Audit**
   ```bash
   # Run Lighthouse
   lighthouse https://simpelv2.kejaksaan.go.id --view
   ```

### Bundle Analysis

```bash
# Build and analyze
cd antarmuka/portal
trunk build --release

# Check bundle sizes
ls -lh dist/*.wasm
ls -lh dist/*.js

# Analyze with wasm-opt
wasm-opt dist/*.wasm --print-features
```

### Performance Monitoring

Add to your app:

```rust
#[component]
pub fn App() -> impl IntoView {
    // Track Core Web Vitals
    #[cfg(target_arch = "wasm32")]
    create_effect(move |_| {
        use shared_microfrontend::utils::monitoring::track_web_vitals;
        track_web_vitals();
    });

    view! { /* ... */ }
}
```

## Common Issues & Solutions

### Issue: Bundle Too Large

**Symptoms**: WASM bundle > 400KB gzipped

**Solutions**:
1. Check dependencies - remove unused crates
2. Enable LTO and size optimization
3. Use `cargo tree` to find large dependencies
4. Consider splitting into multiple microfrontends

### Issue: Slow Initial Load

**Symptoms**: FCP > 2s

**Solutions**:
1. Enable compression (gzip/brotli)
2. Reduce critical route complexity
3. Preload fonts and critical CSS
4. Use HTTP/2 for parallel loading

### Issue: Poor Navigation Performance

**Symptoms**: Slow route transitions

**Solutions**:
1. Implement route preloading
2. Reduce component complexity
3. Use pagination for large lists
4. Optimize re-renders with memos

## Best Practices

1. **Keep Bundles Small**: Target < 400KB per bundle
2. **Measure Regularly**: Run Lighthouse audits weekly
3. **Monitor in Production**: Track real user metrics
4. **Optimize Images**: Use WebP and lazy loading
5. **Cache Aggressively**: Set long cache times for immutable assets
6. **Test on Slow Networks**: Use Chrome DevTools throttling
7. **Profile Before Optimizing**: Measure first, optimize second

## Resources

- [Leptos Performance Guide](https://leptos.dev/performance.html)
- [WASM Optimization](https://rustwasm.github.io/book/reference/code-size.html)
- [Web Vitals](https://web.dev/vitals/)
- [Lighthouse](https://developers.google.com/web/tools/lighthouse)

## Conclusion

Code splitting in SIMPelv2 is achieved through:
1. **Microfrontend architecture** - Natural bundle separation
2. **Build optimization** - Aggressive size reduction
3. **Smart caching** - Long-lived immutable assets
4. **Performance monitoring** - Continuous measurement

This approach provides excellent performance while maintaining developer productivity and code maintainability.
