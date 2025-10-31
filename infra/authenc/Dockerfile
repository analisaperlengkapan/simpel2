# Use the official Rust image as the base image for building
FROM rust:1.90-slim AS builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo files
COPY Cargo.toml Cargo.lock* ./

# Copy all necessary directories
COPY src ./src
COPY benches ./benches
COPY config ./config

# Build the application in release mode (skip benches)
RUN cargo build --release --bins

# Use a minimal base image for the runtime
FROM debian:bookworm-slim

# Install necessary runtime dependencies (if any)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/authenc /app/authenc

# Expose the port the app runs on
EXPOSE 3000

# Command to run the application
CMD ["./authenc"]