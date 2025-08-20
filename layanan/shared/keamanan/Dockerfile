# Multi-stage build for Rust security service with cargo-chef
FROM rust:1.75-alpine AS chef
RUN cargo install cargo-chef --locked

FROM chef AS planner
WORKDIR /workspace
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /workspace
# Cook dependencies
COPY --from=planner /workspace/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and build
COPY . .
WORKDIR /workspace/layanan/berbagi/keamanan
RUN cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates openssl

# Create non-root user
RUN addgroup -g 1001 -S rust && \
    adduser -S rust -u 1001

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /workspace/target/release/layanan-keamanan /app/layanan-keamanan

# Change ownership to non-root user
RUN chown rust:rust /app/layanan-keamanan

USER rust
EXPOSE 3001

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3001/health || exit 1

CMD ["./layanan-keamanan"]
