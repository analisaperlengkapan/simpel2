# HSM Setup Guide for Secreton

This guide provides comprehensive instructions for setting up Hardware Security Mle (HSM) integration with Secreton for enhanced key security.

## Overview

Secreton supports HSM integration for hardware-backed cryptographic operations, providing:
- Hardware-protected key storage
- FIPS 140-2 Level 3+ compliance (hardware dependent)
- Tamper-resistant key operations
- Enhanced security for master keys and encryption keys

## Supported HSM Providers

Secreton supports multiple HSM providers:

1. **PKCS#11** - Standard interface for hardware HSMs (Thales, Gemalto, SafeNet, etc.)
2. **AWS KMS** - Amazon Web Services Key Management Service (planned)
3. **Azure Key Vault** - Microsoft Azure Key Vault (planned)
4. **GCP KMS** - Google Cloud Platform Key Management Service (planned)

## Prerequisites

### For PKCS#11 HSM

1. **Hardware HSM** or **SoftHSM** for testing
2. **PKCS#11 library** installed on the system
3. **HSM initialized** with a token and PIN
4. **Network connectivity** to HSM (for network HSMs)

### For Cloud HSMs

1. **Cloud account** with appropriate permissions
2. **API credentials** configured
3. **Network connectivity** to cloud provider

## Installation

### Option 1: Hardware HSM (Production)

#### 1. Install HSM Drivers

For Thales/SafeNet HSM:
```bash
# Install HSM client software
sudo dpkg -i safenet-client-*.deb

# Verify installation
/opt/safenet/lunaclient/bin/vtl verify
```

For Gemalto HSM:
```bash
# Install Gemalto client
sudo rpm -i gemalto-client-*.rpm

# Configure HSM connection
/opt/gemalto/bin/configure
```

#### 2. Initialize HSM Token

```bash
# Initialize token (if not already done)
pkcs11-tool --module /usr/lib/libCryptoki2_64.so --init-token --label "secreton-hsm" --so-pin <SO_PIN>

# Initialize user PIN
pkcs11-tool --module /usr/lib/libCryptoki2_64.so --init-pin --token-label "secreton-hsm" --so-pin <SO_PIN> --pin <USER_PIN>
```

#### 3. Verify HSM Connectivity

```bash
# List available slots
pkcs11-tool --module /usr/lib/libCryptoki2_64.so --list-slots

# Test token access
pkcs11-tool --module /usr/lib/libCryptoki2_64.so --list-objects --token-label "secreton-hsm" --pin <USER_PIN>
```

### Option 2: SoftHSM (Development/Testing)

SoftHSM provides a software-based PKCS#11 implementation for testing.

#### 1. Install SoftHSM

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install softhsm2
```

**RHEL/CentOS:**
```bash
sudo yum install softhsm
```

**macOS:**
```bash
brew install softhsm
```

#### 2. Initialize SoftHSM

```bash
# Create token directory
mkdir -p /var/lib/softhsm/tokens

# Initialize token
softhsm2-util --init-token --slot 0 --label "secreton-hsm" --so-pin 1234 --pin 5678

# Verify token
softhsm2-util --show-slots
```

#### 3. Find PKCS#11 Library Path

```bash
# Ubuntu/Debian
ls /usr/lib/softhsm/libsofthsm2.so

# RHEL/CentOS
ls /usr/lib64/pkcs11/libsofthsm2.so

# macOS
ls /usr/local/lib/softhsm/libsofthsm2.so
```

## Configuration

### 1. Update Secreton Configuration

Edit `config/secreton.production.toml`:

```toml
[hsm]
# Enable HSM integration
enabled = true

# HSM provider type
provider = "pkcs11"  # Options: pkcs11, aws-kms, azure-keyvault, gcp-kms

# PKCS#11 configuration
pkcs11_library_path = "/usr/lib/softhsm/libsofthsm2.so"  # Adjust for your system
slot_id = 0
token_label = "secreton-hsm"
pin = "${HSM_PIN}"  # Use environment variable for security
key_label_prefix = "secreton-"

# Timeouts
connection_timeout = 30  # seconds
operation_timeout = 60   # seconds

# Health monitoring
health_check_enabled = true
health_check_interval = 60  # seconds

# Retry configuration
max_retries = 3
retry_delay_ms = 1000
```

### 2. Set Environment Variables

**For production (use secrets management):**
```bash
export HSM_PIN="your-secure-pin"
export SECRETON_HSM_ENABLED=true
```

**For development (SoftHSM):**
```bash
export HSM_PIN="5678"
export SECRETON_HSM_ENABLED=true
```

### 3. Verify Configuration

```bash
# Test configuration loading
secreton config validate

