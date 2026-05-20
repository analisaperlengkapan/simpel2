# Secreton CLI

Command-line interface for Secreton engine system.

## Installation

```bash
cargo install --path crates/cli
```

## Usage

### Seal/Unseal Operations

#### Initialize Secret Vault

Initialize the engine and generate Shamir secret shares:

```bash
# Initialize with default settings (5 shares, 3 threshold)
secreton seal init

# Initialize with custom settings
secreton seal init --shares 7 --threshold 4

# Save keys to file
secreton seal init --shares 5 --threshold 3 --output keys.json
```

**Important**: Save the unseal keys securely! You will need the threshold number of keys to unseal the engine.

#### Seal Status

Check the current seal status of the engine:

```bash
secreton seal status
```

Output shows:

- Current state (sealed/unsealing/unsealed)
- Seal type (shamir)
- Total shares and threshold
- Unseal progress (if unsealing)

#### Seal Secret Vault

Seal the engine (blocks all operations):

```bash
secreton seal seal
```

#### Unseal Secret Vault

Unseal the engine with key shares:

```bash
# Interactive mode (prompts for key without echo)
secreton seal unseal

# Provide key directly
secreton seal unseal --key <base64-encoded-key>

# Reset unseal progress
secreton seal unseal --reset
```

You need to provide the threshold number of different keys to unseal the engine.

#### Rekey Operation

Change the number of shares and threshold:

```bash
# Start rekey operation
secreton seal rekey init --shares 7 --threshold 4

# Provide unseal keys (need threshold number)
secreton seal rekey update

# Check rekey progress
secreton seal rekey status

# Cancel rekey operation
secreton seal rekey cancel
```

### Transit Engine Operations

#### Create Encryption Key

```bash
secreton transit create-key my-app-key
```

#### List Keys

```bash
secreton transit list-keys
```

#### Encrypt Data

```bash
# From argument
secreton transit encrypt my-app-key --data "Hello World"

# From stdin
echo "Hello World" | secreton transit encrypt my-app-key
```

#### Decrypt Data

```bash
# From argument
secreton transit decrypt my-app-key --data "engine:v1:..."

# From stdin
echo "engine:v1:..." | secreton transit decrypt my-app-key
```

### KV Secrets Engine Operations

#### Store Secret

```bash
secreton secret put database/prod username=admin password=secret123
```

#### Retrieve Secret

```bash
secreton secret get database/prod
```

#### List Secrets

```bash
secreton secret list
```

#### Delete Secret

```bash
secreton secret delete database/prod
```

### Backup and Restore Operations

#### Create Backup

```bash
# Full backup with default settings
secreton backup create --output backup.bak

# Backup with custom compression
secreton backup create --output backup.bak --compression 9

# Incremental backup
secreton backup create --output incremental.bak --incremental --base-backup full-backup.bak

# Backup without audit logs
secreton backup create --output secrets-only.bak --include-audit false
```

#### Restore from Backup

```bash
# Full restore
secreton backup restore --file backup.bak

# Dry run (verify without applying)
secreton backup restore --file backup.bak --dry-run

# Point-in-time restore
secreton backup restore --file backup.bak --point-in-time "2025-10-29T10:00:00Z"

# Restore only secrets
secreton backup restore --file backup.bak --secrets-only

# Force overwrite existing secrets
secreton backup restore --file backup.bak --force

# Restore to different namespace
secreton backup restore --file backup.bak --target-namespace production
```

#### Verify Backup

```bash
# Verify backup integrity
secreton backup verify --file backup.bak

# Verbose verification
secreton backup verify --file backup.bak --verbose
```

#### List Backups

```bash
# List all backups in directory
secreton backup list --directory /backups

# List with detailed information
secreton backup list --directory /backups --verbose
```

### Replication Management

Secreton supports performance and disaster recovery (DR) replication for high availability.

#### Enable Replication

```bash
# Enable as primary with secondaries
secreton replication enable \
  --mode performance \
  --secondaries https://secondary1:8200 \
  --secondaries https://secondary2:8200

# Enable as secondary
secreton replication enable \
  --mode performance \
  --primary https://primary:8200

# Enable DR replication
secreton replication enable \
  --mode dr \
  --primary https://primary:8200
```

#### Check Replication Status

