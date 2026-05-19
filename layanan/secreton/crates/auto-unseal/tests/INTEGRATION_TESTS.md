# Auto-Unseal Integration Tests

**Task 1.21**: Integration tests for auto-unseal functionality

## Overview

These integration tests validate the complete auto-unseal flow end-to-end, including:

1. **Complete seal → store → restart → auto-unseal flow**
2. **Fallback scenarios** when auto-unseal fails
3. **Configuration persistence** across restarts
4. **Real KMS provider integration** (using test accounts)

## Test Structure

### Mock Provider Tests (Fast, CI-Friendly)

These tests use an in-memory mock KMS provider and don't require external services:

```bash
# Run all mock provider tests
cargo test --package secreton-auto-unseal \
  --test integration_tests

# Run specific test
cargo test --package secreton-auto-unseal \
  --test integration_tests \
  test_complete_auto_unseal_flow
```

**Mock Tests Included:**

1. `test_complete_auto_unseal_flow` - Full seal/unseal cycle
2. `test_fallback_to_manual_unseal` - Fallback when auto-unseal fails
3. `test_auto_unseal_fails_without_fallback` - No fallback configured
4. `test_configuration_persistence` - Config survives restarts
5. `test_retry_with_exponential_backoff` - Retry mechanism
6. `test_successful_unseal_after_retries` - Success after failures
7. `test_checksum_verification` - Data integrity checks
8. `test_multiple_providers_same_master_key` - Multiple provider support

### Real KMS Provider Tests (Requires Credentials)

These tests interact with actual KMS services and are marked with `#[ignore]`:

```bash
# Transit provider
cargo test --package secreton-auto-unseal \
  --test integration_tests \
  --features transit \
  test_transit_provider_integration \
  -- --ignored

# AWS KMS provider
cargo test --package secreton-auto-unseal \
  --test integration_tests \
  --features aws-kms \
  test_aws_kms_provider_integration \
  -- --ignored

# GCP KMS provider
cargo test --package secreton-auto-unseal \
  --test integration_tests \
  --features gcp-kms \
  test_gcp_kms_provider_integration \
  -- --ignored
```

## Test Scenarios

### Scenario 1: Complete Auto-Unseal Flow

**Test:** `test_complete_auto_unseal_flow`

**Steps:**

1. Generate a 32-byte master key
2. Create auto-unseal provider
3. Encrypt master key with provider
4. Store encrypted key in sealed key storage
5. Mark key as active
6. Simulate restart by retrieving key from storage
7. Auto-unseal using AutoUnsealManager
8. Verify decrypted key matches original

**Validates:**

- Requirements 2.1.1, 2.1.2, 2.1.3, 2.1.4 (provider functionality)
- Requirement 2.1.5 (configuration storage)

### Scenario 2: Fallback to Manual Unseal

**Test:** `test_fallback_to_manual_unseal`

**Steps:**

1. Create provider configured to fail
2. Encrypt master key (while provider is working)
3. Configure fallback to manual unseal
4. Attempt auto-unseal (provider now fails)
5. Verify fallback occurs and returns empty key

**Validates:**

- Requirement 2.1.6 (fallback to manual unseal)

### Scenario 3: No Fallback Configured

**Test:** `test_auto_unseal_fails_without_fallback`

**Steps:**

1. Create provider configured to fail
2. Configure NO fallback
3. Attempt auto-unseal
4. Verify operation fails with error

**Validates:**

- Requirement 2.1.6 (fallback configuration)

### Scenario 4: Configuration Persistence

**Test:** `test_configuration_persistence`

**Steps:**

1. Store multiple sealed keys with different providers
2. Set one key as active
3. Simulate restart by retrieving configuration
4. Verify active key is correctly identified
5. List all keys and verify count
6. Query keys by provider type

**Validates:**

- Requirement 2.1.5 (configuration persistence)

### Scenario 5: Retry with Exponential Backoff

**Test:** `test_retry_with_exponential_backoff`

**Steps:**

1. Create provider that always fails
2. Configure retry with exponential backoff
3. Attempt auto-unseal
4. Verify multiple retry attempts occur
5. Verify fallback after max retries

**Validates:**

- Requirement 2.1.6 (retry logic)

### Scenario 6: Successful Unseal After Retries

**Test:** `test_successful_unseal_after_retries`

**Steps:**

1. Create provider that fails initially
2. Configure retry mechanism
3. Enable provider after 2 attempts
4. Verify auto-unseal eventually succeeds

**Validates:**

- Requirement 2.1.6 (retry success)

### Scenario 7: Checksum Verification

**Test:** `test_checksum_verification`

**Steps:**

1. Create sealed key with encrypted data
2. Verify checksum is valid
3. Corrupt encrypted data
4. Verify checksum fails

**Validates:**

- Data integrity (implicit in 2.1.5)

### Scenario 8: Multiple Providers

**Test:** `test_multiple_providers_same_master_key`

**Steps:**

1. Encrypt same master key with 3 different providers
2. Store all encrypted keys
3. Verify each provider can decrypt its own key
4. Verify keys are correctly categorized by provider type

**Validates:**

- Requirements 2.1.1, 2.1.2, 2.1.3, 2.1.4 (all providers)

## Real KMS Provider Integration

### Transit Provider

**Environment Variables:**

