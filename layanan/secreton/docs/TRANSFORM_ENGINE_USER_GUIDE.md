# Transform Engine User Guide

## Overview

The Transform Engine provides three types of data transformation for protecting sensitive information:

1. **Tokenization** (Recommended for Production)
2. **Format-Preserving Encryption (FPE)**
3. **Data Masking**

## Tokenization (Recommended) ✅

**Use Case**: General-purpose secret protection with reversible transformation.

**Advantages**:

- ✅ Reliable and stable
- ✅ Works with any input data
- ✅ Consistent mapping (same input → same token)
- ✅ Fully reversible with proper authorization
- ✅ No input format restrictions

**Example**:

```bash
# Create tokenization transformation
curl -X POST http://localhost:8200/v1/transform/transformation \
  -H "Content-Type: application/json" \
  -d '{
    "name": "ssn-tokenization",
    "transformation_type": "tokenization"
  }'

# Create role
curl -X POST http://localhost:8200/v1/transform/role \
  -H "Content-Type: application/json" \
  -d '{
    "name": "app-role",
    "transformations": ["ssn-tokenization"]
  }'

# Tokenize
curl -X POST http://localhost:8200/v1/transform/encode/app-role/ssn-tokenization
Content-Type: application/json" \
  -d '{"value": "123-45-6789"}'

# Response: {"result": "tok_a1b2c3d4e5f6..."}

# Detokenize
curl -X POST http://localhost:8200/v1/transform/decode/app-role/ssn-tokenization \
  -H "Content-Type: application/json" \
  -d '{"value": "tok_a1b2c3d4e5f6..."}'

# Response: {"result": "123-45-6789"}
```

**Batch Operations**:

```bash
# Batch tokenize
curl -X POST http://localhost:8200/v1/transform/batch/encode/app-role/ssn-tokenization \
  -H "Content-Type: application/json" \
  -d '{
    "values": ["123-45-6789", "987-65-4321", "555-12-3456"]
  }'

# Response: {"results": ["tok_...", "tok_...", "tok_..."]}
```

## Format-Preserving Encryption (FPE) ⚠️

**Use Case**: When you need encrypted data to maintain the same format as the original (e.g., for legacy systems).

**Known Limitations**:

- ⚠️ The underlying FF1 library has issues with certain input patterns
- ⚠️ May fail with inputs like "000000", "10000", or repeated characters
- ⚠️ Not recommended for production use until library issues are resolved

**Supported Alphabets**:

- `numeric`: 0-9 (radix 10)
- `alphanumeric`: a-z, 0-9 (radix 36)
- `alphanumeric_mixed`: A-Z, a-z, 0-9 (radix 62)
- `custom`: Your own character set

**Example** (use with caution):

```bash
# Create FPE transformation
curl -X POST http://localhost:8200/v1/transform/transformation \
  -H "Content-Type: application/json" \
  -d '{
    "name": "card-fpe",
    "transformation_type": "fpe",
    "alphabet": "numeric"
  }'

# Encode (may fail with certain patterns)
curl -X POST http://localhost:8200/v1/transform/encode/app-role/card-fpe \
  -H "Content-Type: application/json" \
  -d '{"value": "4111111111111111"}'
```

**Recommendation**: Use tokenization instead until FPE library issues are resolved.

## Data Masking ✅

**Use Case**: One-way transformation for display purposes (irreversible).

**Masking Patterns**:

### 1. Credit Card Masking

```bash
curl -X POST http://localhost:8200/v1/transform/transformation \
  -H "Content-Type: application/json" \
  -d '{
    "name": "card-mask",
    "transformation_type": "masking",
    "masking_pattern": "credit_card",
    "masking_char": "*"
  }'

# Input:  4111111111111111
# Output: ****-****-****-1111
```

### 2. Email Masking

```bash
curl -X POST http://localhost:8200/v1/transform/transformation \
  -H "Content-Type: application/json" \
  -d '{
    "name": "email-mask",
    "transformation_type": "masking",
    "masking_pattern": "email",
    "masking_char": "*"
  }'

# Input:  john.doe@example.com
# Output: j*******@example.com
```

### 3. Phone Masking

```bash
curl -X POST http://localhost:8200/v1/transform/transformation \
  -H "Content-Type: application/json" \
  -d '{
    "name": "phone-mask",
    "transformation_type": "masking",
    "masking_pattern": "phone",
    "masking_char": "*"
  }'

# Input:  5551234567
# Output: ***-***-4567
```

### 4. Custom Template Masking

```bash
curl -X POST http://localhost:8200/v1/transform/transformation \
  -H "Content-Type: application/json" \
  -d '{
    "name": "custom-mask",
    "transformation_type": "masking",
    "masking_pattern": "custom",
    "template": "###-**-####",
    "masking_char": "*"
  }'

# Template: # = show character, * = mask character
# Input:  123456789
# Output: 123-**-6789
```

