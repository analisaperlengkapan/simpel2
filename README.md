# SIMPelv2 - Sistem Informasi Manajemen Pengelolaan BMN

SIMPelv2 is a comprehensive State Property Management System (Sistem Informasi Manajemen Pengelolaan Barang Milik Negara) for the Attorney General's Office of the Republic of Indonesia (Kejaksaan Agung RI). It utilizes a modern microservices architecture with a Rust-based stack for high performance and security.

## 🏗 Architecture

The project is organized as a Rust workspace with a monorepo structure:

- **Frontend (`antarmuka/`)**: Built with [Leptos](https://leptos.dev), a reactive web framework for Rust (WebAssembly).
  - Divided into multiple microfrontends (e.g., `portal`, `badiklat`, `datun`) shared components (`shared`).
- **Backend (`layanan/`)**: Built with [Axum](https://github.com/tokio-rs/axum), a modular and robust web framework.
  - Divided into domain-specific services matching the frontend modules.
- **Infrastructure (`infra/`)**: Contains infrastructure definitions and core services.
  - Includes `authenc` (Identity & Access Management) and `secreton` (Secret Management).

## 🛠 Tech Stack

- **Language**: Rust (Edition 2024)
- **Frontend**: Leptos (WASM), Trunk (Bundler)
- **Backend**: Axum, Tokio
- **Database**: PostgreSQL (with `deadpool` and `refinery` for migrations)
- **Caching**: Redis
- **Containerization**: Docker & Docker Compose

## 📋 Prerequisites

Ensure you have the following installed:

- **Rust**: Latest stable version (or version specified in `Cargo.toml`).
- **Docker & Docker Compose**: For running databases and infrastructure services.
- **Trunk**: For building and serving the frontend (`cargo install trunk`).
- **Make**: For running build scripts.
- **Protobuf Compiler**: Required for building services (`protoc`).

## 🚀 Getting Started

### 1. Setup Environment

1.  Clone the repository.
2.  Configure your environment variables. Check the `infra` or service directories for specific `.env` requirements. A standard `.env` file is usually required at the root or within specific service directories.

### 2. Start Infrastructure

Start the required databases (PostgreSQL, Redis) using Docker Compose:

```bash
docker compose up -d postgres redis
```

### 3. Running the Application

This project uses a `Makefile` to simplify development tasks.

**Common Commands:**

- **Start Development Environment:**
  ```bash
  make up-dev
  ```

- **Build All Backend Services:**
  ```bash
  cargo build --workspace
  ```

- **Serve a Specific Frontend (e.g., Portal):**
  ```bash
  cd antarmuka/portal
  trunk serve
  ```

- **Run Tests:**
  ```bash
  cargo test
  ```

For a full list of available commands, run:
```bash
make help
```

## 📂 Project Structure

```
simpelv2/
├── antarmuka/          # Frontend applications (Leptos)
│   ├── portal/         # Main entry point
│   ├── shared/         # Shared UI components
│   └── ...             # Other microfrontends
├── layanan/            # Backend services (Axum)
│   ├── shared/         # Shared backend logic
│   └── ...             # Domain services
├── infra/              # Infrastructure (K8s, Docker, Secreton)
├── scripts/            # Automation and build scripts
├── Cargo.toml          # Workspace configuration
├── Makefile            # Build system
└── docker-compose.yml  # Docker composition
```

## 🤝 Contributing

Please refer to `CONTRIBUTING.md` for guidelines on how to contribute to this project.

## 📄 License

This project is licensed under the Apache-2.0 License.
