# Stage 1: Chef
FROM rust:1.95-slim-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /workspace

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder
FROM chef AS builder
COPY --from=planner /workspace/recipe.json recipe.json
# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
# Cook dependencies
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
# Patch Cargo.toml to trim workspace members to just what's needed for this image
RUN sed -i '/members\s*=\s*\[/,/\]/d' Cargo.toml && \
    sed -i '/\[workspace\]/a members = ["lib/core", "lib/backend", "lib/crypto", "lib/perlengkapan", "layanan/integrasi", "layanan/perlengkapan"]' Cargo.toml || true

# Build release
RUN cargo build --release -p layanan-perlengkapan

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

# Copy binary
COPY --from=builder /workspace/target/release/layanan-perlengkapan /app/perlengkapan

# Set ownership
RUN chown -R appuser:appuser /app

USER appuser

# Set environment
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3020

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3020/health || exit 1

EXPOSE 3020

CMD ["/app/perlengkapan"]
