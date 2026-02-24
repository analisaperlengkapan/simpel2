# Secreton Auto-Unseal Guide

## Table of Contents

1. [Overview](#overview)
2. [Benefits of Auto-Unseal](#benefits-of-auto-unseal)
3. [Prerequisites](#prerequisites)
4. [Provider Setup Guides](#provider-setup-guides)
   - [Transit Provider](#transit-provider)
   - [AWS KMS Provider](#aws-kms-provider)
   - [GCP KMS Provider](#gcp-kms-provider)
   - [Azure Key Vault Provider](#azure-key-vault-provider)
5. [Configuration](#configuration)
6. [Testing and Verification](#testing-and-verification)
7. [Troubleshooting](#troubleshooting)
8. [Security Best Practices](#security-best-practices)
9. [Migration from Manual Unseal](#migration-from-manual-unseal)

---

## Overview

Auto-unseal is a critical feature that allows Secreton to automatically unseal itself on startup using an external Key Management Service (KMS) instead of requiring manual entry of Shamir unseal keys. This is essential for:

- **Kubernetes deployments**: Pods can restart automatically without operator intervention
- **High availability**: Nodes can recover from failures without manual unsealing
- **Disaster recovery**: Automated failover without human interaction
- **Operational efficiency**: Reduces operational burden and human error

### How It Works

1. During initialization, Secreton generates a master encryption key
2. The master key is encrypted using the configured KMS provider
3. The encrypted master key is stored in PostgreSQL
4. On startup, Secreton:
   - Retrieves the encrypted master key from storage
   - Calls the KMS provider to decrypt it
   - Loads the decrypted master key into memory
   - Transitions to unsealed state

### Supported Providers

| Provider | Use Case | Authentication |
|----------|----------|----------------|
| **Transit** | Use another Secreton instance | Token-based |
| **AWS KMS** | AWS deployments | IAM roles or access keys |
| **GCP KMS** | GCP deployments | Service accounts |
| **Azure Key Vault** | Azure deployments | Managed identity or service principal |

---

## Benefits of Auto-Unseal

### Operational Benefits

- **Zero-Touch Operations**: No manual intervention required for restarts
- **Kubernetes Native**: Seamless integration with pod lifecycle management
- **High Availability**: Automatic recovery from node failures
- **Disaster Recovery**: Automated failover without operator involvement
- **Reduced MTTR**: Mean Time To Recovery drops from minutes to seconds

### Security Benefits

- **No Shamir Keys in Memory**: Unseal keys never stored on Secreton nodes
- **Centralized Key Management**: Leverage enterprise KMS for key protection
- **Audit Trail**: All unseal operations logged to KMS audit logs
- **Hardware Security**: KMS providers use HSMs for key protection
- **Compliance**: Meets regulatory requirements for key management

### Comparison: Manual vs Auto-Unseal

| Aspect | Manual Unseal | Auto-Unseal |
|--------|---------------|-------------|
| **Restart Time** | 5-15 minutes | 10-30 seconds |
| **Operator Required** | Yes (3 operators for threshold) | No |
| **Kubernetes Compatible** | No | Yes |
| **HA Failover** | Manual | Automatic |
| **Key Storage** | Distributed to operators | Encrypted in KMS |
| **Audit Trail** | Limited | Comprehensive |

---

## Prerequisites

### General Requirements

- Secreton version 0.1.0 or later
- PostgreSQL database for metadata storage
- Network connectivity to chosen KMS provider
- Appropriate permissions/credentials for KMS access

### Provider-Specific Requirements

#### Transit Provider
- Another Secreton instance (unsealed and operational)
- Transit engine enabled on the remote Secreton
- Transit key created for auto-unseal
- Valid authentication token with encrypt/decrypt permissions

#### AWS KMS Provider
- AWS account with KMS access
- KMS key created in target region
- IAM role or access keys with `kms:Encrypt` and `kms:Decrypt` permissions
- Network access to AWS KMS endpoints

#### GCP KMS Provider
- GCP project with Cloud KMS API enabled
- KMS key ring and crypto key created
- Service account with `cloudkms.cryptoKeyVersions.useToEncrypt` and `cloudkms.cryptoKeyVersions.useToDecrypt` permissions
- Service account key file or Workload Identity

#### Azure Key Vault Provider
- Azure subscription with Key Vault created
- Key created in Key Vault
- Managed identity or service principal with `encrypt` and `decrypt` permissions
- Network access to Azure Key Vault endpoints

---

## Provider Setup Guides

### Transit Provider

The Transit provider uses another Secreton instance as the KMS. This is useful for:
- Multi-tier deployments (production Secreton unseals staging Secreton)
- Air-gapped environments (no external KMS access)
- Testing and development

#### Step 1: Prepare the Remote Secreton Instance

On the **remote** Secreton instance (the one that will act as KMS):

```bash
# 1. Ensure Transit engine is enabled (it's enabled by default)
secreton-cli transit list-keys

# 2. Create a dedicated key for auto-unseal
secreton-cli transit create-key auto-unseal-key \
  --type chacha20-poly1305 \
  --exportable false

# 3. Create a policy for auto-unseal access
cat > auto-unseal-policy.hcl <<EOF
path "transit/encrypt/auto-unseal-key" {
  capabilities = ["update"]
}

path "transit/decrypt/auto-unseal-key" {
  capabilities = ["update"]
}
EOF

secreton-cli policy write auto-unseal-policy auto-unseal-policy.hcl

# 4. Create a token with the policy
secreton-cli token create \
  --policy auto-unseal-policy \
  --ttl 0 \
  --renewable true \
  --display-name "Auto-Unseal Token"

# Save the token securely - you'll need it for configuration
```

#### Step 2: Configure Auto-Unseal on Target Secreton

On the **target** Secreton instance (the one that will auto-unseal):

```bash
# Interactive configuration
secreton-cli auto-unseal configure --provider transit

# Non-interactive configuration
secreton-cli auto-unseal configure \
  --provider transit \
  --non-interactive \
  --transit-endpoint "https://secreton-primary.internal:50052" \
  --transit-key-name "auto-unseal-key" \
  --transit-token "hvs.CAESIJ..."
```

#### Step 3: Configuration File Example

Alternatively, configure via `secreton.toml`:

```toml
[auto_unseal]
enabled = true
provider = "transit"

[auto_unseal.transit]
endpoint = "https://secreton-primary.internal:50052"
key_name = "auto-unseal-key"
token = "hvs.CAESIJ..."  # Or use environment variable
timeout_secs = 30
tls_verify = true
ca_cert_path = "/etc/secreton/ca.crt"  # Optional
```

Environment variable for token (recommended):

```bash
export SECRETON_AUTO_UNSEAL_TRANSIT_TOKEN="hvs.CAESIJ..."
```

---

### AWS KMS Provider

The AWS KMS provider is ideal for AWS deployments and supports both IAM roles and access keys.

#### Step 1: Create KMS Key

```bash
# Create a KMS key for auto-unseal
aws kms create-key \
  --description "Secreton Auto-Unseal Key" \
  --key-usage ENCRYPT_DECRYPT \
  --origin AWS_KMS \
  --region us-east-1

# Note the KeyId from the output
# Example: arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012

# Create an alias for easier reference
aws kms create-alias \
  --alias-name alias/secreton-auto-unseal \
  --target-key-id <KeyId> \
  --region us-east-1
```

#### Step 2: Configure IAM Permissions

**Option A: IAM Role (Recommended for EC2/EKS)**

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "kms:Encrypt",
        "kms:Decrypt",
        "kms:DescribeKey"
      ],
      "Resource": "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012"
    }
  ]
}
```

Attach this policy to the IAM role used by Secreton pods/instances.

**Option B: IAM User with Access Keys**

```bash
# Create IAM user
aws iam create-user --user-name secreton-auto-unseal

# Attach policy
aws iam put-user-policy \
  --user-name secreton-auto-unseal \
  --policy-name SecretonAutoUnsealPolicy \
  --policy-document file://kms-policy.json

# Create access keys
aws iam create-access-key --user-name secreton-auto-unseal
```

#### Step 3: Configure Auto-Unseal

```bash
# Using IAM role (no credentials needed)
secreton-cli auto-unseal configure \
  --provider aws-kms \
  --non-interactive \
  --aws-key-id "alias/secreton-auto-unseal" \
  --aws-region "us-east-1"

# Using access keys
secreton-cli auto-unseal configure \
  --provider aws-kms \
  --non-interactive \
  --aws-key-id "alias/secreton-auto-unseal" \
  --aws-region "us-east-1" \
  --aws-access-key-id "AKIAIOSFODNN7EXAMPLE" \
  --aws-secret-access-key "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
```

#### Step 4: Configuration File Example

```toml
[auto_unseal]
enabled = true
provider = "aws-kms"

[auto_unseal.aws_kms]
key_id = "alias/secreton-auto-unseal"
region = "us-east-1"
# Optional: Provide credentials (uses IAM role if omitted)
# access_key_id = "AKIAIOSFODNN7EXAMPLE"
# secret_access_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
```

Environment variables (recommended for credentials):

```bash
export AWS_ACCESS_KEY_ID="AKIAIOSFODNN7EXAMPLE"
export AWS_SECRET_ACCESS_KEY="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
export AWS_REGION="us-east-1"
```

#### Kubernetes Configuration

For EKS deployments, use IAM Roles for Service Accounts (IRSA):

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: secreton
  namespace: secreton-system
  annotations:
    eks.amazonaws.com/role-arn: arn:aws:iam::123456789012:role/secreton-auto-unseal-role
---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: secreton
spec:
  template:
    spec:
      serviceAccountName: secreton
      containers:
      - name: secreton
        env:
        - name: SECRETON_AUTO_UNSEAL_PROVIDER
          value: "aws-kms"
        - name: SECRETON_AUTO_UNSEAL_AWS_KEY_ID
          value: "alias/secreton-auto-unseal"
        - name: SECRETON_AUTO_UNSEAL_AWS_REGION
          value: "us-east-1"
```

---

### GCP KMS Provider

The GCP KMS provider is ideal for GCP deployments and supports both service account keys and Workload Identity.

#### Step 1: Enable Cloud KMS API

```bash
# Enable the API
gcloud services enable cloudkms.googleapis.com --project=my-project

# Create a key ring
gcloud kms keyrings create secreton-keyring \
  --location=global \
  --project=my-project

# Create a crypto key
gcloud kms keys create auto-unseal-key \
  --location=global \
  --keyring=secreton-keyring \
  --purpose=encryption \
  --project=my-project
```

#### Step 2: Configure Service Account Permissions

**Option A: Service Account Key File**

```bash
# Create service account
gcloud iam service-accounts create secreton-auto-unseal \
  --display-name="Secreton Auto-Unseal" \
  --project=my-project

# Grant permissions
gcloud kms keys add-iam-policy-binding auto-unseal-key \
  --location=global \
  --keyring=secreton-keyring \
  --member="serviceAccount:secreton-auto-unseal@my-project.iam.gserviceaccount.com" \
  --role="roles/cloudkms.cryptoKeyEncrypterDecrypter" \
  --project=my-project

# Create and download key file
gcloud iam service-accounts keys create secreton-sa-key.json \
  --iam-account=secreton-auto-unseal@my-project.iam.gserviceaccount.com \
  --project=my-project
```

**Option B: Workload Identity (GKE)**

```bash
# Create Kubernetes service account
kubectl create serviceaccount secreton -n secreton-system

# Bind to GCP service account
gcloud iam service-accounts add-iam-policy-binding \
  secreton-auto-unseal@my-project.iam.gserviceaccount.com \
  --role roles/iam.workloadIdentityUser \
  --member "serviceAccount:my-project.svc.id.goog[secreton-system/secreton]" \
  --project=my-project

# Annotate Kubernetes service account
kubectl annotate serviceaccount secreton \
  -n secreton-system \
  iam.gke.io/gcp-service-account=secreton-auto-unseal@my-project.iam.gserviceaccount.com
```

#### Step 3: Configure Auto-Unseal

```bash
# Using service account key file
secreton-cli auto-unseal configure \
  --provider gcp-kms \
  --non-interactive \
  --gcp-project-id "my-project" \
  --gcp-location "global" \
  --gcp-key-ring "secreton-keyring" \
  --gcp-crypto-key "auto-unseal-key" \
  --gcp-credentials-file "/etc/secreton/secreton-sa-key.json"

# Using Workload Identity (no credentials file needed)
secreton-cli auto-unseal configure \
  --provider gcp-kms \
  --non-interactive \
  --gcp-project-id "my-project" \
  --gcp-location "global" \
  --gcp-key-ring "secreton-keyring" \
  --gcp-crypto-key "auto-unseal-key"
```

#### Step 4: Configuration File Example

```toml
[auto_unseal]
enabled = true
provider = "gcp-kms"

[auto_unseal.gcp_kms]
project_id = "my-project"
location = "global"
key_ring = "secreton-keyring"
crypto_key = "auto-unseal-key"
# Optional: Provide credentials file (uses default credentials if omitted)
# credentials_file = "/etc/secreton/secreton-sa-key.json"
```

Environment variable for credentials:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/etc/secreton/secreton-sa-key.json"
```

#### Kubernetes Configuration (GKE with Workload Identity)

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: secreton
  namespace: secreton-system
  annotations:
    iam.gke.io/gcp-service-account: secreton-auto-unseal@my-project.iam.gserviceaccount.com
---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: secreton
spec:
  template:
    spec:
      serviceAccountName: secreton
      containers:
      - name: secreton
        env:
        - name: SECRETON_AUTO_UNSEAL_PROVIDER
          value: "gcp-kms"
        - name: SECRETON_AUTO_UNSEAL_GCP_PROJECT_ID
          value: "my-project"
        - name: SECRETON_AUTO_UNSEAL_GCP_LOCATION
          value: "global"
        - name: SECRETON_AUTO_UNSEAL_GCP_KEY_RING
          value: "secreton-keyring"
        - name: SECRETON_AUTO_UNSEAL_GCP_CRYPTO_KEY
          value: "auto-unseal-key"
```

---

### Azure Key Vault Provider

The Azure Key Vault provider is ideal for Azure deployments and supports both managed identity and service principals.

#### Step 1: Create Key Vault and Key

```bash
# Create resource group
az group create --name secreton-rg --location eastus

# Create Key Vault
az keyvault create \
  --name secreton-kv \
  --resource-group secreton-rg \
  --location eastus \
  --enable-rbac-authorization false

# Create key
az keyvault key create \
  --vault-name secreton-kv \
  --name auto-unseal-key \
  --protection software \
  --kty RSA \
  --size 2048
```

#### Step 2: Configure Permissions

**Option A: Managed Identity (Recommended for AKS/VMs)**

```bash
# Create managed identity
az identity create \
  --name secreton-identity \
  --resource-group secreton-rg

# Get identity details
IDENTITY_ID=$(az identity show \
  --name secreton-identity \
  --resource-group secreton-rg \
  --query principalId -o tsv)

# Grant Key Vault permissions
az keyvault set-policy \
  --name secreton-kv \
  --object-id $IDENTITY_ID \
  --key-permissions encrypt decrypt get
```

**Option B: Service Principal**

```bash
# Create service principal
az ad sp create-for-rbac \
  --name secreton-auto-unseal \
  --skip-assignment

# Note the appId, password, and tenant from output

# Grant Key Vault permissions
az keyvault set-policy \
  --name secreton-kv \
  --spn <appId> \
  --key-permissions encrypt decrypt get
```

#### Step 3: Configure Auto-Unseal

```bash
# Using managed identity (no credentials needed)
secreton-cli auto-unseal configure \
  --provider azure-kv \
  --non-interactive \
  --azure-vault-name "secreton-kv" \
  --azure-key-name "auto-unseal-key"

# Using service principal
secreton-cli auto-unseal configure \
  --provider azure-kv \
  --non-interactive \
  --azure-vault-name "secreton-kv" \
  --azure-key-name "auto-unseal-key" \
  --azure-tenant-id "12345678-1234-1234-1234-123456789012" \
  --azure-client-id "87654321-4321-4321-4321-210987654321" \
  --azure-client-secret "your-client-secret"
```

#### Step 4: Configuration File Example

```toml
[auto_unseal]
enabled = true
provider = "azure-kv"

[auto_unseal.azure_kv]
vault_name = "secreton-kv"
key_name = "auto-unseal-key"
# Optional: Provide service principal credentials (uses managed identity if omitted)
# tenant_id = "12345678-1234-1234-1234-123456789012"
# client_id = "87654321-4321-4321-4321-210987654321"
# client_secret = "your-client-secret"
```

Environment variables for credentials:

```bash
export AZURE_TENANT_ID="12345678-1234-1234-1234-123456789012"
export AZURE_CLIENT_ID="87654321-4321-4321-4321-210987654321"
export AZURE_CLIENT_SECRET="your-client-secret"
```

#### Kubernetes Configuration (AKS with Managed Identity)

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: secreton
  namespace: secreton-system
  annotations:
    azure.workload.identity/client-id: "87654321-4321-4321-4321-210987654321"
---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: secreton
spec:
  template:
    metadata:
      labels:
        azure.workload.identity/use: "true"
    spec:
      serviceAccountName: secreton
      containers:
      - name: secreton
        env:
        - name: SECRETON_AUTO_UNSEAL_PROVIDER
          value: "azure-kv"
        - name: SECRETON_AUTO_UNSEAL_AZURE_VAULT_NAME
          value: "secreton-kv"
        - name: SECRETON_AUTO_UNSEAL_AZURE_KEY_NAME
          value: "auto-unseal-key"
```

---

## Configuration

### Configuration Methods

Secreton supports three methods for auto-unseal configuration (in order of precedence):

1. **Environment Variables** (highest priority)
2. **Configuration File** (`secreton.toml`)
3. **CLI Configuration** (stored in database)

### Complete Configuration Reference

#### Common Settings

```toml
[auto_unseal]
# Enable auto-unseal
enabled = true

# Provider type: "transit", "aws-kms", "gcp-kms", "azure-kv"
provider = "aws-kms"

# Fallback to manual unseal if auto-unseal fails
fallback_enabled = true

# Maximum retry attempts before fallback
max_retries = 3

# Retry delay in seconds (exponential backoff)
retry_delay_secs = 5
```

#### Transit Provider Settings

```toml
[auto_unseal.transit]
endpoint = "https://secreton-primary.internal:50052"
key_name = "auto-unseal-key"
token = "${SECRETON_AUTO_UNSEAL_TRANSIT_TOKEN}"  # Use env var
timeout_secs = 30
tls_verify = true
ca_cert_path = "/etc/secreton/ca.crt"
```

#### AWS KMS Provider Settings

```toml
[auto_unseal.aws_kms]
key_id = "alias/secreton-auto-unseal"
region = "us-east-1"
# Optional: credentials (uses IAM role if omitted)
access_key_id = "${AWS_ACCESS_KEY_ID}"
secret_access_key = "${AWS_SECRET_ACCESS_KEY}"
# Optional: custom endpoint (for LocalStack, etc.)
endpoint = "http://localhost:4566"
```

#### GCP KMS Provider Settings

```toml
[auto_unseal.gcp_kms]
project_id = "my-project"
location = "global"
key_ring = "secreton-keyring"
crypto_key = "auto-unseal-key"
# Optional: credentials file (uses default credentials if omitted)
credentials_file = "${GOOGLE_APPLICATION_CREDENTIALS}"
```

#### Azure Key Vault Provider Settings

```toml
[auto_unseal.azure_kv]
vault_name = "secreton-kv"
key_name = "auto-unseal-key"
# Optional: service principal credentials (uses managed identity if omitted)
tenant_id = "${AZURE_TENANT_ID}"
client_id = "${AZURE_CLIENT_ID}"
client_secret = "${AZURE_CLIENT_SECRET}"
```

### Environment Variables Reference

| Variable | Description | Example |
|----------|-------------|---------|
| `SECRETON_AUTO_UNSEAL_ENABLED` | Enable auto-unseal | `true` |
| `SECRETON_AUTO_UNSEAL_PROVIDER` | Provider type | `aws-kms` |
| `SECRETON_AUTO_UNSEAL_FALLBACK_ENABLED` | Enable fallback | `true` |
| **Transit** | | |
| `SECRETON_AUTO_UNSEAL_TRANSIT_ENDPOINT` | Transit endpoint | `https://...` |
| `SECRETON_AUTO_UNSEAL_TRANSIT_KEY_NAME` | Key name | `auto-unseal-key` |
| `SECRETON_AUTO_UNSEAL_TRANSIT_TOKEN` | Auth token | `hvs.CAESIJ...` |
| **AWS KMS** | | |
| `SECRETON_AUTO_UNSEAL_AWS_KEY_ID` | KMS key ID/ARN | `alias/secreton` |
| `SECRETON_AUTO_UNSEAL_AWS_REGION` | AWS region | `us-east-1` |
| `AWS_ACCESS_KEY_ID` | AWS access key | `AKIAIOSFODNN...` |
| `AWS_SECRET_ACCESS_KEY` | AWS secret key | `wJalrXUtnFEMI...` |
| **GCP KMS** | | |
| `SECRETON_AUTO_UNSEAL_GCP_PROJECT_ID` | GCP project | `my-project` |
| `SECRETON_AUTO_UNSEAL_GCP_LOCATION` | Location | `global` |
| `SECRETON_AUTO_UNSEAL_GCP_KEY_RING` | Key ring | `secreton-keyring` |
| `SECRETON_AUTO_UNSEAL_GCP_CRYPTO_KEY` | Crypto key | `auto-unseal-key` |
| `GOOGLE_APPLICATION_CREDENTIALS` | Credentials file | `/path/to/key.json` |
| **Azure Key Vault** | | |
| `SECRETON_AUTO_UNSEAL_AZURE_VAULT_NAME` | Vault name | `secreton-kv` |
| `SECRETON_AUTO_UNSEAL_AZURE_KEY_NAME` | Key name | `auto-unseal-key` |
| `AZURE_TENANT_ID` | Tenant ID | `12345678-...` |
| `AZURE_CLIENT_ID` | Client ID | `87654321-...` |
| `AZURE_CLIENT_SECRET` | Client secret | `your-secret` |

---

## Testing and Verification

### Step 1: Test Configuration

Before restarting Secreton, test the auto-unseal configuration:

```bash
secreton-cli auto-unseal test
```

Expected output:
```
🧪 Testing Auto-Unseal Configuration

   Testing provider connectivity...
   ✅ Provider connectivity: OK
   ✅ Encryption test: OK
   ✅ Decryption test: OK

✅ Auto-unseal configuration is working correctly
   Average latency: 45.23ms
```

### Step 2: Check Auto-Unseal Status

```bash
secreton-cli auto-unseal status
```

Expected output:
```
🔍 Auto-Unseal Status

   Status: ✅ ENABLED
   Provider: aws-kms
   Key ID: alias/secreton-auto-unseal
   Region: us-east-1
   Health: ✅ HEALTHY
   Last Health Check: 2026-02-18T10:30:45Z
```

### Step 3: Verify Health Endpoint

```bash
curl -s http://localhost:8200/v1/sys/health | jq '.auto_unseal'
```

Expected output:
```json
{
  "enabled": true,
  "provider": "aws-kms",
  "key_id": "alias/secreton-auto-unseal",
  "region": "us-east-1",
  "health_status": "healthy",
  "provider_healthy": true,
  "fallback_enabled": true,
  "last_health_check": "2026-02-18T10:30:45Z"
}
```

### Step 4: Test Restart

Perform a controlled restart to verify auto-unseal works:

```bash
# 1. Check current seal status
secreton-cli status

# 2. Restart Secreton
systemctl restart secreton
# OR for Kubernetes:
kubectl rollout restart statefulset/secreton -n secreton-system

# 3. Wait for startup (10-30 seconds)
sleep 30

# 4. Verify unsealed status
secreton-cli status
```

Expected output:
```
Seal Status: unsealed
Seal Type: auto-unseal (aws-kms)
Initialized: true
```

### Step 5: Check Audit Logs

Verify auto-unseal operations are logged:

```bash
# Check audit logs for unseal events
secreton-cli audit list --filter "operation=auto-unseal"
```

Expected output:
```
Timestamp: 2026-02-18T10:35:12Z
Operation: auto-unseal
Provider: aws-kms
Status: success
Duration: 234ms
```

---

## Troubleshooting

### Common Issues and Solutions

#### Issue 1: "Provider connectivity failed"

**Symptoms:**
```
❌ Test failed: Provider connectivity failed
Error: Failed to connect to KMS provider
```

**Possible Causes:**
- Network connectivity issues
- Incorrect endpoint URL
- Firewall blocking access

**Solutions:**

1. **Check network connectivity:**
   ```bash
   # For AWS KMS
   curl -v https://kms.us-east-1.amazonaws.com

   # For GCP KMS
   curl -v https://cloudkms.googleapis.com

   # For Azure Key Vault
   curl -v https://secreton-kv.vault.azure.net

   # For Transit
   curl -v https://secreton-primary.internal:50052
   ```

2. **Verify DNS resolution:**
   ```bash
   nslookup kms.us-east-1.amazonaws.com
   ```

3. **Check firewall rules:**
   ```bash
   # Ensure outbound HTTPS (443) is allowed
   # For Transit, ensure gRPC port (50052) is allowed
   ```

#### Issue 2: "Permission denied" or "Access denied"

**Symptoms:**
```
❌ Test failed: Permission denied
Error: User is not authorized to perform: kms:Decrypt
```

**Possible Causes:**
- Insufficient IAM/RBAC permissions
- Incorrect service account/role
- Key policy restrictions

**Solutions:**

1. **AWS KMS - Verify IAM permissions:**
   ```bash
   # Check current IAM role/user
   aws sts get-caller-identity

   # Test KMS permissions
   aws kms describe-key --key-id alias/secreton-auto-unseal

   # Test encrypt/decrypt
   echo "test" | base64 | aws kms encrypt \
     --key-id alias/secreton-auto-unseal \
     --plaintext fileb:///dev/stdin \
     --query CiphertextBlob \
     --output text
   ```

2. **GCP KMS - Verify service account permissions:**
   ```bash
   # Check current service account
   gcloud auth list

   # Test KMS permissions
   gcloud kms keys describe auto-unseal-key \
     --location=global \
     --keyring=secreton-keyring

   # Test encrypt/decrypt
   echo "test" | gcloud kms encrypt \
     --location=global \
     --keyring=secreton-keyring \
     --key=auto-unseal-key \
     --plaintext-file=- \
     --ciphertext-file=-
   ```

3. **Azure Key Vault - Verify access policies:**
   ```bash
   # Check current identity
   az account show

   # List Key Vault access policies
   az keyvault show --name secreton-kv --query properties.accessPolicies

   # Test key access
   az keyvault key show --vault-name secreton-kv --name auto-unseal-key
   ```

4. **Transit - Verify token permissions:**
   ```bash
   # Check token capabilities
   secreton-cli token capabilities transit/encrypt/auto-unseal-key
   secreton-cli token capabilities transit/decrypt/auto-unseal-key
   ```

#### Issue 3: "Encrypted master key not found"

**Symptoms:**
```
❌ Auto-unseal failed: Encrypted master key not found in storage
```

**Possible Causes:**
- First-time setup not completed
- Database migration not run
- Master key not encrypted with KMS

**Solutions:**

1. **Check if Secreton is initialized:**
   ```bash
   secreton-cli status
   ```

2. **If not initialized, initialize with auto-unseal:**
   ```bash
   # This will encrypt the master key with the configured KMS
   secreton-cli init
   ```

3. **If already initialized with manual unseal, migrate to auto-unseal:**
   See [Migration from Manual Unseal](#migration-from-manual-unseal) section.

#### Issue 4: "Auto-unseal timeout"

**Symptoms:**
```
❌ Auto-unseal failed: Operation timed out after 30s
```

**Possible Causes:**
- KMS provider slow to respond
- Network latency issues
- KMS provider rate limiting

**Solutions:**

1. **Increase timeout in configuration:**
   ```toml
   [auto_unseal.aws_kms]
   timeout_secs = 60  # Increase from default 30s
   ```

2. **Check KMS provider status:**
   ```bash
   # AWS
   aws health describe-events --filter eventTypeCategories=issue

   # GCP
   gcloud status

   # Azure
   az status
   ```

3. **Monitor network latency:**
   ```bash
   # Measure latency to KMS endpoint
   time curl -s https://kms.us-east-1.amazonaws.com > /dev/null
   ```

#### Issue 5: "Fallback to manual unseal triggered"

**Symptoms:**
```
⚠️  Auto-unseal failed after 3 retries, falling back to manual unseal
```

**Possible Causes:**
- Temporary KMS provider outage
- Network issues
- Configuration error

**Solutions:**

1. **Check auto-unseal logs:**
   ```bash
   journalctl -u secreton -n 100 | grep auto-unseal
   ```

2. **Manually unseal to restore service:**
   ```bash
   secreton-cli seal unseal
   # Enter unseal keys when prompted
   ```

3. **Fix the underlying issue and test:**
   ```bash
   secreton-cli auto-unseal test
   ```

4. **Restart to retry auto-unseal:**
   ```bash
   systemctl restart secreton
   ```

#### Issue 6: "Invalid credentials" (AWS/GCP/Azure)

**Symptoms:**
```
❌ Test failed: Invalid credentials
Error: The security token included in the request is invalid
```

**Solutions:**

1. **AWS - Refresh credentials:**
   ```bash
   # If using temporary credentials, refresh them
   aws sts get-session-token

   # Verify credentials work
   aws sts get-caller-identity
   ```

2. **GCP - Refresh service account key:**
   ```bash
   # Generate new service account key
   gcloud iam service-accounts keys create new-key.json \
     --iam-account=secreton-auto-unseal@my-project.iam.gserviceaccount.com

   # Update configuration
   export GOOGLE_APPLICATION_CREDENTIALS="/path/to/new-key.json"
   ```

3. **Azure - Refresh service principal secret:**
   ```bash
   # Create new client secret
   az ad sp credential reset --id <client-id>

   # Update configuration with new secret
   ```

### Debugging Tips

#### Enable Debug Logging

```toml
[logging]
level = "debug"
```

Or via environment variable:
```bash
export RUST_LOG=secreton=debug
```

#### Check Secreton Logs

```bash
# Systemd
journalctl -u secreton -f

# Kubernetes
kubectl logs -f statefulset/secreton -n secreton-system

# Docker
docker logs -f secreton
```

#### Test KMS Provider Manually

**AWS KMS:**
```bash
# Encrypt test data
echo "test" | base64 > plaintext.txt
aws kms encrypt \
  --key-id alias/secreton-auto-unseal \
  --plaintext fileb://plaintext.txt \
  --output text \
  --query CiphertextBlob > ciphertext.txt

# Decrypt test data
aws kms decrypt \
  --ciphertext-blob fileb://ciphertext.txt \
  --output text \
  --query Plaintext | base64 -d
```

**GCP KMS:**
```bash
# Encrypt test data
echo "test" > plaintext.txt
gcloud kms encrypt \
  --location=global \
  --keyring=secreton-keyring \
  --key=auto-unseal-key \
  --plaintext-file=plaintext.txt \
  --ciphertext-file=ciphertext.txt

# Decrypt test data
gcloud kms decrypt \
  --location=global \
  --keyring=secreton-keyring \
  --key=auto-unseal-key \
  --ciphertext-file=ciphertext.txt \
  --plaintext-file=-
```

**Azure Key Vault:**
```bash
# Encrypt test data
echo "test" | base64 > plaintext.txt
az keyvault key encrypt \
  --vault-name secreton-kv \
  --name auto-unseal-key \
  --algorithm RSA-OAEP \
  --value @plaintext.txt > ciphertext.json

# Decrypt test data
az keyvault key decrypt \
  --vault-name secreton-kv \
  --name auto-unseal-key \
  --algorithm RSA-OAEP \
  --value $(jq -r .result ciphertext.json) | base64 -d
```

---

## Security Best Practices

### 1. Use Managed Identities/IAM Roles

**✅ DO:**
- Use IAM roles for EC2/EKS (AWS)
- Use Workload Identity for GKE (GCP)
- Use Managed Identity for AKS/VMs (Azure)

**❌ DON'T:**
- Store access keys in configuration files
- Commit credentials to version control
- Share credentials across environments

### 2. Principle of Least Privilege

Grant only the minimum required permissions:

**AWS KMS Policy:**
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "kms:Decrypt",
        "kms:Encrypt",
        "kms:DescribeKey"
      ],
      "Resource": "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012"
    }
  ]
}
```

**GCP KMS Role:**
```bash
# Use predefined role with minimal permissions
gcloud kms keys add-iam-policy-binding auto-unseal-key \
  --location=global \
  --keyring=secreton-keyring \
  --member="serviceAccount:secreton@my-project.iam.gserviceaccount.com" \
  --role="roles/cloudkms.cryptoKeyEncrypterDecrypter"
```

### 3. Enable Audit Logging

Ensure all KMS operations are logged:

**AWS CloudTrail:**
```bash
# Enable CloudTrail for KMS events
aws cloudtrail create-trail \
  --name secreton-kms-audit \
  --s3-bucket-name my-audit-bucket

aws cloudtrail start-logging --name secreton-kms-audit
```

**GCP Cloud Audit Logs:**
```bash
# Audit logs are enabled by default for Cloud KMS
# Verify in Cloud Console: IAM & Admin > Audit Logs
```

**Azure Monitor:**
```bash
# Enable diagnostic settings for Key Vault
az monitor diagnostic-settings create \
  --resource /subscriptions/.../resourceGroups/secreton-rg/providers/Microsoft.KeyVault/vaults/secreton-kv \
  --name kv-audit \
  --logs '[{"category": "AuditEvent", "enabled": true}]' \
  --workspace /subscriptions/.../resourceGroups/monitoring/providers/Microsoft.OperationalInsights/workspaces/my-workspace
```

### 4. Rotate KMS Keys Regularly

**AWS KMS:**
```bash
# Enable automatic key rotation (yearly)
aws kms enable-key-rotation --key-id alias/secreton-auto-unseal
```

**GCP KMS:**
```bash
# Set rotation period (90 days)
gcloud kms keys update auto-unseal-key \
  --location=global \
  --keyring=secreton-keyring \
  --rotation-period=90d \
  --next-rotation-time=$(date -d "+90 days" +%Y-%m-%dT%H:%M:%S%z)
```

**Azure Key Vault:**
```bash
# Azure doesn't support automatic rotation, rotate manually:
az keyvault key create \
  --vault-name secreton-kv \
  --name auto-unseal-key \
  --protection software
```

### 5. Use Separate Keys per Environment

**✅ DO:**
```
Production:   alias/secreton-auto-unseal-prod
Staging:      alias/secreton-auto-unseal-staging
Development:  alias/secreton-auto-unseal-dev
```

**❌ DON'T:**
```
All environments: alias/secreton-auto-unseal
```

### 6. Enable Fallback to Manual Unseal

Always configure fallback for disaster recovery:

```toml
[auto_unseal]
enabled = true
fallback_enabled = true  # Critical for DR
max_retries = 3
```

### 7. Monitor Auto-Unseal Health

Set up alerts for auto-unseal failures:

```bash
# Prometheus alert rule
- alert: SecretonAutoUnsealFailed
  expr: secreton_auto_unseal_failures_total > 0
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "Secreton auto-unseal failed"
    description: "Auto-unseal has failed {{ $value }} times in the last 5 minutes"
```

### 8. Secure Transit Provider

If using Transit provider:

- Use mTLS for all connections
- Rotate Transit tokens regularly
- Use dedicated Transit key for auto-unseal
- Monitor Transit key usage

```bash
# Create non-expiring token with minimal permissions
secreton-cli token create \
  --policy auto-unseal-policy \
  --ttl 0 \
  --renewable true \
  --display-name "Auto-Unseal Token" \
  --no-default-policy
```

### 9. Protect Configuration Files

Ensure auto-unseal configuration is protected:

```bash
# Set restrictive permissions
chmod 600 /etc/secreton/secreton.toml
chown secreton:secreton /etc/secreton/secreton.toml

# For Kubernetes, use Secrets
kubectl create secret generic secreton-config \
  --from-file=secreton.toml \
  --namespace=secreton-system

# Mount as read-only
volumes:
- name: config
  secret:
    secretName: secreton-config
    defaultMode: 0400
```

### 10. Regular Security Audits

Perform regular audits:

```bash
# Check KMS key usage
aws kms get-key-rotation-status --key-id alias/secreton-auto-unseal

# Review IAM policies
aws iam get-role-policy --role-name secreton-auto-unseal-role --policy-name SecretonAutoUnsealPolicy

# Check audit logs
aws cloudtrail lookup-events \
  --lookup-attributes AttributeKey=ResourceName,AttributeValue=alias/secreton-auto-unseal \
  --max-results 50
```

---

## Migration from Manual Unseal

If you have an existing Secreton deployment using manual Shamir unseal, follow these steps to migrate to auto-unseal.

### Prerequisites

- Secreton is currently unsealed and operational
- You have access to the unseal keys
- Auto-unseal provider is configured and tested
- Backup of current configuration and data

### Migration Steps

#### Step 1: Backup Current State

```bash
# Backup Raft snapshot
secreton-cli backup create --output /backup/secreton-pre-autounseal.tar.gz

# Backup PostgreSQL database
pg_dump -h localhost -U secreton secreton > /backup/secreton-db-pre-autounseal.sql

# Backup configuration
cp /etc/secreton/secreton.toml /backup/secreton.toml.backup
```

#### Step 2: Configure Auto-Unseal

```bash
# Configure auto-unseal (don't restart yet)
secreton-cli auto-unseal configure --provider aws-kms

# Test configuration
secreton-cli auto-unseal test
```

#### Step 3: Migrate Master Key

This step re-encrypts the master key with the KMS provider:

```bash
# Migrate to auto-unseal
secreton-cli seal migrate-to-auto-unseal

# This will:
# 1. Retrieve current master key from memory
# 2. Encrypt it with the configured KMS provider
# 3. Store encrypted master key in database
# 4. Update seal configuration
```

Expected output:
```
🔄 Migrating to Auto-Unseal

   Current seal type: shamir
   Target seal type: auto-unseal (aws-kms)

   ⚠️  This operation will:
   1. Encrypt master key with AWS KMS
   2. Store encrypted key in database
   3. Update seal configuration
   4. Require restart to take effect

   Continue? [y/N]: y

   ✅ Master key encrypted with AWS KMS
   ✅ Encrypted key stored in database
   ✅ Seal configuration updated

   ⚠️  Important:
   1. Keep your Shamir unseal keys as backup
   2. Restart Secreton to activate auto-unseal
   3. Test auto-unseal after restart
```

#### Step 4: Restart Secreton

```bash
# Restart Secreton
systemctl restart secreton

# OR for Kubernetes
kubectl rollout restart statefulset/secreton -n secreton-system

# Wait for startup
sleep 30
```

#### Step 5: Verify Auto-Unseal

```bash
# Check seal status
secreton-cli status

# Expected output:
# Seal Status: unsealed
# Seal Type: auto-unseal (aws-kms)
# Initialized: true

# Verify auto-unseal status
secreton-cli auto-unseal status

# Check audit logs
secreton-cli audit list --filter "operation=auto-unseal" --limit 5
```

#### Step 6: Test Failover

Perform a controlled test to ensure auto-unseal works:

```bash
# Seal Secreton
secreton-cli seal seal

# Restart (should auto-unseal)
systemctl restart secreton

# Wait and verify
sleep 30
secreton-cli status
```

### Rollback Procedure

If auto-unseal fails, you can rollback to manual unseal:

#### Option 1: Use Fallback (if enabled)

If `fallback_enabled = true`, Secreton will automatically fall back to manual unseal:

```bash
# Unseal manually with Shamir keys
secreton-cli seal unseal
# Enter unseal keys when prompted
```

#### Option 2: Disable Auto-Unseal

```bash
# Disable auto-unseal
secreton-cli auto-unseal disable

# Restart Secreton
systemctl restart secreton

# Unseal manually
secreton-cli seal unseal
```

#### Option 3: Restore from Backup

If something goes wrong:

```bash
# Stop Secreton
systemctl stop secreton

# Restore configuration
cp /backup/secreton.toml.backup /etc/secreton/secreton.toml

# Restore database
psql -h localhost -U secreton secreton < /backup/secreton-db-pre-autounseal.sql

# Start Secreton
systemctl start secreton

# Unseal manually
secreton-cli seal unseal
```

### Post-Migration Checklist

- [ ] Auto-unseal working on restart
- [ ] Health endpoint shows auto-unseal status
- [ ] Audit logs show auto-unseal operations
- [ ] Fallback to manual unseal tested
- [ ] Monitoring alerts configured
- [ ] Documentation updated
- [ ] Team trained on new procedures
- [ ] Shamir keys stored securely as backup

---

## Appendix

### A. Quick Reference Commands

```bash
# Configure auto-unseal
secreton-cli auto-unseal configure --provider <provider>

# Check status
secreton-cli auto-unseal status

# Test configuration
secreton-cli auto-unseal test

# Disable auto-unseal
secreton-cli auto-unseal disable

# Migrate from manual unseal
secreton-cli seal migrate-to-auto-unseal

# Check health endpoint
curl http://localhost:8200/v1/sys/health | jq '.auto_unseal'
```

### B. Provider Comparison Matrix

| Feature | Transit | AWS KMS | GCP KMS | Azure Key Vault |
|---------|---------|---------|---------|-----------------|
| **Cost** | Free (self-hosted) | ~$1/month + API calls | ~$1/month + API calls | ~$1/month + operations |
| **Latency** | Low (internal) | Medium (AWS region) | Medium (GCP region) | Medium (Azure region) |
| **Availability** | Depends on Secreton | 99.99% SLA | 99.95% SLA | 99.9% SLA |
| **Key Rotation** | Manual | Automatic (yearly) | Automatic (configurable) | Manual |
| **HSM Support** | Optional (PKCS#11) | Yes (CloudHSM) | Yes (Cloud HSM) | Yes (Premium tier) |
| **Audit Logging** | Secreton audit | CloudTrail | Cloud Audit Logs | Azure Monitor |
| **Multi-Region** | Manual setup | Yes | Yes | Yes |
| **Air-Gapped** | Yes | No | No | No |
| **Compliance** | Self-managed | FIPS 140-2 Level 3 | FIPS 140-2 Level 3 | FIPS 140-2 Level 2 |

### C. Performance Benchmarks

Typical auto-unseal latency (p50/p95/p99):

| Provider | p50 | p95 | p99 | Notes |
|----------|-----|-----|-----|-------|
| **Transit** | 15ms | 30ms | 50ms | Same datacenter |
| **AWS KMS** | 50ms | 100ms | 200ms | Same region |
| **GCP KMS** | 45ms | 95ms | 180ms | Same region |
| **Azure Key Vault** | 60ms | 120ms | 250ms | Same region |

Startup time comparison:

| Seal Type | Startup Time | Notes |
|-----------|--------------|-------|
| **Manual Unseal** | 5-15 minutes | Requires 3 operators |
| **Auto-Unseal** | 10-30 seconds | Fully automated |

### D. Troubleshooting Checklist

When auto-unseal fails, check:

- [ ] Network connectivity to KMS provider
- [ ] DNS resolution for KMS endpoints
- [ ] Firewall rules allow outbound HTTPS/gRPC
- [ ] IAM/RBAC permissions are correct
- [ ] KMS key exists and is enabled
- [ ] Credentials are valid and not expired
- [ ] Configuration file syntax is correct
- [ ] Environment variables are set correctly
- [ ] Secreton logs for detailed error messages
- [ ] KMS provider status (no outages)
- [ ] Encrypted master key exists in database
- [ ] Fallback to manual unseal is enabled

### E. Support and Resources

**Documentation:**
- [Secreton AGENTS.md](../AGENTS.md) - Main developer guide
- [Seal/Unseal Security](../SEAL_UNSEAL_SECURITY_FIX_SUMMARY.md) - Security architecture
- [Production Readiness](../PRODUCTION_READINESS_REPORT.md) - Deployment guide

**Provider Documentation:**
- [AWS KMS Documentation](https://docs.aws.amazon.com/kms/)
- [GCP Cloud KMS Documentation](https://cloud.google.com/kms/docs)
- [Azure Key Vault Documentation](https://docs.microsoft.com/en-us/azure/key-vault/)

**Community:**
- GitHub Issues: [github.com/kejaksaan-ri/simpelv2/issues](https://github.com/kejaksaan-ri/simpelv2/issues)
- Internal Wiki: [wiki.kejaksaan.go.id/secreton](https://wiki.kejaksaan.go.id/secreton)

---

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-02-18 | Initial release |

---

**Document Status:** COMPLETE
**Last Updated:** 2026-02-18
**Maintained by:** SIMPelv2 DevOps Team
**Review Cycle:** Quarterly