```bash
# Show status in table format
secreton replication status

# Show status in JSON format
secreton replication status --format json

# Show status in YAML format
secreton replication status --format yaml
```

#### Monitor Replication Lag

```bash
# Show lag metrics
secreton replication lag

# Show lag in JSON format
secreton replication lag --format json
```

#### Promote Secondary to Primary

```bash
# Promote with confirmation
secreton replication promote

# Promote without confirmation
secreton replication promote --yes
```

#### Manage Secondary Nodes

```bash
# Add a secondary node
secreton replication add-secondary https://secondary3:8200

# Remove a secondary node
secreton replication remove-secondary node-abc123
```

#### Disable Replication

```bash
# Disable replication
secreton replication disable

# Force disable
secreton replication disable --force
```

For detailed replication documentation, see [Replication CLI Guide](docs/REPLICATION_CLI.md).

#### List Backups

```bash
# List backups in current directory
secreton backup list

# List backups in specific directory
secreton backup list --directory /backups

# Detailed listing
secreton backup list --directory /backups --detailed
```

For detailed restore documentation, see [RESTORE_GUIDE.md](RESTORE_GUIDE.md).

### Authentication Operations

#### Login

Authenticate to the Secreton engine and store a token locally:

```bash
# Login with username/password (interactive)
secreton login

# Login with username/password (non-interactive)
secreton login --username admin

# Login with an existing token
secreton login --method token --token stn.your_token_here
```

The CLI will securely prompt for your password without echoing it to the terminal.

#### Logout

Clear the stored authentication token:

```bash
secreton logout
```

### Policy Management Operations

#### List Policies

Display all available policies:

```bash
# List all policies
secreton policy list

# List policies in a specific namespace
secreton policy list --namespace production

# Output as JSON
secreton policy list --format json
```

#### Read Policy

View the contents of a specific policy:

```bash
# Read policy in TOML format (default)
secreton policy read my-policy

# Read policy in JSON format
secreton policy read my-policy --format json
```

#### Write Policy

Create or update a policy from a file:

```bash
# Write policy from TOML file
secreton policy write my-policy policy.toml

# The policy file should be in TOML format (see examples below)
```

#### Delete Policy

Remove a policy:

```bash
# Delete with confirmation prompt
secreton policy delete my-policy

# Delete without confirmation
secreton policy delete my-policy --force
```

#### Format Policy File

Format a policy file according to standard conventions:

```bash
# Format a policy file (modifies in place)
secreton policy fmt policy.toml

# Check if formatting is needed without modifying
secreton policy fmt policy.toml --check

# Format to JSON
secreton policy fmt policy.toml --format json
```

#### Validate Policy File

Check policy syntax without applying it:

```bash
secreton policy validate policy.toml
```

#### Test Policy

Test if a policy would allow a specific operation:

```bash
secreton policy test my-policy --path secret/data/database --action read
```

### Token Management Operations

#### Create Token

Generate a new authentication token:

```bash
# Create token with specific policies
secreton token create --policies default,admin

# Create token with TTL
secreton token create --policies default --ttl 1h

# Create non-renewable token
secreton token create --policies default --renewable false

# Create token with display name
secreton token create --policies default --display-name "CI/CD Pipeline"
```

#### Lookup Token

View information about a token:

```bash
# Lookup current token
secreton token lookup

# Lookup specific token
secreton token lookup stn.specific_token_here
```

#### Renew Token

Extend the TTL of the current token:

```bash
# Renew with default increment
secreton token renew

# Renew with specific increment
secreton token renew --increment 2h
```

#### Revoke Token

Invalidate a token:

```bash
secreton token revoke stn.token_to_revoke
```

#### Token Capabilities

Check what operations a token can perform on a path:

```bash
secreton token capabilities secret/data/database
```

### Operator Diagnostic Commands

#### Diagnose

Run comprehensive diagnostics on engine connectivity and status:

```bash
secreton operator diagnose
```

This command checks:

- Connectivity to the engine server
- Secret Vault initialization status
- Secret Vault seal status
- Authentication status and token validity

### Audit Log Commands

#### List Audit Logs

View audit logs with optional filtering:

