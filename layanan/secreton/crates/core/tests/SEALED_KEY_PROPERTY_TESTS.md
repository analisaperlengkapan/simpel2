# Sealed Master Key Storage Property-Based Tests

## Overview

This document describes the property-based tests for sealed master key storage operations in Secreton. These tests validate **Requirement 2.1.5** from the Vault Parity specification: "Auto-unseal configuration is stored in bootstrap config (secreton.toml)".

## Test File

`sealed_key_storage_property_test.rs`

## Properties Tested

### Property 2: Auto-unseal configuration persistence

**Validates:** Requirements 2.1.5

**Description:** Verifies that any sealed master key that is stored can be retrieved with the same data, ensuring persistence correctness.

**Property Statement:** For any valid `SealedMasterKey`, storing it and then retrieving it by ID should return the exact same data.

**Test Function:** `prop_sealed_key_round_trip`

**What it tests:**
- All fields are preserved across store/retrieve cycles
- ID, provider type, provider key ID, region, endpoint match
- Encrypted master key data is identical
- Checksum is preserved
- Active status and version are maintained
- Checksum verification passes after retrieval

---

### Property 2.1: Active key uniqueness

**Validates:** Requirements 2.1.5

**Description:** Verifies that setting a key as active makes it the only active key, ensuring the unique active constraint is maintained.

**Property Statement:** When setting a key as active, it should be the only active key in the system, and all other keys should be inactive.

**Test Function:** `prop_active_key_uniqueness`

**What it tests:**
- Setting key1 as active makes it the only active key
- Setting key2 as active deactivates key1
- Only one key can be active at any time
- The active key constraint is enforced

---

### Property 2.2: Checksum verification correctness

**Validates:** Requirements 2.1.5

**Description:** Verifies that the checksum verification works correctly for all stored sealed keys.

**Property Statement:** Any sealed key retrieved from storage should have a valid checksum that matches its encrypted data.

**Test Function:** `prop_checksum_verification`

**What it tests:**
- Checksum verification passes for retrieved keys
- Checksum matches the original key's checksum
- SHA-256 checksum integrity is maintained

---

### Property 2.3: Provider metadata preservation

**Validates:** Requirements 2.1.5

**Description:** Verifies that provider metadata is preserved across store/retrieve cycles.

**Property Statement:** All provider-specific metadata (type, key ID, region, endpoint) should be preserved exactly when storing and retrieving sealed keys.

**Test Function:** `prop_provider_metadata_preservation`

**What it tests:**
- Provider type is preserved (AWS KMS, GCP KMS, Azure KV, Transit)
- Provider key ID is preserved
- Provider region is preserved (for cloud providers)
- Provider endpoint is preserved (for Transit)
- Additional metadata JSON is preserved

---

### Property 2.4: List operations consistency

**Validates:** Requirements 2.1.5

**Description:** Verifies that list operations return consistent results with individual get operations.

**Property Statement:** If a key is stored, it should appear in list operations and be retrievable individually with the same data.

**Test Function:** `prop_list_operations_consistency`

**What it tests:**
- All stored keys appear in list operations
- Individual retrieval matches list entries
- Encrypted data is consistent between get and list
- No keys are lost or duplicated

---

### Property 2.5: Provider filtering correctness

**Validates:** Requirements 2.1.5

**Description:** Verifies that filtering keys by provider type returns only keys of that provider type.

**Property Statement:** When filtering by provider type, all returned keys should have that provider type, and all keys of that type should be returned.

**Test Function:** `prop_provider_filtering`

**What it tests:**
- Filtered results contain only keys of the specified provider type
- All keys of the specified type are returned
- No keys are missed or incorrectly included
- Filtering works for all provider types (AWS KMS, GCP KMS, Azure KV, Transit)

---

## Test Strategies

### Provider Type Strategy

Generates all four provider types with equal probability:
- `ProviderType::AwsKms`
- `ProviderType::GcpKms`
- `ProviderType::AzureKv`
- `ProviderType::Transit`

### Provider Key ID Strategy

