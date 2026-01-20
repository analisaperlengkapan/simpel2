# Deploying Authenc with Hybrid Configuration

## Quick Start (5 Minutes)

### 1. Prepare Environment

```bash
cd /srv/proyek/simpelv2/infra/authenc

# Copy and edit environment file
cp .env.example .env
nano .env
```

**Essential settings**:
```bash
# Database
DB_PASSWORD=strong_password_here
DATABASE_URL=postgresql://postgres:strong_password_here@postgres:5432/authenc

# Secreton
SECRETON_API_URL=http://secreton:8200
SECRETON_TOKEN=your_secreton_token

# Secrets
JWT_SECRET=generate_strong_jwt_secret_here
```

### 2. Generate Signing Keys

```bash
# Generate cryptographic keys
cargo run --bin generate-signing-keys

# Copy output to .env
ED25519_PRIVATE_KEY_BASE64=...
ECDSA_P256_PRIVATE_KEY_BASE64=...
```

### 3. Store Secrets in Secreton

```bash
# Store JWT secret
vault kv put secret/authenc/jwt_secret \
  value="$(grep JWT_SECRET .env | cut -d= -f2)"

# Store signing keys
vault kv put secret/authenc/ed25519_private_key \
  value="$(grep ED25519_PRIVATE_KEY_BASE64 .env | cut -d= -f2)"

# Verify
vault kv get secret/authenc/jwt_secret
```

### 4. Start Services

```bash
# Development
docker-compose up -d

# Production
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Or use helper script
./docker-start.sh prod
```

### 5. Verify Deployment

```bash
# Check services
docker-compose ps

# Test API
curl http://localhost:8088/health

# View logs
docker-compose logs -f authenc

# Access Grafana
# URL: http://localhost:3000
# Username: admin
# Password: (from .env GRAFANA_PASSWORD)
```

## Configuration Management

### View Current Configuration

```bash
# Get all configuration
curl -X GET http://localhost:8088/admin/config \
  -H "Authorization: Bearer $TOKEN"

# Get by category
curl -X GET http://localhost:8088/admin/config/category/captcha \
  -H "Authorization: Bearer $TOKEN"

# Get specific setting
curl -X GET http://localhost:8088/admin/config/captcha.default_difficulty \
  -H "Authorization: Bearer $TOKEN"
```

### Update Configuration

```bash
# Update CAPTCHA difficulty
curl -X PUT http://localhost:8088/admin/config/captcha.default_difficulty \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "value": 5,
    "reason": "Increased for production"
  }'

# Update rate limit
curl -X PUT http://localhost:8088/admin/config/rate_limit.requests_per_minute \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "value": 200,
    "reason": "Increased capacity"
  }'
```

### Hot Reload Configuration

```bash
# Reload all configuration without restart
curl -X POST http://localhost:8088/admin/config/reload \
  -H "Authorization: Bearer $TOKEN"
```

### View Change History

```bash
# See all changes to a setting
curl -X GET http://localhost:8088/admin/config/captcha.default_difficulty/history \
  -H "Authorization: Bearer $TOKEN"
```

## Database Configuration

### Query Configuration

```bash
# Connect to database
docker-compose exec postgres psql -U postgres -d authenc

# View all configuration
SELECT key, value, category FROM authenc.configuration ORDER BY category, key;

# View specific category
SELECT key, value FROM authenc.configuration WHERE category = 'captcha';

# View change history
SELECT key, old_value, new_value, changed_by, created_at
FROM authenc.configuration_history
ORDER BY created_at DESC LIMIT 20;
```

### Update Configuration Directly

```bash
# Update via SQL (not recommended - use API instead)
UPDATE authenc.configuration
SET value = '5'::jsonb
WHERE key = 'captcha.default_difficulty';
```

## Architecture

```
┌─────────────────────────────────────────┐
│        Authenc Service                  │
├─────────────────────────────────────────┤
│                                         │
│  1. Load Bootstrap Config               │
│     (authenc.toml - minimal)            │
│           ↓                             │
│  2. Connect to Database                 │
│     (PostgreSQL)                        │
│           ↓                             │
│  3. Load Full Config                    │
│     (authenc.configuration table)       │
│           ↓                             │
│  4. Load Secrets                        │
│     (Secreton vault)                    │
│           ↓                             │
│  5. Start Service                       │
│     (Ready for requests)                │
│                                         │
└─────────────────────────────────────────┘
```

## Configuration Hierarchy

```
Environment Variables (highest)
    ↓
Secreton Secrets
    ↓
Database Configuration
    ↓
Bootstrap TOML
    ↓
Default Values (lowest)
```

## Security Checklist

- [ ] All passwords changed from defaults
- [ ] JWT_SECRET is strong (32+ chars, random)
- [ ] Signing keys generated and stored in Secreton
- [ ] Database credentials in .env (not committed)
- [ ] Secreton token in .env (not committed)
- [ ] TLS enabled in production
- [ ] CORS origins restricted
- [ ] Admin user created
- [ ] config-admin role assigned
- [ ] Audit logging enabled

## Troubleshooting

### Services Won't Start

