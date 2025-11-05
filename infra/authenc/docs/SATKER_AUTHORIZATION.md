# Satker-Aware Authorization

This document describes the Satker (Satuan Kerja / Organizational Unit) hierarchy and authorization system implemented in Authenc.

## Overview

The Satker authorization system provides hierarchical organization-aware access control for the Attorney General's Office (Kejaksaan RI). It enables:

- **Hierarchical Organization Structure**: Model the organizational hierarchy from Pusat (central) to Kejaksaan Negeri (district offices)
- **Permission Inheritance**: Parent satkers can access child satker resources
- **Cross-Satker Operations**: Validate operations that span multiple organizational units
- **Satker-Scoped Admin Roles**: Assign administrative privileges at specific organizational levels

## Architecture

### Satker Hierarchy

```
PUSAT (Kejaksaan Agung RI)
├── KT-DKI (Kejaksaan Tinggi DKI Jakarta)
│   ├── KN-JAKPUS (Kejaksaan Negeri Jakarta Pusat)
│   └── KN-JAKSEL (Kejaksaan Negeri Jakarta Selatan)
├── KT-JABAR (Kejaksaan Tinggi Jawa Barat)
│   └── KN-BANDUNG (Kejaksaan Negeri Bandung)
└── ...
```

### Components

1. **Satker Model** (`models/satker.rs`):
   - `Satker`: Organizational unit entity
   - `SatkerHierarchy`: Efficient hierarchy traversal
   - `SatkerPermissionScope`: Permission scope definition
   - `CrossSatkerValidation`: Cross-satker operation validation

2. **Authorization Service** (`services/satker_authorization.rs`):
   - `SatkerAuthorizationService`: Main authorization logic
   - Hierarchy-aware access control
   - Permission caching for pe

3. **Database Operations** (`database/satker_operations.rs`):
   - CRUD operations for satkers
   - Hierarchy queries
   - Efficient indexing

4. **API Handlers** (`handlers/satker.rs`):
   - REST endpoints for satker management
   - Authorization check endpoints
   - Hierarchy query endpoints

## Database Schema

### Tables

#### `satkers`
Stores organizational units with hierarchical relationships.

```sql
CREATE TABLE satkers (
    id UUID PRIMARY KEY,
    code VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    parent_code VARCHAR(50) REFERENCES satkers(code),
    level INTEGER NOT NULL,
    satker_type JSONB NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    attributes JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);
```

#### `satker_permissions`
Explicit satker-level permissions for users.

```sql
CREATE TABLE satker_permissions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    satker_code VARCHAR(50) NOT NULL REFERENCES satkers(code),
    permission_type VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    action VARCHAR(50),
    include_children BOOLEAN NOT NULL DEFAULT false,
    include_parents BOOLEAN NOT NULL DEFAULT false,
    granted_by UUID REFERENCES users(id),
    granted_at TIMESTAMP WITH TIME ZONE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE,
    attributes JSONB
);
```

#### `satker_admin_roles`
Satker-scoped administrative role assignments.

```sql
CREATE TABLE satker_admin_roles (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    satker_code VARCHAR(50) NOT NULL REFERENCES satkers(code),
    admin_level VARCHAR(50) NOT NULL, -- 'AdminSatker', 'AdminWilayah', 'AdminEselonI', 'AdminPusat'
    scope_data JSONB,
    assigned_by UUID REFERENCES users(id),
    assigned_at TIMESTAMP WITH TIME ZONE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE,
    active BOOLEAN NOT NULL DEFAULT true
);
```

## API Endpoints

### Satker Management

#### List Satkers
```http
GET /api/v1/satkers?parent_code=PUSAT&level=1&search=Jakarta
```

#### Get Satker
```http
GET /api/v1/satkers/{code}
```

#### Create Satker
```http
POST /api/v1/satkers
Content-Type: application/json

{
  "code": "KN-EXAMPLE",
  "name": "Kejaksaan Negeri Example",
  "description": "Example district office",
  "parent_code": "KT-DKI",
  "level": 2,
  "satker_type": "KejaksaanNegeri"
}
```

