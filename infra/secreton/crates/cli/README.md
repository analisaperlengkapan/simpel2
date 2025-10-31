# Secreton CLI

Command-line interface for Secreton vault system.

## Installation

```bash
cargo install --path crates/cli
```

## Usage

### Seal/Unseal Operations

#### Initialize Vault

Initialize the vault and generate Shamir secret shares:

```bash
# Initialize with default settings (5 shares, 3 threshold)
secreton seal init

# Initialize with custom settings
secreton seal init --shares 7 --threshold 4

# Save keys to file
secreton seal init --shares 5 --threshold 3 --output keys.json
```

**Important**: Save the unseal keys securely! You will need the threshold number of keys to unseal the vault.

#### Seal Status

Check the current seal status of the vault:

```bash
secreton seal status
```

Output shows:
- Current state (sealed/unsealing/unsealed)
- Seal type (shamir)
- Total shares and threshold
- Unseal progress (if unsealing)

#### Seal Vault

Seal the vault (blocks all operations):

```bash
secreton seal seal
```

#### Unseal Vault

Unseal the vault with key shares:

```bash
# Interactive mode (prompts for key without echo)
secreton seal unseal

# Provide key directly
secreton seal unseal --key <base64-encoded-key>

# Reset unseal progress
secreton seal unseal --reset
```

You need to provide the threshold number of different keys to unseal the vault.

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
secreton transit decrypt my-app-key --data "vault:v1:..."

# From stdin
echo "vault:v1:..." | secreton transit decrypt my-app-key
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
# List backups in current directory
secreton backup list

# List backups in specific directory
secreton backup list --directory /backups

# Detailed listing
secreton backup list --directory /backups --detailed
```

For detailed restore documentation, see [RESTORE_GUIDE.md](RESTORE_GUIDE.md).

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

### Configuration File

```toml
server_url = "https://secreton.example.com"
timeout = 30
```

### Environment Variables

```bash
export SECRETON_SERVER_URL="https://secreton.example.com"
export SECRETON_TOKEN="your-token-here"
```

### Command-Line Flags

```bash
secreton --server https://secreton.example.com seal status
```

## Examples

### Complete Vault Setup Workflow

```bash
# 1. Initialize vault
secreton seal init --shares 5 --threshold 3 --output keys.json

# 2. Check status (should be unsealed after init)
secreton seal status

# 3. Create encryption key
secreton transit create-key app-key

# 4. Store a secret
secreton secret put app/config api_key=abc123

# 5. Seal the vault
secreton seal seal

# 6. Unseal with 3 keys
secreton seal unseal  # Enter key 1
secreton seal unseal  # Enter key 2
secreton seal unseal  # Enter key 3

# 7. Verify unsealed
secreton seal status
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
5. **Audit access** - Monitor who unseals the vault and when
6. **Limit access** - Only authorized operators should have unseal keys

## Troubleshooting

### Connection Refused

```bash
# Check server URL
secreton --server https://secreton.example.com status

# Verify server is running
curl https://secreton.example.com/health
```

### Invalid Unseal Key

- Verify the key is base64-encoded
- Check that you're using the correct key from initialization
- Ensure the key hasn't been corrupted

### Threshold Not Met

- You need to provide the threshold number of **different** keys
- Providing the same key multiple times won't work
- Check the threshold with `secreton seal status`

### Vault Already Sealed/Unsealed

- Check current status with `secreton seal status`
- Use `secreton seal unseal --reset` to reset unseal progress

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
