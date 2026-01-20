# Multi-stage production Dockerfile for Secreton
# Optimized for security, size, and performance
#
# Security features:
# - Non-root user execution
# - Read-only filesystem compatible
# - Minimal attack surface
# - Security scanning friendly

# ============================================================
# Build stage - Rust compilation with all build dependencies
# ============================================================
FROM rust:1.90-slim AS builder

WORKDIR /build

# Install build dependencies (minimal set)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Copy all source files
COPY . .

# Build the API server binary with production features
RUN cargo build --release -p secreton-api --bin api_server --features raft-consensus 2>&1 | tail -20

# Strip debug symbols for smaller binary (~50-70% size reduction)
RUN find target/release/deps -name "api_server-*" -type f -executable | head -1 | xargs -I {} sh -c 'strip {} && cp {} /build/api_server'

# ============================================================
# Runtime stage - Minimal production image
# ============================================================
FROM debian:trixie-slim

# Security labels (OCI standard)
LABEL org.opencontainers.image.title="Secreton Vault" \
    org.opencontainers.image.description="Enterprise Security Vault System" \
    org.opencontainers.image.vendor="SIMPel Kejaksaan RI" \
    org.opencontainers.image.version="0.1.0" \
    org.opencontainers.image.authors="DevSecOps Team" \
    org.opencontainers.image.licenses="Proprietary" \
    org.opencontainers.image.source="https://gitlab.kejaksaan.go.id/simpel/secreton" \
    # Security scanning metadata
    security.privileged="false" \
    security.capabilities.drop="ALL" \
    security.read-only-root-filesystem="true"

# Install minimal runtime dependencies and create non-root user
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    tini \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean \
    # Create non-root user for security
    && groupadd -r secreton -g 1000 \
    && useradd -r -u 1000 -g secreton -d /app -s /sbin/nologin secreton \
    # Create required directories
    && mkdir -p /app/data /app/config /app/logs /app/certs \
    && chown -R secreton:secreton /app

WORKDIR /app

# Copy binary from builder (owned by root, executed by secreton)
COPY --from=builder --chown=root:root /build/api_server /app/api_server

# Copy configuration files (read-only)
COPY --chown=secreton:secreton secreton.toml /app/config/secreton.toml
COPY --chown=secreton:secreton secreton.toml.example /app/config/secreton.toml.example

# Set executable permission (no write)
RUN chmod 555 /app/api_server \
    && chmod 444 /app/config/*.toml

# Switch to non-root user
USER secreton

# Healthcheck (with proper timeout and start period for cold starts)
HEALTHCHECK --interval=30s --timeout=10s --start-period=15s --retries=3 \
    CMD curl -sf http://localhost:8200/v1/health || exit 1

# Expose ports
# 8200 - REST API (HTTP/HTTPS)
# 8201 - gRPC API
# 8300 - Raft cluster communication
EXPOSE 8200 8201 8300

# Production environment variables
ENV RUST_LOG=info,secreton=info \
    RUST_BACKTRACE=0 \
    SECRETON_CONFIG=/app/config/secreton.toml \
    # Security: disable colored output for log parsing
    NO_COLOR=1 \
    # Memory optimization
    MALLOC_ARENA_MAX=2

# Use tini as init process for proper signal handling and zombie reaping
ENTRYPOINT ["/usr/bin/tini", "--"]

# Run the server (config path set via SECRETON_CONFIG env var)
CMD ["/app/api_server"]
