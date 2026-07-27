# Multi-stage build for Portal with Nginx - PORT 8080
#
# Toolchain: STABLE, pinned — see the same note in
# `antarmuka/perlengkapan/Dockerfile`. The floating `rustlang/rust:nightly` tag
# ICE'd on tokio under the fat-LTO release profile; no FE crate needs nightly.
FROM rust:1.97-bookworm AS chef
RUN cargo install cargo-chef --locked

FROM chef as planner
WORKDIR /workspace
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
WORKDIR /workspace

# Install cmake and build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
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
# Pin + --locked: an unlocked `cargo install trunk` on the nightly base resolves
# trunk's transitive deps to latest, which currently pulls a broken `time` combo
# (E0119 conflicting impl) and fails the build. --locked uses trunk's own lockfile.
RUN cargo install trunk --version 0.21.14 --locked
RUN cargo install wasm-bindgen-cli --version 0.2.100

# Copy source code
COPY . .

# Build portal
WORKDIR /workspace/antarmuka/portal

# Ensure CSS file exists (handled by trunk)
RUN mkdir -p dist

# Set environment variables for wasm-bindgen
ENV WASM_BINDGEN_WEAKREF=1
ENV RUST_BACKTRACE=1

# Fix wasm-bindgen cache issue
RUN mkdir -p /root/.cache/trunk/wasm-bindgen-0.2.100
RUN cp /usr/local/cargo/bin/wasm-bindgen /root/.cache/trunk/wasm-bindgen-0.2.100/wasm-bindgen || true

# Set build-time environment variables for WASM baking
# Build-time fake values, not used anymore but just to keep Trunk happy if still using env!
ARG PORTAL_URL="PORTAL_URL_PLACEHOLDER"
ARG AUTHENC_URL="AUTHENC_URL_PLACEHOLDER"
ENV PORTAL_URL=$PORTAL_URL
ENV AUTHENC_URL=$AUTHENC_URL

# Build with trunk (release mode)
# Assets served at /portal/ path prefix
RUN trunk build --release --public-url "/portal/"

# Stage 2: Nginx untuk serving hasil build - LISTEN ON PORT 8080
# nginx-unprivileged runs as non-root (uid 101) and listens on 8080 by default —
# matches our port and clears Trivy DS-0002 (no root container). pid relocated to
# /tmp in nginx.conf; html chowned so the entrypoint can write config.json.
FROM nginxinc/nginx-unprivileged:alpine
USER root

# Copy portal-specific nginx config
COPY antarmuka/portal/nginx.conf /etc/nginx/nginx.conf

# Copy build output ke Nginx html dir
COPY --from=builder /workspace/antarmuka/portal/dist /usr/share/nginx/html

# trunk builds with --public-url /portal/ (see above), so index.html requests
# every hashed asset as /portal/<file> — but the dist is copied flat into the
# web root. Without this link nginx 404s the .js/.wasm and the Leptos app
# never mounts (e2e run 28566784769). The self-symlink makes /portal/<file>
# resolve to the same flat files, matching the /portal prefix used behind the
# cluster ingress as well.
RUN ln -sn . /usr/share/nginx/html/portal

# Copy entrypoint script
COPY antarmuka/portal/entrypoint.sh /docker-entrypoint.sh

# Make the web root writable (entrypoint writes config.json at runtime) and the
# entrypoint executable, then drop back to the non-root nginx user.
RUN chown -R nginx:nginx /usr/share/nginx/html && chmod +x /docker-entrypoint.sh
USER nginx

# Expose port 8080 (match K8s deployment)
EXPOSE 8080

ENTRYPOINT ["/docker-entrypoint.sh"]
# Run nginx in foreground
CMD ["nginx", "-g", "daemon off;"]