```bash
# Check logs
docker-compose logs authenc

# Verify environment
cat .env | grep -E "^[A-Z_]+" | head -20

# Check port availability
netstat -tlnp | grep 8088
```

### Configuration Not Loading

```bash
# Check database connection
docker-compose exec postgres psql -U postgres -d authenc \
  -c "SELECT COUNT(*) FROM authenc.configuration;"

# Check if tables exist
docker-compose exec postgres psql -U postgres -d authenc \
  -c "\dt authenc.configuration*"
```

### Secrets Not Available

```bash
# Verify Secreton is running
curl http://secreton:8200/health

# Check Secreton logs
docker-compose logs secreton

# Verify token
echo $SECRETON_TOKEN
```

### API Returns 403 Forbidden

```bash
# Check user has admin role
docker-compose exec postgres psql -U postgres -d authenc \
  -c "SELECT u.username, r.name FROM authenc.users u
      JOIN authenc.user_roles ur ON u.id = ur.user_id
      JOIN authenc.roles r ON ur.role_id = r.id;"

# Assign config-admin role if needed
docker-compose exec postgres psql -U postgres -d authenc \
  -c "INSERT INTO authenc.user_roles (user_id, role_id)
      SELECT u.id, r.id FROM authenc.users u, authenc.roles r
      WHERE u.username = 'admin' AND r.name = 'config-admin';"
```

## Monitoring

### Check Service Health

```bash
# API health
curl http://localhost:8088/health

# Metrics
curl http://localhost:9090/metrics

# Prometheus
curl http://localhost:9091/api/v1/targets

# Grafana
# URL: http://localhost:3000
```

### View Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f authenc

# Last 100 lines
docker-compose logs --tail=100 authenc

# Follow with grep
docker-compose logs -f authenc | grep -i config
```

## Backup & Restore

### Backup Database

```bash
# Full backup
docker-compose exec postgres pg_dump -U postgres authenc | \
  gzip > backup-$(date +%Y%m%d-%H%M%S).sql.gz

# Backup configuration only
docker-compose exec postgres pg_dump -U postgres -t authenc.configuration authenc | \
  gzip > config-backup-$(date +%Y%m%d-%H%M%S).sql.gz
```

### Restore Database

```bash
# Restore full backup
gunzip < backup-20240101-120000.sql.gz | \
  docker-compose exec -T postgres psql -U postgres authenc

# Restore configuration only
gunzip < config-backup-20240101-120000.sql.gz | \
  docker-compose exec -T postgres psql -U postgres authenc
```

## Maintenance

### Regular Tasks

**Daily**:
- Monitor error rates
- Check disk space
- Review security logs

**Weekly**:
- Review configuration changes
- Check backup integrity
- Update dependencies

**Monthly**:
- Rotate secrets
- Review audit logs
- Capacity planning

**Quarterly**:
- Full security audit
- Disaster recovery drill
- Performance review

### Cleanup

```bash
# Remove old logs
docker-compose exec postgres psql -U postgres -d authenc \
  -c "DELETE FROM audit.logs WHERE created_at < NOW() - INTERVAL '90 days';"

# Remove old config history
docker-compose exec postgres psql -U postgres -d authenc \
  -c "DELETE FROM authenc.configuration_history
      WHERE created_at < NOW() - INTERVAL '1 year';"

# Vacuum database
docker-compose exec postgres psql -U postgres -d authenc \
  -c "VACUUM ANALYZE;"
```

## Advanced Topics

### Custom Configuration

```bash
# Add custom configuration
curl -X PUT http://localhost:8088/admin/config/custom.setting \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "value": "custom_value",
    "reason": "Custom setting for feature X"
  }'

# Query custom configuration
curl -X GET http://localhost:8088/admin/config/custom.setting \
  -H "Authorization: Bearer $TOKEN"
```

### Configuration Templates

```bash
# Store configuration template
curl -X PUT http://localhost:8088/admin/config/template.production \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "value": {
      "captcha_difficulty": 5,
      "rate_limit_rpm": 200,
      "enable_mfa": true
    },
    "reason": "Production template"
  }'
```

### Scheduled Configuration Changes

```bash
# Example: Increase rate limit during business hours
# (Implement via cron job or scheduler)

curl -X PUT http://localhost:8088/admin/config/rate_limit.requests_per_minute \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "value": 300,
    "reason": "Business hours increase (9 AM)"
  }'
```

## Documentation

- **Full Guide**: `HYBRID_CONFIG_GUIDE.md`
- **Implementation**: `IMPLEMENTATION_SUMMARY.md`
- **Docker Setup**: `DOCKER_SETUP.md`
- **Production**: `PRODUCTION_READINESS.md`
- **Quick Start**: `QUICKSTART.md`

## Support

For issues or questions:
1. Check logs: `docker-compose logs -f`
2. Review configuration: `SELECT * FROM authenc.configuration;`
3. Check audit trail: `SELECT * FROM authenc.configuration_history;`
4. Verify Secreton: `curl http://secreton:8200/health`
5. Review documentation

---

**Status**: ✅ Production Ready

**Last Updated**: November 28, 2024

**Version**: 1.0