```bash
# List recent audit logs (default: 50 entries)
secreton audit list

# Filter by user
secreton audit list --user admin@example.com

# Filter by operation
secreton audit list --operation secret.read

# Filter by resource path
secreton audit list --path secret/data/database

# Filter by time range
secreton audit list --start-time 2025-12-01T00:00:00Z --end-time 2025-12-31T23:59:59Z

# Combine filters
secreton audit list --user admin --operation policy.write --limit 100

# Output as JSON
secreton audit list --format json

# Output as CSV
secreton audit list --format csv
```

### System Operations

#### Health Check

```bash
secreton status
```

## Configuration

The CLI can be configured via:

1. Configuration file (`~/.secreton/config.toml`)
2. Environment variables
3. Command-line flags

Configuration precedence (highest to lowest):

- Command-line flags
- Environment variables
- Configuration file
- Default values

### Configuration File

The configuration file is stored at `~/.secreton/config.toml`:

```toml
server_url = "https://secreton.example.com:8200"
default_namespace = "default"
```

### Configuration Commands

```bash
# Set server URL
secreton config set server https://secreton.example.com:8200

# Set default namespace
secreton config set namespace production

# Get server URL
secreton config get server

# Show all configuration
secreton config show
```

### Environment Variables

```bash
# Server address
export SECRETON_ADDR="https://secreton.example.com:8200"

# Default namespace
export SECRETON_NAMESPACE="production"
```

### Command-Line Flags

```bash
# Override server URL for a single command
secreton --server https://secreton.example.com:8200 seal status

# Override namespace for a single command
secreton --namespace production policy list

# Enable verbose logging
secreton --verbose seal status
```

## Policy File Format

Policies are defined in TOML format. Here's the structure:

```toml
# Policy metadata
name = "my-policy"
description = "Description of what this policy allows"
namespace = "default"

# Policy rules
[[rules]]
effect = "allow"  # or "deny"
path = "secret/data/database/*"
capabilities = ["read", "list"]

[[rules]]
effect = "allow"
path = "secret/data/database/credentials"
capabilities = ["read"]
mfa = true  # Require MFA for this operation

# Optional conditions
[rules.condition]
time_range = { start = "2025-01-01T00:00:00Z", end = "2025-12-31T23:59:59Z" }
allowed_ips = ["192.168.1.0/24", "10.0.0.0/8"]
```

### Example Policies

Example policy files are available in `layanan/secreton/examples/policies/`:

- `read-only.toml` - Read-only access to secrets
- `admin.toml` - Full administrative access
- `database-secrets.toml` - Scoped access to database credentials
- `transit-only.toml` - Encryption/decryption only access
- `namespace-scoped.toml` - Namespace-isolated access

## Examples

### Complete Secret Vault Setup Workflow

```bash
# 1. Initialize engine
secreton seal init --shares 5 --threshold 3 --output keys.json

# 2. Check status (should be unsealed after init)
secreton seal status

# 3. Login with root token (from initialization)
secreton login --method token --token <root-token-from-init>

# 4. Create a policy for application access
cat > app-policy.toml <<EOF
name = "app-access"
description = "Application access to secrets"
namespace = "default"

[[rules]]
effect = "allow"
path = "secret/data/app/*"
capabilities = ["read", "list"]

[[rules]]
effect = "allow"
path = "transit/encrypt/app-key"
capabilities = ["update"]

[[rules]]
effect = "allow"
path = "transit/decrypt/app-key"
capabilities = ["update"]
EOF

secreton policy write app-access app-policy.toml

# 5. Create a token for the application
secreton token create --policies app-access --ttl 24h --display-name "Production App"

# 6. Create encryption key
secreton transit create-key app-key

# 7. Store application secrets
secreton secret put app/config api_key=abc123 db_password=secret

# 8. Test the policy
secreton policy test app-access --path secret/data/app/config --action read

# 9. Verify audit logs
secreton audit list --limit 10
```

### Daily Operations Workflow

```bash
# 1. Check engine status
secreton operator diagnose

# 2. Login (if needed)
secreton login

# 3. List available secrets
secreton secret list

# 4. Retrieve a secret
secreton secret get app/config

# 5. Encrypt sensitive data
echo "sensitive data" | secreton transit encrypt app-key

# 6. Decrypt data
secreton transit decrypt app-key --data "engine:v1:..."

# 7. Check token expiration
secreton token lookup

# 8. Renew token if needed
secreton token renew --increment 1h
```

