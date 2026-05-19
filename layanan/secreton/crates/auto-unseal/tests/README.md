# Auto-Unseal Property Tests

This directory contains property-based tests for auto-unseal providers.

## Overview

Property-based tests verify that auto-unseal providers correctly implement the encrypt/decrypt round-trip property across a wide range of inputs. These tests use [proptest](https://github.com/proptest-rs/proptest) to generate random test cases.

## Test Files

- `aws_kms_property_tests.rs` - AWS KMS provider tests (validates Requirements 2.1.2)
- `gcp_kms_property_tests.rs` - GCP KMS provider tests (validates Requirements 2.1.3)
- `transit_property_tests.rs` - Transit provider tests (validates Requirements 2.1.1)

## Running Tests

### Prerequisites

These tests require real cloud credentials and are marked with `#[ignore]` by default. They are integration tests that interact with actual KMS services.

### AWS KMS Tests

```bash
# Set up AWS credentials
export AWS_KMS_KEY_ID="alias/secreton-test"
export AWS_REGION="us-east-1"

# Optional: Use LocalStack for testing
export AWS_KMS_ENDPOINT="http://localhost:4566"

# Run tests
cargo test --package secreton-auto-unseal \
  --test aws_kms_property_tests \
  --features aws-kms \
  -- --ignored --test-threads=1
```

### GCP KMS Tests

```bash
# Set up GCP credentials
export GCP_KMS_KEY_NAME="projects/my-project/locations/us-central1/keyRings/my-ring/cryptoKeys/my-key"
export GCP_PROJECT_ID="my-project"
export GCP_LOCATION="us-central1"
export GCP_KEY_RING="my-ring"
export GCP_CRYPTO_KEY="my-key"
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account-key.json"

# Run tests
cargo test --package secreton-auto-unseal \
  --test gcp_kms_property_tests \
  --features gcp-kms \
  -- --ignored --test-threads=1
```

### Transit Tests

```bash
# Set up Transit endpoint
export SECRETON_TRANSIT_ENDPOINT="https://secreton.internal:8200"
export SECRETON_TRANSIT_KEY_NAME="autounseal"
export SECRETON_TRANSIT_TOKEN="s.xxxxxxxxxxxxxx"

# Run tests
cargo test --package secreton-auto-unseal \
  --test transit_property_tests \
  --features transit \
  -- --ignored --test-threads=1
```

## Test Configuration

### Proptest Configuration

All property tests use 100 test cases by default:

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn my_property_test(input in strategy) {
        // test implementation
    }
}
```

You can override this with the `PROPTEST_CASES` environment variable:

```bash
PROPTEST_CASES=1000 cargo test --features gcp-kms -- --ignored
```

### Test Parallelism

Use `--test-threads=1` to avoid rate limiting from cloud providers:

```bash
cargo test --features gcp-kms -- --ignored --test-threads=1
```

## Test Structure

Each provider test file contains:

### Property-Based Tests

1. **Round-trip property** - Main correctness property

   ```
   ∀ plaintext: decrypt(encrypt(plaintext)) = plaintext
   ```

2. **Master key size** - Specific test for 32-byte keys

3. **Idempotency** - Multiple encrypt/decrypt cycles

4. **Different plaintexts** - Different inputs produce different outputs

5. **Non-deterministic encryption** - Same input produces different ciphertexts (due to nonce/IV)

### Integration Tests

1. **Master key encryption** - Realistic 32-byte master key scenario
2. **Health check** - Provider connectivity and permissions
3. **Metadata** - Provider metadata verification
4. **Empty plaintext** - Edge case: zero-length input
5. **Large plaintext** - Edge case: 4KB input
6. **Invalid ciphertext** - Error handling
7. **Concurrent operations** - Thread safety

## CI/CD Integration

These tests are NOT run in CI by default because they require cloud credentials. To run them in CI:

1. Set up cloud credentials as secrets
2. Create a dedicated test key in each cloud provider
3. Run tests in a separate CI job:

```yaml
test-gcp-kms:
  runs-on: self-hosted
  steps:
    - uses: actions/checkout@v4
    - name: Set up GCP credentials
      uses: google-github-actions/auth@v1
      with:
        credentials_json: ${{ secrets.GCP_CREDENTIALS }}
    - name: Run GCP KMS tests
      run: |
        cargo test --package secreton-auto-unseal \
          --test gcp_kms_property_tests \
          --features gcp-kms \
          -- --ignored --test-threads=1
      env:
        GCP_KMS_KEY_NAME: ${{ secrets.GCP_KMS_KEY_NAME }}
        GCP_PROJECT_ID: ${{ secrets.GCP_PROJECT_ID }}
        GCP_LOCATION: us-central1
        GCP_KEY_RING: secreton-test
        GCP_CRYPTO_KEY: autounseal-test
