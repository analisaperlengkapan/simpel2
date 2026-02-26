# Namespace Management

This module provides hierarchical namespace management for SIMKARI multi-tenancy with path resolution and validation.

## Overview

The namespace system supports the Kejaksaan RI organizational hierarchy:
- **Pusat** (Central) - National level (Kejaksaan Agung)
- **Wilayah** (Regional) - Provincial level (Kejaksaan Tinggi)
- **Satker** (Work Unit) - Individual units (Kejaksaan Negeri, etc.)

## Module Structure

- `hierarchy.rs` - Namespace hierarchy management (Pusat -> Wilayah -> Satker)
- `access.rs` - Access control based on JWT claims
- `path.rs` - **NEW**: Path parsing and resolution for namespace-scoped secrets
- `validation.rs` - **NEW**: Validation utilities for secret operations

## Secret Path Format

All secrets are scoped to namespaces using the format:
```
{namespace_id}/path/to/secret
```

### Examples

- `satker-kja001/db/password` - Database password in Kejaksaan Negeri Medan
- `wilayah-sumut/config/api-key` - API key for Sumatera Utara region
- `pusat/master/encryption-key` - Master encryption key at central level
- `satker-kja001/app/db/prod/password` - Nested path for production database

### Valid Namespace IDs

- `pusat` - Central/National level (Kejaksaan Agung)
- `wilayah-{code}` - Regional level (e.g., `wilayah-sumut`, `wilayah-jabar`)
- `satker-{code}` - Work unit level (e.g., `satker-kja001`, `satker-kja101`)

## Access Control Rules

Access is enforced based on the `admin_level` field in JWT claims:

### AdminLevel::Pusat
- **Access**: All namespaces
- **Use Case**: National administrators at Kejaksaan Agung

### AdminLevel::EselonI
- **Access**: Directorate-specific namespaces (currently all, can be refined)
- **Use Case**: Directorate-level administrators

### AdminLevel::Wilayah
- **Access**: Own wilayah and all child satker namespaces
- **Requires**: `wilayah_code` in JWT claims (e.g., "SUMUT")
- **Use Case**: Regional administrators at Kejaksaan Tinggi
- **Example**: Wilayah SUMUT admin can access:
  - `wilayah-sumut/*`
  - `satker-kja001/*` (Medan)
  - `satker-kja002/*` (Binjai)

### AdminLevel::Satker
- **Access**: Only own satker namespace
- **Requires**: `satker_code` in JWT claims (e.g., "KJA001")
- **Use Case**: Unit-level administrators at Kejaksaan Negeri
- **Example**: Satker KJA001 admin can only access:
  - `satker-kja001/*`

## Usage Examples

### 1. Parsing Secret Paths

```rust
use secreton_core::namespace::NamespacePath;

// Parse a secret path
let path = NamespacePath::parse("satker-kja001/db/password")?;
assert_eq!(path.namespace_id, "satker-kja001");
assert_eq!(path.secret_path, "db/password");
assert_eq!(path.full_path, "satker-kja001/db/password");

// Validate against hierarchy
path.validate_with_hierarchy(&hierarchy)?;

// Get namespace details
let namespace = path.get_namespace(&hierarchy).unwrap();
println!("Namespace: {}", namespace.name);

// Check namespace type
let ns_type = path.namespace_type();
assert_eq!(ns_type, Some(NamespaceType::Satker));
```

### 2. Validating Secret Operations

```rust
use secreton_core::namespace::{NamespaceValidator, JwtClaims};
use std::sync::Arc;

// Create validator
let hierarchy = Arc::new(hierarchy);
let access_control = Arc::new(NamespaceAccessControl::new((*hierarchy).clone()));
let audit_logger = Some(Arc::new(audit_logger));
let validator = NamespaceValidator::new(hierarchy, access_control, audit_logger);

// Validate read operation
let claims = JwtClaims { /* from JWT token */ };
let path = validator.validate_read(
    &claims,
    "satker-kja001/db/password",
    Some("192.168.1.1".to_string())
).await?;

// Validate write operation (includes quota check)
let path = validator.validate_write(
    &claims,
    "satker-kja001/new/secret",
    Some("192.168.1.1".to_string())
).await?;

// Validate delete operation
let path = validator.validate_delete(
    &claims,
    "satker-kja001/old/secret",
    Some("192.168.1.1".to_string())
).await?;

// Validate list operation
let path = validator.validate_list(
    &claims,
    "satker-kja001/db/",
    Some("192.168.1.1".to_string())
).await?;
```