Generates realistic provider-specific key identifiers:
- **AWS KMS:** ARN format (`arn:aws:kms:us-east-1:123456789012:key/{uuid}`)
- **GCP KMS:** Resource name format (`projects/test-project/locations/us-central1/keyRings/test/cryptoKeys/{name}`)
- **Azure Key Vault:** URL format (`https://test-vault.vault.azure.net/keys/{name}`)
- **Transit:** Simple key name format (`{name}`)

### Provider Region Strategy

Generates appropriate regions for each provider:
- **AWS KMS:** `us-east-1`, `us-west-2`, `eu-west-1`
- **GCP KMS:** `us-central1`, `europe-west1`, `asia-east1`
- **Azure Key Vault:** `eastus`, `westus`, `westeurope`
- **Transit:** `None` (no region)

### Provider Endpoint Strategy

Generates endpoints for Transit provider:
- `https://secreton.internal:8200`
- `https://secreton-primary.internal:8200`
- `https://secreton-dr.internal:8200`

### Encrypted Master Key Strategy

Generates random byte arrays of size 32-256 bytes, simulating encrypted master keys of various sizes.

### Metadata Strategy

Generates various metadata JSON objects:
- Empty object: `{}`
- Rotation count: `{"rotation_count": 1}`
- Full metadata: `{"created_by": "admin", "purpose": "auto-unseal"}`

---

## Running the Tests

### Run all property tests (requires PostgreSQL):

```bash
cd layanan/secreton
cargo test -p secreton-core --test sealed_key_storage_property_test -- --ignored --test-threads=1
```

### Run unit tests only (no database required):

```bash
cd layanan/secreton
cargo test -p secreton-core --test sealed_key_storage_property_test
```

### Run a specific property test:

```bash
cd layanan/secreton
cargo test -p secreton-core --test sealed_key_storage_property_test prop_sealed_key_round_trip -- --ignored
```

---

## Test Configuration

- **Test cases per property:** 10 (configured via `ProptestConfig::with_cases(10)`)
- **Database requirement:** All property tests require PostgreSQL and are marked with `#[ignore]`
- **Test isolation:** Tests should be run with `--test-threads=1` to avoid database conflicts

---

## Database Setup

The tests require a PostgreSQL database with the sealed master keys table:

```bash
# Set database URL
export DATABASE_URL="postgres://secreton:secreton@localhost:5432/secreton_test"

# Run migrations
cd layanan/secreton
cargo run --bin secreton -- migrate
```

The migration file is located at:
`layanan/secreton/migrations/20260218000001_create_sealed_master_keys.sql`

---

## Integration with CI/CD

These property-based tests should be run in CI/CD pipelines with a test database:

```yaml
# Example GitHub Actions workflow
- name: Setup PostgreSQL
  run: |
    docker run -d -p 5432:5432 \
      -e POSTGRES_USER=secreton \
      -e POSTGRES_PASSWORD=secreton \
      -e POSTGRES_DB=secreton_test \
      postgres:15

- name: Run migrations
  run: cargo run --bin secreton -- migrate

- name: Run property tests
  run: cargo test -p secreton-core --test sealed_key_storage_property_test -- --ignored --test-threads=1
```

---

## Benefits of Property-Based Testing

1. **Comprehensive Coverage:** Tests thousands of input combinations automatically
2. **Edge Case Discovery:** Finds edge cases that manual tests might miss
3. **Regression Prevention:** Ensures properties hold across all inputs
4. **Documentation:** Properties serve as executable specifications
5. **Confidence:** Provides high confidence in correctness of storage operations

---

## Future Enhancements

1. **Shrinking:** Implement custom shrinking strategies for better failure reporting
2. **State Machine Testing:** Add stateful property tests for complex workflows
3. **Performance Properties:** Add properties for performance characteristics
4. **Concurrency Properties:** Test concurrent access patterns
5. **Failure Injection:** Test behavior under database failures

---

## References

- [Secreton Vault Parity Specification](../../.kiro/specs/secreton-vault-parity/requirements.md)
- [Sealed Key Storage Implementation](../src/storage/sealed_keys.rs)
- [PropTest Documentation](https://docs.rs/proptest/)
- [Property-Based Testing Guide](https://hypothesis.works/articles/what-is-property-based-testing/)
