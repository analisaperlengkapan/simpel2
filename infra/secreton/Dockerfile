# ============================================================================
# Multi-stage build for Secreton - Build from workspace root
# Run with: docker build -t secreton -f infra/secreton/Dockerfile .
# ============================================================================

# Stage 1: Builder
FROM rust:1.93-slim-bookworm AS builder

WORKDIR /workspace

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace Cargo files
COPY Cargo.toml Cargo.lock ./

# Patch Cargo.toml: delete old members list and add new minimal one
RUN sed -i '/members\s*=\s*\[/,/\]/d' Cargo.toml && \
    sed -i '/\[workspace\]/a members = ["lib/common", "infra/secreton", "infra/secreton/crates/core", "infra/secreton/crates/api", "infra/secreton/crates/storage", "infra/secreton/crates/crypto", "infra/secreton/crates/types", "infra/secreton/crates/agent", "infra/secreton/crates/cli", "infra/secreton/crates/grpc", "infra/secreton/crates/hsm", "infra/secreton/crates/k8s-operator"]' Cargo.toml || true

# Copy lib-common (workspace dependency)
COPY lib/common ./lib/common

# Copy secreton crate (entire infra/secreton directory)
COPY infra/secreton ./infra/secreton

# Build secreton api_server
WORKDIR /workspace/infra/secreton
RUN cargo build --release -p secreton-api --bin api_server --features raft-consensus

# Find and copy the binary
RUN find /workspace/target/release -name "api_server" -type f -executable | head -1 | xargs -I {} cp {} /workspace/api_server || \
    cp /workspace/target/release/api_server /workspace/api_server

# Strip binary
RUN strip /workspace/api_server 2>/dev/null || true
RUN ls -la /workspace/target/release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    tini \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r secreton -g 1000 \
    && useradd -r -u 1000 -g secreton -d /app -s /sbin/nologin secreton \
    && mkdir -p /app/data /app/config /app/logs /app/certs \
    && chown -R secreton:secreton /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder --chown=root:root /workspace/api_server /app/api_server

# Copy configuration
COPY infra/secreton/secreton.toml /app/config/secreton.toml

RUN chmod 555 /app/api_server

USER secreton

HEALTHCHECK --interval=30s --timeout=10s --start-period=15s --retries=3 \
    CMD curl -sf http://localhost:8200/v1/health || exit 1

EXPOSE 8200 8201 8300

ENV RUST_LOG=info,secreton=info \
    SECRETON_CONFIG=/app/config/secreton.toml

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/app/api_server"]
