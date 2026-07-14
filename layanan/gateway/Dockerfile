# Stage 1: Cargo Chef - Preparation
FROM rust:1.97-slim-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /usr/src/app

FROM chef AS planner
COPY . .
# Trim the workspace to just the gateway (it has no path-deps on lib/*), so the
# dependency recipe — and therefore the `cook` step below — stays lean instead
# of compiling the whole monorepo's dep tree.
RUN sed -i '/^members = \[/,/^\]/c\members = [\n    "layanan/gateway",\n]' Cargo.toml && \
    sed -i '/^default-members = \[/,/^\]/c\default-members = [\n    "layanan/gateway",\n]' Cargo.toml
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cargo Chef - Build
FROM chef AS builder
COPY --from=planner /usr/src/app/recipe.json recipe.json
# protobuf-compiler is required by tonic-prost-build (the gateway compiles the
# authenc/secreton/integrasi protos as a gRPC client).
RUN apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 update \
    && apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 install -y pkg-config libssl-dev protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source (brings back the full Cargo.toml) and re-trim, then build. The
# proto files under layanan/{authenc,secreton,integrasi}/proto come in via
# `COPY . .` and are read by the gateway's build.rs.
COPY . .
RUN sed -i '/^members = \[/,/^\]/c\members = [\n    "layanan/gateway",\n]' Cargo.toml && \
    sed -i '/^default-members = \[/,/^\]/c\default-members = [\n    "layanan/gateway",\n]' Cargo.toml
RUN cargo build --release -p layanan-gateway --bin gateway

# ============================================
# Runtime stage
# ============================================
FROM debian:bookworm-slim

WORKDIR /usr/app

RUN apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 update \
    && apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 install -y --no-install-recommends libssl3 ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/gateway /usr/app/gateway

# Non-root system user (Trivy DS-0002). Binds :8090 (>1024, no privilege).
RUN groupadd --system --gid 10001 app \
    && useradd --system --uid 10001 --gid app --no-create-home app \
    && chown -R app:app /usr/app
USER app

ENV RUST_LOG=info
ENV GATEWAY_PORT=8090
# Upstream gRPC endpoints — overridden per environment (compose / Helm).
ENV AUTHENC_GRPC_URL=http://authenc:50051
ENV SECRETON_GRPC_URL=http://secreton:50053
ENV INTEGRASI_GRPC_URL=http://layanan-integrasi:50052

EXPOSE 8090

# CMD (not ENTRYPOINT) so one-shot overrides can replace the command — matches
# the other layanan/* Dockerfiles.
CMD ["./gateway"]
