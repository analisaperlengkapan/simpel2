# Use the official Rust image as the base image for building
FROM rust:1.90-slim AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files (if exists)
COPY Cargo.toml ./
COPY Cargo.lock* ./

# Copy the source code
COPY crates ./crates

# Build the application in release mode
RUN cargo build --release --bin secreton

# Use a minimal base image for the runtime
FROM debian:bookworm-slim

# Install necessary runtime dependencies (if any)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/secreton /app/secreton

# Expose the port the app runs on
EXPOSE 8080

# Command to run the application
CMD ["./secreton"]