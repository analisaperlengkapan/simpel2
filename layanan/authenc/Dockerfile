# ============================================================================
# Multi-stage build for Authenc - Build from workspace root
# Run with: docker build -t authenc -f layanan/authenc/Dockerfile .
# ============================================================================

# Stage 1: Builder
FROM rust:1.95-slim-bookworm AS builder

WORKDIR /workspace

# Install build dependencies (including protobuf for gRPC)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    g++ \
    gcc \
    pkg-config \
    libssl-dev \
    ca-certificates \
    protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

# Copy workspace Cargo files
COPY Cargo.toml Cargo.lock ./

# Patch Cargo.toml: delete old members list and add minimal set for authenc build
# Note: layanan/authenc itself is excluded (legacy shell, no src/) - binary is in crates/api
RUN sed -i '/members\s*=\s*\[/,/\]/d' Cargo.toml && \
    sed -i '/\[workspace\]/a members = ["lib/common", "layanan/authenc/crates/types", "layanan/authenc/crates/crypto", "layanan/authenc/crates/storage", "layanan/authenc/crates/core", "layanan/authenc/crates/api", "layanan/authenc/crates/webauthn", "layanan/authenc/crates/federation", "layanan/authenc/crates/iam-api"]' Cargo.toml || true

# Copy lib-common (workspace dependency)
COPY lib/common ./lib/common

# Copy authenc crate
COPY layanan/authenc ./layanan/authenc

# Build authenc binary (defined in crates/api/src/main.rs)
# captcha-debug: use --build-arg CAPTCHA_DEBUG=1 for E2E test images only
ARG CAPTCHA_DEBUG=0
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN if [ "$CAPTCHA_DEBUG" = "1" ]; then \
        cargo build --release --bin authenc --features authenc-api/captcha-debug; \
    else \
        cargo build --release --bin authenc; \
    fi

# Stage 2: Runtime - Use Ubuntu 24.04 for GLIBC compatibility (aligns with dev environment)
FROM ubuntu:24.04

# Install runtime dependencies only (including curl for healthcheck)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean

# Create non-root user for security
RUN useradd -m -u 1001 authenc

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /workspace/target/release/authenc /app/authenc

# Copy configuration files
COPY layanan/authenc/authenc.toml /app/authenc.toml

# Copy migration files for auto-migration on startup
COPY layanan/authenc/migrations /app/migrations

# Create config directory for environment-specific overrides
RUN mkdir -p /app/config && chown -R authenc:authenc /app

# Switch to non-root user
USER authenc

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8088/health || exit 1

# Expose ports
EXPOSE 8088 9088 9090

# Run the application
CMD ["./authenc"]