## Audit Statistics

Track tokenization usage without exposing original values:

```bash
curl -X GET http://localhost:8200/v1/transform/audit

# Response:
{
  "total_tokens": 150,
  "transformations": [
    {
      "transformation_name": "ssn-tokenization",
      "token_count": 100,
      "total_encode_operations": 120,
      "total_decode_operations": 80,
      "oldest_token": "2024-01-01T00:00:00Z",
      "newest_token": "2024-01-15T12:30:00Z"
    }
  ],
  "generated_at": "2024-01-15T14:00:00Z"
}
```

## Best Practices

### 1. Use Tokenization for Production ✅

- Most reliable and stable
- No input format restrictions
- Fully tested and production-ready

### 2. Use Masking for Display ✅

- Perfect for showing partial data to users
- Irreversible (cannot recover original)
- Fast and efficient

### 3. Avoid FPE Until Library Fixed ⚠️

- Known issues with certain input patterns
- May cause unexpected failures
- Wait for library updates or use tokenization

### 4. Implement Proper Access Control

```bash
# Create separate roles for different access levels
curl -X POST http://localhost:8200/v1/transform/role \
  -H "Content-Type: application/json" \
  -d '{
    "name": "read-only-role",
    "transformations": ["ssn-tokenization"]
  }'

curl -X POST http://localhost:8200/v1/transform/role \
  -H "Content-Type: application/json" \
  -d '{
    "name": "admin-role",
    "transformations": ["ssn-tokenization", "card-mask"]
  }'
```

### 5. Use Batch Operations for Performance

```bash
# Process multiple values in one request
curl -X POST http://localhost:8200/v1/transform/batch/encode/app-role/ssn-tokenization \
  -H "Content-Type: application/json" \
  -d '{
    "values": ["value1", "value2", "value3", "..."]
  }'
```

### 6. Monitor Usage with Audit Statistics

```bash
# Regular monitoring
curl -X GET http://localhost:8200/v1/transform/audit | jq '.transformations[] | {name, token_count, operations: (.total_encode_operations + .total_decode_operations)}'
```

## Error Handling

### Common Errors

**1. Transformation Not Found**

```json
{
  "error": "Transformation not found: my-transform"
}
```

Solution: Create the transformation first.

**2. Role Not Found**

```json
{
  "error": "Role not found: my-role"
}
```

Solution: Create the role and grant access to transformations.

**3. Access Denied**

```json
{
  "error": "Access denied: role my-role cannot use transformation other-transform"
}
```

Solution: Add the transformation to the role's allowed list.

**4. FPE Encryption Failed** ⚠️

```json
{
  "error": "Encode failed: Encryption failed: The given numeral string is invalid for radix 10"
}
```

Solution: Use tokenization instead of FPE, or avoid problematic input patterns.

**5. Masking is Irreversible**

```json
{
  "error": "Decode failed: Masking is irreversible"
}
```

Solution: Masking cannot be reversed. Use tokenization or FPE if you need reversibility.

## Migration from FPE to Tokenization

If you're currently using FPE and experiencing issues:

```bash
# 1. Create new tokenization transformation
curl -X POST http://localhost:8200/v1/transform/transformation \
  -H "Content-Type: application/json" \
  -d '{
    "name": "new-tokenization",
    "transformation_type": "tokenization"
  }'

# 2. Update role to include new transformation
curl -X POST http://localhost:8200/v1/transform/role \
  -H "Content-Type: application/json" \
  -d '{
    "name": "app-role",
    "transformations": ["old-fpe", "new-tokenization"]
  }'

# 3. Gradually migrate data
# - Decode with old FPE transformation
# - Encode with new tokenization transformation
# - Update application to use new transformation

# 4. Remove old FPE transformation once migration complete
```

## Performance Considerations

- **Tokenization**: O(1) lookup, very fast
- **FPE**: Cryptographic operations, slower but format-preserving
- **Masking**: String manipulation, very fast

**Batch Operations**: Use batch endpoints for processing multiple values to reduce network overhead and improve throughput.

## Security Notes

1. **Tokenization tokens are deterministic**: Same input always produces same token
2. **Masking is one-way**: Cannot recover original data
3. **FPE is deterministic**: Same input + key + tweak produces same ciphertext
4. **Access control**: Always use roles to restrict transformation access
5. **Audit logging**: All operations are logged for compliance

## Support

For issues or questions:

- Check the troubleshooting section above
- Review audit logs for operation history
- Consult the API documentation for detailed endpoint specifications
- For FPE issues, consider using tokenization as a workaround
