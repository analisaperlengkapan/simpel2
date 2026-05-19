# Secreton Terraform Provider Integration

This document describes how to integrate Secreton with Terraform for infrastructure-as-code secret management.

## Overview

The Secreton Terraform provider enables you to:

- Read secrets from Secreton as Terraform data sources
- Manage secrets, policies, and authentication methods as Terraform resources
- Inject secrets into other Terraform resources securely

## Architecture

```
┌─────────────────┐
│   Terraform     │
│   Configuration │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Secreton      │
│   Provider      │
│   (Go)          │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Secreton      │
│   REST API      │
│   (Rust)        │
└─────────────────┘
```

## Provider Configuration

### Basic Configuration

```hcl
terraform {
  required_providers {
    secreton = {
      source  = "cipherce/secreton"
      version = "~> 1.0"
    }
  }
}

provider "secreton" {
  address = "https://secreton.internal:8200"
  token   = var.secreton_token

  # Optional: Namespace
  namespace = "production"

  # Optional: TLS configuration
  ca_cert_file = "/path/to/ca.pem"
  skip_tls_verify = false
}
```

### Environment Variables

The provider supports the following environment variables:

- `SECRETON_ADDR`: Secreton server address
- `SECRETON_TOKEN`: Authentication token
- `SECRETON_NAMESPACE`: Default namespace
- `SECRETON_CACERT`: Path to CA certificate
- `SECRETON_SKIP_VERIFY`: Skip TLS verification (not recommended)

## Data Sources

### secreton_kv_secret_v2

Read a secret from the KV v2 secrets engine:

```hcl
data "secreton_kv_secret_v2" "database" {
  mount = "secret"
  name  = "database/prod"
}

# Use the secret in other resources
resource "aws_db_instance" "main" {
  username = data.secreton_kv_secret_v2.database.data["username"]
  password = data.secreton_kv_secret_v2.database.data["password"]

  # ... other configuration
}
```

**API Endpoint**: `GET /v1/{mount}/data/{name}`

**Response Format**:

```json
{
  "data": {
    "data": {
      "username": "admin",
      "password": "secret123"
    },
    "metadata": {
      "created_time": "2024-01-01T00:00:00Z",
      "version": 1
    }
  }
}
```

### secreton_transit_encrypt

Encrypt data using the Transit engine:

```hcl
data "secreton_transit_encrypt" "sensitive_data" {
  backend   = "transit"
  key       = "my-key"
  plaintext = base64encode("sensitive information")
}

output "encrypted_data" {
  value = data.secreton_transit_encrypt.sensitive_data.ciphertext
}
```

**API Endpoint**: `POST /v1/transit/encrypt/{key}`

**Request**:

```json
{
  "plaintext": "base64-encoded-data"
}
```

**Response**:

```json
{
  "data": {
    "ciphertext": "engine:v1:encrypted-data"
  }
}
```

### secreton_policy_document

Generate a Secreton policy document:

```hcl
data "secreton_policy_document" "app_policy" {
  rule {
    path         = "secret/data/myapp/*"
    capabilities = ["read", "list"]
  }

  rule {
    path         = "transit/encrypt/myapp-key"
    capabilities = ["update"]
  }
}

resource "secreton_policy" "app" {
  name   = "myapp-policy"
  policy = data.secreton_policy_document.app_policy.hcl
}
```

## Resources

### secreton_kv_secret_v2

Manage a KV v2 secret:

```hcl
resource "secreton_kv_secret_v2" "database" {
  mount = "secret"
  name  = "database/prod"

  data_json = jsonencode({
    username = "admin"
    password = random_password.db_password.result
  })

  # Optional: Custom metadata
  custom_metadata = {
    environment = "production"
    owner       = "platform-team"
  }
}
```

**API Endpoints**:

- Create/Update: `POST /v1/{mount}/data/{name}`
- Read: `GET /v1/{mount}/data/{name}`
- Delete: `DELETE /v1/{mount}/data/{name}`

### secreton_policy

Manage a policy:

```hcl
resource "secreton_policy" "app" {
  name = "myapp-policy"

  policy = <<EOT
path "secret/data/myapp/*" {
  capabilities = ["read", "list"]
}

path "transit/encrypt/myapp-key" {
  capabilities = ["update"]
}
EOT
}
```

**API Endpoints**:

- Create/Update: `PUT /v1/sys/policies/acl/{name}`
- Read: `GET /v1/sys/policies/acl/{name}`
- Delete: `DELETE /v1/sys/policies/acl/{name}`

### secreton_auth_backend

Enable an authentication backend:

```hcl
resource "secreton_auth_backend" "kubernetes" {
  type = "kubernetes"
  path = "kubernetes"

  description = "Kubernetes authentication"
}
```

