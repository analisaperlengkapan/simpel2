# Auto-Unseal Health Status

## Overview

The health endpoint (`/health` and `/sys/health`) now includes auto-unseal status information when auto-unseal is configured. This provides visibility into the auto-unseal provider configuration and health for monitoring and alerting purposes.

## Configuration

Auto-unseal status is determined by environment variables:

### Enable Auto-Unseal
```bash
SECRETON_AUTO_UNSEAL_ENABLED=true
SECRETON_AUTO_UNSEAL_PROVIDER=aws-kms|gcp-kms|azure-kv|transit
```

### Provider-Specific Configuration

#### AWS KMS
```bash
SECRETON_AUTO_UNSEAL_PROVIDER=aws-kms
SECRETON_AUTO_UNSEAL_AWS_KEY_ID=alias/secreton-unseal
SECRETON_AUTO_UNSEAL_AWS_REGION=us-east-1
```

#### GCP KMS
```bash
SECRETON_AUTO_UNSEAL_PROVIDER=gcp-kms
SECRETON_AUTO_UNSEAL_GCP_KEY_NAME=projects/my-project/locations/us-central1/keyRings/secreton/cryptoKeys/unseal
SECRETON_AUTO_UNSEAL_GCP_LOCATION=us-central1
```

#### Azure Key Vault
```bash
SECRETON_AUTO_UNSEAL_PROVIDER=azure-kv
SECRETON_AUTO_UNSEAL_AZURE_KEY_NAME=secreton-unseal
SECRETON_AUTO_UNSEAL_AZURE_VAULT_URL=https://myvault.vault.azure.net
```

#### Transit (Another Secreton Instance)
```bash
SECRETON_AUTO_UNSEAL_PROVIDER=transit
SECRETON_AUTO_UNSEAL_TRANSIT_KEY_NAME=auto-unseal-key
SECRETON_AUTO_UNSEAL_TRANSIT_ENDPOINT=https://secreton.internal:50052
```

### Fallback Configuration
```bash
SECRETON_AUTO_UNSEAL_FALLBACK_ENABLED=true  # Default: true
```

## Health Response Format

### Basic Health Endpoint (`/health`)

```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "0.1.0",
    "uptime_seconds": 3600,
    "dependencies": {
      "storage": {
        "healthy": true,
        "message": null,
        "response_time_ms": 5
      },
      "crypto": {
        "healthy": true,
        "message": null,
        "response_time_ms": 2
      },
      "audit": {
        "healthy": true,
        "message": null,
        "response_time_ms": 3
      }
    },
    "auto_unseal": {
      "enabled": true,
      "provider": "aws-kms",
      "provider_key_id": "alias/secreton-unseal",
      "provider_region": "us-east-1",
      "provider_healthy": true,
      "last_unseal": "2026-02-18T10:30:00Z",
      "fallback_enabled": true
    }
  },
  "metadata": {
    "timestamp": "2026-02-18T12:00:00Z",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

### Without Auto-Unseal

When auto-unseal is not configured, the `auto_unseal` field is omitted:

```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "0.1.0",
    "uptime_seconds": 3600,
    "dependencies": {
      "storage": { "healthy": true },
      "crypto": { "healthy": true },
      "audit": { "healthy": true }
    }
  }
}
```

## Auto-Unseal Status Fields

| Field | Type | Description |
|-------|------|-------------|
| `enabled` | boolean | Whether auto-unseal is enabled |
| `provider` | string? | Provider type (aws-kms, gcp-kms, azure-kv, transit) |
| `provider_key_id` | string? | Key identifier (ARN, resource ID, key name) |
| `provider_region` | string? | Cloud provider region (AWS, GCP) |
| `provider_endpoint` | string? | Custom endpoint URL (Azure, Transit) |
| `provider_healthy` | boolean | Whether the provider is reachable and healthy |
| `last_unseal` | datetime? | Timestamp of last successful auto-unseal |
| `fallback_enabled` | boolean | Whether fallback to manual unseal is enabled |

## Monitoring and Alerting

### Prometheus Metrics (Future)

When integrated with the auto-unseal manager, the following metrics will be available:

```
# HELP secreton_auto_unseal_enabled Whether auto-unseal is enabled
# TYPE secreton_auto_unseal_enabled gauge
secreton_auto_unseal_enabled{provider="aws-kms"} 1

# HELP secreton_auto_unseal_provider_healthy Whether the auto-unseal provider is healthy
# TYPE secreton_auto_unseal_provider_healthy gauge
secreton_auto_unseal_provider_healthy{provider="aws-kms",region="us-east-1"} 1

