#!/bin/bash

# 🚀 SIMPelv2 Project Initialization Tool
# Create new microservices and components

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

WORKSPACE_ROOT="/var/www/simpelv2"

# Initialize new project components
init_project() {
    local component_type="${1:-service}"
    local name="${2:-new-component}"

    echo -e "\n${GREEN}🚀 Initializing New Project Component${NC}"
    echo "===================================="

    cd "$WORKSPACE_ROOT"

    case "$component_type" in
        "service"|"microservice")
            echo "🦀 Creating new microservice: $name"

            local service_dir="layanan/$name"
            if [[ -d "$service_dir" ]]; then
                echo "❌ Service $name already exists"
                return 1
            fi

            mkdir -p "$service_dir/src"

            # Use AI tools to generate the service
            if [[ -f "scripts/tools/ai/ai-tools.sh" ]]; then
                bash scripts/tools/ai/ai-tools.sh generate service "$name"
            else
                echo "⚠️  AI tools not found, creating basic structure"
                create_basic_service "$service_dir" "$name"
            fi

            # Add to workspace
            echo "📝 Adding to workspace Cargo.toml..."
            if ! grep -q "layanan/$name" Cargo.toml 2>/dev/null; then
                # Add to members array
                sed -i "/members = \[/a\\    \"layanan/$name\"," Cargo.toml
            fi

            echo "✅ Service $name created successfully"
            ;;

        "frontend"|"antarmuka")
            echo "🌐 Creating new frontend: $name"

            local frontend_dir="antarmuka/$name"
            if [[ -d "$frontend_dir" ]]; then
                echo "❌ Frontend $name already exists"
                return 1
            fi

            mkdir -p "$frontend_dir/src"
            create_basic_frontend "$frontend_dir" "$name"

            echo "✅ Frontend $name created successfully"
            ;;

        *)
            echo "❌ Unknown component type: $component_type"
            echo "Usage: init_project [service|frontend] <name>"
            return 1
            ;;
    esac
}

create_basic_service() {
    local service_dir="$1"
    local name="$2"

    # Create Cargo.toml
    cat > "$service_dir/Cargo.toml" <<EOF
[package]
name = "$name"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
EOF

    # Create main.rs
    cat > "$service_dir/src/main.rs" <<EOF
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting $name service...");

    // TODO: Implement service logic

    Ok(())
}
EOF

    # Create Dockerfile
    cat > "$service_dir/Dockerfile" <<EOF
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/$name /app/
WORKDIR /app
EXPOSE 7000
CMD ["./$name"]
EOF
}

create_basic_frontend() {
    local frontend_dir="$1"
    local name="$2"

    # Create Cargo.toml
    cat > "$frontend_dir/Cargo.toml" <<EOF
[package]
name = "$name"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = { version = "0.6", features = ["csr", "nightly"] }
leptos_meta = { version = "0.6", features = ["csr", "nightly"] }
leptos_router = { version = "0.6", features = ["csr", "nightly"] }
wasm-bindgen = "0.2"

[lib]
crate-type = ["cdylib"]
EOF

    # Create lib.rs
    cat > "$frontend_dir/src/lib.rs" <<EOF
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html lang="en"/>
        <Title text="$name - SIMPelv2"/>

        <Router>
            <nav>
                <A href="/">"Home"</A>
            </nav>
            <Routes>
                <Route path="" view=HomePage/>
            </Routes>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <main>
            <h1>"Welcome to $name"</h1>
            <p>"This is a new SIMPelv2 frontend component."</p>
        </main>
    }
}
EOF

    # Create index.html
    cat > "$frontend_dir/index.html" <<EOF
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1"/>
    <title>$name - SIMPelv2</title>
</head>
<body>
    <div id="root"></div>
</body>
</html>
EOF

    # Create Trunk.toml
    cat > "$frontend_dir/Trunk.toml" <<EOF
[build]
target = "index.html"

[watch]
ignore = ["dist"]

[serve]
address = "127.0.0.1"
port = 3000
EOF
}

# Main execution
main() {
    case "${1:-help}" in
        "service"|"microservice")
            if [[ -z "${2:-}" ]]; then
                echo "❌ Service name required"
                echo "Usage: $0 service <name>"
                exit 1
            fi
            init_project "service" "$2"
            ;;
        "frontend"|"antarmuka")
            if [[ -z "${2:-}" ]]; then
                echo "❌ Frontend name required"
                echo "Usage: $0 frontend <name>"
                exit 1
            fi
            init_project "frontend" "$2"
            ;;
        "help"|"-h"|"--help")
            echo -e "${CYAN}🚀 SIMPelv2 Project Initialization Tool${NC}"
            echo ""
            echo "Usage: $0 <command> <name>"
            echo ""
            echo "Commands:"
            echo "  service <name>    Create new microservice in layanan/"
            echo "  frontend <name>   Create new frontend in antarmuka/"
            echo "  help             Show this help"
            echo ""
            echo "Examples:"
            echo "  $0 service auth"
            echo "  $0 frontend dashboard"
            ;;
        *)
            echo "❌ Unknown command: $1"
            echo "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
}

main "$@"
