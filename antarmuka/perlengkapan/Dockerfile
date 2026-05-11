# Multi-stage build for Perlengkapan with Nginx - PORT 8080
FROM rustlang/rust:nightly as chef
RUN cargo install cargo-chef --locked

FROM chef as planner
WORKDIR /workspace
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
WORKDIR /workspace

# Install cmake and build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev protobuf-compiler \
    cmake clang libclang-dev perl \
    libc6-dev libc6-dev-i386 llvm lld \
    && rm -rf /var/lib/apt/lists/*
ENV BINDGEN_EXTRA_CLANG_ARGS="-I/usr/include -I/usr/include/x86_64-linux-gnu"

# Cook dependencies
COPY --from=planner /workspace/recipe.json recipe.json
ENV CC_wasm32_unknown_unknown=clang
ENV CFLAGS_wasm32_unknown_unknown="--target=wasm32-unknown-unknown"
ENV AR_wasm32_unknown_unknown=llvm-ar
RUN cargo chef cook --release --recipe-path recipe.json

# Install WASM toolchain
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk
RUN cargo install wasm-bindgen-cli --version 0.2.100

# Copy source code
COPY . .

# Build perlengkapan
WORKDIR /workspace/antarmuka/perlengkapan

# Ensure CSS file exists (handled by trunk)
RUN mkdir -p dist

# Set environment variables for wasm-bindgen
ENV WASM_BINDGEN_WEAKREF=1
ENV RUST_BACKTRACE=1

# Fix wasm-bindgen cache issue
RUN mkdir -p /root/.cache/trunk/wasm-bindgen-0.2.100
RUN cp /usr/local/cargo/bin/wasm-bindgen /root/.cache/trunk/wasm-bindgen-0.2.100/wasm-bindgen || true

# Build with trunk (release mode).
# Assets served at /perlengkapan/simpel/v2/ path prefix (canonical
# microfrontend mount point). Browser URL `/perlengkapan/simpel/v2/...`
# matches `<base href>` in index.html dan WASM router base path.
RUN trunk build --release --public-url "/perlengkapan/simpel/v2/"

# Stage 2: Nginx for serving build output - LISTEN ON PORT 8080
FROM nginx:alpine

# Copy perlengkapan-specific nginx config
COPY antarmuka/perlengkapan/nginx.conf /etc/nginx/nginx.conf

# Copy build output to Nginx html dir di path canonical
# /usr/share/nginx/html/perlengkapan/simpel/v2 — match URL prefix.
RUN rm -rf /usr/share/nginx/html/* && \
    mkdir -p /usr/share/nginx/html/perlengkapan/simpel
COPY --from=builder /workspace/antarmuka/perlengkapan/dist /usr/share/nginx/html/perlengkapan/simpel/v2

# Redirect HTML untuk path parent. Saat user akses /perlengkapan/ atau
# /perlengkapan/simpel/, browser dapat meta-refresh ke canonical
# /perlengkapan/simpel/v2/. Redirect dilakukan di HTML (BUKAN di Istio)
# supaya tetap di sisi frontend dan tidak menimbulkan loop/error di
# gateway.
RUN cat > /usr/share/nginx/html/perlengkapan/index.html <<'EOF'
<!DOCTYPE html>
<html lang="id"><head>
<meta charset="UTF-8">
<title>SIMPEL — Kejaksaan RI</title>
<meta http-equiv="refresh" content="0; url=/perlengkapan/simpel/v2/">
<link rel="canonical" href="/perlengkapan/simpel/v2/">
<script>window.location.replace('/perlengkapan/simpel/v2/');</script>
</head><body><p>Memuat aplikasi SIMPEL v2&hellip; <a href="/perlengkapan/simpel/v2/">Klik di sini bila tidak otomatis diarahkan</a>.</p></body></html>
EOF
RUN cp /usr/share/nginx/html/perlengkapan/index.html /usr/share/nginx/html/perlengkapan/simpel/index.html

# Change ownership so entrypoint can write to config.json
RUN chown -R nginx:nginx /usr/share/nginx/html

# Copy entrypoint script
COPY antarmuka/perlengkapan/entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

# Expose port 8080 (match K8s deployment)
EXPOSE 8080

ENTRYPOINT ["/docker-entrypoint.sh"]
# Run nginx in foreground
CMD ["nginx", "-g", "daemon off;"]
