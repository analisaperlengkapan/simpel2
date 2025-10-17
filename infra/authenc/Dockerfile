# Use the official Rust image as the base image for building
FROM rust:1.90-slim AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml file
COPY Cargo.toml ./

# Copy the source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

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