# Check HSM connectivity
secreton hsm status
```

## Usage

### Starting Secreton with HSM

```bash
# Start Secreton server
secreton server --config /etc/secreton/config.toml

# Check logs for HSM initialization
tail -f /var/log/secreton/secreton.log | grep HSM
```

Expected log output:
```
INFO Initializing HSM backend with provider: Pkcs11
INFO HSM backend initialized successfully
INFO ✅ HSM backend initialized successfully
```

### Generating Keys in HSM

```bash
# Generate RSA key in HSM
curl -X POST http://localhost:8200/v1/hsm/keys/my-rsa-key \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "algorithm": "RSA",
    "key_size": 2048,
    "usage": ["sign", "verify"]
  }'

# Generate AES key in HSM
curl -X POST http://localhost:8200/v1/hsm/keys/my-aes-key \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "algorithm": "AES",
    "key_size": 256,
    "usage": ["encrypt", "decrypt"]
  }'
```

### Using HSM Keys

```bash
# Sign data with HSM key
curl -X POST http://localhost:8200/v1/hsm/sign/my-rsa-key \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "data": "SGVsbG8gV29ybGQ=",
    "algorithm": "RSA-SHA256"
  }'

# Encrypt data with HSM key
curl -X POST http://localhost:8200/v1/hsm/encrypt/my-aes-key \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "plaintext": "SGVsbG8gV29ybGQ="
  }'
```

### Monitoring HSM Health

```bash
# Check HSM health via API
curl http://localhost:8200/health/detailed

# Expected response includes HSM status
{
  "status": "healthy",
  "checks": {
    "hsm": {
      "status": "healthy",
      "message": "HSM is connected and operational",
      "response_time_ms": 15,
      "details": {
        "connected": true,
        "provider": "pkcs11"
      }
    }
  }
}
```

## Security Best Practices

### 1. PIN Management

- **Never hardcode PINs** in configuration files
- Use **environment variables** or **secrets management** (Vault, AWS Secrets Manager)
- Rotate PINs regularly according to security policy
- Use different PINs for different environments

### 2. Access Control

- Limit HSM access to Secreton service account only
- Use **mTLS** for network HSMs
- Enable **audit logging** for all HSM operations
- Implement **role-based access control** (RBAC)

### 3. Key Management

- Generate keys directly in HSM (never import)
- Use **non-exportable keys** for maximum security
- Implement **key rotation** policies
- Maintain **key inventory** and lifecycle management

### 4. Network Security

- Use **dedicated network** for HSM communication
- Enable **firewall rules** to restrict HSM access
- Use **VPN** or **private network** for cloud HSMs
- Monitor **network traffic** for anomalies

### 5. Backup and Recovery

- Backup HSM configuration (not keys)
- Document **disaster recovery** procedures
- Test **failover scenarios** regularly
- Maintain **offline backup** of critical keys (if policy allows)

## Troubleshooting

### HSM Not Detected

**Symptom:** `HSM not initialized` error

**Solutions:**
1. Verify PKCS#11 library path:
   ```bash
   ls -l /usr/lib/softhsm/libsofthsm2.so
   ```

2. Check token initialization:
   ```bash
   softhsm2-util --show-slots
   ```

3. Verify PIN is correct:
   ```bash
   pkcs11-tool --module /usr/lib/softhsm/libsofthsm2.so --login --pin 5678
   ```

### Connection Timeout

**Symptom:** `HSM connection failed: timeout`

**Solutions:**
1. Increase connection timeout in config
2. Check network connectivity to HSM
3. Verify HSM is not overloaded
4. Check firewall rules

### Authentication Failed

**Symptom:** `HSM authentication failed: invalid PIN`

**Solutions:**
1. Verify PIN environment variable is set correctly
2. Check token is not locked (too many failed attempts)
3. Reinitialize token if necessary:
   ```bash
   softhsm2-util --init-token --slot 0 --label "secreton-hsm" --so-pin 1234 --pin 5678
   ```

### Key Generation Failed

**Symptom:** `HSM operation failed: key generation error`

**Solutions:**
1. Check HSM has sufficient storage
2. Verify key parameters are supported
3. Check HSM permissions
4. Review HSM logs for details

### Performance Issues

**Symptom:** Slow HSM operations

**Solutions:**
1. Enable connection pooling
2. Increase operation timeout
3. Use batch operations where possible
4. Consider HSM hardware upgrade
5. Check network latency (for network HSMs)

## Monitoring and Alerting

### Metrics to Monitor

1. **HSM Availability**
   - Connection status
   - Health check success rate
   - Response time

2. **Operation Metrics**
   - Key generation rate
   - Signature operations per second
   - Encryption/decryption throughput
   - Error rate

3. **Resource Utilization**
   - HSM CPU usage
   - Memory usage
   - Storage capacity
   - Network bandwidth

### Prometheus Metrics

Secreton exposes HSM metrics at `/metrics`:

```
# HSM health status (1 = healthy, 0 = unhealthy)
secreton_hsm_health_status 1

