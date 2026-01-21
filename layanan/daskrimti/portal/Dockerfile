FROM rust:1.83-slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY lib/common ./lib/common
COPY layanan/daskrimti/portal ./layanan/daskrimti/portal

# Build release
RUN cargo build --release -p layanan-daskrimti-portal

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/portal /app/portal

# Set environment
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3010

EXPOSE 3010

CMD ["/app/portal"]
