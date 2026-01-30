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
    sed -i '/resolver = "2"/a members = ["lib/common", "layanan/daskrimti/integrasi"]' Cargo.toml

# Copy lib-common
COPY lib/common lib/common

# Copy the service code
COPY layanan/daskrimti/integrasi layanan/daskrimti/integrasi

# Build
WORKDIR /usr/src/app/layanan/daskrimti/integrasi
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/app

# Install runtime dependencies
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /usr/src/app/target/release/layanan-integrasi /usr/app/layanan-integrasi

# Create output dir
RUN mkdir -p output

# Set env
ENV RUST_LOG=info

ENTRYPOINT ["./layanan-integrasi"]