```

## Troubleshooting

### Tests are skipped

If tests show as "ignored", you need to pass `--ignored` flag:

```bash
cargo test --features gcp-kms -- --ignored
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

Or reduce the number of test cases:

```bash
PROPTEST_CASES=10 cargo test --features gcp-kms -- --ignored
```

## References

- [Proptest Documentation](https://docs.rs/proptest/)
- [AWS KMS Documentation](https://docs.aws.amazon.com/kms/)
- [GCP KMS Documentation](https://cloud.google.com/kms/docs)
- [Secreton Auto-Unseal Design](../../.kiro/specs/secreton-vault-parity/design.md)

## Azure Key Vault Provider

**Status:** ⚠️ **Requires Azure SDK API Compatibility Fixes**

The Azure Key Vault provider implementation and property tests are complete in structure, but require resolving Azure SDK 0.20 API compatibility issues.

### Known Issue

The `CryptographParamtersEncryption` enum in `azure_security_keyvault` 0.20 doesn't have clear documentation for RSA-OAEP-256 variant names. The implementation currently returns an error indicating this needs to be resolved.

### Resolution Options

1. Check Azure SDK 0.20 source code for correct enum variants
2. Upgrade to a newer Azure SDK version (0.21+) with better documentation
3. Use the Azure REST API directly instead of the SDK

### Test File

`tests/azure_kv_property_tests.rs` - Azure Key Vault provider tests (validates Requirements 2.1.4)

### Property Tests Implemented

All test structures are complete and will work once the Azure SDK issue is resolved:

- ✅ Round-trip encryption/decryption
- ✅ Master key size (32 bytes) round-trip
- ✅ Idempotency across multiple cycles
- ✅ Different plaintexts produce different ciphertexts
- ✅ Non-deterministic encryption (RSA-OAEP padding)

### Integration Tests Implemented

- ✅ Master key encryption scenario
- ✅ Health check verification
- ✅ Metadata validation
- ✅ Invalid ciphertext handling
- ✅ Empty plaintext handling
- ✅ Large plaintext handling (190 bytes for RSA-2048)

### Error Handling Tests

- ✅ Invalid key name
- ✅ Invalid vault name
- ✅ Invalid key name format

### Running Tests (once Azure SDK issue is resolved)

```bash
# Set up Azure credentials
export AZURE_VAULT_NAME="secreton-test"
export AZURE_KEY_NAME="auto-unseal-key"

# Optional: Use service principal authentication
export AZURE_TENANT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_SECRET="your-client-secret"

# Run tests
cargo test --package secreton-auto-unseal \
  --test azure_kv_property_tests \
  --features azure-kv \
  -- --ignored --test-threads=1
```

### Authentication Methods Supported

- **Managed Identity** (recommended for Azure VMs/AKS)
- **Service Principal** (AZURE_TENANT_ID, AZURE_CLIENT_ID, AZURE_CLIENT_SECRET)
- **Azure CLI credentials**

### Required Permissions

The Azure service principal or managed identity needs:

- `Microsoft.KeyVault/vaults/keys/encrypt/action`
- `Microsoft.KeyVault/vaults/keys/decrypt/action`
- `Microsoft.KeyVault/vaults/keys/read`
