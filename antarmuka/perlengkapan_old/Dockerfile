# Multi-stage build for Perlengkapan Microfrontend with Nginx
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
    libc6-dev libc6-dev-i386 llvm \
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

# Set build-time environment variables for WASM baking
ARG PORTAL_URL="https://10.1.7.121/portal"
ARG AUTHENC_URL="https://10.1.7.121/api/auth"
ENV PORTAL_URL=$PORTAL_URL
ENV AUTHENC_URL=$AUTHENC_URL

# Build with trunk (release mode)
RUN trunk build --release --public-url "/perlengkapan/"

# Stage 2: Nginx untuk serving hasil build - LISTEN ON PORT 8080
FROM nginx:alpine

# Copy main nginx config
COPY infra/nginx/nginx.conf /etc/nginx/nginx.conf

# Inject WASM MIME type to standard mime.types (ensure it exists)
RUN sed -i 's|}|    application/wasm wasm;\n}|' /etc/nginx/mime.types

# Copy build output ke Nginx html dir
COPY --from=builder /workspace/antarmuka/perlengkapan/dist /usr/share/nginx/html

EXPOSE 8080
CMD ["nginx", "-g", "daemon off;"]
