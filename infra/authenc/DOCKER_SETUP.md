# Authenc Docker Setup Guide

## Overview

This guide provides instructions for running the Authenc IAM service using Docker and Docker Compose. The setup includes PostgreSQL, Redis, Prometheus, and Grafana for a complete production-ready deployment.

## Quick Start

### 1. Prepare Environment

```bash
# Copy example environment file
cp .env.example .env

# Edit .env with your settings
nano .env
```

**Critical settings to update:**
- `DB_PASSWORD`: Strong PostgreSQL password
- `REDIS_PASSWORD`: Strong Redis password
- `JWT_SECRET`: Generate a strong JWT secret
- `GRAFANA_PASSWORD`: Grafana admin password
- `SECRETON_TOKEN`: Token for Secreton integration (if using)

### 2. Generate Signing Keys (Production)

For production, generate cryptographic signing keys:

```bash
# Generate Ed25519 signing key
cargo run --bin generate-signing-keys

# Copy the base64-encoded keys to your .env file
# ED25519_PRIVATE_KEY_BASE64=...
```

### 3. Start Services

#### Development Environment
```bash
docker-compose up -d
```

#### Production Environment
```bash
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### 4. Verify Services

```bash
# Check service status
docker-compose ps

# View logs
docker-compose logs -f authenc

# Test API health
curl http://localhost:8088/health

# Access Grafana
# URL: http://localhost:3000
# Username: admin
# Password: (from GRAFANA_PASSWORD in .env)
```

## Configuration

### Unified Configuration File

The `authenc.toml` file contains all service configuration:
- Server settings (host, ports, TLS)
- Database connection
- Security policies
- CAPTCHA system
- Rate limiting
- Features flags

Environment variables override TOML settings. Priority order:
1. Environment variables (highest priority)
2. authenc.toml
3. Default values (lowest priority)

### Environment-Specific Overrides

**Development:**
```bash
AUTHENC_ENV=development
RUST_LOG=debug
CAPTCHA_DEFAULT_DIFFICULTY=2
```

**Production:**
```bash
AUTHENC_ENV=production
RUST_LOG=warn
CAPTCHA_DEFAULT_DIFFICULTY=3
TLS_ENABLED=true
```

## Services

### PostgreSQL (Port 5432)
- Database: authenc
- User: postgres
- Volume: `authenc-postgres-data`
- Healthcheck: Enabled

### Redis (Port 6379)
- Cache, sessions, rate limiting
- Password protected
- Volume: `authenc-redis-data`
- Healthcheck: Enabled

### Authenc (Port 8088)
- HTTP API: 8088
- gRPC: 9088
- Metrics: 9090
- Healthcheck: Enabled

### Prometheus (Port 9091)
- Metrics collection
- Retention: 30 days (production)
- Volume: `authenc-prometheus-data`

### Grafana (Port 3000)
- Visualization dashboard
- Pre-configured data sources
- Volume: `authenc-grafana-data`

## Common Operations

### View Logs
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f authenc

# Last 100 lines
docker-compose logs --tail=100 authenc
```

### Database Management
```bash
# Access PostgreSQL
docker-compose exec postgres psql -U postgres -d authenc

# Backup database
docker-compose exec postgres pg_dump -U postgres authenc > backup.sql

# Restore database
docker-compose exec -T postgres psql -U postgres authenc < backup.sql
```

### Redis Management
```bash
# Access Redis CLI
docker-compose exec redis redis-cli -a ${REDIS_PASSWORD}

# Monitor Redis
docker-compose exec redis redis-cli -a ${REDIS_PASSWORD} MONITOR
```

### Restart Services
```bash
# Restart all
docker-compose restart

# Restart specific service
docker-compose restart authenc

# Full restart (stop and start)
docker-compose down && docker-compose up -d
```

### Stop Services
```bash
# Stop all services (keep volumes)
docker-compose stop

# Stop and remove containers (keep volumes)
docker-compose down

# Stop and remove everything (including volumes)
docker-compose down -v
```

## Production Deployment

### Security Checklist

- [ ] Change all default passwords in `.env`
- [ ] Generate strong JWT_SECRET
- [ ] Generate Ed25519 signing keys
- [ ] Enable TLS (TLS_ENABLED=true)
- [ ] Set AUTHENC_ENV=production
- [ ] Set RUST_LOG=warn
- [ ] Disable API docs (ENABLE_API_DOCS=false)
- [ ] Configure CORS_ALLOWED_ORIGINS
- [ ] Set up email (SMTP_*) for notifications
- [ ] Configure Secreton integration
- [ ] Set up log rotation and retention
- [ ] Configure backups for PostgreSQL

### Scaling

For high-traffic production:

1. **Database**: Increase `max_connections` in docker-compose.prod.yml
2. **Redis**: Increase `maxmemory` and configure eviction policy
3. **Authenc**: Deploy multiple instances with load balancer
4. **Monitoring**: Configure alert thresholds in Prometheus

### Monitoring

Access Grafana dashboard:
- URL: `http://localhost:3000`
- Default credentials: admin / (GRAFANA_PASSWORD)

Pre-configured dashboards:
- Authenc Service Metrics
- Database Performance
- Redis Usage
- System Resources

## Troubleshooting

### Service Won't Start
```bash
# Check logs
docker-compose logs authenc

# Verify environment variables
docker-compose config | grep AUTHENC_ENV

# Check port availability
netstat -tlnp | grep 8088
```

### Database Connection Failed
```bash
# Verify PostgreSQL is running
docker-compose ps postgres

# Check database logs
docker-compose logs postgres

# Test connection
docker-compose exec postgres psql -U postgres -d authenc -c "SELECT 1"
```

### Redis Connection Issues
```bash
# Verify Redis is running
docker-compose ps redis

# Test connection
docker-compose exec redis redis-cli -a ${REDIS_PASSWORD} PING

# Check Redis logs
docker-compose logs redis
```

### High Memory Usage
```bash
# Check container stats
docker stats

# Reduce Redis maxmemory in docker-compose.prod.yml
# Increase PostgreSQL connection pool if needed
```

## Cleanup

```bash
# Remove stopped containers
docker-compose down

# Remove unused volumes
docker volume prune

# Remove unused images
docker image prune

# Full cleanup (WARNING: removes all data)
docker-compose down -v
docker system prune -a
```

## Integration with Other Services

### Secreton (Secrets Management)
```bash
# Set Secreton URL
SECRETON_API_URL=http://secreton:8200
SECRETON_TOKEN=your_token_here
```

### External Monitoring
```bash
# Prometheus scrape config
- job_name: 'authenc'
  static_configs:
    - targets: ['authenc:9090']
```

## Performance Tuning

### Database
- Increase `max_connections` for high concurrency
- Enable connection pooling
- Configure query caching

### Redis
- Set appropriate `maxmemory` policy
- Enable persistence with `appendonly yes`
- Monitor memory usage

### Authenc
- Adjust `rate_limit_requests_per_minute`
- Configure CAPTCHA difficulty
- Enable caching for frequently accessed data

## Support

For issues or questions:
1. Check logs: `docker-compose logs -f`
2. Review configuration: `authenc.toml`
3. Verify environment: `docker-compose config`
4. Check service health: `curl http://localhost:8088/health`
