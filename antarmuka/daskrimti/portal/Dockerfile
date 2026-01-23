# Multi-stage build for Portal with Nginx - PORT 8080
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

# Build portal
WORKDIR /workspace/antarmuka/daskrimti/portal

# Ensure CSS file exists (handled by trunk)
RUN mkdir -p dist

# Set environment variables for wasm-bindgen
ENV WASM_BINDGEN_WEAKREF=1
ENV RUST_BACKTRACE=1

# Fix wasm-bindgen cache issue
RUN mkdir -p /root/.cache/trunk/wasm-bindgen-0.2.100
RUN cp /usr/local/cargo/bin/wasm-bindgen /root/.cache/trunk/wasm-bindgen-0.2.100/wasm-bindgen || true

# Build with trunk (release mode)
RUN trunk build --release --public-url "/"

# Stage 2: Nginx untuk serving hasil build - LISTEN ON PORT 8080
FROM nginx:alpine

# Copy custom nginx config untuk listen di port 8080
RUN echo 'server { \n\
    listen 8080; \n\
    listen [::]:8080; \n\
    server_name _; \n\
    root /usr/share/nginx/html; \n\
    index index.html; \n\
    \n\
    # Gzip compression \n\
    gzip on; \n\
    gzip_vary on; \n\
    gzip_min_length 1024; \n\
    gzip_types text/plain text/css text/xml text/javascript application/x-javascript application/javascript application/xml+rss application/json application/wasm; \n\
    \n\
    # Security headers \n\
    add_header X-Frame-Options "SAMEORIGIN" always; \n\
    add_header X-Content-Type-Options "nosniff" always; \n\
    add_header X-XSS-Protection "1; mode=block" always; \n\
    \n\
    # WASM mime type \n\
    types { \n\
    application/wasm wasm; \n\
    } \n\
    \n\
    # SPA routing - all routes to index.html \n\
    location / { \n\
    try_files $uri $uri/ /index.html; \n\
    } \n\
    \n\
    # Cache static assets \n\
    location ~* \\.(wasm|js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ { \n\
    expires 1y; \n\
    add_header Cache-Control "public, immutable"; \n\
    } \n\
    \n\
    # Health check endpoint \n\
    location /health { \n\
    access_log off; \n\
    return 200 "healthy\\n"; \n\
    add_header Content-Type text/plain; \n\
    } \n\
    }' > /etc/nginx/conf.d/default.conf

# Copy build output ke Nginx html dir
COPY --from=builder /workspace/antarmuka/daskrimti/portal/dist /usr/share/nginx/html

# Expose port 8080 (match K8s deployment)
EXPOSE 8080

# Run nginx in foreground
CMD ["nginx", "-g", "daemon off;"]