# HELP secreton_auto_unseal_last_success_timestamp Unix timestamp of last successful auto-unseal
# TYPE secreton_auto_unseal_last_success_timestamp gauge
secreton_auto_unseal_last_success_timestamp{provider="aws-kms"} 1708254600
```

### Alert Rules

Example Prometheus alert rules:

```yaml
groups:
  - name: secreton_auto_unseal
    rules:
      - alert: AutoUnsealProviderUnhealthy
        expr: secreton_auto_unseal_provider_healthy == 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Auto-unseal provider is unhealthy"
          description: "Secreton auto-unseal provider {{ $labels.provider }} has been unhealthy for 5 minutes"

      - alert: AutoUnsealNotConfigured
        expr: secreton_auto_unseal_enabled == 0 and secreton_sealed == 1
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Secreton is sealed without auto-unseal"
          description: "Secreton has been sealed for 10 minutes without auto-unseal configured"
```

## Kubernetes Health Probes

The health endpoint can be used for Kubernetes readiness and liveness probes:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: secreton
spec:
  containers:
    - name: secreton
      image: localhost:32000/simpelv2/secreton:latest
      ports:
        - containerPort: 8200
      livenessProbe:
        httpGet:
          path: /health
          port: 8200
        initialDelaySeconds: 30
        periodSeconds: 10
      readinessProbe:
        httpGet:
          path: /health
          port: 8200
        initialDelaySeconds: 10
        periodSeconds: 5
```

## Implementation Notes

### Current Status

The health endpoint now includes auto-unseal status based on environment variables. The implementation:

1. ✅ Reads auto-unseal configuration from environment variables
2. ✅ Returns provider type, key ID, region/endpoint
3. ✅ Includes fallback configuration status
4. ⚠️ Provider health check is basic (checks if key ID is configured)
5. ⚠️ Last unseal timestamp is not yet tracked

### Future Enhancements

When the `AutoUnsealManager` is fully integrated into `ServiceContainer`:

1. **Real Provider Health Checks**: Call `provider.health_check()` to verify connectivity
2. **Last Unseal Tracking**: Store and retrieve last successful unseal timestamp from storage
3. **Provider Metadata**: Use `provider.metadata()` for accurate configuration details
4. **Metrics Integration**: Expose auto-unseal metrics via Prometheus
5. **Audit Integration**: Track auto-unseal attempts in audit log

### Integration Points

The health handler checks auto-unseal status via the `check_auto_unseal_status()` function:

```rust
async fn check_auto_unseal_status(state: &AppState) -> Option<AutoUnsealStatus> {
    // Reads environment variables
    // Returns None if auto-unseal not configured
    // Returns Some(AutoUnsealStatus) with provider details
}
```

When `AutoUnsealManager` is added to `ServiceContainer`, update this function to:

```rust
async fn check_auto_unseal_status(state: &AppState) -> Option<AutoUnsealStatus> {
    if let Some(ref auto_unseal_manager) = state.auto_unseal_manager {
        let provider = auto_unseal_manager.provider();
        let metadata = provider.metadata();
        let provider_healthy = provider.health_check().await.is_ok();
        let last_unseal = auto_unseal_manager.last_unseal_timestamp().await;

        Some(AutoUnsealStatus {
            enabled: true,
            provider: Some(metadata.provider_type),
            provider_key_id: Some(metadata.key_id),
            provider_region: metadata.region,
            provider_endpoint: metadata.endpoint,
            provider_healthy,
            last_unseal,
            fallback_enabled: auto_unseal_manager.fallback_enabled(),
        })
    } else {
        None
    }
}
```

## Testing

Run the health endpoint tests:

```bash
cargo test --test health_auto_unseal_test -p secreton-api
```

Test with environment variables:

```bash
export SECRETON_AUTO_UNSEAL_ENABLED=true
export SECRETON_AUTO_UNSEAL_PROVIDER=aws-kms
export SECRETON_AUTO_UNSEAL_AWS_KEY_ID=alias/secreton-unseal
export SECRETON_AUTO_UNSEAL_AWS_REGION=us-east-1

curl http://localhost:8200/health | jq '.data.auto_unseal'
```

## References

- **Requirements**: AC 2.1.9 - Health endpoint reports auto-unseal status
- **Design**: Section 3.1 - Auto-Unseal Component
- **Related Tasks**:
  - Task 1.1-1.10: Auto-unseal provider implementations
  - Task 1.17: CLI commands for auto-unseal configuration
  - Task 3.1-3.13: Comprehensive health checks

---

**Last Updated**: 2026-02-18
**Status**: Implemented (Basic)
**Next Steps**: Integrate with AutoUnsealManager when available
