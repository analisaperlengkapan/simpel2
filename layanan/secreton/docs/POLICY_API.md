# Policy Management API Documentation

## Overview

The Policy Management API provides comprehensive endpoints for creating, managing, and testing access control policies in Secreton. Policies define fine-grained access control rules with support for wildcards, conditions, MFA requirements, and multi-approval workflows.

## Authentication

All policy management endpoints require admin-level authentication. Include a valid JWT token in the Authorization header:

```
Authorization: Bearer <jwt_token>
```

The JWT token must have `admin_level: Pusat` or `admin_level: EselonI` to access policy management endpoints.

## REST API Endpoints

### List Policies

List all policies with optional filtering and pagination.

**Endpoint:** `GET /v1/sys/policies`

**Query Parameters:**
- `namespace` (optional): Filter by namespace (default: all namespaces)
- `is_active` (optional): Filter by active status (true/false)
- `search` (optional): Search by name or description
- `limit` (optional): Number of results per page (default: 20, max: 100)
- `offset` (optional): Offset for pagination (default: 0)

**Example Request:**
```bash
curl -X GET "https://secreton.example.com/v1/sys/policies?namespace=default&is_active=true&limit=10" \
  -H "Authorization: Bearer <token>"
```

**Example Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "name": "read-secrets",
      "namespace": "default",
      "description": "Allow reading secrets",
      "rules": [
        {
          "effect": "allow",
          "action": "read",
          "path": "secret/*",
          "condition": null,
          "control_group": null,
          "mfa": null
        }
      ],
      "version": 1,
      "is_active": true,
      "created_at": "2025-01-01T00:00:00Z",
      "updated_at": "2025-01-01T00:00:00Z",
      "created_by": "admin",
      "updated_by": null,
      "stats": {
        "evaluations_total": 1000,
        "evaluations_allowed": 950,
        "evaluations_denied": 50,
        "cache_hits": 800,
        "cache_misses": 200,
        "last_evaluated_at": "2025-01-01T12:00:00Z"
      }
    }
  ],
  "total": 1,
  "limit": 10,
  "offset": 0
}
```

### Create Policy

Create a new policy with validation.

**Endpoint:** `POST /v1/sys/policies/{name}`

**Path Parameters:**
- `name`: Policy name (alphanumeric with hyphens or underscores)

**Request Body:**
```json
{
  "description": "Policy description",
  "rules": [
    {
      "effect": "allow",
      "action": "read",
      "path": "secret/data/*",
      "condition": null,
      "control_group": null,
      "mfa": null
    }
  ],
  "namespace": "default"
}
```

**Example Request:**
```bash
curl -X POST "https://secreton.example.com/v1/sys/policies/my-policy" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Allow reading all secrets",
    "rules": [
      {
        "effect": "allow",
        "action": "read",
        "path": "secret/*"
      }
    ],
    "namespace": "default"
  }'
```

**Example Response:**
```json
{
  "success": true,
  "data": {
    "id":
ame": "my-policy",
    "namespace": "default",
    "description": "Allow reading all secrets",
    "rules": [...],
    "version": 1,
    "is_active": true,
    "created_at": "2025-01-01T00:00:00Z",
    "updated_at": "2025-01-01T00:00:00Z",
    "created_by": "admin",
    "updated_by": null,
    "stats": {
      "evaluations_total": 0,
      "evaluations_allowed": 0,
      "evaluations_denied": 0,
      "cache_hits": 0,
      "cache_misses": 0,
      "last_evaluated_at": null
    }
  },
  "message": "Policy created successfully"
}
```

### Get Policy

Get a policy by name with evaluation statistics.

**Endpoint:** `GET /v1/sys/policies/{name}`

**Path Parameters:**
- `name`: Policy name

**Example Request:**
```bash
curl -X GET "https://secreton.example.com/v1/sys/policies/my-policy" \
  -H "Authorization: Bearer <token>"
```

**Example Response:**
```json
{
  "success": true,
  "data": {
    "id": 2,
    "name": "my-policy",
    "namespace": "default",
    "description": "Allow reading all secrets",
    "rules": [...],
    "version": 1,
    "is_active": true,
    "created_at": "2025-01-01T00:00:00Z",
    "updated_at": "2025-01-01T00:00:00Z",
    "created_by": "admin",
    "updated_by": null,
    "stats": {...}
  }
}
```

### Update Policy

Update an existing policy with version control.

**Endpoint:** `PUT /v1/sys/policies/{name}`

**Path Parameters:**
- `name`: Policy name

**Request Body:**
```json
{
  "description": "Updated description",
  "rules": [
    {
      "effect": "allow",
      "action": "read",
      "path": "secret/data/*"
    },
    {
      "effect": "deny",
      "action": "delete",
      "path": "secret/data/protected/*"
    }
  ],
  "is_active": true
}
```

**Example Request:**
```bash
curl -X PUT "https://secreton.example.com/v1/sys/policies/my-policy" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Updated policy",
    "rules": [...]
  }'
