# Authenc Hybrid Configuration System

## Overview

Authenc implements a production-grade hybrid configuration system that combines:

1. **Minimal Bootstrap Config** (authenc.toml)
   - Only database and Secreton connection details
   - Immutable during runtime
   - Committed to version control

2. **Centralized Database Config** (authenc.configuration table)
   - All application configuration
   - Hot reload without restart
   - Complete audit trail
   - Scalable across instances

3. **Secrets Management** (Secreton)
   - All sensitive data (keys, passwords)
   - Encrypted storage
   - Key rotation
   - Access control

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Authenc Startup                       │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│  1. Load Bootstrap Config (authenc.toml)                │
│     - Database connection                               │
│     - Secreton URL & token                              │
│     - Server port                                       │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│  2. Connect to Database                                 │
│     - Establish connection pool                         │
│     - Run migrations                                    │
│     - Initialize tables                                 │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│  3. Load Full Config from Database                      │
│     - Query authenc.configuration table                 │
│     - Cache in memory (TTL: 5 minutes)                  │
│     - Apply environment variable overrides              │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│  4. Load Secrets from Secreton                          │
│     - JWT_SECRET                                        │
│     - Signing keys (Ed25519, ECDSA)                     │
│     - SMTP password                                     │
│     - Encryption keys                                   │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│  5. Start Service                                       │
│     - All configuration loaded                          │
│     - Secrets available                                 │
│     - Ready for requests                                │
└─────────────────────────────────────────────────────────┘
```

## Configuration Hierarchy

```
Environment Variables (highest priority)
    ↓
Secreton Secrets (for sensitive data)
    ↓
Database Configuration (authenc.configuration)
    ↓
Bootstrap TOML (authenc.toml)
    ↓
Default Values (lowest priority)
```

## Bootstrap Configuration (authenc.toml)

Minimal file containing only:

```toml
[server]
host = "0.0.0.0"
port = 8088

[database]
host = "postgres"
port = 5432
database = "authenc"

[secreton]
enabled = true
url = "${SECRETON_API_URL}"
token = "${SECRETON_TOKEN}"

[config_loader]
enabled = true
load_from_database = true
cache_ttl = 300
allow_hot_reload = true
```

## Database Configuration Table

```sql
CREATE TABLE authenc.configuration (
    id UUID PRIMARY KEY,
    key VARCHAR(255) UNIQUE NOT NULL,
    value JSONB NOT NULL,
    description TEXT,
    is_secret BOOLEAN DEFAULT FALSE,
    is_system BOOLEAN DEFAULT FALSE,
    category VARCHAR(100),
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    updated_by UUID,
    version INT
);
```

### Configuration Categories

- **server**: Server settings (host, port, TLS)
- **database**: Database settings
- **security**: Security policies (JWT, passwords, brute force)
- **rate_limit**: Rate limiting configuration
- **captcha**: CAPTCHA system settings
- **features**: Feature flags
- **observability**: Logging and metrics
- **email**: Email/SMTP settings
- **integrations**: Third-party integrations

### Example Configuration Values

```sql
-- Server configuration
INSERT INTO authenc.configuration (key, value, category) VALUES
    ('server.port', '8088'::jsonb, 'server'),
    ('server.tls_enabled', 'false'::jsonb, 'server');

-- Security configuration
INSERT INTO authenc.configuration (key, value, category) VALUES
    ('security.jwt_expiry', '3600'::jsonb, 'security'),
    ('security.password_min_length', '12'::jsonb, 'security');

-- CAPTCHA configuration
INSERT INTO authenc.configuration (key, value, category) VALUES
    ('captcha.enabled', 'true'::jsonb, 'captcha'),
    ('captcha.default_difficulty', '3'::jsonb, 'captcha');
