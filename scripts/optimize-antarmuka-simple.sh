#!/bin/bash

# Simple Antarmuka Optimization Script
set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🚀 Optimizing existing microfrontends...${NC}"

# Get actual microfrontend directories (excluding shared)
MICROFRONTENDS=($(find "$PROJECT_ROOT/antarmuka" -maxdepth 2 -name "Cargo.toml" -exec dirname {} \; | grep -v shared | sort))

optimize_microfrontend() {
    local mf_path="$1"
    local mf_name=$(basename "$mf_path")

    echo -e "${YELLOW}📝 Optimizing: $mf_name${NC}"

    # Get current port from existing Trunk.toml or assign default
    local port=8080
    if [[ -f "$mf_path/Trunk.toml" ]] && grep -q "port" "$mf_path/Trunk.toml"; then
        port=$(grep "port" "$mf_path/Trunk.toml" | head -1 | sed 's/.*= *//' | tr -d ' ')
    fi

    # Create optimized Trunk.toml
    cat > "$mf_path/Trunk.toml" << EOF
[build]
target = "index.html"
dist = "dist"
release = true

# Advanced WASM optimization
[[build.wasm_opt]]
args = [
    "-Oz",                    # Maximum size optimization
    "--enable-bulk-memory",   # Enable bulk memory operations
    "--strip-debug",          # Remove debug information
    "--dce",                  # Dead code elimination
    "--vacuum",               # Final vacuum pass
]

# Advanced wasm-bindgen configuration
[[build.wasm-bindgen]]
target = "web"
typescript = false         # Disable TS generation for smaller bundles
debug = false             # No debug in production

[build.rust]
optimization-level = "z"   # Maximum size optimization
debug = false
lto = true

[serve]
port = $port
addresses = ["0.0.0.0"]
open = false

[watch]
watch = ["src", "styles"]
ignore = ["target", "dist", "Cargo.lock", ".git"]
debounce = 100
EOF

    # Add release profile to Cargo.toml if not exists
    if [[ -f "$mf_path/Cargo.toml" ]] && ! grep -q "\[profile.release\]" "$mf_path/Cargo.toml"; then
        cat >> "$mf_path/Cargo.toml" << 'EOF'

# WASM optimized build profile
[profile.release]
opt-level = "z"           # Maximum size optimization
lto = true                # Enable Link Time Optimization
codegen-units = 1         # Single codegen unit
panic = "abort"           # Smaller binary size
strip = true              # Strip symbols
debug = false            # No debug info
EOF
    fi

    # Create optimized CSS if styles directory exists or create it
    mkdir -p "$mf_path/styles"
    cat > "$mf_path/styles/main.css" << 'EOF'
/* Optimized CSS for SIMPelv2 */
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
body{font-family:system-ui,-apple-system,sans-serif;line-height:1.6;color:#2d3748;background:#f7fafc}
.container{max-width:1200px;margin:0 auto;padding:1rem}
.card{background:#fff;border-radius:8px;padding:1.5rem;box-shadow:0 1px 3px rgba(0,0,0,0.1);margin:1rem 0}
.btn{background:#2b6cb0;color:#fff;border:none;padding:0.75rem 1.5rem;border-radius:4px;cursor:pointer;font-size:0.9rem;transition:background 0.2s}
.btn:hover{background:#2c5aa0}
input,select,textarea{width:100%;padding:0.75rem;border:1px solid #e2e8f0;border-radius:4px;font-size:0.9rem}
input:focus,select:focus,textarea:focus{outline:none;border-color:#3182ce;box-shadow:0 0 0 1px #3182ce}
table{width:100%;border-collapse:collapse}
th,td{padding:0.75rem;text-align:left;border-bottom:1px solid #e2e8f0}
th{background:#f7fafc;font-weight:600}
.text-center{text-align:center}
.text-right{text-align:right}
.mt-4{margin-top:1rem}
.mb-4{margin-bottom:1rem}
.hidden{display:none}
.loading{display:flex;justify-content:center;align-items:center;min-height:200px;color:#718096}
@media(max-width:768px){.container{padding:0.5rem}.card{margin:0.5rem 0;padding:1rem}table{font-size:0.8rem}th,td{padding:0.5rem}}
EOF

    echo -e "   ✅ Optimized $mf_name (port: $port)"
}

# Clean build artifacts
echo -e "${BLUE}🧹 Cleaning build artifacts...${NC}"
find "$PROJECT_ROOT/antarmuka" -name "target" -type d -exec rm -rf {} + 2>/dev/null || true
find "$PROJECT_ROOT/antarmuka" -name "dist" -type d -exec rm -rf {} + 2>/dev/null || true
find "$PROJECT_ROOT/antarmuka" -name ".stage" -type d -exec rm -rf {} + 2>/dev/null || true

# Optimize each microfrontend
for mf_path in "${MICROFRONTENDS[@]}"; do
    optimize_microfrontend "$mf_path"
done

# Create build script
cat > "$PROJECT_ROOT/scripts/build-antarmuka.sh" << 'EOF'
#!/bin/bash
# Build all optimized microfrontends

set -e
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🏗️  Building all SIMPelv2 microfrontends...${NC}"

# Set WASM optimization environment
export CARGO_PROFILE_RELEASE_OPT_LEVEL=z
export CARGO_PROFILE_RELEASE_LTO=true

MICROFRONTENDS=($(find "$PROJECT_ROOT/antarmuka" -maxdepth 2 -name "Cargo.toml" -exec dirname {} \; | grep -v shared | sort))

for mf_path in "${MICROFRONTENDS[@]}"; do
    mf_name=$(basename "$mf_path")
    echo -e "${GREEN}Building $mf_name...${NC}"
    cd "$mf_path"
    if trunk build --release; then
        echo -e "   ✅ Built $mf_name"
        # Show bundle size
        if [[ -d "dist" ]]; then
            size=$(du -sh dist 2>/dev/null | cut -f1)
            echo -e "   📦 Bundle size: $size"
        fi
    else
        echo -e "   ❌ Failed to build $mf_name"
    fi
done

echo -e "${GREEN}✅ Build completed!${NC}"
EOF

chmod +x "$PROJECT_ROOT/scripts/build-antarmuka.sh"

echo -e "\n${GREEN}🎉 Optimization completed!${NC}"
echo -e "${BLUE}📊 Summary:${NC}"
echo -e "   ✅ Optimized $(echo "${MICROFRONTENDS[@]}" | wc -w) microfrontends"
echo -e "   ✅ Enhanced WASM optimization settings"
echo -e "   ✅ Minimized CSS files"
echo -e "   ✅ Generated unified build script"
echo -e "\n${BLUE}Next steps:${NC}"
echo -e "   • Build: ./scripts/build-antarmuka.sh"
echo -e "   • Test individual: cd antarmuka/portal && trunk serve"