```

**Example Response:**
```json
{
  "success": true,
  "data": {
    "id": 2,
    "name": "my-policy",
    "namespace": "default",
    "description": "Updated policy",
    "rules": [...],
    "version": 2,
    "is_active": true,
    "created_at": "2025-01-01T00:00:00Z",
    "updated_at": "2025-01-01T01:00:00Z",
    "created_by": "admin",
    "updated_by": "admin",
    "stats": {...}
  },
  "message": "Policy updated successfully"
}
```

### Delete Policy

Delete a policy with dependency checking.

**Endpoint:** `DELETE /v1/sys/policies/{name}`

**Path Parameters:**
- `name`: Policy name

**Example Request:**
```bash
curl -X DELETE "https://secreton.example.com/v1/sys/policies/my-policy" \
  -H "Authorization: Bearer <token>"
```

**Example Response:**
```json
{
  "success": true,
  "data": null,
  "message": "Policy deleted successfully"
}
```

**Error Response (if dependencies exist):**
```json
{
  "success": false,
  "error": {
    "code": "INVALID_INPUT",
    "message": "Cannot delete policy 'my-policy' because it is referenced by: other-policy-1, other-policy-2"
  }
}
```

### Test Policy

Test policy evaluation without applying it.

**Endpoint:** `POST /v1/sys/policies/{name}/test`

**Path Parameters:**
- `name`: Policy name

**Request Body:**
```json
{
  "user": "user123",
  "path": "secret/data/database/password",
  "action": "read",
  "context": {
    "client_ip": "192.168.1.100",
    "mfa_passed": true,
    "user_level": 5
  }
}
```

**Example Request:**
```bash
curl -X POST "https://secreton.example.com/v1/sys/policies/my-policy/test" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "user": "user123",
    "path": "secret/data/database/password",
    "action": "read",
    "context": {
      "mfa_passed": true
    }
  }'
