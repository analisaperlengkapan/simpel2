# SIMPEL Portal - Gateway to Justice Technology

## 📋 Overview

**Portal SIMPEL** adalah gateway utama untuk seluruh ekosistem aplikasi manajemen aset Kejaksaan RI. Portal ini menyediakan Single Sign-On (SSO), dashboard terpadu, dan routing ke semua microfrontend dalam sistem SIMPEL.

## 🎯 Features

### Core Functionality

- ✅ **Single Sign-On (SSO)** - Autentikasi terpusat untuk semua aplikasi
- ✅ **Unified Dashboard** - Dashboard overview semua modul
- ✅ **Microfrontend Router** - Gateway ke semua aplikasi SIMPEL
- ✅ **User Profile Management** - Manajemen profil dan preferensi
- ✅ **Notification Center** - Notifikasi real-time lintas modul
- ✅ **Quick Access Menu** - Akses cepat ke fungsi favorit
- ✅ **Role-Based Navigation** - Menu dinamis sesuai peran pengguna
- ✅ **Search Global** - Pencarian lintas modul

### Technical Features

- 🚀 **Leptos 0.8.x** - Modern reactive web framework
- 📦 **WebAssembly** - High-performance browser execution
- 🎨 **Shared Components** - Government-compliant UI library
- 🔒 **Security** - JWT authentication, OAuth 2.0, RBAC
- ♿ **Accessibility** - WCAG 2.1 AA compliant
- 📱 **Responsive** - Mobile-first design
- 🌐 **i18n Ready** - Multi-language support (ID, EN)

## 🏗️ Architecture

```
antarmuka/portal/
├── Cargo.toml              # Dependencies & build config
├── Trunk.toml              # WASM build settings
├── Dockerfile              # Container image
├── index.html              # HTML template
├── src/
│   ├── lib.rs             # Library entry point
│   ├── main.rs            # Application entry
│   ├── app.rs             # Main App component
│   ├── components/        # Reusable UI components
│   │   ├── header.rs      # Top navigation bar
│   │   ├── footer.rs      # Footer component
│   │   ├── sidebar.rs     # Sidebar navigation
│   │   ├── dashboard.rs   # Dashboard widgets
│   │   └── auth/          # Authentication components
│   ├── pages/             # Page components
│   │   ├── home.rs        # Home page
│   │   ├── login.rs       # Login page
│   │   ├── profile.rs     # User profile
│   │   └── settings.rs    # Settings page
│   └── utils/             # Utility functions
└── styles/                # CSS stylesheets
    └── main.css           # Main stylesheet
```

## 🚀 Getting Started

### Prerequisites

- Rust 1.75+ with wasm32-unknown-unknown target
- Trunk 0.18+ for WASM bundling
- Node.js 20+ (for tooling)

### Installation

```bash
# Install Rust target
rustup target add wasm32-unknown-unknown

# Install Trunk
cargo install trunk

# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli
```

### Development

```bash
# Navigate to portal
cd antarmuka/portal

# Run development server with hot reload
trunk serve --port 8080

# Open browser
# http://localhost:8080
```

### Build for Production

```bash
# Build optimized WASM bundle
trunk build --release

# Output in dist/ directory
ls -lh dist/
```

## 🔧 Configuration

### Environment Variables

```bash
# API Gateway endpoint
API_BASE_URL=https://api.simpelv2.kejaksaan.go.id

# Authentication service
AUTH_URL=https://auth.simpelv2.kejaksaan.go.id

# OAuth configuration
OAUTH_CLIENT_ID=simpelv2-portal
OAUTH_REDIRECT_URI=https://portal.simpelv2.kejaksaan.go.id/callback

# Portal port (development)
PORT=8080
```

### Trunk Configuration

```toml
# Trunk.toml
[build]
target = "index.html"
dist = "dist"
public_url = "/"

[[proxy]]
backend = "http://localhost:3000/api"

[[hooks]]
stage = "pre_build"
command = "npm"
command_arguments = ["run", "build:css"]
```

## 📚 Microfrontend Integration

### Available Modules

| Module             | Description                  | Port | URL                     |
| ------------------ | ---------------------------- | ---- | ----------------------- |
| **Badiklat**       | Training & Education         | 8093 | /badiklat               |
| **Datun**          | Criminal Prosecution         | 8081 | /datun                  |
| **Intel**          | Intelligence & Surveillance  | 8082 | /intel                  |
| **Pidum**          | General Criminal Prosecution | 8088 | /pidum                  |
| **Pidsus**         | Special Crimes Prosecution   | 8087 | /pidsus                 |
| **Pidmil**         | Military Crimes Prosecution  | 8086 | /pidmil                 |
| **Pengawasan**     | Supervision & Oversight      | 8085 | /pengawasan             |
| **Pemulihan Aset** | Asset Recovery               | 8084 | /pemulihan-aset         |
| **Keuangan**       | Financial Management         | 8090 | /pembinaan/keuangan     |
| **Perencanaan**    | Strategic Planning           | 8091 | /pembinaan/perencanaan  |
| **Perlengkapan**   | Equipment Management         | 8092 | /pembinaan/perlengkapan |

### Navigation Example

```rust
use leptos::prelude::*;
use leptos_router::*;

#[component]
pub fn ModuleSelector() -> impl IntoView {
    view! {
        <div class="module-grid">
            <ModuleCard
                title="Badiklat"
                description="Training & Education"
                icon="fa-graduation-cap"
                href="/badiklat"
            />
            <ModuleCard
                title="Datun"
                description="Criminal Prosecution"
                icon="fa-gavel"
                href="/datun"
            />
            // ... more modules
        </div>
    }
}

#[component]
pub fn ModuleCard(
    title: &'static str,
    description: &'static str,
    icon: &'static str,
    href: &'static str,
) -> impl IntoView {
    view! {
        <a href=href class="module-card">
            <i class={format!("fa {}", icon)}></i>
            <h3>{title}</h3>
            <p>{description}</p>
        </a>
    }
}
```

