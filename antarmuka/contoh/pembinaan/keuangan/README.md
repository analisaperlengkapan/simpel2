# SIMPelv2 Keuangan - Financial Management


## 📋 Overview

**Keuangan - Financial Management** Sistem manajemen keuangan dan penganggaran BMN

## 🎯 Features

### Core Functionality
- ✅ **Case Management** - Manajemen perkara komprehensif
- ✅ **Document Processing** - Pemrosesan dokumen hukum
- ✅ **Workflow Automation** - Otomasi alur kerja
- ✅ **Reporting & Analytics** - Pelaporan dan analisis data
- ✅ **Integration** - Integrasi dengan sistem lain
- ✅ **Audit Trail** - Logging aktivitas lengkap

### Technical Features
- 🚀 **Leptos 0.8.x** - Modern reactive web framework
- 📦 **WebAssembly** - High-performance browser execution
- 🎨 **Shared Components** - Government-compliant UI library
- 🔒 **Security** - JWT authentication, RBAC, audit logging
- ♿ **Accessibility** - WCAG 2.1 AA compliant
- 📱 **Responsive** - Mobile-first design

## 🏗️ Architecture

```
antarmuka/pembinaan/keuangan/
├── Cargo.toml              # Dependencies & build config
├── Trunk.toml              # WASM build settings
├── Dockerfile              # Container image
├── index.html              # HTML template
├── src/
│   ├── lib.rs             # Library entry point
│   ├── app.rs             # Main App component
│   ├── components/        # Reusable UI components
│   ├── pages/             # Page components
│   ├── types.rs           # Type definitions
│   └── utils/             # Utility functions
└── styles/                # CSS stylesheets
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
# Navigate to microfrontend
cd antarmuka/pembinaan/keuangan

# Run development server with hot reload
trunk serve --port 8090

# Open browser
# http://localhost:8090
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

# Microfrontend port (development)
PORT=8090
```

### Trunk Configuration

```toml
# Trunk.toml
[build]
target = "index.html"
dist = "dist"

[[proxy]]
backend = "http://localhost:3000/api"
```

## 📚 Usage Examples

### Component Example

```rust
use leptos::prelude::*;
use lib_ui::prelude::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    let (count, set_count) = signal(0);

    view! {
        <div class="dashboard">
            <h2>"Dashboard"</h2>
            <Card>
                <p>"Count: " {count}</p>
                <Button on:click=move |_| set_count.update(|n| *n += 1)>
                    "Increment"
                </Button>
            </Card>
        </div>
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
```

## 📦 Dependencies

### Core Dependencies
- **leptos**: 0.8.x - Reactive web framework
- **leptos_router**: 0.8.x - Client-side routing
- **leptos_meta**: 0.8.x - Meta tags management
- **shared-microfrontend**: 0.4.0 - Shared UI components

### Utilities
- **serde**: 1.0 - Serialization/deserialization
- **gloo**: 0.11 - Web API wrappers
- **uuid**: 1.11 - Unique identifiers
- **log**: 0.4 - Logging facade

## 🚢 Deployment

### Docker Build

```bash
# Build container image
docker build -t simpelv2-pembinaan-keuangan:latest .

# Run container
docker run -p 8090:8080 simpelv2-pembinaan-keuangan:latest
```

### Kubernetes Deployment

```bash
# Apply k8s manifests
kubectl apply -f k8s/pembinaan-keuangan-deployment.yaml

# Check status
kubectl get pods -l app=pembinaan-keuangan
```

## 🔒 Security

### Authentication
- JWT tokens via Portal SSO
- Automatic token refresh
- Secure cookie storage

### Authorization
- Role-Based Access Control (RBAC)
- Permission checks per action
- Audit logging for sensitive operations

### Content Security Policy
```
default-src 'self';
script-src 'self' 'unsafe-inline';
style-src 'self' 'unsafe-inline';
connect-src 'self' https://api.simpelv2.kejaksaan.go.id;
```

## 📊 Performance

### WASM Bundle Size
- **Development**: ~800KB (unoptimized)
- **Production**: ~250KB (optimized + gzipped)

### Load Performance
- First Contentful Paint: <1.5s
- Time to Interactive: <3.0s
- Lighthouse Score: 95+

## 🤝 Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for development guidelines.

## 📄 License

MIT License - See [LICENSE](../../LICENSE) for details.

## 📞 Support

- **Documentation**: https://docs.simpelv2.kejaksaan.go.id
- **Issue Tracker**: https://gitlab.com/analisiskebutuhan/simpelv2_web/issues
- **Email**: simpelv2@kejaksaan.go.id

---

**Version**: 0.4.0
**Last Updated**: October 1, 2025
**Maintainer**: SIMPelv2 Team