**API Endpoints**:

- Enable: `POST /v1/sys/auth/{path}`
- Read: `GET /v1/sys/auth/{path}`
- Disable: `DELETE /v1/sys/auth/{path}`

### secreton_transit_key

Manage a Transit encryption key:

```hcl
resource "secreton_transit_key" "app_key" {
  backend = "transit"
  name    = "myapp-key"

  type                 = "aes256-gcm96"
  deletion_allowed     = false
  exportable           = false
  allow_plaintext_backup = false
}
```

**API Endpoints**:

- Create: `POST /v1/transit/keys/{name}`
- Read: `GET /v1/transit/keys/{name}`
- Delete: `DELETE /v1/transit/keys/{name}`

## Complete Example

### Application Infrastructure with Secrets

```hcl
terraform {
  required_providers {
    secreton = {
      source  = "cipherce/secreton"
      version = "~> 1.0"
    }
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "secreton" {
  address = "https://secreton.internal:8200"
  token   = var.secreton_token
}

# Generate a random password
resource "random_password" "db_password" {
  length  = 32
  special = true
}

# Store the password in Secreton
resource "secreton_kv_secret_v2" "database" {
  mount = "secret"
  name  = "database/prod"

  data_json = jsonencode({
    username = "app_user"
    password = random_password.db_password.result
    host     = aws_db_instance.main.endpoint
    port     = 5432
    database = "myapp"
  })
}

# Create a policy for the application
resource "secreton_policy" "app" {
  name = "myapp-policy"

  policy = <<EOT
path "secret/data/database/prod" {
  capabilities = ["read"]
}

path "transit/encrypt/myapp-key" {
  capabilities = ["update"]
}

path "transit/decrypt/myapp-key" {
  capabilities = ["update"]
}
EOT
}

# Enable Kubernetes auth
resource "secreton_auth_backend" "kubernetes" {
  type = "kubernetes"
  path = "kubernetes"
}

# Configure Kubernetes auth
resource "secreton_kubernetes_auth_backend_config" "config" {
  backend            = secreton_auth_backend.kubernetes.path
  kubernetes_host    = "https://kubernetes.default.svc"
  kubernetes_ca_cert = file("/var/run/secrets/kubernetes.io/serviceaccount/ca.crt")
}

# Create a Kubernetes auth role
resource "secreton_kubernetes_auth_backend_role" "app" {
  backend                          = secreton_auth_backend.kubernetes.path
  role_name                        = "myapp"
  bound_service_account_names      = ["myapp"]
  bound_service_account_namespaces = ["production"]
  token_policies                   = [secreton_policy.app.name]
  token_ttl                        = 3600
}

# Create Transit key for encryption
resource "secreton_transit_key" "app_key" {
  backend = "transit"
  name    = "myapp-key"
  type    = "aes256-gcm96"
}

# Output the secret path for application configuration
output "secret_path" {
  value = "secret/data/database/prod"
}

output "transit_key" {
  value = secreton_transit_key.app_key.name
}
```

### Using Secrets in Application Deployment

```hcl
# Read secrets for use in Kubernetes deployment
data "secreton_kv_secret_v2" "app_config" {
  mount = "secret"
  name  = "myapp/config"
}

resource "kubernetes_deployment" "app" {
  metadata {
    name      = "myapp"
    namespace = "production"
  }

  spec {
    template {
      spec {
        service_account_name = "myapp"

        container {
          name  = "app"
          image = "myapp:latest"

          env {
            name  = "SECRETON_ADDR"
            value = "https://secreton.internal:8200"
          }

          env {
            name  = "SECRETON_ROLE"
            value = "myapp"
          }

          env {
            name  = "SECRET_PATH"
            value = "secret/data/database/prod"
          }

          # Application will authenticate using Kubernetes auth
          # and fetch secrets at runtime
        }
      }
    }
  }
}
```

## API Endpoints Reference

### KV Secrets Engine

| Operation | Method | Endpoint | Description |
|-----------|--------|----------|-------------|
| Read Secret | GET | `/v1/{mount}/data/{path}` | Read a secret |
| Write Secret | POST | `/v1/{mount}/data/{path}` | Create/update a secret |
| Delete Secret | DELETE | `/v1/{mount}/data/{path}` | Delete a secret |
| List Secrets | LIST | `/v1/{mount}/metadata/{path}` | List secrets at path |
| Read Metadata | GET | `/v1/{mount}/metadata/{path}` | Read secret metadata |

### Transit Engine

