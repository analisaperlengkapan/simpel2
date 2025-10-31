# Multi-stage build for Perlengkapan Microfrontend with Nginx
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

# Build perlengkapan
WORKDIR /workspace/antarmuka/pembinaan/perlengkapan

# Ensure CSS file exists in dist directory
RUN mkdir -p dist
COPY dist/tailwind.css dist/ || true

# Set environment variables for wasm-bindgen
ENV WASM_BINDGEN_WEAKREF=1
ENV RUST_BACKTRACE=1

# Fix wasm-bindgen cache issue
RUN mkdir -p /root/.cache/trunk/wasm-bindgen-0.2.100
RUN cp /usr/local/cargo/bin/wasm-bindgen /root/.cache/trunk/wasm-bindgen-0.2.100/wasm-bindgen || true

# Build with trunk (release mode)
RUN trunk build --release --public-url "/"

# Stage 2: Nginx untuk serving hasil build
FROM nginx:alpine

# Copy build output ke Nginx html dir
COPY --from=builder /workspace/antarmuka/pembinaan/perlengkapan/dist /usr/share/nginx/html

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
