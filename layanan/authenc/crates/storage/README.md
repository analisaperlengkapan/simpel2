# authenc-storage

Database layer for Authenc identity provider.

## Purpose

This crate implements the database layer with PostgreSQL:

- **Database**: Connection pool management with deadpool-postgres (20 connections default)
- **Prepared Statement Cache**: Query optimization for <10ms p95 latency
- **Transaction Support**: ACID guarantees for complex operations
- **Store Implementations**: PostgreSQL implementations of all store traits

## Store Implementations

### Implemented

- ✅ **PostgresUserStore**: User CRUD operations with search and filtering
  - `get_user(id)` - Get user by ID
  - `get_user_by_username(username, realm_id)` - Get user by username
  - `get_user_by_email(email, realm_id)` - Get user by email
  - `create_user(request)` - Create new user with validation
  - `update_user(id, request)` - Update user fields
  - `delete_user(id)` - Soft delete (disable account)
  - `list_users(realm_id, offset, limit)` - Paginated user list
  - `username_exists(username, realm_id)` - Check username availability
  - `email_exists(email, realm_id)` - Check email availability

### To Be Implemented

- `PostgresSessionStore`: Session management
- `PostgresRealmStore`: Realm configuration
- `PostgresClientStore`: OAuth2 client management
- `PostgresRoleStore`: Role and permission management

## Usage

```rust
use authenc_storage::{Database, PostgresUserStore};
use authenc_types::{
    domain::CreateUserRequest,
    traits::UserStore,
    RealmId,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database connection pool
    let db = Arc::new(Database::new(
        "postgres://authenc:password@localhost:5432/authenc",
        20  // pool size
    ).await?);

    // Initialize user store
    let user_store = PostgresUserStore::new(Arc::clone(&db));

    // Create a user
    let user = user_store.create_user(CreateUserRequest {
        username: "john_doe".to_string(),
        email: "john@example.com".to_string(),
        password: "$argon2id$...", // Pre-hashed
        realm_id: RealmId::new(),
    }).await?;

    println!("User created: {}", user.id);

    Ok(())
}
```

See `examples/user_store_example.rs` for a complete example.

## Performance

- Connection pooling with configurable pool size (default: 20)
- Prepared statement caching for frequently used queries
- <10ms p95 query latency (REQ-PERF-004)
- Batch operations support
- Index optimization for common queries

## Requirements

Implements requirements:
- REQ-ARCH-005 (Storage layer)
- REQ-PERF-004 (Database query latency <10ms p95)
- REQ-SCALE-001 (Connection pooling for horizontal scaling)
- REQ-USER-001 (User creation)
- REQ-USER-002 (User profile updates)
- REQ-USER-003 (User deletion - soft delete)
- REQ-USER-004 (User search and filtering)