## 🔐 Authentication Flow

### SSO Integration

```rust
use leptos::prelude::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    let login = move |_| {
        // Redirect to SSO provider
        window()
            .location()
            .set_href(&format!(
                "{}/oauth/authorize?client_id={}&redirect_uri={}",
                AUTH_URL, CLIENT_ID, REDIRECT_URI
            ))
            .expect("Failed to redirect");
    };

    view! {
        <div class="login-container">
            <div class="login-card">
                <Logo />
                <h1>"SIMPEL Portal"</h1>
                <p>"Kejaksaan Republik Indonesia"</p>
                <Button on:click=login variant="primary" size="large">
                    "Login with SSO"
                </Button>
            </div>
        </div>
    }
}
```

### Token Management

```rust
use wasm_bindgen::prelude::*;
use web_sys::Storage;

pub struct AuthService {
    storage: Storage,
}

impl AuthService {
    pub fn store_token(&self, token: &str) {
        self.storage
            .set_item("auth_token", token)
            .expect("Failed to store token");
    }

    pub fn get_token(&self) -> Option<String> {
        self.storage
            .get_item("auth_token")
            .ok()
            .flatten()
    }

    pub fn clear_token(&self) {
        self.storage
            .remove_item("auth_token")
            .expect("Failed to clear token");
    }
}
```

## 🧪 Testing

```bash
# Run unit tests
cargo test

# Run WASM tests in browser
wasm-pack test --headless --chrome

# Integration tests
cargo test --features integration-tests

# E2E tests
npm run test:e2e
```

## 📦 Dependencies

### Core Dependencies

- **leptos**: 0.8.x - Reactive web framework
- **leptos_router**: 0.8.x - Client-side routing
- **leptos_meta**: 0.8.x - Meta tags management
- **lib-ui**: Shared UI components (gunakan `lib_ui::prelude::*` pada contoh kode)

### Utilities

- **serde**: 1.0 - Serialization/deserialization
- **gloo**: 0.11 - Web API wrappers
- **uuid**: 1.11 - Unique identifiers
- **log**: 0.4 - Logging facade

## 🚢 Deployment

### Docker Build

```bash
# Build container image
docker build -t simpelv2-portal:latest .

# Run container
docker run -p 8080:8080 simpelv2-portal:latest
```

### Kubernetes Deployment

```bash
# Apply k8s manifests
kubectl apply -f k8s/portal-deployment.yaml

# Check status
kubectl get pods -l app=portal

# Access via ingress
kubectl get ingress portal
```

### Nginx Configuration

```nginx
server {
    listen 80;
    server_name portal.simpelv2.kejaksaan.go.id;

    root /usr/share/nginx/html;
    index index.html;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;

    # CSP header
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; connect-src 'self' https://api.simpelv2.kejaksaan.go.id;" always;

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

    # WASM mime type
    location ~ \.wasm$ {
        add_header Content-Type application/wasm;
    }
}
```

## 🔒 Security

### Content Security Policy

```
default-src 'self';
script-src 'self' 'unsafe-inline' 'unsafe-eval';
style-src 'self' 'unsafe-inline';
img-src 'self' data: https:;
font-src 'self' data:;
connect-src 'self' https://api.simpelv2.kejaksaan.go.id;
frame-ancestors 'none';
```

### Authentication & Authorization

- OAuth 2.0 / OpenID Connect
- JWT tokens with refresh mechanism
- Role-Based Access Control (RBAC)
- Multi-Factor Authentication (MFA) support
- Session timeout and auto-logout

### Security Headers

```
X-Frame-Options: SAMEORIGIN
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Referrer-Policy: no-referrer-when-downgrade
Strict-Transport-Security: max-age=31536000; includeSubDomains
```

## 📊 Performance

### WASM Bundle Size

- **Development**: ~1.2MB (unoptimized)
- **Production**: ~320KB (optimized + gzipped)

### Load Performance

- First Contentful Paint: <1.2s
- Time to Interactive: <2.5s
- Largest Contentful Paint: <2.0s
- Lighthouse Score: 98+

### Performance Optimizations

- Code splitting per route
- Lazy loading for modules
- Image optimization
- CSS purging
- WASM streaming compilation

## 🎨 Theming

### Custom Theme

```rust
use shared_microfrontend::theme::*;

pub fn apply_custom_theme() {
    let theme = Theme {
        primary_color: "#1E40AF", // Blue
        secondary_color: "#059669", // Green
        accent_color: "#DC2626", // Red
        background_color: "#F9FAFB",
        text_color: "#111827",
        font_family: "Inter, system-ui, sans-serif",
    };

    set_theme(theme);
}
```

## 🤝 Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for development guidelines.

## 📄 License

MIT License - See [LICENSE](../../LICENSE) for details.

## 📞 Support

- **Documentation**: https://docs.simpelv2.kejaksaan.go.id
- **Issue Tracker**: https://gitlab.com/analisiskebutuhan/simpelv2_web/issues
- **Email**: simpelv2@kejaksaan.go.id
- **Helpdesk**: +62-21-1234-5678

---

**Version**: 0.4.0
**Last Updated**: October 1, 2025
**Maintainer**: SIMPEL Team
