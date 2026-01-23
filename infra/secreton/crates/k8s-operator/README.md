# Secreton Kubernetes Operator

The Secreton Kubernetes Operator enables automatic synchronization of secrets from Secreton to Kubernetes Secret objects using a declarative Custom Resource Definition (CRD).

## Features

- **Declarative Secret Sync**: Define SecretSync resources to automatically sync secrets from Secreton
- **Automatic Refresh**: Configurable refresh intervals to keep secrets up-to-date
- **Multiple Auth Methods**: Support for token-based, Kubernetes service account, and AppRole authentication
- **Data Transformation**: Transform secret keys and values before creating Kubernetes Secrets
- **Status Tracking**: Monitor sync status, last sync time, and error messages
- **Namespace Isolation**: Secrets are synced within their respective namespaces

## Installation

### Prerequisites

- Kubernetes cluster (v1.31+)
- Secreton server accessible from the cluster
- kubectl configured to access your cluster

### Deploy the Operator

1. Install the CRD:

```bash
kubectl apply -f deploy/crd.yaml
```

2. Create a secret with your Secreton token:

```bash
kubectl create secret generic secreton-token \
  --from-literal=token=YOUR_SECRETON_TOKEN \
  -n secreton-system
```

3. Deploy the operator:

```bash
kubectl apply -f deploy/operator.yaml
```

## Usage

### Basic Example

Create a SecretSync resource to sync a secret from Secreton:

```yaml
apiVersion: secreton.cipherce.io/v1
kind: SecretSync
metadata:
  name: database-credentials
  namespace: production
spec:
  secretonPath: /secret/data/database/prod
  targetSecret: database-creds
  refreshInterval: 300
  auth:
    method: token
    tokenSecret:
      name: secreton-token
      key: token
```

Apply the resource:

```bash
kubectl apply -f secretsync.yaml
```

The operator will:
1. Fetch the secret from Secreton at `/secret/data/database/prod`
2. Create a Kubernetes Secret named `database-creds` in the `production` namespace
3. Refresh the secret every 300 seconds

### With Data Transformation

Transform secret keys before creating the Kubernetes Secret:

```yaml
apiVersion: secreton.cipherce.io/v1
kind: SecretSync
metadata:
  name: app-config
  namespace: production
spec:
  secretonPath: /secret/data/myapp/config
  targetSecret: app-config
  transform:
    mappings:
      db_host: DATABASE_HOST
      db_port: DATABASE_PORT
      db_user: DATABASE_USER
      db_password: DATABASE_PASSWORD
  auth:
    method: token
    tokenSecret:
      name: secreton-token
      key: token
```

### Check Sync Status

View the status of a SecretSync resource:

```bash
kubectl get secretsync database-credentials -n production -o yaml
```

The status section shows:
- Current phase (Pending, Syncing, Synced, Failed)
- Last sync time
- Last successful sync time
- Error messages (if any)
- Number of sync attempts

## Configuration

### SecretSync Spec

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `secretonPath` | string | Yes | Path to the secret in Secreton (e.g., `/secret/data/myapp/config`) |
| `targetSecret` | string | Yes | Name of the target Kubernetes Secret to create/update |
| `secretonNamespace` | string | No | Secreton namespace (default: "default") |
| `refreshInterval` | integer | No | Refresh interval in seconds (default: 300, minimum: 10) |
| `secretonUrl` | string | No | Secreton server URL (can be configured globally) |
| `auth` | object | No | Authentication configuration |
| `transform` | object | No | Data transformation rules |

### Authentication Methods

#### Token-based Authentication

```yaml
auth:
  method: token
  tokenSecret:
    name: secreton-token
    key: token
```

#### Kubernetes Service Account (TODO)

```yaml
auth:
  method: kubernetes
  serviceAccount: secreton-sync
```

#### AppRole (TODO)

```yaml
auth:
  method: approle
  approle:
    roleId: my-role-id
    secretId:
      name: approle-secret
      key: secret-id
```

### Data Transformation

#### Key Mappings

Map Secreton keys to different Kubernetes Secret keys:

```yaml
transform:
  mappings:
    source_key: TARGET_KEY
    another_key: ANOTHER_TARGET_KEY
```

## Monitoring

The operator exposes metrics for monitoring:

- `secretsync_reconciliations_total`: Total number of reconciliations
- `secretsync_reconciliation_errors_total`: Total number of reconciliation errors
- `secretsync_sync_duration_seconds`: Duration of sync operations

## Troubleshooting

### Secret Not Syncing

1. Check the SecretSync status:
   ```bash
   kubectl describe secretsync <name> -n <namespace>
   ```

2. Check operator logs:
   ```bash
   kubectl logs -n secreton-system -l app=secreton-operator
   ```

3. Verify Secreton connectivity:
   ```bash
   kubectl run -it --rm debug --image=curlimages/curl --restart=Never -- \
     curl -H "X-Secret Vault-Token: $TOKEN" https://secreton.internal:8200/v1/sys/health
   ```

### Authentication Errors

- Verify the token secret exists and contains a valid token
- Check that the token has the necessary permissions in Secreton
- Ensure the Secreton URL is correct and accessible from the cluster

### Refresh Not Working

- Verify the `refreshInterval` is at least 10 seconds
- Check that the SecretSync resour not in a Failed state
- Review operator logs for reconciliation errors

## Development

### Building

```bash
cargo build --release
```

### Running Locally

```bash
export SECRETON_URL=https://secreton.internal:8200
export SECRETON_TOKEN=your-token
cargo run
```

### Testing

```bash
cargo test
```

## Architecture

The operator consists of:

1. **CRD Definition**: Defines the SecretSync custom resource
2. **Controller**: Watches for SecretSync resources and reconciles them
3. **Reconciler**: Core logic for fetching secrets and creating Kubernetes Secrets
4. **Error Handling**: Comprehensive error handling with retry logic

## Requirements

Validates: Requirements 5.1

## License

Apache-2.0