### 3. Access Control

```rust
use secreton_core::namespace::{NamespaceAccessControl, AdminLevel};

let access_control = NamespaceAccessControl::new(hierarchy);

// Check access to specific namespace
let claims = JwtClaims {
    admin_level: AdminLevel::Satker,
    satker_code: Some("kja001".to_string()),
    // ... other fields
};

let has_access = access_control.check_access(&claims, "satker-kja001")?;

// Get all accessible namespaces
let accessible = access_control.get_accessible_namespaces(&claims);

// Validate secret access (convenience method)
let namespace_id = access_control.validate_secret_access(
    &claims,
    "satker-kja001/db/password"
)?;
```

### 4. Filtering Paths

```rust
// Filter paths by accessible namespaces
let all_paths = vec![
    "satker-kja001/db/password".to_string(),
    "satker-kja002/db/password".to_string(),
    "wilayah-sumut/config/key".to_string(),
];

let accessible_paths = validator.filter_accessible_paths(&claims, &all_paths);
// Returns only paths in namespaces the user can access
```

### 5. Creating Hierarchy

```rust
use secreton_core::namespace::NamespaceHierarchy;

// Create hierarchy
let mut hierarchy = NamespaceHierarchy::new(
    "Kejaksaan Agung RI".to_string(),
    "admin".to_string()
);

// Add wilayah
hierarchy.add_wilayah(
    "wilayah-sumut".to_string(),
    "Kejaksaan Tinggi Sumatera Utara".to_string(),
    "admin".to_string(),
)?;

// Add satker
hierarchy.add_satker(
    "satker-kja001".to_string(),
    "Kejaksaan Negeri Medan".to_string(),
    "wilayah-sumut".to_string(),
    "admin".to_string(),
)?;
```

## JWT Claims Structure

```rust
pub struct JwtClaims {
    pub sub: String,                    // User ID
    pub name: String,                   // User name
    pub email: String,                  // User email
    pub satker_code: Option<String>,    // e.g., "KJA001"
    pub wilayah_code: Option<String>,   // e.g., "SUMUT"
    pub admin_level: AdminLevel,        // Access level
    pub roles: Vec<String>,             // User roles
    pub permissions: Vec<String>,       // User permissions
    pub exp: i64,                       // Token expiration
    pub iat: i64,                       // Token issued at
    pub iss: String,                    // Token issuer
    pub metadata: HashMap<String, String>,
}
```

## Audit Logging

All namespace operations are automatically logged with:
- **Action**: Operation performed (read, write, delete, list)
- **Actor**: User identity (from JWT claims)
- **Namespace**: Namespace ID
- **Resource**: Full secret path
- **Status**: Access status (success, denied, failure)
- **IP**: Client IP address
- **Timestamp**: When the operation occurred

Example audit log entry:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-10-24T10:00:00Z",
  "action": "secret.read",
  "actor": "user123",
  "resource_type": "secret",
  "resource_id": "satker-kja001/db/password",
  "status": "Success",
  "ip": "192.168.1.1",
  "namespace": "satker-kja001",
  "metadata": {
    "path": "satker-kja001/db/password"
  }
}
```

## Quota Management

Namespaces have configurable quotas:
- **max_secrets**: Maximum number of secrets (default: 10,000)
- **max_storage_bytes**: Maximum storage size (default: 10 GB)
- **max_leases**: Maximum number of leases (default: 1,000)
- **max_policies**: Maximum number of policies (default: 100)

Write operations automatically check quotas and reject if exceeded:

```rust
// Quota check is automatic in validate_write()
let result = validator.validate_write(&claims, "satker-kja001/new/secret", None).await;