```

**Example Response:**
```json
{
  "success": true,
  "data": {
    "allowed": true,
    "matched_rules": [
      "Rule 1: allow read on secret/data/*"
    ],
    "evaluation_time_ms": 0.523
  },
  "message": "Policy evaluation: ALLOWED"
}
```

## Policy Rule Syntax

### Basic Structure

```json
{
  "effect": "allow" | "deny",
  "action": "read" | "create" | "update" | "delete" | "list" | "sudo" | "*",
  "path": "secret/path/pattern",
  "condition": { ... },
  "control_group": { ... },
  "mfa": true | false
}
```

### Path Patterns

- **Exact match:** `secret/data/foo` - matches only `secret/data/foo`
- **Single wildcard:** `secret/data/*` - matches `secret/data/foo` but not `secret/data/foo/bar`
- **Double wildcard:** `secret/data/**` - matches `secret/data/foo` and `secret/data/foo/bar`
- **Glob pattern:** `secret/*/password` - matches `secret/db/password` and `secret/api/password`

### Capabilities

- `read` - Read operations
- `create` - Create operations
- `update` - Update operations
- `delete` - Delete operations
- `list` - List operations
- `sudo` - Administrative operations
- `*` - All operations

### Conditions

#### Time-based Conditions

```json
{
  "condition": {
    "time_range": {
      "start": "2025-01-01T00:00:00Z",
      "end": "2025-12-31T23:59:59Z"
    }
  }
}
```

#### IP-based Conditions

```json
{
  "condition": {
    "allowed_ips": ["192.168.1.0/24", "10.0.0.1"]
  }
}
```

#### Expression Conditions

```json
{
  "condition": {
    "expression": {
      "field": "user_level",
      "op": ">=",
      "value": 5
    }
  }
}
```

**Supported operators:**
- `==`, `eq` - Equality
- `!=`, `ne` - Inequality
- `>`, `gt` - Greater than
- `>=`, `gte` - Greater than or equal
- `<`, `lt` - Less than
- `<=`, `lte` - Less than or equal
- `contains` - Array/string contains
- `in` - Value in array
- `matches` - Glob pattern matching

### Control Groups (Multi-Approval)

```json
{
  "control_group": {
    "required_approvals": 2,
    "approved_by": ["admin1", "admin2"]
  }
}
```

### MFA Requirement

```json
{
  "mfa": true
}
```

## Policy Precedence

1. **Deny overrides allow** - If any rule denies access, access is denied
2. **Explicit rules** - More specific paths take precedence
3. **Default deny** - If no rule matches, access is denied

## Examples

### Example 1: Basic Read Policy

```json
{
  "name": "read-only",
  "description": "Allow reading all secrets",
  "rules": [
    {
      "effect": "allow",
      "action": "read",
      "path": "secret/**"
    }
  ],
  "namespace": "default"
}
```

### Example 2: Restricted Delete Policy

```json
{
  "name": "restricted-delete",
  "description": "Allow delete except protected paths",
  "rules": [
    {
      "effect": "allow",
      "action": "delete",
      "path": "secret/**"
    },
    {
      "effect": "deny",
      "action": "delete",
      "path": "secret/protected/**"
    }
  ],
  "namespace": "default"
}
```

### Example 3: MFA-Required Policy

```json
{
  "name": "mfa-required",
  "description": "Require MFA for critical operations",
  "rules": [
    {
      "effect": "allow",
      "action": "delete",
      "path": "secret/critical/**",
      "mfa": true
    }
  ],
  "namespace": "default"
}
```

### Example 4: Time-Based Policy

```json
{
  "name": "business-hours",
  "description": "Allow access only during business hours",
  "rules": [
    {
      "effect": "allow",
      "action": "*",
      "path": "secret/**",
      "condition": {
        "time_range": {
          "start": "2025-01-01T09:00:00Z",
          "end": "2025-12-31T17:00:00Z"
        }
      }
    }
  ],
  "namespace": "default"
}
```

### Example 5: IP-Restricted Policy

```json
{
  "name": "ip-restricted",
  "description": "Allow access only from specific IPs",
  "rules": [
    {
      "effect": "allow",
      "action": "*",
      "path": "secret/**",
      "condition": {
        "allowed_ips": ["192.168.1.0/24", "10.0.0.0/8"]
      }
    }
  ],
  "namespace": "default"
}
```

### Example 6: Multi-Approval Policy

```json
{
  "name": "multi-approval",
  "description": "Require multiple approvals for critical operations",
  "rules": [
    {
      "effect": "allow",
      "action": "delete",
      "path": "secret/critical/**",
      "control_group": {
        "required_approvals": 2,
        "approved_by": []
      }
    }
  ],
  "namespace": "default"
}
```

### Example 7: Complex Conditional Policy

```json
{
  "name": "complex-policy",
  "description": "Complex policy with multiple conditions",
  "rules": [
    {
      "effect": "allow",
      "action": "delete",
      "path": "secret/critical/**",
      "condition": {
        "time_range": {
          "start": "2025-01-01T00:00:00Z",
          "end": "2099-01-01T00:00:00Z"
        },
        "allowed_ips": ["192.168.1.0/24"],
        "expression": {
          "field": "user_level",
          "op": ">=",
          "value": 8
        }
      },
      "mfa": true
    }
  ],
  "namespace": "default"
}
```

## Error Codes

- `INVALID_INPUT` - Invalid request parameters or policy syntax
- `NOT_FOUND` - Policy not found
- `ALREADY_EXISTS` - Policy with the same name already exists
- `PERMISSION_DENIED` - Insufficient permissions to perform operation
- `INTERNAL_ERROR` - Internal server error

## Best Practices

1. **Use descriptive names** - Policy names should clearly indicate their purpose
2. **Start with deny** - Use deny rules for sensitive paths, then allow specific access
3. **Test policies** - Always test policies before applying them to production
4. **Version control** - Keep track of policy versions and changes
5. **Minimal permissions** - Grant only the minimum required permissions
6. **Use namespaces** - Organize policies by namespace for better management
7. **Document policies** - Add clear descriptions to explain policy intent
8. **Regular audits** - Review policy evaluation statistics regularly
9. **MFA for critical operations** - Require MFA for sensitive operations
10. **IP restrictions** - Use IP-based conditions for additional security

## gRPC API

The same functionality is available via gRPC. See `infra/proto/secreton.proto` for message definitions.

Example gRPC call:
```go
client := secreton.NewSecretonServiceClient(conn)
resp, err := client.ListPolicies(ctx, &secreton.ListPoliciesRequest{
    Namespace: "default",
    IsActive: true,
    Limit: 10,
})
```

## Monitoring

Policy operations emit the following metrics:

- `secreton_policy_evaluations_total{policy_name, decision}` - Total policy evaluations
- `secreton_policy_evaluation_duration_seconds{policy_name}` - Policy evaluation duration
- `secreton_policy_cache_hits_total{policy_name}` - Policy cache hits
- `secreton_policy_cache_misses_total{policy_name}` - Policy cache misses
- `secreton_policy_operations_total{operation}` - Policy CRUD operations

All policy operations are logged to the audit log with the following fields:
- `operation` - create_policy, update_policy, delete_policy, test_policy
- `policy_name` - Name of the policy
- `user` - User who performed the operation
- `timestamp` - Operation timestamp
- `result` - success or failure

## Integration with Namespace Isolation

Policies are scoped to namespaces. Users can only manage policies in their own namespace or child namespaces based on their admin level:

- **Pusat** - Can manage policies in all namespaces
- **Eselon I** - Can manage policies in directorate namespaces
- **Wilayah** - Can manage policies in wilayah and child satker namespaces
- **Satker** - Can only view policies in own satker namespace

## Caching

Policy evaluations are cached for 60 seconds to improve performance. The cache is automatically invalidated when:
- Policy is updated
- Policy is deleted
- Policy is deactivated

Cache statistics are available in the policy stats response.
