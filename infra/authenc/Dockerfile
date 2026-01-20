# ============================================================================
# Multi-stage build for optimized production Docker image
# ============================================================================

# Stage 1: Builder - Use Debian trixie for consistent GLIBC version
FROM rust:1.90-slim-bookworm AS builder

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

# Set the working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock* ./

# Copy build.rs for proto compilation
COPY build.rs ./

# Create proto directory and copy proto files
COPY proto ./proto

# Copy source code
COPY src ./src
COPY benches ./benches

# Build in release mode with optimizations
# Use RUSTFLAGS to ensure compatibility
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release --bins

# Stage 2: Runtime - Use same base as builder for GLIBC compatibility
FROM debian:trixie-slim

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
COPY --from=builder /app/target/release/authenc /app/authenc

# Copy configuration files
COPY authenc.toml /app/authenc.toml

# Copy migration files for auto-migration on startup
COPY migrations /app/migrations

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