match result {
    Ok(path) => {
        // Quota OK, proceed with write
    }
    Err(CoreError::QuotaExceeded { message }) => {
        // Quota exceeded, reject operation
        return Err(StatusCode::INSUFFICIENT_STORAGE);
    }
    Err(e) => {
        // Other error
    }
}
```

## Security Considerations

1. **Path Traversal Prevention**: Paths containing `..` are rejected
2. **Invalid Characters**: Null bytes, newlines, and other control characters are rejected
3. **Namespace Validation**: All operations validate namespace exists before proceeding
4. **Access Control**: Every operation checks user permissions based on JWT claims
5. **Audit Trail**: All operations are logged for compliance and security monitoring
6. **Quota Enforcement**: Write operations check namespace quotas to prevent abuse

## Error Handling

The module provides detailed error types:

- `PathResolutionError::InvalidFormat` - Path doesn't match expected format
- `PathResolutionError::NamespaceNotFound` - Namespace doesn't exist in hierarchy
- `PathResolutionError::EmptyPath` - Path is empty
- `PathResolutionError::InvalidNamespaceId` - Namespace ID format is invalid
- `PathResolutionError::ValidationFailed` - Path contains invalid characters or patterns
- `CoreError::Authorization` - Access denied to namespace
- `CoreError::QuotaExceeded` - Namespace quota exceeded
- `CoreError::NotFound` - Resource not found
- `CoreError::Validation` - General validation error

## Middleware Integration

The namespace validation is integrated into the API middleware:

```rust
// In middleware.rs
pub async fn namespace_validation_middleware(
    State(state): State<ApiState>,
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Extract JWT claims from request context
    let context = request.extensions().get::<RequestContext>();

    if let Some(ctx) = context {
        if let Some(claims) = &ctx.jwt_claims {
            // Extract path from request
            let path = extract_secret_path_from_request(&request);

            if let Some(secret_path) = path {
                // Validate using NamespaceValidator
                let validator = &state.namespace_validator;
                let operation = request.method().as_str();

                let result = match operation {
                    "GET" => validator.validate_read(claims, &secret_path, ctx.client_ip.clone()).await,
                    "POST" | "PUT" => validator.validate_write(claims, &secret_path, ctx.client_ip.clone()).await,
                    "DELETE" => validator.validate_delete(claims, &secret_path, ctx.client_ip.clone()).await,
                    _ => return Err((StatusCode::METHOD_NOT_ALLOWED, "Method not allowed")),
                };

                if let Err(e) = result {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json(serde_json::json!({
                            "error": e.to_string()
                        })),
                    ));
                }
            }
        }
    }

    Ok(next.run(request).await)
}
```

## Testing

Comprehensive tests are provided in each module:

```bash
# Test path parsing
cargo test --package secreton-core namespace::path

# Test access control
cargo test --package secreton-core namespace::access

# Test validation
cargo test --package secreton-core namespace::validation

# Test all namespace functionality
cargo test --package secreton-core namespace
```

Tests cover:
- Path parsing with valid and invalid formats
- Namespace validation against hierarchy
- Access control for all admin levels
- Secret operation validation (read, write, delete, list)
- Quota enforcement
- Path filtering
- Error handling

## Integration with Authenc

JWT claims are extracted from Authenc tokens in the auth middleware:

1. User authenticates with Authenc
2. Authenc issues JWT with `satker_code`, `wilayah_code`, `admin_level`
3. Secreton validates JWT and extracts claims
4. Claims are added to `RequestContext`
5. Namespace middleware validates access based on claims
6. All operations are audited with namespace information

## Performance Considerations

- **Path Parsing**: O(1) - Simple string split operation
- **Namespace Lookup**: O(1) - HashMap-based lookup
- **Access Check**: O(n) where n is hierarchy depth (max 3 levels)
- **Path Filtering**: O(m) where m is number of paths to filter

For high-performance scenarios, consider:
- Caching parsed paths
- Caching access control decisions (with TTL)
- Batch validation for multiple paths

## Future Enhancements

- [x] Path parsing and validation
- [x] Namespace-scoped secret operations
- [x] Quota enforcement
- [x] Audit logging with namespace information
- [ ] Directorate-specific filtering for EselonI level
- [ ] Namespace delegation (temporary access grants)
- [ ] Dynamic namespace creation via API
- [ ] Metrics for namespace access patterns
- [ ] Path-based policy evaluation
- [ ] Namespace templates for quick setup