```

## Secrets Management (Secreton)

Sensitive data stored in Secreton:

```bash
# Store secrets in Secreton
vault kv put secret/authenc/jwt_secret value="your_jwt_secret"
vault kv put secret/authenc/ed25519_private_key value="base64_encoded_key"
vault kv put secret/authenc/smtp_password value="smtp_password"
```

### Secrets to Load

```toml
[secreton]
secrets_to_load = [
    "jwt_secret",
    "ed25519_private_key",
    "ecdsa_p256_private_key",
    "ecdsa_p384_private_key",
    "ecdsa_p521_private_key",
    "smtp_password",
    "encryption_key"
]
```

## Configuration Management API

### Get All Configuration

```bash
curl -X GET http://localhost:8088/admin/config \
  -H "Authorization: Bearer $TOKEN"
```

Response:
```json
{
  "status": "success",
  "data": {
    "server.port": 8088,
    "captcha.enabled": true,
    "security.jwt_expiry": 3600
  }
}
```

### Get Configuration by Category

```bash
curl -X GET http://localhost:8088/admin/config/category/captcha \
  -H "Authorization: Bearer $TOKEN"
```

### Get Specific Configuration

```bash
curl -X GET http://localhost:8088/admin/config/captcha.default_difficulty \
  -H "Authorization: Bearer $TOKEN"
```

### Update Configuration

```bash
curl -X PUT http://localhost:8088/admin/config/captcha.default_difficulty \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "value": 5,
    "reason": "Increased difficulty for production"
  }'
```

### Delete Configuration

```bash
curl -X DELETE http://localhost:8088/admin/config/custom_setting \
  -H "Authorization: Bearer $TOKEN"
```

### Hot Reload Configuration

```bash
curl -X POST http://localhost:8088/admin/config/reload \
  -H "Authorization: Bearer $TOKEN"
```

### View Change History

```bash
curl -X GET http://localhost:8088/admin/config/captcha.default_difficulty/history \
  -H "Authorization: Bearer $TOKEN"
```

Response:
```json
{
  "status": "success",
  "key": "captcha.default_difficulty",
  "history": [
    {
      "key": "captcha.default_difficulty",
      "old_value": 3,
      "new_value": 5,
      "reason": "Increased difficulty for production",
      "created_at": "2024-11-28T10:30:00Z",
      "changed_by": "admin"
    }
  ]
}
```

## Configuration Change Audit Trail

All configuration changes are recorded in `authenc.configuration_history`:

```sql
CREATE TABLE authenc.configuration_history (
    id UUID PRIMARY KEY,
    config_id UUID NOT NULL,
    key VARCHAR(255) NOT NULL,
    old_value JSONB,
    new_value JSONB,
    changed_by UUID,
    change_reason TEXT,
    created_at TIMESTAMP
);
```

### Query Change History

```sql
-- View all changes to a configuration
SELECT * FROM authenc.configuration_history
WHERE key = 'captcha.default_difficulty'
ORDER BY created_at DESC;

-- View changes by user
SELECT * FROM authenc.configuration_history
WHERE changed_by = 'user_id'
ORDER BY created_at DESC;

-- View recent changes
SELECT * FROM authenc.configuration_history
WHERE created_at > NOW() - INTERVAL '24 hours'
ORDER BY created_at DESC;
```

## Caching Strategy

Configuration is cached in memory with:

- **TTL**: 5 minutes (configurable)
- **Invalidation**: Automatic on update
- **Hot reload**: Manual via API

### Cache Behavior

```
Request for config.key
    ↓
Check memory cache
    ↓
Cache hit? → Return cached value
    ↓
Cache miss? → Load from database
    ↓
Cache value with TTL
    ↓
Return value
```

## Environment Variable Overrides

Environment variables override database configuration:

```bash
# Override via environment variable
export CAPTCHA_DEFAULT_DIFFICULTY=5

