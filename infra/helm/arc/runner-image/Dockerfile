# Custom ARC runner image untuk simpel2.
# Image actions-runner resmi minimal (tanpa C toolchain) → build Rust gagal
# `linker cc not found`. Tambahkan toolchain build yang dibutuhkan workspace
# (tonic/prost→protoc, aws-lc-sys→cmake/clang, openssl-sys→libssl-dev/pkg-config).
FROM ghcr.io/actions/actions-runner:2.334.0

USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential \
      pkg-config \
      protobuf-compiler \
      libssl-dev \
      cmake \
      clang \
      libclang-dev \
      perl \
      curl wget git unzip xz-utils jq file ca-certificates \
 && rm -rf /var/lib/apt/lists/*
USER runner