#### Update Satker
```http
PUT /api/v1/satkers/{code}
Content-Type: application/json

{
  "name": "Updated Name",
  "description": "Updated description"
}
```

### Hierarchy Queries

#### Get Satker Hierarchy
```http
GET /api/v1/satkers/{code}/hierarchy
```

Response:
```json
{
  "satker": { "code": "KN-JAKPUS", "name": "...", ... },
  "parent": { "code": "KT-DKI", "name": "...", ... },
  "children": [],
  "ancestors": [
    { "code": "KT-DKI", "name": "...", ... },
    { "code": "PUSAT", "name": "...", ... }
  ],
  "descendants": [],
  "level": 2
}
```

#### Get Root Satkers
```http
GET /api/v1/satkers/roots
```

### Authorization Checks

#### Check Satker Access
```http
POST /api/v1/satkers/check-access
Content-Type: application/json

{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "target_satker_code": "KN-JAKPUS"
}
```

Response:
```json
{
  "allowed": true,
  "reason": "Access granted based on user roles and hierarchy"
}
```

#### Validate Cross-Satker Operation
```http
POST /api/v1/satkers/validate-cross-operation
Content-Type: application/json

{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "source_satker": "KN-JAKPUS",
  "target_satker": "KN-JAKSEL",
  "operation": "read",
  "resource_type": "document"
}
```

Response:
```json
{
  "allowed": true,
  "reason": "User has permission to perform 'read' on 'document' across satkers",
  "missing_permissions": [],
  "involved_satkers": ["KN-JAKPUS", "KN-JAKSEL"]
}
```

#### Get Accessible Satkers for User
```http
GET /api/v1/satkers/users/{user_id}/accessible
```

Response:
```json
["KN-JAKPUS", "KN-JAKSEL", "KT-DKI", "PUSAT"]
```

#### Check Management Permission
```http
GET /api/v1/satkers/users/{user_id}/can-manage/{satker_code}
```

Response:
```json
{
  "allowed": true,
  "reason": "User has management permissions for this satker"
}
```

## Usage Examples

### Initialize Satker Authorization Service

```rust
use authenc::services::satker_authorization::SatkerAuthorizationService;
use authenc::database::Database;

// Load all satkers from database
let satkers = db.get_all_satkers().await?;

// Create authorization service
let satker_auth = Arc::new(SatkerAuthorizationService::new(satkers));
```

### Check User Access to Satker

```rust
// Get user from database
let user = db.get_user_by_id(&user_id).await?;

// Check if user can access target satker
let can_access = satker_auth
    .can_access_satker(&user, "KN-JAKPUS")
    .await?;

if can_access {
    // Allow operation
} else {
    // Deny operation
}
```

### Validate Cross-Satker Operation

```rust
let validation = satker_auth
    .validate_cross_satker_operation(
        &user,
        "KN-JAKPUS",  // source satker
        "KN-JAKSEL",  // target satker
        "transfer",   // operation
        "document"    // resource type
    )
    .await?;

if validation.allowed {
    // Proceed with operation
} else {
    // Return error with missing permissions
    return Err(AuthencError::forbidden(validation.reason));
}
```

### Get Accessible Satkers

```rust
let accessible = satker_auth
    .get_accessible_satkers(&user)
    .await?;

// Filter resources by accessible satkers
let resources = db
    .get_resources()
    .await?
    .into_iter()
    .filter(|r| accessible.contains(&r.satker_code))
    .collect();
```

## Authorization Rules

### Hierarchy-Based Access

1. **Own Satker**: Users can always access their own satker
2. **Parent Access**: Parent satkers can access child satker resources
3. **Pusat Access**: Pusat (central) level can access all satkers
4. **Sibling Restriction**: Sibling satkers cannot access each other's resources (unless explicitly granted)

### Admin Levels

1. **AdminPusat**: Full access to all satkers
2. **AdminEselonI**: Access to all satkers (similar to Pusat)
3. **AdminWilayah**: Access to satkers within the wilayah (region)
4. **AdminSatker**: Access to specific satker and its children

