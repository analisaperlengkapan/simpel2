---
applyTo: "layanan/**/*.rs"
---

# Backend Microservice Guidelines

## Service Structure
```
layanan/[domain]/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point with axum::serve()
│   ├── config.rs         # Configuration loading
│   ├── error.rs          # Custom error types
│   ├── handlers/         # HTTP request handlers
│   │   └── mod.rs
│   ├── services/         # Business logic
│   │   └── mod.rs
│   └── models/           # Data models
│       └── mod.rs
└── migrations/           # Refinery SQL migrations
```

## Main Entry Point
```rust
use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::init();

    let state = AppState::new().await?;

    let app = Router::new()
        .route("/health", get(health))
        .nest("/api/v1", api_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
```

## Inter-Service Communication
```rust
// ✅ CORRECT - gRPC to Authenc
use tonic::transport::Channel;

let channel = Channel::from_static("https://authenc:9088")
    .tls_config(tls)?
    .connect().await?;
let mut client = AuthClient::new(channel);

// ❌ WRONG - Direct HTTP calls to infrastructure
reqwest::get("http://authenc:8088/validate").await?;
```

## Database Access
```rust
use deadpool_postgres::Pool;
use lib_storage::postgres::create_pool;

let pool = create_pool(&config.database_url).await?;
```

## Build Commands
```bash
cargo build -p layanan-portal
cargo test -p layanan-portal
cargo run -p layanan-portal
```
