FROM rust:1.85-slim-bookworm as builder

WORKDIR /workspace

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./

# Patch Cargo.toml to only include lib-common and this service
RUN sed -i '/members\s*=\s*\[/,/\]/d' Cargo.toml && \
    sed -i '/\[workspace\]/a members = ["lib/common", "layanan/pembinaan/perlengkapan"]' Cargo.toml || true

COPY lib/common ./lib/common
COPY layanan/pembinaan/perlengkapan ./layanan/pembinaan/perlengkapan

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
