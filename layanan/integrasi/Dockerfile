# Stage 1: Cargo Chef - Preparation
FROM rust:1.97-slim-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /usr/src/app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cargo Chef - Build
FROM chef AS builder
COPY --from=planner /usr/src/app/recipe.json recipe.json
# Install build dependencies
RUN apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 update && apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 install -y pkg-config libssl-dev protobuf-compiler && rm -rf /var/lib/apt/lists/*
# Cook dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and build
COPY . .
# Patch Cargo.toml
RUN sed -i '/^members = \[/,/^\]/c\members = [\n    "lib/core", "lib/backend", "lib/crypto",\n    "lib/perlengkapan",\n    "layanan/integrasi",\n]' Cargo.toml && \
    sed -i '/^default-members = \[/,/^\]/c\default-members = [\n    "layanan/integrasi",\n]' Cargo.toml

# Build the gRPC server, the out-of-band schema migration runner, and the
# scheduler. The migrate binary doesn't need the grpc feature but builds fine
# under it.
#
# `scheduler` is what the three sync CronJobs (siman/mysimkari/monsakti) run. It
# was NOT built or copied here, so every scheduled sync in production would have
# failed with `exec: "scheduler": executable file not found in $PATH` — external
# master data would never have synced at all. Staging could not catch it: staging
# suspends all three CronJobs by design and runs on mock data
# (project_staging_mock_external_data), so the first execution would have been in
# production. Keep this list in sync with the COPY lines below.
RUN cargo build --release -p layanan-integrasi --features grpc \
    --bin layanan-integrasi-server --bin integrasi-migrate --bin scheduler

# ============================================
# Runtime stage
# ============================================
FROM debian:bookworm-slim

WORKDIR /usr/app

RUN apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 update && apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 install -y --no-install-recommends libssl3 ca-certificates curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/layanan-integrasi-server /usr/app/layanan-integrasi
COPY --from=builder /usr/src/app/target/release/integrasi-migrate /usr/app/integrasi-migrate
COPY --from=builder /usr/src/app/target/release/scheduler /usr/app/scheduler

# Run as a non-root system user (Trivy DS-0002). The server binds 8080/50051
# (>1024, no privilege needed) and only writes under /usr/app/output.
RUN mkdir -p output \
    && groupadd --system --gid 10001 app \
    && useradd --system --uid 10001 --gid app --no-create-home app \
    && chown -R app:app /usr/app
USER app

ENV RUST_LOG=info
ENV GRPC_HOST=0.0.0.0
ENV GRPC_PORT=50051
ENV HTTP_HOST=0.0.0.0
ENV HTTP_PORT=8080

EXPOSE 50051
EXPOSE 8080

# Use CMD (not ENTRYPOINT) so a one-shot override can REPLACE the default server
# command. With ENTRYPOINT, docker-compose `command:` is only CMD (args appended
# to the entrypoint), so the `integrasi-migrate` compose service ran the server +
# an ignored arg instead of the migrate binary — it never exited, hanging the
# `service_completed_successfully` gate and timing out the e2e stack bring-up.
# Matches layanan/authenc/Dockerfile (`CMD ["./authenc"]`); Helm Jobs override via
# k8s `command:` which replaces both ENTRYPOINT and CMD.
CMD ["./layanan-integrasi"]