# This takes precedence over database value
```

## Production Deployment Checklist

- [ ] Bootstrap authenc.toml configured
- [ ] Database connection tested
- [ ] Secreton integration verified
- [ ] Default configuration loaded
- [ ] All secrets in Secreton
- [ ] Admin user created
- [ ] config-admin role assigned
- [ ] Configuration API tested
- [ ] Audit logging enabled
- [ ] Backups configured

## Advantages of Hybrid Approach

### Security
✅ Secrets stored securely in Secreton
✅ Configuration changes audited
✅ No secrets in files
✅ Role-based access control

### Scalability
✅ Multiple instances share config
✅ No per-instance configuration
✅ Centralized management
✅ Easy horizontal scaling

### Operational
✅ Hot reload without restart
✅ Configuration versioning
✅ Change history tracking
✅ Rollback capability

### Maintainability
✅ Single source of truth
✅ Clear audit trail
✅ Easy troubleshooting
✅ Configuration as data

## Migration from Old Setup

### Step 1: Backup Current Configuration

```bash
# Export current config
docker-compose exec postgres pg_dump -U postgres authenc > backup.sql
```

### Step 2: Update authenc.toml

```bash
# Replace with minimal bootstrap config
cp authenc.toml authenc.toml.backup
# Update with new minimal version
```

### Step 3: Load Configuration into Database

```bash
# Run initialization script
docker-compose exec postgres psql -U postgres -d authenc < scripts/init-db.sql
```

### Step 4: Migrate Secrets to Secreton

```bash
# Store secrets in Secreton
vault kv put secret/authenc/jwt_secret value="$JWT_SECRET"
vault kv put secret/authenc/ed25519_private_key value="$KEY"
```

### Step 5: Update Environment Variables

```bash
# Update .env with Secreton details
SECRETON_API_URL=http://secreton:8200
SECRETON_TOKEN=your_token
```

### Step 6: Restart Services

```bash
docker-compose restart authenc
```

## Troubleshooting

### Configuration Not Loading

```bash
# Check database connection
docker-compose exec postgres psql -U postgres -d authenc -c \
  "SELECT COUNT(*) FROM authenc.configuration;"

# Check logs
docker-compose logs authenc | grep -i config
```

### Secrets Not Available

```bash
# Verify Secreton connection
curl http://secreton:8200/health

# Check Secreton logs
docker-compose logs secreton | grep -i error
```

### Cache Issues

```bash
# Reload configuration
curl -X POST http://localhost:8088/admin/config/reload \
  -H "Authorization: Bearer $TOKEN"

# Clear cache manually
# (Happens automatically on update)
```

## Best Practices

1. **Use Categories**: Organize configuration by category
2. **Document Changes**: Always provide reason for changes
3. **Version Control**: Track configuration changes
4. **Audit Trail**: Review change history regularly
5. **Backup Secrets**: Backup Secreton regularly
6. **Test Changes**: Test configuration changes in staging first
7. **Monitor**: Monitor configuration API usage
8. **Rotate Secrets**: Rotate secrets regularly (quarterly minimum)

## Advanced Topics

### Configuration Versioning

```sql
-- Query specific version
SELECT * FROM authenc.configuration
WHERE key = 'captcha.default_difficulty'
AND version = 5;
```

### Configuration Templates

```sql
-- Store configuration templates
INSERT INTO authenc.configuration (key, value, category) VALUES
    ('template.production', '{"captcha_difficulty": 5, ...}'::jsonb, 'templates');
```

### Configuration Validation

```rust
// Validate configuration before applying
fn validate_config(key: &str, value: &Value) -> Result<()> {
    match key {
        "captcha.default_difficulty" => {
            let difficulty = value.as_i64().ok_or("Invalid difficulty")?;
            if difficulty < 1 || difficulty > 10 {
                return Err("Difficulty must be between 1 and 10");
            }
        }
        _ => {}
    }
    Ok(())
}
```

## Support

For issues or questions:
1. Check logs: `docker-compose logs authenc`
2. Review configuration: `SELECT * FROM authenc.configuration;`
3. Check audit trail: `SELECT * FROM authenc.configuration_history;`
4. Verify Secreton: `curl http://secreton:8200/health`
