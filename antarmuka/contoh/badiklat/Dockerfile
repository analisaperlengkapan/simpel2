# Stage 1: Install cargo-chef
FROM rustlang/rust:nightly as chef
RUN cargo install cargo-chef --locked

# Stage 2: Plan dependencies for workspace
FROM chef as planner
WORKDIR /workspace
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies and application
FROM chef as builder
WORKDIR /workspace

# Copy recipe and cook dependencies
COPY --from=planner /workspace/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Install trunk and wasm target
RUN rustup target add wasm32-unknown-unknown && cargo install trunk

# Copy source code
COPY . .

# Change to service directory
WORKDIR /workspace/antarmuka/badiklat

EXPOSE 7007
CMD ["trunk", "serve", "--port", "7007", "--public-url", "/"] 