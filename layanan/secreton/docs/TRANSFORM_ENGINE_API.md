# Transform Engine API Reference

## Base URL

```
http://localhost:8200/v1/transform
```

## Authentication

All endpoints require valid authentication token in the `X-Secret Vault-Token` header.

## Endpoints

### Transformation Management

#### Create Transformation

Create a new transformation configuration.

**Endpoint**: `POST /transformation`

**Request Body**:
```json
{
  "name": "string (required)",
  "transformation_type": "fpe|tokenization|masking (required)",
  "template": "string (optional, for masking)",
  "alphabet": "numeric|alphanumeric|alphanumeric_mixed|custom (optional, for FPE)",
  "tweak_source": "string (optional, for FPE)",
  "masking_char": "char (optional, default: '*')",
  "masking_pattern": "default|credit_card|email|phone|custom (optional)"
}
```

**Example - Tokenization**:
```bash
curl -X POST http://localhost:8200/v1/transform/transformation \
  -H "Content-Type: application/json" \
  -H "X-Secret Vault-Token: $TOKEN" \
  -d '{
    "name": "ssn-protection",
    "transformation_type": "tokenization"
  }'
```

**Example - Credit Card Masking**:
```bash
curl -X POST http://localhost:8200/v1/transform/transformation \
  -H "Content-Type: application/json" \
  -H "X-Secret Vault-Token: $TOKEN" \
  -d '{
    "name": "card-masking",
    "transformation_type": "masking",
    "masking_pattern": "credit_card",
    "masking_char": "*"
  }'
```

**Response**: `200 OK`
```json
{
  "success": true,
  "data": {
    "name": "ssn-protection",
    "transformation_type": "tokenization",
    "template": null,
    "alphabet": null,
    "tweak_source": null,
    "masking_char": "*",
    "masking_pattern": null,
    "created_at": "2024-01-15T10:00:00Z"
  }
}
```

#### Get Transformation

Retrieve transformation configuration.

**Endpoint**: `GET /transformation/:name`

**Example**:
```bash
curl -X GET http://localhost:8200/v1/transform/transformation/ssn-protection \
  -H "X-Secret Vault-Token: $TOKEN"
```

**Response**: `200 OK`
```json
{
  "success": true,
  "data": {
    "name": "ssn-protection",
    "transformation_type": "tokenization",
    "created_at": "2024-01-15T10:00:00Z"
  }
}
```

#### List Transformations

List all transformation names.

**Endpoint**: `GET /transformation`

**Example**:
```bash
curl -X GET http://localhost:8200/v1/transform/transformation \
  -H "X-Secret Vault-Token: $TOKEN"
```

**Response**: `200 OK`
```json
{
  "success": true,
  "data": ["ssn-protection", "card-masking", "email-mask"]
}
```

#### Delete Transformation

Delete a transformation configuration.

**Endpoint**: `DELETE /transformation/:name`

**Example**:
```bash
curl -X DELETE http://localhost:8200/v1/transform/transformation/old-transform \
  -H "X-Secret Vault-Token: $TOKEN"
```

**Response**: `200 OK`
```json
{
  "success": true,
  "data": null
}
```

### Role Management

#### Create Role

Create a role with access to specific transformations.

**Endpoint**: `POST /role`

**Request Body**:
```json
{
  "name": "string (required)",
  "transformations": ["string"] (required, array of transformation names)
}
```

**Example**:
```bash
curl -X POST http://localhost:8200/v1/transform/role \
  -H "Content-Type: application/json" \
  -H "X-Secret Vault-Token: $TOKEN" \
  -d '{
    "name": "application-role",
    "transformations": ["ssn-protection", "card-masking"]
  }'
```

**Response**: `200 OK`
```json
{
  "success": true,
  "data": {
    "name": "application-role",
    "transformations": ["ssn-protection", "card-masking"],
    "created_at": "2024-01-15T10:00:00Z"
  }
}
```

#### Get Role

Retrieve role configuration.

**Endpoint**: `GET /role/:name`

**Example**:
```bash
curl -X GET http://localhost:8200/v1/transform/role/application-role \
  -H "X-Secret Vault-Token: $TOKEN"
```

