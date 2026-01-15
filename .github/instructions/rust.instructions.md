---
applyTo: "**/*.rs"
---

# Rust Code Guidelines for SIMPelv2

## Workspace Dependency Pattern
```toml
# Always use workspace = true
[dependencies]
axum = { workspace = true }
tokio = { workspace = true, features = ["full"] }
serde = { workspace = true, features = ["derive"] }
```

## Leptos 0.8.x Frontend Patterns
```rust
// ✅ CORRECT
use leptos::prelude::*;
let (count, set_count) = signal(0);
view! { <button on:click=move |_| set_count.update(|n| *n += 1)>"Click"</button> }

// ❌ WRONG - Old patterns
use leptos::*;                    // Use leptos::prelude::*
let count = create_signal(0);    // Use signal()
on_click=...                     // Use on:click=
```

## Axum 0.8.x Backend Patterns
```rust
// ✅ CORRECT - Modern Axum
use axum::{Router, routing::{get, post}, extract::{State, Json}};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/health", get(health_check))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ❌ WRONG - Deprecated
axum::Server::bind(&addr).serve(app.into_make_service()).await?;
```

## Error Handling
```rust
// Use thiserror for custom errors
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };
        (status, Json(serde_json::json!({"error": msg}))).into_response()
    }
}
```

## Security Requirements
- Use `Ed25519` for signing (NOT RSA)
- Use `ChaCha20-Poly1305` for symmetric encryption
- Use `X25519` for key exchange
- Never use `std::env::var()` for secrets - use Secreton client
- All service-to-service communication via gRPC with mTLS

## Database Patterns
```rust
// Use deadpool-postgres
use deadpool_postgres::Pool;

pub async fn get_item(pool: &Pool, id: Uuid) -> Result<Item, AppError> {
    let client = pool.get().await?;
    let row = client.query_one("SELECT * FROM items WHERE id = $1", &[&id]).await?;
    Ok(Item::from_row(row)?)
}
```
