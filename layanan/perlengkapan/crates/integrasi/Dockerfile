# Build stage
FROM rust:latest as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev protobuf-compiler

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Transform Cargo.toml on the fly to only include necessary workspace members
RUN sed -i '/members = \[/,/\]/d' Cargo.toml && \
    sed -i '/default-members = \[/,/\]/d' Cargo.toml && \
    sed -i '/resolver = "2"/a members = ["lib/common", "lib/perlengkapan", "layanan/perlengkapan/crates/integrasi"]' Cargo.toml

# Copy lib-common and lib-perlengkapan
COPY lib/common lib/common
COPY lib/perlengkapan lib/perlengkapan

# Copy the service code
COPY layanan/perlengkapan/crates/integrasi layanan/perlengkapan/crates/integrasi

# Build CLI binary (batch job)
WORKDIR /usr/src/app
RUN cargo build --release -p layanan-perlengkapan-integrasi

# Build gRPC server binary
RUN cargo build --release -p layanan-perlengkapan-integrasi --features grpc --bin layanan-integrasi-grpc

# ============================================
# Runtime stage - CLI (for CronJob)
# ============================================
FROM debian:bookworm-slim as cli

WORKDIR /usr/app

# Install runtime dependencies
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the CLI binary
COPY --from=builder /usr/src/app/target/release/layanan-perlengkapan-integrasi /usr/app/layanan-integrasi

# Create output dir
RUN mkdir -p output

# Set env
ENV RUST_LOG=info

ENTRYPOINT ["./layanan-integrasi"]

# ============================================
# Runtime stage - gRPC Server
# ============================================
FROM debian:bookworm-slim as grpc

WORKDIR /usr/app

# Install runtime dependencies
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the gRPC binary
COPY --from=builder /usr/src/app/target/release/layanan-integrasi-grpc /usr/app/layanan-integrasi-grpc

# Set env
ENV RUST_LOG=info
ENV GRPC_HOST=0.0.0.0
ENV GRPC_PORT=50051

EXPOSE 50051

ENTRYPOINT ["./layanan-integrasi-grpc"]
