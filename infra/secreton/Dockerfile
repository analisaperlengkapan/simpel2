# Multi-stage production Dockerfile for Secreton
# Optimized for security, size, and performance

# Build stage
FROM rust:1.90-slim AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/types/Cargo.toml ./crates/types/
COPY crates/core/Cargo.toml ./crates/core/
COPY crates/crypto/Cargo.toml ./crates/crypto/
COPY crates/storage/Cargo.toml ./crates/storage/
COPY crates/api/Cargo.toml ./crates/api/
COPY crates/agent/Cargo.toml ./crates/agent/
COPY crates/cli/Cargo.toml ./crates/cli/
COPY crates/hsm/Cargo.toml ./crates/hsm/
COPY crates/grpc/Cargo.toml ./crates/grpc/

# Create dummy files for dependency caching
RUN mkdir -p crates/types/src crates/core/src crates/crypto/src crates/storage/src \
    crates/api/src crates/agent/src crates/cli/src crates/hsm/src crates/grpc/src && \
    echo "fn main() {}" > crates/types/src/lib.rs && \
    echo "fn main() {}" > crates/core/src/lib.rs && \
    echo "fn main() {}" > crates/crypto/src/lib.rs && \
    echo "fn main() {}" > crates/storage/src/lib.rs && \
    echo "fn main() {}" > crates/api/src/lib.rs && \
    echo "fn main() {}" > crates/agent/src/lib.rs && \
    echo "fn main() {}" > crates/cli/src/main.rs && \
    echo "fn main() {}" > crates/hsm/src/lib.rs && \
    echo "fn main() {}" > crates/grpc/src/lib.rs

# Build dependencies (will be cached)
RUN cargo build --release --bin secreton

# Copy actual source code
COPY crates ./crates

# Build the actual application
RUN touch crates/*/src/**/*.rs && \
    cargo build --release --bin secreton

# Strip debug symbols to reduce binary size
RUN strip /build/target/release/secreton

# Runtime stage - minimal distroless image
FROM gcr.io/distroless/cc-debian12

# Add security labels
LABEL org.opencontainers.image.title="Secreton Vault" \
      org.opencontainers.image.description="Enterprise Security Vault System" \
      org.opencontainers.image.vendor="Cipherce Security" \
      org.opencontainers.image.version="0.1.0" \
      org.opencontainers.image.source="https://github.com/cipherce/secreton"

# Create non-root user
USER nonroot:nonroot

WORKDIR /app

# Copy binary from builder
COPY --from=builder --chown=nonroot:nonroot /build/target/release/secreton /app/secreton

# Copy configuration example
COPY --chown=nonroot:nonroot secreton.toml.example /app/secreton.toml.example

# Healthcheck
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/app/secreton", "status"] || exit 1

# Expose ports
# 8200 - API (HTTP)
# 8201 - API (HTTPS)
# 8300 - Cluster communication (Raft)
EXPOSE 8200 8201 8300

# Set environment variables
ENV RUST_LOG=info \
    RUST_BACKTRACE=1

# Use exec form for proper signal handling
ENTRYPOINT ["/app/secreton"]
CMD ["server"]
