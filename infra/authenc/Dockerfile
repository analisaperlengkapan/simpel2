# ============================================================================
# Multi-stage build for Authenc - Build from workspace root
# Run with: docker build -t authenc -f infra/authenc/Dockerfile .
# ============================================================================

# Stage 1: Builder
FROM rust:1.90-slim-bookworm AS builder

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

# Patch Cargo.toml: delete old members list and add new minimal one
RUN sed -i '/members\s*=\s*\[/,/\]/d' Cargo.toml && \
    sed -i '/\[workspace\]/a members = ["lib/common", "infra/authenc"]' Cargo.toml || true

# Copy lib-common (workspace dependency)
COPY lib/common ./lib/common

# Copy authenc crate
COPY infra/authenc ./infra/authenc

# Build authenc
WORKDIR /workspace/infra/authenc
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release

# Stage 2: Runtime - Use same base as builder for GLIBC compatibility
FROM debian:bookworm-slim

# Install runtime dependencies only (including curl for healthcheck)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean

# Create non-root user for security
RUN useradd -m -u 1000 authenc

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /workspace/target/release/authenc /app/authenc

# Copy configuration files
COPY infra/authenc/authenc.toml /app/authenc.toml

# Copy migration files for auto-migration on startup
COPY infra/authenc/migrations /app/migrations

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
