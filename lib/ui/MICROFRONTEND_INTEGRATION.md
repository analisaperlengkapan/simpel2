# 🚀 Microfrontend Integratio

Panduan lengkap untuk mengintegrasikan microfrontend baru ke dalam ekosistem SIMPelv2.

## 📖 Table of Contents

1. [Prerequisites](#prerequisites)
2. [Project Setup](#project-setup)
3. [Authentication Integration](#authentication-integration)
4. [Shared Components Integration](#shared-components-integration)
5. [Routing Setup](#routing-setup)
6. [API Integration](#api-integration)
7. [Styling](#styling)
8. [Testing](#testing)
9. [Deployment](#deployment)
10. [Troubleshooting](#troubleshooting)

---

## Prerequisites

Sebelum memulai, pastikan Anda memiliki:

- ✅ Rust 1.75+ installed
- ✅ Trunk 0.21+ installed (`cargo install trunk`)
- ✅ wasm32-unknown-unknown target (`rustup target add wasm32-unknown-unknown`)
- ✅ Access ke repository SIMPelv2
- ✅ Understanding of Leptos framework

---

## Project Setup

### 1. Create New Microfrontend

```bash
# Navigate to antarmuka directory
cd antarmuka

# Create new microfrontend directory
mkdir my_module
cd my_module

# Initialize Cargo project
cargo init --lib
```

### 2. Configure Cargo.toml

```toml
[package]
name = "my-module-microfrontend"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
# Leptos framework
leptos = { workspace = true }
leptos_router = { workspace = true }
leptos_meta = { workspace = true }

# Shared library
shared-microfrontend = { path = "../shared" }

# WASM bindings
wasm-bindgen = { workspace = true }
web-sys = { workspace = true }
js-sys = { workspace = true }

# Utilities
serde = { workspace = true }
serde_json = { workspace = true }
gloo = { workspace = true }
chrono = { workspace = true }

[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
```

### 3. Create Trunk.toml

```toml
[build]
target = "index.html"
release = true
dist = "dist"
public-url = "/my-module"

[watch]
ignore = ["dist"]

[serve]
address = "127.0.0.1"
port = 8081
open = false
```

### 4. Create index.html

```html
<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>My Module - SIMPelv2</title>

    <!-- Shared styles -->
    <link rel="stylesheet" href="/styles/main.css">

    <!-- Module-specific styles -->
    <link data-trunk rel="css" href="styles/main.css">
</head>
<body>
    <div id="app"></div>
</body>
</html>
```

### 5. Create Basic Structure

```
my_module/
├── src/
│   ├── lib.rs
│   ├── app.rs
│   ├── pages.rs
│   ├── components/
│   │   └── mod.rs
│   └── types.rs
├── styles/
│   └── main.css
├── index.html
├── Trunk.toml
└── Cargo.toml
```

---

## Authentication Integration

### 1. Setup lib.rs

```rust
use leptos::prelude::*;
use shared_microfrontend::prelude::*;

mod app;
mod pages;
mod components;
mod types;

use app::App;

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
```

### 2. Create App Component (app.rs)

```rust
use leptos::prelude::*;
use leptos_router::*;
use shared_microfrontend::prelude::*;
use shared_microfrontend::components::auth::LoginRedirectPage;

use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| "Page not found">
                // Default route - login redirect
                <Route path="/" view=|| view! {
                    <LoginRedirectPage
                        app_name="My Module"
                        app_description="Description of my module"
                    />
                } />

                // Protected routes
                <Route path="/dashboard" view=DashboardPage />
                <Route path="/data" view=DataPage />
                <Route path="/settings" view=SettingsPage />
            </Routes>
        </Router>
    }
}
```

### 3. Create Protected Pages (pages.rs)

```rust
use leptos::prelude::*;
use shared_microfrontend::prelude::*;
use shared_microfrontend::components::auth::{ProtectedRoute, LogoutButton, UserProfile};

#[component]
pub fn DashboardPage() -> impl IntoView {
    let auth = use_auth();

    view! {
        <ProtectedRoute>
            <div class="min-h-screen bg-gray-50">
                <AppHeader title="My Module" subtitle="Dashboard">
                    <UserProfile />
                    <LogoutButton />
                </AppHeader>

                <main class="container mx-auto px-4 py-8">
                    <h1 class="text-3xl font-bold mb-6">"Dashboard"</h1>

                    <Show when=move || auth.is_authenticated()>
                        {move || {
                            auth.get_session().map(|session| {
                                view! {
                                    <Card title="Welcome">
                                        <p>"Hello, " <strong>{session.name}</strong></p>
                                        <p class="text-sm text-gray-600">
                                            {session.role.display_name()} " - " {session.division}
                                        </p>
                                    </Card>
                                }
                            })
                        }}
                    </Show>

                    // Your dashboard content
                    <Grid cols=3 gap="1.5rem" class="mt-6">
                        <StatCard title="Total Items" value="1,234" />
                        <StatCard title="Active" value="856" />
                        <StatCard title="Pending" value="378" />
                    </Grid>
                </main>
            </div>
        </ProtectedRoute>
    }
}

#[component]
fn StatCard(title: String, value: String) -> impl IntoView {
    view! {
        <Card>
            <div class="text-center">
                <p class="text-sm text-gray-600 mb-2">{title}</p>
                <p class="text-3xl font-bold text-primary">{value}</p>
            </div>
        </Card>
    }
}

#[component]
pub fn DataPage() -> impl IntoView {
    view! {
        <ProtectedRoute required_permission="data:read">
            <div class="min-h-screen bg-gray-50">
                <AppHeader title="My Module" subtitle="Data">
                    <UserProfile />
                    <LogoutButton />
                </AppHeader>

                <main class="container mx-auto px-4 py-8">
                    <h1 class="text-3xl font-bold mb-6">"Data Management"</h1>

                    // Your data content
                </main>
            </div>
        </ProtectedRoute>
    }
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <ProtectedRoute required_permission="settings:write">
            <div class="min-h-screen bg-gray-50">
                <AppHeader title="My Module" subtitle="Settings">
                    <UserProfile />
                    <LogoutButton />
                </AppHeader>

                <main class="container mx-auto px-4 py-8">
                    <h1 class="text-3xl font-bold mb-6">"Settings"</h1>

                    // Your settings content
                </main>
            </div>
        </ProtectedRoute>
    }
}
```

---

## Shared Components Integration

### Using Layout Components

```rust
use shared_microfrontend::prelude::*;

#[component]
pub fn MyPage() -> impl IntoView {
    v
      <Container max_width="1280px">
            <Grid cols=2 gap="2rem">
                <Card title="Section 1">
                    <p>"Content 1"</p>
                </Card>
                <Card title="Section 2">
                    <p>"Content 2"</p>
                </Card>
            </Grid>
        </Container>
    }
}
```

### Using Form Components

```rust
#[component]
pub fn MyForm() -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (role, set_role) = signal(String::new());
    let (agreed, set_agreed) = signal(false);

    let handle_submit = move |_| {
        // Handle form submission
        logging::log!("Submitting: {} - {}", name.get(), email.get());
    };

    view! {
        <Card title="User Form">
            <form on:submit=handle_submit class="space-y-4">
                <Input
                    id="name"
                    name="name"
                    label="Full Name"
                    value=name
                    required=true
                    on_input=move |val| set_name.set(val)
                />

                <Input
                    id="email"
                    name="email"
                    label="Email"
                    input_type="email"
                    value=email
                    required=true
                    on_input=move |val| set_email.set(val)
                />

                <Select
                    id="role"
                    name="role"
                    label="Role"
                    value=role
                    options=vec![
                        SelectOption { value: "user".to_string(), label: "User".to_string() },
                        SelectOption { value: "admin".to_string(), label: "Admin".to_string() },
                    ]
                    on_change=move |val| set_role.set(val)
                />

                <Checkbox
                    id="terms"
                    name="terms"
                    label="I agree to the terms"
                    checked=agreed
                    on_change=move |val| set_agreed.set(val)
                />

                <Button
                    variant=ButtonVariant::Primary
                    full_width=true
                    disabled=move || !agreed.get()
                >
                    "Submit"
                </Button>
            </form>
        </Card>
    }
}
```

### Using Data Display Components

```rust
#[component]
pub fn DataList() -> impl IntoView {
    #[derive(Clone)]
    struct Item {
        id: u32,
        name: String,
        status: String,
    }

    let items = signal(vec![
        Item { id: 1, name: "Item 1".to_string(), status: "Active".to_string() },
        Item { id: 2, name: "Item 2".to_string(), status: "Pending".to_string() },
    ]).0;

    let columns = vec![
        TableColumn::new("id", "ID").sortable(),
        TableColumn::new("name", "Name").sortable(),
        TableColumn::new("status", "Status"),
    ];

    view! {
        <Card title="Data Table">
            <Table
                columns=columns
                data=items
                sortable=true
                paginated=true
                page_size=10
                striped=true
            />
        </Card>
    }
}
```

---

## Routing Setup

### Basic Routing

```rust
<Router>
    <Routes>
        <Route path="/" view=HomePage />
        <Route path="/dashboard" view=DashboardPage />
        <Route path="/users" view=UsersPage />
        <Route path="/users/:id" view=UserDetailPage />
    </Routes>
</Router>
```

### Nested Routing

```rust
<Router>
    <Routes>
        <Route path="/" view=HomePage />
        <Route path="/admin" view=AdminLayout>
            <Route path="" view=AdminDashboard />
            <Route path="users" view=AdminUsers />
            <Route path="settings" view=AdminSettings />
        </Route>
    </Routes>
</Router>
```

### Route Parameters

```rust
#[component]
pub fn UserDetailPage() -> impl IntoView {
    let params = use_params_map();
    let user_id = move || params.get().get("id").cloned().unwrap_or_default();

    view! {
        <div>
            <h1>"User Details: " {user_id}</h1>
        </div>
    }
}
```

---

## API Integration

### Create API Client

```rust
// src/api/client.rs
use gloo::net::http::Request;
use serde::{Deserialize, Serialize};
use shared_microfrontend::hooks::use_auth;

pub async fn api_get<T>(endpoint: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let auth = use_auth();
    let token = auth.get_session()
        .and_then(|s| s.access_token.clone())
        .ok_or("Not authenticated")?;

    let response = Request::get(&format!("/api{}", endpoint))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("API error: {}", response.status()))
    }
}

pub async fn api_post<T, R>(endpoint: &str, body: &T) -> Result<R, String>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let auth = use_auth();
    let token = auth.get_session()
        .and_then(|s| s.access_token.clone())
        .ok_or("Not authenticated")?;

    let response = Request::post(&format!("/api{}", endpoint))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("API error: {}", response.status()))
    }
}
```

### Use API in Components

```rust
use leptos::prelude::*;
use crate::api::client::*;

#[derive(Clone, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[component]
pub fn UsersPage() -> impl IntoView {
    let (users, set_users) = signal(Vec::<User>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    // Fetch users on mount
    create_effect(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match api_get::<Vec<User>>("/users").await {
                Ok(data) => {
                    set_users.set(data);
                    set_error.set(None);
                },
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    });

    view! {
        <div>
            <Show when=move || loading.get()>
                <Loading message="Loading users..." />
            </Show>

            <Show when=move || error.get().is_some()>
                <Alert
                    alert_type=AlertType::Error
                    message=move || error.get().unwrap_or_default()
                />
            </Show>

            <Show when=move || !loading.get() && error.get().is_none()>
                <Table data=users /* ... */ />
            </Show>
        </div>
    }
}
```


---

## Styling

### 1. Create Module Styles

```css
/* styles/main.css */

/* Module-specific variables */
:root {
    --module-primary: #3B82F6;
    --module-secondary: #10B981;
}

/* Module-specific styles */
.module-card {
    background: var(--color-white);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
}

.module-header {
    background: linear-gradient(135deg, var(--module-primary), var(--module-secondary));
    color: white;
    padding: var(--space-8);
}

/* Responsive overrides */
@media (max-width: 768px) {
    .module-card {
        padding: var(--space-4);
    }
}
```

### 2. Use Tailwind Classes

```rust
view! {
    <div class="bg-white rounded-lg shadow-sm p-6">
        <h2 class="text-2xl font-bold text-gray-900 mb-4">
            "Title"
        </h2>
        <p class="text-gray-600">
            "Content"
        </p>
    </div>
}
```

### 3. Conditional Styling

```rust
let (is_active, set_active) = signal(false);

view! {
    <div class=move || {
        if is_active.get() {
            "bg-blue-100 border-blue-500"
        } else {
            "bg-gray-100 border-gray-300"
        }
    }>
        "Content"
    </div>
}
```

---

## Testing

### 1. Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_validation() {
        let data = MyData {
            name: "Test".to_string(),
            value: 42,
        };

        assert!(data.is_valid());
    }

    #[test]
    fn test_formatting() {
        let result = format_data(1000);
        assert_eq!(result, "1.000");
    }
}
```

### 2. Component Tests

```rust
#[cfg(test)]
mod component_tests {
    use leptos::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_button_renders() {
        mount_to_body(|| view! {
            <Button variant=ButtonVariant::Primary>
                "Click me"
            </Button>
        });

        let button = document()
            .query_selector("button")
            .unwrap()
            .unwrap();

        assert_eq!(button.text_content().unwrap(), "Click me");
    }
}
```

### 3. IntegrationTests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_api_integration() {
        let result = api_get::<Vec<User>>("/users").await;
        assert!(result.is_ok());
    }
}
```

---

## Deployment

### 1. Build for Production

```bash
# Build with Trunk
trunk build --release

# Output will be in dist/
ls dist/
# index.html
# my-module-microfrontend-*.wasm
# my-module-microfrontend-*.js
```

### 2. Configure Nginx

```nginx
# /etc/nginx/conf.d/my-module.conf

server {
    listen 80;
    server_name my-module.simpelv2.kejaksaan.go.id;

    root /usr/share/nginx/html/my-module;
    index index.html;

    # WASM mime type
    types {
        application/wasm wasm;
    }

    # Gzip compression
    gzip on;
    gzip_types text/css application/javascript application/wasm;

    # SPA routing
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API proxy
    location /api/ {
        proxy_pass https://api.simpelv2.kejaksaan.go.id/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # Cache static assets
    location ~* \.(wasm|js|css)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

### 3. Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Install trunk and wasm target
RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown

# Build
RUN cd antarmuka/my_module && trunk build --release

# Production image
FROM nginx:alpine

# Copy built files
COPY --from=builder /app/antarmuka/my_module/dist /usr/share/nginx/html/my-module

# Copy nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### 4. Kubernetes Deployment

```yaml
# k8s/my-module-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-module
  namespace: simpelv2
spec:
  replicas: 3
  selector:
    matchLabels:
      app: my-module
  template:
    metadata:
      labels:
        app: my-module
    spec:
      containers:
      - name: my-module
        image: registry.kejaksaan.go.id/simpelv2/my-module:latest
        ports:
        - containerPort: 80
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "200m"
---
apiVersion: v1
kind: Service
metadata:
  name: my-module
  namespace: simpelv2
spec:
  selector:
    app: my-module
  ports:
  - port: 80
    targetPort: 80
  type: ClusterIP
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: my-module
  namespace: simpelv2
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
  - hosts:
    - my-module.simpelv2.kejaksaan.go.id
    secretName: my-module-tls
  rules:
  - host: my-module.simpelv2.kejaksaan.go.id
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: my-module
            port:
              number: 80
```

---

## Troubleshooting

### Issue: WASM Module Not Loading

**Symptoms:**
- Blank page
- Console error: "Failed to fetch WASM module"

**Solutions:**
1. Check WASM mime type in nginx:
   ```nginx
   types {
       application/wasm wasm;
   }
   ```

2. Verify CORS headers:
   ```nginx
   add_header Access-Control-Allow-Origin "*";
   ```

3. Check file permissions:
   ```bash
   chmod 644 dist/*.wasm
   ```

---

### Issue: Authentication Not Working

**Symptoms:**
- Infinite redirect loop
- Session not persisting

**Solutions:**
1. Verify portal URL is correct:
   ```rust
   // Check environment variable
   let portal_url = env!("PORTAL_URL", "http://localhost:8080");
   ```

2. Check localStorage:
   ```javascript
   // In browser console
   localStorage.getItem('user_session')
   ```

3. Verify CORS settings allow credentials:
   ```nginx
   add_header Access-Control-Allow-Credentials "true";
   ```

---

### Issue: Styles Not Applied

**Symptoms:**
- Components look unstyled
- Missing colors/spacing

**Solutions:**
1. Ensure main.css is imported in index.html:
   ```html
   <link rel="stylesheet" href="/styles/main.css">
   ```

2. Check CSS file path in Trunk.toml:
   ```toml
   [[hooks]]
   stage = "pre_build"
   command = "cp"
   command_arguments = ["../shared/styles/main.css", "styles/"]
   ```

3. Verify Tailwind classes are correct

---

### Issue: API Calls Failing

**Symptoms:**
- 401 Unauthorized errors
- CORS errors

**Solutions:**
1. Check JWT token is included:
   ```rust
   .header("Authorization", &format!("Bearer {}", token))
   ```

2. Verify API endpoint URL:
   ```rust
   let api_url = "/api/users"; // Should start with /api
   ```

3. Check Envoy/Nginx routing:
   ```nginx
   location /api/ {
       proxy_pass https://api.simpelv2.kejaksaan.go.id/;
   }
   ```

---

### Issue: Build Errors

**Symptoms:**
- Compilation errors
- Missing dependencies

**Solutions:**
1. Update dependencies:
   ```bash
   cargo update
   ```

2. Clean build cache:
   ```bash
   cargo clean
   trunk clean
   ```

3. Verify workspace dependencies:
   ```toml
   [dependencies]
   leptos = { workspace = true }
   ```

---

## Best Practices

### 1. Code Organization

```
my_module/
├── src/
│   ├── lib.rs           # Entry point
│   ├── app.rs           # Main app component
│   ├── pages/           # Page components
│   │   ├── mod.rs
│   │   ├── dashboard.rs
│   │   └── users.rs
│   ├── components/      # Reusable components
│   │   ├── mod.rs
│   │   ├── header.rs
│   │   └── sidebar.rs
│   ├── api/             # API client
│   │   ├── mod.rs
│   │   └── client.rs
│   ├── types.rs         # Type definitions
│   └── utils.rs         # Utility functions
```

### 2. Error Handling

```rust
// Use Result for error handling
pub async fn fetch_data() -> Result<Vec<Data>, String> {
    api_get("/data")
        .await
        .map_err(|e| format!("Failed to fetch data: {}", e))
}

// Display errors to users
<Show when=move || error.get().is_some()>
    <Alert
        alert_type=AlertType::Error
        message=move || error.get().unwrap_or_default()
    />
</Show>
```

### 3. Loading States

```rust
// Always show loading indicators
<Show
    when=move || !loading.get()
    fallback=|| view! { <Loading message="Loading..." /> }
>
    <DataTable data=data />
</Show>
```

### 4. Accessibility

```rust
// Use semantic HTML
<nav aria-label="Main navigation">
    <NavMenu items=items />
</nav>

// Provide alt text
<OptimizedImage src="/logo.png" alt="Company Logo" />

// Use proper form labels
<Input id="email" label="Email Address" /* ... */ />
```

### 5. Performance

```rust
// Use memos for expensive computations
let filtered_data = create_memo(move |_| {
    data.get()
        .iter()
        .filter(|item| item.status == "active")
        .cloned()
        .collect::<Vec<_>>()
});

// Debounce search inputs
let debounced_search = use_debounce(search, 300);
```

---

## Checklist

Before deploying your microfrontend, ensure:

- [ ] Authentication integration complete
- [ ] All routes protected with `ProtectedRoute`
- [ ] Logout button added to header
- [ ] API client configured with JWT tokens
- [ ] Error handling implemented
- [ ] Loading states for async operations
- [ ] Responsive design tested (mobile, tablet, desktop)
- [ ] Accessibility tested (keyboard navigation, screen readers)
- [ ] Unit tests written for business logic
- [ ] Integration tests for API calls
- [ ] Build succeeds with `trunk build --release`
- [ ] Nginx configuration created
- [ ] Docker image builds successfully
- [ ] Kubernetes manifests created
- [ ] Documentation updated

---

## Additional Resources

- **Leptos Documentation**: https://leptos.dev
- **Shared Library Reference**: `/antarmuka/shared/COMPONENT_REFERENCE.md`
- **Quick Start Guide**: `/antarmuka/shared/QUICK_START.md`
- **Authentication Guide**: `/antarmuka/shared/README.md#authentication-integration`
- **Deployment Guide**: `/docs/DEPLOYMENT_INTEGRATION_GUIDE.md`

---

## Support

Untuk bantuan lebih lanjut:

- **Email**: dev@kejaksaan.go.id
- **Internal Wiki**: https://wiki.kejaksaan.go.id/simpelv2
- **Issue Tracker**: https://gitlab.kejaksaan.go.id/simpelv2/issues

---

**Built with ❤️ by Tim Pengembang SIMPelv2**
**Kejaksaan Agung Republik Indonesia**

