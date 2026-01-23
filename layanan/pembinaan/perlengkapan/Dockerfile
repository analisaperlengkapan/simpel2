FROM rust:1.90-slim-bookworm as builder

WORKDIR /app

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
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/layanan-perlengkapan /app/perlengkapan

# Set environment
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3020

EXPOSE 3020

CMD ["/app/perlengkapan"]
