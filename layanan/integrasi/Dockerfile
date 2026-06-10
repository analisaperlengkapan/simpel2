# Stage 1: Cargo Chef - Preparation
FROM rust:1.96-slim-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /usr/src/app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cargo Chef - Build
FROM chef AS builder
COPY --from=planner /usr/src/app/recipe.json recipe.json
# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev protobuf-compiler && rm -rf /var/lib/apt/lists/*
# Cook dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and build
COPY . .
# Patch Cargo.toml
RUN sed -i '/^members = \[/,/^\]/c\members = [\n    "lib/core", "lib/backend", "lib/crypto",\n    "lib/perlengkapan",\n    "layanan/integrasi",\n]' Cargo.toml && \
    sed -i '/^default-members = \[/,/^\]/c\default-members = [\n    "layanan/integrasi",\n]' Cargo.toml

# Build the gRPC server plus the out-of-band schema migration runner. The
# migrate binary doesn't need the grpc feature but builds fine under it.
RUN cargo build --release -p layanan-integrasi --features grpc \
    --bin layanan-integrasi-server --bin integrasi-migrate

# ============================================
# Runtime stage
# ============================================
FROM debian:bookworm-slim

WORKDIR /usr/app

RUN apt-get update && apt-get install -y libssl3 ca-certificates curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/layanan-integrasi-server /usr/app/layanan-integrasi
COPY --from=builder /usr/src/app/target/release/integrasi-migrate /usr/app/integrasi-migrate

RUN mkdir -p output

ENV RUST_LOG=info
ENV GRPC_HOST=0.0.0.0
ENV GRPC_PORT=50051
ENV HTTP_HOST=0.0.0.0
ENV HTTP_PORT=8080

EXPOSE 50051
EXPOSE 8080

ENTRYPOINT ["./layanan-integrasi"]