**Response**: `200 OK`
```json
{
  "success": true,
  "data": {
    "name": "application-role",
    "transformations": ["ssn-protection", "card-masking"],
    "created_at": "2024-01-15T10:00:00Z"
  }
}
```

#### List Roles

List all role names.

**Endpoint**: `GET /role`

**Example**:
```bash
curl -X GET http://localhost:8200/v1/transform/role \
  -H "X-Secret Vault-Token: $TOKEN"
```

**Response**: `200 OK`
```json
{
  "success": true,
  "data": ["application-role", "admin-role", "read-only-role"]
}
```

### Encode/Decode Operations

#### Encode Value

Transform a single value (tokenize, encrypt, or mask).

**Endpoint**: `POST /encode/:role/:transformation`

**Request Body**:
```json
{
  "value": "string (required)",
  "tweak": "string (optional, for FPE)"
}
```

**Example - Tokenization**:
```bash
curl -X POST http://localhost:8200/v1/transform/encode/application-role/ssn-protection \
  -H "Content-Type: application/json" \
  -H "X-Secret Vault-Token: $TOKEN" \
  -d '{
    "value": "123-45-6789"
  }'
```

**Response**: `200 OK`
```json
{
  "success": true,
  "data": {
    "result": "tok_a1b2c3d4e5f6g7h8i9j0"
  }
}
```

**Example - Masking**:
```bash
curl -X POST http://localhost:8200/v1/transform/encode/application-role/card-masking \
  -H "Content-Type: application/json" \
  -H "X-Secret Vault-Token: $TOKEN" \
  -d '{
    "value": "4111111111111111"
  }'
```

**Response**: `200 OK`
```json
{
  "success": true,
  "data": {
    "result": "****-****-****-1111"
  }
}
```

#### Decode Value

Reverse transformation (only for tokenization and FPE, not masking).

**Endpoint**: `POST /decode/:role/:transformation`

**Request Body**:
```json
{
  "value": "string (required)",
  "tweak": "string (optional, for FPE)"
}
```

**Example**:
```bash
curl -X POST http://localhost:8200/v1/transform/decode/application-role/ssn-protection \
  -H "Content-Type: application/json" \
  -H "X-Secret Vault-Token: $TOKEN" \
  -d '{
    "value": "tok_a1b2c3d4e5f6g7h8i9j0"
  }'
```

**Response**: `200 OK`
```json
{
  "success": true,
  "data": {
    "result": "123-45-6789"
  }
}
```

### Batch Operations

#### Batch Encode

Transform multiple values in a single request.

**Endpoint**: `POST /batch/encode/:role/:transformation`

**Request Body**:
```json
{
  "values": ["string"] (required, array of values),
  "tweak": "string (optional, for FPE)"
}
```

**Example**:
```bash
curl -X POST http://localhost:8200/v1/transform/batch/encode/application-role/ssn-protection \
  -H "Content-Type: application/json" \
  -H "X-Secret Vault-Token: $TOKEN" \
  -d '{
    "values": ["123-45-6789", "987-65-4321", "555-12-3456"]
  }'
```

**Response**: `200 OK`
```json
{
  "success": true,
  "data": {
    "results": [
      "tok_a1b2c3d4e5f6g7h8i9j0",
      "tok_z9y8x7w6v5u4t3s2r1q0",
      "tok_m1n2o3p4q5r6s7t8u9v0"
    ]
  }
}
```

#### Batch Decode

Reverse transformation for multiple values.

**Endpoint**: `POST /batch/decode/:role/:transformation`

**Request Body**:
```json
{
  "values": ["string"] (required, array of tokens),
  "tweak": "string (optional, for FPE)"
}
```

**Example**:
```bash
curl -X POST http://localhost:8200/v1/transform/batch/decode/application-role/ssn-protection \
  -H "Content-Type: application/json" \
  -H "X-Secret Vault-Token: $TOKEN" \
  -d '{
    "values": [
      "tok_a1b2c3d4e5f6g7h8i9j0",
      "tok_z9y8x7w6v5u4t3s2r1q0"
    ]
  }'
```

**Response**: `200 OK`
```json
{
  "success": true,
  "data": {
    "results": ["123-45-6789", "987-65-4321"]
  }
}
```

### Audit

#### Get Audit Statistics

Retrieve tokenization usage statistics without exposing original values.

**Endpoint**: `GET /audit`