### Policy Management Workflow

```bash
# 1. List all policies
secreton policy list

# 2. Read an existing policy
secreton policy read default

# 3. Create a new policy file
cat > database-admin.toml <<EOF
name = "database-admin"
description = "Full access to database secrets"
namespace = "default"

[[rules]]
effect = "allow"
path = "secret/data/database/*"
capabilities = ["create", "read", "update", "delete", "list"]
EOF

# 4. Validate the policy
secreton policy validate database-admin.toml

# 5. Format the policy
secreton policy fmt database-admin.toml

# 6. Write the policy to engine
secreton policy write database-admin database-admin.toml

# 7. Test the policy
secreton policy test database-admin --path secret/data/database/prod --action update

# 8. Create a token with the policy
secreton token create --policies database-admin --ttl 8h
```

### Backup and Restore Workflow

```bash
# 1. Create a full backup
secreton backup create --output backup-$(date +%Y%m%d).bak

# 2. Verify backup integrity
secreton backup verify --file backup-20251202.bak

# 3. List available backups
secreton backup list

# 4. Restore from backup (dry run first)
secreton backup restore --file backup-20251202.bak --dry-run

# 5. Perform actual restore
secreton backup restore --file backup-20251202.bak
```

### Seal/Unseal Workflow

```bash
# 1. Check seal status
secreton seal status

# 2. Seal the engine (for maintenance)
secreton seal seal

# 3. Unseal with threshold number of keys
secreton seal unseal  # Enter key 1
secreton seal unseal  # Enter key 2
secreton seal unseal  # Enter key 3

# 4. Verify unsealed
secreton seal status

# 5. If needed, reset unseal progress
secreton seal unseal --reset
```

### Secure Key Input

The CLI supports secure key input without echoing to the terminal:

```bash
# Prompts for key without displaying it
secreton seal unseal

# For automation, use environment variable or file
export UNSEAL_KEY="base64-encoded-key"
secreton seal unseal --key "$UNSEAL_KEY"
```

## Security Best Practices

1. **Never store all unseal keys together** - Distribute them to different trusted operators
2. **Use secure channels** - Transfer keys via secure, encrypted channels
3. **Backup keys securely** - Store backups in secure, offline locations
4. **Rotate regularly** - Use the rekey operation to rotate keys periodically
5. **Audit access** - Monitor who unseals the engine and when
6. **Limit access** - Only authorized operators should have unseal keys

## Troubleshooting

### Connection Refused

```bash
# Run diagnostics to check connectivity
secreton operator diagnose

# Check server URL
secreton --server https://secreton.example.com status

# Verify server is running
curl https://secreton.example.com/health
```

### Authentication Issues

```bash
# Check if you're authenticated
secreton token lookup

# If token expired, login again
secreton login

# Verify token with server
secreton operator diagnose
```

### Permission Denied

```bash
# Check your token's capabilities for a path
secreton token capabilities secret/data/myapp

# View your token's policies
secreton token lookup

# Test if a policy allows an operation
secreton policy test my-policy --path secret/data/myapp --action read
```

### Invalid Unseal Key

- Verify the key is base64-encoded
- Check that you're using the correct key from initialization
- Ensure the key hasn't been corrupted

### Threshold Not Met

- You need to provide the threshold number of **different** keys
- Providing the same key multiple times won't work
- Check the threshold with `secreton seal status`

### Secret Vault Already Sealed/Unsealed

- Check current status with `secreton seal status`
- Use `secreton seal unseal --reset` to reset unseal progress

### Policy Validation Errors

```bash
# Validate policy syntax
secreton policy validate my-policy.toml

# Format policy file
secreton policy fmt my-policy.toml

# Check example policies for reference
ls layanan/secreton/examples/policies/
```

### Token Expired

```bash
# Check token status
secreton token lookup

# Renew if renewable
secreton token renew

# Otherwise, login again
secreton login
```

### Audit Log Access Denied

- Ensure your token has audit read permissions
- Check with: `secreton token capabilities /v1/audit/logs`
- Contact your engine administrator for audit access

## Development

### Building

```bash
cargo build --package secreton-cli
```

### Testing

```bash
cargo test --package secreton-cli
```

### Running Locally

```bash
cargo run --package secreton-cli -- seal status
```

## License

See LICENSE file in the repository root.
