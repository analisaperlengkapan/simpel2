# 🤖 AGENTS.md - Backend Services

> **Context**: This directory (`layanan/`) contains the **Backend Microservices** built with **Axum (HTTP)** and **Tonic (gRPC)**.

## 🏗️ Architecture Pattern

- **Framework**: Axum 0.8.x
- **State**: `Arc<AppState>` containing connection pools and gRPC clients.
- **Dependency Injection**: Use Traits for Repositories to enable mocking.

## 🛠️ Implementation Rules

### 1. 📝 Request Models

**Separate** your API request structs from your Database models.

- `CreateUserRequest`: Minimal fields, no ID, no timestamps.
- `User`: Full DB model with `Uuid`, `DateTime<Utc>`, `Option`s.

```rust
// ❌ BAD: Reusing DB model for input
// pub async fn create_user(Json(user): Json<User>) -> ...

// ✅ GOOD: Specific request struct
#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> ...
```

### 2. 🔍 Validation Endpoints

You **MUST** implement `GET /:id` for resources, even if the UI only lists them initially. This allows:
- Frontend to fetch details for editing.
- Validation logic to check existence.

```rust
// Ensure this exists
.route("/api/v1/items/{id}", get(get_item_by_id))
```

### 3. 💾 Database (PostgreSQL)

**WE DO NOT USE AN ORM (Diesel/SeaORM).**
We use `tokio-postgres` / `deadpool-postgres` with raw SQL.

#### Rules:
1.  **UUID Generation**: Generate UUIDs in **Rust Code** (`Uuid::new_v4()`), NOT in SQL (`gen_random_uuid()`).
    - *Reason*: `pgcrypto` extension might not be available/allowed in all envs.
2.  **Migrations**: Use **Programmatic Migrations**.
    - Define schema as a `const STR` in your code.
    - Execute it on service startup.
    - Do NOT rely on external `.sql` files or CLI tools for migrations in production.
3.  **No SQL Injection**: Always use Parameterized Queries (`$1`, `$2`).

```rust
// ✅ CORRECT
let id = Uuid::new_v4();
client.execute(
    "INSERT INTO users (id, name) VALUES ($1, $2)",
    &[&id, &name]
).await?;
```

---

## 🧪 Testing

### 1. 🎭 Mocking with Traits

Define a `Repository` trait to mock DB interactions.

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_user(&self, id: Uuid) -> Result<User, DbError>;
}

// In handler:
// pub async fn get_user(State(repo): State<Arc<dyn UserRepository>>) ...
```

### 2. 🧱 Unit Tests (`mockall`)

Use `mockall` to mock the repository in handler tests.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_create_user() {
        let mut mock_repo = MockUserRepository::new();
        mock_repo.expect_create()
            .returning(|_| Ok(User::new()));

        // Test handler with mock_repo
    }
}
```

### 3. ⚙️ Test Environment

- **Protobuf**: If running tests that involve gRPC build, ensure `PROTOC` env var is set if `protoc` is not in global PATH.
- **No DB**: Unit tests should NOT try to connect to a real Postgres (use mocks). Integration tests that need DB should be clearly marked or skipped in CI if no DB service is available.
