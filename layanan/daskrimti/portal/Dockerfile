FROM rust:latest as builder

WORKDIR /workspace

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./

# Patch Cargo.toml to only include lib-common and this service
RUN sed -i '/members\s*=\s*\[/,/\]/d' Cargo.toml && \
    sed -i '/\[workspace\]/a members = ["lib/common", "layanan/daskrimti/portal"]' Cargo.toml || true

COPY lib/common ./lib/common
COPY infra/authenc/proto ./infra/authenc/proto
COPY layanan/daskrimti/portal ./layanan/daskrimti/portal

# Build release
RUN cargo build --release -p layanan-portal

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
COPY --from=builder /workspace/target/release/portal /app/portal

# Set ownership
RUN chown -R appuser:appuser /app

USER appuser

# Set environment
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3010

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3010/health || exit 1

EXPOSE 3010

CMD ["/app/portal"]
