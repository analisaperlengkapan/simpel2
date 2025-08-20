# Rust backend service with cargo-chef
FROM rust:1.75 as chef
RUN cargo install cargo-chef --locked

FROM chef as planner
WORKDIR /workspace
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
WORKDIR /workspace

COPY --from=planner /workspace/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /workspace/target/release /app
WORKDIR /app
EXPOSE 7000
CMD ["./src"]
