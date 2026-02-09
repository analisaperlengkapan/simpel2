# Multi-stage build with cargo-chef for workspace compatibility
FROM rustlang/rust:nightly as chef
RUN cargo install cargo-chef --locked

FROM chef as planner
WORKDIR /workspace
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
WORKDIR /workspace

# Cook dependencies
COPY --from=planner /workspace/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Install WASM toolchain
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk
RUN cargo install wasm-bindgen-cli --version 0.2.100

# Copy source code
COPY . .

# Build the specific service
WORKDIR /workspace/antarmuka/pengawasan

# Set environment variables for WASM
ENV WASM_BINDGEN_WEAKREF=1
ENV RUST_BACKTRACE=1

EXPOSE 7004
CMD ["trunk", "serve", "--port", "7004", "--public-url", "/"]