### Role Scopes

1. **RoleScope::Pusat**: Access to all satkers
2. **RoleScope::Wilayah(code)**: Access to satkers in the wilayah
3. **RoleScope::Satker(code)**: Access to specific satker and descendants

## Performance Considerations

### Caching

The authorization service implements caching for:
- Authorization decisions (keyed by user_id, satker codes, operation, resource type)
- Hierarchy traversal results

Cache is automatically invalidated when:
- Satker hierarchy is updated
- User permissions change
- Explicit cache clear is requested

### Database Indexes

Efficient indexes are created for:
- Satker code lookups
- Parent-child relationships
- Hierarchy traversal
- User satker assignments

### Batch Operations

For bulk authorization checks, use batch operations:

```rust
// Get all accessible satkers once
let accessible = satker_auth.get_accessible_satkers(&user).await?;

// Filter resources in memory
let filtered_resources: Vec<_> = resources
    .into_iter()
    .filter(|r| accessible.contains(&r.satker_code))
    .collect();
```

## Security Considerations

1. **Least Privilege**: Users should only have access to satkers they need
2. **Explicit Grants**: Cross-satker access should be explicitly granted
3. **Audit Logging**: All satker operations are logged in `satker_audit_logs`
4. **Time-Based Restrictions**: Permissions can have expiration dates
5. **Admin Oversight**: Admin role assignments are tracked with assigned_by

## Migration Guide

### Adding Satker Awareness to Existing Code

1. **Update User Model**: Ensure users have `satker_code` field
2. **Add Satker Checks**: Wrap resource access with satker authorization
3. **Update Queries**: Filter queries by accessible satkers
4. **Add Audit Logs**: Log satker-related operations

Example:

```rust
// Before
let documents = db.get_all_documents().await?;

// After
let accessible_satkers = satker_auth.get_accessible_satkers(&user).await?;
let documents = db
    .get_all_documents()
    .await?
    .into_iter()
    .filter(|d| accessible_satkers.contains(&d.satker_code))
    .collect();
```

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_parent_can_access_child() {
    let satkers = vec![
        create_test_satker("PARENT", None, 0),
        create_test_satker("CHILD", Some("PARENT"), 1),
    ];
    let service = SatkerAuthorizationService::new(satkers);

    let role = Role {
        scope: RoleScope::Satker("PARENT".to_string()),
        // ...
    };
    let user = create_test_user("PARENT", vec![role]);

    let can_access = service.can_access_satker(&user, "CHILD").await.unwrap();
    assert!(can_access);
}
```

### Integration Tests

Test with real database:

```rust
#[tokio::test]
async fn test_cross_satker_validation() {
    let db = Database::new(&test_config()).await.unwrap();

    // Create test satkers
    db.create_satker("TEST1", "Test 1", None, None, 0, SatkerType::Pusat, None).await.unwrap();
    db.create_satker("TEST2", "Test 2", None, Some("TEST1"), 1, SatkerType::KejaksaanTinggi, None).await.unwrap();

    // Create authorization service
    let satkers = db.get_all_satkers().await.unwrap();
    let service = SatkerAuthorizationService::new(satkers);

    // Test validation
    // ...
}
```

## Troubleshooting

### Common Issues

1. **Access Denied**: Check user's satker_code and role scopes
2. **Hierarchy Not Loading**: Verify satker parent_code references are correct
3. **Performance Issues**: Check cache hit rate and consider warming cache
4. **Inconsistent Results**: Clear cache after hierarchy changes

### Debug Logging

Enable debug logging for satker authorization:

```rust
tracing::debug!(
    user_id = %user.id,
    user_satker = %user.satker_code,
    target_satker = %target_satker_code,
    "Checking satker access"
);
```

## Future Enhancements

1. **Dynamic Hierarchy**: Support runtime hierarchy changes without restart
2. **Delegation**: Allow users to delegate access to specific satkers
3. **Temporary Access**: Time-limited cross-satker access grants
4. **Access Requests**: Workflow for requesting cross-satker access
5. **Analytics**: Dashboard for satker access patterns and usage

