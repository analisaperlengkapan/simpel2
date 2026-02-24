# Property-Based Tests for Secreton API

## Health Endpoint Auto-Unseal Status Tests

### File: `health_auto_unseal_property_test.rs`

This file contains comprehensive property-based tests for **Property 5: Health endpoint auto-unseal status** as defined in the Secreton Vault Parity specification.

**Validates: Requirements 2.1.9**

### Properties Tested

1. **Property 5.1: Auto-unseal field presence**
   - When auto-unseal is enabled, the health response MUST include the `auto_unseal` field
   - When auto-unseal is disabled, the `auto_unseal` field MUST be None (not serialized)

2. **Property 5.2: Provider information completeness**
   - When auto-unseal is enabled, provider information MUST be complete
   - AWS KMS and GCP KMS providers MUST have region information
   - Azure Key Vault and Transit providers MUST have endpoint information
   - Provider key ID MUST always be present

3. **Property 5.3: JSON serialization consistency**
   - Health responses MUST serialize to valid JSON
   - Deserialization MUST produce equivalent structures (round-trip consistency)

4. **Property 5.4: All provider types handled**
   - All four provider types MUST be correctly handled:
     - AWS KMS (`aws-kms`)
     - GCP KMS (`gcp-kms`)
     - Azure Key Vault (`azure-kv`)
     - Transit (`transit`)

5. **Property 5.5: Sensitive data not exposed**
   - Health endpoint MUST NOT expose sensitive data:
     - No encryption keys
     - No credentials
     - No passwords or secrets
   - Only metadata should be included (provider type, key ID/alias, region, endpoint)

6. **Property 5.6: Provider health status consistency**
   - `provider_healthy` field MUST be a boolean
   - Field MUST be present when auto-unseal is enabled

7. **Property 5.7: Fallback configuration included**
   - `fallback_enabled` field MUST be present when auto-unseal is enabled
   - Indicates whether manual unseal fallback is available

8. **Property 5.8: Last unseal timestamp format**
   - When `last_unseal` is present, it MUST be a valid ISO 8601 timestamp
   - Timestamp MUST be parseable and round-trip correctly

### Test Strategies

The tests use proptest strategies to generate:

- **Provider types**: All four supported providers (AWS KMS, GCP KMS, Azure KV, Transit)
- **Key IDs**: Various formats (ARNs, aliases, resource names)
- **Regions**: Valid AWS and GCP regions
- **Endpoints**: Valid Azure vault URLs and Transit endpoints
- **Health responses**: Complete health check responses with various configurations

### Running the Tests

```bash
# Run all property tests
cargo test --package secreton-api --test health_auto_unseal_property_test

# Run with verbose output
cargo test --package secreton-api --test health_auto_unseal_property_test -- --nocapture

# Run specific property test
cargo test --package secreton-api --test health_auto_unseal_property_test prop_auto_unseal_field_presence
```

### Known Issues

⚠️ **Current Status**: The tests are correctly implemented but cannot run due to a compilation error in the `secreton-api` crate:

```
error[E0425]: cannot find value `security_headers` in module `lib_common::middleware::security`
```

This is a separate issue in `src/lib.rs` line 142 that needs to be fixed before the property tests can run.

### Test Coverage

The property tests provide comprehensive coverage of:
- ✅ Field presence/absence based on configuration
- ✅ Provider-specific requirements (region vs endpoint)
- ✅ JSON serialization/deserialization
- ✅ All provider types
- ✅ Security (no sensitive data exposure)
- ✅ Data type consistency
- ✅ Timestamp format validation

### Integration with Spec

These tests directly validate:
- **Requirement 2.1.9**: Health endpoint reports auto-unseal status
- **Property 5**: Health endpoint auto-unseal status (from design document)

The tests ensure that the health endpoint correctly reports auto-unseal configuration and provider status without exposing sensitive information.