**Example**:
```bash
curl -X GET http://localhost:8200/v1/transform/audit \
  -H "X-Secret Vault-Token: $TOKEN"
```

**Response**: `200 OK`
```json
{
  "success": true,
  "data": {
    "total_tokens": 1500,
    "transformations": [
      {
        "transformation_name": "ssn-protection",
        "token_count": 1000,
        "total_encode_operations": 1200,
        "total_decode_operations": 800,
        "oldest_token": "2024-01-01T00:00:00Z",
        "newest_token": "2024-01-15T10:00:00Z"
      },
      {
        "transformation_name": "card-masking",
        "token_count": 500,
        "total_encode_operations": 500,
        "total_decode_operations": 0,
        "oldest_token": "2024-01-05T00:00:00Z",
        "newest_token": "2024-01-15T09:00:00Z"
      }
    ],
    "generated_at": "2024-01-15T10:30:00Z"
  }
}
```

## Error Responses

### 400 Bad Request

Invalid request parameters or transformation configuration.

```json
{
  "success": false,
  "error": {
    "code": "BAD_REQUEST",
    "message": "Failed to create transformation: Transformation already exists: ssn-protection"
  }
}
```

### 404 Not Found

Transformation or role not found.

```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Transformation not found"
  }
}
```

### 403 Forbidden

Role does not have access to the transformation.

```json
{
  "success": false,
  "error": {
    "code": "FORBIDDEN",
    "message": "Access denied: role application-role cannot use transformation admin-transform"
  }
}
```

### 500 Internal Server Error

Server-side error (e.g., FPE library issues).

```json
{
  "success": false,
  "error": {
    "code": "INTERNAL_ERROR",
    "message": "Encode failed: Encryption failed: The given numeral string is invalid for radix 10"
  }
}
```

## Masking Patterns Reference

### Credit Card Pattern

**Configuration**:
```json
{
  "masking_pattern": "credit_card",
  "masking_char": "*"
}
```

**Behavior**:
- Validates 13-19 digit card numbers
- Shows last 4 digits
- Formats with dashes for 16-digit cards

**Examples**:
- Input: `4111111111111111` → Output: `****-****-****-1111`
- Input: `378282246310005` → Output: `***********0005`

### Email Pattern

**Configuration**:
```json
{
  "masking_pattern": "email",
  "masking_char": "*"
}
```

**Behavior**:
- Shows first character of local part
- Shows complete domain
- Masks remaining local part characters

**Examples**:
- Input: `john.doe@example.com` → Output: `j*******@example.com`
- Input: `admin@company.org` → Output: `a****@company.org`

### Phone Pattern

**Configuration**:
```json
{
  "masking_pattern": "phone",
  "masking_char": "*"
}
```

**Behavior**:
- Validates minimum 7 digits
- Shows last 4 digits
- Formats with dashes for 10-digit US numbers

**Examples**:
- Input: `5551234567` → Output: `***-***-4567`
- Input: `12345678901` → Output: `*******8901`

### Custom Pattern

**Configuration**:
```json
{
  "masking_pattern": "custom",
  "template": "###-**-####",
  "masking_char"
}
```

**Template Syntax**:
- `#` = Show character from input
- `*` = Mask character
- Other characters = Literal (e.g., `-`)

**Examples**:
- Template: `###-**-####`, Input: `123456789` → Output: `123-**-6789`
- Template: `****-####`, Input: `12345678` → Output: `****-5678`

## Best Practices

1. **Use Tokenization for Production** ✅
   - Most reliable and tested
   - No input format restrictions

2. **Avoid FPE Until Library Fixed** ⚠️
   - Known issues with certain patterns
   - Use tokenization instead

3. **Use Batch Operations**
   - Better performance for multiple values
   - Single role access check

4. **Implement Proper Access Control**
   - Create separate roles for different access levels
   - Limit transformation access per role

5. **Monitor with Audit Statistics**
   - Track usage patterns
   - Identify potential issues

6. **Handle Errors Gracefully**
   - Check for 403 (access denied)
   - Handle 500 (FPE failures) with fallback

## Rate Limiting

Currently no rate limiting is enforced. Future versions may implement per-client rate limits.

## Versioning

API version is included in the URL path: `/v1/transform`

Breaking changes will result in a new version: `/v2/transform`