| Operation | Method | Endpoint | Description |
|-----------|--------|----------|-------------|
| Create Key | POST | `/v1/transit/keys/{name}` | Create encryption key |
| Read Key | GET | `/v1/transit/keys/{name}` | Read key configuration |
| Encrypt | POST | `/v1/transit/encrypt/{name}` | Encrypt data |
| Decrypt | POST | `/v1/transit/decrypt/{name}` | Decrypt data |
| Rotate Key | POST | `/v1/transit/keys/{name}/rotate` | Rotate encryption key |

### Policy Management

| Operation | Method | Endpoint | Description |
|-----------|--------|----------|-------------|
| Create/Update | PUT | `/v1/sys/policies/acl/{name}` | Create or update policy |
| Read | GET | `/v1/sys/policies/acl/{name}` | Read policy |
| Delete | DELETE | `/v1/sys/policies/acl/{name}` | Delete policy |
| List | LIST | `/v1/sys/policies/acl` | List all policies |

### Authentication

| Operation | Method | Endpoint | Description |
|-----------|--------|----------|-------------|
| Enable Auth | POST | `/v1/sys/auth/{path}` | Enable auth method |
| Read Auth | GET | `/v1/sys/auth/{path}` | Read auth configuration |
| Disable Auth | DELETE | `/v1/sys/auth/{path}` | Disable auth method |
| List Auth | GET | `/v1/sys/auth` | List auth methods |

## Best Practices

### 1. Use Data Sources for Read-Only Access

Prefer data sources over resources when you only need to read secrets:

```hcl
# Good: Use data source for reading
data "secreton_kv_secret_v2" "config" {
  mount = "secret"
  name  = "app/config"
}

# Avoid: Don't use resource for read-only access
```

### 2. Store Sensitive Values Securely

Never hardcode sensitive values in Terraform configurations:

```hcl
# Good: Use variables
variable "secreton_token" {
  type      = string
  sensitive = true
}

# Bad: Hardcoded token
provider "secreton" {
  token = "s.abc123..."  # Don't do this!
}
```

### 3. Use Remote State with Encryption

Store Terraform state in a secure backend with encryption:

```hcl
terraform {
  backend "s3" {
    bucket         = "terraform-state"
    key            = "prod/terraform.tfstate"
    encrypt        = true
    dynamodb_table = "terraform-locks"
  }
}
```

### 4. Implement Least Privilege

Create specific policies for each application:

```hcl
resource "secreton_policy" "app" {
  name = "myapp-policy"

  policy = <<EOT
# Only allow reading specific secrets
path "secret/data/myapp/*" {
  capabilities = ["read"]
}

# Deny everything else
path "*" {
  capabilities = ["deny"]
}
EOT
}
```

### 5. Use Namespaces for Isolation

Leverage Secreton namespaces for multi-tenancy:

```hcl
provider "secreton" {
  address   = "https://secreton.internal:8200"
  token     = var.secreton_token
  namespace = "team-a"
}
```

## Troubleshooting

### Authentication Errors

```
Error: failed to read secret: permission denied
```

**Solution**: Verify the token has the necessary permissions:

```bash
# Check token capabilities
curl -H "X-Secret Vault-Token: $TOKEN" \
  https://secreton.internal:8200/v1/sys/capabilities-self \
  -d '{"paths": ["secret/data/myapp/config"]}'
```

### Connection Errors

```
Error: failed to connect to Secreton: connection refused
```

**Solution**: Verify network connectivity and Secreton address:

```bash
# Test connectivity
curl -k https://secreton.internal:8200/v1/sys/health
```

### TLS Certificate Errors

```
Error: x509: certificate signed by unknown authority
```

**Solution**: Provide the CA certificate:

```hcl
provider "secreton" {
  address      = "https://secreton.internal:8200"
  ca_cert_file = "/path/to/ca.pem"
}
```

## Requirements

Validates: Requirements 5.2

## Implementation Notes

The Terraform provider should be implemented in Go using the Terraform Plugin SDK. The provider will interact with Secreton's REST API endpoints documented above.

### Provider Repository Structure

```
terraform-provider-secreton/
├── main.go
├── provider/
│   ├── provider.go
│   ├── data_source_kv_secret.go
│   ├── resource_kv_secret.go
│   ├── resource_policy.go
│   └── ...
├── client/
│   └── client.go
├── docs/
│   └── ...
└── examples/
    └── ...
```

### Client Implementation

The Go client should implement:

- HTTP client with retry logic
- Token-based authentication
- TLS configuration
- Request/response handling
- Error handling

## References

- [Terraform Plugin SDK](https://github.com/hashicorp/terraform-plugin-sdk)
- [Terraform Provider Development](https://developer.hashicorp.com/terraform/plugin)
- [HashiCorp Secret Vault Terraform Provider](https://registry.terraform.io/providers/hashicorp/engine/latest/docs) (reference implementation)