# HSM operation duration in seconds
secreton_hsm_operation_duration_seconds{operation="sign"} 0.015

# HSM operation total count
secreton_hsm_operations_total{operation="sign",status="success"} 1234

# HSM connection errors
secreton_hsm_connection_errors_total 0
```

### Alerting Rules

Example Prometheus alerting rules:

```yaml
groups:
  - name: hsm_alerts
    rules:
      - alert: HSMUnhealthy
        expr: secreton_hsm_health_status == 0
        for: 5m
        annotations:
          summary: "HSM is unhealthy"
          description: "HSM has been unhealthy for 5 minutes"

      - alert: HSMHighErrorRate
        expr: rate(secreton_hsm_operations_total{status="error"}[5m]) > 0.1
        for: 5m
        annotations:
          summary: "High HSM error rate"
          description: "HSM error rate is above 10%"

      - alert: HSMSlowOperations
        expr: secreton_hsm_operation_duration_seconds{quantile="0.99"} > 1.0
        for: 10m
        annotations:
          summary: "HSM operations are slow"
          description: "99th percentile HSM operation time is above 1 second"
```

## Compliance and Auditing

### FIPS 140-2 Compliance

For FIPS 140-2 compliance:

1. Use FIPS-validated HSM hardware
2. Enable FIPS mode in configuration:
   ```toml
   [hsm]
   fips_mode = true
   ```
3. Use only FIPS-approved algorithms
4. Maintain audit trail of all operations

### Audit Logging

All HSM operations are logged:

```json
{
  "timestamp": "2025-10-29T10:00:00Z",
  "operation": "hsm_key_generate",
  "key_id": "my-rsa-key",
  "algorithm": "RSA",
  "key_size": 2048,
  "user": "admin@kejaksaan.go.id",
  "status": "success",
  "duration_ms": 150
}
```

### Compliance Reports

Generate compliance reports:

```bash
# Generate HSM usage report
secreton hsm report --start-date 2025-10-01 --end-date 2025-10-31

# Export audit logs
secreton audit export --filter "operation=hsm_*" --format json > hsm_audit.json
```

## Migration from Software Keys

### Planning

1. Identify keys to migrate
2. Plan migration schedule
3. Test migration in staging
4. Prepare rollback plan

### Migration Steps

1. **Generate new keys in HSM:**
   ```bash
   secreton hsm generate-key --name master-key-v2 --algorithm RSA --size 4096
   ```

2. **Re-encrypt data with new keys:**
   ```bash
   secreton migrate reencrypt --old-key master-key-v1 --new-key master-key-v2
   ```

3. **Update key references:**
   ```bash
   secreton config update-key-reference --from master-key-v1 --to master-key-v2
   ```

4. **Verify migration:**
   ```bash
   secreton verify-encryption --key master-key-v2
   ```

5. **Retire old keys:**
   ```bash
   secreton key retire --name master-key-v1
   ```

## Support

For HSM-related issues:

1. Check Secreton logs: `/var/log/secreton/secreton.log`
2. Review HSM vendor documentation
3. Contact HSM vendor support
4. Open issue on Secreton GitHub repository

## References

- [PKCS#11 Specification](http://docs.oasis-open.org/pkcs11/pkcs11-base/v2.40/os/pkcs11-base-v2.40-os.html)
- [FIPS 140-2 Standard](https://csrc.nist.gov/publications/detail/fips/140/2/final)
- [SoftHSM Documentation](https://www.opendnssec.org/softhsm/)
- [Secreton Documentation](https://github.com/kejaksaan/secreton/docs)

## Changelog

- **2025-10-29**: Initial HSM setup guide created
- Added PKCS#11 support documentation
- Added SoftHSM testing instructions
- Added security best practices
- Added troubleshooting guide