```bash
export SECRETON_TRANSIT_ENDPOINT="https://secreton.internal:8200"
export SECRETON_TRANSIT_KEY_NAME="autounseal"
export SECRETON_TRANSIT_TOKEN="s.xxxxxxxxxxxxxx"
```

**Run Test:**

```bash
cargo test --package secreton-auto-unseal \
  --test integration_tests \
  --features transit \
  test_transit_provider_integration \
  -- --ignored
```

### AWS KMS Provider

**Environment Variables:**

```bash
export AWS_KMS_KEY_ID="alias/secreton-test"
export AWS_REGION="us-east-1"

# Optional: Use LocalStack for testing
export AWS_KMS_ENDPOINT="http://localhost:4566"

# Optional: Explicit credentials
export AWS_ACCESS_KEY_ID="your-access-key"
export AWS_SECRET_ACCESS_KEY="your-secret-key"
```

**Run Test:**

```bash
cargo test --package secreton-auto-unseal \
  --test integration_tests \
  --features aws-kms \
  test_aws_kms_provider_integration \
  -- --ignored
```

### GCP KMS Provider

**Environment Variables:**

```bash
export GCP_PROJECT_ID="my-project"
export GCP_LOCATION="us-central1"
export GCP_KEY_RING="secreton-test"
export GCP_CRYPTO_KEY="autounseal-test"
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account-key.json"
```

**Run Test:**

```bash
cargo test --package secreton-auto-unseal \
  --test integration_tests \
  --features gcp-kms \
  test_gcp_kms_provider_integration \
  -- --ignored
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Auto-Unseal Integration Tests

on: [push, pull_request]

jobs:
  mock-tests:
    runs-on: self-hosted
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Run mock provider tests
        run: |
          cargo test --package secreton-auto-unseal \
            --test integration_tests

  aws-kms-tests:
    runs-on: self-hosted
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: us-east-1

      - name: Run AWS KMS tests
        run: |
          cargo test --package secreton-auto-unseal \
            --test integration_tests \
            --features aws-kms \
            test_aws_kms_provider_integration \
            -- --ignored
        env:
          AWS_KMS_KEY_ID: ${{ secrets.AWS_KMS_KEY_ID }}

  gcp-kms-tests:
    runs-on: self-hosted
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Authenticate to Google Cloud
        uses: google-github-actions/auth@v1
        with:
          credentials_json: ${{ secrets.GCP_CREDENTIALS }}

      - name: Run GCP KMS tests
        run: |
          cargo test --package secreton-auto-unseal \
            --test integration_tests \
            --features gcp-kms \
            test_gcp_kms_provider_integration \
            -- --ignored
        env:
          GCP_PROJECT_ID: ${{ secrets.GCP_PROJECT_ID }}
          GCP_LOCATION: us-central1
          GCP_KEY_RING: secreton-test
          GCP_CRYPTO_KEY: autounseal-test
```

## Local Testing with LocalStack

For local AWS KMS testing without real AWS credentials:

```bash
# Start LocalStack
docker run -d -p 4566:4566 localstack/localstack

# Create test key
aws --endpoint-url=http://localhost:4566 kms create-key \
  --description "Secreton auto-unseal test key"

# Create alias
aws --endpoint-url=http://localhost:4566 kms create-alias \
  --alias-name alias/secreton-test \
  --target-key-id <key-id-from-above>

# Run tests
export AWS_KMS_ENDPOINT="http://localhost:4566"
export AWS_KMS_KEY_ID="alias/secreton-test"
export AWS_REGION="us-east-1"

cargo test --package secreton-auto-unseal \
  --test integration_tests \
  --features aws-kms \
  test_aws_kms_provider_integration \
  -- --ignored
```

## Troubleshooting

### Tests are skipped

If tests show as "ignored", you need to pass `--ignored` flag:

```bash
cargo test --features aws-kms -- --ignored
```

### Authentication errors

- **AWS**: Verify `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` are set
- **GCP**: Verify `GOOGLE_APPLICATION_CREDENTIALS` points to a valid service account key
- **Transit**: Verify `SECRETON_TRANSIT_TOKEN` is valid and not expired

### Permission errors

Ensure the credentials have the following permissions:

- **AWS KMS**: `kms:Encrypt`, `kms:Decrypt`, `kms:DescribeKey`
- **GCP KMS**: `cloudkms.cryptoKeyVersions.useToEncrypt`, `cloudkms.cryptoKeyVersions.useToDecrypt`, `cloudkms.cryptoKeys.get`
- **Transit**: `read` and `update` capabilities on `transit/encrypt/<key>` and `transit/decrypt/<key>`

### Rate limiting

If you encounter rate limiting errors, reduce test parallelism:

```bash
cargo test --features gcp-kms -- --ignored --test-threads=1
```

## Test Coverage

These integration tests provide comprehensive coverage of:

- ✅ Complete seal/unseal lifecycle
- ✅ Fallback mechanisms (enabled and disabled)
- ✅ Configuration persistence across restarts
- ✅ Retry logic with exponential backoff
- ✅ Checksum verification for data integrity
- ✅ Multiple provider support
- ✅ Real KMS provider integration (AWS, GCP, Transit)
- ✅ Error handling and edge cases

## References

- [Auto-Unseal Design Document](../.kiro/specs/secreton-vault-parity/design.md)
- [Auto-Unseal Requirements](../.kiro/specs/secreton-vault-parity/requirements.md)
- [Property-Based Tests](./README.md)
- [Fallback Documentation](../FALLBACK.md)
