# ============================================================================
# Multi-stage build for Secreton - Optimized for size and performance
# ============================================================================

# Stage 1: Preparation
FROM rust:1.97-slim-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /usr/src/app

# Stage 2: Planner
FROM chef AS planner
COPY . .
# Patch Cargo.toml for workspace build (exclude ignored members)
RUN sed -i '/members\s*=\s*\[/,/\]/d' Cargo.toml && \
    sed -i '/\[workspace\]/a members = ["lib/core", "lib/backend", "lib/crypto", "layanan/secreton", "layanan/secreton/crates/core", "layanan/secreton/crates/api", "layanan/secreton/crates/storage", "layanan/secreton/crates/crypto", "layanan/secreton/crates/types", "layanan/secreton/crates/agent", "layanan/secreton/crates/cli", "layanan/secreton/crates/grpc", "layanan/secreton/crates/hsm", "layanan/secreton/crates/k8s-operator"]' Cargo.toml || true
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder
FROM chef AS builder
COPY --from=planner /usr/src/app/recipe.json recipe.json
# Install build dependencies
RUN apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 update && apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
# Cook dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and build
COPY . .
# Patch Cargo.toml for workspace build (exclude ignored members)
RUN sed -i '/members\s*=\s*\[/,/\]/d' Cargo.toml && \
    sed -i '/\[workspace\]/a members = ["lib/core", "lib/backend", "lib/crypto", "layanan/secreton", "layanan/secreton/crates/core", "layanan/secreton/crates/api", "layanan/secreton/crates/storage", "layanan/secreton/crates/crypto", "layanan/secreton/crates/types", "layanan/secreton/crates/agent", "layanan/secreton/crates/cli", "layanan/secreton/crates/grpc", "layanan/secreton/crates/hsm", "layanan/secreton/crates/k8s-operator"]' Cargo.toml || true

RUN cargo build --release -p secreton-api --bin api_server --features raft-consensus

# Stage 4: Runtime
FROM debian:bookworm-slim

RUN apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 update && apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl3 \
    tini \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r secreton -g 1000 \
    && useradd -r -u 1000 -g secreton -d /app -s /sbin/nologin secreton \
    && mkdir -p /app/data /app/config /app/logs /app/certs \
    && chown -R secreton:secreton /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/api_server /app/api_server
# Copy configuration
COPY layanan/secreton/secreton.toml /app/config/secreton.toml

RUN chmod 555 /app/api_server && \
    chown -R secreton:secreton /app

USER secreton

HEALTHCHECK --interval=30s --timeout=10s --start-period=15s --retries=3 \
    CMD curl -sf http://localhost:8200/v1/health || exit 1

EXPOSE 8200 8201 8300

ENV RUST_LOG=info,secreton=info \
    SECRETON_CONFIG=/app/config/secreton.toml

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/app/api_server"]
